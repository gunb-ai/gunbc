//! CLI for gunbc-bootstrap.

use gunbc_bootstrap::build_bootstrap_graph;
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
            _ => {}
        }
        i += 1;
    }

    // Build the graph
    let dag = build_bootstrap_graph();

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("write_files", "files_written", Value::StrList(vec![]));
        mocks.set_value("write_files", "write_count", Value::Int(0));
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    println!("gunbc-bootstrap");
    println!("  mode: {}", if dry_run { "dry-run" } else { "generate" });
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

                // Print relevant outputs
                for (port, value) in &entry.outputs {
                    match value {
                        Value::Str(s) if port.ends_with("_content") => {
                            if dry_run {
                                println!("  {}:", port);
                                println!("--- START ---");
                                println!("{}", s);
                                println!("--- END ---");
                            } else {
                                println!("  {}: ({} bytes)", port, s.len());
                            }
                        }
                        Value::Str(s) if s.len() < 100 => println!("  {}: {}", port, s),
                        Value::StrList(list) if !list.is_empty() => {
                            println!("  {}: {}", port, list.join(", "));
                        }
                        Value::StrList(_) => println!("  {}: (none)", port),
                        Value::Int(n) => println!("  {}: {}", port, n),
                        _ => {}
                    }
                }
            }

            // Summary
            if !dry_run {
                if let Some(entry) = log.get("write_files") {
                    if let Some(Value::StrList(files)) = entry.outputs.get("files_written") {
                        if files.is_empty() {
                            println!("\nNo files changed.");
                        } else {
                            println!("\nFiles written: {}", files.join(", "));
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("gunbc-bootstrap - Generate all build infrastructure");
    println!();
    println!("USAGE:");
    println!("    gunbc-bootstrap [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Show what would be generated");
    println!("    -h, --help           Print this help message");
    println!();
    println!("GENERATES:");
    println!("    Makefile             Build and tool targets");
    println!("    .gitignore           Git ignore patterns");
}
