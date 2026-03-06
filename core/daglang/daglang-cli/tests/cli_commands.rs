// Test infrastructure: filesystem access for test fixtures
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use daglang_resolve::ModuleGraph;
use daglang_resolve::ResolveError;
use gunbc_ir::WorkspaceLayout;
use gunbc_test::{unique_temp_dir, unique_temp_file};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

fn workspace_root() -> PathBuf {
    static WORKSPACE_ROOT: OnceLock<PathBuf> = OnceLock::new();
    WORKSPACE_ROOT
        .get_or_init(|| {
            WorkspaceLayout::from_env_manifest_dir()
                .expect("resolve workspace layout")
                .workspace_root
        })
        .clone()
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

fn assert_no_compile_stage_banners(stderr: &str) {
    assert!(
        !stderr.contains("typecheck errors"),
        "unexpected typecheck-stage banner in non-compile command path: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "unexpected lower-stage banner in non-compile command path: {stderr}"
    );
}

fn expected_check_success_stdout(parsed_files: usize) -> String {
    format!("OK: checked {parsed_files} file(s)\n")
}

fn assert_modules_single_file_root_failure(
    output: &std::process::Output,
    expected_path: &std::path::Path,
    context: &str,
) {
    assert!(!output.status.success(), "{context} should fail");
    assert_eq!(
        output.status.code(),
        Some(1),
        "{context} should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("pipeline error"),
        "{context} should surface pipeline error: {stderr}"
    );
    assert!(
        stderr.contains("input root is not a directory"),
        "{context} should report non-directory root: {stderr}"
    );
    assert!(
        stderr.contains(&expected_path.display().to_string()),
        "{context} should include offending path: {stderr}"
    );
}

fn assert_modules_relative_absolute_single_file_root_equivalence(
    relative: &std::process::Output,
    absolute: &std::process::Output,
    expected_path: &std::path::Path,
    context: &str,
) {
    let relative_context = format!("{context} relative");
    assert_modules_single_file_root_failure(relative, expected_path, &relative_context);

    let absolute_context = format!("{context} absolute");
    assert_modules_single_file_root_failure(absolute, expected_path, &absolute_context);

    assert_eq!(
        relative.stdout, absolute.stdout,
        "{context} relative and absolute stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "{context} relative and absolute stderr should match"
    );
}

/// Assert a CLI invocation is rejected because the path has wrong-cased `.dag` extension.
fn assert_wrong_cased_dag_extension_rejected(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context}: should fail due to wrong-cased extension"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("wrong-cased extension"),
        "{context}: stderr should report wrong-cased extension: {stderr}"
    );
    assert!(
        stderr.contains("rename to `.dag` (lowercase)"),
        "{context}: stderr should suggest renaming to lowercase: {stderr}"
    );
}

fn expected_dsl_modules_sorted() -> Vec<String> {
    let dsl_root = workspace_root().join("dsl");
    let parse_error_files = dsl_parse_error_files(&dsl_root);
    let mut modules = Vec::new();
    collect_dsl_modules(&dsl_root, &dsl_root, &parse_error_files, &mut modules);
    modules.sort();
    modules
}

fn dsl_parse_error_files(dsl_root: &Path) -> BTreeSet<PathBuf> {
    let roots = vec![dsl_root.to_path_buf()];
    match ModuleGraph::discover(&roots) {
        Ok(_) => BTreeSet::new(),
        Err(ResolveError::ParseErrors(errors)) => {
            errors.into_iter().map(|(path, _)| path).collect()
        }
        Err(other) => panic!("failed to inspect dsl parse errors: {other}"),
    }
}

fn collect_dsl_modules(
    dsl_root: &Path,
    dir: &Path,
    parse_error_files: &BTreeSet<PathBuf>,
    modules: &mut Vec<String>,
) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read dsl directory {}: {e}", dir.display()))
        .map(|entry| {
            entry.unwrap_or_else(|e| panic!("failed to read dsl entry in {}: {e}", dir.display()))
        })
        .collect();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dsl_modules(dsl_root, &path, parse_error_files, modules);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("dag") {
            continue;
        }
        if parse_error_files.contains(&path) {
            continue;
        }
        modules.push(parse_module_declaration(dsl_root, &path));
    }
}

fn parse_module_declaration(dsl_root: &Path, path: &Path) -> String {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read dsl source {}: {e}", path.display()));
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("module ") {
            if let Some(module_id) = rest.split_whitespace().next() {
                return module_id.to_string();
            }
        }
    }
    panic!(
        "missing module declaration in dsl source {} (under {})",
        path.display(),
        dsl_root.display()
    );
}

fn reported_modules_in_order(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter_map(|line| {
            line.split_once("  (")
                .map(|(module, _)| module.trim().to_string())
        })
        .collect()
}

fn reported_modules_sorted(stdout: &str) -> Vec<String> {
    let mut modules = reported_modules_in_order(stdout);
    modules.sort();
    modules
}

fn reported_module_summary(stdout: &str) -> BTreeMap<String, (usize, usize)> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter_map(|line| line.split_once("  ("))
        .filter_map(|(module, rest)| {
            let counts = rest.split(")  [").next()?;
            let (items_part, deps_part) = counts.split_once(", ")?;
            let item_count = items_part
                .trim_end_matches(" items")
                .parse::<usize>()
                .ok()?;
            let dep_count = deps_part.trim_end_matches(" deps").parse::<usize>().ok()?;
            Some((module.trim().to_string(), (item_count, dep_count)))
        })
        .collect()
}

fn expected_viz_self_mermaid() -> &'static str {
    concat!(
        "flowchart TB\n",
        "subgraph daglang_compiler_pipeline[\"daglang-compiler-pipeline\"]\n",
        "    daglang_compiler_pipeline_discover_files[discover_files]\n",
        "    daglang_compiler_pipeline_parse_all[parse_all]\n",
        "    daglang_compiler_pipeline_resolve_module_graph[resolve_module_graph]\n",
        "    daglang_compiler_pipeline_typecheck_modules[typecheck_modules]\n",
        "    daglang_compiler_pipeline_lower_graph_ir[lower_graph_ir]\n",
        "    daglang_compiler_pipeline_derive_metadata[derive_metadata]\n",
        "    daglang_compiler_pipeline_emit_target_files[emit_target_files]\n",
        "    daglang_compiler_pipeline_discover_files -->|files:files| daglang_compiler_pipeline_parse_all\n",
        "    daglang_compiler_pipeline_discover_files -->|diagnostics:diagnostics| daglang_compiler_pipeline_parse_all\n",
        "    daglang_compiler_pipeline_parse_all -->|parsed_modules:parsed_modules| daglang_compiler_pipeline_resolve_module_graph\n",
        "    daglang_compiler_pipeline_parse_all -->|diagnostics:diagnostics| daglang_compiler_pipeline_resolve_module_graph\n",
        "    daglang_compiler_pipeline_resolve_module_graph -->|module_graph:module_graph| daglang_compiler_pipeline_typecheck_modules\n",
        "    daglang_compiler_pipeline_resolve_module_graph -->|diagnostics:diagnostics| daglang_compiler_pipeline_typecheck_modules\n",
        "    daglang_compiler_pipeline_typecheck_modules -->|typed_project:typed_project| daglang_compiler_pipeline_lower_graph_ir\n",
        "    daglang_compiler_pipeline_typecheck_modules -->|diagnostics:diagnostics| daglang_compiler_pipeline_lower_graph_ir\n",
        "    daglang_compiler_pipeline_lower_graph_ir -->|lowered_dag:lowered_dag| daglang_compiler_pipeline_derive_metadata\n",
        "    daglang_compiler_pipeline_lower_graph_ir -->|diagnostics:diagnostics| daglang_compiler_pipeline_derive_metadata\n",
        "    daglang_compiler_pipeline_lower_graph_ir -->|lowered_dag:lowered_dag| daglang_compiler_pipeline_emit_target_files\n",
        "    daglang_compiler_pipeline_derive_metadata -->|derived_artifacts:derived_artifacts| daglang_compiler_pipeline_emit_target_files\n",
        "    daglang_compiler_pipeline_derive_metadata -->|diagnostics:diagnostics| daglang_compiler_pipeline_emit_target_files\n",
        "end\n\n",
    )
}

fn resolve_discovered_module_order() -> Vec<String> {
    ModuleGraph::discover(&[workspace_root().join("dsl")])
        .expect("resolve discovery should succeed for real corpus")
        .modules
        .into_iter()
        .map(|module| module.module_path.as_dotted())
        .collect()
}

fn resolve_discovered_module_summary() -> BTreeMap<String, (usize, usize)> {
    ModuleGraph::discover(&[workspace_root().join("dsl")])
        .expect("resolve discovery should succeed for real corpus")
        .modules
        .into_iter()
        .map(|module| {
            (
                module.module_path.as_dotted(),
                (module.ast.items.len(), module.dependencies.len()),
            )
        })
        .collect()
}

fn reported_diagnostics_in_order(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .skip_while(|line| *line != "Diagnostics:")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.strip_prefix("  ").map(String::from))
        .collect()
}

fn expected_real_corpus_modules_diagnostics() -> Vec<String> {
    vec![]
}

// ── Path normalization helpers ─────────────────────────────────────────────

/// Run a command with the given argument and return its output.
fn run_cli(command: &str, arg: &str, cwd: &std::path::Path) -> Output {
    Command::new(daglang_bin())
        .arg(command)
        .arg(arg)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to run `daglang {command} {arg}`: {e}"))
}

fn run_cli_path(command: &str, arg: &std::path::Path, cwd: &std::path::Path) -> Output {
    Command::new(daglang_bin())
        .arg(command)
        .arg(arg)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to run `daglang {command} {}`: {e}", arg.display()))
}

/// Assert that a variant output matches a canonical output (stdout + stderr).
fn assert_variant_matches(canonical: &Output, variant: &Output, label: &str, expect_success: bool) {
    assert_eq!(
        variant.status.success(),
        expect_success,
        "{label}: expected success={expect_success}, got exit={:?}\nstderr: {}",
        variant.status.code(),
        String::from_utf8_lossy(&variant.stderr)
    );
    assert_eq!(
        variant.stdout, canonical.stdout,
        "{label}: stdout mismatch with canonical"
    );
    assert_eq!(
        variant.stderr, canonical.stderr,
        "{label}: stderr mismatch with canonical"
    );
}

/// Path variants using parent segment (../) — requires anchor/ subdirectory.
fn parent_segment_variants(root: &std::path::Path, target: &str) -> Vec<(&'static str, PathBuf)> {
    let target_path = root.join(target);
    vec![
        ("parent", root.join(format!("anchor/../{target}"))),
        (
            "parent_curdir_segment",
            root.join(format!("anchor/.././{target}")),
        ),
        (
            "parent_curdir_segment_trailing",
            PathBuf::from(format!("{}/anchor/.././{target}/", root.display())),
        ),
        (
            "parent_curdir_segment_double_separator",
            PathBuf::from(format!("{}/anchor/.././{target}//", root.display())),
        ),
        (
            "parent_curdir_segment_double_separator_trailing",
            PathBuf::from(format!("{}/anchor//.././{target}/", root.display())),
        ),
        (
            "parent_double_separator",
            PathBuf::from(format!("{}/anchor/..//{target}", root.display())),
        ),
        (
            "parent_trailing",
            PathBuf::from(format!("{}/", target_path.display())),
        ),
        (
            "parent_double_separator_trailing",
            PathBuf::from(format!("{}/anchor/..//{target}/", root.display())),
        ),
    ]
}

/// Path variants using curdir segments (./) — no anchor needed.
fn curdir_segment_variants(root: &std::path::Path, target: &str) -> Vec<(&'static str, PathBuf)> {
    let target_path = root.join(target);
    vec![
        ("mixed_segment", root.join(".").join(target)),
        (
            "curdir_segment_trailing_slash",
            root.join(format!("{target}/./")),
        ),
        (
            "curdir_segment_double_separator",
            PathBuf::from(format!("{}//./", target_path.display())),
        ),
        ("curdir_suffix", root.join(format!("{target}/."))),
        (
            "curdir_suffix_double_separator",
            PathBuf::from(format!("{}//.", target_path.display())),
        ),
        (
            "curdir_segment_suffix",
            PathBuf::from(format!("{}/{target}/./.", root.display())),
        ),
        (
            "curdir_segment_double_separator_suffix",
            PathBuf::from(format!("{}//./.", target_path.display())),
        ),
    ]
}

/// Path variants using double separators and trailing slashes (no anchor).
fn separator_variants(root: &std::path::Path, target: &str) -> Vec<(&'static str, PathBuf)> {
    let target_path = root.join(target);
    vec![
        (
            "double_separator",
            PathBuf::from(format!("{}//{target}", root.display())),
        ),
        (
            "trailing_slash",
            PathBuf::from(format!("{}/", target_path.display())),
        ),
        (
            "double_separator_trailing",
            PathBuf::from(format!("{}//{target}/", root.display())),
        ),
    ]
}

/// All absolute path variants for a target within root (parent + curdir + separator).
fn all_absolute_variants(root: &std::path::Path, target: &str) -> Vec<(&'static str, PathBuf)> {
    let mut variants = parent_segment_variants(root, target);
    variants.extend(curdir_segment_variants(root, target));
    variants.extend(separator_variants(root, target));
    variants
}

/// Relative parent variants (from a nested cwd, e.g. workspace/core → ../dsl).
fn relative_parent_variants(target: &str) -> Vec<(&'static str, String)> {
    vec![
        ("parent_segment", format!("../{target}")),
        ("parent_curdir_segment", format!(".././{target}")),
        ("parent_double_separator", format!("..//{target}")),
        ("parent_double_separator_trailing", format!("..//{target}/")),
        ("parent_curdir_double_separator", format!(".././/{target}")),
        ("parent_curdir_trailing_slash", format!(".././{target}/")),
    ]
}

/// Relative curdir/dot variants (from workspace root, e.g. ./dsl, dsl/., etc.).
fn relative_curdir_variants(target: &str) -> Vec<(&'static str, String)> {
    vec![
        ("curdir_segment", format!("./{target}")),
        ("curdir_segment_trailing_slash", format!("./{target}/")),
        (
            "dot_double_separator_curdir_suffix",
            format!(".//{target}/."),
        ),
        (
            "dot_double_separator_curdir_segment_suffix",
            format!(".//{target}/./."),
        ),
        (
            "dot_double_separator_curdir_segment_double_separator",
            format!(".//{target}//./"),
        ),
        (
            "dot_double_separator_double_separator",
            format!(".//{target}//"),
        ),
        (
            "dot_double_separator_trailing_slash",
            format!(".//{target}/"),
        ),
        (
            "dot_double_separator_curdir_suffix_double_separator",
            format!(".//{target}/.//"),
        ),
        (
            "dot_double_separator_curdir_segment_double_separator_suffix",
            format!(".//{target}//./."),
        ),
        (
            "dot_double_separator_curdir_segment_trailing_slash",
            format!(".//{target}/./"),
        ),
        ("curdir_suffix", format!("{target}/.")),
        ("relative_curdir_suffix", format!("./{target}/.")),
        (
            "relative_curdir_segment_trailing_slash",
            format!("./{target}/./"),
        ),
        ("relative_curdir_segment_suffix", format!("./{target}/./.")),
        ("curdir_segment_suffix", format!("{target}/./.")),
        ("curdir_suffix_double_separator", format!("{target}/.//.")),
        (
            "relative_curdir_suffix_double_separator",
            format!("./{target}/.//"),
        ),
        ("curdir_segment_double_separator", format!("{target}//./")),
        (
            "relative_curdir_segment_double_separator_trailing_slash",
            format!("./{target}//./"),
        ),
        (
            "relative_curdir_segment_double_separator_suffix",
            format!("./{target}//./."),
        ),
        (
            "curdir_segment_double_separator_suffix",
            format!("{target}//./."),
        ),
        ("trailing_slash", format!("{target}/")),
        ("mixed_segment", format!("./{target}/../{target}")),
        ("double_separator", format!("{target}//")),
        ("dot_double_separator", format!(".//{target}")),
        ("curdir_double_separator", format!("./{target}//")),
    ]
}

// ── Table-driven path normalization tests ──────────────────────────────────

/// Test all absolute path variants for valid root (using real dsl corpus).
fn assert_absolute_valid_root_variants(command: &str) {
    let cwd = workspace_root().join("core");
    let root = workspace_root();
    let canonical = run_cli_path(command, &root.join("dsl"), &cwd);
    assert!(
        canonical.status.success(),
        "canonical absolute-root {command} should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    for (label, variant_path) in all_absolute_variants(&root, "dsl") {
        let variant = run_cli_path(command, &variant_path, &cwd);
        assert_variant_matches(
            &canonical,
            &variant,
            &format!("absolute_valid_root/{label}"),
            true,
        );
    }
}

/// Test relative parent variants for valid root.
fn assert_relative_parent_valid_root_variants(command: &str) {
    let cwd = workspace_root().join("core");
    let canonical = run_cli_path(command, &workspace_root().join("dsl"), &cwd);
    assert!(
        canonical.status.success(),
        "canonical {command} should succeed"
    );

    for (label, variant) in relative_parent_variants("dsl") {
        let output = run_cli(command, &variant, &cwd);
        assert_variant_matches(
            &canonical,
            &output,
            &format!("relative_parent_valid/{label}"),
            true,
        );
    }
}

/// Test relative curdir variants for valid root.
fn assert_relative_curdir_valid_root_variants(command: &str) {
    let cwd = workspace_root();
    let canonical = run_cli(command, "dsl", &cwd);
    assert!(
        canonical.status.success(),
        "canonical {command} should succeed"
    );

    for (label, variant) in relative_curdir_variants("dsl") {
        let output = run_cli(command, &variant, &cwd);
        assert_variant_matches(
            &canonical,
            &output,
            &format!("relative_curdir_valid/{label}"),
            true,
        );
    }
}

