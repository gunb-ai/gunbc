//! gunbc-ci main entry point.
//!
//! This is a handwritten main.rs (not generated) because the CI tool is the
//! bootstrap that runs codegen for all other tools. It cannot depend on
//! generated code because it needs to run BEFORE codegen.
//!
//! The CI pipeline uses the resource acquisition pattern internally - the
//! `prep` node checks if codegen is needed and runs it if so.

use gunbc_ci::build_ci_graph;
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::Value;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let dry_run = args.iter().any(|a| a == "-n" || a == "--dry-run");
    
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }
    
    // Build the CI graph
    let dag = match build_ci_graph() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error building CI graph: {}", e);
            process::exit(1);
        }
    };
    
    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("report", "overall_success", Value::Bool(true));
        mocks.set_value("report", "report", Value::Str("<DRY-RUN>".to_string()));
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };
    
    // Print header
    println!("gunbc-ci");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    // Execute the CI pipeline
    match execute_with_mode(&dag, mode) {
        Ok(log) => {
            for entry in &log.entries {
                let marker = if entry.was_intercepted { " [DRY-RUN]" } else { "" };
                println!("[{}]{}", entry.node_id, marker);
                
                for (port, value) in &entry.outputs {
                    print_value(port, value);
                }
            }
            
            // Check overall_success and exit with appropriate code
            for entry in &log.entries {
                if let Some(Value::Bool(false)) = entry.outputs.get("overall_success") {
                    process::exit(1);
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
                // Don't truncate stderr/stdout - we want to see full output
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
        Value::StrList(list) => println!("  {}: [{} items]", port, list.len()),
        Value::MapStrStr(map) => println!("  {}: {{{} entries}}", port, map.len()),
        Value::Json(_) => println!("  {}: <JSON>", port),
        _ => {}
    }
}

fn print_help() {
    println!("gunbc-ci - CI orchestration tool");
    println!();
    println!("USAGE:");
    println!("    gunbc-ci [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Don't perform actual I/O");
    println!("    -h, --help       Print this help");
    println!();
    println!("The CI pipeline runs: SetupDeps -> Prep -> Build -> Test/Lint -> Report");
    println!();
    println!("The Prep stage automatically runs codegen if generated files are missing.");
    println!("This is the resource acquisition (upsert) pattern - check -> create if needed.");
}
