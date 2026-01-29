//! CLI for gunbc-makegen.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::Value;
use gunbc_makegen::build_makegen_graph;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_path = "Makefile".to_string();
    let mut dry_run = false;

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output_path = args[i].clone();
                }
            }
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
    let dag = build_makegen_graph();

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "write_makefile",
            "written_path",
            Value::Str("<DRY-RUN: would write>".to_string()),
        );
        mocks.set_value(
            "write_makefile",
            "content",
            Value::Str("<DRY-RUN>".to_string()),
        );
        mocks.set_value("write_makefile", "changed", Value::Bool(true));
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    println!("gunbc-makegen");
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

                // Print relevant outputs
                for (port, value) in &entry.outputs {
                    match value {
                        Value::Str(s) if s.len() < 100 => println!("  {}: {}", port, s),
                        Value::Str(s) if port == "makefile_content" || port == "content" => {
                            if !entry.was_intercepted {
                                println!("  {}: ", port);
                                println!("--- START ---");
                                println!("{}", s);
                                println!("--- END ---");
                            }
                        }
                        Value::Str(s) => println!("  {}: {}...", port, &s[..50]),
                        Value::StrList(list) => println!("  {}: [{} items]", port, list.len()),
                        Value::Int(n) => println!("  {}: {}", port, n),
                        Value::Bool(b) => println!("  {}: {}", port, b),
                        _ => {}
                    }
                }
            }

            // Show rendered content in dry-run mode
            if dry_run {
                if let Some(entry) = log.get("render_makefile") {
                    if let Some(Value::Str(content)) = entry.outputs.get("makefile_content") {
                        println!();
                        println!("Generated Makefile:");
                        println!("--- START ---");
                        println!("{}", content);
                        println!("--- END ---");
                    }
                }
            }

            // Final status
            if let Some(entry) = log.get("write_makefile") {
                println!();
                if entry.was_intercepted {
                    println!("Would have written to: {}", output_path);
                } else {
                    if let Some(Value::Str(path)) = entry.outputs.get("written_path") {
                        println!("Written to: {}", path);
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
    println!("gunbc-makegen - Generate Makefile from tool registry");
    println!();
    println!("USAGE:");
    println!("    gunbc-makegen [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output <PATH>  Output Makefile path (default: Makefile)");
    println!("    -n, --dry-run        Don't actually write the file");
    println!("    -h, --help           Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    gunbc-makegen                    # Generate Makefile");
    println!("    gunbc-makegen --dry-run          # Preview without writing");
    println!("    gunbc-makegen -o build/Makefile  # Custom output path");
}
