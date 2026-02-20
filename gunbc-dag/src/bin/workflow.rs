//! gunbc-workflow planner entrypoint (WF5).

use std::collections::BTreeMap;
use std::path::PathBuf;

use gunbc_dag::{
    bootstrap_workflow_spec, ci_workflow_spec, default_process_unit_registry, explain_plan,
    gist_diff_workflow_spec, gist_snapshot_workflow_spec, plan_workflow_with_mode,
    test_all_workflow_spec, DryRunMode, MissReason, PlanExplain, PlannerInputs,
    PlannerWorkflowSpec,
};
use serde::Serialize;

// ============================================================================
// Workflow Registry — single source of truth for all known workflows
// ============================================================================

/// One workflow entry. All CLI dispatch, help text, and error messages derive
/// from this table.
struct WorkflowEntry {
    /// Primary name (used in help and error messages).
    name: &'static str,
    /// Accepted aliases (underscore variants, abbreviations).
    aliases: &'static [&'static str],
    /// Short description for help output.
    description: &'static str,
    /// Builder function.
    build: fn() -> Result<PlannerWorkflowSpec, String>,
}

/// The single table of all registered workflows. Add a new workflow here and
/// it automatically appears in dispatch, help, and error messages.
const WORKFLOWS: &[WorkflowEntry] = &[
    WorkflowEntry {
        name: "ci",
        aliases: &[],
        description: "CI pipeline workflow",
        build: ci_workflow_spec,
    },
    WorkflowEntry {
        name: "test-all",
        aliases: &["test_all"],
        description: "Full test suite workflow",
        build: test_all_workflow_spec,
    },
    WorkflowEntry {
        name: "gist-snapshot",
        aliases: &["gist_snapshot"],
        description: "Gist snapshot workflow (WF14/WF15)",
        build: gist_snapshot_workflow_spec,
    },
    WorkflowEntry {
        name: "gist-diff",
        aliases: &["gist_diff"],
        description: "Gist diff workflow (WF14/WF15)",
        build: gist_diff_workflow_spec,
    },
    WorkflowEntry {
        name: "bootstrap",
        aliases: &[],
        description: "Bootstrap workflow (WF14/WF15)",
        build: bootstrap_workflow_spec,
    },
];

fn lookup_workflow(name: &str) -> Option<&'static WorkflowEntry> {
    WORKFLOWS
        .iter()
        .find(|entry| entry.name == name || entry.aliases.contains(&name))
}

fn workflow_names_for_display() -> String {
    WORKFLOWS
        .iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>()
        .join(", ")
}

// ============================================================================
// CLI
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    workflow: String,
    workspace_root: PathBuf,
    dry_run_mode: DryRunMode,
    output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

