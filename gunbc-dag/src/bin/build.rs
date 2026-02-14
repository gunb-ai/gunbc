//! gunbc-build main entry point.
//!
//! Local development build pipeline: build → (test + clippy) → summary.
//! Progress display is automatic based on terminal capabilities.

#![deny(dead_code)]
use gunbc_dag::build::build_build_graph;
use gunbc_exec::{execute_and_display, BoundaryMocks, ExecutionMode};
use std::io::IsTerminal;
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_lib_transport::preflight::ensure_lint_upsert;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--dry-run" => dry_run = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("error: unknown flag '{}'", other);
                process::exit(1);
            }
        }
        i += 1;
    }

    if let Err(err) = ensure_lint_upsert() {
        eprintln!("preflight failed: {}", err);
        process::exit(1);
    }

    // Build the graph
    let dag = match build_build_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || Value::Response(TransportResponse::Shell(ShellResponse::ok("")));

        // Build transport
        mocks.set_value("execute_build", "response", ok_shell());

        // Test transport
        mocks.set_value("execute_test", "response", ok_shell());
        mocks.set_value("execute_test", "skip", Value::Bool(false));

        // Clippy transport
        mocks.set_value("execute_clippy", "response", ok_shell());
        mocks.set_value("execute_clippy", "skip", Value::Bool(false));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Print header
    println!("build");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    // Execute and display (progress or classic based on terminal)
    let animated = std::io::stdout().is_terminal();
    execute_and_display(&dag, mode, animated, Some("overall_success"), None);
}

fn print_help() {
    println!("build - Build, test, and lint the project");
    println!();
    println!("USAGE:");
    println!("    gunbc-build [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
    println!();
    println!("Pipeline: build -> (test + clippy) -> summary");
    println!("Progress display is automatic based on terminal capabilities.");
}
