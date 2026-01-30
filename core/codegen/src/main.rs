//! CLI and DAG generator - generates main.rs, graph.rs, and CI YAML for all tools.
//!
//! This is a transaction-based code generator:
//! - `commit` (default): Generate CLIs, build binaries, create bin directory
//! - `rollback`: Remove all generated artifacts
//! - `codegen`: Just generate CLIs (partial commit)
//! - `daggen`: Generate graph.rs from declarative DAG definitions
//! - `cigen`: Generate CI workflow YAML (GitHub Actions and GitLab CI)
//!
//! Usage:
//!   gunbc-codegen                    # same as 'commit'
//!   gunbc-codegen commit             # full build transaction
//!   gunbc-codegen rollback           # undo all generated files
//!   gunbc-codegen codegen            # just generate CLIs
//!   gunbc-codegen daggen             # generate graph.rs files
//!   gunbc-codegen cigen              # generate CI YAML files
//!   gunbc-codegen codegen --dry-run  # preview codegen
//!
//! # Architecture Note
//!
//! This tool is the bootstrapper - it generates CLIs for other tools. As such,
//! it cannot use the transport pattern (which would create a circular dependency).
//! It uses direct filesystem and process operations by design.
//!
//! Future improvement: Express codegen as a DAG executed by a minimal bootstrap
//! executor that doesn't depend on the generated tools.

use gunbc_codegen::{
    all_cleanable_outputs, all_tools, generate_cli_with_import, generate_graph_rs, FileWriter,
};
use gunbc_ir::transport::ci::{CiRenderer, GitHubActionsProvider, GitLabCiProvider, RenderConfig};
use std::env;
use std::fs;
use std::io;
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
        "cigen" => cmd_cigen(dry_run),
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'gunbc-codegen --help' for usage");
            std::process::exit(1);
        }
    }
}

/// Full build transaction: codegen → cargo build → setup bin directory
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
        match run_cargo_build() {
            Ok(()) => println!("  cargo build --release: success"),
            Err(e) => {
                eprintln!("Cargo build failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("  (dry-run: would run cargo build --release)");
    }
    
    // Step 3: Setup bin directory (cross-platform)
    println!("\n[3/3] Setting up bin directory...");
    if !dry_run {
        match setup_bin_directory() {
            Ok(()) => println!("  bin -> target/release (symlink or copy)"),
            Err(e) => {
                eprintln!("Warning: Could not setup bin directory: {}", e);
                eprintln!("         Binaries are available at target/release/");
                // Non-fatal - binaries are still built
            }
        }
    } else {
        println!("  (dry-run: would setup bin -> target/release)");
    }
    
    println!("\nCommit complete. Binaries available at ./bin/ or ./target/release/");
}

/// Run cargo build --release
/// 
/// Note: This is the bootstrapper - it can't use the transport pattern
/// because it needs to build the transport layer first.
#[allow(clippy::disallowed_methods)]
fn run_cargo_build() -> io::Result<()> {
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "cargo exited with status: {}",
            status
        )))
    }
}

/// Setup bin directory - symlink on Unix, copy on Windows, with fallback
#[allow(clippy::disallowed_methods)] // Codegen main.rs is the bootstrapper, allowed to use fs ops
fn setup_bin_directory() -> io::Result<()> {
    let bin_path = Path::new("bin");
    let target_path = Path::new("target/release");
    
    // Remove existing bin directory/symlink/file
    if bin_path.exists() || bin_path.is_symlink() {
        if bin_path.is_dir() && !bin_path.is_symlink() {
            fs::remove_dir_all(bin_path)?;
        } else {
            fs::remove_file(bin_path)?;
        }
    }
    
    // Try symlink first (works on Unix and some Windows configurations)
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target_path, bin_path)
    }
    
    // On Windows, try to create a directory junction, or fall back to documenting the location
    #[cfg(windows)]
    {
        // Windows symlinks require admin privileges, so just create a simple
        // marker file pointing users to the right location
        let marker_content = "Binaries are in target/release/\n";
        fs::write(bin_path.join(".location"), marker_content)?;
        return Ok(());
    }
    
    // Fallback for other platforms
    #[cfg(not(any(unix, windows)))]
    {
        Ok(()) // Just skip - binaries are in target/release
    }
}

