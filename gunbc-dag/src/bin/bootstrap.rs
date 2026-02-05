//! gunbc-bootstrap main entry point.
//!
//! Bootstrap tool for initializing gunbc projects.
//! Progress display is automatic based on terminal capabilities.

use gunbc_dag::build_bootstrap_graph;
use gunbc_exec::{execute_and_display, BoundaryMocks, ExecutionMode, TerminalProfile};
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::Value;
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
            _ => {}
        }
        i += 1;
    }

    // Detect terminal environment
    let profile = TerminalProfile::detect();

    // Build the graph
    let dag = match build_bootstrap_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        let ok_shell = || {
            Value::Response(TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            }))
        };

        // Scan workspace: returns a mock directory listing
        mocks.set_value(
            "execute_scan_workspace",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse {
                exit_code: 0,
                stdout: "crates/example\n".to_string(),
                stderr: String::new(),
            })),
        );

        // Makefile transport executor
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_response",
            ok_shell(),
        );
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_makefile_transport",
            "makefile_content",
            Value::Str("<DRY-RUN>".to_string()),
        );

        // Gitignore transport executor
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_response",
            ok_shell(),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_gitignore_transport",
            "gitignore_content",
            Value::Str("<DRY-RUN>".to_string()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Print header
    println!("bootstrap");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    // Execute and display (progress or classic based on terminal)
    execute_and_display(&dag, mode, &profile, None, None);
}

fn print_help() {
    println!("bootstrap - Generate Makefile and .gitignore");
    println!();
    println!("USAGE:");
    println!("    bootstrap [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
