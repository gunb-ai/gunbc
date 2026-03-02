//! gunbc-workflow main entry point.
//!
//! Supports two modes:
//! - plan mode: emit deterministic planner explainability (`--plan`)
//! - execute mode: run workflow unit commands

#![deny(dead_code)]

use std::path::PathBuf;
use std::process;

use gunbc_dag::{
    ci_workflow_spec, default_process_unit_registry, execute_workflow_plan, explain_plan,
    plan_workflow, test_all_workflow_spec, tool_workflow_spec, workflow_unit_commands,
    BlockedReason, MissReason, PlannerInputs, PlannerWorkflowSpec,
};
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
struct CliArgs {
    workflow: Option<String>,
    plan_only: bool,
    format: OutputFormat,
    workspace_root: PathBuf,
    dry_run: bool,
    help: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            workflow: None,
            plan_only: false,
            format: OutputFormat::Text,
            workspace_root,
            dry_run: false,
            help: false,
        }
    }
}

fn main() {
    if let Err(message) = run() {
        eprintln!("error: {message}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let argv: Vec<String> = std::env::args().collect();
    let args = parse_args(&argv)?;
    if args.help {
        print_help();
        return Ok(());
    }

    let workflow_name = args
        .workflow
        .as_deref()
        .ok_or_else(|| "missing workflow name".to_string())?;

    let spec = workflow_spec_for_name(workflow_name)?;
    let registry = default_process_unit_registry();
    let plan = plan_workflow(
        &spec,
        &registry,
        &PlannerInputs::new(),
        &args.workspace_root,
    )
    .map_err(|error| format!("failed to plan workflow '{workflow_name}': {error}"))?;

    if args.plan_only {
        let explain = explain_plan(&spec, &plan);
        match args.format {
            OutputFormat::Json => {
                println!("{}", render_plan_json(&spec.id.0, &explain));
            }
            OutputFormat::Text => {
                print_plan_text(&spec.id.0, &explain);
            }
        }
        return Ok(());
    }

    let commands = workflow_unit_commands(workflow_name).map_err(|error| {
        format!("workflow '{workflow_name}' cannot execute with unit commands: {error}")
    })?;
    let summary =
        execute_workflow_plan(&spec, &plan, &commands, &args.workspace_root, args.dry_run);

    match args.format {
        OutputFormat::Json => println!("{}", render_execution_json(&summary)),
        OutputFormat::Text => print_execution_text(&summary),
    }

    if summary.success() {
        Ok(())
    } else {
        Err(format!(
            "workflow '{}' failed (failed={}, pending_approvals={}, skipped={})",
            summary.workflow_id, summary.failed, summary.pending_approvals, summary.skipped
        ))
    }
}

fn parse_args(argv: &[String]) -> Result<CliArgs, String> {
    let mut args = CliArgs::default();
    let mut idx = 1usize;
    while idx < argv.len() {
        let arg = &argv[idx];
        if arg == "-h" || arg == "--help" {
            args.help = true;
            idx += 1;
            continue;
        }
        if arg == "-n" || arg == "--dry-run" {
            args.dry_run = true;
            idx += 1;
            continue;
        }
        if arg == "--plan" {
            args.plan_only = true;
            if let Some(next) = argv.get(idx + 1) {
                if !next.starts_with('-') && args.workflow.is_none() {
                    args.workflow = Some(next.clone());
                    idx += 1;
                }
            }
            idx += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            args.format = parse_format(value)?;
            idx += 1;
            continue;
        }
        if arg == "--format" {
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--format requires a value (text|json)".to_string())?;
            args.format = parse_format(value)?;
            idx += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--workspace-root=") {
            args.workspace_root = PathBuf::from(value);
            idx += 1;
            continue;
        }
        if arg == "--workspace-root" {
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--workspace-root requires a path".to_string())?;
            args.workspace_root = PathBuf::from(value);
            idx += 2;
            continue;
        }
        if arg.starts_with('-') {
            return Err(format!("unknown flag '{arg}'"));
        }
        if args.workflow.is_some() {
            return Err(format!(
                "multiple workflow names provided ('{}' and '{}')",
                args.workflow.as_deref().unwrap_or_default(),
                arg
            ));
        }
        args.workflow = Some(arg.clone());
        idx += 1;
    }
    Ok(args)
}

fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(format!(
            "unsupported --format value '{other}' (expected text|json)"
        )),
    }
}

