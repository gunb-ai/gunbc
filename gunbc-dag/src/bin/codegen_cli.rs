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
//! This binary lives in gunbc-dag (not gunbc-codegen) and discovers
//! `#[tool_target(...)]` registrations directly from workspace source files.
//! The codegen library remains in core/codegen as a leaf crate.

#![deny(dead_code)]
use cargo_metadata::MetadataCommand;
use gunbc_cli::BinaryArgs;
use gunbc_codegen::{core_outputs, generate_cli_with_import, FileWriter, ToolDef};
use gunbc_dag::WorkspaceBinary;
use gunbc_exec::{print_attention, run_freshness_steps, AttentionLevel};
use gunbc_ir::resource::{
    check_manifest_freshness, codegen_resource_def, load_manifest_default,
    update_resource_manifest, FreshnessOptions, ManagedResource, ManifestEntry, ManifestFreshness,
    ManifestUpdateError, ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::transport::ci::{
    yaml_block, CacheConfig, CiRenderer, GitHubActionsProvider, GitLabCiProvider, RenderConfig,
};
use gunbc_ir::WorkspaceLayout;
use gunbc_lib_transport::TransportIo;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

// Force-link crates that register live test targets used by ci_live_test_secrets().
use gunbc_lib_llm_ops as _;

fn main() {
    let args: Vec<String> = env::args().collect();
    let parsed = match BinaryArgs::new().parse(&args) {
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

    // Secrets required by live flow tests (derived from testgen metadata).
    let ci_secrets: Vec<String> = gunbc_dag::ci::graph::ci_live_test_secrets()
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
    write!(yaml, "\n\nname: {}\n\n", config.workflow_name).unwrap();

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

    write!(
        yaml,
        "jobs:\n  {}:\n    runs-on: {}\n    timeout-minutes: {}\n    steps:\n",
        config.workflow_name, config.runner.id, config.timeout_minutes,
    )
    .unwrap();

    if let Some(checkout) = &config.checkout {
        yaml.push_str("      - name: Checkout\n        uses: actions/checkout@v4\n");
        if let Some(depth) = checkout.fetch_depth {
            write!(yaml, "        with:\n          fetch-depth: {}\n", depth).unwrap();
        }
        yaml.push('\n');
    }

    yaml.push_str("      - name: Setup Rust\n        uses: dtolnay/rust-toolchain@stable\n\n");

    if let Some(cache) = &config.cache {
        yaml.push_str("      - name: Cache Cargo\n        uses: actions/cache@v4\n        with:\n");
        yaml_block(&mut yaml, "          path: |", &cache.paths, |p| {
            format!("            {}", p)
        });
        writeln!(yaml, "          key: {}", cache.key).unwrap();
        yaml_block(
            &mut yaml,
            "          restore-keys: |",
            &cache.restore_keys,
            |k| format!("            {}", k),
        );
    }

    write!(
        yaml,
        "      - name: Run CI Pipeline\n        run: {}\n",
        config.tool.command(),
    )
    .unwrap();

    // Step-level env: CARGO_INCREMENTAL overrides dtolnay/rust-toolchain's
    // CARGO_INCREMENTAL=0 (set via $GITHUB_ENV). Step-level env takes precedence.
    yaml.push_str("        env:\n");
    yaml.push_str("          CARGO_INCREMENTAL: \"1\"\n");
    for secret in &config.secrets_env {
        writeln!(yaml, "          {}: ${{{{ secrets.{} }}}}", secret, secret).unwrap();
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

    write!(
        yaml,
        "{}:\n  stage: ci\n  script:\n    - {}\n",
        config.workflow_name,
        config.tool.command(),
    )
    .unwrap();

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

#[derive(Default)]
struct ParsedToolTarget {
    name: Option<String>,
    crate_name: Option<String>,
    description: Option<String>,
    builder: Option<String>,
    builder_args: Option<String>,
    custom_import: Option<String>,
    success_port: Option<String>,
    mock_spec: Option<String>,
    entrypoints_json: Option<String>,
    package: Option<String>,
    binary: Option<String>,
    has_invocation: bool,
    returns_result: bool,
    enable_step_mode: bool,
    skip: bool,
}

fn extract_attr_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(',');
    let needle = format!("{key} = \"");
    if let Some(after) = trimmed.strip_prefix(&needle) {
        let end = after.find('"')?;
        return Some(after[..end].to_string());
    }
    let raw_needle = format!("{key} = r#\"");
    if let Some(after) = trimmed.strip_prefix(&raw_needle) {
        let end = after.find("\"#")?;
        return Some(after[..end].to_string());
    }
    None
}

fn parse_tool_target_block(
    path: &Path,
    start_line: usize,
    lines: &[&str],
) -> Result<Option<ToolDef>, String> {
    let mut parsed = ParsedToolTarget::default();

    for line in lines {
        let trimmed = line.trim();

        if let Some(v) = extract_attr_value(trimmed, "name") {
            parsed.name = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "crate_name") {
            parsed.crate_name = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "description") {
            parsed.description = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "builder") {
            parsed.builder = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "args") {
            parsed.builder_args = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "import") {
            parsed.custom_import = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "success_port") {
            parsed.success_port = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "mock_spec") {
            parsed.mock_spec = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "entrypoints") {
            parsed.entrypoints_json = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "package") {
            parsed.package = Some(v);
        }
        if let Some(v) = extract_attr_value(trimmed, "binary") {
            parsed.binary = Some(v);
        }
        if trimmed.contains("has_invocation") && !trimmed.contains("has_invocation =") {
            parsed.has_invocation = true;
        }
        if trimmed.contains("returns_result") && !trimmed.contains("returns_result =") {
            parsed.returns_result = true;
        }
        if trimmed.contains("enable_step_mode") && !trimmed.contains("enable_step_mode =") {
            parsed.enable_step_mode = true;
        }
        if trimmed.contains("skip") && !trimmed.contains("skip =") {
            parsed.skip = true;
        }
    }

    if parsed.skip {
        return Ok(None);
    }

    let name = parsed.name.ok_or_else(|| {
        format!(
            "{}:{}: tool_target missing required field `name`",
            path.display(),
            start_line
        )
    })?;
    let crate_name = parsed.crate_name.ok_or_else(|| {
        format!(
            "{}:{}: tool_target missing required field `crate_name`",
            path.display(),
            start_line
        )
    })?;
    let description = parsed.description.ok_or_else(|| {
        format!(
            "{}:{}: tool_target missing required field `description`",
            path.display(),
            start_line
        )
    })?;
    let builder = parsed.builder.ok_or_else(|| {
        format!(
            "{}:{}: tool_target missing required field `builder`",
            path.display(),
            start_line
        )
    })?;

    let mut tool = ToolDef::new(
        crate_name.clone(),
        name.clone(),
        description.clone(),
        builder.clone(),
        parsed.builder_args.clone().unwrap_or_default(),
    );

    if parsed.returns_result {
        tool = tool.returns_result();
    }
    if let Some(port) = parsed.success_port {
        tool = tool.check_success(port);
    }
    if parsed.enable_step_mode {
        tool = tool.enable_step_mode();
    }
    if let Some(import) = parsed.custom_import {
        tool = tool.import(import);
    }
    if let Some(mock_spec) = parsed.mock_spec {
        tool = tool.mock_spec_call(mock_spec);
    }
    if let Some(entrypoints) = parsed.entrypoints_json {
        if !entrypoints.is_empty() {
            tool = tool.entrypoints_json(&entrypoints);
        }
    }
    if parsed.has_invocation {
        let binary_component = parsed.binary.as_deref().unwrap_or(name.as_str());
        let invocation = match parsed.package.as_deref() {
            Some(pkg) if parsed.binary.is_some() || pkg != name => {
                gunbc_ir::cargo::CargoInvocation::composed(binary_component, pkg)
            }
            _ => gunbc_ir::cargo::CargoInvocation::standalone(binary_component),
        };
        tool = tool.invocation(invocation);
    }

    Ok(Some(tool))
}

fn parse_tool_defs_from_file(path: &Path, content: &str) -> Result<Vec<ToolDef>, String> {
    let mut in_tool_target = false;
    let mut attr_start_line = 0usize;
    let mut block_lines: Vec<&str> = Vec::new();
    let mut tool_defs = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if !in_tool_target && trimmed.starts_with("#[") && trimmed.contains("tool_target(") {
            in_tool_target = true;
            attr_start_line = idx + 1;
            block_lines.clear();
        }
        if in_tool_target {
            block_lines.push(line);
            if trimmed.contains(")]") {
                if let Some(tool) = parse_tool_target_block(path, attr_start_line, &block_lines)? {
                    tool_defs.push(tool);
                }
                in_tool_target = false;
                block_lines.clear();
            }
        }
    }

    if in_tool_target {
        return Err(format!(
            "{}:{}: unterminated tool_target attribute block",
            path.display(),
            attr_start_line
        ));
    }

    Ok(tool_defs)
}

#[allow(clippy::disallowed_methods)] // Build-time source discovery for generator tooling.
fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir)
            .map_err(|e| format!("failed to read source discovery dir {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                format!(
                    "failed to read source discovery entry in {}: {e}",
                    dir.display()
                )
            })?;
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if matches!(name, "target" | "buck-out" | ".git") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

