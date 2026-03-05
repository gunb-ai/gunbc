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
//! This binary lives in gunbc-dag (not gunbc-codegen) and discovers tools
//! via structural entrypoint inference. The codegen library remains in core/codegen
//! as a leaf crate.

#![deny(dead_code)]
use cargo_metadata::MetadataCommand;
use gunbc_cli::parse;
use gunbc_codegen::{core_outputs, generate_cli_with_import, FileWriter, ToolDef};
use gunbc_exec::{print_attention, run_freshness_steps, AttentionLevel};
use gunbc_ir::resource::{
    check_manifest_freshness, codegen_resource_def, load_manifest_default,
    update_resource_manifest, FreshnessOptions, ManagedResource, ManifestEntry, ManifestFreshness,
    ManifestUpdateError, ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::WorkspaceLayout;
use gunbc_lib_transport::TransportIo;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

fn main() {
    let args: Vec<String> = env::args().collect();
    let parsed = match parse(&args, &[]) {
        Ok(parsed) => parsed,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "Argument parsing failed",
                &e.to_string(),
            );
            std::process::exit(1);
        }
    };
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;
    let command = match parse_command_arg(&args) {
        Ok(command) => command,
        Err(e) => {
            print_attention(AttentionLevel::Error, "Invalid command", &e);
            print_attention(
                AttentionLevel::Info,
                "Usage",
                "Run 'gunbc-codegen --help' for usage",
            );
            std::process::exit(1);
        }
    };

    // Skip freshness check for codegen-only (no side effects beyond writing main.rs files)
    if command != "codegen" {
        if let Some(steps) = gunbc_lib_transport::check_and_plan_freshness() {
            if let Err(e) = run_freshness_steps(&steps) {
                print_attention(
                    AttentionLevel::Error,
                    "Freshness check failed",
                    &e.to_string(),
                );
                std::process::exit(1);
            }
        }
    }

    match command {
        "commit" => cmd_commit(dry_run),
        "rollback" => cmd_rollback(dry_run),
        "codegen" => cmd_codegen(dry_run),
        "cigen" => cmd_cigen(dry_run),
        _ => {
            print_attention(
                AttentionLevel::Error,
                "Unknown command",
                &format!("Unknown command: {command}"),
            );
            print_attention(
                AttentionLevel::Info,
                "Usage",
                "Run 'gunbc-codegen --help' for usage",
            );
            std::process::exit(1);
        }
    }
}

