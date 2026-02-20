//! gunbc-workflow planner + executor entrypoint (WF5/WF6/WF7/WF9).
//!
//! Modes:
//! - `gunbc-workflow --plan <workflow>`: Plan only (WF5 explainability).
//! - `gunbc-workflow <workflow>`: Execute workflow via planner (WF6+).
//! - `gunbc-workflow <workflow> --dry-run`: Dry-run execution (no shell commands).

use std::collections::BTreeMap;
use std::path::PathBuf;

use gunbc_dag::{
    all_tool_workflow_names, check_slo, ci_workflow_spec, default_process_unit_registry,
    default_slo_budgets, execute_workflow_plan, explain_plan, plan_workflow_with_mode,
    render_execution_report, test_all_workflow_spec, tool_workflow_spec, workflow_unit_commands,
    CapabilityAction, DryRunMode, MissReason, PlannerInputs, SloBudget,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    /// Plan-only: show execute set and miss reasons (WF5).
    Plan,
    /// Execute: run the workflow via planner (WF6/WF7).
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    mode: Mode,
    workflow: String,
    workspace_root: PathBuf,
    dry_run_mode: DryRunMode,
    dry_run: bool,
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
            let mut all_names = vec!["ci", "test-all"];
            all_names.extend(all_tool_workflow_names());
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

    match args.mode {
        Mode::Plan => {
            let explain = explain_plan(&spec, &plan);
            println!(
                "{}",
                render_plan_output(&spec.id.0, &explain, args.output_format)
            );
        }
        Mode::Run => {
            let commands = match workflow_unit_commands(&args.workflow) {
                Ok(commands) => commands,
                Err(error) => {
                    eprintln!("error: {error}");
                    std::process::exit(2);
                }
            };

            println!("gunbc-workflow {}", args.workflow);
            println!("  mode: {}", if args.dry_run { "dry-run" } else { "real" });
            println!("  units: {}", plan.nodes.len());
            println!();

            let summary =
                execute_workflow_plan(&spec, &plan, &commands, &args.workspace_root, args.dry_run);
            let explain = explain_plan(&spec, &plan);

            // SLO check (WF9).
            let slo_budgets = default_slo_budgets();
            let slo_budget = slo_budgets
                .iter()
                .find(|b| b.workflow_id == args.workflow)
                .cloned()
                .unwrap_or(SloBudget {
                    workflow_id: args.workflow.clone(),
                    warm_noop_ms: 10_000,
                    total_max_ms: 600_000,
                });
            let slo_result = check_slo(&summary, &slo_budget);

            println!();
            println!(
                "{}",
                render_execution_report(&summary, &explain, &slo_result)
            );

            if !summary.success() {
                std::process::exit(1);
            }
            if !slo_result.is_pass() {
                // SLO failure is a warning in execution mode, not a hard exit.
                // CI gate can check exit code or parse report.
                eprintln!("warning: SLO budget exceeded (see report above)");
            }
        }
    }
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut workflow: Option<String> = None;
    let mut workspace_root: Option<PathBuf> = None;
    let mut dry_run_mode = DryRunMode::Strict;
    let mut output_format = OutputFormat::Text;
    let mut mode = Mode::Run;
    let mut dry_run = false;
    let mut positional: Option<String> = None;

    let mut i = 1usize;
    while i < argv.len() {
        let arg = argv[i].as_str();
        match arg {
            "-h" | "--help" => return Err("help requested".to_string()),
            "--plan" => {
                mode = Mode::Plan;
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--plan requires <workflow>".to_string())?;
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
                // In plan mode: controls strictness. In run mode: skips execution.
                dry_run = true;
                if i + 1 < argv.len() {
                    let next = argv[i + 1].as_str();
                    if next == "strict" || next == "lenient" {
                        i += 1;
                        dry_run_mode = parse_dry_run_mode(next)?;
                    }
                }
            }
            "-n" => {
                dry_run = true;
            }
            "--format" => {
                if mode == Mode::Plan {
                    i += 1;
                    let value = argv
                        .get(i)
                        .ok_or_else(|| "--format requires <text|json>".to_string())?;
                    output_format = parse_output_format(value)?;
                } else {
                    return Err("unknown argument '--format'".to_string());
                }
            }
            _ if arg.starts_with("--plan=") => {
                mode = Mode::Plan;
                workflow = Some(arg["--plan=".len()..].to_string());
            }
            _ if arg.starts_with("--workspace-root=") => {
                workspace_root = Some(PathBuf::from(arg["--workspace-root=".len()..].to_string()));
            }
            _ if arg.starts_with("--dry-run=") => {
                dry_run_mode = parse_dry_run_mode(&arg["--dry-run=".len()..])?;
                dry_run = true;
            }
            _ if arg.starts_with("--format=") => {
                if mode == Mode::Plan {
                    output_format = parse_output_format(&arg["--format=".len()..])?;
                } else {
                    return Err(format!("unknown argument '{arg}'"));
                }
            }
            _ if !arg.starts_with('-') => {
                positional = Some(arg.to_string());
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
        i += 1;
    }

    // Positional argument sets workflow for run mode.
    if workflow.is_none() {
        if let Some(pos) = positional {
            workflow = Some(pos);
        }
    }

    let workflow = workflow.ok_or_else(|| {
        if mode == Mode::Plan {
            "missing required --plan <workflow>".to_string()
        } else {
            "missing workflow name: use 'gunbc-workflow ci' or 'gunbc-workflow --plan ci'"
                .to_string()
        }
    })?;
    let workspace_root = workspace_root
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    Ok(Args {
        mode,
        workflow,
        workspace_root,
        dry_run_mode,
        dry_run,
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
    let mut all_names = vec!["ci", "test-all"];
    all_names.extend(all_tool_workflow_names());
    println!("gunbc-workflow - workflow planner and executor");
    println!();
    println!("USAGE:");
    println!("  gunbc-workflow <workflow> [--dry-run] [--workspace-root <path>]");
    println!(
        "  gunbc-workflow --plan <workflow> [--workspace-root <path>] [--dry-run <strict|lenient>]"
    );
    println!();
    println!("WORKFLOWS:");
    println!("  {}", all_names.join(", "));
    println!();
    println!("OPTIONS:");
    println!("  --plan <name>           Plan-only mode (show execute set + miss reasons)");
    println!("  --workspace-root <dir>  Workspace root for ledger/CAS paths");
    println!("  --dry-run               Dry-run: show commands without executing (run mode)");
    println!("  --dry-run <mode>        Dry-run strictness for plan mode (strict|lenient)");
    println!("  -n                      Alias for --dry-run");
    println!("  --format <text|json>    Plan output format (plan mode only)");
    println!("  -h, --help              Show this help");
    println!();
    println!("EXAMPLES:");
    println!("  gunbc-workflow ci                    # Run CI via planner");
    println!("  gunbc-workflow test-all              # Run test-all via planner");
    println!("  gunbc-workflow ci --dry-run          # Preview CI without execution");
    println!("  gunbc-workflow --plan ci             # Show plan only");
    println!("  gunbc-workflow --plan ci --format json  # Plan in JSON format");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn parse_args_supports_plan_mode() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "--plan".to_string(),
            "ci".to_string(),
        ])
        .expect("parse should succeed");
        assert_eq!(args.mode, Mode::Plan);
        assert_eq!(args.workflow, "ci");
    }

    #[test]
    fn parse_args_supports_run_mode_positional() {
        let args = parse_args(vec!["gunbc-workflow".to_string(), "ci".to_string()])
            .expect("parse should succeed");
        assert_eq!(args.mode, Mode::Run);
        assert_eq!(args.workflow, "ci");
    }

    #[test]
    fn parse_args_supports_test_all_positional() {
        let args = parse_args(vec!["gunbc-workflow".to_string(), "test-all".to_string()])
            .expect("parse should succeed");
        assert_eq!(args.mode, Mode::Run);
        assert_eq!(args.workflow, "test-all");
    }

    #[test]
    fn parse_args_supports_dry_run_flag() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "ci".to_string(),
            "--dry-run".to_string(),
        ])
        .expect("parse should succeed");
        assert!(args.dry_run);
    }

    #[test]
    fn parse_args_supports_n_flag() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "ci".to_string(),
            "-n".to_string(),
        ])
        .expect("parse should succeed");
        assert!(args.dry_run);
    }

    #[test]
    fn parse_args_supports_equals_syntax() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "--plan=ci".to_string(),
            "--workspace-root=/tmp/x".to_string(),
        ])
        .expect("parse should succeed");
        assert_eq!(args.mode, Mode::Plan);
        assert_eq!(args.workflow, "ci");
        assert_eq!(args.workspace_root, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn parse_args_rejects_missing_workflow() {
        let error = parse_args(vec!["gunbc-workflow".to_string()])
            .expect_err("missing workflow should fail");
        assert!(error.contains("missing workflow"));
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
        assert!(args.dry_run);
    }

    #[test]
    fn parse_args_sets_dry_run_for_equals_mode_syntax() {
        let args = parse_args(vec![
            "gunbc-workflow".to_string(),
            "ci".to_string(),
            "--dry-run=strict".to_string(),
        ])
        .expect("parse should succeed");
        assert_eq!(args.dry_run_mode, DryRunMode::Strict);
        assert!(args.dry_run);
    }

    #[test]
    fn parse_args_rejects_unknown_flags_in_run_mode() {
        let error = parse_args(vec![
            "gunbc-workflow".to_string(),
            "dag-viz".to_string(),
            "--repo-path".to_string(),
            ".".to_string(),
            "--format".to_string(),
            "svg".to_string(),
        ])
        .expect_err("run mode should reject unknown passthrough flags");
        assert!(error.contains("unknown argument"));
    }

    #[test]
    fn parse_args_rejects_unknown_flags_in_plan_mode() {
        let error = parse_args(vec![
            "gunbc-workflow".to_string(),
            "--plan".to_string(),
            "dag-viz".to_string(),
            "--repo-path".to_string(),
            ".".to_string(),
        ])
        .expect_err("plan mode should reject unknown flags");
        assert!(error.contains("unknown argument"));
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
            capability_status: BTreeMap::new(),
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
            capability_status: BTreeMap::new(),
        };
        let a = render_plan_output("ci", &explain, OutputFormat::Json);
        let b = render_plan_output("ci", &explain, OutputFormat::Json);
        assert_eq!(a, b);
    }

    #[test]
    fn parse_args_accepts_tool_workflow_names() {
        for name in all_tool_workflow_names() {
            let args = parse_args(vec![
                "gunbc-workflow".to_string(),
                "--plan".to_string(),
                name.to_string(),
            ])
            .unwrap_or_else(|_| panic!("parse should succeed for {name}"));
            assert_eq!(args.workflow, *name);
        }
    }
}