#[allow(clippy::disallowed_methods)] // Build-time source discovery for generator tooling.
fn discover_tool_defs_from_workspace_sources(
    workspace_root: &Path,
) -> Result<Vec<ToolDef>, String> {
    let mut by_name: BTreeMap<String, ToolDef> = BTreeMap::new();
    for path in collect_rust_files(workspace_root)? {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        for tool in parse_tool_defs_from_file(&path, &content)? {
            let name = tool.meta.tool_name.to_string();
            if let Some(prev) = by_name.insert(name.clone(), tool) {
                return Err(format!(
                    "duplicate tool_target name `{}` discovered (existing crate `{}`, new file `{}`)",
                    name,
                    prev.meta.crate_name,
                    path.display()
                ));
            }
        }
    }
    if by_name.is_empty() {
        return Err("no #[tool_target] registrations discovered from source".to_string());
    }
    Ok(by_name.into_values().collect())
}

#[allow(clippy::disallowed_methods)] // Build-time DSL module discovery (not runtime I/O)
fn discover_dsl_module_names(root: &Path, module_kind: &str) -> Result<BTreeSet<String>, String> {
    let entries = fs::read_dir(root).map_err(|e| {
        format!(
            "failed to read DSL {module_kind} discovery root {}: {e}",
            root.display()
        )
    })?;
    let mut modules = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "failed to read DSL {module_kind} entry in {}: {e}",
                root.display()
            )
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("dag") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| {
                format!(
                    "failed to parse UTF-8 module stem for DSL {module_kind} file {}",
                    path.display()
                )
            })?;
        modules.insert(stem.to_string());
    }
    Ok(modules)
}

