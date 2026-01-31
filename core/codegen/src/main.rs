//! CLI and DAG generator - generates main.rs, graph.rs, CI YAML, and config files.
//!
//! This is a transaction-based code generator:
//! - `commit` (default): Generate CLIs, build binaries, create bin directory
//! - `rollback`: Remove all generated artifacts
//! - `codegen`: Just generate CLIs (partial commit)
//! - `daggen`: Generate graph.rs from declarative DAG definitions
//! - `cigen`: Generate CI workflow YAML (GitHub Actions and GitLab CI)
//! - `clippy-toml`: Generate clippy.toml from ClippyConfig
//!
//! Usage:
//!   gunbc-codegen                    # same as 'commit'
//!   gunbc-codegen commit             # full build transaction
//!   gunbc-codegen rollback           # undo all generated files
//!   gunbc-codegen codegen            # just generate CLIs
//!   gunbc-codegen daggen             # generate graph.rs files
//!   gunbc-codegen cigen              # generate CI YAML files
//!   gunbc-codegen clippy-toml        # generate clippy.toml
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

use gunbc_clippy::ClippyConfigRenderer;
use gunbc_codegen::{
    all_cleanable_outputs, all_tools, generate_cli_with_import, generate_graph_rs, FileWriter,
};
use gunbc_ir::transport::ci::{CiRenderer, GitHubActionsProvider, GitLabCiProvider, RenderConfig};
use gunbc_ir::Renderable;
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
        "clippy-toml" => cmd_clippy_toml(dry_run),
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

/// Generate CI workflow YAML files.
///
/// Generates both GitHub Actions and GitLab CI configurations.
/// The CI tool (gunbc-ci) has a handwritten main.rs that handles
/// the resource acquisition pattern internally - it runs codegen
/// if generated files are missing.
fn cmd_cigen(dry_run: bool) {
    println!("gunbc-codegen: cigen");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();
    
    let writer = FileWriter::new(dry_run);
    
    let github_provider = GitHubActionsProvider;
    let gitlab_provider = GitLabCiProvider::default();
    
    // Generate CI YAML for gunbc-ci
    // gunbc-ci is special - it has a handwritten main.rs that handles codegen internally
    let codegen = gunbc_ir::CargoInvocation::standalone("codegen");
    let tool = gunbc_ir::CargoInvocation::composed("ci", "dag");
    // Repo-level cargo config: colored output + warnings-as-errors
    let cargo_env = gunbc_ir::CargoEnv {
        term_color: gunbc_ir::TermColor::Always,
        warnings: gunbc_ir::Warnings::Deny,
    };
    let config = RenderConfig::new("ci", tool)
        .with_generator(
            &codegen.binary,
            &format!("{} -- cigen", codegen.command()),
        )
        .with_runner(gunbc_ir::transport::github_actions::ubuntu_latest())
        .with_cargo_env(cargo_env)
        .with_git(gunbc_ir::GitConfig::default());
    
    // Generate GitHub Actions YAML
    let ci_yaml = generate_github_actions_template(&config);
    let github_path = github_provider.output_path("ci");
    
    match writer.write(Path::new(&github_path), &ci_yaml) {
        Ok(result) => {
            let status = if result.written {
                if result.changed { "written" } else { "unchanged" }
            } else {
                "dry-run"
            };
            println!("  [ci] {} ({})", github_path, status);
        }
        Err(e) => {
            eprintln!("  [ci] GitHub Actions ERROR: {}", e);
        }
    }
    
    // Generate GitLab CI YAML
    let gitlab_yaml = generate_gitlab_ci_template(&config);
    let gitlab_path = gitlab_provider.output_path("ci");
    
    match writer.write(Path::new(&gitlab_path), &gitlab_yaml) {
        Ok(result) => {
            let status = if result.written {
                if result.changed { "written" } else { "unchanged" }
            } else {
                "dry-run"
            };
            println!("  [ci] {} ({})", gitlab_path, status);
        }
        Err(e) => {
            eprintln!("  [ci] GitLab CI ERROR: {}", e);
        }
    }
    
    println!();
    println!("Generated: 2 CI files");
}

/// Generate clippy.toml from ClippyConfig.
///
/// Uses the transport pattern preset which enforces:
/// - Direct I/O operations must go through the transport layer
/// - Process execution uses the tool acquisition pattern
fn cmd_clippy_toml(dry_run: bool) {
    println!("gunbc-codegen: clippy-toml");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    let writer = FileWriter::new(dry_run);
    let renderer = ClippyConfigRenderer::transport_pattern();
    let content = renderer.render();

    let clippy_path = Path::new("clippy.toml");

    match writer.write(clippy_path, &content) {
        Ok(result) => {
            let status = if result.written {
                if result.changed { "written" } else { "unchanged" }
            } else {
                "dry-run"
            };
            println!("  {} ({})", clippy_path.display(), status);
        }
        Err(e) => {
            eprintln!("  ERROR: {}", e);
            std::process::exit(1);
        }
    }

    println!();
    println!("Generated: clippy.toml");
}