fn parse_command_arg(args: &[String]) -> Result<&str, String> {
    let positionals: Vec<&str> = args
        .iter()
        .skip(1)
        .filter(|arg| !arg.starts_with('-'))
        .map(|arg| arg.as_str())
        .collect();

    if positionals.len() > 1 {
        let extras = positionals[1..].join(" ");
        return Err(format!(
            "unexpected extra positional arguments: {extras} (expected at most one command)"
        ));
    }

    Ok(positionals.first().copied().unwrap_or("commit"))
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
        print_attention(
            AttentionLevel::Error,
            "Codegen failed",
            "CLI generation returned errors",
        );
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
                print_attention(AttentionLevel::Error, "Cargo build failed", &e.to_string());
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
                print_attention(
                    AttentionLevel::Warning,
                    "Could not setup bin directory",
                    &e.to_string(),
                );
                print_attention(
                    AttentionLevel::Info,
                    "Fallback",
                    "Binaries are available at target/debug/",
                );
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
        target_path.display().to_string(),
        bin_path.display().to_string(),
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
        let path_str = path.display().to_string();
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
        let args = vec!["-rf".to_string(), path.display().to_string()];
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

    let mut targets: Vec<String> = core_outputs().into_iter().map(|s| s.to_string()).collect();
    targets.push(normalize_path(&codegen_bin_dir()));
    targets.push(normalize_path(&codegen_lib_dir()));
    targets.sort();
    targets.dedup();
    let mut errors = Vec::new();

    for target in &targets {
        let path = Path::new(target);
        let exists = match io.file_exists(path) {
            Ok(exists) => exists,
            Err(e) => {
                eprintln!("  failed to probe {}: {}", target, e);
                errors.push((target.clone(), e.to_string()));
                continue;
            }
        };
        if exists {
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
        print_attention(
            AttentionLevel::Error,
            "Codegen failed",
            "CLI generation returned errors",
        );
        std::process::exit(1);
    }

    // Update resource manifest to record successful codegen
    update_manifest_after_codegen(dry_run, &io);
}

/// Generate CI workflow YAML files via the DSL cigen tool.
///
/// Builds and executes the DSL graph from tools/cigen.dag. The graph
/// handles both GitHub Actions and GitLab CI rendering via content_upsert.
fn cmd_cigen(dry_run: bool) {
    println!("gunbc-codegen: cigen");
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    let dag = match gunbc_dag::dsl_builder::build_dsl_graph_for_entrypoint(
        "tools/cigen.dag",
        Some("cigen"),
        None,
    ) {
        Ok(dag) => dag,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "Cigen graph build failed",
                &e.to_string(),
            );
            std::process::exit(1);
        }
    };

    let mode = if dry_run {
        gunbc_exec::ExecutionMode::DryRun(gunbc_exec::BoundaryMocks::default())
    } else {
        gunbc_exec::ExecutionMode::Real
    };

    gunbc_exec::execute_and_display(&dag, mode, false, None, None);
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
/// Discovers tools from DSL structural entrypoint inference — the DSL file IS the
/// registration. No inventory, no regex source parsing, no allowlists.
fn discover_codegen_tools(_workspace_root: &Path) -> Result<Vec<ToolDef>, String> {
    let tools = gunbc_dag::dsl_registry::try_discover_tool_defs_from_dsl()
        .map_err(|e| format!("DSL discovery failed: {e}"))?;
    if tools.is_empty() {
        return Err("no DSL tool entrypoints discovered".to_string());
    }
    Ok(tools)
}