fn validate_required_dsl_modules_for_codegen(
    tool_modules: &BTreeSet<String>,
    pipeline_modules: &BTreeSet<String>,
) -> Result<(), String> {
    // Derive required tool modules from the tool registry (dsl_module field)
    // and WorkspaceBinary (which covers manual/internal tools without registrations).
    let registry_modules: BTreeSet<&str> = gunbc_tool_registry::dsl_module_to_targets()
        .keys()
        .copied()
        .collect();
    let binary_tool_modules: BTreeSet<&str> = WorkspaceBinary::all()
        .iter()
        .copied()
        .filter(|binary| binary.is_dsl_tool_module())
        .map(WorkspaceBinary::tool_name)
        .collect();

    // Required tools = union of registry dsl_module names and workspace binary tool modules.
    let required_tools: BTreeSet<&str> = registry_modules.union(&binary_tool_modules).copied().collect();

    // Required pipelines: workspace binaries that map to DSL pipeline modules.
    let required_pipelines: BTreeSet<&str> = WorkspaceBinary::all()
        .iter()
        .copied()
        .filter(|binary| binary.is_dsl_pipeline_module())
        .map(WorkspaceBinary::tool_name)
        .collect();

    let missing_tools: Vec<&str> = required_tools
        .iter()
        .copied()
        .filter(|name| !tool_modules.contains(*name))
        .collect();
    let missing_pipelines: Vec<&str> = required_pipelines
        .iter()
        .copied()
        .filter(|name| !pipeline_modules.contains(*name))
        .collect();

    if missing_tools.is_empty() && missing_pipelines.is_empty() {
        return Ok(());
    }

    let mut parts = Vec::new();
    if !missing_tools.is_empty() {
        parts.push(format!("tools: {}", missing_tools.join(", ")));
    }
    if !missing_pipelines.is_empty() {
        parts.push(format!("pipelines: {}", missing_pipelines.join(", ")));
    }
    Err(format!(
        "missing required DSL modules for codegen discovery: {}",
        parts.join("; ")
    ))
}

