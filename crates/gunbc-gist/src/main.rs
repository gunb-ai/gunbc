//! CLI for gunbc-gist.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::build_gist_graph;
use gunbc_ir::Value;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut repo_path = ".".to_string();
    let mut extensions: Vec<String> = vec![];
    let mut public = false;
    let mut dry_run = false;

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" | "-r" => {
                i += 1;
                if i < args.len() {
                    repo_path = args[i].clone();
                }
            }
            "--ext" | "-e" => {
                i += 1;
                if i < args.len() {
                    extensions.push(args[i].clone());
                }
            }
            "--public" | "-p" => {
                public = true;
            }
            "--dry-run" | "-n" => {
                dry_run = true;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {
                // Treat unknown args as repo path
                if !args[i].starts_with('-') {
                    repo_path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    // Build the graph
    let dag = build_gist_graph(extensions, public);

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        // Mock the transport boundary
        mocks.set_value(
            "execute_transport",
            "url",
            Value::Str("https://gist.github.com/dry-run-mock".to_string()),
        );
        mocks.set_value(
            "execute_transport",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Shell(
                gunbc_ir::transport::ShellResponse {
                    exit_code: 0,
                    stdout: "https://gist.github.com/dry-run-mock\n".to_string(),
                    stderr: String::new(),
                },
            )),
        );
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    // For now, we need to inject the repo_path into the first node
    // This is a limitation that we'll address with seed inputs
    // For now, we execute manually with injected inputs
    
    println!("gunbc-gist");
    println!("  repo: {}", repo_path);
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    match execute_with_mode(&dag, mode) {
        Ok(log) => {
            for entry in &log.entries {
                let marker = if entry.was_intercepted { " [DRY-RUN]" } else { "" };
                println!("[{}]{}", entry.node_id, marker);
                
                // Print summary of outputs
                for (port, value) in &entry.outputs {
                    match value {
                        Value::Str(s) if s.len() < 100 => println!("  {}: {}", port, s),
                        Value::Str(s) => println!("  {}: {}...", port, &s[..50]),
                        Value::StrList(list) => println!("  {}: [{} items]", port, list.len()),
                        Value::MapStrStr(map) => println!("  {}: {{{} entries}}", port, map.len()),
                        _ => println!("  {}: {:?}", port, value),
                    }
                }
            }

            // Print final URL if available
            if let Some(entry) = log.get("execute_transport") {
                if let Some(Value::Str(url)) = entry.outputs.get("url") {
                    println!();
                    println!("Gist URL: {}", url);
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
    println!("gunbc-gist - Create a GitHub gist from code files");
    println!();
    println!("USAGE:");
    println!("    gunbc-gist [OPTIONS] [REPO_PATH]");
    println!();
    println!("OPTIONS:");
    println!("    -r, --repo <PATH>    Repository path (default: current directory)");
    println!("    -e, --ext <EXT>      Filter by file extension (can be repeated)");
    println!("    -p, --public         Create a public gist (default: secret)");
    println!("    -n, --dry-run        Don't actually create the gist");
    println!("    -h, --help           Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    gunbc-gist                        # Gist all files in current dir");
    println!("    gunbc-gist -e rs -e toml          # Only .rs and .toml files");
    println!("    gunbc-gist --dry-run              # Preview without creating");
    println!("    gunbc-gist ~/myproject --public   # Public gist of ~/myproject");
}
