//! Planner-based test-all runner.
//!
//! Executes the test-all workflow via the planner. Same architecture as
//! ci_runner.rs but uses `test_all_workflow_spec` and `test_all_unit_commands`.
//!
//! Usage:
//!   gunbc-test-all              # run all tests (including ignored)
//!   gunbc-test-all --dry-run    # print commands without executing
//!   gunbc-test-all --plan       # explain the plan and exit

#![allow(clippy::disallowed_methods)]

use gunbc_app::{
    check_slo, default_process_unit_registry, default_slo_budgets, execute_workflow_plan,
    explain_plan, plan_workflow, render_execution_report, test_all_unit_commands,
    test_all_workflow_spec, PlannerInputs, SloResult,
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

    let spec = test_all_workflow_spec().unwrap_or_else(|e| {
        eprintln!("error: failed to build test-all workflow spec: {e}");
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
        println!("test-all workflow plan ({} units):", plan.nodes.len());
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

    let commands = test_all_unit_commands().unwrap_or_else(|e| {
        eprintln!("error: failed to load test-all unit commands: {e}");
        process::exit(1);
    });

    println!(
        "gunbc-test-all: {} ({} units)",
        if dry_run { "dry-run" } else { "executing" },
        plan.nodes.len()
    );

    let summary = execute_workflow_plan(&spec, &plan, &commands, &root, dry_run);

    let slo_result = default_slo_budgets()
        .iter()
        .find(|b| b.workflow_id == "test-all")
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
    println!("gunbc-test-all - Full test suite runner (planner-based)");
    println!();
    println!("USAGE:");
    println!("    gunbc-test-all [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Print commands without executing");
    println!("    --plan           Show execution plan and exit");
    println!("    -h, --help       Print this help");
}