fn main() {
    let args = match parse_args(std::env::args().collect()) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("error: {error}");
            print_help();
            std::process::exit(1);
        }
    };

    let entry = match lookup_workflow(&args.workflow) {
        Some(entry) => entry,
        None => {
            eprintln!(
                "error: unknown workflow '{}': expected one of: {}",
                args.workflow,
                workflow_names_for_display()
            );
            std::process::exit(1);
        }
    };
    let spec = match (entry.build)() {
        Ok(spec) => spec,
        Err(error) => {
            eprintln!("error: failed to build workflow spec: {error}");
            std::process::exit(1);
        }
    };

    let registry = default_process_unit_registry();
    let inputs = PlannerInputs::new();
    let plan = match plan_workflow_with_mode(
        &spec,
        &registry,
        &inputs,
        &args.workspace_root,
        args.dry_run_mode,
    ) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("error: failed to compute workflow plan: {error}");
            std::process::exit(1);
        }
    };
    let explain = explain_plan(&spec, &plan);
    println!(
        "{}",
        render_plan_output(&spec.id.0, &explain, args.output_format)
    );
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut workflow: Option<String> = None;
    let mut workspace_root: Option<PathBuf> = None;
    let mut dry_run_mode = DryRunMode::Strict;
    let mut output_format = OutputFormat::Text;

    let mut i = 1usize;
    while i < argv.len() {
        let arg = argv[i].as_str();
        match arg {
            "-h" | "--help" => return Err("help requested".to_string()),
            "--plan" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--plan requires a workflow name".to_string())?;
                workflow = Some(value.clone());
            }
            "--workspace-root" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--workspace-root requires a path".to_string())?;
                workspace_root = Some(PathBuf::from(value));
            }
            "--dry-run" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--dry-run requires <strict|lenient>".to_string())?;
                dry_run_mode = parse_dry_run_mode(value)?;
            }
            "--format" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--format requires <text|json>".to_string())?;
                output_format = parse_output_format(value)?;
            }
            _ if arg.starts_with("--plan=") => {
                workflow = Some(arg["--plan=".len()..].to_string());
            }
            _ if arg.starts_with("--workspace-root=") => {
                workspace_root = Some(PathBuf::from(arg["--workspace-root=".len()..].to_string()));
            }
            _ if arg.starts_with("--dry-run=") => {
                dry_run_mode = parse_dry_run_mode(&arg["--dry-run=".len()..])?;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(&arg["--format=".len()..])?;
            }
            other => {
                return Err(format!("unknown argument '{other}'"));
            }
        }
        i += 1;
    }

    let workflow = workflow.ok_or_else(|| "missing required --plan <workflow>".to_string())?;
    let workspace_root = workspace_root
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    Ok(Args {
        workflow,
        workspace_root,
        dry_run_mode,
        output_format,
    })
}

fn parse_dry_run_mode(raw: &str) -> Result<DryRunMode, String> {
    match raw.to_ascii_lowercase().as_str() {
        "strict" => Ok(DryRunMode::Strict),
        "lenient" => Ok(DryRunMode::Lenient),
        other => Err(format!(
            "unknown --dry-run mode '{}': expected strict or lenient",
            other
        )),
    }
}

fn parse_output_format(raw: &str) -> Result<OutputFormat, String> {
    match raw.to_ascii_lowercase().as_str() {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(format!(
            "unknown --format value '{}': expected text or json",
            other
        )),
    }
}

// ============================================================================
// Plan rendering
// ============================================================================

fn render_plan_output(workflow_name: &str, explain: &PlanExplain, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => render_plan_text(workflow_name, explain),
        OutputFormat::Json => render_plan_json(workflow_name, explain),
    }
}