fn validate_codegen_dsl_coverage(
    tools: &[ToolDef],
    tool_modules: &BTreeSet<String>,
    pipeline_modules: &BTreeSet<String>,
) -> Result<(), String> {
    // Derive module→targets mapping from the tool registry's dsl_module field.
    // Modules not in this map must be explicitly known as workspace-binary
    // modules; otherwise they are unmapped and should fail closed.
    let module_to_targets = gunbc_tool_registry::dsl_module_to_targets();
    let known_tool_modules: BTreeSet<&str> = module_to_targets
        .keys()
        .copied()
        .chain(
            WorkspaceBinary::all()
                .iter()
                .copied()
                .filter(|binary| binary.is_dsl_tool_module())
                .map(WorkspaceBinary::tool_name),
        )
        .collect();
    let known_pipeline_modules: BTreeSet<&str> = WorkspaceBinary::all()
        .iter()
        .copied()
        .filter(|binary| binary.is_dsl_pipeline_module())
        .map(WorkspaceBinary::tool_name)
        .collect();

    let unknown_tools: Vec<String> = tool_modules
        .iter()
        .filter(|module| !known_tool_modules.contains(module.as_str()))
        .cloned()
        .collect();
    let unknown_pipelines: Vec<String> = pipeline_modules
        .iter()
        .filter(|module| !known_pipeline_modules.contains(module.as_str()))
        .cloned()
        .collect();
    if !unknown_tools.is_empty() || !unknown_pipelines.is_empty() {
        let mut parts = Vec::new();
        if !unknown_tools.is_empty() {
            parts.push(format!("unmapped DSL tool modules: {}", unknown_tools.join(", ")));
        }
        if !unknown_pipelines.is_empty() {
            parts.push(format!(
                "unmapped DSL pipeline modules: {}",
                unknown_pipelines.join(", ")
            ));
        }
        return Err(format!("codegen DSL coverage validation failed: {}", parts.join("; ")));
    }

    let tool_name_set: BTreeSet<&str> = tools
        .iter()
        .map(|tool| tool.meta.tool_name.as_ref())
        .collect();

    let mut missing_targets = Vec::new();
    for module in tool_modules {
        if let Some(targets) = module_to_targets.get(module.as_str()) {
            for target in targets {
                if !tool_name_set.contains(target) {
                    missing_targets.push(format!("{module}->{target}"));
                }
            }
        }
    }

    if missing_targets.is_empty() {
        return Ok(());
    }

    Err(format!(
        "codegen DSL coverage validation failed: missing generated targets for mapped DSL modules: {}",
        missing_targets.join(", ")
    ))
}

