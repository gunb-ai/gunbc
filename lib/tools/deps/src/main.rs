//! CLI for gunbc-deps.

use gunbc_deps::{build_deps_graph, DepsManifest, Installer};
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::Value;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut manifest_path = "deps.toml".to_string();
    let mut dry_run = false;
    let mut list_only = false;

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest" | "-m" => {
                i += 1;
                if i < args.len() {
                    manifest_path = args[i].clone();
                }
            }
            "--dry-run" | "-n" => {
                dry_run = true;
            }
            "--list" | "-l" => {
                list_only = true;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            "install" => {
                // Default command, do nothing
            }
            _ => {}
        }
        i += 1;
    }

    // List mode - just show dependencies
    if list_only {
        list_dependencies(&manifest_path);
        return;
    }

    // Build the graph
    let dag = build_deps_graph();

    // Set up execution mode
    let mode = if dry_run {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("execute_installs", "executed", Value::Bool(false));
        mocks.set_value(
            "execute_installs",
            "script",
            Value::Str("<DRY-RUN>".to_string()),
        );
        ExecutionMode::DryRun(mocks)
    } else {
        ExecutionMode::Real
    };

    let installer = Installer::new();
    println!("gunbc-deps");
    println!("  manifest: {}", manifest_path);
    println!("  platform: {}", installer.platform());
    println!("  mode: {}", if dry_run { "dry-run" } else { "install" });
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
                        Value::Str(s) if port == "install_script" || port == "script" => {
                            if !s.is_empty() && s != "<DRY-RUN>" {
                                println!("  {}:", port);
                                println!("--- SCRIPT ---");
                                println!("{}", s);
                                println!("--- END ---");
                            }
                        }
                        Value::Str(s) if s.len() < 100 => println!("  {}: {}", port, s),
                        Value::StrList(list) if !list.is_empty() => {
                            println!("  {}: {}", port, list.join(", "));
                        }
                        Value::StrList(_) => println!("  {}: (empty)", port),
                        Value::Int(n) => println!("  {}: {}", port, n),
                        Value::Bool(b) => println!("  {}: {}", port, b),
                        _ => {}
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

fn list_dependencies(manifest_path: &str) {
    match DepsManifest::load(manifest_path) {
        Ok(manifest) => {
            let installer = Installer::new();
            println!("Dependencies in {}:", manifest_path);
            println!();

            for dep in &manifest.dependency {
                let installed = installer.is_installed(&dep.verify);
                let status = if installed { "installed" } else { "missing" };
                println!("  {} [{}]", dep.name, status);
                println!("    verify: {}", dep.verify);

                if let Some(install) = dep.install_for(installer.platform()) {
                    println!("    method: {}", install.method);
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading manifest: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("gunbc-deps - Tool dependency management");
    println!();
    println!("USAGE:");
    println!("    gunbc-deps [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    install              Install dependencies (default)");
    println!();
    println!("OPTIONS:");
    println!("    -m, --manifest <PATH>  Manifest file path (default: deps.toml)");
    println!("    -n, --dry-run          Show what would be installed");
    println!("    -l, --list             List dependencies and their status");
    println!("    -h, --help             Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    gunbc-deps install           # Install all dependencies");
    println!("    gunbc-deps --dry-run         # Preview install scripts");
    println!("    gunbc-deps --list            # List dependencies");
}