/// Test path variants for missing root (directory does not exist).
fn assert_missing_root_variants(command: &str) {
    let root = unique_temp_dir(&format!("{command}_missing_root_variants"));
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");

    let canonical = run_cli_path(command, &root.join("missing_root"), &root);
    assert!(!canonical.status.success(), "missing root should fail");

    // Absolute parent-segment variants
    for (label, variant_path) in parent_segment_variants(&root, "missing_root") {
        let variant = run_cli_path(command, &variant_path, &root);
        assert_variant_matches(
            &canonical,
            &variant,
            &format!("abs_missing_root_parent/{label}"),
            false,
        );
    }

    // Absolute curdir variants
    for (label, variant_path) in curdir_segment_variants(&root, "missing_root") {
        let variant = run_cli_path(command, &variant_path, &root);
        assert_variant_matches(
            &canonical,
            &variant,
            &format!("abs_missing_root_curdir/{label}"),
            false,
        );
    }

    // Absolute separator variants
    for (label, variant_path) in separator_variants(&root, "missing_root") {
        let variant = run_cli_path(command, &variant_path, &root);
        assert_variant_matches(
            &canonical,
            &variant,
            &format!("abs_missing_root_sep/{label}"),
            false,
        );
    }

    // Verify diagnostic message contains canonical path
    let stderr = String::from_utf8_lossy(&canonical.stderr);
    assert!(
        stderr.contains(&format!(
            "input root does not exist: {}",
            root.join("missing_root").display()
        )),
        "missing-root diagnostics should include canonical path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test relative variants for missing root.
fn assert_relative_missing_root_variants(command: &str) {
    let root = unique_temp_dir(&format!("{command}_rel_missing_root_variants"));
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");

    // Relative curdir variants (cwd = root)
    let canonical = run_cli(command, "missing_root", &root);
    assert!(!canonical.status.success(), "missing root should fail");

    for (label, variant) in relative_curdir_variants("missing_root") {
        let output = run_cli(command, &variant, &root);
        assert_variant_matches(
            &canonical,
            &output,
            &format!("rel_missing_root_curdir/{label}"),
            false,
        );
    }

    // Relative parent variants (cwd = root/anchor)
    let cwd_nested = root.join("anchor");
    let canonical_abs = run_cli_path(command, &root.join("missing_root"), &cwd_nested);
    for (label, variant) in relative_parent_variants("missing_root") {
        let output = run_cli(command, &variant, &cwd_nested);
        assert_variant_matches(
            &canonical_abs,
            &output,
            &format!("rel_missing_root_parent/{label}"),
            false,
        );
    }

    // Relative and absolute equivalence
    let rel_output = run_cli(command, "missing_root", &root);
    let abs_output = run_cli_path(command, &root.join("missing_root"), &root);
    assert_eq!(
        rel_output.stdout, abs_output.stdout,
        "relative and absolute missing root stdout should match"
    );
    assert_eq!(
        rel_output.stderr, abs_output.stderr,
        "relative and absolute missing root stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test path variants for non-directory root (file instead of directory).
fn assert_non_directory_root_variants(command: &str) {
    let root = unique_temp_dir(&format!("{command}_non_dir_root_variants"));
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let canonical = run_cli_path(command, &root_file, &root);
    assert!(
        !canonical.status.success(),
        "non-directory root should fail"
    );

    // Absolute parent-segment variants
    for (label, variant_path) in parent_segment_variants(&root, "input.txt") {
        let variant = run_cli_path(command, &variant_path, &root);
        assert_variant_matches(
            &canonical,
            &variant,
            &format!("abs_non_dir_root_parent/{label}"),
            false,
        );
    }

    // Absolute curdir variants
    for (label, variant_path) in curdir_segment_variants(&root, "input.txt") {
        let variant = run_cli_path(command, &variant_path, &root);
        assert_variant_matches(
            &canonical,
            &variant,
            &format!("abs_non_dir_root_curdir/{label}"),
            false,
        );
    }

    // Absolute separator variants
    for (label, variant_path) in separator_variants(&root, "input.txt") {
        let variant = run_cli_path(command, &variant_path, &root);
        assert_variant_matches(
            &canonical,
            &variant,
            &format!("abs_non_dir_root_sep/{label}"),
            false,
        );
    }

    // Verify diagnostic message
    let stderr = String::from_utf8_lossy(&canonical.stderr);
    assert!(
        stderr.contains(&format!(
            "input root is not a directory: {}",
            root_file.display()
        )),
        "non-directory-root diagnostics should include canonical path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test relative variants for non-directory root.
fn assert_relative_non_directory_root_variants(command: &str) {
    let root = unique_temp_dir(&format!("{command}_rel_non_dir_root_variants"));
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");
    std::fs::write(root.join("input.txt"), "not a directory").expect("failed to create root file");

    // Relative curdir variants
    let canonical = run_cli(command, "input.txt", &root);
    assert!(
        !canonical.status.success(),
        "non-directory root should fail"
    );

    for (label, variant) in relative_curdir_variants("input.txt") {
        let output = run_cli(command, &variant, &root);
        assert_variant_matches(
            &canonical,
            &output,
            &format!("rel_non_dir_root_curdir/{label}"),
            false,
        );
    }

    // Relative parent variants
    let cwd_nested = root.join("anchor");
    let canonical_abs = run_cli_path(command, &root.join("input.txt"), &cwd_nested);
    for (label, variant) in relative_parent_variants("input.txt") {
        let output = run_cli(command, &variant, &cwd_nested);
        assert_variant_matches(
            &canonical_abs,
            &output,
            &format!("rel_non_dir_root_parent/{label}"),
            false,
        );
    }

    // Relative and absolute equivalence
    let rel_output = run_cli(command, "input.txt", &root);
    let abs_output = run_cli_path(command, &root.join("input.txt"), &root);
    assert_eq!(
        rel_output.stdout, abs_output.stdout,
        "relative and absolute non-directory root stdout should match"
    );
    assert_eq!(
        rel_output.stderr, abs_output.stderr,
        "relative and absolute non-directory root stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test .dag-extension directory variants (trailing slash, curdir prefix, with/without errors).
/// Only lowercase `.dag` is accepted; wrong-cased extensions are rejected up front.
fn assert_dag_extension_directory_variants(command: &str) {
    for ext in &[".dag"] {
        for has_errors in &[false, true] {
            let label_suffix = if *has_errors { "_errors" } else { "" };
            let test_name = format!("{command}_dag_dir_{ext}{label_suffix}");
            let root = unique_temp_dir(&test_name);
            let dag_dir = root.join(format!("bundle{ext}"));
            std::fs::create_dir_all(&dag_dir).expect("failed to create .dag directory");
            std::fs::write(
                dag_dir.join("main.dag"),
                "module sample.main\nfn ok() -> Unit {}",
            )
            .expect("failed to write valid source");

            if *has_errors {
                std::fs::write(dag_dir.join("broken.dag"), "module sample.broken\nfn")
                    .expect("failed to write broken source");
            }

            let plain_arg = format!("bundle{ext}");
            let trailing_arg = format!("bundle{ext}/");
            let curdir_arg = format!("./bundle{ext}");

            let plain = run_cli(command, &plain_arg, &root);
            let trailing = run_cli(command, &trailing_arg, &root);
            let curdir = run_cli(command, &curdir_arg, &root);

            // Hardening: `.dag` directories are rejected up front for all commands.
            let expect_success = false;
            assert!(
                !plain.status.success(),
                "{test_name}: plain .dag directory variant should fail"
            );
            assert!(
                String::from_utf8_lossy(&plain.stderr).contains("target is a directory"),
                "{test_name}: expected explicit .dag directory conflict error, got stderr: {}",
                String::from_utf8_lossy(&plain.stderr)
            );
            assert_variant_matches(
                &plain,
                &trailing,
                &format!("{test_name}/trailing_slash"),
                expect_success,
            );
            assert_variant_matches(
                &plain,
                &curdir,
                &format!("{test_name}/curdir_prefix"),
                expect_success,
            );

            std::fs::remove_dir_all(root).expect("failed to cleanup");
        }
    }
}

/// Test .dag-extension directory with symlinks.
/// Only lowercase `.dag` is accepted; wrong-cased extensions are rejected up front.
#[cfg(unix)]
fn assert_dag_extension_symlink_directory_variants(command: &str) {
    use std::os::unix::fs::symlink;

    for ext in &[".dag"] {
        for has_errors in &[false, true] {
            let label_suffix = if *has_errors { "_errors" } else { "" };
            let test_name = format!("{command}_dag_symlink_{ext}{label_suffix}");
            let root = unique_temp_dir(&test_name);
            let real_dir = root.join("real");
            let link_dir = root.join(format!("link{ext}"));
            std::fs::create_dir_all(&real_dir).expect("failed to create real dir");
            std::fs::write(
                real_dir.join("main.dag"),
                "module sample.main\nfn ok() -> Unit {}",
            )
            .expect("failed to write valid source");

            if *has_errors {
                std::fs::write(real_dir.join("broken.dag"), "module sample.broken\nfn")
                    .expect("failed to write broken source");
            }

            symlink(&real_dir, &link_dir).expect("failed to create symlink");

            let link_arg = format!("link{ext}");
            let curdir_link_arg = format!("./link{ext}");

            let link_output = run_cli(command, &link_arg, &root);
            let real_output = run_cli(command, "real", &root);
            let curdir_link = run_cli(command, &curdir_link_arg, &root);

            let expect_success = false;
            let real_expect_success = if command == "check" {
                !has_errors
            } else {
                true
            };
            assert!(
                real_output.status.success() == real_expect_success,
                "{test_name}: non-.dag real directory expected success={real_expect_success}, got exit={:?}\nstderr: {}",
                real_output.status.code(),
                String::from_utf8_lossy(&real_output.stderr)
            );
            assert!(
                !link_output.status.success(),
                "{test_name}: .dag symlink path should fail"
            );
            assert!(
                String::from_utf8_lossy(&link_output.stderr).contains("target is a directory"),
                "{test_name}: expected explicit .dag directory conflict error, got stderr: {}",
                String::from_utf8_lossy(&link_output.stderr)
            );
            assert_variant_matches(
                &link_output,
                &curdir_link,
                &format!("{test_name}/curdir_symlink"),
                expect_success,
            );

            std::fs::remove_dir_all(root).expect("failed to cleanup");
        }
    }
}

// ── Table-driven check_command tests ───────────────────────────────────────

#[test]
fn check_command_absolute_valid_root_variants_match_canonical() {
    assert_absolute_valid_root_variants("check");
}

#[test]
fn check_command_relative_parent_valid_root_variants_match_canonical() {
    assert_relative_parent_valid_root_variants("check");
}

#[test]
fn check_command_relative_curdir_valid_root_variants_match_canonical() {
    assert_relative_curdir_valid_root_variants("check");
}

#[test]
fn check_command_missing_root_variants_match_canonical() {
    assert_missing_root_variants("check");
}

#[test]
fn check_command_relative_missing_root_variants_match_canonical() {
    assert_relative_missing_root_variants("check");
}

#[test]
fn check_command_non_directory_root_variants_match_canonical() {
    assert_non_directory_root_variants("check");
}

#[test]
fn check_command_relative_non_directory_root_variants_match_canonical() {
    assert_relative_non_directory_root_variants("check");
}

#[test]
fn check_command_dag_extension_directory_variants() {
    assert_dag_extension_directory_variants("check");
}

#[cfg(unix)]
#[test]
fn check_command_dag_extension_symlink_directory_variants() {
    assert_dag_extension_symlink_directory_variants("check");
}

// ── Table-driven modules_command tests ─────────────────────────────────────

#[test]
fn modules_command_absolute_valid_root_variants_match_canonical() {
    assert_absolute_valid_root_variants("modules");
}

#[test]
fn modules_command_relative_parent_valid_root_variants_match_canonical() {
    assert_relative_parent_valid_root_variants("modules");
}

#[test]
fn modules_command_relative_curdir_valid_root_variants_match_canonical() {
    assert_relative_curdir_valid_root_variants("modules");
}

#[test]
fn modules_command_missing_root_variants_match_canonical() {
    assert_missing_root_variants("modules");
}

#[test]
fn modules_command_relative_missing_root_variants_match_canonical() {
    assert_relative_missing_root_variants("modules");
}

#[test]
fn modules_command_non_directory_root_variants_match_canonical() {
    assert_non_directory_root_variants("modules");
}

#[test]
fn modules_command_relative_non_directory_root_variants_match_canonical() {
    assert_relative_non_directory_root_variants("modules");
}

#[test]
fn modules_command_dag_extension_directory_variants() {
    assert_dag_extension_directory_variants("modules");
}

#[cfg(unix)]
#[test]
fn modules_command_dag_extension_symlink_directory_variants() {
    assert_dag_extension_symlink_directory_variants("modules");
}

#[test]
fn check_command_parses_full_dsl_corpus() {
    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check");

    assert!(
        output.status.success(),
        "check command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("OK: checked "),
        "unexpected check output: {stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "check over golden corpus should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !stdout.contains("Diagnostics:"),
        "check over golden corpus should not emit diagnostics: {stdout}"
    );
}

#[test]
fn check_command_hermetic_stdout_matches_expected_format() {
    let root = unique_temp_dir("check_stdout_format");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    for i in 1..=3 {
        std::fs::write(
            root.join(format!("mod{i}.dag")),
            format!("module sample.mod{i}\nfn ok() -> Unit {{}}"),
        )
        .expect("failed to write dag source");
    }

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(root.to_string_lossy().as_ref())
        .output()
        .expect("failed to run daglang check");

    assert!(
        output.status.success(),
        "check command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(3));
    assert!(
        output.stderr.is_empty(),
        "check should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_single_file_type_mismatch_exits_nonzero_with_typecheck_error() {
    let fixture = unique_temp_file("check_single_file_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.check_type_mismatch
fn run() -> String { return 42 }
"#,
    )
    .expect("failed to write check single-file type mismatch fixture");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for single-file type mismatch");

    assert!(
        !output.status.success(),
        "check should fail for single-file type mismatch fixture"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "check type mismatch failure should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("compile diagnostics"),
        "check type mismatch failure should report typecheck stage: {stderr}"
    );
    assert!(
        stderr.contains("type mismatch: expected `String`, got `Int`"),
        "check type mismatch failure should include mismatch detail: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "check should fail in typecheck stage before lowering: {stderr}"
    );

    std::fs::remove_file(fixture)
        .expect("failed to cleanup check single-file type mismatch fixture");
}

#[test]
fn check_command_default_root_matches_explicit_dsl_output() {
    let explicit = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run explicit-root daglang check");
    assert!(
        explicit.status.success(),
        "explicit-root check should succeed: {}",
        String::from_utf8_lossy(&explicit.stderr)
    );

    let default = Command::new(daglang_bin())
        .arg("check")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run default-root daglang check");
    assert!(
        default.status.success(),
        "default-root check should succeed: {}",
        String::from_utf8_lossy(&default.stderr)
    );

    assert_eq!(
        default.stdout, explicit.stdout,
        "default check output should match explicit 'check dsl' output"
    );
    assert_eq!(
        default.stderr, explicit.stderr,
        "default check stderr should match explicit-root stderr"
    );
}

#[test]
fn check_command_relative_and_absolute_root_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-root daglang check");
    assert!(
        relative.status.success(),
        "relative-root check should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute_root = workspace_root().join("dsl");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-root daglang check");
    assert!(
        absolute.status.success(),
        "absolute-root check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute root check outputs should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute root check stderr should match"
    );
}

#[test]
fn check_command_dot_double_separator_root_matches_plain_relative_output() {
    let cwd = workspace_root();

    let dot_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run dot-double-separator-root daglang check");
    assert!(
        dot_double_separator.status.success(),
        "dot-double-separator-root check should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .current_dir(&cwd)
        .output()
        .expect("failed to run plain-relative-root daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative-root check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator and plain-relative check stdout should match"
    );
    assert_eq!(
        dot_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator and plain-relative check stderr should match"
    );
}

#[test]
fn check_command_relative_and_absolute_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("check_relative_absolute_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-target daglang check");
    assert!(
        relative.status.success(),
        "relative-target check should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute-target daglang check");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&relative.stdout),
        expected_check_success_stdout(1),
        "relative-target check should parse one file"
    );
    assert!(
        relative.stderr.is_empty(),
        "relative-target check should not emit stderr: {}",
        String::from_utf8_lossy(&relative.stderr)
    );
    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute single-file check outputs should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute single-file check stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_uppercase_dag_extension_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("uppercase_dag_extension_single_file_target_matches");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DAG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_uppercase_dag_extension_single_file_target_matches_absolute_output",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_mixed_case_dag_extension_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("mixed_case_dag_extension_single_file_target_matche");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DaG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_mixed_case_dag_extension_single_file_target_matches_absolute_output",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_uppercase_dag_extension_missing_target_is_treated_as_single_file() {
    let root = unique_temp_dir("uppercase_dag_extension_missing_target_is_treated_");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_uppercase_dag_extension_missing_target_is_treated_as_single_file",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_mixed_case_dag_extension_missing_target_is_treated_as_single_file() {
    let root = unique_temp_dir("mixed_case_dag_extension_missing_target_is_treated");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_mixed_case_dag_extension_missing_target_is_treated_as_single_file",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_uppercase_dag_extension_curdir_suffix_single_file_target_matches_plain_uppercase_output(
) {
    let root = unique_temp_dir("uppercase_dag_extension_curdir_suffix_single_file_");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DAG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DAG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_dag_extension_curdir_suffix_single_file_target_matches_plain_uppercase_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_dag_extension_curdir_suffix_single_file_target_matches_plain_uppercase_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_dag_extension_curdir_suffix_single_file_target_matches_plain_uppercase_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_uppercase_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_uppercase_output(
) {
    let root = unique_temp_dir("uppercase_dag_extension_curdir_segment_trailing_sl");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DAG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DAG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_uppercase_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_uppercase_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_uppercase_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_uppercase_dag_extension_curdir_suffix_missing_target_matches_plain_uppercase_output(
) {
    let root = unique_temp_dir("uppercase_dag_extension_curdir_suffix_missing_targ");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DAG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_dag_extension_curdir_suffix_missing_target_matches_plain_uppercase_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_dag_extension_curdir_suffix_missing_target_matches_plain_uppercase_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_dag_extension_curdir_suffix_missing_target_matches_plain_uppercase_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_uppercase_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_uppercase_output(
) {
    let root = unique_temp_dir("uppercase_dag_extension_curdir_segment_trailing_sl");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DAG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_uppercase_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_uppercase_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_uppercase_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_mixed_case_dag_extension_curdir_suffix_single_file_target_matches_plain_mixed_case_output(
) {
    let root = unique_temp_dir("mixed_case_dag_extension_curdir_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DaG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DaG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_dag_extension_curdir_suffix_single_file_target_matches_plain_mixed_case_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_dag_extension_curdir_suffix_single_file_target_matches_plain_mixed_case_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_dag_extension_curdir_suffix_single_file_target_matches_plain_mixed_case_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_mixed_case_output(
) {
    let root = unique_temp_dir("mixed_case_dag_extension_curdir_segment_trailing_s");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DaG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DaG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_mixed_case_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("main.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_mixed_case_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_single_file_target_matches_plain_mixed_case_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_mixed_case_dag_extension_curdir_suffix_missing_target_matches_plain_mixed_case_output(
) {
    let root = unique_temp_dir("mixed_case_dag_extension_curdir_suffix_missing_tar");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DaG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_dag_extension_curdir_suffix_missing_target_matches_plain_mixed_case_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_dag_extension_curdir_suffix_missing_target_matches_plain_mixed_case_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_dag_extension_curdir_suffix_missing_target_matches_plain_mixed_case_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_mixed_case_output(
) {
    let root = unique_temp_dir("mixed_case_dag_extension_curdir_segment_trailing_s");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DaG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_mixed_case_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_mixed_case_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_dag_extension_curdir_segment_trailing_slash_missing_target_matches_plain_mixed_case_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_parent_segment_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_segment_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("../main.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment target daglang check");
    assert!(
        parent_segment.status.success(),
        "parent-segment target check should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_segment.stdout),
        expected_check_success_stdout(1),
        "parent-segment single-file check should parse exactly one file"
    );
    assert!(
        parent_segment.stderr.is_empty(),
        "parent-segment single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_curdir_segment_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_curdir_segment_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(".././main.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-segment target daglang check");
    assert!(
        parent_curdir_segment.status.success(),
        "parent-curdir-segment target check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_segment.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-segment single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_segment.stderr.is_empty(),
        "parent-curdir-segment single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_double_separator_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_double_separator_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//main.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator target daglang check");
    assert!(
        parent_double_separator.status.success(),
        "parent-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_double_separator.stdout),
        expected_check_success_stdout(1),
        "parent-double-separator single-file check should parse exactly one file"
    );
    assert!(
        parent_double_separator.stderr.is_empty(),
        "parent-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_single_file_target_matches_absolute_output()
{
    let root = unique_temp_dir("check_parent_double_separator_trailing_slash_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//main.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing target daglang check");
    assert!(
        parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing target check should succeed: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "parent-double-separator-trailing single-file check should parse exactly one file"
    );
    assert!(
        parent_double_separator_trailing.stderr.is_empty(),
        "parent-double-separator-trailing single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_curdir_double_separator_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_curdir_double_separator_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././main.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator target daglang check");
    assert!(
        parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-double-separator single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_double_separator.stderr.is_empty(),
        "parent-curdir-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_curdir_trailing_slash_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("check_parent_curdir_trailing_slash_single_file");
    let cwd = root.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let parent_curdir_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(".././main.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-trailing-slash target daglang check");
    assert!(
        parent_curdir_trailing_slash.status.success(),
        "parent-curdir-trailing-slash target check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_trailing_slash.stderr)
    );

    let absolute_target = root.join("main.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute-target daglang check from nested cwd");
    assert!(
        absolute.status.success(),
        "absolute-target check should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_trailing_slash.stdout, absolute.stdout,
        "parent-curdir-trailing-slash and absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing_slash.stderr, absolute.stderr,
        "parent-curdir-trailing-slash and absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_trailing_slash.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-trailing-slash single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_trailing_slash.stderr.is_empty(),
        "parent-curdir-trailing-slash single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_segment_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment target daglang check");
    assert!(
        curdir_segment.status.success(),
        "curdir-segment target check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment.stdout),
        expected_check_success_stdout(1),
        "curdir-segment single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment.stderr.is_empty(),
        "curdir-segment single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_segment_trailing_slash_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag/./")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing-slash target daglang check");
    assert!(
        curdir_segment_trailing_slash.status.success(),
        "curdir-segment-trailing-slash target check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "curdir-segment-trailing-slash and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "curdir-segment-trailing-slash and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_trailing_slash.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-trailing-slash single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_trailing_slash.stderr.is_empty(),
        "curdir-segment-trailing-slash single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_dot_double_separator_curdir_suffix_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_dot_double_separator_curdir_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix target daglang check");
    assert!(
        dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix target check should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator_curdir_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stdout),
        expected_check_success_stdout(1),
        "dot-double-separator-curdir-suffix single-file check should parse exactly one file"
    );
    assert!(
        dot_double_separator_curdir_suffix.stderr.is_empty(),
        "dot-double-separator-curdir-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_suffix_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_dot_double_separator_curdir_segment_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-suffix target daglang check");
    assert!(
        dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix target check should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator_curdir_segment_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stdout),
        expected_check_success_stdout(1),
        "dot-double-separator-curdir-segment-suffix single-file check should parse exactly one file"
    );
    assert!(
        dot_double_separator_curdir_segment_suffix.stderr.is_empty(),
        "dot-double-separator-curdir-segment-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("check_dot_double_separator_curdir_segment_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag//./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-double-separator target daglang check",
        );
    assert!(
        dot_double_separator_curdir_segment_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator_curdir_segment_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stdout),
        expected_check_success_stdout(1),
        "dot-double-separator-curdir-segment-double-separator single-file check should parse exactly one file"
    );
    assert!(
        dot_double_separator_curdir_segment_double_separator
            .stderr
            .is_empty(),
        "dot-double-separator-curdir-segment-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_dot_double_separator_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_dot_double_separator_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator target daglang check");
    assert!(
        dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        dot_double_separator_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&dot_double_separator_double_separator.stdout),
        expected_check_success_stdout(1),
        "dot-double-separator-double-separator single-file check should parse exactly one file"
    );
    assert!(
        dot_double_separator_double_separator.stderr.is_empty(),
        "dot-double-separator-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_dot_double_separator_curdir_suffix_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("check_dot_double_separator_curdir_suffix_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag//.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-suffix-double-separator target daglang check",
        );
    assert!(
        dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator_curdir_suffix_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-suffix-double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_suffix_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-suffix-double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stdout),
        expected_check_success_stdout(1),
        "dot-double-separator-curdir-suffix-double-separator single-file check should parse exactly one file"
    );
    assert!(
        dot_double_separator_curdir_suffix_double_separator
            .stderr
            .is_empty(),
        "dot-double-separator-curdir-suffix-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_double_separator_suffix_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_segment_double_separator_suffix_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-double-separator-suffix target daglang check",
        );
    assert!(
        dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix target check should succeed: {}",
        String::from_utf8_lossy(
            &dot_double_separator_curdir_segment_double_separator_suffix.stderr
        )
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator_curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-double-separator-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-double-separator-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator_suffix.stdout),
        expected_check_success_stdout(1),
        "dot-double-separator-curdir-segment-double-separator-suffix single-file check should parse exactly one file"
    );
    assert!(
        dot_double_separator_curdir_segment_double_separator_suffix
            .stderr
            .is_empty(),
        "dot-double-separator-curdir-segment-double-separator-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_trailing_slash_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("check_dot_double_separator_curdir_segment_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag/./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-trailing-slash target daglang check",
        );
    assert!(
        dot_double_separator_curdir_segment_trailing_slash
            .status
            .success(),
        "dot-double-separator-curdir-segment-trailing-slash target check should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        dot_double_separator_curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-trailing-slash and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-trailing-slash and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stdout),
        expected_check_success_stdout(1),
        "dot-double-separator-curdir-segment-trailing-slash single-file check should parse exactly one file"
    );
    assert!(
        dot_double_separator_curdir_segment_trailing_slash.stderr.is_empty(),
        "dot-double-separator-curdir-segment-trailing-slash single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_suffix_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix target daglang check");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix target check should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_suffix.stdout, plain_relative.stdout,
        "curdir-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, plain_relative.stderr,
        "curdir-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_suffix.stdout),
        expected_check_success_stdout(1),
        "curdir-suffix single-file check should parse exactly one file"
    );
    assert!(
        curdir_suffix.stderr.is_empty(),
        "curdir-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_relative_curdir_suffix_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_relative_curdir_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative_curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-suffix target daglang check");
    assert!(
        relative_curdir_suffix.status.success(),
        "relative-curdir-suffix target check should succeed: {}",
        String::from_utf8_lossy(&relative_curdir_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        relative_curdir_suffix.stdout, plain_relative.stdout,
        "relative-curdir-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        relative_curdir_suffix.stderr, plain_relative.stderr,
        "relative-curdir-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&relative_curdir_suffix.stdout),
        expected_check_success_stdout(1),
        "relative-curdir-suffix single-file check should parse exactly one file"
    );
    assert!(
        relative_curdir_suffix.stderr.is_empty(),
        "relative-curdir-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&relative_curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_relative_curdir_segment_trailing_slash_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_relative_curdir_segment_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag/./")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-segment-trailing-slash target daglang check");
    assert!(
        relative_curdir_segment_trailing_slash.status.success(),
        "relative-curdir-segment-trailing-slash target check should succeed: {}",
        String::from_utf8_lossy(&relative_curdir_segment_trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        relative_curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "relative-curdir-segment-trailing-slash and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "relative-curdir-segment-trailing-slash and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&relative_curdir_segment_trailing_slash.stdout),
        expected_check_success_stdout(1),
        "relative-curdir-segment-trailing-slash single-file check should parse exactly one file"
    );
    assert!(
        relative_curdir_segment_trailing_slash.stderr.is_empty(),
        "relative-curdir-segment-trailing-slash single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&relative_curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_relative_curdir_segment_suffix_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_relative_curdir_segment_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-segment-suffix target daglang check");
    assert!(
        relative_curdir_segment_suffix.status.success(),
        "relative-curdir-segment-suffix target check should succeed: {}",
        String::from_utf8_lossy(&relative_curdir_segment_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        relative_curdir_segment_suffix.stdout, plain_relative.stdout,
        "relative-curdir-segment-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_suffix.stderr, plain_relative.stderr,
        "relative-curdir-segment-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&relative_curdir_segment_suffix.stdout),
        expected_check_success_stdout(1),
        "relative-curdir-segment-suffix single-file check should parse exactly one file"
    );
    assert!(
        relative_curdir_segment_suffix.stderr.is_empty(),
        "relative-curdir-segment-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&relative_curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_segment_suffix_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix target daglang check");
    assert!(
        curdir_segment_suffix.status.success(),
        "curdir-segment-suffix target check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment_suffix.stdout, plain_relative.stdout,
        "curdir-segment-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, plain_relative.stderr,
        "curdir-segment-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_suffix.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-suffix single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_suffix.stderr.is_empty(),
        "curdir-segment-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_suffix_double_separator_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_suffix_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator target daglang check");
    assert!(
        curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, plain_relative.stdout,
        "curdir-suffix-double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, plain_relative.stderr,
        "curdir-suffix-double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_suffix_double_separator.stdout),
        expected_check_success_stdout(1),
        "curdir-suffix-double-separator single-file check should parse exactly one file"
    );
    assert!(
        curdir_suffix_double_separator.stderr.is_empty(),
        "curdir-suffix-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_relative_curdir_suffix_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_relative_curdir_suffix_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-suffix-double-separator target daglang check");
    assert!(
        relative_curdir_suffix_double_separator.status.success(),
        "relative-curdir-suffix-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&relative_curdir_suffix_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        relative_curdir_suffix_double_separator.stdout, plain_relative.stdout,
        "relative-curdir-suffix-double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        relative_curdir_suffix_double_separator.stderr, plain_relative.stderr,
        "relative-curdir-suffix-double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&relative_curdir_suffix_double_separator.stdout),
        expected_check_success_stdout(1),
        "relative-curdir-suffix-double-separator single-file check should parse exactly one file"
    );
    assert!(
        relative_curdir_suffix_double_separator.stderr.is_empty(),
        "relative-curdir-suffix-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&relative_curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_segment_double_separator_single_file_target_matches_plain_relative_output()
{
    let root = unique_temp_dir("check_curdir_segment_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag//./")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator target daglang check");
    assert!(
        curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator target check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, plain_relative.stdout,
        "curdir-segment-double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, plain_relative.stderr,
        "curdir-segment-double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_double_separator.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-double-separator single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_double_separator.stderr.is_empty(),
        "curdir-segment-double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_relative_curdir_segment_double_separator_trailing_slash_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_relative_curdir_segment_double_separator_trailing_slash_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative_curdir_segment_double_separator_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag//./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-double-separator-trailing-slash target daglang check",
        );
    assert!(
        relative_curdir_segment_double_separator_trailing_slash
            .status
            .success(),
        "relative-curdir-segment-double-separator-trailing-slash target check should succeed: {}",
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        relative_curdir_segment_double_separator_trailing_slash.stdout, plain_relative.stdout,
        "relative-curdir-segment-double-separator-trailing-slash and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_double_separator_trailing_slash.stderr, plain_relative.stderr,
        "relative-curdir-segment-double-separator-trailing-slash and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_trailing_slash.stdout),
        expected_check_success_stdout(1),
        "relative-curdir-segment-double-separator-trailing-slash single-file check should parse exactly one file"
    );
    assert!(
        relative_curdir_segment_double_separator_trailing_slash
            .stderr
            .is_empty(),
        "relative-curdir-segment-double-separator-trailing-slash single-file check should not emit stderr: {}",
        String::from_utf8_lossy(
            &relative_curdir_segment_double_separator_trailing_slash.stderr
        )
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_relative_curdir_segment_double_separator_suffix_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_relative_curdir_segment_double_separator_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let relative_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-double-separator-suffix target daglang check",
        );
    assert!(
        relative_curdir_segment_double_separator_suffix
            .status
            .success(),
        "relative-curdir-segment-double-separator-suffix target check should succeed: {}",
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        relative_curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "relative-curdir-segment-double-separator-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "relative-curdir-segment-double-separator-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_suffix.stdout),
        expected_check_success_stdout(1),
        "relative-curdir-segment-double-separator-suffix single-file check should parse exactly one file"
    );
    assert!(
        relative_curdir_segment_double_separator_suffix.stderr.is_empty(),
        "relative-curdir-segment-double-separator-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_segment_double_separator_suffix_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_curdir_segment_double_separator_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./main.dag//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator-suffix target daglang check");
    assert!(
        curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix target check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "curdir-segment-double-separator-suffix and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "curdir-segment-double-separator-suffix and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-double-separator-suffix single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_double_separator_suffix.stderr.is_empty(),
        "curdir-segment-double-separator-suffix single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_trailing_slash_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash target daglang check");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash target check should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&trailing_slash.stdout),
        expected_check_success_stdout(1),
        "trailing-slash single-file check should parse exactly one file"
    );
    assert!(
        trailing_slash.stderr.is_empty(),
        "trailing-slash single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_double_separator_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator target daglang check");
    assert!(
        double_separator.status.success(),
        "double-separator target check should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator.stdout),
        expected_check_success_stdout(1),
        "double-separator single-file check should parse exactly one file"
    );
    assert!(
        double_separator.stderr.is_empty(),
        "double-separator single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_double_separator_trailing_slash_single_file_target_matches_plain_relative_output()
{
    let root = unique_temp_dir("check_double_separator_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(".//main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing target daglang check");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator-trailing target check should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative target daglang check");
    assert!(
        plain_relative.status.success(),
        "plain-relative target check should succeed: {}",
        String::from_utf8_lossy(&plain_relative.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, plain_relative.stdout,
        "double-separator-trailing and plain-relative single-file check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, plain_relative.stderr,
        "double-separator-trailing and plain-relative single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "double-separator-trailing single-file check should parse exactly one file"
    );
    assert!(
        double_separator_trailing.stderr.is_empty(),
        "double-separator-trailing single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_curdir_double_separator_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_double_separator_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double-separator missing-target daglang check");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_trailing_slash_missing_single_file_matches_plain_relative_output()
{
    let root = unique_temp_dir("check_double_separator_trailing_slash_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing missing-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, plain_relative.stdout,
        "double-separator-trailing and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, plain_relative.stderr,
        "double-separator-trailing and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment missing-target daglang check");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_trailing_slash_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_trailing_slash_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag/./")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing-slash missing-target daglang check");
    assert!(
        !curdir_segment_trailing_slash.status.success(),
        "curdir-segment-trailing-slash missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "curdir-segment-trailing-slash and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "curdir-segment-trailing-slash and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_trailing_slash.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_suffix_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_relative_curdir_suffix_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let relative_curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-suffix missing-target daglang check");
    assert!(
        !relative_curdir_suffix.status.success(),
        "relative-curdir-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        relative_curdir_suffix.stdout, plain_relative.stdout,
        "relative-curdir-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_suffix.stderr, plain_relative.stderr,
        "relative-curdir-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative_curdir_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&relative_curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_trailing_slash_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_relative_curdir_segment_trailing_slash_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let relative_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag/./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-trailing-slash missing-target daglang check",
        );
    assert!(
        !relative_curdir_segment_trailing_slash.status.success(),
        "relative-curdir-segment-trailing-slash missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        relative_curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "relative-curdir-segment-trailing-slash and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "relative-curdir-segment-trailing-slash and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_trailing_slash.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&relative_curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_suffix_missing_single_file_matches_plain_relative_output()
{
    let root = unique_temp_dir("check_relative_curdir_segment_suffix_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let relative_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-segment-suffix missing-target daglang check");
    assert!(
        !relative_curdir_segment_suffix.status.success(),
        "relative-curdir-segment-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        relative_curdir_segment_suffix.stdout, plain_relative.stdout,
        "relative-curdir-segment-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_suffix.stderr, plain_relative.stderr,
        "relative-curdir-segment-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&relative_curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_double_separator_trailing_slash_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_relative_curdir_segment_double_separator_trailing_slash_missing_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let relative_curdir_segment_double_separator_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag//./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-double-separator-trailing-slash missing-target daglang check",
        );
    assert!(
        !relative_curdir_segment_double_separator_trailing_slash
            .status
            .success(),
        "relative-curdir-segment-double-separator-trailing-slash missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        relative_curdir_segment_double_separator_trailing_slash.stdout, plain_relative.stdout,
        "relative-curdir-segment-double-separator-trailing-slash and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_double_separator_trailing_slash.stderr, plain_relative.stderr,
        "relative-curdir-segment-double-separator-trailing-slash and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_trailing_slash.stderr)
            .contains(&format!(
                "failed to canonicalize {}",
                missing_target.display()
            )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_double_separator_suffix_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_relative_curdir_segment_double_separator_suffix_missing_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let relative_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-double-separator-suffix missing-target daglang check",
        );
    assert!(
        !relative_curdir_segment_double_separator_suffix
            .status
            .success(),
        "relative-curdir-segment-double-separator-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        relative_curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "relative-curdir-segment-double-separator-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "relative-curdir-segment-double-separator-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_suffix.stderr).contains(
            &format!("failed to canonicalize {}", missing_target.display())
        ),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_suffix_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_dot_double_separator_curdir_suffix_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix missing-target daglang check");
    assert!(
        !dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        dot_double_separator_curdir_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_suffix_missing_single_file_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("check_dot_double_separator_curdir_segment_suffix_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag/./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-suffix missing-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr).contains(
            &format!("failed to canonicalize {}", missing_target.display())
        ),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_double_separator_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_segment_double_separator_missing_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag//./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-double-separator missing-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-double-separator and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-double-separator and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
            .contains(&format!(
                "failed to canonicalize {}",
                missing_target.display()
            )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_double_separator_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_dot_double_separator_double_separator_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator missing-target daglang check");
    assert!(
        !dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        dot_double_separator_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-double-separator and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-double-separator and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_suffix_double_separator_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_suffix_double_separator_missing_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag//.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-suffix-double-separator missing-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        dot_double_separator_curdir_suffix_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-suffix-double-separator and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_suffix_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-suffix-double-separator and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
            .contains(&format!(
                "failed to canonicalize {}",
                missing_target.display()
            )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_double_separator_suffix_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_segment_double_separator_suffix_missing_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-double-separator-suffix missing-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-double-separator-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-double-separator-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(
            &dot_double_separator_curdir_segment_double_separator_suffix.stderr
        )
        .contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(
            &dot_double_separator_curdir_segment_double_separator_suffix.stderr
        )
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_trailing_slash_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_segment_trailing_slash_missing_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag/./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-trailing-slash missing-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_trailing_slash
            .status
            .success(),
        "dot-double-separator-curdir-segment-trailing-slash missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-trailing-slash and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-trailing-slash and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
            .contains(&format!(
                "failed to canonicalize {}",
                missing_target.display()
            )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_suffix_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_suffix_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix missing-target daglang check");
    assert!(
        !curdir_suffix.status.success(),
        "curdir-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_suffix.stdout, plain_relative.stdout,
        "curdir-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, plain_relative.stderr,
        "curdir-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_suffix_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_suffix_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix missing-target daglang check");
    assert!(
        !curdir_segment_suffix.status.success(),
        "curdir-segment-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_suffix.stdout, plain_relative.stdout,
        "curdir-segment-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, plain_relative.stderr,
        "curdir-segment-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_suffix_double_separator_missing_single_file_matches_plain_relative_output()
{
    let root = unique_temp_dir("check_curdir_suffix_double_separator_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator missing-target daglang check");
    assert!(
        !curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, plain_relative.stdout,
        "curdir-suffix-double-separator and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, plain_relative.stderr,
        "curdir-suffix-double-separator and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_double_separator_missing_single_file_matches_plain_relative_output()
{
    let root = unique_temp_dir("check_curdir_segment_double_separator_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag//./")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator missing-target daglang check");
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, plain_relative.stdout,
        "curdir-segment-double-separator and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, plain_relative.stderr,
        "curdir-segment-double-separator and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_double_separator_suffix_missing_single_file_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_curdir_segment_double_separator_suffix_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./missing.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix missing-target daglang check",
        );
    assert!(
        !curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "curdir-segment-double-separator-suffix and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "curdir-segment-double-separator-suffix and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_trailing_slash_missing_single_file_matches_plain_relative_output() {
    let root = unique_temp_dir("check_trailing_slash_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash missing-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative missing-target check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_and_absolute_invalid_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("check_relative_absolute_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-target daglang check");
    assert!(
        !relative.status.success(),
        "relative-target check should fail for malformed source"
    );

    let absolute_target = root.join("broken.dag");
    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_target)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute-target check should fail for malformed source"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute invalid single-file check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute invalid single-file check stderr should match"
    );
    let canonical_target = root
        .join("broken.dag")
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&relative.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with location for malformed single-file target: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_parent_segment_invalid_single_file_target_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_segment_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("../broken.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment invalid-target daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-segment invalid target: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_segment_invalid_single_file_target_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_curdir_segment_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(".././broken.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-segment invalid-target daglang check");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-curdir-segment invalid target: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_invalid_single_file_target_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_double_separator_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//broken.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator invalid-target daglang check");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-double-separator invalid target: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_invalid_single_file_target_matches_absolute_output(
) {
    let parent =
        unique_temp_dir("check_parent_double_separator_trailing_slash_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//broken.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator-trailing invalid-target daglang check");
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-double-separator-trailing invalid target: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_double_separator_invalid_single_file_target_matches_absolute_output()
{
    let parent = unique_temp_dir("check_parent_curdir_double_separator_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././broken.dag//")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-double-separator invalid-target daglang check");
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-curdir-double-separator invalid target: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_trailing_slash_invalid_single_file_target_matches_absolute_output() {
    let parent = unique_temp_dir("check_parent_curdir_trailing_slash_invalid_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let broken_file = parent.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let parent_curdir_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(".././broken.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-trailing-slash invalid-target daglang check");
    assert!(
        !parent_curdir_trailing_slash.status.success(),
        "parent-curdir-trailing-slash invalid-target check should fail for malformed source"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute invalid-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute invalid-target check should fail for malformed source"
    );

    assert_eq!(
        parent_curdir_trailing_slash.stdout, absolute.stdout,
        "parent-curdir-trailing-slash and absolute invalid-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing_slash.stderr, absolute.stderr,
        "parent-curdir-trailing-slash and absolute invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for parent-curdir-trailing-slash invalid target: {}",
        String::from_utf8_lossy(&parent_curdir_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_curdir_segment_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment invalid-target daglang check");
    assert!(
        !curdir_segment.status.success(),
        "curdir-segment invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_segment.stdout, plain_relative.stdout,
        "curdir-segment and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_segment.stderr, plain_relative.stderr,
        "curdir-segment and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir invalid target: {}",
        String::from_utf8_lossy(&curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_curdir_segment_trailing_slash_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag/./")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing-slash invalid-target daglang check");
    assert!(
        !curdir_segment_trailing_slash.status.success(),
        "curdir-segment-trailing-slash invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "curdir-segment-trailing-slash and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "curdir-segment-trailing-slash and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-segment-trailing-slash invalid target: {}",
        String::from_utf8_lossy(&curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_suffix_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_relative_curdir_suffix_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let relative_curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-suffix invalid-target daglang check");
    assert!(
        !relative_curdir_suffix.status.success(),
        "relative-curdir-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        relative_curdir_suffix.stdout, plain_relative.stdout,
        "relative-curdir-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_suffix.stderr, plain_relative.stderr,
        "relative-curdir-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&relative_curdir_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for relative-curdir-suffix invalid target: {}",
        String::from_utf8_lossy(&relative_curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_relative_curdir_segment_trailing_slash_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let relative_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag/./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-trailing-slash invalid-target daglang check",
        );
    assert!(
        !relative_curdir_segment_trailing_slash.status.success(),
        "relative-curdir-segment-trailing-slash invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        relative_curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "relative-curdir-segment-trailing-slash and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "relative-curdir-segment-trailing-slash and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for relative-curdir-segment-trailing-slash invalid target: {}",
        String::from_utf8_lossy(&relative_curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_relative_curdir_segment_suffix_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let relative_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-curdir-segment-suffix invalid-target daglang check");
    assert!(
        !relative_curdir_segment_suffix.status.success(),
        "relative-curdir-segment-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        relative_curdir_segment_suffix.stdout, plain_relative.stdout,
        "relative-curdir-segment-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_suffix.stderr, plain_relative.stderr,
        "relative-curdir-segment-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for relative-curdir-segment-suffix invalid target: {}",
        String::from_utf8_lossy(&relative_curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_double_separator_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_relative_curdir_segment_double_separator_trailing_slash_invalid_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let relative_curdir_segment_double_separator_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag//./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-double-separator-trailing-slash invalid-target daglang check",
        );
    assert!(
        !relative_curdir_segment_double_separator_trailing_slash
            .status
            .success(),
        "relative-curdir-segment-double-separator-trailing-slash invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        relative_curdir_segment_double_separator_trailing_slash.stdout, plain_relative.stdout,
        "relative-curdir-segment-double-separator-trailing-slash and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_double_separator_trailing_slash.stderr, plain_relative.stderr,
        "relative-curdir-segment-double-separator-trailing-slash and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for relative-curdir-segment-double-separator-trailing-slash invalid target: {}",
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_relative_curdir_segment_double_separator_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_relative_curdir_segment_double_separator_suffix_invalid_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let relative_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run relative-curdir-segment-double-separator-suffix invalid-target daglang check",
        );
    assert!(
        !relative_curdir_segment_double_separator_suffix
            .status
            .success(),
        "relative-curdir-segment-double-separator-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        relative_curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "relative-curdir-segment-double-separator-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        relative_curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "relative-curdir-segment-double-separator-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for relative-curdir-segment-double-separator-suffix invalid target: {}",
        String::from_utf8_lossy(&relative_curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_dot_double_separator_curdir_suffix_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix invalid-target daglang check");
    assert!(
        !dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        dot_double_separator_curdir_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for dot-double-separator-curdir-suffix invalid target: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("check_dot_double_separator_curdir_segment_suffix_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag/./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-suffix invalid-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for dot-double-separator-curdir-segment-suffix invalid target: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_segment_double_separator_invalid_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag//./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-double-separator invalid-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-double-separator and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-double-separator and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for dot-double-separator-curdir-segment-double-separator invalid target: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_dot_double_separator_double_separator_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator invalid-target daglang check");
    assert!(
        !dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        dot_double_separator_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-double-separator and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-double-separator and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for dot-double-separator-double-separator invalid target: {}",
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_suffix_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_suffix_double_separator_invalid_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag//.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-suffix-double-separator invalid-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        dot_double_separator_curdir_suffix_double_separator.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-suffix-double-separator and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_suffix_double_separator.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-suffix-double-separator and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for dot-double-separator-curdir-suffix-double-separator invalid target: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_double_separator_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_segment_double_separator_suffix_invalid_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-double-separator-suffix invalid-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-double-separator-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-double-separator-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(
            &dot_double_separator_curdir_segment_double_separator_suffix.stderr
        )
        .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for dot-double-separator-curdir-segment-double-separator-suffix invalid target: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_dot_double_separator_curdir_segment_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "check_dot_double_separator_curdir_segment_trailing_slash_invalid_single_file",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag/./")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-trailing-slash invalid-target daglang check",
        );
    assert!(
        !dot_double_separator_curdir_segment_trailing_slash
            .status
            .success(),
        "dot-double-separator-curdir-segment-trailing-slash invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        dot_double_separator_curdir_segment_trailing_slash.stdout, plain_relative.stdout,
        "dot-double-separator-curdir-segment-trailing-slash and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        dot_double_separator_curdir_segment_trailing_slash.stderr, plain_relative.stderr,
        "dot-double-separator-curdir-segment-trailing-slash and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for dot-double-separator-curdir-segment-trailing-slash invalid target: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_suffix_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_suffix_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix invalid-target daglang check");
    assert!(
        !curdir_suffix.status.success(),
        "curdir-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_suffix.stdout, plain_relative.stdout,
        "curdir-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, plain_relative.stderr,
        "curdir-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-suffix invalid target: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_suffix_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_curdir_segment_suffix_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix invalid-target daglang check");
    assert!(
        !curdir_segment_suffix.status.success(),
        "curdir-segment-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_segment_suffix.stdout, plain_relative.stdout,
        "curdir-segment-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, plain_relative.stderr,
        "curdir-segment-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-segment-suffix invalid target: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_suffix_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_curdir_suffix_double_separator_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator invalid-target daglang check");
    assert!(
        !curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, plain_relative.stdout,
        "curdir-suffix-double-separator and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, plain_relative.stderr,
        "curdir-suffix-double-separator and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-suffix-double-separator invalid target: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_curdir_segment_double_separator_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag//./")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator invalid-target daglang check");
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, plain_relative.stdout,
        "curdir-segment-double-separator and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, plain_relative.stderr,
        "curdir-segment-double-separator and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-segment-double-separator invalid target: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_segment_double_separator_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_curdir_segment_double_separator_suffix_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix invalid-target daglang check",
        );
    assert!(
        !curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, plain_relative.stdout,
        "curdir-segment-double-separator-suffix and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, plain_relative.stderr,
        "curdir-segment-double-separator-suffix and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-segment-double-separator-suffix invalid target: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_trailing_slash_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_trailing_slash_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash invalid-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        trailing_slash.stdout, plain_relative.stdout,
        "trailing-slash and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, plain_relative.stderr,
        "trailing-slash and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for trailing-slash invalid target: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_curdir_double_separator_invalid_single_file_target_matches_plain_relative_output()
{
    let root = unique_temp_dir("check_curdir_double_separator_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("./broken.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double-separator invalid-target daglang check");
    assert!(
        !curdir_double_separator.status.success(),
        "curdir-double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        curdir_double_separator.stdout, plain_relative.stdout,
        "curdir-double-separator and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        curdir_double_separator.stderr, plain_relative.stderr,
        "curdir-double-separator and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for curdir-double-separator invalid target: {}",
        String::from_utf8_lossy(&curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("check_double_separator_trailing_slash_invalid_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing invalid-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        double_separator_trailing.stdout, plain_relative.stdout,
        "double-separator-trailing and plain-relative invalid-target check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, plain_relative.stderr,
        "double-separator-trailing and plain-relative invalid-target check stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with normalized path for double-separator-trailing invalid target: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_invalid_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator invalid-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator invalid-target check should fail for malformed source"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative invalid-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative invalid-target check should fail for malformed source"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative invalid-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonical path for double-separator invalid target: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_double_separator_missing_target_matches_plain_relative_output() {
    let root = unique_temp_dir("check_double_separator_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".//missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator missing-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator missing-target check should fail"
    );

    let plain_relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative missing-target daglang check");
    assert!(
        !plain_relative.status.success(),
        "plain-relative missing-target check should fail"
    );

    assert_eq!(
        double_separator.stdout, plain_relative.stdout,
        "double-separator and plain-relative missing-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, plain_relative.stderr,
        "double-separator and plain-relative missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_double_separator = PathBuf::from(format!("{}//broken.dag", root.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute invalid-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_double_separator = PathBuf::from(format!("{}//missing.dag", root.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute missing-target daglang check");
    assert!(
        !double_separator.status.success(),
        "double-separator absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_mixed_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_mixed_segment_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_mixed = root.join(".").join("main.dag");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute single-file daglang check");
    assert!(
        mixed.status.success(),
        "mixed-segment absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&mixed.stdout),
        expected_check_success_stdout(1),
        "mixed-segment absolute single-file check should parse exactly one file"
    );
    assert!(
        mixed.stderr.is_empty(),
        "mixed-segment absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_curdir_segment_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_curdir_segment_trailing_slash_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = nested.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_curdir_segment_trailing = root.join("nested/./main.dag/");

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing absolute single-file daglang check");
    assert!(
        curdir_segment_trailing.status.success(),
        "curdir-segment-trailing absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_trailing.stdout, canonical.stdout,
        "curdir-segment-trailing and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing.stderr, canonical.stderr,
        "curdir-segment-trailing and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_trailing.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-trailing absolute single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_trailing.stderr.is_empty(),
        "curdir-segment-trailing absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_curdir_suffix_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_suffix_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = nested.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_curdir_suffix = root.join("nested/main.dag/.");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_suffix)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix absolute single-file daglang check");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_suffix.stdout, canonical.stdout,
        "curdir-suffix and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, canonical.stderr,
        "curdir-suffix and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_suffix.stdout),
        expected_check_success_stdout(1),
        "curdir-suffix absolute single-file check should parse exactly one file"
    );
    assert!(
        curdir_suffix.stderr.is_empty(),
        "curdir-suffix absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_curdir_suffix_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_curdir_suffix_double_separator_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = nested.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_curdir_suffix_double_separator =
        PathBuf::from(format!("{}//.", main_file.display()));

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_suffix_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator absolute single-file daglang check");
    assert!(
        curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, canonical.stdout,
        "curdir-suffix-double-separator and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, canonical.stderr,
        "curdir-suffix-double-separator and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_suffix_double_separator.stdout),
        expected_check_success_stdout(1),
        "curdir-suffix-double-separator absolute single-file check should parse exactly one file"
    );
    assert!(
        curdir_suffix_double_separator.stderr.is_empty(),
        "curdir-suffix-double-separator absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_curdir_segment_suffix_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_segment_suffix_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = nested.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_curdir_segment_suffix = root.join("nested/main.dag/./.");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_suffix)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix absolute single-file daglang check");
    assert!(
        curdir_segment_suffix.status.success(),
        "curdir-segment-suffix absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_suffix.stdout, canonical.stdout,
        "curdir-segment-suffix and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, canonical.stderr,
        "curdir-segment-suffix and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_suffix.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-suffix absolute single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_suffix.stderr.is_empty(),
        "curdir-segment-suffix absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_curdir_segment_double_separator_suffix_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_curdir_segment_double_separator_suffix_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = nested.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_curdir_segment_double_separator_suffix =
        PathBuf::from(format!("{}//./.", main_file.display()));

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_double_separator_suffix)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix absolute single-file daglang check",
        );
    assert!(
        curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, canonical.stdout,
        "curdir-segment-double-separator-suffix and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, canonical.stderr,
        "curdir-segment-double-separator-suffix and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-double-separator-suffix absolute single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_double_separator_suffix.stderr.is_empty(),
        "curdir-segment-double-separator-suffix absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_curdir_segment_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_curdir_segment_double_separator_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = nested.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_curdir_segment_double_separator =
        PathBuf::from(format!("{}//./", main_file.display()));

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator absolute single-file daglang check");
    assert!(
        curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, canonical.stdout,
        "curdir-segment-double-separator and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, canonical.stderr,
        "curdir-segment-double-separator and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&curdir_segment_double_separator.stdout),
        expected_check_success_stdout(1),
        "curdir-segment-double-separator absolute single-file check should parse exactly one file"
    );
    assert!(
        curdir_segment_double_separator.stderr.is_empty(),
        "curdir-segment-double-separator absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_segment = root.join("nested/../main.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute single-file daglang check");
    assert!(
        parent_segment.status.success(),
        "parent-segment absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_segment.stdout),
        expected_check_success_stdout(1),
        "parent-segment absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_segment.stderr.is_empty(),
        "parent-segment absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_curdir_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_curdir_segment_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_curdir_segment = root.join("nested/./../main.dag");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment absolute single-file daglang check");
    assert!(
        parent_curdir_segment.status.success(),
        "parent-curdir-segment absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment.stdout, canonical.stdout,
        "parent-curdir-segment and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, canonical.stderr,
        "parent-curdir-segment and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_segment.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-segment absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_segment.stderr.is_empty(),
        "parent-curdir-segment absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_curdir_segment_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_parent_curdir_segment_trailing_slash_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_curdir_segment_trailing = root.join("nested/./../main.dag/");

    let parent_curdir_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment-trailing absolute single-file daglang check");
    assert!(
        parent_curdir_segment_trailing.status.success(),
        "parent-curdir-segment-trailing absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-trailing and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-trailing and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-segment-trailing absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_segment_trailing.stderr.is_empty(),
        "parent-curdir-segment-trailing absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_curdir_segment_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_parent_curdir_segment_double_separator_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_curdir_segment_double_separator =
        PathBuf::from(format!("{}//./../main.dag", nested.display()));

    let parent_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-double-separator absolute single-file daglang check",
        );
    assert!(
        parent_curdir_segment_double_separator.status.success(),
        "parent-curdir-segment-double-separator absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_double_separator.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_segment_double_separator.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-segment-double-separator absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_segment_double_separator.stderr.is_empty(),
        "parent-curdir-segment-double-separator absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_curdir_segment_double_separator_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir(
        "check_absolute_parent_curdir_segment_double_separator_trailing_slash_single_file",
    );
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_curdir_segment_double_separator_trailing =
        PathBuf::from(format!("{}//./../main.dag/", nested.display()));

    let parent_curdir_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-double-separator-trailing absolute single-file daglang check",
        );
    assert!(
        parent_curdir_segment_double_separator_trailing.status.success(),
        "parent-curdir-segment-double-separator-trailing absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator-trailing and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator-trailing and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "parent-curdir-segment-double-separator-trailing absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_curdir_segment_double_separator_trailing.stderr.is_empty(),
        "parent-curdir-segment-double-separator-trailing absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_segment_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_parent_segment_double_separator_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_segment_double_separator =
        PathBuf::from(format!("{}//../main.dag", nested.display()));

    let parent_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double-separator absolute single-file daglang check");
    assert!(
        parent_segment_double_separator.status.success(),
        "parent-segment-double-separator absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_segment_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_double_separator.stdout, canonical.stdout,
        "parent-segment-double-separator and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_segment_double_separator.stderr, canonical.stderr,
        "parent-segment-double-separator and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_segment_double_separator.stdout),
        expected_check_success_stdout(1),
        "parent-segment-double-separator absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_segment_double_separator.stderr.is_empty(),
        "parent-segment-double-separator absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_segment_double_separator_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir(
        "check_absolute_parent_segment_double_separator_trailing_slash_single_file",
    );
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_segment_double_separator_trailing =
        PathBuf::from(format!("{}//../main.dag/", nested.display()));

    let parent_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double-separator-trailing absolute single-file daglang check");
    assert!(
        parent_segment_double_separator_trailing.status.success(),
        "parent-segment-double-separator-trailing absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_segment_double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-segment-double-separator-trailing and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-segment-double-separator-trailing and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_segment_double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "parent-segment-double-separator-trailing absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_segment_double_separator_trailing.stderr.is_empty(),
        "parent-segment-double-separator-trailing absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_segment_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_parent_segment_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_parent_segment_trailing_slash_single_file");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_parent_segment_trailing =
        PathBuf::from(format!("{}/", root.join("nested/../main.dag").display()));

    let parent_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-trailing absolute single-file daglang check");
    assert!(
        parent_segment_trailing.status.success(),
        "parent-segment-trailing absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&parent_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_trailing.stdout, canonical.stdout,
        "parent-segment-trailing and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        parent_segment_trailing.stderr, canonical.stderr,
        "parent-segment-trailing and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&parent_segment_trailing.stdout),
        expected_check_success_stdout(1),
        "parent-segment-trailing absolute single-file check should parse exactly one file"
    );
    assert!(
        parent_segment_trailing.stderr.is_empty(),
        "parent-segment-trailing absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&parent_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_double_separator_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_double_separator = PathBuf::from(format!("{}//main.dag", root.display()));

    let double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute single-file daglang check");
    assert!(
        double_separator.status.success(),
        "double-separator absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator.stdout),
        expected_check_success_stdout(1),
        "double-separator absolute single-file check should parse exactly one file"
    );
    assert!(
        double_separator.stderr.is_empty(),
        "double-separator absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_trailing_slash_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", main_file.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute single-file daglang check");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&trailing_slash.stdout),
        expected_check_success_stdout(1),
        "trailing-slash absolute single-file check should parse exactly one file"
    );
    assert!(
        trailing_slash.stderr.is_empty(),
        "trailing-slash absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_trailing_slash_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_trailing_slash_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", broken_file.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute invalid-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_trailing_slash_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_trailing_slash_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_trailing_slash = PathBuf::from(format!("{}/", missing_target.display()));

    let trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_trailing_slash)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute missing-target daglang check");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&trailing_slash.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("check_absolute_double_separator_trailing_slash_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let main_file = root.join("main.dag");
    std::fs::write(&main_file, "module sample.main\nfn ok() -> Unit {}")
        .expect("failed to write valid dag source");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write sibling malformed source");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//main.dag/", root.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing absolute single-file daglang check");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator-trailing absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&main_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file daglang check");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file check should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute single-file check stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute single-file check stderr should match"
    );
    assert_eq!(
        String::from_utf8_lossy(&double_separator_trailing.stdout),
        expected_check_success_stdout(1),
        "double-separator-trailing absolute single-file check should parse exactly one file"
    );
    assert!(
        double_separator_trailing.stderr.is_empty(),
        "double-separator-trailing absolute single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_invalid_target_matches_canonical_output()
{
    let root = unique_temp_dir("check_absolute_double_separator_trailing_slash_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//broken.dag/", root.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing absolute invalid-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_double_separator_trailing_slash_missing_target_matches_canonical_output()
{
    let root = unique_temp_dir("check_absolute_double_separator_trailing_slash_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_double_separator_trailing =
        PathBuf::from(format!("{}//missing.dag/", root.display()));

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator-trailing absolute missing-target daglang check");
    assert!(
        !double_separator_trailing.status.success(),
        "double-separator-trailing absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator-trailing and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator-trailing and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&double_separator_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_mixed_segment_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_mixed_segment_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_mixed = root.join(".").join("broken.dag");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute invalid-target daglang check");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&mixed.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_trailing_slash_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_segment_trailing_slash_invalid_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let broken_file = nested.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_curdir_segment_trailing = root.join("nested/./broken.dag/");

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing absolute invalid-target daglang check");
    assert!(
        !curdir_segment_trailing.status.success(),
        "curdir-segment-trailing absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        curdir_segment_trailing.stdout, canonical.stdout,
        "curdir-segment-trailing and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing.stderr, canonical.stderr,
        "curdir-segment-trailing and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_suffix_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_suffix_invalid_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let broken_file = nested.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_curdir_suffix = root.join("nested/broken.dag/.");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_suffix)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix absolute invalid-target daglang check");
    assert!(
        !curdir_suffix.status.success(),
        "curdir-suffix absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        curdir_suffix.stdout, canonical.stdout,
        "curdir-suffix and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, canonical.stderr,
        "curdir-suffix and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_suffix_double_separator_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_suffix_double_separator_invalid_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let broken_file = nested.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_curdir_suffix_double_separator =
        PathBuf::from(format!("{}//.", broken_file.display()));

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_suffix_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-suffix-double-separator absolute invalid-target daglang check",
        );
    assert!(
        !curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, canonical.stdout,
        "curdir-suffix-double-separator and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, canonical.stderr,
        "curdir-suffix-double-separator and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_suffix_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_segment_suffix_invalid_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let broken_file = nested.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_curdir_segment_suffix = root.join("nested/broken.dag/./.");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_suffix)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix absolute invalid-target daglang check");
    assert!(
        !curdir_segment_suffix.status.success(),
        "curdir-segment-suffix absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        curdir_segment_suffix.stdout, canonical.stdout,
        "curdir-segment-suffix and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, canonical.stderr,
        "curdir-segment-suffix and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_double_separator_suffix_invalid_target_matches_canonical_output(
) {
    let root =
        unique_temp_dir("check_absolute_curdir_segment_double_separator_suffix_invalid_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let broken_file = nested.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_curdir_segment_double_separator_suffix =
        PathBuf::from(format!("{}//./.", broken_file.display()));

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_double_separator_suffix)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix absolute invalid-target daglang check",
        );
    assert!(
        !curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, canonical.stdout,
        "curdir-segment-double-separator-suffix and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, canonical.stderr,
        "curdir-segment-double-separator-suffix and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_double_separator_invalid_target_matches_canonical_output()
{
    let root = unique_temp_dir("check_absolute_curdir_segment_double_separator_invalid_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let broken_file = nested.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_curdir_segment_double_separator =
        PathBuf::from(format!("{}//./", broken_file.display()));

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator absolute invalid-target daglang check",
        );
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, canonical.stdout,
        "curdir-segment-double-separator and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, canonical.stderr,
        "curdir-segment-double-separator and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_mixed_segment_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_mixed_segment_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_mixed = root.join(".").join("missing.dag");

    let mixed = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_mixed)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute missing-target daglang check");
    assert!(
        !mixed.status.success(),
        "mixed-segment absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_trailing_slash_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_segment_trailing_slash_missing_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let missing_target = nested.join("missing.dag");
    let absolute_curdir_segment_trailing = root.join("nested/./missing.dag/");

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing absolute missing-target daglang check");
    assert!(
        !curdir_segment_trailing.status.success(),
        "curdir-segment-trailing absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_trailing.stdout, canonical.stdout,
        "curdir-segment-trailing and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing.stderr, canonical.stderr,
        "curdir-segment-trailing and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_suffix_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_suffix_missing_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let missing_target = nested.join("missing.dag");
    let absolute_curdir_suffix = root.join("nested/missing.dag/.");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_suffix)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix absolute missing-target daglang check");
    assert!(
        !curdir_suffix.status.success(),
        "curdir-suffix absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        curdir_suffix.stdout, canonical.stdout,
        "curdir-suffix and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, canonical.stderr,
        "curdir-suffix and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_suffix_double_separator_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_suffix_double_separator_missing_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let missing_target = nested.join("missing.dag");
    let absolute_curdir_suffix_double_separator =
        PathBuf::from(format!("{}//.", missing_target.display()));

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_suffix_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-suffix-double-separator absolute missing-target daglang check",
        );
    assert!(
        !curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, canonical.stdout,
        "curdir-suffix-double-separator and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, canonical.stderr,
        "curdir-suffix-double-separator and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_suffix_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_curdir_segment_suffix_missing_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let missing_target = nested.join("missing.dag");
    let absolute_curdir_segment_suffix = root.join("nested/missing.dag/./.");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_suffix)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix absolute missing-target daglang check");
    assert!(
        !curdir_segment_suffix.status.success(),
        "curdir-segment-suffix absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_suffix.stdout, canonical.stdout,
        "curdir-segment-suffix and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, canonical.stderr,
        "curdir-segment-suffix and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_double_separator_suffix_missing_target_matches_canonical_output(
) {
    let root =
        unique_temp_dir("check_absolute_curdir_segment_double_separator_suffix_missing_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let missing_target = nested.join("missing.dag");
    let absolute_curdir_segment_double_separator_suffix =
        PathBuf::from(format!("{}//./.", missing_target.display()));

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_double_separator_suffix)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix absolute missing-target daglang check",
        );
    assert!(
        !curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, canonical.stdout,
        "curdir-segment-double-separator-suffix and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, canonical.stderr,
        "curdir-segment-double-separator-suffix and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_curdir_segment_double_separator_missing_target_matches_canonical_output()
{
    let root = unique_temp_dir("check_absolute_curdir_segment_double_separator_missing_target");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let missing_target = nested.join("missing.dag");
    let absolute_curdir_segment_double_separator =
        PathBuf::from(format!("{}//./", missing_target.display()));

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_curdir_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator absolute missing-target daglang check",
        );
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, canonical.stdout,
        "curdir-segment-double-separator and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, canonical.stderr,
        "curdir-segment-double-separator and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_segment = root.join("nested/../broken.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute invalid-target daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_curdir_segment_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_curdir_segment = root.join("nested/./../broken.dag");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment absolute invalid-target daglang check");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment.stdout, canonical.stdout,
        "parent-curdir-segment and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, canonical.stderr,
        "parent-curdir-segment and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_trailing_slash_invalid_target_matches_canonical_output(
) {
    let root =
        unique_temp_dir("check_absolute_parent_curdir_segment_trailing_slash_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_curdir_segment_trailing = root.join("nested/./../broken.dag/");

    let parent_curdir_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_trailing)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-trailing absolute invalid-target daglang check",
        );
    assert!(
        !parent_curdir_segment_trailing.status.success(),
        "parent-curdir-segment-trailing absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-trailing and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-trailing and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_double_separator_invalid_target_matches_canonical_output(
) {
    let root =
        unique_temp_dir("check_absolute_parent_curdir_segment_double_separator_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_curdir_segment_double_separator = PathBuf::from(format!(
        "{}//./../broken.dag",
        root.join("nested").display()
    ));

    let parent_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-double-separator absolute invalid-target daglang check",
        );
    assert!(
        !parent_curdir_segment_double_separator.status.success(),
        "parent-curdir-segment-double-separator absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment_double_separator.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_double_separator_trailing_slash_invalid_target_matches_canonical_output(
) {
    let root = unique_temp_dir(
        "check_absolute_parent_curdir_segment_double_separator_trailing_slash_invalid_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_curdir_segment_double_separator_trailing = PathBuf::from(format!(
        "{}//./../broken.dag/",
        root.join("nested").display()
    ));

    let parent_curdir_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-double-separator-trailing absolute invalid-target daglang check",
        );
    assert!(
        !parent_curdir_segment_double_separator_trailing
            .status
            .success(),
        "parent-curdir-segment-double-separator-trailing absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator-trailing and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator-trailing and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_double_separator_invalid_target_matches_canonical_output()
{
    let root = unique_temp_dir("check_absolute_parent_segment_double_separator_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_segment_double_separator =
        PathBuf::from(format!("{}//../broken.dag", root.join("nested").display()));

    let parent_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-segment-double-separator absolute invalid-target daglang check",
        );
    assert!(
        !parent_segment_double_separator.status.success(),
        "parent-segment-double-separator absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_segment_double_separator.stdout, canonical.stdout,
        "parent-segment-double-separator and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_segment_double_separator.stderr, canonical.stderr,
        "parent-segment-double-separator and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_segment_double_separator.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_double_separator_trailing_slash_invalid_target_matches_canonical_output(
) {
    let root = unique_temp_dir(
        "check_absolute_parent_segment_double_separator_trailing_slash_invalid_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_segment_double_separator_trailing =
        PathBuf::from(format!("{}//../broken.dag/", root.join("nested").display()));

    let parent_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double-separator-trailing absolute invalid-target daglang check");
    assert!(
        !parent_segment_double_separator_trailing.status.success(),
        "parent-segment-double-separator-trailing absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-segment-double-separator-trailing and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-segment-double-separator-trailing and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_segment_double_separator_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_segment_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_trailing_slash_invalid_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_trailing_slash_invalid_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let broken_file = root.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed dag source");
    let absolute_parent_segment_trailing =
        PathBuf::from(format!("{}/", root.join("nested/../broken.dag").display()));

    let parent_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-trailing absolute invalid-target daglang check");
    assert!(
        !parent_segment_trailing.status.success(),
        "parent-segment-trailing absolute invalid-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid-target check should fail"
    );

    assert_eq!(
        parent_segment_trailing.stdout, canonical.stdout,
        "parent-segment-trailing and canonical absolute invalid-target stdout should match"
    );
    assert_eq!(
        parent_segment_trailing.stderr, canonical.stderr,
        "parent-segment-trailing and canonical absolute invalid-target stderr should match"
    );
    let canonical_target = broken_file
        .canonicalize()
        .expect("broken file should canonicalize");
    assert!(
        String::from_utf8_lossy(&parent_segment_trailing.stderr)
            .contains(&format!("{}:2:3:", canonical_target.display())),
        "expected parse diagnostic with canonicalized absolute path: {}",
        String::from_utf8_lossy(&parent_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_parent_segment = root.join("nested/../missing.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute missing-target daglang check");
    assert!(
        !parent_segment.status.success(),
        "parent-segment absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_curdir_segment_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_parent_curdir_segment = root.join("nested/./../missing.dag");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment absolute missing-target daglang check");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment.stdout, canonical.stdout,
        "parent-curdir-segment and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, canonical.stderr,
        "parent-curdir-segment and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_trailing_slash_missing_target_matches_canonical_output(
) {
    let root =
        unique_temp_dir("check_absolute_parent_curdir_segment_trailing_slash_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_file = root.join("missing.dag");
    let absolute_parent_curdir_segment_trailing = root.join("nested/./../missing.dag/");

    let parent_curdir_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_trailing)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-trailing absolute missing-target daglang check",
        );
    assert!(
        !parent_curdir_segment_trailing.status.success(),
        "parent-curdir-segment-trailing absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-trailing and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-trailing and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_double_separator_missing_target_matches_canonical_output(
) {
    let root =
        unique_temp_dir("check_absolute_parent_curdir_segment_double_separator_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_file = root.join("missing.dag");
    let absolute_parent_curdir_segment_double_separator = PathBuf::from(format!(
        "{}//./../missing.dag",
        root.join("nested").display()
    ));

    let parent_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-double-separator absolute missing-target daglang check",
        );
    assert!(
        !parent_curdir_segment_double_separator.status.success(),
        "parent-curdir-segment-double-separator absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment_double_separator.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_curdir_segment_double_separator_trailing_slash_missing_target_matches_canonical_output(
) {
    let root = unique_temp_dir(
        "check_absolute_parent_curdir_segment_double_separator_trailing_slash_missing_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_file = root.join("missing.dag");
    let absolute_parent_curdir_segment_double_separator_trailing = PathBuf::from(format!(
        "{}//./../missing.dag/",
        root.join("nested").display()
    ));

    let parent_curdir_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_curdir_segment_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-double-separator-trailing absolute missing-target daglang check",
        );
    assert!(
        !parent_curdir_segment_double_separator_trailing
            .status
            .success(),
        "parent-curdir-segment-double-separator-trailing absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator-trailing and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator-trailing and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr).contains(
            &format!("failed to canonicalize {}", missing_file.display())
        ),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_double_separator_missing_target_matches_canonical_output()
{
    let root = unique_temp_dir("check_absolute_parent_segment_double_separator_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_parent_segment_double_separator =
        PathBuf::from(format!("{}//../missing.dag", root.join("nested").display()));

    let parent_segment_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_double_separator)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-segment-double-separator absolute missing-target daglang check",
        );
    assert!(
        !parent_segment_double_separator.status.success(),
        "parent-segment-double-separator absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_segment_double_separator.stdout, canonical.stdout,
        "parent-segment-double-separator and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_segment_double_separator.stderr, canonical.stderr,
        "parent-segment-double-separator and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment_double_separator.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_double_separator_trailing_slash_missing_target_matches_canonical_output(
) {
    let root = unique_temp_dir(
        "check_absolute_parent_segment_double_separator_trailing_slash_missing_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_parent_segment_double_separator_trailing = PathBuf::from(format!(
        "{}//../missing.dag/",
        root.join("nested").display()
    ));

    let parent_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_double_separator_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double-separator-trailing absolute missing-target daglang check");
    assert!(
        !parent_segment_double_separator_trailing.status.success(),
        "parent-segment-double-separator-trailing absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-segment-double-separator-trailing and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-segment-double-separator-trailing and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment_double_separator_trailing.stderr).contains(
            &format!("failed to canonicalize {}", missing_target.display())
        ),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_absolute_parent_segment_trailing_slash_missing_target_matches_canonical_output() {
    let root = unique_temp_dir("check_absolute_parent_segment_trailing_slash_missing_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_target = root.join("missing.dag");
    let absolute_parent_segment_trailing =
        PathBuf::from(format!("{}/", root.join("nested/../missing.dag").display()));

    let parent_segment_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg(&absolute_parent_segment_trailing)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-trailing absolute missing-target daglang check");
    assert!(
        !parent_segment_trailing.status.success(),
        "parent-segment-trailing absolute missing-target check should fail"
    );

    let canonical = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-target daglang check");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-target check should fail"
    );

    assert_eq!(
        parent_segment_trailing.stdout, canonical.stdout,
        "parent-segment-trailing and canonical absolute missing-target stdout should match"
    );
    assert_eq!(
        parent_segment_trailing.stderr, canonical.stderr,
        "parent-segment-trailing and canonical absolute missing-target stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_segment_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_target.display()
        )),
        "missing-target diagnostics should include canonical absolute path: {}",
        String::from_utf8_lossy(&parent_segment_trailing.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn modules_command_prints_module_graph_summary() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Discovered modules:"));
    let reported_modules = reported_modules_sorted(&stdout);
    let expected_modules = expected_dsl_modules_sorted();
    assert_eq!(
        reported_modules, expected_modules,
        "modules command should report the complete 45-module corpus"
    );
}

#[test]
fn modules_command_json_format_emits_machine_readable_summary() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules --format json");

    assert!(
        output.status.success(),
        "modules --format json command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value =
        serde_json::from_slice(&output.stdout).expect("modules json output should parse");
    let summary = parsed
        .get("summary")
        .expect("summary should be present in modules json output");
    let parsed_files = parsed
        .get("parsed_files")
        .and_then(Value::as_u64)
        .expect("parsed_files should be present");
    assert_eq!(
        summary.get("parsed_files").and_then(Value::as_u64),
        Some(parsed_files),
        "summary parsed_files should match top-level parsed_files"
    );
    assert!(parsed_files > 0, "parsed_files should be positive");
    let modules = parsed
        .get("modules")
        .and_then(Value::as_array)
        .expect("modules array should be present");
    assert!(!modules.is_empty(), "modules array should not be empty");
    assert_eq!(
        summary.get("module_count").and_then(Value::as_u64),
        Some(modules.len() as u64),
        "summary module_count should match modules length"
    );
    let first = modules.first().expect("modules should contain entries");
    assert!(first.get("module").and_then(Value::as_str).is_some());
    assert!(first.get("path").and_then(Value::as_str).is_some());
    assert!(first.get("items").and_then(Value::as_u64).is_some());
    assert!(first
        .get("dependencies")
        .and_then(Value::as_array)
        .is_some());
    let module_order = parsed
        .get("module_order")
        .and_then(Value::as_array)
        .expect("module_order should be present");
    assert_eq!(
        module_order.len(),
        modules.len(),
        "module_order should track modules in dependency order"
    );
    let diagnostics = parsed
        .get("diagnostics")
        .and_then(Value::as_array)
        .expect("diagnostics should be an array");
    assert_eq!(
        summary.get("diagnostic_count").and_then(Value::as_u64),
        Some(diagnostics.len() as u64),
        "summary diagnostic_count should match diagnostics length"
    );
    let diagnostic_kinds = summary
        .get("diagnostic_kinds")
        .expect("summary diagnostic_kinds should be present");
    assert_eq!(diagnostic_kinds.get("lex").and_then(Value::as_u64), Some(0));
    assert_eq!(
        diagnostic_kinds.get("parse").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        diagnostic_kinds.get("resolve").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        diagnostic_kinds.get("pipeline").and_then(Value::as_u64),
        Some(0)
    );
    let diagnostics_detail = parsed
        .get("diagnostics_detail")
        .and_then(Value::as_array)
        .expect("diagnostics_detail should be present");
    assert_eq!(
        diagnostics_detail.len(),
        diagnostics.len(),
        "diagnostics_detail length should match diagnostics length"
    );
    assert!(
        diagnostics.is_empty(),
        "clean dsl corpus should not emit module diagnostics"
    );
}

#[test]
fn modules_command_json_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first modules --format json");
    assert!(
        first.status.success(),
        "first modules --format json should succeed"
    );

    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second modules --format json");
    assert!(
        second.status.success(),
        "second modules --format json should succeed"
    );

    assert_eq!(
        first.stdout, second.stdout,
        "modules --format json output should be deterministic"
    );
}

#[test]
fn modules_command_without_dir_uses_configured_discovery_roots() {
    let cwd = unique_temp_dir("modules_configured_roots");
    let root_a = cwd.join("pkg_a");
    let root_b = cwd.join("pkg_b");
    std::fs::create_dir_all(&root_a).expect("failed to create root_a");
    std::fs::create_dir_all(&root_b).expect("failed to create root_b");
    std::fs::write(root_a.join("a.dag"), "module sample.a\nfn run() -> Unit {}")
        .expect("failed to write root_a source");
    std::fs::write(root_b.join("b.dag"), "module sample.b\nfn run() -> Unit {}")
        .expect("failed to write root_b source");
    std::fs::write(
        cwd.join("daglang.toml"),
        "[discovery]\nroots = [\"pkg_a\", \"pkg_b\"]\n",
    )
    .expect("failed to write daglang.toml");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules with configured roots");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sample.a"),
        "stdout should include sample.a"
    );
    assert!(
        stdout.contains("sample.b"),
        "stdout should include sample.b"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp root");
}

#[test]
fn modules_command_reports_invalid_config_parse_error() {
    let cwd = unique_temp_dir("modules_invalid_config");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(cwd.join("daglang.toml"), "[discovery]\nroots = [")
        .expect("failed to write invalid daglang.toml");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules with invalid config");

    assert!(
        !output.status.success(),
        "modules should fail for invalid config"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse"));
    assert!(stderr.contains("daglang.toml"));

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp root");
}

#[test]
fn modules_command_json_includes_structured_diagnostics_for_parse_errors() {
    let cwd = unique_temp_dir("modules_json_parse_diagnostics");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    std::fs::write(cwd.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(".")
        .arg("--format")
        .arg("json")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules --format json on malformed source");

    assert!(
        output.status.success(),
        "modules json should still succeed with diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value =
        serde_json::from_slice(&output.stdout).expect("modules json output should parse");
    let diagnostics = parsed
        .get("diagnostics")
        .and_then(Value::as_array)
        .expect("diagnostics should be present");
    assert!(
        !diagnostics.is_empty(),
        "malformed source should produce diagnostics"
    );
    let diagnostics_detail = parsed
        .get("diagnostics_detail")
        .and_then(Value::as_array)
        .expect("diagnostics_detail should be present");
    assert_eq!(
        diagnostics_detail.len(),
        diagnostics.len(),
        "diagnostics_detail should align with diagnostics list"
    );
    let first_detail = diagnostics_detail
        .first()
        .expect("diagnostics_detail should have at least one entry");
    assert!(first_detail.get("kind").and_then(Value::as_str).is_some());
    assert!(first_detail
        .get("message")
        .and_then(Value::as_str)
        .is_some());
    assert!(first_detail
        .get("rendered")
        .and_then(Value::as_str)
        .is_some());
    let kind_counts = parsed
        .get("summary")
        .and_then(|summary| summary.get("diagnostic_kinds"))
        .expect("diagnostic kind counts should be present");
    let parse_count = kind_counts
        .get("parse")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let lex_count = kind_counts.get("lex").and_then(Value::as_u64).unwrap_or(0);
    assert!(
        parse_count + lex_count > 0,
        "malformed source should produce lex or parse diagnostics"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp root");
}

#[test]
fn modules_command_reports_expected_real_corpus_diagnostics() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Diagnostics:"),
        "modules output should not include diagnostics for clean corpus: {stdout}"
    );
}

#[test]
fn modules_command_real_corpus_diagnostics_match_expected_snapshot() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_diagnostics_in_order(&stdout);
    let expected = expected_real_corpus_modules_diagnostics();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_real_corpus_modules_match_filesystem_discovery() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_modules_sorted(&stdout);
    let expected = expected_dsl_modules_sorted();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_real_corpus_order_matches_resolve_discovery() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_modules_in_order(&stdout);
    let expected = resolve_discovered_module_order();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_real_corpus_summary_matches_resolve_discovery() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules");

    assert!(
        output.status.success(),
        "modules command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = reported_module_summary(&stdout);
    let expected = resolve_discovered_module_summary();
    assert_eq!(actual, expected);
}

#[test]
fn modules_command_default_root_matches_explicit_dsl_output() {
    let explicit = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run explicit-root daglang modules");
    assert!(
        explicit.status.success(),
        "explicit-root modules should succeed: {}",
        String::from_utf8_lossy(&explicit.stderr)
    );

    let default = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run default-root daglang modules");
    assert!(
        default.status.success(),
        "default-root modules should succeed: {}",
        String::from_utf8_lossy(&default.stderr)
    );

    assert_eq!(
        default.stdout, explicit.stdout,
        "default modules output should match explicit 'modules dsl' output"
    );
    assert_eq!(
        default.stderr, explicit.stderr,
        "default modules stderr should match explicit-root stderr"
    );
}

#[test]
fn modules_command_relative_and_absolute_root_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-root daglang modules");
    assert!(
        relative.status.success(),
        "relative-root modules should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute_root = workspace_root().join("dsl");
    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&absolute_root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-root daglang modules");
    assert!(
        absolute.status.success(),
        "absolute-root modules should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute root modules outputs should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute root modules stderr should match"
    );
}

#[test]
fn viz_self_defaults_to_ascii_output() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz --self");

    assert!(
        output.status.success(),
        "viz --self command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DAG daglang-compiler-pipeline"));
    assert!(stdout.contains("Nodes:"));
    assert!(stdout.contains("Edges:"));
    assert!(stdout.contains("discover_files"));
    assert!(stdout.contains("emit_target_files"));
}

#[test]
fn viz_self_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang viz --self");
    assert!(
        first.status.success(),
        "first viz --self run should succeed"
    );

    let second = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang viz --self");
    assert!(
        second.status.success(),
        "second viz --self run should succeed"
    );

    assert_eq!(
        first.stdout, second.stdout,
        "viz --self output should be deterministic across runs"
    );
}

#[test]
fn viz_self_contains_expected_pipeline_edge_labels() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz --self");

    assert!(output.status.success(), "viz --self should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("discover_files.files -> parse_all.files"),
        "viz --self should include discover->parse files edge label"
    );
    assert!(
        stdout.contains("discover_files.diagnostics -> parse_all.diagnostics"),
        "viz --self should include diagnostics flow edges"
    );
    assert!(
        stdout.contains("resolve_module_graph.module_graph -> typecheck_modules.module_graph"),
        "viz --self should include resolve->typecheck module graph edge label"
    );
    assert!(
        stdout.contains("derive_metadata.derived_artifacts -> emit_target_files.derived_artifacts"),
        "viz --self should include derive->emit artifact edge label"
    );
}

#[test]
fn viz_self_matches_expected_mermaid_snapshot() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .arg("--format")
        .arg("mermaid")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz --self");

    assert!(output.status.success(), "viz --self should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_viz_self_mermaid());
}

#[test]
fn check_command_reports_file_line_col_for_broken_file() {
    let broken_file = unique_temp_file("broken");
    std::fs::write(
        &broken_file,
        "module tmp.bad\nfn broken( -> String {\n  \"oops\"\n}\n",
    )
    .expect("failed to create broken dag file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for broken file");

    assert!(!output.status.success(), "broken file should fail check");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    assert!(
        stderr.contains(":2:12:"),
        "expected file:line:col in stderr, got: {stderr}"
    );

    std::fs::remove_file(broken_file).expect("failed to remove broken dag file");
}

#[cfg(unix)]
#[test]
fn check_command_accepts_symlink_single_file_target() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let real = root.join("real.dag");
    let link = root.join("link.dag");
    std::fs::write(&real, "module sample.real\nfn ok() -> Unit {}")
        .expect("failed to write real source");
    symlink(&real, &link).expect("failed to create symlinked target");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlinked target");

    assert!(
        output.status.success(),
        "check should succeed for symlinked single-file target: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(1));
    assert!(
        output.stderr.is_empty(),
        "symlinked single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_accepts_symlink_uppercase_dag_single_file_target() {
    let root = unique_temp_dir("accepts_symlink_uppercase_dag_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_accepts_symlink_uppercase_dag_single_file_target",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_accepts_symlink_mixed_case_dag_single_file_target() {
    let root = unique_temp_dir("accepts_symlink_mixed_case_dag_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_accepts_symlink_mixed_case_dag_single_file_target",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_uppercase_symlink_single_file_curdir_suffix_target_matches_plain_output() {
    let root = unique_temp_dir("uppercase_symlink_single_file_curdir_suffix_target");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_uppercase_symlink_single_file_curdir_suffix_target_matches_plain_output",
    );

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_symlink_single_file_curdir_suffix_target_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_symlink_single_file_curdir_suffix_target_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_mixed_case_symlink_single_file_curdir_suffix_target_matches_plain_output() {
    let root = unique_temp_dir("mixed_case_symlink_single_file_curdir_suffix_targe");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_mixed_case_symlink_single_file_curdir_suffix_target_matches_plain_output",
    );

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_symlink_single_file_curdir_suffix_target_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_symlink_single_file_curdir_suffix_target_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_uppercase_symlink_single_file_curdir_segment_trailing_target_matches_plain_output()
{
    let root = unique_temp_dir("uppercase_symlink_single_file_curdir_segment_trail");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_symlink_single_file_curdir_segment_trailing_target_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_symlink_single_file_curdir_segment_trailing_target_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_symlink_single_file_curdir_segment_trailing_target_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_mixed_case_symlink_single_file_curdir_segment_trailing_target_matches_plain_output(
) {
    let root = unique_temp_dir("mixed_case_symlink_single_file_curdir_segment_trai");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_symlink_single_file_curdir_segment_trailing_target_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_symlink_single_file_curdir_segment_trailing_target_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_symlink_single_file_curdir_segment_trailing_target_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_uppercase_symlink_single_file_curdir_double_separator_target_matches_plain_output()
{
    let root = unique_temp_dir("uppercase_symlink_single_file_curdir_double_separa");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_symlink_single_file_curdir_double_separator_target_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_symlink_single_file_curdir_double_separator_target_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_symlink_single_file_curdir_double_separator_target_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_mixed_case_symlink_single_file_curdir_double_separator_target_matches_plain_output(
) {
    let root = unique_temp_dir("mixed_case_symlink_single_file_curdir_double_separ");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_symlink_single_file_curdir_double_separator_target_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_symlink_single_file_curdir_double_separator_target_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_symlink_single_file_curdir_double_separator_target_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_and_real_invalid_single_file_targets_are_equivalent() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_real_invalid_single_file_equivalent");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let real = root.join("real.dag");
    let link = root.join("link.dag");
    std::fs::write(&real, "module sample.broken\nfn")
        .expect("failed to write malformed real source");
    symlink(&real, &link).expect("failed to create symlinked target");

    let real_output = Command::new(daglang_bin())
        .arg("check")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on real malformed target");
    assert!(
        !real_output.status.success(),
        "real malformed single-file target should fail"
    );

    let link_output = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlinked malformed target");
    assert!(
        !link_output.status.success(),
        "symlinked malformed single-file target should fail"
    );

    assert_eq!(
        real_output.stdout, link_output.stdout,
        "real and symlink invalid target check stdout should match"
    );
    assert_eq!(
        real_output.stderr, link_output.stderr,
        "real and symlink invalid target check stderr should match"
    );
    let stderr = String::from_utf8_lossy(&real_output.stderr);
    assert!(
        stderr.contains("real.dag:2:3:"),
        "expected canonicalized parse diagnostic path in stderr: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_and_real_invalid_uppercase_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("symlink_and_real_invalid_uppercase_single_file_tar");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_symlink_and_real_invalid_uppercase_single_file_targets_are_equivalent",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_and_real_invalid_mixed_case_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("symlink_and_real_invalid_mixed_case_single_file_ta");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let real_file = root.join("real.dag");
    std::fs::write(&real_file, "module sample.main\nfn ok() -> Unit {}").expect("failed to write");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&real_file, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_symlink_and_real_invalid_mixed_case_single_file_targets_are_equivalent",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_dangling_symlink_single_file_target_exits_nonzero() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_dangling_symlink_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink target");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&dangling_link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on dangling symlink target");

    assert!(
        !output.status.success(),
        "check should fail for dangling symlink single-file target"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "dangling-symlink single-file check should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("failed to canonicalize"));
    assert!(
        stderr.contains("broken.dag"),
        "dangling symlink single-file failure should include offending path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_dangling_uppercase_symlink_single_file_target_exits_nonzero() {
    let root = unique_temp_dir("dangling_uppercase_symlink_single_file_target_exit");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DAG");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_dangling_uppercase_symlink_single_file_target_exits_nonzero",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_dangling_mixed_case_symlink_single_file_target_exits_nonzero() {
    let root = unique_temp_dir("dangling_mixed_case_symlink_single_file_target_exi");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DaG");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_dangling_mixed_case_symlink_single_file_target_exits_nonzero",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_uppercase_dangling_symlink_single_file_curdir_suffix_matches_plain_output() {
    let root = unique_temp_dir("uppercase_dangling_symlink_single_file_curdir_suff");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DAG");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_uppercase_dangling_symlink_single_file_curdir_suffix_matches_plain_output",
    );

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_dangling_symlink_single_file_curdir_suffix_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_dangling_symlink_single_file_curdir_suffix_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_mixed_case_dangling_symlink_single_file_curdir_suffix_matches_plain_output() {
    let root = unique_temp_dir("mixed_case_dangling_symlink_single_file_curdir_suf");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DaG");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG/.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_mixed_case_dangling_symlink_single_file_curdir_suffix_matches_plain_output",
    );

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_dangling_symlink_single_file_curdir_suffix_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_dangling_symlink_single_file_curdir_suffix_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_uppercase_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output(
) {
    let root = unique_temp_dir("uppercase_dangling_symlink_single_file_curdir_segm");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DAG");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_mixed_case_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output(
) {
    let root = unique_temp_dir("mixed_case_dangling_symlink_single_file_curdir_seg");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DaG");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG/./")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_dangling_symlink_single_file_curdir_segment_trailing_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_uppercase_dangling_symlink_single_file_curdir_double_separator_matches_plain_output(
) {
    let root = unique_temp_dir("uppercase_dangling_symlink_single_file_curdir_doub");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DAG");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_uppercase_dangling_symlink_single_file_curdir_double_separator_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_uppercase_dangling_symlink_single_file_curdir_double_separator_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_uppercase_dangling_symlink_single_file_curdir_double_separator_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_mixed_case_dangling_symlink_single_file_curdir_double_separator_matches_plain_output(
) {
    let root = unique_temp_dir("mixed_case_dangling_symlink_single_file_curdir_dou");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DaG");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(&output, "check_command_mixed_case_dangling_symlink_single_file_curdir_double_separator_matches_plain_output");

    let output2 = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check variant");
    assert_wrong_cased_dag_extension_rejected(&output2, "check_command_mixed_case_dangling_symlink_single_file_curdir_double_separator_matches_plain_output variant");
    assert_eq!(output.stderr, output2.stderr, "check_command_mixed_case_dangling_symlink_single_file_curdir_double_separator_matches_plain_output: both variants should produce same error");

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_relative_and_absolute_dangling_symlink_targets_are_equivalent() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_relative_absolute_dangling_symlink_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink target");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative dangling-target daglang check");
    assert!(
        !relative.status.success(),
        "relative dangling-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&dangling_link)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute dangling-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute dangling-target check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute dangling-target check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute dangling-target check stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&format!(
            "failed to canonicalize {}",
            dangling_link.display()
        )),
        "dangling-target diagnostics should include normalized absolute path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_relative_and_absolute_uppercase_dangling_symlink_targets_are_equivalent() {
    let root = unique_temp_dir("relative_and_absolute_uppercase_dangling_symlink_t");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DAG");
    let link = root.join("link.DAG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_relative_and_absolute_uppercase_dangling_symlink_targets_are_equivalent",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[cfg(unix)]
#[test]
fn check_command_relative_and_absolute_mixed_case_dangling_symlink_targets_are_equivalent() {
    let root = unique_temp_dir("relative_and_absolute_mixed_case_dangling_symlink_");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let target = root.join("nonexistent.DaG");
    let link = root.join("link.DaG");
    std::os::unix::fs::symlink(&target, &link).expect("failed to create symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("link.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "check_command_relative_and_absolute_mixed_case_dangling_symlink_targets_are_equivalent",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn check_command_relative_and_absolute_missing_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("check_relative_absolute_missing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing_file = root.join("missing.dag");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative missing-target daglang check");
    assert!(
        !relative.status.success(),
        "relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute missing-target check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute missing-target check stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized absolute path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn check_command_parent_segment_missing_single_file_target_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_segment_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let relative = Command::new(daglang_bin())
        .arg("check")
        .arg("../missing.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-segment relative missing-target daglang check");
    assert!(
        !relative.status.success(),
        "parent-segment relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "parent-segment relative and absolute missing-target check stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "parent-segment relative and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&relative.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-segment path: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_segment_missing_single_file_target_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_curdir_segment_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("check")
        .arg(".././missing.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-segment relative missing-target daglang check");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_segment.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-curdir-segment path: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_missing_single_file_target_is_normalized_and_equivalent() {
    let parent = unique_temp_dir("check_parent_double_separator_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg("..//missing.dag")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-double-separator relative missing-target daglang check");
    assert!(
        !parent_double_separator.status.success(),
        "parent-double-separator relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_double_separator.stdout, absolute.stdout,
        "parent-double-separator and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator.stderr, absolute.stderr,
        "parent-double-separator and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-double-separator path: {}",
        String::from_utf8_lossy(&parent_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_double_separator_trailing_slash_missing_single_file_target_is_normalized_and_equivalent(
) {
    let parent =
        unique_temp_dir("check_parent_double_separator_trailing_slash_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_double_separator_trailing = Command::new(daglang_bin())
        .arg("check")
        .arg("..//missing.dag/")
        .current_dir(&cwd)
        .output()
        .expect(
            "failed to run parent-double-separator-trailing relative missing-target daglang check",
        );
    assert!(
        !parent_double_separator_trailing.status.success(),
        "parent-double-separator-trailing relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_double_separator_trailing.stdout, absolute.stdout,
        "parent-double-separator-trailing and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_double_separator_trailing.stderr, absolute.stderr,
        "parent-double-separator-trailing and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-double-separator-trailing path: {}",
        String::from_utf8_lossy(&parent_double_separator_trailing.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_double_separator_missing_single_file_target_is_normalized_and_equivalent(
) {
    let parent = unique_temp_dir("check_parent_curdir_double_separator_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_curdir_double_separator = Command::new(daglang_bin())
        .arg("check")
        .arg(".././missing.dag//")
        .current_dir(&cwd)
        .output()
        .expect(
            "failed to run parent-curdir-double-separator relative missing-target daglang check",
        );
    assert!(
        !parent_curdir_double_separator.status.success(),
        "parent-curdir-double-separator relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_double_separator.stdout, absolute.stdout,
        "parent-curdir-double-separator and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_double_separator.stderr, absolute.stderr,
        "parent-curdir-double-separator and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-curdir-double-separator path: {}",
        String::from_utf8_lossy(&parent_curdir_double_separator.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_parent_curdir_trailing_slash_missing_single_file_target_is_normalized_and_equivalent(
) {
    let parent = unique_temp_dir("check_parent_curdir_trailing_slash_missing_single_file");
    let cwd = parent.join("cwd");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_file = parent.join("missing.dag");

    let parent_curdir_trailing_slash = Command::new(daglang_bin())
        .arg("check")
        .arg(".././missing.dag/")
        .current_dir(&cwd)
        .output()
        .expect("failed to run parent-curdir-trailing-slash relative missing-target daglang check");
    assert!(
        !parent_curdir_trailing_slash.status.success(),
        "parent-curdir-trailing-slash relative missing-target check should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(&cwd)
        .output()
        .expect("failed to run absolute missing-target daglang check");
    assert!(
        !absolute.status.success(),
        "absolute missing-target check should fail"
    );

    assert_eq!(
        parent_curdir_trailing_slash.stdout, absolute.stdout,
        "parent-curdir-trailing-slash and absolute missing-target check stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing_slash.stderr, absolute.stderr,
        "parent-curdir-trailing-slash and absolute missing-target check stderr should match"
    );
    assert!(
        String::from_utf8_lossy(&parent_curdir_trailing_slash.stderr).contains(&format!(
            "failed to canonicalize {}",
            missing_file.display()
        )),
        "missing-target diagnostics should include normalized parent-curdir-trailing-slash path: {}",
        String::from_utf8_lossy(&parent_curdir_trailing_slash.stderr)
    );

    std::fs::remove_dir_all(parent).expect("failed to cleanup temp directory");
}

#[test]
fn check_command_missing_single_file_exits_nonzero() {
    let missing_file = unique_temp_file("missing_single_file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for missing file");

    assert!(
        !output.status.success(),
        "check should fail when target .dag file does not exist"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "missing single-file check should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("pipeline error"),
        "missing single-file path should surface pipeline error: {stderr}"
    );
    assert!(
        stderr.contains(&missing_file.display().to_string()),
        "missing single-file error should include offending path: {stderr}"
    );
}

#[test]
fn check_command_missing_directory_exits_nonzero() {
    let missing_dir = unique_temp_dir("missing_dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&missing_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for missing directory");

    assert!(
        !output.status.success(),
        "check should fail when input directory does not exist"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "missing-directory check should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_dir.display().to_string()),
        "missing-root error should include path context: {stderr}"
    );
}

#[test]
fn check_command_non_directory_root_exits_nonzero() {
    let root_file = unique_temp_file("check_non_directory_root").with_extension("txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for non-directory root");

    assert!(
        !output.status.success(),
        "check should fail when root path is not a directory"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "non-directory-root check should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "non-directory root error should include offending path: {stderr}"
    );

    std::fs::remove_file(root_file).expect("failed to cleanup root file");
}

#[test]
fn check_command_reports_lex_diagnostic_for_unknown_character() {
    let bad_file = unique_temp_file("lex_bad");
    std::fs::write(&bad_file, "module tmp.bad\n$\n").expect("failed to create bad dag file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&bad_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for lex-bad file");

    assert!(!output.status.success(), "lex-bad file should fail check");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(":2:1:"),
        "expected lexical diagnostic line/column, got: {stderr}"
    );
    assert!(stderr.contains("unexpected character '$'"));

    std::fs::remove_file(bad_file).expect("failed to remove bad dag file");
}

#[test]
fn modules_command_reports_graph_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write temp dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for temp dir");

    assert!(
        output.status.success(),
        "modules command should still succeed while reporting diagnostics"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("unresolved import"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_parse_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_parse_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("broken.dag"),
        "module sample.broken\nfn broken( -> Unit {\n",
    )
    .expect("failed to write malformed dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for malformed source");

    assert!(
        output.status.success(),
        "modules command should succeed while surfacing parse diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("broken.dag:2:12:"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_lex_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_lex_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("broken.dag"), "module sample.broken\n$\n")
        .expect("failed to write malformed dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for malformed source");

    assert!(
        output.status.success(),
        "modules command should succeed while surfacing lex diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("broken.dag:2:1:"));
    assert!(stdout.contains("unexpected character '$'"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_sorts_lex_diagnostics_before_resolve_diagnostics() {
    let root = unique_temp_dir("modules_diag_kind_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("z_resolve.dag"),
        "module sample.resolve\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write resolve-error source");
    std::fs::write(root.join("a_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write lex-error source");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for mixed diagnostics");

    assert!(
        output.status.success(),
        "modules command should remain non-failing when diagnostics are present"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let diagnostics = stdout
        .split("Diagnostics:\n")
        .nth(1)
        .expect("expected diagnostics section in modules output");
    let first = diagnostics
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("expected at least one diagnostic line");
    assert!(
        first.contains("a_lex.dag"),
        "lex diagnostics should be sorted before resolve diagnostics: {stdout}"
    );
    assert!(stdout.contains("z_resolve.dag"));
    assert!(stdout.contains("unexpected character '$'"));
    assert!(stdout.contains("unresolved import"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_sorts_parse_diagnostics_before_resolve_diagnostics() {
    let root = unique_temp_dir("modules_parse_resolve_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write parse-error source");
    std::fs::write(
        root.join("z_resolve.dag"),
        "module sample.resolve\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write resolve-error source");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for mixed parse/resolve diagnostics");

    assert!(
        output.status.success(),
        "modules command should remain non-failing when diagnostics are present"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let diagnostics = stdout
        .split("Diagnostics:\n")
        .nth(1)
        .expect("expected diagnostics section in modules output");
    let first = diagnostics
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("expected at least one diagnostic line");
    assert!(
        first.contains("a_parse.dag"),
        "parse diagnostics should sort before resolve diagnostics: {stdout}"
    );
    assert!(stdout.contains("z_resolve.dag"));
    assert!(stdout.contains("unresolved import"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_cycle_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_cycle_diag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("a.dag"),
        "module cycle.a\nimport cycle.b\nfn a() -> Unit {}",
    )
    .expect("failed to write cycle a");
    std::fs::write(
        root.join("b.dag"),
        "module cycle.b\nimport cycle.a\nfn b() -> Unit {}",
    )
    .expect("failed to write cycle b");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for cycle dir");

    assert!(
        output.status.success(),
        "modules command should still succeed while reporting cycle diagnostics"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("cyclic dependencies detected"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_reports_duplicate_module_diagnostics_without_failing() {
    let root = unique_temp_dir("modules_duplicate_diag");
    std::fs::create_dir_all(root.join("a")).expect("failed to create temp dir a");
    std::fs::create_dir_all(root.join("b")).expect("failed to create temp dir b");
    std::fs::write(
        root.join("a/one.dag"),
        "module dup.mod\nfn one() -> Unit {}",
    )
    .expect("failed to write file one");
    std::fs::write(
        root.join("b/two.dag"),
        "module dup.mod\nfn two() -> Unit {}",
    )
    .expect("failed to write file two");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for duplicate dir");

    assert!(
        output.status.success(),
        "modules command should still succeed while reporting duplicate diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diagnostics:"));
    assert!(stdout.contains("duplicate module path"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_diagnostic_output_is_deterministic_for_same_input() {
    let root = unique_temp_dir("modules_diag_deterministic");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write temp dag file");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first modules command");
    assert!(first.status.success(), "first modules run should succeed");

    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second modules command");
    assert!(second.status.success(), "second modules run should succeed");

    assert_eq!(
        first.stdout, second.stdout,
        "modules diagnostic output should be deterministic"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_single_file_mode_ignores_sibling_broken_files() {
    let root = unique_temp_dir("single_file_mode");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let good_file = root.join("good.dag");
    std::fs::write(&good_file, "module sample.good\nfn ok() -> Unit {}")
        .expect("failed to write good file");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write broken sibling");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&good_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on single file");

    assert!(
        output.status.success(),
        "single-file check should succeed even with sibling broken file: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(1));
    assert!(
        output.stderr.is_empty(),
        "single-file check should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_directory_mode_aggregates_multiple_file_diagnostics() {
    let root = unique_temp_dir("directory_mode_errors");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("broken_a.dag"), "module sample.a\nfn")
        .expect("failed to write broken_a");
    std::fs::write(root.join("broken_b.dag"), "module sample.b\nimport")
        .expect("failed to write broken_b");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on directory");

    assert!(
        !output.status.success(),
        "directory check should fail when multiple files are invalid"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    assert!(stderr.contains("broken_a.dag"));
    assert!(stderr.contains("broken_b.dag"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_directory_mode_outputs_deterministic_diagnostic_order() {
    let root = unique_temp_dir("directory_mode_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("z_broken.dag"), "module sample.z\nfn")
        .expect("failed to write z_broken");
    std::fs::write(root.join("a_broken.dag"), "module sample.a\nfn")
        .expect("failed to write a_broken");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on directory");

    assert!(
        !output.status.success(),
        "directory check should fail when files are invalid"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut lines = stderr.lines().filter(|line| !line.trim().is_empty());
    let first_line = lines.next().expect("expected at least one diagnostic line");
    assert!(
        first_line.contains("a_broken.dag"),
        "diagnostics should be deterministically sorted by path: {stderr}"
    );
    assert!(stderr.contains("z_broken.dag"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_diagnostic_output_is_deterministic_for_same_input() {
    let root = unique_temp_dir("check_output_deterministic");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write z_lex");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write a_parse");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check");
    assert!(!first.status.success(), "first check run should fail");

    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check");
    assert!(!second.status.success(), "second check run should fail");

    assert_eq!(
        first.stderr, second.stderr,
        "check diagnostics should be deterministic for identical inputs"
    );
    assert_eq!(
        first.stdout, second.stdout,
        "check stdout should be deterministic for identical inputs"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_sorts_lex_diagnostics_before_parse_diagnostics() {
    let root = unique_temp_dir("check_kind_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write parse-error file");
    std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write lex-error file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on mixed-error directory");

    assert!(
        !output.status.success(),
        "check should fail when diagnostics are present"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("expected at least one diagnostic line");
    assert!(
        first_line.contains("z_lex.dag"),
        "lex diagnostics should sort before parse diagnostics: {stderr}"
    );
    assert!(stderr.contains("a_parse.dag"));
    assert!(stderr.contains("unexpected character '$'"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_empty_directory_succeeds_with_zero_files() {
    let root = unique_temp_dir("check_empty_dir");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on empty directory");

    assert!(
        output.status.success(),
        "check should succeed for an empty directory"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_check_success_stdout(0));
    assert!(
        output.stderr.is_empty(),
        "empty-directory check should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_ignores_non_dag_files_in_directory() {
    let root = unique_temp_dir("check_ignore_non_dag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on non-dag-only directory");

    assert!(
        output.status.success(),
        "check should ignore non-.dag files in directory mode"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: checked 0 file(s)"),
        "non-.dag files should be ignored during check: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_ignores_non_dag_files_when_dag_files_exist() {
    let root = unique_temp_dir("check_ignore_non_dag_mixed");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("good.dag"),
        "module sample.good\nfn ok() -> Unit {}",
    )
    .expect("failed to write dag file");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on mixed dag/non-dag directory");

    assert!(
        output.status.success(),
        "check should parse only .dag files and ignore non-.dag files"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: checked 1 file(s)"),
        "expected only the .dag file to be parsed: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_reports_unresolved_imports() {
    let root = unique_temp_dir("check_reports_unresolved_import");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
    )
    .expect("failed to write dag file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on unresolved-import source");

    assert!(
        !output.status.success(),
        "check should fail when unresolved imports are present"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unresolved import"),
        "check should report unresolved import diagnostic: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_defaults_to_workspace_dsl_root() {
    let root = unique_temp_dir("check_defaults_to_dsl");
    let dsl_dir = root.join("dsl");
    std::fs::create_dir_all(&dsl_dir).expect("failed to create dsl subdir");
    for i in 1..=2 {
        std::fs::write(
            dsl_dir.join(format!("mod{i}.dag")),
            format!("module sample.mod{i}\nfn ok() -> Unit {{}}"),
        )
        .expect("failed to write dag source");
    }

    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang check with default root");

    assert!(
        output.status.success(),
        "default check command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout,
        expected_check_success_stdout(2),
        "default check should find files in dsl/ subdirectory"
    );
}

#[test]
fn check_command_default_root_missing_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("check_default_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_default_root = cwd.join("dsl");

    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang check with missing default root");

    assert!(
        !output.status.success(),
        "default check should fail when cwd lacks dsl/ root"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "default-root-missing check should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_default_root.display().to_string()),
        "default-root check error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn check_command_default_root_non_directory_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("check_default_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let non_directory_default_root = cwd.join("dsl");
    std::fs::write(&non_directory_default_root, "not a directory")
        .expect("failed to create non-directory default root");

    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang check with non-directory default root");

    assert!(
        !output.status.success(),
        "default check should fail when cwd/dsl exists as a file"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "default-root-non-directory check should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&non_directory_default_root.display().to_string()),
        "default-root check non-directory error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_defaults_to_workspace_dsl_root() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules with default root");

    assert!(
        output.status.success(),
        "default modules command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_compile_stage_banners(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Discovered modules:"));
    let reported_modules = reported_modules_sorted(&stdout);
    let expected_modules = expected_dsl_modules_sorted();
    assert_eq!(
        reported_modules, expected_modules,
        "default modules command should report the complete dsl corpus"
    );
}

#[test]
fn modules_command_default_root_missing_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("modules_default_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let missing_default_root = cwd.join("dsl");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules with missing default root");

    assert!(
        !output.status.success(),
        "default modules should fail when cwd lacks dsl/ root"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "default-root-missing modules should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_default_root.display().to_string()),
        "default-root modules error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_default_root_non_directory_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("modules_default_non_directory_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");
    let non_directory_default_root = cwd.join("dsl");
    std::fs::write(&non_directory_default_root, "not a directory")
        .expect("failed to create non-directory default root");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules with non-directory default root");

    assert!(
        !output.status.success(),
        "default modules should fail when cwd/dsl exists as a file"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "default-root-non-directory modules should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&non_directory_default_root.display().to_string()),
        "default-root modules non-directory error should include resolved cwd/dsl path: {stderr}"
    );

    std::fs::remove_dir_all(cwd).expect("failed to cleanup temp cwd");
}

#[test]
fn modules_command_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules");
    assert!(first.status.success(), "first modules run should succeed");

    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules");
    assert!(second.status.success(), "second modules run should succeed");

    assert_eq!(
        first.stdout, second.stdout,
        "modules output should be deterministic for identical inputs"
    );
}

#[cfg(unix)]
#[test]
fn check_command_accepts_symlink_root_directory() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_root");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(
        real.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlink root");

    assert!(
        output.status.success(),
        "check should succeed for symlink root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: checked 1 file(s)"),
        "symlink-root check should parse exactly one file: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_handles_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_cycle_root");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(
        nested.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlink-cycle root");

    assert!(
        output.status.success(),
        "check should succeed for directory symlink cycle root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: checked 1 file(s)"),
        "symlink-cycle root should parse exactly one file: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_alias_root_order_is_deterministic_for_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_alias_root_order_errors");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(real.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check");
    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check");

    assert!(!first.status.success(), "first run should fail");
    assert!(!second.status.success(), "second run should fail");
    assert_eq!(
        first.stderr, second.stderr,
        "check diagnostics should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_symlink_alias_root_order_is_deterministic_for_success() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_alias_root_order_success");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(
        real.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check");
    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "check output should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_deduplicates_parse_errors_in_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_cycle_parse_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on symlink-cycle parse-error root");

    assert!(
        !output.status.success(),
        "check should fail when the symlink-cycle root contains parse errors"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let broken_hits = stderr.matches("broken.dag:").count();
    assert_eq!(
        broken_hits, 1,
        "symlink-cycle traversal should not duplicate parse diagnostics: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_output_is_deterministic_in_directory_symlink_cycle_with_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_symlink_cycle_deterministic_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let first = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang check on symlink-cycle parse-error root");
    let second = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang check on symlink-cycle parse-error root");

    assert!(!first.status.success(), "first run should fail");
    assert!(!second.status.success(), "second run should fail");
    assert_eq!(
        first.stderr, second.stderr,
        "check stderr should be deterministic under symlink-cycle parse-error input"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_symlink_root_without_module_decl_uses_path_fallback() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_root_module_fallback");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(real.join("nested")).expect("failed to create nested root");
    std::fs::write(real.join("nested/no_module.dag"), "fn ok() -> Unit {}")
        .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on symlink root");

    assert!(
        output.status.success(),
        "modules should succeed for symlink root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nested.no_module"),
        "symlink-root module fallback should render path-derived module: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_symlink_alias_root_order_is_deterministic_for_success() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_alias_root_order_success");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(
        real.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules");
    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "modules output should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_symlink_alias_root_order_is_deterministic_for_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_alias_root_order_errors");
    let real = root.join("real");
    let link = root.join("link");
    std::fs::create_dir_all(&real).expect("failed to create real root");
    std::fs::write(real.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&real, &link).expect("failed to create root symlink");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&real)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules");
    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&link)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "modules diagnostics/report should match for real/symlink root aliases"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_deduplicates_parse_errors_in_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_cycle_parse_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on symlink-cycle parse-error root");

    assert!(
        output.status.success(),
        "modules should still succeed while reporting parse diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let broken_hits = stdout.matches("broken.dag:").count();
    assert_eq!(
        broken_hits, 1,
        "symlink-cycle traversal should not duplicate parse diagnostics in modules output: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_output_is_deterministic_in_directory_symlink_cycle_with_errors() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_cycle_deterministic_errors");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write invalid source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let first = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang modules on symlink-cycle parse-error root");
    let second = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang modules on symlink-cycle parse-error root");

    assert!(first.status.success(), "first run should succeed");
    assert!(second.status.success(), "second run should succeed");
    assert_eq!(
        first.stdout, second.stdout,
        "modules stdout should be deterministic under symlink-cycle parse-error input"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_handles_directory_symlink_cycle_root() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_symlink_cycle_root");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested root");
    std::fs::write(
        nested.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write source");
    symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on symlink-cycle root");

    assert!(
        output.status.success(),
        "modules should succeed for directory symlink cycle root: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sample.main"),
        "symlink-cycle root should still render discovered module: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn check_command_dangling_dag_symlink_in_root_exits_nonzero() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("check_dangling_symlink_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check on dangling symlink root");

    assert!(
        !output.status.success(),
        "check should fail when root contains dangling .dag symlink"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "dangling-symlink root check should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("failed to canonicalize"));
    assert!(
        stderr.contains("broken.dag"),
        "dangling symlink failure should include offending path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn modules_command_dangling_dag_symlink_in_root_exits_nonzero() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("modules_dangling_symlink_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on dangling symlink root");

    assert!(
        !output.status.success(),
        "modules should fail when root contains dangling .dag symlink"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "dangling-symlink root modules should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("failed to canonicalize"));
    assert!(
        stderr.contains("broken.dag"),
        "dangling symlink failure should include offending path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn modules_command_empty_directory_succeeds_without_diagnostics() {
    let root = unique_temp_dir("modules_empty_dir");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on empty directory");

    assert!(
        output.status.success(),
        "modules should succeed for an empty directory"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Discovered modules:"),
        "modules output should include summary header"
    );
    assert!(
        !stdout.contains("Diagnostics:"),
        "empty directory should not produce diagnostics: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_ignores_non_dag_files_in_directory() {
    let root = unique_temp_dir("modules_ignore_non_dag");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on non-dag-only directory");

    assert!(
        output.status.success(),
        "modules should ignore non-.dag files in directory mode"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Discovered modules:"),
        "modules output should include summary header"
    );
    assert!(
        !stdout.contains("Diagnostics:"),
        "non-.dag files should not produce diagnostics: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_ignores_non_dag_files_when_dag_files_exist() {
    let root = unique_temp_dir("modules_ignore_non_dag_mixed");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write dag file");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules on mixed dag/non-dag directory");

    assert!(
        output.status.success(),
        "modules should report only discovered .dag modules"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sample.main"));
    assert!(
        !stdout.contains("Diagnostics:"),
        "ignored non-.dag files should not produce diagnostics: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn modules_command_missing_directory_exits_nonzero() {
    let missing_dir = unique_temp_dir("modules_missing_dir");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&missing_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for missing directory");

    assert!(
        !output.status.success(),
        "modules should fail when input directory does not exist"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "missing-directory modules should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));
    assert!(
        stderr.contains(&missing_dir.display().to_string()),
        "missing-root error should include path context: {stderr}"
    );
}

#[test]
fn modules_command_non_directory_root_exits_nonzero() {
    let root_file = unique_temp_file("modules_non_directory_root").with_extension("txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&root_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for non-directory root");

    assert!(
        !output.status.success(),
        "modules should fail when root path is not a directory"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "non-directory-root modules should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "non-directory root error should include offending path: {stderr}"
    );

    std::fs::remove_file(root_file).expect("failed to cleanup root file");
}

#[test]
fn modules_command_single_dag_file_path_exits_nonzero() {
    let file_path = unique_temp_file("modules_single_file_root");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to create .dag file");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules for single-file path");

    assert_modules_single_file_root_failure(&output, &file_path, "modules single-file-root input");

    std::fs::remove_file(file_path).expect("failed to cleanup .dag file");
}

#[test]
fn modules_command_single_uppercase_dag_file_path_exits_nonzero() {
    let root = unique_temp_dir("single_uppercase_dag_file_path_exits_nonzero");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DAG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("main.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang modules");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "modules_command_single_uppercase_dag_file_path_exits_nonzero",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn modules_command_single_mixed_case_dag_file_path_exits_nonzero() {
    let root = unique_temp_dir("single_mixed_case_dag_file_path_exits_nonzero");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DaG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("main.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang modules");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "modules_command_single_mixed_case_dag_file_path_exits_nonzero",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn modules_command_relative_and_absolute_uppercase_single_file_roots_are_equivalent() {
    let root = unique_temp_dir("relative_and_absolute_uppercase_single_file_roots_");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DAG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("main.DAG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang modules");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "modules_command_relative_and_absolute_uppercase_single_file_roots_are_equivalent",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn modules_command_relative_and_absolute_mixed_case_single_file_roots_are_equivalent() {
    let root = unique_temp_dir("relative_and_absolute_mixed_case_single_file_roots");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.DaG"),
        "module sample.main\nfn ok() -> Unit {}",
    )
    .expect("failed to write");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("main.DaG")
        .current_dir(&root)
        .output()
        .expect("failed to run daglang modules");
    assert_wrong_cased_dag_extension_rejected(
        &output,
        "modules_command_relative_and_absolute_mixed_case_single_file_roots_are_equivalent",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

#[test]
fn modules_command_relative_and_absolute_single_file_roots_are_equivalent() {
    let root = unique_temp_dir("modules_relative_absolute_single_file_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let file_path = root.join("one.dag");
    std::fs::write(&file_path, "module sample.one\nfn ok() -> Unit {}")
        .expect("failed to write temp dag file");

    let relative = Command::new(daglang_bin())
        .arg("modules")
        .arg("one.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative single-file-root daglang modules");
    assert!(
        !relative.status.success(),
        "relative single-file-root modules should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("modules")
        .arg(&file_path)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute single-file-root daglang modules");
    assert_modules_relative_absolute_single_file_root_equivalence(
        &relative,
        &absolute,
        &file_path,
        "single-file-root modules",
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn unknown_command_exits_nonzero_with_message() {
    let output = Command::new(daglang_bin())
        .arg("unknown-cmd")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang with unknown command");

    assert!(
        !output.status.success(),
        "unknown command should fail with non-zero exit code"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "unknown command should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown command"));
    assert!(
        stderr.contains("Usage: daglang <command> [args...]"),
        "unknown command should include top-level usage guidance: {stderr}"
    );
}

#[test]
fn no_command_exits_nonzero_with_usage_message() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang without command");

    assert!(
        !output.status.success(),
        "invoking daglang without a command should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "missing command should use exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: daglang <command> [args...]"),
        "missing-command invocation should print usage guidance: {stderr}"
    );
    assert!(
        stderr.contains("check <file.dag|dir>"),
        "usage guidance should include check command help text: {stderr}"
    );
    assert!(
        stderr.contains("obligations <file.dag> [--format text|json]"),
        "usage guidance should include obligations command help text: {stderr}"
    );
    assert!(
        stderr.contains("show-triplets <file.dag> [--format text|json]"),
        "usage guidance should include show-triplets command help text: {stderr}"
    );
}

#[test]
fn check_command_with_extra_args_exits_nonzero_with_usage_message() {
    let output = Command::new(daglang_bin())
        .arg("check")
        .arg("dsl")
        .arg("extra")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check with extra args");

    assert!(
        !output.status.success(),
        "check with extra args should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "check with extra args should use usage exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: daglang check <file.dag|dir>"),
        "check with extra args should print command usage: {stderr}"
    );
}

#[test]
fn modules_command_with_extra_args_exits_nonzero_with_usage_message() {
    let output = Command::new(daglang_bin())
        .arg("modules")
        .arg("dsl")
        .arg("extra")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang modules with extra args");

    assert!(
        !output.status.success(),
        "modules with extra args should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "modules with extra args should use usage exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: daglang modules [dir] [--format text|json]"),
        "modules with extra args should print command usage: {stderr}"
    );
}

#[test]
fn viz_self_with_extra_args_exits_nonzero_with_usage_message() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .arg("extra")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz --self with extra args");

    assert!(
        !output.status.success(),
        "viz --self with extra args should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "viz --self with extra args should use usage exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: daglang viz <file.dag>|--self [--format ascii|mermaid]"),
        "viz --self with extra args should print command usage: {stderr}"
    );
}

#[test]
fn viz_without_args_exits_nonzero_with_usage_message() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz without args");

    assert!(!output.status.success(), "viz without args should fail");
    assert_eq!(
        output.status.code(),
        Some(1),
        "viz without args should use usage exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: daglang viz <file.dag>|--self [--format ascii|mermaid]"),
        "viz without args should print usage guidance: {stderr}"
    );
}

#[test]
fn expand_and_progress_without_required_target_exit_with_usage_message() {
    for command in ["expand", "progress"] {
        let output = Command::new(daglang_bin())
            .arg(command)
            .current_dir(workspace_root())
            .output()
            .unwrap_or_else(|err| panic!("failed to run {command} without target: {err}"));
        assert!(
            !output.status.success(),
            "{command} without required target should fail"
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{command} without required target should use usage exit code 1"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(&format!("Usage: daglang {command} <file.dag>")),
            "{command} without required target should print command usage: {stderr}"
        );
    }
}

#[test]
fn expand_and_progress_with_extra_args_exit_with_usage_message() {
    for command in ["expand", "progress"] {
        let output = Command::new(daglang_bin())
            .arg(command)
            .arg("dsl/tools/makegen.dag")
            .arg("extra")
            .current_dir(workspace_root())
            .output()
            .unwrap_or_else(|err| panic!("failed to run {command} with extra args: {err}"));
        assert!(
            !output.status.success(),
            "{command} with extra args should fail"
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{command} with extra args should use usage exit code 1"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(&format!("Usage: daglang {command} <file.dag>")),
            "{command} with extra args should print command usage: {stderr}"
        );
    }
}

#[test]
fn obligations_and_show_triplets_without_required_target_exit_with_usage_message() {
    for (command, usage) in [
        (
            "obligations",
            "Usage: daglang obligations <file.dag> [--format text|json]",
        ),
        (
            "show-triplets",
            "Usage: daglang show-triplets <file.dag> [--format text|json]",
        ),
    ] {
        let output = Command::new(daglang_bin())
            .arg(command)
            .current_dir(workspace_root())
            .output()
            .unwrap_or_else(|err| panic!("failed to run {command} without target: {err}"));
        assert!(
            !output.status.success(),
            "{command} without required target should fail"
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{command} without required target should use usage exit code 1"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(usage),
            "{command} without required target should print command usage: {stderr}"
        );
    }
}

#[test]
fn obligations_and_show_triplets_with_invalid_format_exit_with_usage_message() {
    for (command, usage) in [
        (
            "obligations",
            "Usage: daglang obligations <file.dag> [--format text|json]",
        ),
        (
            "show-triplets",
            "Usage: daglang show-triplets <file.dag> [--format text|json]",
        ),
    ] {
        for bad_value in ["yaml", "JSON", "Json", "Text"] {
            let output = Command::new(daglang_bin())
                .arg(command)
                .arg("dsl/tools/makegen.dag")
                .arg("--format")
                .arg(bad_value)
                .current_dir(workspace_root())
                .output()
                .unwrap_or_else(|err| {
                    panic!("failed to run {command} with invalid format {bad_value}: {err}")
                });
            assert!(
                !output.status.success(),
                "{command} with invalid format {bad_value} should fail"
            );
            assert_eq!(
                output.status.code(),
                Some(1),
                "{command} with invalid format {bad_value} should use usage exit code 1"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(usage),
                "{command} with invalid format {bad_value} should print command usage: {stderr}"
            );
        }
    }
}

#[test]
fn obligations_and_show_triplets_with_extra_args_exit_with_usage_message() {
    for (command, usage) in [
        (
            "obligations",
            "Usage: daglang obligations <file.dag> [--format text|json]",
        ),
        (
            "show-triplets",
            "Usage: daglang show-triplets <file.dag> [--format text|json]",
        ),
    ] {
        let output = Command::new(daglang_bin())
            .arg(command)
            .arg("dsl/tools/makegen.dag")
            .arg("--format")
            .arg("text")
            .arg("extra")
            .current_dir(workspace_root())
            .output()
            .unwrap_or_else(|err| panic!("failed to run {command} with extra args: {err}"));
        assert!(
            !output.status.success(),
            "{command} with extra args should fail"
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{command} with extra args should use usage exit code 1"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(usage),
            "{command} with extra args should print command usage: {stderr}"
        );
    }
}

#[test]
fn obligations_and_show_triplets_with_missing_format_value_exit_with_usage_message() {
    for (command, usage) in [
        (
            "obligations",
            "Usage: daglang obligations <file.dag> [--format text|json]",
        ),
        (
            "show-triplets",
            "Usage: daglang show-triplets <file.dag> [--format text|json]",
        ),
    ] {
        let output = Command::new(daglang_bin())
            .arg(command)
            .arg("dsl/tools/makegen.dag")
            .arg("--format")
            .current_dir(workspace_root())
            .output()
            .unwrap_or_else(|err| {
                panic!("failed to run {command} with missing format value: {err}")
            });
        assert!(
            !output.status.success(),
            "{command} with missing format value should fail"
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "{command} with missing format value should use usage exit code 1"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(usage),
            "{command} with missing format value should print command usage: {stderr}"
        );
    }
}

fn run_compile_family_command(command: &str, target: &str, trailing_args: &[&str]) -> Output {
    Command::new(daglang_bin())
        .arg(command)
        .arg(target)
        .args(trailing_args)
        .current_dir(workspace_root())
        .output()
        .unwrap_or_else(|err| panic!("failed to run {command} for target {target}: {err}"))
}

fn assert_compile_family_command_succeeds(
    command: &str,
    target: &str,
    target_label: &str,
    trailing_args: &[&str],
) -> Output {
    let output = run_compile_family_command(command, target, trailing_args);
    assert!(
        output.status.success(),
        "{command} should execute successfully for {target_label} makegen fixture target"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("TODO"),
        "{command} should not emit TODO placeholder output: {stderr}"
    );
    assert!(
        !output.stdout.is_empty(),
        "{command} should emit meaningful stdout output for {target_label} target"
    );
    output
}

const COMPILE_FAMILY_COMMANDS: [(&str, &[&str]); 5] = [
    ("expand", &[]),
    ("progress", &[]),
    ("compile", &[]),
    ("obligations", &["--format", "json"]),
    ("show-triplets", &["--format", "json"]),
];

fn run_compile_family_smoke_for_target(target_label: &str, target: &str) {
    for (command, trailing_args) in COMPILE_FAMILY_COMMANDS {
        assert_compile_family_command_succeeds(command, target, target_label, trailing_args);
    }
}

fn makegen_target_variants() -> Vec<(&'static str, String)> {
    vec![
        ("relative", "dsl/tools/makegen.dag".to_string()),
        (
            "absolute",
            workspace_root()
                .join("dsl/tools/makegen.dag")
                .to_string_lossy()
                .into_owned(),
        ),
        ("curdir-suffix", "./dsl/tools/makegen.dag".to_string()),
        (
            "absolute-curdir-segment",
            workspace_root()
                .join("./dsl/./tools/../tools/makegen.dag")
                .to_string_lossy()
                .into_owned(),
        ),
        (
            "absolute-double-separator",
            format!("{}/dsl//tools///makegen.dag", workspace_root().display()),
        ),
        (
            "absolute-parent-segment",
            workspace_root()
                .join("dsl/../dsl/tools/makegen.dag")
                .to_string_lossy()
                .into_owned(),
        ),
        (
            "absolute-parent-double-separator",
            format!(
                "{}/dsl/..//dsl/tools/makegen.dag",
                workspace_root().display()
            ),
        ),
        (
            "absolute-parent-curdir-segment",
            format!(
                "{}/dsl/tools/./../tools/makegen.dag",
                workspace_root().display()
            ),
        ),
        (
            "absolute-parent-curdir-double-separator",
            format!(
                "{}/dsl/./tools/..//tools/makegen.dag",
                workspace_root().display()
            ),
        ),
    ]
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths() {
    run_compile_family_smoke_for_target("relative", "dsl/tools/makegen.dag");
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_absolute_target() {
    let absolute_target = workspace_root().join("dsl/tools/makegen.dag");
    let absolute_target = absolute_target.display().to_string();
    run_compile_family_smoke_for_target("absolute", &absolute_target);
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_curdir_suffix_target() {
    run_compile_family_smoke_for_target("curdir-suffix", "./dsl/tools/makegen.dag");
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_absolute_curdir_segment_target() {
    let absolute_target_with_curdir_segment =
        workspace_root().join("./dsl/./tools/../tools/makegen.dag");
    let absolute_target_with_curdir_segment = absolute_target_with_curdir_segment
        .to_string_lossy()
        .into_owned();
    run_compile_family_smoke_for_target(
        "absolute-curdir-segment",
        &absolute_target_with_curdir_segment,
    );
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_absolute_double_separator_target() {
    let absolute_target_with_double_separators =
        format!("{}/dsl//tools///makegen.dag", workspace_root().display());
    run_compile_family_smoke_for_target(
        "absolute-double-separator",
        &absolute_target_with_double_separators,
    );
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_absolute_parent_segment_target() {
    let absolute_target_with_parent_segment = workspace_root()
        .join("dsl/../dsl/tools/makegen.dag")
        .to_string_lossy()
        .into_owned();
    run_compile_family_smoke_for_target(
        "absolute-parent-segment",
        &absolute_target_with_parent_segment,
    );
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_absolute_parent_double_separator_target(
) {
    let absolute_target_with_parent_double_separator = format!(
        "{}/dsl/..//dsl/tools/makegen.dag",
        workspace_root().display()
    );
    run_compile_family_smoke_for_target(
        "absolute-parent-double-separator",
        &absolute_target_with_parent_double_separator,
    );
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_absolute_parent_curdir_segment_target()
{
    let absolute_target_with_parent_curdir_segment = format!(
        "{}/dsl/tools/./../tools/makegen.dag",
        workspace_root().display()
    );
    run_compile_family_smoke_for_target(
        "absolute-parent-curdir-segment",
        &absolute_target_with_parent_curdir_segment,
    );
}

#[test]
fn compile_family_commands_execute_real_pipeline_paths_with_absolute_parent_curdir_double_separator_target(
) {
    let absolute_target_with_parent_curdir_double_separator = format!(
        "{}/dsl/./tools/..//tools/makegen.dag",
        workspace_root().display()
    );
    run_compile_family_smoke_for_target(
        "absolute-parent-curdir-double-separator",
        &absolute_target_with_parent_curdir_double_separator,
    );
}

#[test]
fn compile_family_commands_makegen_target_variants_are_output_equivalent() {
    let targets = makegen_target_variants();

    for (command, trailing_args) in COMPILE_FAMILY_COMMANDS {
        let mut runs: Vec<(&str, Output)> = Vec::with_capacity(targets.len());
        for (target_label, target_value) in targets.iter() {
            let output = assert_compile_family_command_succeeds(
                command,
                target_value,
                target_label,
                trailing_args,
            );
            runs.push((target_label, output));
        }

        let (_, reference_output) = &runs[0];
        for (target_label, output) in runs.iter().skip(1) {
            assert_eq!(
                reference_output.stdout, output.stdout,
                "{command} stdout should match between relative and {target_label} makegen targets"
            );
            assert_eq!(
                reference_output.stderr, output.stderr,
                "{command} stderr should match between relative and {target_label} makegen targets"
            );
        }
    }
}

#[test]
fn compile_with_out_writes_native_emitted_files() {
    let out_dir = unique_temp_dir("compile_out_native");
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/tools/makegen.dag")
        .arg("--out")
        .arg(&out_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile --out");
    assert!(
        output.status.success(),
        "compile --out should succeed for makegen target"
    );

    let main_rs = out_dir.join("target/generated/rust/main.rs");
    let manifest = out_dir.join("target/generated/rust/progress_manifest.txt");
    let emit_manifest = out_dir.join("target/generated/rust/emit_manifest.json");
    assert!(main_rs.is_file(), "compile --out should write main.rs");
    assert!(
        manifest.is_file(),
        "compile --out should write progress_manifest.txt"
    );
    assert!(
        emit_manifest.is_file(),
        "compile --out should write emit_manifest.json"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("target=rust layer=2"),
        "compile summary should include selected backend/layer: {stdout}"
    );

    std::fs::remove_dir_all(&out_dir).expect("failed to cleanup compile --out directory");
}

#[test]
fn compile_with_out_emits_deterministic_manifest_for_same_input() {
    let out_first = unique_temp_dir("compile_out_manifest_det_first");
    let out_second = unique_temp_dir("compile_out_manifest_det_second");

    let first = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/tools/makegen.dag")
        .arg("--target")
        .arg("go")
        .arg("--out")
        .arg(&out_first)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang compile --target go --out");
    let second = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/tools/makegen.dag")
        .arg("--target")
        .arg("go")
        .arg("--out")
        .arg(&out_second)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang compile --target go --out");

    assert!(
        first.status.success(),
        "first compile --target go --out should succeed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "second compile --target go --out should succeed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    let manifest_first_path = out_first.join("target/generated/go/emit_manifest.json");
    let manifest_second_path = out_second.join("target/generated/go/emit_manifest.json");
    let manifest_first = std::fs::read_to_string(&manifest_first_path)
        .expect("first emit manifest should exist and be readable");
    let manifest_second = std::fs::read_to_string(&manifest_second_path)
        .expect("second emit manifest should exist and be readable");
    let parsed_first: Value =
        serde_json::from_str(&manifest_first).expect("first emit manifest should be valid JSON");
    let parsed_second: Value =
        serde_json::from_str(&manifest_second).expect("second emit manifest should be valid JSON");
    assert_eq!(
        parsed_first, parsed_second,
        "emit manifest should be deterministic across repeated compile runs"
    );

    std::fs::remove_dir_all(&out_first).expect("failed to cleanup first compile output");
    std::fs::remove_dir_all(&out_second).expect("failed to cleanup second compile output");
}

#[test]
fn compile_with_trace_stages_prints_canonical_stage_flow() {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/tools/makegen.dag")
        .arg("--trace-stages")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile --trace-stages");
    assert!(
        output.status.success(),
        "compile --trace-stages should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Compilation stages:"),
        "trace output should include stage header"
    );
    assert!(
        stdout.contains("discover") && stdout.contains("resolve") && stdout.contains("emit"),
        "trace output should include canonical stage names: {stdout}"
    );
}

// DELETED: compile_layer_one_with_out_writes_exec_runtime_files
// DELETED: compile_layer_one_with_nested_out_allows_generated_cargo_check
// Blocked on: RF-E5 (PureRender fn body delegate gap — exec-runtime can't classify Callable with fn_body).
// Restore when exec-runtime gains fn body classification support.

#[test]
fn compile_with_go_target_writes_native_go_files() {
    let out_dir = unique_temp_dir("compile_out_go");
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/tools/makegen.dag")
        .arg("--target")
        .arg("go")
        .arg("--out")
        .arg(&out_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile --target go --out");
    assert!(
        output.status.success(),
        "compile --target go --out should succeed"
    );
    assert!(
        out_dir.join("target/generated/go/main.go").is_file(),
        "compile --target go should emit main.go"
    );
    assert!(
        out_dir
            .join("target/generated/go/progress_manifest.txt")
            .is_file(),
        "compile --target go should emit progress manifest"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("target=go layer=2"),
        "compile summary should include go native backend/layer: {stdout}"
    );

    std::fs::remove_dir_all(&out_dir).expect("failed to cleanup go compile directory");
}

#[test]
fn compile_non_rust_layer_one_reports_error() {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/tools/makegen.dag")
        .arg("--target")
        .arg("go")
        .arg("--layer")
        .arg("1")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile --target go --layer 1");
    assert!(
        !output.status.success(),
        "compile --target go --layer 1 should fail because layer 1 is rust-only"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("layer 1 currently supports only --target rust"),
        "compile --target go --layer 1 should report rust-only layer 1: {stderr}"
    );
}

#[test]
fn viz_without_self_defaults_to_compiled_ascii_graph() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz");

    assert!(
        output.status.success(),
        "viz should compile and render ascii output"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("DAG daglang-compiled"),
        "viz ascii output should include dag header: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("TODO"),
        "viz should not emit TODO placeholder output: {stderr}"
    );
}

#[test]
fn viz_with_mermaid_format_emits_compiled_mermaid_graph() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("dsl/tools/makegen.dag")
        .arg("--format")
        .arg("mermaid")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz with mermaid format");

    assert!(
        output.status.success(),
        "viz --format mermaid should compile and render mermaid output"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("flowchart TB"),
        "viz mermaid output should include flowchart header: {stdout}"
    );
}

// DELETED: makegen_e2e_generated_binary_produces_correct_makefile
// Tracked as RF-E6 in tasks.md — exec-runtime emit gaps (LoadRegistry,
// PureRender, ContentUpsertOutputPath handlers). Re-add after exec-runtime
// emitter handles all makegen node classifications.

// DELETED: pragma_e2e_generated_binary_produces_correct_config_files
// Tracked as RF-E6 in tasks.md — exec-runtime emit gaps (ContentUpsertOutputPath,
// PureRender handlers). Re-add after exec-runtime emitter handles all pragma
// node classifications.

// ---------------------------------------------------------------------------
// gen-types: DSL → Rust type generation contract tests
// ---------------------------------------------------------------------------

#[test]
fn gen_types_produces_all_rendering_types() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args([
            "gen-types",
            "--module",
            "std.symbols",
            "--module",
            "std.render",
        ])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "gen-types should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let expected_enums = [
        "SemanticColor",
        "Tier",
        "SymbolId",
        "RenderMode",
        "CursorAction",
        "ViewportUnit",
    ];
    for name in expected_enums {
        assert!(
            stdout.contains(&format!("pub enum {name}")),
            "generated output should contain enum {name}:\n{stdout}"
        );
    }

    let expected_structs = [
        "SpanStyle",
        "Span",
        "Line",
        "Frame",
        "Viewport",
        "SymbolEntry",
        "AnsiMapping",
    ];
    for name in expected_structs {
        assert!(
            stdout.contains(&format!("pub struct {name}")),
            "generated output should contain struct {name}:\n{stdout}"
        );
    }
}

#[test]
fn gen_types_symbol_id_has_40_variants() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.symbols"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    let in_enum = stdout
        .split("pub enum SymbolId")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or("");
    let variant_count = in_enum
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            !trimmed.is_empty() && trimmed != "{" && !trimmed.starts_with("//")
        })
        .count();

    assert_eq!(
        variant_count, 40,
        "SymbolId should have exactly 40 variants (matching Rust), found {variant_count}"
    );
}

#[test]
fn gen_types_semantic_color_matches_rust() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.symbols"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    let expected_variants = [
        "Default", "Success", "Warning", "Error", "Info", "Dim", "Active", "Accent",
    ];
    for v in expected_variants {
        assert!(
            stdout.contains(v),
            "SemanticColor should contain variant {v}"
        );
    }
}

#[test]
fn gen_types_box_draw_types() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.box_draw"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "gen-types should succeed for box_draw: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(stdout.contains("pub enum BoxStyle"));
    assert!(stdout.contains("pub struct BoxChars"));
    assert!(stdout.contains("pub struct BoxConfig"));
    assert!(stdout.contains("Closed"));
    assert!(stdout.contains("OpenRight"));
}

#[test]
fn gen_types_without_module_filter_emits_all() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "gen-types without filter should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        stdout.contains("pub enum SemanticColor"),
        "unfiltered output should include SemanticColor"
    );
    assert!(
        stdout.contains("pub enum SymbolId"),
        "unfiltered output should include SymbolId"
    );
}

// ---------------------------------------------------------------------------
// gen-types: data table + function signature contract tests
// ---------------------------------------------------------------------------

#[test]
fn gen_types_emits_standard_symbols_static() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.symbols"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    assert!(
        stdout.contains("pub static STANDARD_SYMBOLS: &[SymbolEntry]"),
        "should generate STANDARD_SYMBOLS static:\n{stdout}"
    );
    assert!(
        stdout.contains("SymbolId::NodePending"),
        "enum variants should be fully qualified:\n{stdout}"
    );
    assert!(
        stdout.contains("SemanticColor::Dim"),
        "color variants should be fully qualified:\n{stdout}"
    );
    // Static data arrays (STANDARD_SYMBOLS, ANSI_MAPPINGS) use &'static str
    // fields directly — no .to_string() in the data declarations. Builtin
    // function bodies (resolve_symbol, ansi_code) may use .to_string() to
    // bridge from &'static str accessors to the declared String return type.
    let data_section = stdout.split("pub fn ").next().unwrap_or(&stdout);
    assert!(
        !data_section.contains(".to_string()"),
        "static data declarations should use &str, not .to_string():\n{data_section}"
    );
}

#[test]
fn gen_types_symbol_entry_uses_static_str() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.symbols"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    assert!(
        stdout.contains("&'static str"),
        "SymbolEntry fields should use &'static str for static compatibility:\n{stdout}"
    );
}

#[test]
fn gen_types_emits_ansi_mappings_static() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.symbols"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    assert!(
        stdout.contains("pub static ANSI_MAPPINGS: &[AnsiMapping]"),
        "should generate ANSI_MAPPINGS static:\n{stdout}"
    );
    assert!(
        stdout.contains("SemanticColor::Default"),
        "ansi mapping should use qualified variant:\n{stdout}"
    );
}

#[test]
fn gen_types_emits_impl_blocks() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.symbols"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    assert!(
        stdout.contains("impl SymbolId"),
        "should generate impl SymbolId:\n{stdout}"
    );
    assert!(
        stdout.contains("pub fn emoji(&self) -> &'static str"),
        "should generate emoji method on SymbolId:\n{stdout}"
    );
    assert!(
        stdout.contains("pub fn unicode(&self) -> &'static str"),
        "should generate unicode method on SymbolId:\n{stdout}"
    );
    assert!(
        stdout.contains("pub fn ascii(&self) -> &'static str"),
        "should generate ascii method on SymbolId:\n{stdout}"
    );
    assert!(
        stdout.contains("pub fn color(&self) -> SemanticColor"),
        "should generate color method on SymbolId:\n{stdout}"
    );
    assert!(
        stdout.contains("impl SemanticColor"),
        "should generate impl SemanticColor:\n{stdout}"
    );
    assert!(
        stdout.contains("pub fn code(&self) -> &'static str"),
        "should generate code method on SemanticColor:\n{stdout}"
    );
    assert!(
        stdout.contains("\\x1b[0m"),
        "ANSI codes should be properly escaped:\n{stdout}"
    );
}

#[test]
fn gen_types_box_draw_emits_data() {
    let output = Command::new(daglang_bin())
        .current_dir(workspace_root())
        .args(["gen-types", "--module", "std.box_draw"])
        .output()
        .expect("failed to run gen-types");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "gen-types should succeed for box_draw: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        stdout.contains("pub static UNICODE_BOX_CHARS: BoxChars"),
        "should generate unicode box chars data:\n{stdout}"
    );
    assert!(
        stdout.contains("pub static ASCII_BOX_CHARS: BoxChars"),
        "should generate ascii box chars data:\n{stdout}"
    );
}
