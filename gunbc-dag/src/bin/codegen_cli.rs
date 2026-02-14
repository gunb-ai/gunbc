//! CLI and DAG generator - generates main.rs, graph.rs, CI YAML, and config files.
//!
//! This is a transaction-based code generator:
//! - `commit` (default): Generate CLIs, build binaries, create bin directory
//! - `rollback`: Remove all generated artifacts
//! - `codegen`: Just generate CLIs (partial commit)
//! - `cigen`: Generate CI workflow YAML (GitHub Actions and GitLab CI)
//!
//! Usage:
//!   gunbc-codegen                    # same as 'commit'
//!   gunbc-codegen commit             # full build transaction
//!   gunbc-codegen rollback           # undo all generated files
//!   gunbc-codegen codegen            # just generate CLIs
//!   gunbc-codegen cigen              # generate CI YAML files
//!   gunbc-codegen codegen --dry-run  # preview codegen
//!
//! # Architecture Note
//!
//! This binary lives in gunbc-dag (not gunbc-codegen) so that inventory
//! registrations from all tool crates are linked in. The codegen library
//! remains in core/codegen as a leaf crate.

#![deny(dead_code)]
use cargo_metadata::MetadataCommand;
use gunbc_codegen::{
    all_cleanable_outputs, derive_tool_defs, generate_cli_with_import, FileWriter,
};
use gunbc_dag::WorkspaceBinary;
use gunbc_ir::resource::{
    check_manifest_freshness, codegen_resource_def, load_manifest_default,
    update_resource_manifest, FreshnessOptions, ManagedResource, ManifestEntry, ManifestFreshness,
    ManifestUpdateError, ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::transport::ci::{
    yaml_block, CacheConfig, CiRenderer, GitHubActionsProvider, GitLabCiProvider, RenderConfig,
};
use gunbc_ir::{CODEGEN_BIN_DIR, CODEGEN_LIB_DIR};
use gunbc_lib_transport::preflight::ensure_lint_upsert;
use gunbc_lib_transport::TransportIo;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

// Force-link crates that register tool targets via inventory.
// Without these references, the linker may dead-strip the inventory symbols
// and derive_tool_defs() would return an empty list.
use gunbc_clippy::clippy_tool;
use gunbc_dag::bootstrap::bootstrap_tool;
use gunbc_dag::makegen::makegen_tool;
use gunbc_deps::deps_tool;
use gunbc_gist::{gist_diff_tool, gist_recent_tool, gist_snapshot_tool};
use gunbc_lib_review::review_tool;

fn main() {
    // Touch the functions to prevent the linker from stripping them.
    let _: fn() = clippy_tool;
    let _: fn() = gist_snapshot_tool;
    let _: fn() = gist_diff_tool;
    let _: fn() = gist_recent_tool;
    let _: fn() = deps_tool;
    let _: fn() = review_tool;
    let _: fn() = makegen_tool;
    let _: fn() = bootstrap_tool;

    let args: Vec<String> = env::args().collect();

    // Parse command (first non-flag argument)
    let command = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("commit");

    let dry_run = args.iter().any(|a| a == "-n" || a == "--dry-run");

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    if let Err(err) = ensure_lint_upsert() {
        eprintln!("preflight failed: {}", err);
        std::process::exit(1);
    }

    match command {
        "commit" => cmd_commit(dry_run),
        "rollback" => cmd_rollback(dry_run),
        "codegen" => cmd_codegen(dry_run),
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
    let io = TransportIo::new();
    println!("gunbc-codegen: commit transaction");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    // Step 1: Generate CLIs
    println!("[1/3] Generating CLIs...");
    if !codegen_clis(dry_run, &io) {
        eprintln!("Codegen failed");
        std::process::exit(1);
    }

    // Update resource manifest to record successful codegen
    update_manifest_after_codegen(dry_run, &io);

    // Step 2: Build with cargo
    println!("\n[2/3] Building binaries...");
    if !dry_run {
        match run_cargo_build(&io) {
            Ok(()) => println!("  cargo build: success"),
            Err(e) => {
                eprintln!("Cargo build failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("  (dry-run: would run cargo build)");
    }

    // Step 3: Setup bin directory (cross-platform)
    println!("\n[3/3] Setting up bin directory...");
    if !dry_run {
        match setup_bin_directory(&io) {
            Ok(()) => println!("  bin -> target/debug (symlink or copy)"),
            Err(e) => {
                eprintln!("Warning: Could not setup bin directory: {}", e);
                eprintln!("         Binaries are available at target/debug/");
                // Non-fatal - binaries are still built
            }
        }
    } else {
        println!("  (dry-run: would setup bin -> target/debug)");
    }

    println!("\nCommit complete. Binaries available at ./bin/ or ./target/debug/");
}

/// Run cargo build via the transport boundary.
fn run_cargo_build(io: &dyn ResourceIo) -> Result<(), ResourceError> {
    let cmd = gunbc_ir::cargo::CargoCommand::new(gunbc_ir::cargo::Subcommand::Build);
    let full_args = cmd.to_args();
    io.command_output(&full_args[0], &full_args[1..])
        .map(|_| ())
}

/// Setup bin directory - symlink on Unix, marker on Windows.
fn setup_bin_directory(io: &dyn ResourceIo) -> Result<(), ResourceError> {
    let bin_path = Path::new("bin");
    let target_path = Path::new("target/debug");

    // Remove existing bin directory/symlink/file
    remove_path(io, bin_path)?;

    setup_bin_link(io, bin_path, target_path)
}

#[cfg(unix)]
fn setup_bin_link(
    io: &dyn ResourceIo,
    bin_path: &Path,
    target_path: &Path,
) -> Result<(), ResourceError> {
    let args = vec![
        "-s".to_string(),
        target_path.to_string_lossy().into_owned(),
        bin_path.to_string_lossy().into_owned(),
    ];
    io.command_output("ln", &args)?;
    Ok(())
}

#[cfg(windows)]
fn setup_bin_link(
    io: &dyn ResourceIo,
    bin_path: &Path,
    _target_path: &Path,
) -> Result<(), ResourceError> {
    let marker_content = "Binaries are in target/debug/\n";
    let marker_path = bin_path.join(".location");
    io.write_file(&marker_path, marker_content.as_bytes())?;
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn setup_bin_link(
    _io: &dyn ResourceIo,
    _bin_path: &Path,
    _target_path: &Path,
) -> Result<(), ResourceError> {
    Ok(())
}

fn remove_path(io: &dyn ResourceIo, path: &Path) -> Result<(), ResourceError> {
    if !io.file_exists(path)? {
        return Ok(());
    }

    #[cfg(windows)]
    {
        let path_str = path.to_string_lossy().into_owned();
        let _ = io.command_output(
            "cmd",
            &[
                "/C".to_string(),
                "rmdir".to_string(),
                "/S".to_string(),
                "/Q".to_string(),
                path_str.clone(),
            ],
        );
        let _ = io.command_output(
            "cmd",
            &[
                "/C".to_string(),
                "del".to_string(),
                "/F".to_string(),
                "/Q".to_string(),
                path_str,
            ],
        );
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        let args = vec!["-rf".to_string(), path.to_string_lossy().into_owned()];
        io.command_output("rm", &args)?;
        Ok(())
    }
}

/// Rollback: remove all generated artifacts
fn cmd_rollback(dry_run: bool) {
    let io = TransportIo::new();
    println!("gunbc-codegen: rollback transaction");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    let targets = all_cleanable_outputs();
    let mut errors = Vec::new();

    for target in &targets {
        let path = Path::new(target);
        if io.file_exists(path).unwrap_or(false) {
            if dry_run {
                println!("  would remove: {}", target);
            } else {
                match remove_path(&io, path) {
                    Ok(()) => println!("  removed: {}", target),
                    Err(e) => {
                        eprintln!("  failed to remove {}: {}", target, e);
                        errors.push((target.clone(), e.to_string()));
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
        println!(
            "\nRollback completed with {} error(s). Some files may remain.",
            errors.len()
        );
    }
}

/// Just generate CLIs (partial transaction)
fn cmd_codegen(dry_run: bool) {
    println!("gunbc-codegen: codegen only");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    let io = TransportIo::new();
    if !dry_run && should_skip_codegen(&io) {
        return;
    }

    if !codegen_clis(dry_run, &io) {
        eprintln!("Codegen failed");
        std::process::exit(1);
    }

    // Update resource manifest to record successful codegen
    update_manifest_after_codegen(dry_run, &io);
}

/// Generate CI workflow YAML files.
///
/// Generates both GitHub Actions and GitLab CI configurations.
fn cmd_cigen(dry_run: bool) {
    println!("gunbc-codegen: cigen");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    let io = TransportIo::new();
    let writer = FileWriter::new(dry_run, &io);

    let github_provider = GitHubActionsProvider;
    let gitlab_provider = GitLabCiProvider::default();

    // Generate CI YAML for gunbc-ci
    let codegen = WorkspaceBinary::Codegen.invocation();
    let tool = WorkspaceBinary::Ci.invocation();

    // Derive permissions from CI workflow integrations (checkout, GCP WIF, etc.)
    let ci_perms: Vec<(String, String)> = gunbc_dag::ci::graph::ci_workflow_permissions()
        .into_iter()
        .map(|(scope, level)| {
            (
                scope.as_yaml_key().to_string(),
                level.as_yaml_value().to_string(),
            )
        })
        .collect();

    // GCP secrets required by live flow tests
    let ci_secrets: Vec<String> = gunbc_dag::ci::graph::ci_gcp_secrets()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let config = RenderConfig::new("ci", tool)
        .with_generator(&codegen.binary, &format!("{} -- cigen", codegen.command()))
        .with_runner(gunbc_ir::transport::github_actions::ubuntu_latest())
        .with_cargo_env(gunbc_ir::CargoEnv::ci())
        .with_git(gunbc_ir::GitConfig::default())
        .with_cache(CacheConfig::rust())
        .with_permissions(ci_perms)
        .with_secrets_env(ci_secrets);

    let outputs: Vec<(&str, String, String)> = vec![
        (
            "GitHub Actions",
            generate_github_actions_template(&config),
            github_provider.output_path("ci"),
        ),
        (
            "GitLab CI",
            generate_gitlab_ci_template(&config),
            gitlab_provider.output_path("ci"),
        ),
    ];

    for (label, yaml, path) in &outputs {
        match writer.write_if_changed(Path::new(path), yaml) {
            Ok(result) => {
                let status = if dry_run {
                    "dry-run"
                } else if result.changed {
                    "written"
                } else {
                    "unchanged"
                };
                println!("  [ci] {} ({})", path, status);
            }
            Err(e) => {
                eprintln!("  [ci] {} ERROR: {}", label, e);
            }
        }
    }

    println!();
    println!("Generated: {} CI files", outputs.len());
}

/// Generate GitHub Actions YAML template.
fn generate_github_actions_template(config: &RenderConfig) -> String {
    let mut yaml = String::new();

    yaml.push_str(&config.header("#"));
    yaml.push_str(&format!("\n\nname: {}\n\n", config.workflow_name));

    let branches = config.git.ci_branches();
    yaml.push_str("on:\n  push:\n");
    yaml_block(&mut yaml, "    branches:", &branches, |b| {
        format!("      - {}", b)
    });
    yaml.push_str("  pull_request:\n");
    yaml_block(&mut yaml, "    branches:", &branches, |b| {
        format!("      - {}", b)
    });

    yaml_block(
        &mut yaml,
        "permissions:",
        &config.permissions,
        |(scope, level)| format!("  {}: {}", scope, level),
    );

    yaml_block(&mut yaml, "env:", &config.all_env(), |(k, v)| {
        format!("  {}: {}", k, v)
    });

    yaml.push_str(&format!(
        "jobs:\n  {}:\n    runs-on: {}\n    timeout-minutes: {}\n    steps:\n",
        config.workflow_name, config.runner.id, config.timeout_minutes,
    ));

    if let Some(checkout) = &config.checkout {
        yaml.push_str("      - name: Checkout\n        uses: actions/checkout@v4\n");
        if let Some(depth) = checkout.fetch_depth {
            yaml.push_str(&format!(
                "        with:\n          fetch-depth: {}\n",
                depth
            ));
        }
        yaml.push('\n');
    }

    yaml.push_str("      - name: Setup Rust\n        uses: dtolnay/rust-toolchain@stable\n\n");

    if let Some(cache) = &config.cache {
        yaml.push_str("      - name: Cache Cargo\n        uses: actions/cache@v4\n        with:\n");
        yaml_block(&mut yaml, "          path: |", &cache.paths, |p| {
            format!("            {}", p)
        });
        yaml.push_str(&format!("          key: {}\n", cache.key));
        yaml_block(
            &mut yaml,
            "          restore-keys: |",
            &cache.restore_keys,
            |k| format!("            {}", k),
        );
    }

    yaml.push_str(&format!(
        "      - name: Run CI Pipeline\n        run: {}\n",
        config.tool.command(),
    ));

    // Step-level env: CARGO_INCREMENTAL overrides dtolnay/rust-toolchain's
    // CARGO_INCREMENTAL=0 (set via $GITHUB_ENV). Step-level env takes precedence.
    yaml.push_str("        env:\n");
    yaml.push_str("          CARGO_INCREMENTAL: \"1\"\n");
    for secret in &config.secrets_env {
        yaml.push_str(&format!(
            "          {}: ${{{{ secrets.{} }}}}\n",
            secret, secret
        ));
    }

    yaml
}

/// Generate GitLab CI YAML template.
fn generate_gitlab_ci_template(config: &RenderConfig) -> String {
    let mut yaml = String::new();

    yaml.push_str(&config.header("#"));
    yaml.push_str("\n\nimage: rust:latest\n\n");

    yaml_block(&mut yaml, "variables:", &config.all_env(), |(k, v)| {
        format!("  {}: \"{}\"", k, v)
    });

    yaml.push_str("stages:\n  - ci\n\n");

    yaml.push_str(
        "cache:\n  key: cargo-${CI_COMMIT_REF_SLUG}\n  paths:\n    - .cargo/\n    - target/\n\n",
    );

    yaml.push_str(&format!(
        "{}:\n  stage: ci\n  script:\n    - {}\n",
        config.workflow_name,
        config.tool.command(),
    ));

    yaml
}

/// Resolve workspace package names to their directory paths using cargo metadata.
fn resolve_workspace_packages() -> Option<(PathBuf, HashMap<String, PathBuf>)> {
    let metadata = MetadataCommand::new().no_deps().exec().ok()?;
    let workspace_root = PathBuf::from(metadata.workspace_root.as_std_path());
    let member_ids: HashSet<_> = metadata.workspace_members.into_iter().collect();

    let mut packages = HashMap::new();
    for package in metadata.packages {
        if !member_ids.contains(&package.id) {
            continue;
        }
        let manifest_dir = package
            .manifest_path
            .as_std_path()
            .parent()
            .map(|p| p.to_path_buf())?;
        packages.insert(package.name.clone(), manifest_dir);
    }

    Some((workspace_root, packages))
}

/// Compute a relative path from `from` to `to`.
fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from_parts: Vec<_> = from.components().collect();
    let to_parts: Vec<_> = to.components().collect();

    let mut i = 0;
    while i < from_parts.len() && i < to_parts.len() && from_parts[i] == to_parts[i] {
        i += 1;
    }

    let mut result = PathBuf::new();
    for _ in i..from_parts.len() {
        result.push("..");
    }
    for comp in &to_parts[i..] {
        result.push(comp.as_os_str());
    }
    result
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Ensure a Cargo.toml has a `[[bin]]` entry for the given binary.
fn ensure_bin_entry(doc: &mut DocumentMut, bin_name: &str, bin_path: &str) -> Result<bool, String> {
    let entry = doc
        .as_table_mut()
        .entry("bin")
        .or_insert(Item::ArrayOfTables(ArrayOfTables::new()));

    let bins = entry
        .as_array_of_tables_mut()
        .ok_or_else(|| "Cargo.toml [bin] must be an array of tables".to_string())?;

    if bins
        .iter()
        .any(|t| t.get("name").and_then(|v| v.as_str()) == Some(bin_name))
    {
        return Ok(false);
    }

    let mut table = Table::new();
    table["name"] = value(bin_name);
    table["path"] = value(bin_path);
    bins.push(table);
    Ok(true)
}

/// Generate CLI main.rs files for all tools and register binary targets.
fn codegen_clis(dry_run: bool, io: &dyn ResourceIo) -> bool {
    let writer = FileWriter::new(dry_run, io);
    let tools = derive_tool_defs();
    let output_dir = CODEGEN_BIN_DIR;

    struct BinRegistration {
        tool_name: String,
        bin_name: String,
        cargo_toml_path: PathBuf,
        doc: DocumentMut,
        rel_path: String,
    }

    let mut errors: Vec<String> = Vec::new();
    let mut registrations: Vec<BinRegistration> = Vec::new();

    let Some((workspace_root, package_dirs)) = resolve_workspace_packages() else {
        eprintln!("  ERROR: could not resolve workspace packages via cargo metadata");
        return false;
    };

    for tool in &tools {
        let Some(inv) = &tool.invocation else {
            continue;
        };
        let package_name = inv.package.as_ref().unwrap_or(&inv.binary);
        let Some(crate_dir) = package_dirs.get(package_name.as_str()) else {
            errors.push(format!(
                "[{}] package '{}' not found in workspace",
                tool.meta.tool_name, package_name
            ));
            continue;
        };

        let cargo_toml_path = crate_dir.join("Cargo.toml");
        let cargo_content = match io.read_file(&cargo_toml_path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!(
                        "[{}] could not parse {} as UTF-8: {}",
                        tool.meta.tool_name,
                        cargo_toml_path.display(),
                        e
                    ));
                    continue;
                }
            },
            Err(e) => {
                errors.push(format!(
                    "[{}] could not read {}: {}",
                    tool.meta.tool_name,
                    cargo_toml_path.display(),
                    e
                ));
                continue;
            }
        };

        let doc = match cargo_content.parse::<DocumentMut>() {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!(
                    "[{}] could not parse {}: {}",
                    tool.meta.tool_name,
                    cargo_toml_path.display(),
                    e
                ));
                continue;
            }
        };

        let bin_abs_path = workspace_root
            .join(CODEGEN_BIN_DIR)
            .join(&tool.meta.tool_name)
            .join("main.rs");
        let rel_path = normalize_path(&relative_path(crate_dir, &bin_abs_path));

        registrations.push(BinRegistration {
            tool_name: tool.meta.tool_name.clone(),
            bin_name: inv.binary.clone(),
            cargo_toml_path,
            doc,
            rel_path,
        });
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("  ERROR: {}", err);
        }
        return false;
    }

    for tool in &tools {
        let code =
            generate_cli_with_import(&tool.meta, &tool.entrypoints, tool.custom_import.as_deref());
        let tool_dir = Path::new(output_dir).join(&tool.meta.tool_name);
        let main_path = tool_dir.join("main.rs");

        match writer.write_if_changed(&main_path, &code) {
            Ok(result) => {
                let status = if dry_run {
                    "dry-run"
                } else if result.changed {
                    "written"
                } else {
                    "unchanged"
                };
                println!(
                    "  [{}] {} ({})",
                    tool.meta.tool_name,
                    main_path.display(),
                    status
                );
            }
            Err(e) => {
                errors.push(format!(
                    "[{}] could not write {}: {}",
                    tool.meta.tool_name,
                    main_path.display(),
                    e
                ));
            }
        }
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("  ERROR: {}", err);
        }
        return false;
    }

    // Ensure [[bin]] entries exist in target Cargo.toml files.
    println!();
    println!("  Registering binary targets...");

    for reg in &mut registrations {
        match ensure_bin_entry(&mut reg.doc, &reg.bin_name, &reg.rel_path) {
            Ok(false) => {
                println!(
                    "  [{}] {} (already registered)",
                    reg.tool_name,
                    reg.cargo_toml_path.display()
                );
            }
            Ok(true) => match writer.write_if_changed(&reg.cargo_toml_path, reg.doc.to_string()) {
                Ok(result) => {
                    let status = if dry_run {
                        "dry-run"
                    } else if result.changed {
                        "registered"
                    } else {
                        "unchanged"
                    };
                    println!(
                        "  [{}] {} → {} ({})",
                        reg.tool_name,
                        reg.bin_name,
                        reg.cargo_toml_path.display(),
                        status
                    );
                }
                Err(e) => {
                    errors.push(format!(
                        "[{}] could not write {}: {}",
                        reg.tool_name,
                        reg.cargo_toml_path.display(),
                        e
                    ));
                }
            },
            Err(e) => {
                errors.push(format!(
                    "[{}] could not update {}: {}",
                    reg.tool_name,
                    reg.cargo_toml_path.display(),
                    e
                ));
            }
        }
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("  ERROR: {}", err);
        }
        return false;
    }

    true
}

// ============================================================================
// Resource Manifest Support
// ============================================================================

#[derive(Clone)]
struct CodegenResource {
    def: ResourceDef,
    outputs: Vec<PathBuf>,
}

impl CodegenResource {
    fn new() -> Self {
        Self {
            def: codegen_resource_def(),
            outputs: vec![
                PathBuf::from(CODEGEN_BIN_DIR),
                PathBuf::from(CODEGEN_LIB_DIR),
            ],
        }
    }
}

impl ManagedResource for CodegenResource {
    fn definition(&self) -> &ResourceDef {
        &self.def
    }

    fn create(
        &self,
        manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<ManifestEntry, ResourceError> {
        let (key, file_count, input_files) = self.compute_key_with_file_list(manifest, io)?;
        Ok(ManifestEntry::new(key, file_count)
            .with_outputs(self.outputs.clone())
            .with_input_files(input_files))
    }
}

fn should_skip_codegen(io: &dyn ResourceIo) -> bool {
    let output_exists = codegen_outputs_exist(io);
    let manifest = match load_manifest_default(io) {
        Ok(m) if m.is_empty() => return false,
        Ok(m) => m,
        Err(e) => {
            eprintln!("  Warning: could not load resource manifest: {}", e);
            return false;
        }
    };

    let resource = CodegenResource::new();
    match check_manifest_freshness(
        &resource,
        &manifest,
        FreshnessOptions {
            output_exists: Some(output_exists),
            use_mtime: true,
        },
        io,
    ) {
        ManifestFreshness::Fresh => {
            println!("  Codegen outputs are fresh (manifest + outputs). Skipping.");
            true
        }
        ManifestFreshness::Stale(reason) => {
            println!("  Codegen outputs are stale: {}", reason);
            false
        }
        ManifestFreshness::Missing => false,
        ManifestFreshness::Error(err) => {
            eprintln!("  Warning: could not verify codegen freshness: {}", err);
            false
        }
    }
}

fn codegen_outputs_exist(io: &dyn ResourceIo) -> bool {
    let mut paths: Vec<PathBuf> = Vec::new();

    for tool in derive_tool_defs() {
        if tool.invocation.is_none() {
            continue;
        }
        paths.push(
            Path::new(CODEGEN_BIN_DIR)
                .join(&tool.meta.tool_name)
                .join("main.rs"),
        );
    }

    if paths.is_empty() {
        return true;
    }

    paths
        .iter()
        .all(|path| io.file_exists(path).unwrap_or(false))
}

/// Update the resource manifest after successful codegen.
fn update_manifest_after_codegen(dry_run: bool, io: &dyn ResourceIo) {
    if dry_run {
        println!("\n  (dry-run: would update resource manifest)");
        return;
    }

    println!("\n  Updating resource manifest...");
    let resource = CodegenResource::new();

    match update_resource_manifest(&resource, io) {
        Ok(()) => {
            println!("  Updated resource manifest: target/.resource-manifest.json");
        }
        Err(ManifestUpdateError::Load(e)) => {
            eprintln!("  ERROR: Could not load manifest: {}", e);
            eprintln!("  Codegen outputs exist but freshness cannot be verified.");
            eprintln!("  CI --mode=verify will fail until manifest is written.");
        }
        Err(ManifestUpdateError::Save(e)) => {
            eprintln!("  ERROR: Could not write manifest: {}", e);
            eprintln!("  Codegen outputs exist but freshness cannot be verified.");
            eprintln!("  CI --mode=verify will fail until manifest is written.");
        }
        Err(ManifestUpdateError::Acquire(e)) => {
            eprintln!("  ERROR: Could not update manifest: {}", e);
            eprintln!("  Codegen outputs exist but freshness cannot be verified.");
            eprintln!("  CI --mode=verify will fail until manifest is written.");
        }
    }
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
    println!("    cigen        Generate CI workflow YAML (GitHub Actions & GitLab CI)");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Preview changes without writing");
    println!("    -h, --help       Print this help");
    println!();
    println!("EXAMPLES:");
    println!("    gunbc-codegen                # full build");
    println!("    gunbc-codegen rollback       # clean everything");
    println!("    gunbc-codegen codegen -n     # preview CLI generation");
    println!("    gunbc-codegen cigen          # generate CI YAML files");
}
