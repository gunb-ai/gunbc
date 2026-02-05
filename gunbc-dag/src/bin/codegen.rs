//! gunbc-codegen-dag main entry point.
//!
//! Upsert-style codegen prep: checks for generated CLI entrypoints,
//! runs the bootstrapper if missing, and writes a stamp file.

use gunbc_dag::codegen::build_codegen_graph;
use gunbc_dag::CODEGEN_STAMP_PATH;
use gunbc_exec::{execute_and_display, BoundaryMocks, ExecutionMode, TerminalProfile};
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
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
    let dag = match build_codegen_graph() {
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
            Value::Response(TransportResponse::Shell(ShellResponse::ok("")))
        };
        let missing_shell = || {
            Value::Response(TransportResponse::Shell(ShellResponse::failed(1, "missing")))
        };

        // Simulate missing codegen outputs so the codegen step runs.
        mocks.set_value("execute_codegen_exists", "response", missing_shell());

        // Codegen command execution
        mocks.set_value("execute_codegen", "response", ok_shell());
        mocks.set_value("execute_codegen", "skip", Value::Bool(false));

        // Stamp write
        mocks.set_value(
            "execute_stamp_write",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: CODEGEN_STAMP_PATH.to_string(),
                operation: FileOp::Write,
                success: true,
                content: None,
                exists: None,
                error: None,
            })),
        );
        mocks.set_value("execute_stamp_write", "skip", Value::Bool(false));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Print header
    println!("codegen");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    // Execute and display (progress or classic based on terminal)
    execute_and_display(&dag, mode, &profile, Some("prep_success"), None);
}

fn print_help() {
    println!("codegen - Upsert CLI entrypoints");
    println!();
    println!("USAGE:");
    println!("    gunbc-codegen-dag [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
    println!();
    println!("Checks for generated CLI entrypoints and runs gunbc-codegen if missing.");
}
