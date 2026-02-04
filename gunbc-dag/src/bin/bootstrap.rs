//! gunbc-bootstrap main entry point.
//!
//! Bootstrap tool for initializing gunbc projects.

use gunbc_dag::build_bootstrap_graph;
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
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
            "-h" | "--help" => { print_help(); return; }
            _ => {}
        }
        i += 1;
    }

    
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
        mocks.set_value("write_makefile", "written_path", Value::Str("<DRY-RUN>".to_string()));
        mocks.set_value("write_gitignore", "written_path", Value::Str("<DRY-RUN>".to_string()));

        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };
    
    // Print header
    println!("bootstrap");

    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    // Execute
    match execute_with_mode(&dag, mode) {
        Ok(log) => {
            for entry in &log.entries {
                let marker = if entry.was_intercepted { " [DRY-RUN]" } else { "" };
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
    println!("bootstrap - Generate Makefile and .gitignore");
    println!();
    println!("USAGE:");
    println!("    bootstrap [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
}
