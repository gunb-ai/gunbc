//! CLI for gunbc-ci.

use gunbc_ci::build_ci_graph;
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::Value;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut dry_run = false;

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" | "-n" => {
                dry_run = true;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            "run" => {
                // Default command
            }
            _ => {}
        }
        i += 1;
    }

    // Build the graph
    let dag = build_ci_graph();

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("report", "overall_success", Value::Bool(true));
        mocks.set_value(
            "report",
            "report",
            Value::Str("<DRY-RUN: CI report would be generated>".to_string()),
        );
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    println!("gunbc-ci");
    println!("  mode: {}", if dry_run { "dry-run" } else { "run" });
    println!();

    match execute_with_mode(&dag, mode) {
        Ok(log) => {
            let mut overall_success = true;

            for entry in &log.entries {
                let marker = if entry.was_intercepted {
                    " [DRY-RUN]"
                } else {
                    ""
                };
                
                // Determine status
                let status = if entry.node_id == "report" {
                    match entry.outputs.get("overall_success") {
                        Some(Value::Bool(b)) => {
                            overall_success = *b;
                            if *b { "SUCCESS" } else { "FAILURE" }
                        }
                        _ => "UNKNOWN",
                    }
                } else {
                    let success_key = format!("{}_success", entry.node_id);
                    match entry.outputs.get(&success_key) {
                        Some(Value::Bool(true)) => "PASS",
                        Some(Value::Bool(false)) => {
                            overall_success = false;
                            "FAIL"
                        }
                        _ => match entry.outputs.get("deps_checked") {
                            Some(Value::Bool(true)) => "OK",
                            _ => "...",
                        },
                    }
                };

                println!("[{}]{} - {}", entry.node_id, marker, status);

                // Print report if present
                if entry.node_id == "report" {
                    if let Some(Value::Str(report)) = entry.outputs.get("report") {
                        println!("{}", report);
                    }
                }
            }

            // Exit with appropriate code
            if !overall_success && !dry_run {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("gunbc-ci - CI orchestration binary");
    println!();
    println!("USAGE:");
    println!("    gunbc-ci [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    run                  Run CI pipeline (default)");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Show what would be run without executing");
    println!("    -h, --help           Print this help message");
    println!();
    println!("CI PIPELINE:");
    println!("    1. Setup dependencies (via deps.toml)");
    println!("    2. Build (cargo build)");
    println!("    3. Test (cargo test)");
    println!("    4. Lint (cargo clippy)");
    println!("    5. Report results");
}
