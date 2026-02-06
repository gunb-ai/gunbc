//! gunbc-makegen main entry point.
//!
//! Generates Makefile from tool registry.

#![deny(dead_code)]
use gunbc_dag::build_makegen_graph;
use gunbc_exec::{
    execute_and_display, execute_with_mode_and_inputs, BoundaryMocks, ExecutionMode,
    TerminalProfile,
};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut path = "Makefile".to_string();
    let mut dry_run = false;
    let mut check = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--path" => {
                i += 1;
                if i < args.len() {
                    path = args[i].clone();
                }
            }
            "-n" | "--dry-run" => dry_run = true,
            "--check" => check = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Build the graph
    let dag = match build_makegen_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building graph: {}", e);
            process::exit(1);
        }
    };

    // Set up entrypoint inputs
    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
        match port_name.0.as_str() {
            "path" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Str(path.clone()),
                );
            }
            "check_mode" => {
                input_mocks.set_input(
                    node_id.0.clone(),
                    port_name.0.clone(),
                    Value::Bool(check),
                );
            }
            _ => {}
        }
    }

    // Set up execution mode
    // In --check mode, we run Real (read transport must execute), but check_mode=true
    // forces compare_content to set skip=true on the write transport.
    // In --dry-run mode (without --check), mock all transports.
    let mode = if dry_run && !check {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "execute_read",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: path.clone(),
                operation: FileOp::Read,
                success: true,
                content: Some("<DRY-RUN>".to_string()),
                exists: None,
                error: None,
            })),
        );
        mocks.set_value(
            "execute_write",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: path.clone(),
                operation: FileOp::Write,
                success: true,
                content: Some("<DRY-RUN>".to_string()),
                exists: None,
                error: None,
            })),
        );
        mocks.set_value(
            "execute_write",
            "written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_write",
            "content",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value("execute_write", "skip", Value::Bool(false));
        mocks.set_value(
            "execute_write",
            "skip_reason",
            Value::Str(String::new()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    if check {
        // Check mode: bypass display, use execute_with_mode_and_inputs directly
        match execute_with_mode_and_inputs(&dag, mode, Some(&input_mocks)) {
            Ok(log) => {
                // Scan log for compare_content.fresh
                let fresh = log
                    .entries
                    .iter()
                    .find(|e| e.node_id == "compare_content")
                    .and_then(|e| e.outputs.get("fresh"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if fresh {
                    println!("makegen --check: 1 file up to date");
                } else {
                    eprintln!("makegen --check: drift detected");
                    eprintln!("  DRIFT  {}", path);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Detect terminal environment
        let profile = TerminalProfile::detect();

        // Print header
        println!("makegen");
        println!("  path: {}", path);
        println!(
            "  mode: {}",
            if dry_run { "dry-run" } else { "real" }
        );
        println!();

        // Execute and display (progress or classic based on terminal)
        execute_and_display(&dag, mode, &profile, None, Some(&input_mocks));
    }
}

fn print_help() {
    println!("makegen - Generate Makefile from tool registry");
    println!();
    println!("USAGE:");
    println!("    makegen [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --path <VAL>            Output Makefile path");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("        --check          Verify generated files match disk");
    println!("    -h, --help           Print this help");
}
