//! Planner-based CI runner.
//!
//! Replaces the generated DAG-tool CI binary with a workflow planner execution.
//! The workflow spec comes from `dsl/workflows/ci.dag`, commands are derived
//! structurally from the workflow catalog, and the planner computes
//! materialization keys for all units.
//!
//! Usage:
//!   gunbc-ci              # run full CI pipeline
//!   gunbc-ci --dry-run    # print commands without executing
//!   gunbc-ci --plan       # explain the plan and exit

#![allow(clippy::disallowed_methods)]

use gunbc_app::{
    check_slo, ci_unit_commands, ci_workflow_spec, default_process_unit_registry,
    default_slo_budgets, execute_workflow_plan, explain_plan, plan_workflow,
    render_execution_report, PlannerInputs, SloResult,
};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let plan_only = args.iter().any(|a| a == "--plan");
    let help = args.iter().any(|a| a == "--help" || a == "-h");

    if help {
        print_help();
        return;
    }

    let spec = ci_workflow_spec().unwrap_or_else(|e| {
        eprintln!("error: failed to build CI workflow spec: {e}");
        process::exit(1);
    });

    let registry = default_process_unit_registry().unwrap_or_else(|e| {
        eprintln!("error: failed to build process unit registry: {e}");
        process::exit(1);
    });

    let root = env::current_dir().unwrap_or_else(|e| {
        eprintln!("error: cannot determine working directory: {e}");
        process::exit(1);
    });

    let plan = plan_workflow(&spec, &registry, &PlannerInputs::new(), &root).unwrap_or_else(|e| {
        eprintln!("error: workflow planning failed: {e}");
        process::exit(1);
    });

    let explain = explain_plan(&spec, &plan);

    if plan_only {
        println!("CI workflow plan ({} units):", plan.nodes.len());
        for node in &explain.execute_set {
            println!("  - {}", node.0);
        }
        println!();
        println!("critical-path:");
        for node in &explain.critical_path {
            println!("  - {}", node.0);
        }
        return;
    }

    let commands = ci_unit_commands().unwrap_or_else(|e| {
        eprintln!("error: failed to load CI unit commands: {e}");
        process::exit(1);
    });

    println!(
        "gunbc-ci: {} ({} units)",
        if dry_run { "dry-run" } else { "executing" },
        plan.nodes.len()
    );

    let summary = execute_workflow_plan(&spec, &plan, &commands, &root, dry_run);

    // Compute SLO result.
    let slo_result = default_slo_budgets()
        .iter()
        .find(|b| b.workflow_id == "ci")
        .map(|budget| check_slo(&summary, budget))
        .unwrap_or(SloResult::Pass);

    let report = render_execution_report(&summary, &explain, &slo_result);
    println!();
    println!("{report}");

    if !summary.success() {
        process::exit(1);
    }
}

fn print_help() {
    println!("gunbc-ci - CI pipeline runner (planner-based)");
    println!();
    println!("USAGE:");
    println!("    gunbc-ci [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Print commands without executing");
    println!("    --plan           Show execution plan and exit");
    println!("    -h, --help       Print this help");
}
