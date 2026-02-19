//! gunbc-workflow planner entrypoint (WF5).

use std::path::PathBuf;

use gunbc_dag::{
    ci_workflow_spec, default_process_unit_registry, explain_plan, plan_workflow_with_mode,
    test_all_workflow_spec, DryRunMode, MissReason, PlannerInputs,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    workflow: String,
    workspace_root: PathBuf,
    dry_run_mode: DryRunMode,
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
        other => Err(format!(
            "unknown workflow '{}': expected ci or test-all",
            other
        )),
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
    println!("{}", render_plan_output(&spec.id.0, &explain));
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut workflow: Option<String> = None;
    let mut workspace_root: Option<PathBuf> = None;
    let mut dry_run_mode = DryRunMode::Strict;

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
            _ if arg.starts_with("--plan=") => {
                workflow = Some(arg["--plan=".len()..].to_string());
            }
            _ if arg.starts_with("--workspace-root=") => {
                workspace_root = Some(PathBuf::from(arg["--workspace-root=".len()..].to_string()));
            }
            _ if arg.starts_with("--dry-run=") => {
                dry_run_mode = parse_dry_run_mode(&arg["--dry-run=".len()..])?;
            }
            other => {
                return Err(format!("unknown argument '{other}'"));
            }
        }
        i += 1;
    }

    let workflow = workflow.ok_or_else(|| "missing required --plan <ci|test-all>".to_string())?;
    let workspace_root = workspace_root
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    Ok(Args {
        workflow,
        workspace_root,
        dry_run_mode,
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

fn render_plan_output(workflow_name: &str, explain: &gunbc_dag::PlanExplain) -> String {
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
    println!("gunbc-workflow - workflow planner/explainability");
    println!();
    println!("USAGE:");
    println!(
        "  gunbc-workflow --plan <ci|test-all> [--workspace-root <path>] [--dry-run <strict|lenient>]"
    );
    println!();
    println!("FLAGS:");
    println!("  --plan <name>           Workflow to plan (ci or test-all)");
    println!("  --workspace-root <dir>  Workspace root for ledger/CAS paths");
    println!("  --dry-run <mode>        Dry-run strictness (default: strict)");
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
        let a = render_plan_output("ci", &explain);
        let b = render_plan_output("ci", &explain);
        assert_eq!(a, b);
    }
}
