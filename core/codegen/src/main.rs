//! CLI and DAG generator - generates main.rs and graph.rs for all tools.
//!
//! This is a transaction-based code generator:
//! - `commit` (default): Generate CLIs, build binaries, create symlink
//! - `rollback`: Remove all generated artifacts
//! - `codegen`: Just generate CLIs (partial commit)
//! - `daggen`: Generate graph.rs from declarative DAG definitions
//!
//! Usage:
//!   gunbc-codegen                    # same as 'commit'
//!   gunbc-codegen commit             # full build transaction
//!   gunbc-codegen rollback           # undo all generated files
//!   gunbc-codegen codegen            # just generate CLIs
//!   gunbc-codegen daggen             # generate graph.rs files
//!   gunbc-codegen codegen --dry-run  # preview codegen

use gunbc_codegen::{
    all_cleanable_outputs, all_tools, generate_cli_with_import, generate_graph_rs, FileWriter,
};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse command (first non-flag argument)
    let command = args.iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("commit");
    
    let dry_run = args.iter().any(|a| a == "-n" || a == "--dry-run");
    
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }
    
    match command {
        "commit" => cmd_commit(dry_run),
        "rollback" => cmd_rollback(dry_run),
        "codegen" => cmd_codegen(dry_run),
        "daggen" => cmd_daggen(dry_run),
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'gunbc-codegen --help' for usage");
            std::process::exit(1);
        }
    }
}

/// Full build transaction: codegen → cargo build → symlink
fn cmd_commit(dry_run: bool) {
    println!("gunbc-codegen: commit transaction");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    // Step 1: Generate CLIs
    println!("[1/3] Generating CLIs...");
    if !codegen_clis(dry_run) {
        eprintln!("Codegen failed");
        std::process::exit(1);
    }
    
    // Step 2: Build with cargo
    println!("\n[2/3] Building binaries...");
    if !dry_run {
        let status = Command::new("cargo")
            .args(["build", "--release"])
            .status()
            .expect("Failed to run cargo");
        
        if !status.success() {
            eprintln!("Cargo build failed");
            std::process::exit(1);
        }
    } else {
        println!("  (dry-run: would run cargo build --release)");
    }
    
    // Step 3: Create symlink
    println!("\n[3/3] Creating bin symlink...");
    if !dry_run {
        let _ = fs::remove_file("bin");
        #[cfg(unix)]
        std::os::unix::fs::symlink("target/release", "bin")
            .expect("Failed to create symlink");
        println!("  bin -> target/release");
    } else {
        println!("  (dry-run: would create bin -> target/release)");
    }
    
    println!("\nCommit complete. Binaries available at ./bin/");
}

/// Rollback: remove all generated artifacts
fn cmd_rollback(dry_run: bool) {
    println!("gunbc-codegen: rollback transaction");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    let targets = all_cleanable_outputs();
    
    for target in &targets {
        let path = Path::new(target);
        if path.exists() || path.is_symlink() {
            if dry_run {
                println!("  would remove: {}", target);
            } else {
                if path.is_dir() {
                    fs::remove_dir_all(path).ok();
                } else {
                    fs::remove_file(path).ok();
                }
                println!("  removed: {}", target);
            }
        }
    }
    
    if dry_run {
        println!("\nDry-run complete. No files removed.");
    } else {
        println!("\nRollback complete. Run 'gunbc-codegen commit' to rebuild.");
    }
}

/// Just generate CLIs (partial transaction)
fn cmd_codegen(dry_run: bool) {
    println!("gunbc-codegen: codegen only");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    codegen_clis(dry_run);
}

/// Generate graph.rs files from declarative DAG definitions.
fn cmd_daggen(dry_run: bool) {
    println!("gunbc-codegen: daggen");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    let writer = FileWriter::new(dry_run);
    let tools = all_tools();
    let output_dir = "buck-out/gen/lib";
    
    let mut generated = 0;
    let mut skipped = 0;
    
    for tool in &tools {
        if let Some(code) = generate_graph_rs(&tool) {
            let tool_dir = Path::new(output_dir).join(&tool.meta.tool_name);
            let graph_path = tool_dir.join("graph.rs");
            
            match writer.write(&graph_path, &code) {
                Ok(result) => {
                    let status = if result.written {
                        if result.changed { "written" } else { "unchanged" }
                    } else {
                        "dry-run"
                    };
                    println!("  [{}] {} ({})", tool.meta.tool_name, graph_path.display(), status);
                    generated += 1;
                }
                Err(e) => {
                    eprintln!("  [{}] ERROR: {}", tool.meta.tool_name, e);
                }
            }
        } else {
            println!("  [{}] skipped (no declarative DAG)", tool.meta.tool_name);
            skipped += 1;
        }
    }
    
    println!();
    println!("Generated: {}, Skipped: {}", generated, skipped);
}

/// Generate CLI main.rs files for all tools
fn codegen_clis(dry_run: bool) -> bool {
    let writer = FileWriter::new(dry_run);
    let tools = all_tools();
    let output_dir = "buck-out/gen/bin";
    
    let mut success = true;
    
    for tool in &tools {
        let code = generate_cli_with_import(
            &tool.meta,
            &tool.entrypoints,
            &tool.boundaries,
            tool.custom_import.as_deref(),
        );
        let tool_dir = Path::new(output_dir).join(&tool.meta.tool_name);
        let main_path = tool_dir.join("main.rs");
        
        match writer.write(&main_path, &code) {
            Ok(result) => {
                let status = if result.written {
                    if result.changed { "written" } else { "unchanged" }
                } else {
                    "dry-run"
                };
                println!("  [{}] {} ({})", tool.meta.tool_name, main_path.display(), status);
            }
            Err(e) => {
                eprintln!("  [{}] ERROR: {}", tool.meta.tool_name, e);
                success = false;
            }
        }
    }
    
    success
}

fn print_help() {
    println!("gunbc-codegen - Transaction-based code generator");
    println!();
    println!("USAGE:");
    println!("    gunbc-codegen [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    commit     Generate CLIs, build binaries, create symlink (default)");
    println!("    rollback   Remove all generated artifacts (clean)");
    println!("    codegen    Just generate CLIs (partial commit)");
    println!("    daggen     Generate graph.rs from declarative DAG definitions");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Preview changes without writing");
    println!("    -h, --help       Print this help");
    println!();
    println!("EXAMPLES:");
    println!("    gunbc-codegen                # full build");
    println!("    gunbc-codegen rollback       # clean everything");
    println!("    gunbc-codegen codegen -n     # preview CLI generation");
    println!("    gunbc-codegen daggen         # generate graph.rs for tools with DAG defs");
}