/// Emit a YAML block only if `items` is non-empty.
///
/// Writes `header\n`, then each formatted item as a line, then a trailing blank line.
/// Skips entirely when `items` is empty.
fn yaml_block<T>(yaml: &mut String, header: &str, items: &[T], fmt: impl Fn(&T) -> String) {
    if items.is_empty() {
        return;
    }
    yaml.push_str(header);
    yaml.push('\n');
    for item in items {
        yaml.push_str(&fmt(item));
        yaml.push('\n');
    }
    yaml.push('\n');
}

/// Generate GitHub Actions YAML template.
fn generate_github_actions_template(config: &RenderConfig) -> String {
    let mut yaml = String::new();

    yaml.push_str(&config.header("#"));
    yaml.push_str(&format!("\n\nname: {}\n\n", config.workflow_name));

    // Triggers — derived from git config
    let branches = config.git.ci_branches();
    yaml.push_str("on:\n");
    yaml.push_str("  push:\n");
    yaml_block(&mut yaml, "    branches:", &branches, |b| format!("      - {}", b));
    yaml.push_str("  pull_request:\n");
    yaml_block(&mut yaml, "    branches:", &branches, |b| format!("      - {}", b));

    // Environment — derived from cargo env + manual overrides
    yaml_block(&mut yaml, "env:", &config.all_env(), |(k, v)| format!("  {}: {}", k, v));

    // Job — runner from the provider's image catalog
    yaml.push_str("jobs:\n");
    yaml.push_str(&format!("  {}:\n", config.workflow_name));
    yaml.push_str(&format!("    runs-on: {}\n", config.runner.id));
    yaml.push_str("    steps:\n");
    yaml.push_str("      - name: Checkout\n");
    yaml.push_str("        uses: actions/checkout@v4\n");
    yaml.push_str("        with:\n");
    yaml.push_str("          fetch-depth: 1\n");
    yaml.push('\n');
    
    // Rust toolchain
    yaml.push_str("      - name: Setup Rust\n");
    yaml.push_str("        uses: dtolnay/rust-toolchain@stable\n");
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
    
    // Run CI Pipeline
    // gunbc-ci has a handwritten main.rs that handles codegen internally via the prep node.
    // The prep node uses the resource acquisition (upsert) pattern: check if generated
    // files exist, generate them if not. This makes CI self-healing.
    yaml.push_str("      - name: Run CI Pipeline\n");
    yaml.push_str(&format!("        run: {} --release\n", config.tool.command()));
    
    yaml
}

/// Generate GitLab CI YAML template.
fn generate_gitlab_ci_template(config: &RenderConfig) -> String {
    let mut yaml = String::new();

    yaml.push_str(&config.header("#"));
    yaml.push_str("\n\n");
    
    // Image — runner from the provider's image catalog
    yaml.push_str("image: rust:latest\n\n");

    // Variables — derived from cargo env + manual overrides
    yaml_block(&mut yaml, "variables:", &config.all_env(), |(k, v)| format!("  {}: \"{}\"", k, v));
    
    // Stages
    yaml.push_str("stages:\n");
    yaml.push_str("  - ci\n\n");
    
    // Cache
    yaml.push_str("cache:\n");
    yaml.push_str("  key: cargo-${CI_COMMIT_REF_SLUG}\n");
    yaml.push_str("  paths:\n");
    yaml.push_str("    - .cargo/\n");
    yaml.push_str("    - target/\n\n");
    
    // CI job - gunbc-ci handles codegen internally via the prep node
    yaml.push_str(&format!("{}:\n", config.workflow_name));
    yaml.push_str("  stage: ci\n");
    yaml.push_str("  script:\n");
    yaml.push_str(&format!("    - {} --release\n", config.tool.command()));
    
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
    println!("    commit       Generate CLIs, build binaries, create symlink (default)");
    println!("    rollback     Remove all generated artifacts (clean)");
    println!("    codegen      Just generate CLIs (partial commit)");
    println!("    daggen       Generate graph.rs from declarative DAG definitions");
    println!("    cigen        Generate CI workflow YAML (GitHub Actions & GitLab CI)");
    println!("    clippy-toml  Generate clippy.toml from ClippyConfig");
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
    println!("    gunbc-codegen clippy-toml    # generate clippy.toml config");
}