fn discover_codegen_tools(workspace_root: &Path) -> Result<Vec<ToolDef>, String> {
    let tools = discover_tool_defs_from_workspace_sources(workspace_root)?;
    let tool_modules = discover_dsl_module_names(&workspace_root.join("dsl/tools"), "tool")?;
    let pipeline_modules =
        discover_dsl_module_names(&workspace_root.join("dsl/pipelines"), "pipeline")?;
    validate_required_dsl_modules_for_codegen(&tool_modules, &pipeline_modules)?;
    validate_codegen_dsl_coverage(&tools, &tool_modules, &pipeline_modules)?;
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

    paths
        .iter()
        .all(|path| io.file_exists(path).unwrap_or(false))
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
    use super::{
        discover_codegen_tools, parse_command_arg, parse_tool_defs_from_file,
        validate_codegen_dsl_coverage, WorkspaceBinary,
    };
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

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
    fn parse_tool_target_block_supports_raw_entrypoints_and_flags() {
        let attr = "tool_target";
        let src_template = r##"
#[gunbc_tool_registry_macros::__ATTR__(
    name = "sample",
    crate_name = "gunbc-sample",
    description = "Sample tool",
    builder = "build_sample_graph",
    args = "Mode::Default",
    import = "use gunbc_sample::build_sample_graph;",
    success_port = "ok",
    mock_spec = "gunbc_sample::graph_mock::sample_mock_spec()",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path","make_var":"REPO"}]"#,
    package = "sample",
    binary = "sample",
    has_invocation,
    returns_result,
    enable_step_mode
)]
pub fn sample_tool() {}
"##;
        let src = src_template.replace("__ATTR__", attr);
        let defs = parse_tool_defs_from_file(Path::new("sample.rs"), &src)
            .expect("tool_target parser should succeed");
        assert_eq!(defs.len(), 1);
        let tool = &defs[0];
        assert_eq!(tool.meta.tool_name, "sample");
        assert!(tool.meta.returns_result);
        assert!(tool.meta.enable_step_mode);
        assert_eq!(tool.meta.success_port.as_deref(), Some("ok"));
        assert_eq!(tool.entrypoints.len(), 1);
        assert_eq!(tool.entrypoints[0].port_name, "repo_path");
        assert!(tool.invocation.is_some());
    }

    #[test]
    fn codegen_source_discovery_finds_expected_tools() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root should exist")
            .to_path_buf();
        let tools = discover_codegen_tools(&workspace_root)
            .expect("source discovery should return tool defs");
        let names: BTreeSet<String> = tools.iter().map(|t| t.meta.tool_name.to_string()).collect();

        for required in [
            "bootstrap",
            "clippy",
            "deps",
            "gist",
            "gist-diff",
            "gist-recent",
            "makegen",
            "dag-viz",
            "dag-viz-diff",
            "dag-viz-recent",
            "dag-snapshot",
            "review",
        ] {
            assert!(
                names.contains(required),
                "missing tool target from source discovery: {}",
                required
            );
        }
    }

    #[test]
    fn codegen_dsl_coverage_rejects_unknown_tool_module() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root should exist")
            .to_path_buf();
        let tools = discover_codegen_tools(&workspace_root)
            .expect("source discovery should return tool defs");

        let mut tool_modules: BTreeSet<String> = [
            "build",
            "bootstrap",
            "clippy",
            "codegen",
            "dag_viz",
            "deps",
            "docgen",
            "gist",
            "makegen",
            "pragma",
            "testgen",
            "unknown_new_tool",
        ]
        .into_iter()
        .map(|name| name.to_string())
        .collect();
        // Ensure deterministic assertion error path.
        tool_modules.insert("unknown_new_tool".to_string());
        let pipeline_modules: BTreeSet<String> = WorkspaceBinary::all()
            .iter()
            .copied()
            .filter(|binary| binary.is_dsl_pipeline_module())
            .map(|binary| binary.tool_name().to_string())
            .collect();

        let err = validate_codegen_dsl_coverage(&tools, &tool_modules, &pipeline_modules)
            .expect_err("unknown tool module must fail coverage validation");
        assert!(err.contains("unmapped DSL tool modules"));
        assert!(err.contains("unknown_new_tool"));
    }
}