/// Rollback: remove all generated artifacts
#[allow(clippy::disallowed_methods)] // Codegen main.rs is the bootstrapper, allowed to use fs ops
fn cmd_rollback(dry_run: bool) {
    println!("gunbc-codegen: rollback transaction");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    let targets = all_cleanable_outputs();
    let mut errors = Vec::new();
    
    for target in &targets {
        let path = Path::new(target);
        if path.exists() || path.is_symlink() {
            if dry_run {
                println!("  would remove: {}", target);
            } else {
                let result = if path.is_dir() && !path.is_symlink() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };
                
                match result {
                    Ok(()) => println!("  removed: {}", target),
                    Err(e) => {
                        eprintln!("  failed to remove {}: {}", target, e);
                        errors.push((target.clone(), e));
                    }
                }
            }
        }
    }
    
    if dry_run {
        println!("\nDry-run complete. No files removed.");
    } else if errors.is_empty() {
        println!("\nRollback complete. Run 'gunbc-codegen commit' to rebuild.");
    } else {
        println!("\nRollback completed with {} error(s). Some files may remain.", errors.len());
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
        if let Some(code) = generate_graph_rs(tool) {
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

/// Generate CI workflow YAML files for tools with step mode enabled.
///
/// Generates both GitHub Actions and GitLab CI configurations from the
/// CI DAG structure. Currently generates a template based on the tool's
/// enable_step_mode flag.
fn cmd_cigen(dry_run: bool) {
    println!("gunbc-codegen: cigen");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    let writer = FileWriter::new(dry_run);
    let tools = all_tools();
    
    let github_provider = GitHubActionsProvider;
    let gitlab_provider = GitLabCiProvider::default();
    
    let mut generated = 0;
    let mut skipped = 0;
    
    for tool in &tools {
        if tool.meta.enable_step_mode {
            // Generate GitHub Actions YAML
            let config = RenderConfig::new(&tool.meta.tool_name, &tool.meta.crate_name)
                .with_runner("ubuntu-latest")
                .with_env("CARGO_TERM_COLOR", "always")
                .with_branches(vec!["main"]);
            
            // Create a minimal DAG for rendering (since we can't call the graph builder at compile time)
            // The actual step names come from the CI tool's list-steps command at runtime
            let ci_yaml = generate_ci_yaml_template(&github_provider, &config);
            let github_path = github_provider.output_path(&tool.meta.tool_name);
            
            match writer.write(Path::new(&github_path), &ci_yaml) {
                Ok(result) => {
                    let status = if result.written {
                        if result.changed { "written" } else { "unchanged" }
                    } else {
                        "dry-run"
                    };
                    println!("  [{}] {} ({})", tool.meta.tool_name, github_path, status);
                    generated += 1;
                }
                Err(e) => {
                    eprintln!("  [{}] GitHub Actions ERROR: {}", tool.meta.tool_name, e);
                }
            }
            
            // Generate GitLab CI YAML
            let gitlab_yaml = generate_ci_yaml_template(&gitlab_provider, &config);
            let gitlab_path = gitlab_provider.output_path(&tool.meta.tool_name);
            
            match writer.write(Path::new(&gitlab_path), &gitlab_yaml) {
                Ok(result) => {
                    let status = if result.written {
                        if result.changed { "written" } else { "unchanged" }
                    } else {
                        "dry-run"
                    };
                    println!("  [{}] {} ({})", tool.meta.tool_name, gitlab_path, status);
                    generated += 1;
                }
                Err(e) => {
                    eprintln!("  [{}] GitLab CI ERROR: {}", tool.meta.tool_name, e);
                }
            }
        } else {
            skipped += 1;
        }
    }
    
    println!();
    println!("Generated: {} CI files, Skipped: {} tools", generated, skipped);
}

/// Generate CI YAML template for a tool using the given provider.
///
/// Since we can't call the graph builder at compile time (circular dependency),
/// we generate a template that uses the tool's step mode CLI interface.
fn generate_ci_yaml_template<P: CiRenderer>(provider: &P, config: &RenderConfig) -> String {
    if provider.provider_id() == "github-actions" {
        generate_github_actions_template(config)
    } else {
        generate_gitlab_ci_template(config)
    }
}

/// Generate GitHub Actions YAML template.
fn generate_github_actions_template(config: &RenderConfig) -> String {
    let mut yaml = String::new();
    
    yaml.push_str("# Generated by gunbc-codegen. Do not edit manually.\n");
    yaml.push_str(&format!("name: {}\n\n", config.workflow_name));
    
    // Triggers
    yaml.push_str("on:\n");
    yaml.push_str("  push:\n");
    yaml.push_str("    branches:\n");
    for branch in &config.branches {
        yaml.push_str(&format!("      - {}\n", branch));
    }
    yaml.push_str("  pull_request:\n");
    yaml.push_str("    branches:\n");
    for branch in &config.branches {
        yaml.push_str(&format!("      - {}\n", branch));
    }
    yaml.push('\n');
    
    // Environment
    yaml.push_str("env:\n");
    yaml.push_str("  CARGO_TERM_COLOR: always\n");
    for (key, value) in &config.env {
        if key != "CARGO_TERM_COLOR" {
            yaml.push_str(&format!("  {}: {}\n", key, value));
        }
    }
    yaml.push('\n');
    
    // Job
    yaml.push_str("jobs:\n");
    yaml.push_str(&format!("  {}:\n", config.workflow_name));
    yaml.push_str(&format!("    runs-on: {}\n", config.runner));
    yaml.push_str("    steps:\n");
    yaml.push_str("      - name: Checkout\n");
    yaml.push_str("        uses: actions/checkout@v4\n");
    yaml.push_str("        with:\n");
    yaml.push_str("          fetch-depth: 1\n");
    yaml.push('\n');
    
    // Cache
    yaml.push_str("      - name: Cache Cargo\n");
    yaml.push_str("        uses: actions/cache@v4\n");
    yaml.push_str("        with:\n");
    yaml.push_str("          path: |\n");
    yaml.push_str("            ~/.cargo/bin/\n");
    yaml.push_str("            ~/.cargo/registry/index/\n");
    yaml.push_str("            ~/.cargo/registry/cache/\n");
    yaml.push_str("            ~/.cargo/git/db/\n");
    yaml.push_str("            target/\n");
    yaml.push_str("          key: cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}\n");
    yaml.push_str("          restore-keys: |\n");
    yaml.push_str("            cargo-${{ runner.os }}-\n");
    yaml.push('\n');
    
    // Build tool
    yaml.push_str("      - name: Build CI Tool\n");
    yaml.push_str(&format!("        run: cargo build --release -p {}\n\n", config.tool_binary));
    
    // Run full DAG (with step mode, the tool will emit groups for each step)
    yaml.push_str("      - name: Run CI Pipeline\n");
    yaml.push_str(&format!("        run: ./target/release/{}\n", config.tool_binary.replace('-', "_")));
    
    yaml
}

/// Generate GitLab CI YAML template.
fn generate_gitlab_ci_template(config: &RenderConfig) -> String {
    let mut yaml = String::new();
    
    yaml.push_str("# Generated by gunbc-codegen. Do not edit manually.\n\n");
    
    // Image
    yaml.push_str("image: rust:latest\n\n");
    
    // Variables
    yaml.push_str("variables:\n");
    yaml.push_str("  CARGO_TERM_COLOR: always\n");
    for (key, value) in &config.env {
        if key != "CARGO_TERM_COLOR" {
            yaml.push_str(&format!("  {}: \"{}\"\n", key, value));
        }
    }
    yaml.push('\n');
    
    // Stages
    yaml.push_str("stages:\n");
    yaml.push_str("  - build\n");
    yaml.push_str("  - run\n\n");
    
    // Cache
    yaml.push_str("cache:\n");
    yaml.push_str("  key: cargo-${CI_COMMIT_REF_SLUG}\n");
    yaml.push_str("  paths:\n");
    yaml.push_str("    - .cargo/\n");
    yaml.push_str("    - target/\n\n");
    
    // Build job
    yaml.push_str("build:\n");
    yaml.push_str("  stage: build\n");
    yaml.push_str("  script:\n");
    yaml.push_str(&format!("    - cargo build --release -p {}\n", config.tool_binary));
    yaml.push_str("  artifacts:\n");
    yaml.push_str("    paths:\n");
    yaml.push_str(&format!("      - target/release/{}\n", config.tool_binary.replace('-', "_")));
    yaml.push_str("    expire_in: 1 hour\n\n");
    
    // Run job
    yaml.push_str(&format!("{}:\n", config.workflow_name));
    yaml.push_str("  stage: run\n");
    yaml.push_str("  needs:\n");
    yaml.push_str("    - build\n");
    yaml.push_str("  script:\n");
    yaml.push_str(&format!("    - ./target/release/{}\n", config.tool_binary.replace('-', "_")));
    
    yaml
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
    println!("    cigen      Generate CI workflow YAML (GitHub Actions & GitLab CI)");
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
    println!("    gunbc-codegen cigen          # generate CI YAML files");
}
