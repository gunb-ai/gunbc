//! CLI for gunbc-viz - generates DAG visualization data.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{FileResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_viz::build_viz_graph;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_path = "viz-data.json".to_string();
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = args[i].clone();
                }
            }
            "-n" | "--dry-run" => {
                dry_run = true;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Build the DAG
    let dag = build_viz_graph();

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "execute_transport",
            "written_path",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value(
            "execute_transport",
            "response",
            Value::Response(TransportResponse::File(FileResponse::written(&output_path))),
        );
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    println!("gunbc-viz: DAG Visualization Generator");
    println!("  output: {}", output_path);
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    match execute_with_mode(&dag, mode) {
        Ok(log) => {
            for entry in &log.entries {
                let marker = if entry.was_intercepted {
                    " [DRY-RUN]"
                } else {
                    ""
                };
                println!("[{}]{}", entry.node_id, marker);

                // Print summary of outputs
                for (port, value) in &entry.outputs {
                    match value {
                        Value::Str(s) if s.len() < 80 => println!("  {}: {}", port, s),
                        Value::Str(s) => println!("  {}: {}...", port, &s[..60]),
                        Value::Int(i) => println!("  {}: {}", port, i),
                        Value::StrList(list) => println!("  {}: [{} items]", port, list.len()),
                        Value::Json(_) => println!("  {}: <JSON>", port),
                        _ => {}
                    }
                }
            }

            // Print final result
            if let Some(entry) = log.get("execute_transport") {
                if let Some(Value::Str(path)) = entry.outputs.get("written_path") {
                    println!();
                    if entry.was_intercepted {
                        println!("Would write to: {}", output_path);
                    } else {
                        println!("Written: {}", path);
                    }
                }
            }

            // Print collected graphs
            if let Some(entry) = log.get("collect_dags") {
                if let Some(Value::StrList(names)) = entry.outputs.get("graph_names") {
                    println!();
                    println!("Graphs collected:");
                    for name in names {
                        println!("  - {}", name);
                    }
                }
            }

            // Print serve instructions
            if !dry_run {
                println!();
                println!("To view the visualization:");
                println!("  make viz-serve");
                println!();
                println!("Or manually:");
                println!("  python3 -m http.server 8080 &");
                println!("  open http://localhost:8080/viz.html");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("gunbc-viz - Generate DAG visualization data");
    println!();
    println!("USAGE:");
    println!("    gunbc-viz [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output <PATH>  Output JSON path (default: viz-data.json)");
    println!("    -n, --dry-run        Don't actually write the file");
    println!("    -h, --help           Print this help");
    println!();
    println!("After running, open viz.html in a browser to view the visualization.");
}
