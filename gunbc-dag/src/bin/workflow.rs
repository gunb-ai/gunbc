//! gunbc-workflow planner entrypoint (WF5).

use std::collections::BTreeMap;
use std::path::PathBuf;

use gunbc_dag::{
    ci_workflow_spec, default_process_unit_registry, explain_plan, plan_workflow_with_mode,
    test_all_workflow_spec, tool_workflow_spec, CapabilityAction, DryRunMode, MissReason,
    PlannerInputs, TOOL_WORKFLOW_NAMES,
};
use serde::Serialize;

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

    let spec = match args.workflow.as_str() {
        "ci" => ci_workflow_spec(),
        "test-all" | "test_all" => test_all_workflow_spec(),
        name => tool_workflow_spec(name).map_err(|_| {
            let all_names: Vec<&str> = ["ci", "test-all"]
                .iter()
                .copied()
                .chain(TOOL_WORKFLOW_NAMES.iter().copied())
                .collect();
            format!(
                "unknown workflow '{}': expected one of {}",
                name,
                all_names.join(", ")
            )
        }),
    };
    let spec = match spec {
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
                    .ok_or_else(|| "--plan requires <ci|test-all>".to_string())?;
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

fn render_plan_output(
    workflow_name: &str,
    explain: &gunbc_dag::PlanExplain,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Text => render_plan_text(workflow_name, explain),
        OutputFormat::Json => render_plan_json(workflow_name, explain),
    }
}

fn render_plan_text(workflow_name: &str, explain: &gunbc_dag::PlanExplain) -> String {
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

    // WF22: Per-capability hit/miss/execute breakdown
    if !explain.capability_status.is_empty() {
        out.push_str("capabilities:\n");
        for status in explain.capability_status.values() {
            let action_str = match &status.action {
                CapabilityAction::CachedHit { previous_run } => {
                    format!("CachedHit from {previous_run}")
                }
                CapabilityAction::Execute { miss_reason } => {
                    format!("Execute ({})", format_miss_reason(miss_reason))
                }
            };
            let nodes_str = status
                .node_ids
                .iter()
                .map(|n| n.0.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "  - {}: {} [{}]\n",
                status.capability, action_str, nodes_str
            ));
        }
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
    capabilities: Vec<JsonCapabilityStatus>,
}

#[derive(Debug, Serialize)]
struct JsonExecuteNode {
    node_id: String,
    miss_reason: String,
}

#[derive(Debug, Serialize)]
struct JsonCapabilityStatus {
    capability: String,
    action: String,
    detail: String,
    node_ids: Vec<String>,
}

fn render_plan_json(workflow_name: &str, explain: &gunbc_dag::PlanExplain) -> String {
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

    let capabilities = explain
        .capability_status
        .values()
        .map(|status| {
            let (action, detail) = match &status.action {
                CapabilityAction::CachedHit { previous_run } => {
                    ("CachedHit".to_string(), previous_run.clone())
                }
                CapabilityAction::Execute { miss_reason } => {
                    ("Execute".to_string(), format_miss_reason(miss_reason))
                }
            };
            JsonCapabilityStatus {
                capability: status.capability.clone(),
                action,
                detail,
                node_ids: status.node_ids.iter().map(|n| n.0.clone()).collect(),
            }
        })
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&JsonPlanOutput {
        workflow: workflow_name.to_string(),
        execute_set,
        cache_hit_set,
        ready,
        blocked,
        critical_path,
        capabilities,
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

fn print_help() {
    let all_names: Vec<&str> = ["ci", "test-all"]
        .iter()
        .copied()
        .chain(TOOL_WORKFLOW_NAMES.iter().copied())
        .collect();
    println!("gunbc-workflow - workflow planner/explainability");
    println!();
    println!("USAGE:");
    println!(
        "  gunbc-workflow --plan <workflow> [--workspace-root <path>] [--dry-run <strict|lenient>]"
    );
    println!();
    println!("WORKFLOWS:");
    println!("  {}", all_names.join(", "));
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
        assert!(error.contains("--plan requires"));
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

    #[test]
    fn parse_args_accepts_tool_workflow_names() {
        for name in TOOL_WORKFLOW_NAMES {
            let args = parse_args(vec![
                "gunbc-workflow".to_string(),
                "--plan".to_string(),
                name.to_string(),
            ])
            .expect(&format!("parse should succeed for {name}"));
            assert_eq!(args.workflow, *name);
        }
    }
}