fn render_plan_text(workflow_name: &str, explain: &PlanExplain) -> String {
    let mut out = String::new();
    out.push_str(&format!("workflow: {workflow_name}\n"));

    out.push_str("execute-set:\n");
    for node in &explain.execute_set {
        let reason = explain
            .miss_reasons
            .get(node)
            .map(format_miss_reason)
            .unwrap_or_else(|| "unknown".to_string());
        out.push_str(&format!("  - {} ({})\n", node.0, reason));
    }

    out.push_str("cache-hit-set:\n");
    for node in &explain.cache_hit_set {
        out.push_str(&format!("  - {}\n", node.0));
    }

    out.push_str("ready:\n");
    for node in &explain.ready {
        out.push_str(&format!("  - {}\n", node.0));
    }

    out.push_str("blocked:\n");
    for (node, reasons) in &explain.blocked {
        let reason = reasons
            .iter()
            .map(|reason| match reason {
                gunbc_dag::BlockedReason::UncommittedPrerequisite { node_id } => {
                    format!("waiting-for-commit:{}", node_id.0)
                }
                gunbc_dag::BlockedReason::MissingRequiredDataInput { port } => {
                    format!("missing-input:{}", port.0)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("  - {} ({})\n", node.0, reason));
    }

    out.push_str("critical-path:\n");
    for node in &explain.critical_path {
        out.push_str(&format!("  - {}\n", node.0));
    }
    out
}

#[derive(Debug, Serialize)]
struct JsonPlanOutput {
    workflow: String,
    execute_set: Vec<JsonExecuteNode>,
    cache_hit_set: Vec<String>,
    ready: Vec<String>,
    blocked: BTreeMap<String, Vec<String>>,
    critical_path: Vec<String>,
}

#[derive(Debug, Serialize)]
struct JsonExecuteNode {
    node_id: String,
    miss_reason: String,
}

fn render_plan_json(workflow_name: &str, explain: &PlanExplain) -> String {
    let execute_set = explain
        .execute_set
        .iter()
        .map(|node| JsonExecuteNode {
            node_id: node.0.clone(),
            miss_reason: explain
                .miss_reasons
                .get(node)
                .map(format_miss_reason)
                .unwrap_or_else(|| "unknown".to_string()),
        })
        .collect::<Vec<_>>();

    let cache_hit_set = explain
        .cache_hit_set
        .iter()
        .map(|node| node.0.clone())
        .collect::<Vec<_>>();
    let ready = explain
        .ready
        .iter()
        .map(|node| node.0.clone())
        .collect::<Vec<_>>();
    let critical_path = explain
        .critical_path
        .iter()
        .map(|node| node.0.clone())
        .collect::<Vec<_>>();

    let blocked = explain
        .blocked
        .iter()
        .map(|(node, reasons)| {
            let formatted = reasons
                .iter()
                .map(|reason| match reason {
                    gunbc_dag::BlockedReason::UncommittedPrerequisite { node_id } => {
                        format!("waiting-for-commit:{}", node_id.0)
                    }
                    gunbc_dag::BlockedReason::MissingRequiredDataInput { port } => {
                        format!("missing-input:{}", port.0)
                    }
                })
                .collect::<Vec<_>>();
            (node.0.clone(), formatted)
        })
        .collect::<BTreeMap<_, _>>();

    serde_json::to_string_pretty(&JsonPlanOutput {
        workflow: workflow_name.to_string(),
        execute_set,
        cache_hit_set,
        ready,
        blocked,
        critical_path,
    })
    .expect("json plan output should always be serializable")
}

fn format_miss_reason(reason: &MissReason) -> String {
    match reason {
        MissReason::NoPriorRun => "miss:no-prior-run".to_string(),
        MissReason::InputChanged { port, .. } => format!("miss:input-changed:{}", port.0),
        MissReason::UpstreamKeyChanged { port, .. } => {
            format!("miss:upstream-key-changed:{}", port.0)
        }
        MissReason::OpVersionChanged { .. } => "miss:op-version-changed".to_string(),
        MissReason::PolicyVersionChanged { .. } => "miss:policy-version-changed".to_string(),
        MissReason::OutputMissing { port } => format!("miss:output-missing:{}", port.0),
        MissReason::OutputTampered { port, .. } => format!("miss:output-tampered:{}", port.0),
        MissReason::VolatileEffect { effect } => format!("miss:volatile-effect:{}", effect),
    }
}

// ============================================================================
// Help — derived entirely from WORKFLOWS table
// ============================================================================

fn print_help() {
    println!("gunbc-workflow - workflow planner/explainability");
    println!();
    println!("USAGE:");
    println!(
        "  gunbc-workflow --plan <workflow> [--workspace-root <path>] [--dry-run <strict|lenient>]"
    );
    println!();
    println!("WORKFLOWS:");
    let max_name = WORKFLOWS.iter().map(|e| e.name.len()).max().unwrap_or(0);
    for entry in WORKFLOWS {
        println!("  {:<width$}  {}", entry.name, entry.description, width = max_name);
    }
    println!();
    println!("FLAGS:");
    println!("  --plan <name>           Workflow to plan");
    println!("  --workspace-root <dir>  Workspace root for ledger/CAS paths");
    println!("  --dry-run <mode>        Dry-run strictness (default: strict)");
    println!("  --format <text|json>    Output format (default: text)");
    println!("  -h, --help              Show this help");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn parse_args_supports_equals_syntax() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "--plan=ci".to_string(),
            "--workspace-root=/tmp/x".to_string(),
        ])
        .expect("parse should succeed");
        assert_eq!(args.workflow, "ci");
        assert_eq!(args.workspace_root, PathBuf::from("/tmp/x"));
        assert_eq!(args.dry_run_mode, DryRunMode::Strict);
        assert_eq!(args.output_format, OutputFormat::Text);
    }

    #[test]
    fn parse_args_rejects_missing_plan_value() {
        let error = parse_args(vec!["gunbc-workflow".to_string(), "--plan".to_string()])
            .expect_err("missing plan value should fail");
        assert!(error.contains("--plan requires"), "got: {error}");
    }

    #[test]
    fn parse_args_supports_dry_run_mode() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "--plan".to_string(),
            "ci".to_string(),
            "--dry-run".to_string(),
            "lenient".to_string(),
        ])
        .expect("parse should succeed");
        assert_eq!(args.dry_run_mode, DryRunMode::Lenient);
    }

    #[test]
    fn parse_args_supports_json_output_mode() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "--plan".to_string(),
            "ci".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed");
        assert_eq!(args.output_format, OutputFormat::Json);
    }

    #[test]
    fn render_output_is_deterministic_for_same_explain_data() {
        let explain = gunbc_dag::PlanExplain {
            execute_set: vec![gunbc_ir::NodeId::from("ci.codegen")],
            cache_hit_set: vec![],
            miss_reasons: BTreeMap::from([(
                gunbc_ir::NodeId::from("ci.codegen"),
                MissReason::NoPriorRun,
            )]),
            blocked: BTreeMap::new(),
            ready: vec![gunbc_ir::NodeId::from("ci.lint_upsert")],
            critical_path: vec![gunbc_ir::NodeId::from("ci.lint_upsert")],
        };
        let a = render_plan_output("ci", &explain, OutputFormat::Text);
        let b = render_plan_output("ci", &explain, OutputFormat::Text);
        assert_eq!(a, b);
    }

    #[test]
    fn render_json_output_is_deterministic_for_same_explain_data() {
        let explain = gunbc_dag::PlanExplain {
            execute_set: vec![gunbc_ir::NodeId::from("ci.codegen")],
            cache_hit_set: vec![],
            miss_reasons: BTreeMap::from([(
                gunbc_ir::NodeId::from("ci.codegen"),
                MissReason::NoPriorRun,
            )]),
            blocked: BTreeMap::new(),
            ready: vec![gunbc_ir::NodeId::from("ci.lint_upsert")],
            critical_path: vec![gunbc_ir::NodeId::from("ci.lint_upsert")],
        };
        let a = render_plan_output("ci", &explain, OutputFormat::Json);
        let b = render_plan_output("ci", &explain, OutputFormat::Json);
        assert_eq!(a, b);
    }

    // Registry-driven tests

    #[test]
    fn all_workflow_entries_have_valid_builders() {
        for entry in WORKFLOWS {
            let spec = (entry.build)();
            assert!(
                spec.is_ok(),
                "workflow '{}' builder failed: {}",
                entry.name,
                spec.unwrap_err()
            );
        }
    }

    #[test]
    fn lookup_resolves_primary_names() {
        for entry in WORKFLOWS {
            assert!(
                lookup_workflow(entry.name).is_some(),
                "primary name '{}' should resolve",
                entry.name
            );
        }
    }

    #[test]
    fn lookup_resolves_aliases() {
        assert!(lookup_workflow("test_all").is_some());
        assert!(lookup_workflow("gist_snapshot").is_some());
        assert!(lookup_workflow("gist_diff").is_some());
    }

    #[test]
    fn lookup_rejects_unknown() {
        assert!(lookup_workflow("nonexistent").is_none());
    }
}