/// Generate CLI main.rs files for all tools and register binary targets.
fn codegen_clis(dry_run: bool, io: &dyn ResourceIo) -> bool {
    let writer = FileWriter::new(dry_run, io);
    let output_dir = codegen_bin_dir();

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
    let tools = match discover_codegen_tools(&workspace_root) {
        Ok(tools) => tools,
        Err(e) => {
            eprintln!("  ERROR: {e}");
            return false;
        }
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

        let bin_abs_path = output_dir
            .join(tool.meta.tool_name.as_ref())
            .join("main.rs");
        let rel_path = normalize_path(&relative_path(crate_dir, &bin_abs_path));

        registrations.push(BinRegistration {
            tool_name: tool.meta.tool_name.to_string(),
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
        let tool_dir = output_dir.join(tool.meta.tool_name.as_ref());
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
            outputs: vec![codegen_bin_dir(), codegen_lib_dir()],
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
            print_attention(
                AttentionLevel::Warning,
                "Could not load resource manifest",
                &e.to_string(),
            );
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
        ManifestFreshness::FreshWithDiagnostic(note) => {
            println!(
                "  Codegen outputs are fresh (manifest + outputs). Skipping. [{}]",
                note
            );
            true
        }
        ManifestFreshness::Stale(reason) => {
            println!("  Codegen outputs are stale: {}", reason);
            false
        }
        ManifestFreshness::Missing => false,
        ManifestFreshness::Error(err) => {
            print_attention(
                AttentionLevel::Warning,
                "Could not verify codegen freshness",
                &err,
            );
            false
        }
    }
}

fn codegen_outputs_exist(io: &dyn ResourceIo) -> bool {
    let mut paths: Vec<PathBuf> = Vec::new();

    let Some((workspace_root, _)) = resolve_workspace_packages() else {
        return false;
    };
    let tools = match discover_codegen_tools(&workspace_root) {
        Ok(tools) => tools,
        Err(e) => {
            print_attention(AttentionLevel::Warning, "Codegen tool discovery failed", &e);
            return false;
        }
    };

    for tool in tools {
        if tool.invocation.is_none() {
            continue;
        }
        paths.push(
            codegen_bin_dir()
                .join(tool.meta.tool_name.as_ref())
                .join("main.rs"),
        );
    }

    if paths.is_empty() {
        return true;
    }

    for path in &paths {
        match io.file_exists(path) {
            Ok(true) => {}
            Ok(false) => return false,
            Err(e) => {
                print_attention(
                    AttentionLevel::Warning,
                    "Could not probe generated output",
                    &format!("{}: {e}", path.display()),
                );
                return false;
            }
        }
    }
    true
}

fn workspace_layout_or_none() -> Option<WorkspaceLayout> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .ok()
}

fn codegen_bin_dir() -> PathBuf {
    workspace_layout_or_none()
        .map(|layout| layout.codegen_bin_dir())
        .unwrap_or_else(|| PathBuf::from("target/codegen/bin"))
}

fn codegen_lib_dir() -> PathBuf {
    workspace_layout_or_none()
        .map(|layout| layout.codegen_lib_dir())
        .unwrap_or_else(|| PathBuf::from("target/codegen/lib"))
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
            print_attention(
                AttentionLevel::Error,
                "Could not load manifest",
                &e.to_string(),
            );
            print_attention(
                AttentionLevel::Warning,
                "Freshness verification unavailable",
                "Codegen outputs exist but freshness cannot be verified. CI --mode=verify will fail until manifest is written.",
            );
        }
        Err(ManifestUpdateError::Save(e)) => {
            print_attention(
                AttentionLevel::Error,
                "Could not write manifest",
                &e.to_string(),
            );
            print_attention(
                AttentionLevel::Warning,
                "Freshness verification unavailable",
                "Codegen outputs exist but freshness cannot be verified. CI --mode=verify will fail until manifest is written.",
            );
        }
        Err(ManifestUpdateError::Acquire(e)) => {
            print_attention(
                AttentionLevel::Error,
                "Could not update manifest",
                &e.to_string(),
            );
            print_attention(
                AttentionLevel::Warning,
                "Freshness verification unavailable",
                "Codegen outputs exist but freshness cannot be verified. CI --mode=verify will fail until manifest is written.",
            );
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

#[cfg(test)]
mod tests {
    use super::{discover_codegen_tools, parse_command_arg};
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_command_defaults_to_commit() {
        let args = argv(&["gunbc-codegen"]);
        assert_eq!(parse_command_arg(&args).unwrap(), "commit");
    }

    #[test]
    fn parse_command_accepts_single_positional() {
        let args = argv(&["gunbc-codegen", "rollback", "-n"]);
        assert_eq!(parse_command_arg(&args).unwrap(), "rollback");
    }

    #[test]
    fn parse_command_rejects_extra_positionals() {
        let args = argv(&["gunbc-codegen", "commit", "rollback"]);
        let err = parse_command_arg(&args).unwrap_err();
        assert!(err.contains("unexpected extra positional arguments"));
    }

    #[test]
    fn codegen_discovery_finds_expected_tools() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root should exist")
            .to_path_buf();
        let tools =
            discover_codegen_tools(&workspace_root).expect("DSL discovery should return tool defs");
        let names: BTreeSet<String> = tools.iter().map(|t| t.meta.tool_name.to_string()).collect();

        for required in ["bootstrap", "deps", "gist", "makegen", "pragma", "testgen"] {
            assert!(
                names.contains(required),
                "missing tool from DSL discovery: {}",
                required
            );
        }

        // gist.dag is a multi-entrypoint module and should be grouped as one
        // top-level tool with subcommands.
        let gist = tools
            .iter()
            .find(|t| t.meta.tool_name == "gist")
            .expect("gist tool should exist");
        let subcommands: BTreeSet<&str> =
            gist.subcommands.iter().map(|s| s.name.as_str()).collect();
        for subcommand in ["gist-diff", "gist-recent"] {
            assert!(
                subcommands.contains(subcommand),
                "missing gist subcommand from DSL discovery: {}",
                subcommand
            );
        }
    }
}
