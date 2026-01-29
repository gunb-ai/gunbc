//! CLI for gunbc-buck2.

use gunbc_buck2::build_buck2_graph;
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::Value;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut cargo_toml_path = "Cargo.toml".to_string();
    let mut output_path = "BUCK".to_string();
    let mut dry_run = false;

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" | "-i" => {
                i += 1;
                if i < args.len() {
                    cargo_toml_path = args[i].clone();
                }
            }
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
            _ => {
                // Treat unknown args as input path
                if !args[i].starts_with('-') {
                    cargo_toml_path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    // Build the graph
    let dag = build_buck2_graph();

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value(
            "execute_transport",
            "written_path",
            Value::Str("<DRY-RUN: would write to BUCK>".to_string()),
        );
        mocks.set_value(
            "execute_transport",
            "content",
            Value::Str("<DRY-RUN: see generated content above>".to_string()),
        );
        mocks.set_value(
            "execute_transport",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::File(
                gunbc_ir::transport::FileResponse::written(&output_path),
            )),
        );
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    println!("gunbc-buck2");
    println!("  input: {}", cargo_toml_path);
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
                        Value::Str(s) if s.len() < 100 => println!("  {}: {}", port, s),
                        Value::Str(s) if port == "buck_content" || port == "content" => {
                            println!("  {}: ", port);
                            println!("--- START ---");
                            println!("{}", s);
                            println!("--- END ---");
                        }
                        Value::Str(s) => println!("  {}: {}...", port, &s[..50]),
                        Value::StrList(list) => println!("  {}: [{} items]", port, list.len()),
                        Value::MapStrStr(map) => println!("  {}: {{{} entries}}", port, map.len()),
                        Value::Json(_) => println!("  {}: <JSON>", port),
                        _ => println!("  {}: {:?}", port, value),
                    }
                }
            }

            // Print final result
            if let Some(entry) = log.get("execute_transport") {
                if let Some(Value::Str(path)) = entry.outputs.get("written_path") {
                    println!();
                    if entry.was_intercepted {
                        println!("Would have written to: {}", output_path);
                    } else {
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
    println!("gunbc-buck2 - Generate Buck2 BUCK files from Cargo.toml");
    println!();
    println!("USAGE:");
    println!("    gunbc-buck2 [OPTIONS] [CARGO_TOML_PATH]");
    println!();
    println!("OPTIONS:");
    println!("    -i, --input <PATH>   Cargo.toml path (default: Cargo.toml)");
    println!("    -o, --output <PATH>  Output BUCK path (default: BUCK)");
    println!("    -n, --dry-run        Don't actually write the file");
    println!("    -h, --help           Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    gunbc-buck2                        # Process Cargo.toml, write BUCK");
    println!("    gunbc-buck2 --dry-run              # Preview without writing");
    println!("    gunbc-buck2 -i project/Cargo.toml  # Specify input");
}