fn workflow_spec_for_name(name: &str) -> Result<PlannerWorkflowSpec, String> {
    let normalized = name.replace('_', "-");
    match normalized.as_str() {
        "ci" => ci_workflow_spec(),
        "test-all" => test_all_workflow_spec(),
        _ => tool_workflow_spec(&normalized),
    }
}

fn render_plan_json(workflow: &str, explain: &gunbc_dag::PlanExplain) -> Value {
    let execute_set = explain
        .execute_set
        .iter()
        .map(|node| {
            let miss_reason = explain
                .miss_reasons
                .get(node)
                .map(miss_reason_label)
                .unwrap_or_else(|| "miss:unknown".to_string());
            json!({
                "node_id": node.0.clone(),
                "miss_reason": miss_reason,
            })
        })
        .collect::<Vec<_>>();

    let cache_hit_set = explain
        .cache_hit_set
        .iter()
        .map(|node| node.0.clone())
        .collect::<Vec<_>>();

    let critical_path = explain
        .critical_path
        .iter()
        .map(|node| node.0.clone())
        .collect::<Vec<_>>();

    let ready = explain
        .ready
        .iter()
        .map(|node| node.0.clone())
        .collect::<Vec<_>>();

    let blocked = explain
        .blocked
        .iter()
        .map(|(node, reasons)| {
            let reason_values = reasons
                .iter()
                .map(|reason| Value::String(blocked_reason_label(reason)))
                .collect::<Vec<_>>();
            (node.0.clone(), Value::Array(reason_values))
        })
        .collect::<Map<String, Value>>();

    json!({
        "workflow": workflow,
        "execute_set": execute_set,
        "cache_hit_set": cache_hit_set,
        "critical_path": critical_path,
        "blocked": blocked,
        "ready": ready,
    })
}

fn render_execution_json(summary: &gunbc_dag::ExecutionSummary) -> Value {
    let results = summary
        .results
        .iter()
        .map(|result| {
            json!({
                "node_id": result.node_id.0.clone(),
                "success": result.success,
                "cached": result.cached,
                "pending_approval": result.pending_approval,
                "duration_ms": result.duration_ms,
                "miss_reason": result.miss_reason.as_ref().map(miss_reason_label),
            })
        })
        .collect::<Vec<_>>();

    json!({
        "workflow": summary.workflow_id,
        "success": summary.success(),
        "total_units": summary.total_units,
        "cache_hits": summary.cache_hits,
        "executed": summary.executed,
        "failed": summary.failed,
        "pending_approvals": summary.pending_approvals,
        "skipped": summary.skipped,
        "total_duration_ms": summary.total_duration_ms,
        "results": results,
    })
}

fn print_plan_text(workflow: &str, explain: &gunbc_dag::PlanExplain) {
    println!("workflow: {workflow}");
    println!("execute_set: {}", explain.execute_set.len());
    println!("cache_hit_set: {}", explain.cache_hit_set.len());
    println!("critical_path: {}", explain.critical_path.len());
    println!("ready: {}", explain.ready.len());
    println!("blocked: {}", explain.blocked.len());
}

fn print_execution_text(summary: &gunbc_dag::ExecutionSummary) {
    println!("workflow: {}", summary.workflow_id);
    println!("total_units: {}", summary.total_units);
    println!("executed: {}", summary.executed);
    println!("failed: {}", summary.failed);
    println!("pending_approvals: {}", summary.pending_approvals);
    println!("skipped: {}", summary.skipped);
    println!("total_duration_ms: {}", summary.total_duration_ms);
}

fn miss_reason_label(reason: &MissReason) -> String {
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
        MissReason::VolatileEffect { effect } => format!("miss:volatile-effect:{effect}"),
    }
}

fn blocked_reason_label(reason: &BlockedReason) -> String {
    match reason {
        BlockedReason::UncommittedPrerequisite { node_id } => {
            format!("blocked:uncommitted-prerequisite:{}", node_id.0)
        }
        BlockedReason::MissingRequiredDataInput { port } => {
            format!("blocked:missing-required-input:{}", port.0)
        }
    }
}

fn print_help() {
    println!("gunbc-workflow - deterministic workflow planning and execution");
    println!();
    println!("USAGE:");
    println!("    gunbc-workflow [OPTIONS] <workflow>");
    println!("    gunbc-workflow --plan <workflow> [--format=json|text]");
    println!();
    println!("OPTIONS:");
    println!("    --plan                 Emit planner explainability instead of executing");
    println!("    --format=FORMAT        Output format: text (default) or json");
    println!("    --workspace-root PATH  Workspace root for planner/executor context");
    println!("    -n, --dry-run          Print/record execution without running commands");
    println!("    -h, --help             Print this help");
}
