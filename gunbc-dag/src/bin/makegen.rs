//! gunbc-makegen main entry point.
//!
//! Generates Makefile from tool registry.

use gunbc_dag::build_makegen_graph;
use gunbc_exec::{execute_with_mode_and_inputs, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{detect_entrypoints, Value};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut output_path = "Makefile".to_string();
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output-path" => {
                i += 1;
                if i < args.len() {
                    output_path = args[i].clone();
                }
            }
            "-n" | "--dry-run" => dry_run = true,
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

    let mut input_mocks = BoundaryMocks::new();
    let entrypoints = detect_entrypoints(&dag);
    for (node_id, port_name, _) in entrypoints.entrypoint_ports {
        if port_name.0 == "output_path" {
            input_mocks.set_input(
                node_id.0.clone(),
                port_name.0.clone(),
                Value::Str(output_path.clone()),
            );
        }
    }

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "execute_transport",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: output_path.clone(),
                operation: FileOp::Write,
                success: true,
                content: Some("<DRY-RUN>".to_string()),
                exists: None,
                error: None,
            })),
        );
        mocks.set_value(
            "execute_transport",
            "written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_transport",
            "content",
            Value::Str("<DRY-RUN>".to_string()),
        );

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // Print header
    println!("makegen");
    println!("  output_path: {}", output_path);

    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    // Execute
    match execute_with_mode_and_inputs(&dag, mode, Some(&input_mocks)) {
        Ok(log) => {
            for entry in &log.entries {
                let marker = if entry.was_intercepted {
                    " [DRY-RUN]"
                } else {
                    ""
                };
                println!("[{}]{}", entry.node_id, marker);

                for (port, value) in &entry.outputs {
                    print_value(port, value);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn print_value(port: &str, value: &Value) {
    match value {
        Value::Str(s) => {
            if port.ends_with("stderr") || port.ends_with("stdout") {
                if !s.is_empty() {
                    println!("  {}: {}", port, s);
                }
            } else if s.len() < 80 {
                println!("  {}: {}", port, s);
            } else {
                println!("  {}: {}...", port, &s[..60.min(s.len())]);
            }
        }
        Value::Int(i) => println!("  {}: {}", port, i),
        Value::Bool(b) => println!("  {}: {}", port, b),
        Value::List(list) => println!("  {}: [{} items]", port, list.len()),
        Value::Set(set) => println!("  {}: {{{} items}}", port, set.len()),
        Value::Map(map) => println!("  {}: {{{} entries}}", port, map.len()),
        Value::Json(_) => println!("  {}: <JSON>", port),
        _ => {}
    }
}

fn print_help() {
    println!("makegen - Generate Makefile from tool registry");
    println!();
    println!("USAGE:");
    println!("    makegen [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output-path <VAL>     Output Makefile path");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
}
