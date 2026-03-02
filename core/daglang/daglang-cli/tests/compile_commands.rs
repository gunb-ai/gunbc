// Test infrastructure: filesystem access for test fixtures
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use gunbc_ir::WorkspaceLayout;
use gunbc_test::{unique_temp_dir, unique_temp_file};
use serde_json::Value;
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

fn makegen_file() -> PathBuf {
    workspace_root().join("dsl/tools/makegen.dag")
}

fn ci_pipeline_file() -> PathBuf {
    workspace_root().join("dsl/pipelines/ci.dag")
}

fn deps_file() -> PathBuf {
    workspace_root().join("dsl/tools/deps.dag")
}

fn dsl_root_dir() -> PathBuf {
    workspace_root().join("dsl")
}

fn aws_resources_file() -> PathBuf {
    workspace_root().join("dsl/infra/aws/resources.dag")
}

fn azure_resources_file() -> PathBuf {
    workspace_root().join("dsl/infra/azure/resources.dag")
}

fn expected_makegen_expand_snapshot() -> String {
    format!(
        "{}\n\n",
        include_str!("snapshots/makegen_expand.txt").trim_end_matches('\n')
    )
}

fn unique_temp_output_file(name: &str, extension: &str) -> PathBuf {
    let root = unique_temp_dir(name);
    std::fs::create_dir_all(&root).expect("failed to create temp output dir");
    root.join(format!("{name}.{extension}"))
}

fn write_minimal_directory_compile_fixture(root: &Path) {
    std::fs::create_dir_all(root.join("dsl/sample")).expect("failed to create dsl fixture tree");
    std::fs::write(
        root.join("dsl/sample/main.dag"),
        "module sample.main\nfn run() -> Unit { }",
    )
    .expect("failed to write minimal compile fixture");
}

fn assert_typecheck_stage_failure(stderr: &str) {
    assert!(stderr.contains("typecheck errors"));
    assert!(
        !stderr.contains("lower error"),
        "expected failure to remain in typecheck stage: {stderr}"
    );
}

fn assert_lower_stage_failure(stderr: &str) {
    assert!(stderr.contains("lower error"));
    assert!(
        !stderr.contains("typecheck errors"),
        "expected failure to remain in lowering stage: {stderr}"
    );
}

fn assert_no_stage_failures(stderr: &str) {
    assert!(
        !stderr.contains("typecheck errors"),
        "did not expect typecheck-stage banner in successful path: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "did not expect lowering-stage banner in successful path: {stderr}"
    );
}

fn run_compile_with_optional_trailing_slash(root: &Path, input: &str) -> (Output, Output) {
    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg(input)
        .current_dir(root)
        .output()
        .expect("failed to run plain compile invocation");
    let trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(format!("{input}/"))
        .current_dir(root)
        .output()
        .expect("failed to run trailing-slash compile invocation");
    (plain, trailing)
}

fn run_single_target_command_with_optional_trailing_slash_and_args(
    command_name: &str,
    root: &Path,
    input: &str,
    extra_args: &[&str],
) -> (Output, Output) {
    let plain = run_single_target_command(command_name, root, input, extra_args)
        .expect("failed to run plain single-target command invocation");
    let trailing = run_single_target_command(command_name, root, &format!("{input}/"), extra_args)
        .expect("failed to run trailing-slash single-target command invocation");
    (plain, trailing)
}

fn run_single_target_command(
    command_name: &str,
    root: &Path,
    target: &str,
    extra_args: &[&str],
) -> std::io::Result<Output> {
    Command::new(daglang_bin())
        .arg(command_name)
        .arg(target)
        .args(extra_args)
        .current_dir(root)
        .output()
}

fn assert_single_target_command_outputs_match_for_targets(
    command_name: &str,
    root: &Path,
    canonical_target: &str,
    variant_target: &str,
    extra_args: &[&str],
    variant_label: &str,
) {
    let canonical = run_single_target_command(command_name, root, canonical_target, extra_args)
        .expect("failed to run canonical single-target command invocation");
    assert!(
        canonical.status.success(),
        "canonical {command_name} invocation failed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    let variant = run_single_target_command(command_name, root, variant_target, extra_args)
        .expect("failed to run variant single-target command invocation");
    assert!(
        variant.status.success(),
        "{variant_label} {command_name} invocation failed: {}",
        String::from_utf8_lossy(&variant.stderr)
    );

    assert_eq!(
        canonical.stdout, variant.stdout,
        "canonical and {variant_label} {command_name} stdout should match"
    );
    assert_eq!(
        canonical.stderr, variant.stderr,
        "canonical and {variant_label} {command_name} stderr should match"
    );
}

fn assert_single_target_command_failure_outputs_match_for_targets(
    command_name: &str,
    root: &Path,
    canonical_target: &str,
    variant_target: &str,
    extra_args: &[&str],
    variant_label: &str,
) {
    let canonical = run_single_target_command(command_name, root, canonical_target, extra_args)
        .expect("failed to run canonical single-target command invocation");
    assert!(
        !canonical.status.success(),
        "canonical {command_name} invocation should fail for invalid target"
    );

    let variant = run_single_target_command(command_name, root, variant_target, extra_args)
        .expect("failed to run variant single-target command invocation");
    assert!(
        !variant.status.success(),
        "{variant_label} {command_name} invocation should fail for invalid target"
    );

    assert_eq!(
        canonical.stdout, variant.stdout,
        "canonical and {variant_label} {command_name} failing stdout should match"
    );
    assert_eq!(
        canonical.stderr, variant.stderr,
        "canonical and {variant_label} {command_name} failing stderr should match"
    );
}

fn assert_single_target_command_failure_outputs_match_for_variants(
    command_name: &str,
    root: &Path,
    canonical_target: &str,
    variant_targets: &[(&str, &str)],
    extra_args: &[&str],
) {
    for (variant_target, variant_label) in variant_targets {
        assert_single_target_command_failure_outputs_match_for_targets(
            command_name,
            root,
            canonical_target,
            variant_target,
            extra_args,
            variant_label,
        );
    }
}

fn assert_dag_suffixed_directory_is_invalid_single_file_target(
    root: &Path,
    input: &str,
    expected_target: &Path,
    nested_diagnostic_snippet: Option<&str>,
) {
    let (plain, trailing) = run_compile_with_optional_trailing_slash(root, input);
    assert!(
        !plain.status.success(),
        "{input} compile should fail for .dag-suffixed directory target"
    );
    assert!(
        !trailing.status.success(),
        "{input}/ compile should fail for .dag-suffixed directory target"
    );

    assert_eq!(
        plain.stdout, trailing.stdout,
        "{input} plain and trailing-slash compile stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing.stderr,
        "{input} plain and trailing-slash compile stderr should match"
    );

    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", expected_target.display())),
        "{input} compile should fail with normalized single-file target path: {stderr}"
    );
    assert!(
        stderr.contains("target is a directory"),
        "{input} compile should explain the .dag directory/single-file ambiguity: {stderr}"
    );
    assert!(
        stderr.contains("`.dag` paths are treated as single-file targets"),
        "{input} compile should include a disambiguation hint for .dag directories: {stderr}"
    );
    if let Some(snippet) = nested_diagnostic_snippet {
        assert!(
            !stderr.contains(snippet),
            "{input} should fail before parsing nested files: {stderr}"
        );
    }
    assert_no_stage_failures(&stderr);
}

fn assert_single_target_command_treats_dag_directory_as_invalid_single_file_target_with_args(
    command_name: &str,
    root: &Path,
    input: &str,
    expected_target: &Path,
    nested_diagnostic_snippet: Option<&str>,
    extra_args: &[&str],
) {
    let (plain, trailing) = run_single_target_command_with_optional_trailing_slash_and_args(
        command_name,
        root,
        input,
        extra_args,
    );
    assert!(
        !plain.status.success(),
        "{command_name} {input} should fail for .dag-suffixed directory target"
    );
    assert!(
        !trailing.status.success(),
        "{command_name} {input}/ should fail for .dag-suffixed directory target"
    );

    assert_eq!(
        plain.stdout, trailing.stdout,
        "{command_name} {input} plain and trailing-slash stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing.stderr,
        "{command_name} {input} plain and trailing-slash stderr should match"
    );

    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", expected_target.display())),
        "{command_name} should fail with normalized single-file target path: {stderr}"
    );
    assert!(
        stderr.contains("target is a directory"),
        "{command_name} should explain the .dag directory/single-file ambiguity: {stderr}"
    );
    assert!(
        stderr.contains("`.dag` paths are treated as single-file targets"),
        "{command_name} should include a disambiguation hint for .dag directories: {stderr}"
    );
    if let Some(snippet) = nested_diagnostic_snippet {
        assert!(
            !stderr.contains(snippet),
            "{command_name} should fail before parsing nested files: {stderr}"
        );
    }
    assert_no_stage_failures(&stderr);
}

fn obligations_block(output: &str) -> String {
    output
        .lines()
        .skip_while(|line| *line != "TestObligations:")
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

// ── Path variant generators (shared across all command tests) ──────────────

fn run_compile_cli(arg: &str, cwd: &Path) -> Output {
    Command::new(daglang_bin())
        .arg("compile")
        .arg(arg)
        .current_dir(cwd)
        .output()
        .expect("failed to run compile command")
}

fn run_compile_cli_path(path: &Path, cwd: &Path) -> Output {
    run_compile_cli(&path.to_string_lossy(), cwd)
}

fn assert_compile_variant_matches(
    canonical: &Output,
    variant: &Output,
    label: &str,
    expect_success: bool,
) {
    if expect_success {
        assert!(
            variant.status.success(),
            "{label}: variant should succeed: {}",
            String::from_utf8_lossy(&variant.stderr)
        );
    } else {
        assert!(!variant.status.success(), "{label}: variant should fail");
    }
    assert_eq!(
        canonical.stdout, variant.stdout,
        "{label}: stdout should match canonical"
    );
    assert_eq!(
        canonical.stderr, variant.stderr,
        "{label}: stderr should match canonical"
    );
}

/// Absolute path variants with parent segments (requires anchor/ subdir).
fn abs_parent_segment_variants(root: &Path, target: &str) -> Vec<(&'static str, PathBuf)> {
    let target_path = root.join(target);
    vec![
        ("parent_segment", root.join(format!("anchor/../{target}"))),
        (
            "parent_segment_trailing",
            PathBuf::from(format!("{}/anchor/../{target}/", root.display())),
        ),
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

/// Absolute path variants using curdir segments (./).
fn abs_curdir_segment_variants(root: &Path, target: &str) -> Vec<(&'static str, PathBuf)> {
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

/// Absolute path variants using double separators and trailing slashes.
fn abs_separator_variants(root: &Path, target: &str) -> Vec<(&'static str, PathBuf)> {
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

/// All absolute path variants.
fn all_absolute_variants(root: &Path, target: &str) -> Vec<(&'static str, PathBuf)> {
    let mut v = abs_parent_segment_variants(root, target);
    v.extend(abs_curdir_segment_variants(root, target));
    v.extend(abs_separator_variants(root, target));
    v
}

/// Relative parent variants (from nested cwd).
fn rel_parent_variants(target: &str) -> Vec<(&'static str, String)> {
    vec![
        ("parent_segment", format!("../{target}")),
        ("parent_curdir_segment", format!(".././{target}")),
        ("parent_double_separator", format!("..//{target}")),
        ("parent_double_separator_trailing", format!("..//{target}/")),
        ("parent_curdir_double_separator", format!(".././/{target}")),
        ("parent_curdir_trailing_slash", format!(".././{target}/")),
    ]
}

/// Relative curdir/dot variants (from root).
fn rel_curdir_variants(target: &str) -> Vec<(&'static str, String)> {
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

// ── Compile command root path normalization helpers ─────────────────────────

/// Test absolute path variants for valid root.
fn assert_compile_absolute_valid_root_variants() {
    let cwd = workspace_root().join("core");
    let root = workspace_root();
    let canonical = run_compile_cli_path(&root.join("dsl"), &cwd);
    assert!(
        !canonical.status.success(),
        "canonical absolute-root compile should fail on ambiguous full-corpus bindings: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );
    let abs_stderr = String::from_utf8_lossy(&canonical.stderr);
    assert!(
        abs_stderr.contains("compile with --profile") || abs_stderr.contains("ambiguous"),
        "canonical absolute-root compile should report missing profile binding or ambiguous resource: {abs_stderr}"
    );

    for (label, variant_path) in all_absolute_variants(&root, "dsl") {
        let variant = run_compile_cli_path(&variant_path, &cwd);
        assert_compile_variant_matches(
            &canonical,
            &variant,
            &format!("abs_valid_root/{label}"),
            false,
        );
    }
}

/// Test relative parent variants for valid root.
fn assert_compile_relative_parent_valid_root_variants() {
    let cwd = workspace_root().join("core");
    let canonical = run_compile_cli_path(&workspace_root().join("dsl"), &cwd);
    assert!(
        !canonical.status.success(),
        "canonical compile should fail on ambiguous full-corpus bindings"
    );
    let stderr_text = String::from_utf8_lossy(&canonical.stderr);
    assert!(
        stderr_text.contains("compile with --profile") || stderr_text.contains("ambiguous"),
        "canonical compile should report missing profile binding or ambiguous resource: {stderr_text}"
    );

    for (label, variant) in rel_parent_variants("dsl") {
        let output = run_compile_cli(&variant, &cwd);
        assert_compile_variant_matches(
            &canonical,
            &output,
            &format!("rel_parent_valid/{label}"),
            false,
        );
    }
}

/// Test relative curdir variants for valid root.
fn assert_compile_relative_curdir_valid_root_variants() {
    let cwd = workspace_root();
    let canonical = run_compile_cli("dsl", &cwd);
    assert!(
        !canonical.status.success(),
        "canonical compile should fail on ambiguous full-corpus bindings"
    );
    let stderr_text = String::from_utf8_lossy(&canonical.stderr);
    assert!(
        stderr_text.contains("compile with --profile") || stderr_text.contains("ambiguous"),
        "canonical compile should report missing profile binding or ambiguous resource: {stderr_text}"
    );

    for (label, variant) in rel_curdir_variants("dsl") {
        let output = run_compile_cli(&variant, &cwd);
        assert_compile_variant_matches(
            &canonical,
            &output,
            &format!("rel_curdir_valid/{label}"),
            false,
        );
    }
}

/// Test path variants for missing root.
fn assert_compile_missing_root_variants() {
    let root = unique_temp_dir("compile_missing_root_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");

    let canonical = run_compile_cli_path(&root.join("missing_root"), &root);
    assert!(!canonical.status.success(), "missing root should fail");

    for (label, variant_path) in abs_parent_segment_variants(&root, "missing_root") {
        let variant = run_compile_cli_path(&variant_path, &root);
        assert_compile_variant_matches(
            &canonical,
            &variant,
            &format!("abs_missing_root_parent/{label}"),
            false,
        );
    }
    for (label, variant_path) in abs_curdir_segment_variants(&root, "missing_root") {
        let variant = run_compile_cli_path(&variant_path, &root);
        assert_compile_variant_matches(
            &canonical,
            &variant,
            &format!("abs_missing_root_curdir/{label}"),
            false,
        );
    }
    for (label, variant_path) in abs_separator_variants(&root, "missing_root") {
        let variant = run_compile_cli_path(&variant_path, &root);
        assert_compile_variant_matches(
            &canonical,
            &variant,
            &format!("abs_missing_root_sep/{label}"),
            false,
        );
    }

    let stderr = String::from_utf8_lossy(&canonical.stderr);
    assert!(
        stderr.contains(&root.join("missing_root").display().to_string()),
        "missing-root diagnostic should include canonical path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test relative variants for missing root.
fn assert_compile_relative_missing_root_variants() {
    let root = unique_temp_dir("compile_rel_missing_root_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");

    let canonical = run_compile_cli("missing_root", &root);
    assert!(!canonical.status.success(), "missing root should fail");

    for (label, variant) in rel_curdir_variants("missing_root") {
        let output = run_compile_cli(&variant, &root);
        assert_compile_variant_matches(
            &canonical,
            &output,
            &format!("rel_missing_root_curdir/{label}"),
            false,
        );
    }

    let cwd_nested = root.join("anchor");
    let canonical_abs = run_compile_cli_path(&root.join("missing_root"), &cwd_nested);
    for (label, variant) in rel_parent_variants("missing_root") {
        let output = run_compile_cli(&variant, &cwd_nested);
        assert_compile_variant_matches(
            &canonical_abs,
            &output,
            &format!("rel_missing_root_parent/{label}"),
            false,
        );
    }

    // Relative and absolute equivalence
    let rel = run_compile_cli("missing_root", &root);
    let abs = run_compile_cli_path(&root.join("missing_root"), &root);
    assert_eq!(
        rel.stdout, abs.stdout,
        "relative and absolute missing root stdout should match"
    );
    assert_eq!(
        rel.stderr, abs.stderr,
        "relative and absolute missing root stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test path variants for non-directory root.
fn assert_compile_non_directory_root_variants() {
    let root = unique_temp_dir("compile_non_dir_root_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to create root file");

    let canonical = run_compile_cli_path(&root_file, &root);
    assert!(
        !canonical.status.success(),
        "non-directory root should fail"
    );

    for (label, variant_path) in abs_parent_segment_variants(&root, "input.txt") {
        let variant = run_compile_cli_path(&variant_path, &root);
        assert_compile_variant_matches(
            &canonical,
            &variant,
            &format!("abs_non_dir_root_parent/{label}"),
            false,
        );
    }
    for (label, variant_path) in abs_curdir_segment_variants(&root, "input.txt") {
        let variant = run_compile_cli_path(&variant_path, &root);
        assert_compile_variant_matches(
            &canonical,
            &variant,
            &format!("abs_non_dir_root_curdir/{label}"),
            false,
        );
    }
    for (label, variant_path) in abs_separator_variants(&root, "input.txt") {
        let variant = run_compile_cli_path(&variant_path, &root);
        assert_compile_variant_matches(
            &canonical,
            &variant,
            &format!("abs_non_dir_root_sep/{label}"),
            false,
        );
    }

    let stderr = String::from_utf8_lossy(&canonical.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "non-directory-root diagnostic should include canonical path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test relative variants for non-directory root.
fn assert_compile_relative_non_directory_root_variants() {
    let root = unique_temp_dir("compile_rel_non_dir_root_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor dir");
    std::fs::write(root.join("input.txt"), "not a directory").expect("failed to create root file");

    let canonical = run_compile_cli("input.txt", &root);
    assert!(
        !canonical.status.success(),
        "non-directory root should fail"
    );

    for (label, variant) in rel_curdir_variants("input.txt") {
        let output = run_compile_cli(&variant, &root);
        assert_compile_variant_matches(
            &canonical,
            &output,
            &format!("rel_non_dir_root_curdir/{label}"),
            false,
        );
    }

    let cwd_nested = root.join("anchor");
    let canonical_abs = run_compile_cli_path(&root.join("input.txt"), &cwd_nested);
    for (label, variant) in rel_parent_variants("input.txt") {
        let output = run_compile_cli(&variant, &cwd_nested);
        assert_compile_variant_matches(
            &canonical_abs,
            &output,
            &format!("rel_non_dir_root_parent/{label}"),
            false,
        );
    }

    let rel = run_compile_cli("input.txt", &root);
    let abs = run_compile_cli_path(&root.join("input.txt"), &root);
    assert_eq!(
        rel.stdout, abs.stdout,
        "relative and absolute non-directory root stdout should match"
    );
    assert_eq!(
        rel.stderr, abs.stderr,
        "relative and absolute non-directory root stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

// ── Compile command single-file target path normalization helpers ───────────

/// Test relative curdir variants for missing single-file target.
fn assert_compile_missing_single_file_target_variants() {
    let root = unique_temp_dir("compile_missing_sf_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let canonical = run_compile_cli("missing.dag", &root);
    assert!(
        !canonical.status.success(),
        "missing single-file should fail"
    );

    for (label, variant) in rel_curdir_variants("missing.dag") {
        let output = run_compile_cli(&variant, &root);
        assert_compile_variant_matches(
            &canonical,
            &output,
            &format!("missing_sf_curdir/{label}"),
            false,
        );
    }

    let cwd_nested = root.join("anchor");
    let canonical_abs = run_compile_cli_path(&missing, &cwd_nested);
    for (label, variant) in rel_parent_variants("missing.dag") {
        let output = run_compile_cli(&variant, &cwd_nested);
        assert_compile_variant_matches(
            &canonical_abs,
            &output,
            &format!("missing_sf_parent/{label}"),
            false,
        );
    }

    // Relative and absolute equivalence
    let rel = run_compile_cli("missing.dag", &root);
    let abs = run_compile_cli_path(&missing, &root);
    assert_eq!(
        rel.stdout, abs.stdout,
        "relative and absolute missing single-file stdout should match"
    );
    assert_eq!(
        rel.stderr, abs.stderr,
        "relative and absolute missing single-file stderr should match"
    );

    let stderr = String::from_utf8_lossy(&rel.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "missing single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test relative curdir variants for invalid single-file target (directory masquerading as .dag).
fn assert_compile_invalid_single_file_target_variants() {
    let root = unique_temp_dir("compile_invalid_sf_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create temp root");
    let invalid = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid).expect("failed to create invalid target directory");

    let canonical = run_compile_cli("invalid.dag", &root);
    assert!(
        !canonical.status.success(),
        "invalid single-file should fail"
    );

    for (label, variant) in rel_curdir_variants("invalid.dag") {
        let output = run_compile_cli(&variant, &root);
        assert_compile_variant_matches(
            &canonical,
            &output,
            &format!("invalid_sf_curdir/{label}"),
            false,
        );
    }

    let cwd_nested = root.join("anchor");
    let canonical_abs = run_compile_cli_path(&invalid, &cwd_nested);
    for (label, variant) in rel_parent_variants("invalid.dag") {
        let output = run_compile_cli(&variant, &cwd_nested);
        assert_compile_variant_matches(
            &canonical_abs,
            &output,
            &format!("invalid_sf_parent/{label}"),
            false,
        );
    }

    // Relative and absolute equivalence
    let rel = run_compile_cli("invalid.dag", &root);
    let abs = run_compile_cli_path(&invalid, &root);
    assert_eq!(
        rel.stdout, abs.stdout,
        "relative and absolute invalid single-file stdout should match"
    );
    assert_eq!(
        rel.stderr, abs.stderr,
        "relative and absolute invalid single-file stderr should match"
    );

    let stderr = String::from_utf8_lossy(&rel.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid.display())),
        "invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test relative curdir variants for valid single-file target.
fn assert_compile_valid_single_file_target_variants() {
    let root = unique_temp_dir("compile_valid_sf_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create temp root");
    let target = root.join("sample.dag");
    std::fs::write(&target, "module sample.main\nfn run() -> Unit {}").expect("failed to write");

    let canonical = run_compile_cli("sample.dag", &root);
    assert!(
        canonical.status.success(),
        "valid single-file should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    for (label, variant) in rel_curdir_variants("sample.dag") {
        let output = run_compile_cli(&variant, &root);
        assert_compile_variant_matches(
            &canonical,
            &output,
            &format!("valid_sf_curdir/{label}"),
            true,
        );
    }

    let cwd_nested = root.join("anchor");
    let canonical_abs = run_compile_cli_path(&target, &cwd_nested);
    for (label, variant) in rel_parent_variants("sample.dag") {
        let output = run_compile_cli(&variant, &cwd_nested);
        assert_compile_variant_matches(
            &canonical_abs,
            &output,
            &format!("valid_sf_parent/{label}"),
            true,
        );
    }

    // Relative and absolute equivalence
    let rel = run_compile_cli("sample.dag", &root);
    let abs = run_compile_cli_path(&target, &root);
    assert_eq!(
        rel.stdout, abs.stdout,
        "relative and absolute single-file stdout should match"
    );
    assert_eq!(
        rel.stderr, abs.stderr,
        "relative and absolute single-file stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

/// Test extension case variants for single-file targets are rejected.
/// Only lowercase `.dag` is accepted; wrong-cased extensions produce a clear error.
fn assert_compile_extension_case_single_file_target_variants() {
    for ext in &[".DAG", ".DaG"] {
        let label = ext.replace('.', "");
        let root = unique_temp_dir(&format!("compile_ext_case_sf_{label}"));
        std::fs::create_dir_all(&root).expect("failed to create temp root");

        let filename = format!("sample{ext}");
        std::fs::write(
            root.join(&filename),
            "module sample.main\nfn run() -> Unit {}",
        )
        .expect("failed to write");

        let plain = run_compile_cli(&filename, &root);
        assert!(
            !plain.status.success(),
            "{ext} compile should fail due to wrong-cased extension"
        );
        let stderr = String::from_utf8_lossy(&plain.stderr);
        assert!(
            stderr.contains("wrong-cased extension"),
            "{ext} compile should report wrong-cased extension: {stderr}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup");
    }
}

// ── .dag extension directory helpers ───────────────────────────────────────

/// Test .dag-extension directory variants for compile command.
/// Only lowercase `.dag` is accepted; wrong-cased extensions are rejected up front.
fn assert_compile_dag_extension_directory_variants() {
    for ext in &[".dag"] {
        for has_errors in &[false, true] {
            let label_suffix = if *has_errors { "_errors" } else { "" };
            let test_name = format!("compile_dag_dir_{ext}{label_suffix}");
            let root = unique_temp_dir(&test_name);
            let dag_dir = root.join(format!("bundle{ext}"));
            std::fs::create_dir_all(dag_dir.join("sample"))
                .expect("failed to create .dag directory");
            std::fs::write(
                dag_dir.join("sample/main.dag"),
                "module sample.main\nfn run() -> Unit {}",
            )
            .expect("failed to write valid source");

            if *has_errors {
                std::fs::write(
                    dag_dir.join("sample/broken.dag"),
                    "module sample.broken\nfn",
                )
                .expect("failed to write broken source");
            }

            let nested_snippet = if *has_errors {
                Some("broken.dag:")
            } else {
                None
            };

            // Plain vs trailing slash
            assert_dag_suffixed_directory_is_invalid_single_file_target(
                &root,
                &format!("bundle{ext}"),
                &dag_dir,
                nested_snippet,
            );

            // Curdir suffix
            assert_dag_suffixed_directory_is_invalid_single_file_target(
                &root,
                &format!("./bundle{ext}"),
                &dag_dir,
                nested_snippet,
            );

            std::fs::remove_dir_all(root).expect("failed to cleanup");
        }
    }
}

/// Test .dag-extension symlink directory variants for compile command.
/// Only lowercase `.dag` is accepted; wrong-cased extensions are rejected up front.
#[cfg(unix)]
fn assert_compile_dag_extension_symlink_directory_variants() {
    use std::os::unix::fs::symlink;
    let ext = ".dag";
    let test_name = format!("compile_dag_symlink_{ext}");
    let root = unique_temp_dir(&test_name);
    let real_dir = root.join("real");
    let link = root.join(format!("link{ext}"));
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real dir");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source");
    symlink(&real_dir, &link).expect("failed to create symlink");

    // Symlink treated as invalid single-file target
    let (plain, trailing) = run_compile_with_optional_trailing_slash(&root, &format!("link{ext}"));
    assert!(
        !plain.status.success(),
        "symlink .dag dir should fail as single-file target"
    );
    assert_eq!(plain.stdout, trailing.stdout);
    assert_eq!(plain.stderr, trailing.stderr);

    // Curdir suffix
    let curdir = run_compile_cli(&format!("./link{ext}"), &root);
    assert_eq!(
        plain.stdout, curdir.stdout,
        "curdir symlink stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir.stderr,
        "curdir symlink stderr should match"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup");
}

// ── Single-target command path normalization helpers (obligations, show-triplets) ──

/// Test absolute path variants for a single-target command using makegen.dag.
fn assert_single_target_absolute_variants(command_name: &str, extra_args: &[&str]) {
    let root = workspace_root();
    let canonical_target = makegen_file().display().to_string();

    let variants: Vec<(&str, String)> = vec![
        (
            "curdir_segment",
            root.join("./dsl/./tools/../tools/makegen.dag")
                .to_string_lossy()
                .into_owned(),
        ),
        (
            "double_separator",
            format!("{}/dsl//tools///makegen.dag", root.display()),
        ),
        (
            "parent_segment",
            root.join("dsl/../dsl/tools/makegen.dag")
                .to_string_lossy()
                .into_owned(),
        ),
        (
            "parent_curdir_segment",
            root.join("dsl/tools/./../tools/makegen.dag")
                .to_string_lossy()
                .into_owned(),
        ),
        (
            "parent_double_separator",
            format!("{}/dsl/..//dsl/tools/makegen.dag", root.display()),
        ),
        (
            "parent_curdir_double_separator",
            format!("{}/dsl/tools/.//../tools/makegen.dag", root.display()),
        ),
    ];

    for (label, variant_target) in &variants {
        assert_single_target_command_outputs_match_for_targets(
            command_name,
            &root,
            &canonical_target,
            variant_target,
            extra_args,
            &format!("absolute_{label}"),
        );
    }
}

/// Test curdir-suffix and relative-vs-absolute equivalence for single-target commands.
fn assert_single_target_curdir_and_equiv_variants(command_name: &str, extra_args: &[&str]) {
    let root = workspace_root();
    let relative_target = "dsl/tools/makegen.dag";
    let absolute_target = makegen_file().display().to_string();

    // curdir_suffix
    assert_single_target_command_outputs_match_for_targets(
        command_name,
        &root,
        relative_target,
        &format!("./{relative_target}"),
        extra_args,
        "curdir_suffix",
    );

    // relative and absolute equivalence
    assert_single_target_command_outputs_match_for_targets(
        command_name,
        &root,
        relative_target,
        &absolute_target,
        extra_args,
        "relative_absolute_equiv",
    );
}

/// Test extension case variants for single-target commands are rejected.
/// Only lowercase `.dag` is accepted; wrong-cased extensions produce a clear error.
fn assert_single_target_extension_case_variants(command_name: &str, extra_args: &[&str]) {
    for (ext_label, ext) in &[("uppercase", ".DAG"), ("mixed_case", ".DaG")] {
        let root = unique_temp_dir(&format!("{command_name}_{ext_label}_ext_variant"));
        std::fs::create_dir_all(root.join("sample")).expect("failed to create fixture dir");
        let filename = format!("main{ext}");
        std::fs::write(
            root.join(format!("sample/{filename}")),
            "module sample.main\nfn run() -> Unit {}",
        )
        .expect("failed to write source");

        let target = format!("sample/{filename}");
        let output = run_single_target_command(command_name, &root, &target, extra_args)
            .expect("failed to run command");
        assert!(
            !output.status.success(),
            "{command_name} {ext_label} should fail due to wrong-cased extension"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("wrong-cased extension"),
            "{command_name} {ext_label} should report wrong-cased extension: {stderr}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup");
    }
}

/// Test .dag extension directory rejection for single-target commands.
/// Only lowercase `.dag` is accepted; wrong-cased extensions are rejected up front.
fn assert_single_target_dag_ext_directory_variants(command_name: &str, extra_args: &[&str]) {
    for ext in &[".dag"] {
        for is_symlink in &[false, true] {
            let label = if *is_symlink { "symlink" } else { "directory" };
            let test_name = format!("{command_name}_{label}_{ext}_dag_ext");
            let root = unique_temp_dir(&test_name);
            let dag_dir = root.join(format!("bundle{ext}"));

            if *is_symlink {
                #[cfg(unix)]
                {
                    let real_dir = root.join("real");
                    std::fs::create_dir_all(real_dir.join("sample"))
                        .expect("failed to create real dir");
                    std::fs::write(
                        real_dir.join("sample/main.dag"),
                        "module sample.main\nfn run() -> Unit {}",
                    )
                    .expect("failed to write source");
                    std::os::unix::fs::symlink(&real_dir, &dag_dir)
                        .expect("failed to create symlink");
                }
                #[cfg(not(unix))]
                continue;
            } else {
                std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .dag dir");
                std::fs::write(
                    dag_dir.join("sample/main.dag"),
                    "module sample.main\nfn run() -> Unit {}",
                )
                .expect("failed to write source");
            }

            // Plain
            assert_single_target_command_treats_dag_directory_as_invalid_single_file_target_with_args(
                command_name, &root, &format!("bundle{ext}"), &dag_dir, None, extra_args,
            );

            // Curdir suffix
            assert_single_target_command_treats_dag_directory_as_invalid_single_file_target_with_args(
                command_name, &root, &format!("./bundle{ext}"), &dag_dir, None, extra_args,
            );

            // Absolute
            let abs_input = dag_dir.display().to_string();
            assert_single_target_command_treats_dag_directory_as_invalid_single_file_target_with_args(
                command_name, &root, &abs_input, &dag_dir, None, extra_args,
            );

            // Absolute with path manipulation variants
            std::fs::create_dir_all(root.join("nested")).expect("failed to create nested dir");
            let path_variants: Vec<(&str, String)> = vec![
                (
                    "abs_parent_segment",
                    root.join(format!("nested/../bundle{ext}"))
                        .to_string_lossy()
                        .into_owned(),
                ),
                (
                    "abs_parent_double_separator",
                    format!("{}/nested/..//bundle{ext}", root.display()),
                ),
                (
                    "abs_parent_curdir_segment",
                    root.join(format!("nested/./../bundle{ext}"))
                        .to_string_lossy()
                        .into_owned(),
                ),
                (
                    "abs_parent_curdir_double_separator",
                    format!("{}/nested/.//../bundle{ext}", root.display()),
                ),
                (
                    "abs_curdir_segment",
                    root.join(format!("./bundle{ext}"))
                        .to_string_lossy()
                        .into_owned(),
                ),
                (
                    "abs_double_separator",
                    format!("{}/./bundle{ext}", root.display()),
                ),
            ];
            for (_plabel, variant_input) in &path_variants {
                assert_single_target_command_treats_dag_directory_as_invalid_single_file_target_with_args(
                    command_name, &root, variant_input, &dag_dir, None, extra_args,
                );
            }

            std::fs::remove_dir_all(root).expect("failed to cleanup");
        }
    }
}

/// Test .dag extension directory with symlink alias failure outputs for single-target commands.
fn assert_single_target_dag_ext_symlink_fail_variants(command_name: &str, extra_args: &[&str]) {
    #[cfg(unix)]
    {
        let root = unique_temp_dir(&format!("{command_name}_dag_symlink_fail"));
        let real_dir = root.join("bundle.dag");
        std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create dag dir");
        std::fs::write(
            real_dir.join("sample/main.dag"),
            "module sample.main\nfn run() -> Unit {}",
        )
        .expect("failed to write source");
        let link = root.join("alias.dag");
        std::os::unix::fs::symlink(&real_dir, &link).expect("failed to create symlink");

        // Directory vs symlink alias should both fail
        let dir_output = run_single_target_command(command_name, &root, "bundle.dag", extra_args)
            .expect("failed to run directory variant");
        let link_output = run_single_target_command(command_name, &root, "alias.dag", extra_args)
            .expect("failed to run symlink variant");
        assert!(
            !dir_output.status.success(),
            "directory variant should fail"
        );
        assert!(!link_output.status.success(), "symlink variant should fail");

        // Relative and absolute equivalence for directory alias
        let abs_dir = real_dir.display().to_string();
        let abs_dir_output = run_single_target_command(command_name, &root, &abs_dir, extra_args)
            .expect("failed to run abs directory variant");
        assert_eq!(
            dir_output.stdout, abs_dir_output.stdout,
            "directory alias rel/abs stdout"
        );
        assert_eq!(
            dir_output.stderr, abs_dir_output.stderr,
            "directory alias rel/abs stderr"
        );

        // Relative and absolute equivalence for symlink alias
        let abs_link = link.display().to_string();
        let abs_link_output = run_single_target_command(command_name, &root, &abs_link, extra_args)
            .expect("failed to run abs symlink variant");
        assert_eq!(
            link_output.stdout, abs_link_output.stdout,
            "symlink alias rel/abs stdout"
        );
        assert_eq!(
            link_output.stderr, abs_link_output.stderr,
            "symlink alias rel/abs stderr"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup");
    }
}

// ── Table-driven test wrappers ─────────────────────────────────────────────

#[test]
fn compile_command_absolute_valid_root_variants_match_canonical() {
    assert_compile_absolute_valid_root_variants();
}

#[test]
fn compile_command_relative_parent_valid_root_variants_match_canonical() {
    assert_compile_relative_parent_valid_root_variants();
}

#[test]
fn compile_command_relative_curdir_valid_root_variants_match_canonical() {
    assert_compile_relative_curdir_valid_root_variants();
}

#[test]
fn compile_command_missing_root_variants_match_canonical() {
    assert_compile_missing_root_variants();
}

#[test]
fn compile_command_relative_missing_root_variants_match_canonical() {
    assert_compile_relative_missing_root_variants();
}

#[test]
fn compile_command_non_directory_root_variants_match_canonical() {
    assert_compile_non_directory_root_variants();
}

#[test]
fn compile_command_relative_non_directory_root_variants_match_canonical() {
    assert_compile_relative_non_directory_root_variants();
}

#[test]
fn compile_command_missing_single_file_target_variants_match_canonical() {
    assert_compile_missing_single_file_target_variants();
}

#[test]
fn compile_command_invalid_single_file_target_variants_match_canonical() {
    assert_compile_invalid_single_file_target_variants();
}

#[test]
fn compile_command_valid_single_file_target_variants_match_canonical() {
    assert_compile_valid_single_file_target_variants();
}

#[test]
fn compile_command_extension_case_single_file_target_variants() {
    assert_compile_extension_case_single_file_target_variants();
}

#[test]
fn compile_command_dag_extension_directory_variants_match() {
    assert_compile_dag_extension_directory_variants();
}

#[cfg(unix)]
#[test]
fn compile_command_dag_extension_symlink_directory_variants_match() {
    assert_compile_dag_extension_symlink_directory_variants();
}

#[test]
fn obligations_command_absolute_target_variants_match_canonical() {
    assert_single_target_absolute_variants("obligations", &[]);
}

#[test]
fn obligations_command_curdir_and_equiv_variants_match_canonical() {
    assert_single_target_curdir_and_equiv_variants("obligations", &[]);
}

#[test]
fn obligations_command_extension_case_variants_match_canonical() {
    assert_single_target_extension_case_variants("obligations", &[]);
}

#[test]
fn obligations_command_dag_ext_directory_variants_match() {
    assert_single_target_dag_ext_directory_variants("obligations", &[]);
}

#[test]
fn obligations_command_dag_ext_symlink_fail_variants_match() {
    assert_single_target_dag_ext_symlink_fail_variants("obligations", &[]);
}

#[test]
fn obligations_command_json_absolute_target_variants_match_canonical() {
    assert_single_target_absolute_variants("obligations", &["--format", "json"]);
}

#[test]
fn obligations_command_json_curdir_and_equiv_variants_match_canonical() {
    assert_single_target_curdir_and_equiv_variants("obligations", &["--format", "json"]);
}

#[test]
fn obligations_command_json_extension_case_variants_match_canonical() {
    assert_single_target_extension_case_variants("obligations", &["--format", "json"]);
}

#[test]
fn obligations_command_json_dag_ext_directory_variants_match() {
    assert_single_target_dag_ext_directory_variants("obligations", &["--format", "json"]);
}

#[test]
fn obligations_command_json_dag_ext_symlink_fail_variants_match() {
    assert_single_target_dag_ext_symlink_fail_variants("obligations", &["--format", "json"]);
}

#[test]
fn show_triplets_command_absolute_target_variants_match_canonical() {
    assert_single_target_absolute_variants("show-triplets", &[]);
}

#[test]
fn show_triplets_command_curdir_and_equiv_variants_match_canonical() {
    assert_single_target_curdir_and_equiv_variants("show-triplets", &[]);
}

#[test]
fn show_triplets_command_extension_case_variants_match_canonical() {
    assert_single_target_extension_case_variants("show-triplets", &[]);
}

#[test]
fn show_triplets_command_dag_ext_directory_variants_match() {
    assert_single_target_dag_ext_directory_variants("show-triplets", &[]);
}

#[test]
fn show_triplets_command_dag_ext_symlink_fail_variants_match() {
    assert_single_target_dag_ext_symlink_fail_variants("show-triplets", &[]);
}

#[test]
fn show_triplets_command_json_absolute_target_variants_match_canonical() {
    assert_single_target_absolute_variants("show-triplets", &["--format", "json"]);
}

#[test]
fn show_triplets_command_json_curdir_and_equiv_variants_match_canonical() {
    assert_single_target_curdir_and_equiv_variants("show-triplets", &["--format", "json"]);
}

#[test]
fn show_triplets_command_json_extension_case_variants_match_canonical() {
    assert_single_target_extension_case_variants("show-triplets", &["--format", "json"]);
}

#[test]
fn show_triplets_command_json_dag_ext_directory_variants_match() {
    assert_single_target_dag_ext_directory_variants("show-triplets", &["--format", "json"]);
}

#[test]
fn show_triplets_command_json_dag_ext_symlink_fail_variants_match() {
    assert_single_target_dag_ext_symlink_fail_variants("show-triplets", &["--format", "json"]);
}

#[test]
fn compile_command_emits_summary_for_single_file() {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        output.status.success(),
        "compile command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("module(s)"),
        "compile output should contain module summary, got: {stdout}"
    );
    assert!(stdout.contains("target/generated/rust/main.rs"));
}

#[test]
fn compile_command_canonical_json_is_deterministic_for_single_file() {
    let first = Command::new(daglang_bin())
        .arg("compile")
        .arg(makegen_file())
        .arg("--format")
        .arg("canonical-json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first canonical-json compile");
    let second = Command::new(daglang_bin())
        .arg("compile")
        .arg(makegen_file())
        .arg("--format")
        .arg("canonical-json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second canonical-json compile");

    assert!(
        first.status.success(),
        "first canonical-json compile failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "second canonical-json compile failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&first.stderr));
    assert_no_stage_failures(&String::from_utf8_lossy(&second.stderr));
    assert_eq!(
        first.stdout, second.stdout,
        "canonical-json compile output must be deterministic across repeated runs"
    );

    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(
        !stdout.contains("Compiled "),
        "canonical-json mode should output only canonical IR JSON"
    );
    let parsed: Value =
        serde_json::from_slice(&first.stdout).expect("canonical-json output should be valid JSON");
    let nodes = parsed
        .get("nodes")
        .and_then(Value::as_array)
        .expect("canonical-json should include nodes array");
    let edges = parsed
        .get("edges")
        .and_then(Value::as_array)
        .expect("canonical-json should include edges array");
    assert!(
        !nodes.is_empty(),
        "canonical-json nodes array should not be empty"
    );
    assert!(
        !edges.is_empty(),
        "canonical-json edges array should not be empty"
    );
}

#[test]
fn compile_command_canonical_json_is_deterministic_for_ci_pipeline() {
    let first = Command::new(daglang_bin())
        .arg("compile")
        .arg(ci_pipeline_file())
        .arg("--format")
        .arg("canonical-json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first canonical-json compile for ci pipeline");
    let second = Command::new(daglang_bin())
        .arg("compile")
        .arg(ci_pipeline_file())
        .arg("--format")
        .arg("canonical-json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second canonical-json compile for ci pipeline");

    assert!(
        first.status.success(),
        "first canonical-json ci compile failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "second canonical-json ci compile failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&first.stderr));
    assert_no_stage_failures(&String::from_utf8_lossy(&second.stderr));
    assert_eq!(
        first.stdout, second.stdout,
        "canonical-json ci compile output must be deterministic across repeated runs"
    );

    let parsed: Value =
        serde_json::from_slice(&first.stdout).expect("canonical-json output should be valid JSON");
    let nodes = parsed
        .get("nodes")
        .and_then(Value::as_array)
        .expect("canonical-json should include nodes array");
    let edges = parsed
        .get("edges")
        .and_then(Value::as_array)
        .expect("canonical-json should include edges array");
    assert!(
        nodes.len() > 20,
        "ci canonical-json should include substantial node topology"
    );
    assert!(
        edges.len() > 20,
        "ci canonical-json should include substantial edge topology"
    );
}

#[test]
fn compile_command_canonical_json_rejects_out_flag() {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(makegen_file())
        .arg("--format")
        .arg("canonical-json")
        .arg("--out")
        .arg("target/generated/canonical")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run canonical-json compile with --out");

    assert!(
        !output.status.success(),
        "canonical-json with --out should fail usage validation"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: daglang compile <file.dag|dir>"),
        "canonical-json with --out should print compile usage: {stderr}"
    );
    assert!(
        stderr.contains("--format summary|canonical-json"),
        "compile usage should document canonical-json format: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn compile_command_accepts_symlink_single_file_target() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_symlink_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let real = root.join("real.dag");
    let link = root.join("link.dag");
    std::fs::write(&real, "module sample.real\nfn ok() -> Unit {}")
        .expect("failed to write real source");
    symlink(&real, &link).expect("failed to create symlinked target");

    let real_output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&real)
        .current_dir(&root)
        .output()
        .expect("failed to run daglang compile on real target");
    assert!(
        real_output.status.success(),
        "compile should succeed for real single-file target: {}",
        String::from_utf8_lossy(&real_output.stderr)
    );

    let link_output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&link)
        .current_dir(&root)
        .output()
        .expect("failed to run daglang compile on symlinked target");
    assert!(
        link_output.status.success(),
        "compile should succeed for symlinked single-file target: {}",
        String::from_utf8_lossy(&link_output.stderr)
    );

    assert_eq!(
        real_output.stdout, link_output.stdout,
        "real and symlink single-file compile stdout should match"
    );
    assert_eq!(
        real_output.stderr, link_output.stderr,
        "real and symlink single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&link_output.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_accepts_symlink_root_directory() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_symlink_root_directory");
    let real_root = root.join("real_root");
    std::fs::create_dir_all(real_root.join("sample")).expect("failed to create real root fixture");
    std::fs::write(
        real_root.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit { }",
    )
    .expect("failed to write real root dag");
    let link_root = root.join("link_root");
    symlink(&real_root, &link_root).expect("failed to create symlinked root");

    let real_output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&real_root)
        .current_dir(&root)
        .output()
        .expect("failed to run daglang compile on real root");
    assert!(
        real_output.status.success(),
        "compile should succeed for real root directory: {}",
        String::from_utf8_lossy(&real_output.stderr)
    );

    let link_output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&link_root)
        .current_dir(&root)
        .output()
        .expect("failed to run daglang compile on symlinked root");
    assert!(
        link_output.status.success(),
        "compile should succeed for symlinked root directory: {}",
        String::from_utf8_lossy(&link_output.stderr)
    );

    assert_eq!(
        real_output.stdout, link_output.stdout,
        "real and symlink root compile stdout should match"
    );
    assert_eq!(
        real_output.stderr, link_output.stderr,
        "real and symlink root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&link_output.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_dangling_symlink_single_file_target_exits_nonzero() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_dangling_symlink_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink target");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&dangling_link)
        .current_dir(&root)
        .output()
        .expect("failed to run daglang compile on dangling symlink target");
    assert!(
        !output.status.success(),
        "compile should fail for dangling symlink single-file target"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to read"));
    assert!(
        stderr.contains("broken.dag"),
        "dangling symlink compile failure should include offending path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_relative_and_absolute_dangling_symlink_targets_are_equivalent() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_relative_absolute_dangling_symlink_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dangling_target = root.join("missing.dag");
    let dangling_link = root.join("broken.dag");
    symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink target");

    let relative = Command::new(daglang_bin())
        .arg("compile")
        .arg("broken.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative dangling-target daglang compile");
    assert!(
        !relative.status.success(),
        "relative dangling-target compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&dangling_link)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute dangling-target daglang compile");
    assert!(
        !absolute.status.success(),
        "absolute dangling-target compile should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute dangling-target compile stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute dangling-target compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", dangling_link.display())),
        "dangling-target compile diagnostics should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_double_separator_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(
        !plain.status.success(),
        "plain missing-root compile should fail"
    );

    let curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double-separator missing-root compile");
    assert!(
        !curdir_double.status.success(),
        "curdir-double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_double.stdout,
        "plain and curdir-double missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_double.stderr,
        "plain and curdir-double missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-double missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_double_invalid_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("invalid.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain invalid single-file compile");
    assert!(
        !plain.status.success(),
        "plain invalid single-file compile should fail"
    );

    let curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double invalid single-file compile");
    assert!(
        !curdir_double.status.success(),
        "curdir-double invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_double.stdout,
        "plain and curdir-double invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_double.stderr,
        "plain and curdir-double invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir-double invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_double_separator_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_double_separator_single_file_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain single-file compile");
    assert!(
        plain.status.success(),
        "plain single-file compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double single-file compile");
    assert!(
        curdir_double.status.success(),
        "curdir-double single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_double.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_double.stdout,
        "plain and curdir-double single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_double.stderr,
        "plain and curdir-double single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_segment_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg("../dsl")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-segment root compile");
    assert!(
        parent_segment.status.success(),
        "parent-segment root compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_root)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute root compile");
    assert!(
        absolute.status.success(),
        "absolute root compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute root compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_double_separator_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let parent_double = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//dsl")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double root compile");
    assert!(
        parent_double.status.success(),
        "parent-double root compile should succeed: {}",
        String::from_utf8_lossy(&parent_double.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_root)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute root compile");
    assert!(
        absolute.status.success(),
        "absolute root compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double.stdout, absolute.stdout,
        "parent-double and absolute root compile stdout should match"
    );
    assert_eq!(
        parent_double.stderr, absolute.stderr,
        "parent-double and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_double.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_segment_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root child");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root sample");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg("../sample/main.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-segment single-file compile");
    assert!(
        parent_segment.status.success(),
        "parent-segment single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&source)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute single-file compile");
    assert!(
        absolute.status.success(),
        "absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_double_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root child");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root sample");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let parent_double = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//sample/main.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double single-file compile");
    assert!(
        parent_double.status.success(),
        "parent-double single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_double.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&source)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute single-file compile");
    assert!(
        absolute.status.success(),
        "absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_double.stdout, absolute.stdout,
        "parent-double and absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_double.stderr, absolute.stderr,
        "parent-double and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_double.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_trailing_slash_missing_root_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_curdir_trailing_missing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let parent_curdir_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././missing_root/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-trailing missing-root compile");
    assert!(
        !parent_curdir_trailing.status.success(),
        "parent-curdir-trailing missing-root compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&missing)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute missing-root compile");
    assert!(
        !absolute.status.success(),
        "absolute missing-root compile should fail"
    );

    assert_eq!(
        parent_curdir_trailing.stdout, absolute.stdout,
        "parent-curdir-trailing and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing.stderr, absolute.stderr,
        "parent-curdir-trailing and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_trailing.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "parent-curdir-trailing missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_trailing_slash_non_directory_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_trailing_non_directory_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let parent_curdir_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././input.txt/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-trailing non-directory-root compile");
    assert!(
        !parent_curdir_trailing.status.success(),
        "parent-curdir-trailing non-directory-root compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root_file)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute non-directory-root compile");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root compile should fail"
    );

    assert_eq!(
        parent_curdir_trailing.stdout, absolute.stdout,
        "parent-curdir-trailing and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing.stderr, absolute.stderr,
        "parent-curdir-trailing and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_trailing.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "parent-curdir-trailing non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_segment_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_segment_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././dsl")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-segment root compile");
    assert!(
        parent_curdir_segment.status.success(),
        "parent-curdir-segment root compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_root)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute root compile");
    assert!(
        absolute.status.success(),
        "absolute root compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_double_separator_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_double_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let parent_curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././dsl//")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-double root compile");
    assert!(
        parent_curdir_double.status.success(),
        "parent-curdir-double root compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_double.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_root)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute root compile");
    assert!(
        absolute.status.success(),
        "absolute root compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_double.stdout, absolute.stdout,
        "parent-curdir-double and absolute root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_double.stderr, absolute.stderr,
        "parent-curdir-double and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_double.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_trailing_slash_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_trailing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let parent_curdir_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././dsl/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-trailing root compile");
    assert!(
        parent_curdir_trailing.status.success(),
        "parent-curdir-trailing root compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_trailing.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_root)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute root compile");
    assert!(
        absolute.status.success(),
        "absolute root compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_trailing.stdout, absolute.stdout,
        "parent-curdir-trailing and absolute root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing.stderr, absolute.stderr,
        "parent-curdir-trailing and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_trailing_slash_missing_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_curdir_trailing_missing_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let parent_curdir_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././missing.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-trailing missing single-file compile");
    assert!(
        !parent_curdir_trailing.status.success(),
        "parent-curdir-trailing missing single-file compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&missing)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute missing single-file compile");
    assert!(
        !absolute.status.success(),
        "absolute missing single-file compile should fail"
    );

    assert_eq!(
        parent_curdir_trailing.stdout, absolute.stdout,
        "parent-curdir-trailing and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing.stderr, absolute.stderr,
        "parent-curdir-trailing and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_trailing.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "parent-curdir-trailing missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_trailing_slash_invalid_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_curdir_trailing_invalid_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let parent_curdir_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././invalid.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-trailing invalid single-file compile");
    assert!(
        !parent_curdir_trailing.status.success(),
        "parent-curdir-trailing invalid single-file compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&invalid_target)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute invalid single-file compile");
    assert!(
        !absolute.status.success(),
        "absolute invalid single-file compile should fail"
    );

    assert_eq!(
        parent_curdir_trailing.stdout, absolute.stdout,
        "parent-curdir-trailing and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing.stderr, absolute.stderr,
        "parent-curdir-trailing and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_trailing.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "parent-curdir-trailing invalid single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_segment_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_segment_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root child");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root sample");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././sample/main.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-segment single-file compile");
    assert!(
        parent_curdir_segment.status.success(),
        "parent-curdir-segment single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&source)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute single-file compile");
    assert!(
        absolute.status.success(),
        "absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_double_separator_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_double_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root child");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root sample");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let parent_curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././sample//main.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-double single-file compile");
    assert!(
        parent_curdir_double.status.success(),
        "parent-curdir-double single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_double.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&source)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute single-file compile");
    assert!(
        absolute.status.success(),
        "absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_double.stdout, absolute.stdout,
        "parent-curdir-double and absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_double.stderr, absolute.stderr,
        "parent-curdir-double and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_double.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_trailing_slash_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_trailing_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root child");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root sample");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let parent_curdir_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././sample/main.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-trailing single-file compile");
    assert!(
        parent_curdir_trailing.status.success(),
        "parent-curdir-trailing single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_trailing.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&source)
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run absolute single-file compile");
    assert!(
        absolute.status.success(),
        "absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        parent_curdir_trailing.stdout, absolute.stdout,
        "parent-curdir-trailing and absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_trailing.stderr, absolute.stderr,
        "parent-curdir-trailing and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_trailing_slash_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_trailing_slash_single_file_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain single-file compile");
    assert!(
        plain.status.success(),
        "plain single-file compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash single-file compile");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash single-file compile should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    assert_eq!(
        plain.stdout, trailing_slash.stdout,
        "plain and trailing-slash single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing_slash.stderr,
        "plain and trailing-slash single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_single_file_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain single-file compile");
    assert!(
        plain.status.success(),
        "plain single-file compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir = Command::new(daglang_bin())
        .arg("compile")
        .arg("./sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment single-file compile");
    assert!(
        curdir.status.success(),
        "curdir-segment single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir.stderr)
    );

    assert_eq!(
        plain.stdout, curdir.stdout,
        "plain and curdir single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir.stderr,
        "plain and curdir single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_suffix_single_file_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain single-file compile");
    assert!(
        plain.status.success(),
        "plain single-file compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./sample/main.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix single-file compile");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_double_separator_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_double_separator_single_file_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain single-file compile");
    assert!(
        plain.status.success(),
        "plain single-file compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let double_sep = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample//main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator single-file compile");
    assert!(
        double_sep.status.success(),
        "double-separator single-file compile should succeed: {}",
        String::from_utf8_lossy(&double_sep.stderr)
    );

    assert_eq!(
        plain.stdout, double_sep.stdout,
        "plain and double-separator single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, double_sep.stderr,
        "plain and double-separator single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_default_root_missing_in_cwd_exits_nonzero() {
    let root = unique_temp_dir("compile_default_root_missing");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let expected = root.join("dsl");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .current_dir(&root)
        .output()
        .expect("failed to run compile default-root check");

    assert!(
        !output.status.success(),
        "compile default-root run should fail when dsl directory is missing"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "resolve error: invalid discovery root {}: does not exist",
            expected.display()
        )),
        "default-root missing diagnostic should include normalized absolute dsl path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_default_root_non_directory_in_cwd_exits_nonzero() {
    let root = unique_temp_dir("compile_default_root_non_directory");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let dsl_file = root.join("dsl");
    std::fs::write(&dsl_file, "not a directory").expect("failed to write dsl file");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .current_dir(&root)
        .output()
        .expect("failed to run compile default-root check");

    assert!(
        !output.status.success(),
        "compile default-root run should fail when dsl path is not a directory"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "resolve error: invalid discovery root {}: is not a directory",
            dsl_file.display()
        )),
        "default-root non-directory diagnostic should include normalized absolute dsl path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_default_root_matches_explicit_dsl_output() {
    let root = unique_temp_dir("compile_default_root_matches_explicit");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);

    let default = Command::new(daglang_bin())
        .arg("compile")
        .current_dir(&root)
        .output()
        .expect("failed to run default-root compile");
    assert!(
        default.status.success(),
        "default-root compile should succeed: {}",
        String::from_utf8_lossy(&default.stderr)
    );

    let explicit = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run explicit-root compile");
    assert!(
        explicit.status.success(),
        "explicit-root compile should succeed: {}",
        String::from_utf8_lossy(&explicit.stderr)
    );

    assert_eq!(
        default.stdout, explicit.stdout,
        "default-root and explicit dsl-root compile stdout should match"
    );
    assert_eq!(
        default.stderr, explicit.stderr,
        "default-root and explicit dsl-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&default.stderr));
    assert!(
        String::from_utf8_lossy(&default.stdout).contains("Compiled 1 module(s)"),
        "default-root compile summary should report a single compiled module: {}",
        String::from_utf8_lossy(&default.stdout)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_mixed_segment_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_mixed_segment_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_mixed_root = root.join(".").join("dsl");
    let canonical_root = root.join("dsl");

    let mixed = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_mixed_root)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute root compile");
    assert!(
        mixed.status.success(),
        "mixed-segment absolute root compile should succeed: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&mixed.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_trailing_slash_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_curdir_segment_trailing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_curdir_segment_trailing_root = PathBuf::from(format!("{}/./dsl/", root.display()));
    let canonical_root = root.join("dsl");

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_trailing_root)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing absolute root compile");
    assert!(
        curdir_segment_trailing.status.success(),
        "curdir-segment-trailing absolute root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_trailing.stdout, canonical.stdout,
        "curdir-segment-trailing and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing.stderr, canonical.stderr,
        "curdir-segment-trailing and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&curdir_segment_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_double_separator_root_matches_canonical_absolute_output()
{
    let root = unique_temp_dir("compile_absolute_curdir_segment_double_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_root = root.join("dsl");
    let absolute_curdir_segment_double_separator_root =
        PathBuf::from(format!("{}//./", canonical_root.display()));

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_double_separator_root)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator absolute root compile");
    assert!(
        curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator absolute root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, canonical.stdout,
        "curdir-segment-double-separator and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, canonical.stderr,
        "curdir-segment-double-separator and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &curdir_segment_double_separator.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_suffix_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_curdir_suffix_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_root = root.join("dsl");
    let absolute_curdir_suffix_root = root.join("dsl/.");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_suffix_root)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix absolute root compile");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix absolute root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_suffix.stdout, canonical.stdout,
        "curdir-suffix and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, canonical.stderr,
        "curdir-suffix and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&curdir_suffix.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_suffix_double_separator_root_matches_canonical_absolute_output()
{
    let root = unique_temp_dir("compile_absolute_curdir_suffix_double_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_root = root.join("dsl");
    let absolute_curdir_suffix_double_separator_root =
        PathBuf::from(format!("{}//.", canonical_root.display()));

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_suffix_double_separator_root)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator absolute root compile");
    assert!(
        curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator absolute root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, canonical.stdout,
        "curdir-suffix-double-separator and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, canonical.stderr,
        "curdir-suffix-double-separator and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &curdir_suffix_double_separator.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_double_separator_suffix_root_matches_canonical_absolute_output(
) {
    let root = unique_temp_dir("compile_absolute_curdir_segment_double_suffix_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_root = root.join("dsl");
    let absolute_curdir_segment_double_separator_suffix_root =
        PathBuf::from(format!("{}//./.", canonical_root.display()));

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_double_separator_suffix_root)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator-suffix absolute root compile");
    assert!(
        curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix absolute root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, canonical.stdout,
        "curdir-segment-double-separator-suffix and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, canonical.stderr,
        "curdir-segment-double-separator-suffix and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &curdir_segment_double_separator_suffix.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_suffix_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_curdir_segment_suffix_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_root = root.join("dsl");
    let absolute_curdir_segment_suffix_root = root.join("dsl/./.");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_suffix_root)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix absolute root compile");
    assert!(
        curdir_segment_suffix.status.success(),
        "curdir-segment-suffix absolute root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_suffix.stdout, canonical.stdout,
        "curdir-segment-suffix and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, canonical.stderr,
        "curdir-segment-suffix and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&curdir_segment_suffix.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_curdir_segment_root = root.join("anchor/.././dsl");
    let canonical_root = root.join("dsl");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_root)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment absolute root compile");
    assert!(
        parent_curdir_segment.status.success(),
        "parent-curdir-segment absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment.stdout, canonical.stdout,
        "parent-curdir-segment and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, canonical.stderr,
        "parent-curdir-segment and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_trailing_slash_root_matches_canonical_absolute_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_trailing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_curdir_segment_trailing_root =
        PathBuf::from(format!("{}/anchor/.././dsl/", root.display()));
    let canonical_root = root.join("dsl");

    let parent_curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_trailing_root)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment-trailing absolute root compile");
    assert!(
        parent_curdir_segment_trailing.status.success(),
        "parent-curdir-segment-trailing absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-trailing and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-trailing and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_curdir_segment_trailing.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_double_separator_root_matches_canonical_absolute_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_double_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_curdir_segment_double_root =
        PathBuf::from(format!("{}/anchor/.././dsl//", root.display()));
    let canonical_root = root.join("dsl");

    let parent_curdir_segment_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_double_root)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment-double absolute root compile");
    assert!(
        parent_curdir_segment_double.status.success(),
        "parent-curdir-segment-double absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_double.stdout, canonical.stdout,
        "parent-curdir-segment-double and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double.stderr, canonical.stderr,
        "parent-curdir-segment-double and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_curdir_segment_double.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_double_separator_trailing_slash_root_matches_canonical_absolute_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_double_trailing_slash_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_curdir_segment_double_separator_trailing_root =
        PathBuf::from(format!("{}/anchor//.././dsl/", root.display()));
    let canonical_root = root.join("dsl");

    let parent_curdir_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_double_separator_trailing_root)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run parent-curdir-segment-double-separator-trailing absolute root compile",
        );
    assert!(
        parent_curdir_segment_double_separator_trailing
            .status
            .success(),
        "parent-curdir-segment-double-separator-trailing absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator-trailing and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator-trailing and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_curdir_segment_double_separator_trailing.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_parent_segment_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_root = root.join("dsl/sample/..");
    let canonical_root = root.join("dsl");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_root)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute root compile");
    assert!(
        parent_segment.status.success(),
        "parent-segment absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_double_separator_root_matches_canonical_absolute_output()
{
    let root = unique_temp_dir("compile_absolute_parent_segment_double_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_double_root =
        PathBuf::from(format!("{}/dsl/sample/..//", root.display()));
    let canonical_root = root.join("dsl");

    let parent_segment_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_double_root)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double absolute root compile");
    assert!(
        parent_segment_double.status.success(),
        "parent-segment-double absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment_double.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_double.stdout, canonical.stdout,
        "parent-segment-double and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_segment_double.stderr, canonical.stderr,
        "parent-segment-double and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment_double.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_double_separator_trailing_slash_root_matches_canonical_absolute_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_segment_double_trailing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_double_trailing_root =
        PathBuf::from(format!("{}/dsl/sample/..//", root.display()));
    let canonical_root = root.join("dsl");

    let parent_segment_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_double_trailing_root)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double-trailing absolute root compile");
    assert!(
        parent_segment_double_trailing.status.success(),
        "parent-segment-double-trailing absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment_double_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_double_trailing.stdout, canonical.stdout,
        "parent-segment-double-trailing and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_segment_double_trailing.stderr, canonical.stderr,
        "parent-segment-double-trailing and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_segment_double_trailing.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_trailing_slash_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_parent_segment_trailing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_trailing_root =
        PathBuf::from(format!("{}/", root.join("dsl/sample/..").display()));
    let canonical_root = root.join("dsl");

    let parent_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_trailing_root)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-trailing absolute root compile");
    assert!(
        parent_segment_trailing.status.success(),
        "parent-segment-trailing absolute root compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_trailing.stdout, canonical.stdout,
        "parent-segment-trailing and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        parent_segment_trailing.stderr, canonical.stderr,
        "parent-segment-trailing and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_double_separator_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_double_separator_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_double_separator_root = PathBuf::from(format!("{}//dsl", root.display()));
    let canonical_root = root.join("dsl");

    let double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_double_separator_root)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute root compile");
    assert!(
        double_separator.status.success(),
        "double-separator absolute root compile should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&double_separator.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_trailing_slash_root_matches_canonical_absolute_output() {
    let root = unique_temp_dir("compile_absolute_trailing_slash_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_trailing_slash_root = PathBuf::from(format!("{}/", root.join("dsl").display()));
    let canonical_root = root.join("dsl");

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_trailing_slash_root)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute root compile");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash absolute root compile should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&trailing_slash.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_double_separator_trailing_slash_root_matches_canonical_absolute_output()
{
    let root = unique_temp_dir("compile_absolute_double_separator_trailing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_double_separator_trailing_root =
        PathBuf::from(format!("{}//dsl/", root.display()));
    let canonical_root = root.join("dsl");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_double_separator_trailing_root)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator trailing absolute root compile");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator trailing absolute root compile should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute root compile");
    assert!(
        canonical.status.success(),
        "canonical absolute root compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator trailing and canonical absolute-root compile stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator trailing and canonical absolute-root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&double_separator_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_mixed_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("compile_absolute_mixed_segment_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_mixed_target = root.join(".").join("dsl/sample/main.dag");
    let canonical_target = root.join("dsl/sample/main.dag");

    let mixed = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_mixed_target)
        .current_dir(&root)
        .output()
        .expect("failed to run mixed-segment absolute single-file compile");
    assert!(
        mixed.status.success(),
        "mixed-segment absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&mixed.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        mixed.stdout, canonical.stdout,
        "mixed-segment and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        mixed.stderr, canonical.stderr,
        "mixed-segment and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&mixed.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_curdir_segment_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_curdir_segment_trailing_target =
        PathBuf::from(format!("{}/./dsl/sample/main.dag/", root.display()));
    let canonical_target = root.join("dsl/sample/main.dag");

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_trailing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing absolute single-file compile");
    assert!(
        curdir_segment_trailing.status.success(),
        "curdir-segment-trailing absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_trailing.stdout, canonical.stdout,
        "curdir-segment-trailing and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        curdir_segment_trailing.stderr, canonical.stderr,
        "curdir-segment-trailing and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&curdir_segment_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_curdir_segment_double_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_target = root.join("dsl/sample/main.dag");
    let absolute_curdir_segment_double_separator_target =
        PathBuf::from(format!("{}//./", canonical_target.display()));

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_double_separator_target)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator absolute single-file compile");
    assert!(
        curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator.stdout, canonical.stdout,
        "curdir-segment-double-separator and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator.stderr, canonical.stderr,
        "curdir-segment-double-separator and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &curdir_segment_double_separator.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_suffix_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("compile_absolute_curdir_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_target = root.join("dsl/sample/main.dag");
    let absolute_curdir_suffix_target = root.join("dsl/sample/main.dag/.");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_suffix_target)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix absolute single-file compile");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_suffix.stdout, canonical.stdout,
        "curdir-suffix and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        curdir_suffix.stderr, canonical.stderr,
        "curdir-suffix and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&curdir_suffix.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_suffix_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_curdir_suffix_double_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_target = root.join("dsl/sample/main.dag");
    let absolute_curdir_suffix_double_separator_target =
        PathBuf::from(format!("{}//.", canonical_target.display()));

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_suffix_double_separator_target)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator absolute single-file compile");
    assert!(
        curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_suffix_double_separator.stdout, canonical.stdout,
        "curdir-suffix-double-separator and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        curdir_suffix_double_separator.stderr, canonical.stderr,
        "curdir-suffix-double-separator and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &curdir_suffix_double_separator.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_double_separator_suffix_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_curdir_segment_double_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_target = root.join("dsl/sample/main.dag");
    let absolute_curdir_segment_double_separator_suffix_target =
        PathBuf::from(format!("{}//./.", canonical_target.display()));

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_double_separator_suffix_target)
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix absolute single-file compile",
        );
    assert!(
        curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_double_separator_suffix.stdout, canonical.stdout,
        "curdir-segment-double-separator-suffix and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        curdir_segment_double_separator_suffix.stderr, canonical.stderr,
        "curdir-segment-double-separator-suffix and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &curdir_segment_double_separator_suffix.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_curdir_segment_suffix_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("compile_absolute_curdir_segment_suffix_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_target = root.join("dsl/sample/main.dag");
    let absolute_curdir_segment_suffix_target = root.join("dsl/sample/main.dag/./.");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_curdir_segment_suffix_target)
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix absolute single-file compile");
    assert!(
        curdir_segment_suffix.status.success(),
        "curdir-segment-suffix absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        curdir_segment_suffix.stdout, canonical.stdout,
        "curdir-segment-suffix and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        curdir_segment_suffix.stderr, canonical.stderr,
        "curdir-segment-suffix and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&curdir_segment_suffix.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_curdir_segment_target = root.join("anchor/.././dsl/sample/main.dag");
    let canonical_target = root.join("dsl/sample/main.dag");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment absolute single-file compile");
    assert!(
        parent_curdir_segment.status.success(),
        "parent-curdir-segment absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment.stdout, canonical.stdout,
        "parent-curdir-segment and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, canonical.stderr,
        "parent-curdir-segment and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_curdir_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_target = root.join("dsl/sample/main.dag");
    let absolute_parent_curdir_segment_trailing_target = PathBuf::from(format!(
        "{}/anchor/.././dsl/sample/main.dag/",
        root.display()
    ));

    let parent_curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_trailing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment-trailing absolute single-file compile");
    assert!(
        parent_curdir_segment_trailing.status.success(),
        "parent-curdir-segment-trailing absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-trailing and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-trailing and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_curdir_segment_trailing.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_double_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_curdir_segment_double_target = PathBuf::from(format!(
        "{}/anchor/.././dsl/sample//main.dag",
        root.display()
    ));
    let canonical_target = root.join("dsl/sample/main.dag");

    let parent_curdir_segment_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_double_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment-double absolute single-file compile");
    assert!(
        parent_curdir_segment_double.status.success(),
        "parent-curdir-segment-double absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_double.stdout, canonical.stdout,
        "parent-curdir-segment-double and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double.stderr, canonical.stderr,
        "parent-curdir-segment-double and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_curdir_segment_double.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_double_separator_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root =
        unique_temp_dir("compile_absolute_parent_curdir_segment_double_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let canonical_target = root.join("dsl/sample/main.dag");
    let absolute_parent_curdir_segment_double_separator_trailing_target = PathBuf::from(format!(
        "{}/anchor//.././dsl/sample/main.dag/",
        root.display()
    ));

    let parent_curdir_segment_double_separator_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_curdir_segment_double_separator_trailing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-curdir-segment-double-separator-trailing absolute single-file compile");
    assert!(
        parent_curdir_segment_double_separator_trailing.status.success(),
        "parent-curdir-segment-double-separator-trailing absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_curdir_segment_double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stdout, canonical.stdout,
        "parent-curdir-segment-double-separator-trailing and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment_double_separator_trailing.stderr, canonical.stderr,
        "parent-curdir-segment-double-separator-trailing and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_curdir_segment_double_separator_trailing.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("compile_absolute_parent_segment_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_target = root.join("dsl/sample/../sample/main.dag");
    let canonical_target = root.join("dsl/sample/main.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment absolute single-file compile");
    assert!(
        parent_segment.status.success(),
        "parent-segment absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment.stdout, canonical.stdout,
        "parent-segment and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, canonical.stderr,
        "parent-segment and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_segment_double_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_double_target =
        PathBuf::from(format!("{}/dsl/sample/..//sample/main.dag", root.display()));
    let canonical_target = root.join("dsl/sample/main.dag");

    let parent_segment_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_double_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double absolute single-file compile");
    assert!(
        parent_segment_double.status.success(),
        "parent-segment-double absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment_double.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_double.stdout, canonical.stdout,
        "parent-segment-double and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_segment_double.stderr, canonical.stderr,
        "parent-segment-double and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment_double.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_double_separator_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_segment_double_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_double_trailing_target = PathBuf::from(format!(
        "{}/dsl/sample/..//sample/main.dag/",
        root.display()
    ));
    let canonical_target = root.join("dsl/sample/main.dag");

    let parent_segment_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_double_trailing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-double-trailing absolute single-file compile");
    assert!(
        parent_segment_double_trailing.status.success(),
        "parent-segment-double-trailing absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment_double_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_double_trailing.stdout, canonical.stdout,
        "parent-segment-double-trailing and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_segment_double_trailing.stderr, canonical.stderr,
        "parent-segment-double-trailing and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(
        &parent_segment_double_trailing.stderr,
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_segment_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_trailing_target = PathBuf::from(format!(
        "{}/",
        root.join("dsl/sample/../sample/main.dag").display()
    ));
    let canonical_target = root.join("dsl/sample/main.dag");

    let parent_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_parent_segment_trailing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run parent-segment-trailing absolute single-file compile");
    assert!(
        parent_segment_trailing.status.success(),
        "parent-segment-trailing absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_segment_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        parent_segment_trailing.stdout, canonical.stdout,
        "parent-segment-trailing and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_segment_trailing.stderr, canonical.stderr,
        "parent-segment-trailing and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_segment_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_double_separator_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("compile_absolute_double_separator_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_double_separator_target =
        PathBuf::from(format!("{}//dsl//sample//main.dag", root.display()));
    let canonical_target = root.join("dsl/sample/main.dag");

    let double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_double_separator_target)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator absolute single-file compile");
    assert!(
        double_separator.status.success(),
        "double-separator absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&double_separator.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator.stdout, canonical.stdout,
        "double-separator and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        double_separator.stderr, canonical.stderr,
        "double-separator and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&double_separator.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_trailing_slash_single_file_target_matches_canonical_output() {
    let root = unique_temp_dir("compile_absolute_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_trailing_target =
        PathBuf::from(format!("{}/", root.join("dsl/sample/main.dag").display()));
    let canonical_target = root.join("dsl/sample/main.dag");

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_trailing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash absolute single-file compile");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        trailing_slash.stdout, canonical.stdout,
        "trailing-slash and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        trailing_slash.stderr, canonical.stderr,
        "trailing-slash and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&trailing_slash.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_double_separator_trailing_slash_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_double_separator_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_double_separator_trailing_target =
        PathBuf::from(format!("{}//dsl//sample//main.dag/", root.display()));
    let canonical_target = root.join("dsl/sample/main.dag");

    let double_separator_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_double_separator_trailing_target)
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator trailing absolute single-file compile");
    assert!(
        double_separator_trailing.status.success(),
        "double-separator trailing absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&double_separator_trailing.stderr)
    );

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_target)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute single-file compile");
    assert!(
        canonical.status.success(),
        "canonical absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&canonical.stderr)
    );

    assert_eq!(
        double_separator_trailing.stdout, canonical.stdout,
        "double-separator trailing and canonical absolute single-file compile stdout should match"
    );
    assert_eq!(
        double_separator_trailing.stderr, canonical.stderr,
        "double-separator trailing and canonical absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&double_separator_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_missing_root_variants_match_canonical_output() {
    let root = unique_temp_dir("compile_absolute_missing_root_variants");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor directory");
    let canonical_missing_root = root.join("missing_root");

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_missing_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing-root compile");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing-root compile should fail"
    );
    let canonical_stderr = String::from_utf8_lossy(&canonical.stderr);
    assert_no_stage_failures(&canonical_stderr);
    assert!(
        canonical_stderr.contains(&canonical_missing_root.display().to_string()),
        "canonical missing-root diagnostic should contain normalized absolute path: {canonical_stderr}"
    );

    let variants = vec![
        ("mixed", root.join(".").join("missing_root")),
        (
            "curdir_segment_trailing",
            PathBuf::from(format!("{}/./missing_root/", root.display())),
        ),
        (
            "curdir_segment_double_separator",
            PathBuf::from(format!("{}//./", canonical_missing_root.display())),
        ),
        ("curdir_suffix", root.join("missing_root/.")),
        (
            "curdir_suffix_double_separator",
            PathBuf::from(format!("{}//.", canonical_missing_root.display())),
        ),
        (
            "curdir_segment_double_separator_suffix",
            PathBuf::from(format!("{}//./.", canonical_missing_root.display())),
        ),
        (
            "curdir_segment_suffix",
            PathBuf::from(format!("{}/./.", canonical_missing_root.display())),
        ),
        ("parent", root.join("anchor/../missing_root")),
        (
            "parent_curdir_segment",
            root.join("anchor/.././missing_root"),
        ),
        (
            "parent_curdir_segment_trailing",
            PathBuf::from(format!("{}/anchor/.././missing_root/", root.display())),
        ),
        (
            "parent_curdir_segment_double_separator",
            PathBuf::from(format!("{}/anchor/.././missing_root//", root.display())),
        ),
        (
            "parent_curdir_segment_double_separator_trailing",
            PathBuf::from(format!("{}/anchor//.././missing_root/", root.display())),
        ),
        (
            "parent_double_separator",
            PathBuf::from(format!("{}/anchor/..//missing_root", root.display())),
        ),
        (
            "parent_trailing",
            PathBuf::from(format!(
                "{}/",
                root.join("anchor/../missing_root").display()
            )),
        ),
        (
            "parent_double_separator_trailing",
            PathBuf::from(format!("{}/anchor/..//missing_root/", root.display())),
        ),
        (
            "double_separator",
            PathBuf::from(format!("{}//missing_root", root.display())),
        ),
        (
            "trailing_slash",
            PathBuf::from(format!("{}/", canonical_missing_root.display())),
        ),
        (
            "double_separator_trailing",
            PathBuf::from(format!("{}//missing_root/", root.display())),
        ),
    ];

    for (label, variant_path) in variants {
        let variant = Command::new(daglang_bin())
            .arg("compile")
            .arg(&variant_path)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|_| {
                panic!("failed to run {label} absolute missing-root compile variant")
            });
        assert!(
            !variant.status.success(),
            "{label} absolute missing-root compile variant should fail"
        );
        assert_eq!(
            variant.stdout, canonical.stdout,
            "{label} missing-root variant stdout should match canonical output"
        );
        assert_eq!(
            variant.stderr, canonical.stderr,
            "{label} missing-root variant stderr should match canonical output"
        );
    }

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_non_directory_root_variants_match_canonical_output() {
    let root = unique_temp_dir("compile_absolute_non_directory_root_variants");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor directory");
    let canonical_non_directory_root = root.join("input.txt");
    std::fs::write(&canonical_non_directory_root, "not a directory")
        .expect("failed to write non-directory root fixture");

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_non_directory_root)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute non-directory-root compile");
    assert!(
        !canonical.status.success(),
        "canonical absolute non-directory-root compile should fail"
    );
    let canonical_stderr = String::from_utf8_lossy(&canonical.stderr);
    assert_no_stage_failures(&canonical_stderr);
    assert!(
        canonical_stderr.contains(&canonical_non_directory_root.display().to_string()),
        "canonical non-directory-root diagnostic should contain normalized absolute path: {canonical_stderr}"
    );

    let variants = vec![
        ("mixed", root.join(".").join("input.txt")),
        (
            "curdir_segment_trailing",
            PathBuf::from(format!("{}/./input.txt/", root.display())),
        ),
        (
            "curdir_segment_double_separator",
            PathBuf::from(format!("{}//./", canonical_non_directory_root.display())),
        ),
        ("curdir_suffix", root.join("input.txt/.")),
        (
            "curdir_suffix_double_separator",
            PathBuf::from(format!("{}//.", canonical_non_directory_root.display())),
        ),
        (
            "curdir_segment_double_separator_suffix",
            PathBuf::from(format!("{}//./.", canonical_non_directory_root.display())),
        ),
        (
            "curdir_segment_suffix",
            PathBuf::from(format!("{}/./.", canonical_non_directory_root.display())),
        ),
        ("parent", root.join("anchor/../input.txt")),
        ("parent_curdir_segment", root.join("anchor/.././input.txt")),
        (
            "parent_curdir_segment_trailing",
            PathBuf::from(format!("{}/anchor/.././input.txt/", root.display())),
        ),
        (
            "parent_curdir_segment_double_separator",
            PathBuf::from(format!("{}/anchor/.././input.txt//", root.display())),
        ),
        (
            "parent_curdir_segment_double_separator_trailing",
            PathBuf::from(format!("{}/anchor//.././input.txt/", root.display())),
        ),
        (
            "parent_double_separator",
            PathBuf::from(format!("{}/anchor/..//input.txt", root.display())),
        ),
        (
            "parent_trailing",
            PathBuf::from(format!("{}/", root.join("anchor/../input.txt").display())),
        ),
        (
            "parent_double_separator_trailing",
            PathBuf::from(format!("{}/anchor/..//input.txt/", root.display())),
        ),
        (
            "double_separator",
            PathBuf::from(format!("{}//input.txt", root.display())),
        ),
        (
            "trailing_slash",
            PathBuf::from(format!("{}/", canonical_non_directory_root.display())),
        ),
        (
            "double_separator_trailing",
            PathBuf::from(format!("{}//input.txt/", root.display())),
        ),
    ];

    for (label, variant_path) in variants {
        let variant = Command::new(daglang_bin())
            .arg("compile")
            .arg(&variant_path)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|_| {
                panic!("failed to run {label} absolute non-directory-root compile variant")
            });
        assert!(
            !variant.status.success(),
            "{label} absolute non-directory-root compile variant should fail"
        );
        assert_eq!(
            variant.stdout, canonical.stdout,
            "{label} non-directory-root variant stdout should match canonical output"
        );
        assert_eq!(
            variant.stderr, canonical.stderr,
            "{label} non-directory-root variant stderr should match canonical output"
        );
    }

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_missing_single_file_variants_match_canonical_output() {
    let root = unique_temp_dir("compile_absolute_missing_single_file_variants");
    std::fs::create_dir_all(root.join("dsl/sample")).expect("failed to create temp root fixture");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor directory");
    let canonical_missing_single_file = root.join("dsl/sample/missing.dag");

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_missing_single_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute missing single-file compile");
    assert!(
        !canonical.status.success(),
        "canonical absolute missing single-file compile should fail"
    );
    let canonical_stderr = String::from_utf8_lossy(&canonical.stderr);
    assert_no_stage_failures(&canonical_stderr);
    assert!(
        canonical_stderr.contains(&canonical_missing_single_file.display().to_string()),
        "canonical missing single-file diagnostic should contain normalized absolute path: {canonical_stderr}"
    );

    let variants = vec![
        ("mixed", root.join(".").join("dsl/sample/missing.dag")),
        (
            "curdir_segment_trailing",
            PathBuf::from(format!("{}/./dsl/sample/missing.dag/", root.display())),
        ),
        (
            "curdir_segment_double_separator",
            PathBuf::from(format!("{}//./", canonical_missing_single_file.display())),
        ),
        ("curdir_suffix", root.join("dsl/sample/missing.dag/.")),
        (
            "curdir_suffix_double_separator",
            PathBuf::from(format!("{}//.", canonical_missing_single_file.display())),
        ),
        (
            "curdir_segment_double_separator_suffix",
            PathBuf::from(format!("{}//./.", canonical_missing_single_file.display())),
        ),
        (
            "curdir_segment_suffix",
            PathBuf::from(format!("{}/./.", canonical_missing_single_file.display())),
        ),
        ("parent", root.join("dsl/sample/../sample/missing.dag")),
        (
            "parent_curdir_segment",
            root.join("dsl/sample/.././sample/missing.dag"),
        ),
        (
            "parent_curdir_segment_trailing",
            PathBuf::from(format!(
                "{}/dsl/sample/.././sample/missing.dag/",
                root.display()
            )),
        ),
        (
            "parent_curdir_segment_double_separator",
            PathBuf::from(format!(
                "{}/dsl/sample/.././sample//missing.dag",
                root.display()
            )),
        ),
        (
            "parent_curdir_segment_double_separator_trailing",
            PathBuf::from(format!(
                "{}/dsl/sample//.././sample/missing.dag/",
                root.display()
            )),
        ),
        (
            "parent_double_separator",
            PathBuf::from(format!(
                "{}/dsl/sample/..//sample/missing.dag",
                root.display()
            )),
        ),
        (
            "parent_trailing",
            PathBuf::from(format!(
                "{}/",
                root.join("dsl/sample/../sample/missing.dag").display()
            )),
        ),
        (
            "parent_double_separator_trailing",
            PathBuf::from(format!(
                "{}/dsl/sample/..//sample/missing.dag/",
                root.display()
            )),
        ),
        (
            "double_separator",
            PathBuf::from(format!("{}//dsl//sample//missing.dag", root.display())),
        ),
        (
            "trailing_slash",
            PathBuf::from(format!("{}/", canonical_missing_single_file.display())),
        ),
        (
            "double_separator_trailing",
            PathBuf::from(format!("{}//dsl//sample//missing.dag/", root.display())),
        ),
    ];

    for (label, variant_path) in variants {
        let variant = Command::new(daglang_bin())
            .arg("compile")
            .arg(&variant_path)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|_| {
                panic!("failed to run {label} absolute missing single-file compile variant")
            });
        assert!(
            !variant.status.success(),
            "{label} absolute missing single-file compile variant should fail"
        );
        assert_eq!(
            variant.stdout, canonical.stdout,
            "{label} missing single-file variant stdout should match canonical output"
        );
        assert_eq!(
            variant.stderr, canonical.stderr,
            "{label} missing single-file variant stderr should match canonical output"
        );
    }

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_invalid_single_file_variants_match_canonical_output() {
    let root = unique_temp_dir("compile_absolute_invalid_single_file_variants");
    std::fs::create_dir_all(root.join("anchor")).expect("failed to create anchor directory");
    let canonical_invalid_single_file = root.join("dsl/sample/invalid.dag");
    std::fs::create_dir_all(&canonical_invalid_single_file)
        .expect("failed to create invalid single-file target fixture directory");

    let canonical = Command::new(daglang_bin())
        .arg("compile")
        .arg(&canonical_invalid_single_file)
        .current_dir(&root)
        .output()
        .expect("failed to run canonical absolute invalid single-file compile");
    assert!(
        !canonical.status.success(),
        "canonical absolute invalid single-file compile should fail"
    );
    let canonical_stderr = String::from_utf8_lossy(&canonical.stderr);
    assert_no_stage_failures(&canonical_stderr);
    assert!(
        canonical_stderr.contains(&canonical_invalid_single_file.display().to_string()),
        "canonical invalid single-file diagnostic should contain normalized absolute path: {canonical_stderr}"
    );

    let variants = vec![
        ("mixed", root.join(".").join("dsl/sample/invalid.dag")),
        (
            "curdir_segment_trailing",
            PathBuf::from(format!("{}/./dsl/sample/invalid.dag/", root.display())),
        ),
        (
            "curdir_segment_double_separator",
            PathBuf::from(format!("{}//./", canonical_invalid_single_file.display())),
        ),
        ("curdir_suffix", root.join("dsl/sample/invalid.dag/.")),
        (
            "curdir_suffix_double_separator",
            PathBuf::from(format!("{}//.", canonical_invalid_single_file.display())),
        ),
        (
            "curdir_segment_double_separator_suffix",
            PathBuf::from(format!("{}//./.", canonical_invalid_single_file.display())),
        ),
        (
            "curdir_segment_suffix",
            PathBuf::from(format!("{}/./.", canonical_invalid_single_file.display())),
        ),
        ("parent", root.join("anchor/../dsl/sample/invalid.dag")),
        (
            "parent_curdir_segment",
            root.join("anchor/.././dsl/sample/invalid.dag"),
        ),
        (
            "parent_curdir_segment_trailing",
            PathBuf::from(format!(
                "{}/anchor/.././dsl/sample/invalid.dag/",
                root.display()
            )),
        ),
        (
            "parent_curdir_segment_double_separator",
            PathBuf::from(format!(
                "{}/anchor/.././dsl/sample//invalid.dag",
                root.display()
            )),
        ),
        (
            "parent_curdir_segment_double_separator_trailing",
            PathBuf::from(format!(
                "{}/anchor//.././dsl/sample/invalid.dag/",
                root.display()
            )),
        ),
        (
            "parent_double_separator",
            PathBuf::from(format!(
                "{}/anchor/..//dsl/sample/invalid.dag",
                root.display()
            )),
        ),
        (
            "parent_trailing",
            PathBuf::from(format!(
                "{}/",
                root.join("anchor/../dsl/sample/invalid.dag").display()
            )),
        ),
        (
            "parent_double_separator_trailing",
            PathBuf::from(format!(
                "{}/anchor/..//dsl/sample/invalid.dag/",
                root.display()
            )),
        ),
        (
            "double_separator",
            PathBuf::from(format!("{}//dsl//sample//invalid.dag", root.display())),
        ),
        (
            "trailing_slash",
            PathBuf::from(format!("{}/", canonical_invalid_single_file.display())),
        ),
        (
            "double_separator_trailing",
            PathBuf::from(format!("{}//dsl//sample//invalid.dag/", root.display())),
        ),
    ];

    for (label, variant_path) in variants {
        let variant = Command::new(daglang_bin())
            .arg("compile")
            .arg(&variant_path)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|_| {
                panic!("failed to run {label} absolute invalid single-file compile variant")
            });
        assert!(
            !variant.status.success(),
            "{label} absolute invalid single-file compile variant should fail"
        );
        assert_eq!(
            variant.stdout, canonical.stdout,
            "{label} invalid single-file variant stdout should match canonical output"
        );
        assert_eq!(
            variant.stderr, canonical.stderr,
            "{label} invalid single-file variant stderr should match canonical output"
        );
    }

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative root compile");
    assert!(
        plain.status.success(),
        "plain-relative root compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir = Command::new(daglang_bin())
        .arg("compile")
        .arg("./dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment root compile");
    assert!(
        curdir.status.success(),
        "curdir-segment root compile should succeed: {}",
        String::from_utf8_lossy(&curdir.stderr)
    );

    assert_eq!(
        plain.stdout, curdir.stdout,
        "plain-relative and curdir-segment root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir.stderr,
        "plain-relative and curdir-segment root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_suffix_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative root compile");
    assert!(
        plain.status.success(),
        "plain-relative root compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./dsl/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix root compile");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain-relative and curdir-suffix root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain-relative and curdir-suffix root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative root compile");
    assert!(
        plain.status.success(),
        "plain-relative root compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let dot_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator root compile");
    assert!(
        dot_double_separator.status.success(),
        "dot-double-separator root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator.stdout,
        "plain-relative and dot-double-separator root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator.stderr,
        "plain-relative and dot-double-separator root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_single_file_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain single-file compile");
    assert!(
        plain.status.success(),
        "plain single-file compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let dot_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator single-file compile");
    assert!(
        dot_double_separator.status.success(),
        "dot-double-separator single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator.stdout,
        "plain and dot-double-separator single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator.stderr,
        "plain and dot-double-separator single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_double_separator_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_double_separator_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative root compile");
    assert!(
        plain.status.success(),
        "plain-relative root compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-double root compile");
    assert!(
        curdir_double.status.success(),
        "curdir-double root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_double.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_double.stdout,
        "plain-relative and curdir-double root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_double.stderr,
        "plain-relative and curdir-double root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_trailing_slash_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_trailing_slash_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run plain-relative root compile");
    assert!(
        plain.status.success(),
        "plain-relative root compile should succeed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash root compile");
    assert!(
        trailing_slash.status.success(),
        "trailing-slash root compile should succeed: {}",
        String::from_utf8_lossy(&trailing_slash.stderr)
    );

    assert_eq!(
        plain.stdout, trailing_slash.stdout,
        "plain-relative and trailing-slash root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing_slash.stderr,
        "plain-relative and trailing-slash root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_definition() {
    let fixture = unique_temp_file("compile_single_file_duplicate_definition");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run() -> String { "a" }
fn run() -> String { "b" }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-definition fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `run` in module `sample.single`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_callable_reports_ambiguous_call_target() {
    let fixture = unique_temp_file("compile_single_file_duplicate_callable_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-callable fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate callable definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `helper` in module `sample.single`"));
    assert!(
        stderr.contains("ambiguous call target `helper`"),
        "single-file strict mode should report ambiguous call-target diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-callable path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_service_reports_ambiguous_service_call() {
    let fixture = unique_temp_file("compile_single_file_duplicate_service_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-service fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate service definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `FsStorage` in module `sample.single`"));
    assert!(
        stderr.contains("ambiguous service call `FsStorage.read`"),
        "single-file strict mode should report ambiguous service-call diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-service path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_parameter() {
    let fixture = unique_temp_file("compile_single_file_duplicate_parameter");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run(a: String, a: Int) -> String { a }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-parameter fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate parameter"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate parameter `a` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_output_field() {
    let fixture = unique_temp_file("compile_single_file_duplicate_output_field");
    std::fs::write(
        &fixture,
        r#"module sample.single
func run() -> { ok: Bool, ok: String } { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-output fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate output field"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate output field `ok` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_uses_binding() {
    let fixture = unique_temp_file("compile_single_file_duplicate_uses_binding");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-uses fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate uses binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate uses binding `fs` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_resource_uses_reports_ambiguous_used_type() {
    let fixture = unique_temp_file("compile_single_file_duplicate_resource_uses_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-resource-uses fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate resource definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `SharedResource` in module `sample.single`"));
    assert!(
        stderr.contains("ambiguous used resource type `SharedResource`"),
        "single-file strict mode should report ambiguous used resource diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-resource uses path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_duplicate_provides_binding() {
    let fixture = unique_temp_file("compile_single_file_duplicate_provides_binding");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } provides out: Storage provides out: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-provides fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate provides binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate provides binding `out` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_resource_provides_reports_ambiguous_provided_type() {
    let fixture = unique_temp_file("compile_single_file_duplicate_resource_provides_relaxed");
    std::fs::write(
        &fixture,
        r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-resource-provides fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on duplicate resource definition"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `SharedResource` in module `sample.single`"));
    assert!(
        stderr.contains("ambiguous provided resource type `SharedResource`"),
        "single-file strict mode should report ambiguous provided resource diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-resource provides path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_interface_uses_without_provider_hint_reports_lower_ambiguity() {
    let fixture = unique_temp_file("compile_single_file_interface_uses_lower_ambiguity");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource GcsBucket implements ObjectStorage {
  provider: Gcp
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses store: ObjectStorage {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for interface-uses ambiguity fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on provider-ambiguous interface uses"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_lower_stage_failure(&stderr);
    assert!(
        stderr.contains("ambiguous used resource `store: ObjectStorage`"),
        "single-file compile should surface lower-stage resource ambiguity details: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_interface_provides_without_hint_reports_lower_ambiguity() {
    let fixture = unique_temp_file("compile_single_file_interface_provides_lower_ambiguity");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource LocalStore implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource BackupStore implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for interface-provides ambiguity fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on ambiguous interface provides"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_lower_stage_failure(&stderr);
    assert!(
        stderr.contains("ambiguous provided resource `out: Storage`"),
        "single-file compile should surface lower-stage provides ambiguity details: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_use_provide_binding_conflict() {
    let fixture = unique_temp_file("compile_single_file_use_provide_binding_conflict");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage { capability read { input { path: String } output { body: String } } }
func run() -> { ok: Bool } uses io: Storage provides io: Storage { return { ok: true } }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for use/provide-conflict fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on use/provide binding conflict"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("binding `io` is declared in both uses/provides in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_missing_resource_capability() {
    let fixture = unique_temp_file("compile_single_file_missing_resource_capability");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for missing-resource-capability fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on missing resource capability"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(
        stderr.contains("resource `Disk` is missing capability `write` for interface `Storage`")
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_missing_service_operation() {
    let fixture = unique_temp_file("compile_single_file_missing_service_operation");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for missing-service-operation fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on missing service operation"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(
        stderr.contains("service `FsStorage` is missing operation `write` for interface `Storage`")
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_interface_signature_mismatch() {
    let fixture = unique_temp_file("compile_single_file_interface_signature_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for service-signature-mismatch fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on service signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`FsStorage` does not match `Storage.read` contract"));
    assert!(stderr.contains("expected `String` but found `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_fails_on_resource_interface_signature_mismatch() {
    let fixture = unique_temp_file("compile_single_file_resource_signature_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for resource-signature-mismatch fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on resource signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` does not match `Storage.read` contract"));
    assert!(stderr.contains("expected `String` but found `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_service_call_reports_typecheck_error() {
    let fixture = unique_temp_file("compile_unresolved_service_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved service call fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for unresolved service call"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved service call"));
    assert!(stderr.contains("MissingStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_accepts_resource_bound_capability_calls() {
    let fixture = unique_temp_file("compile_resource_bound_capability_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.resources
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for resource-bound capability fixture");

    assert!(
        output.status.success(),
        "single-file compile should accept resource-bound capability calls: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_directory_mode_accepts_resource_bound_capability_calls() {
    let root = unique_temp_dir("resource_bound_capability_directory_mode");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept resource-bound capability calls: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_single_file_unresolved_uses_reports_typecheck_error() {
    let fixture = unique_temp_file("compile_unresolved_uses_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.uses
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved uses fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for unresolved uses binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown used resource type"));
    assert!(stderr.contains("MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_provides_reports_typecheck_error() {
    let fixture = unique_temp_file("compile_unresolved_provides_single_file");
    std::fs::write(
        &fixture,
        r#"module sample.provides
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved provides fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for unresolved provides binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown provided resource type"));
    assert!(stderr.contains("MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_accepts_uses_resource_with_runtime_config_suffix() {
    let fixture = unique_temp_file("compile_single_file_uses_config_suffix");
    std::fs::write(
        &fixture,
        r#"module sample.uses
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for configured uses fixture");

    assert!(
        output.status.success(),
        "single-file compile should accept configured uses resource type: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_accepts_provides_resource_with_runtime_config_suffix() {
    let fixture = unique_temp_file("compile_single_file_provides_config_suffix");
    std::fs::write(
        &fixture,
        r#"module sample.provides
resource ArtifactStore {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for configured provides fixture");

    assert!(
        output.status.success(),
        "single-file compile should accept configured provides resource type: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_imports_fail_in_typecheck_stage() {
    let fixture = unique_temp_file("compile_single_file_unresolved_import");
    std::fs::write(
        &fixture,
        r#"module sample.single
import missing.dep
fn run() -> Unit {}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved import fixture");

    assert!(!output.status.success(), "single-file compile should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved import `missing.dep`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_allows_unit_return_without_tail_expression() {
    let fixture = unique_temp_file("compile_single_file_unit_missing_tail");
    std::fs::write(
        &fixture,
        r#"module sample.single
fn run() -> Unit { let x = 42 }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unit-missing-tail fixture");

    assert!(
        output.status.success(),
        "single-file compile should allow missing tail for Unit return: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "Unit-return success path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "Unit-return success path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_call_targets_fail_in_typecheck_stage() {
    let fixture = unique_temp_file("compile_single_file_unresolved_call_target");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn run() -> String { missing(value: "ok") }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved call-target fixture");

    assert!(!output.status.success(), "single-file compile should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved call target `missing` in `run`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_service_interface_reports_typecheck_error() {
    let fixture = unique_temp_file("compile_single_file_unresolved_service_interface");
    std::fs::write(
        &fixture,
        r#"module sample.services
service FsStorage implements MissingStorage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved service-interface fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on unresolved service interface"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`FsStorage` references unresolved interface `MissingStorage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_resource_interface_reports_typecheck_error() {
    let fixture = unique_temp_file("compile_single_file_unresolved_resource_interface");
    std::fs::write(
        &fixture,
        r#"module sample.resources
resource Disk implements MissingStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for unresolved resource-interface fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail on unresolved resource interface"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` references unresolved interface `MissingStorage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_interface_reports_ambiguous_implements() {
    let fixture = unique_temp_file("compile_single_file_duplicate_interface_ambiguous_impl");
    std::fs::write(
        &fixture,
        r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for duplicate-interface fixture");

    assert!(
        !output.status.success(),
        "single-file compile should fail for duplicate interface definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `Storage` in module `sample.single`"));
    assert!(stderr.contains("`FsStorage` references ambiguous interface `Storage`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_shows_lowered_nodes_and_edges() {
    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand");

    assert!(
        output.status.success(),
        "expand command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Nodes:"));
    assert!(stdout.contains("tools.makegen::render_makefile"));
    assert!(stdout.contains("tools.makegen::makegen"));
}

#[test]
fn expand_command_makegen_output_matches_snapshot() {
    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand");

    assert!(
        output.status.success(),
        "expand command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected_makegen_expand_snapshot());
}

#[test]
fn progress_command_shows_derived_progress_metrics() {
    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress");

    assert!(
        output.status.success(),
        "progress command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Progress:"));
    assert!(stdout.contains("total_nodes:"));
    assert!(stdout.contains("waves:"));
    assert!(stdout.contains("TestObligations:"));
    assert!(stdout.contains("service_transport_prepare_targets:"));
    assert!(stdout.contains("service_param_source_targets:"));
    assert!(stdout.contains("resource_provide_targets:"));
}

#[test]
fn topology_command_shows_graph_topology() {
    let output = Command::new(daglang_bin())
        .arg("topology")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang topology");

    assert!(
        output.status.success(),
        "topology command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Topology:"));
    assert!(stdout.contains("nodes:"));
    assert!(stdout.contains("labels:"));
    assert!(stdout.contains("subdag_boundaries:"));
}

#[test]
fn topology_command_json_format_emits_valid_json_object() {
    let output = Command::new(daglang_bin())
        .arg("topology")
        .arg(makegen_file())
        .args(["--format", "json"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang topology --format json");

    assert!(
        output.status.success(),
        "topology --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("topology --format json should emit JSON");
    assert!(
        parsed.get("topology").is_some(),
        "JSON should have topology key"
    );
    assert!(
        parsed.get("labels").is_some(),
        "JSON should have labels key"
    );
    assert!(
        parsed.get("subdag_boundaries").is_some(),
        "JSON should have subdag_boundaries key"
    );
}

// DELETED: run_command_real_mode_writes_output_file
// DELETED: run_command_dry_run_does_not_write_output_file
// DELETED: run_command_real_mode_reports_not_written_when_output_is_fresh
// Blocked on: content_upsert data-flow wiring gap (map receiver is Skipped).
// Restore when content_upsert wiring is fixed.

#[test]
fn run_command_rejects_duplicate_output_flags_with_usage() {
    let output = Command::new(daglang_bin())
        .arg("run")
        .arg("--output")
        .arg("a.mk")
        .arg("--output=b.mk")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang run with duplicate output flags");

    assert!(
        !output.status.success(),
        "run with duplicate output flags should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "duplicate output flags should use usage exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("duplicate --output"));
    assert!(stderr.contains(
        "Usage: daglang run [--output <path>|--output=<path>] [--dry-run|--check-mode] <file.dag>"
    ));
}

// DELETED: run_command_real_mode_skips_write_when_content_is_fresh
// DELETED: run_command_check_mode_succeeds_when_output_is_fresh
// DELETED: run_command_check_mode_fails_when_output_is_stale_without_overwrite
// Blocked on: content_upsert data-flow wiring gap (map receiver is Skipped).
// Restore when content_upsert wiring is fixed.

#[test]
fn run_command_rejects_conflicting_dry_run_and_check_mode_flags() {
    let output = Command::new(daglang_bin())
        .arg("run")
        .arg("--dry-run")
        .arg("--check-mode")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run conflicting run modes");

    assert!(
        !output.status.success(),
        "run should fail when dry-run and check-mode are combined"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "conflicting run modes should use usage exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot be combined"));
    assert!(stderr.contains(
        "Usage: daglang run [--output <path>|--output=<path>] [--dry-run|--check-mode] <file.dag>"
    ));
}

#[test]
fn progress_command_json_format_emits_valid_json_object() {
    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress --format json");

    assert!(
        output.status.success(),
        "progress --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value =
        serde_json::from_slice(&output.stdout).expect("progress --format json should emit JSON");
    assert!(parsed.get("total_nodes").is_some());
    assert!(parsed.get("total_edges").is_some());
    assert!(parsed.get("waves").is_some());
    assert!(parsed.get("parallel_groups").is_some());
    assert!(parsed.get("capture_modes").is_some());
    assert!(parsed.get("resources").is_some());
    let obligations = parsed
        .get("test_obligations")
        .expect("progress json should include test_obligations object");
    assert!(obligations.get("transport_execution_targets").is_some());
    assert!(obligations.get("resource_release_targets").is_some());
}

#[test]
fn progress_command_json_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang progress --format json");
    assert!(
        first.status.success(),
        "first progress json run failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang progress --format json");
    assert!(
        second.status.success(),
        "second progress json run failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    assert_eq!(
        first.stdout, second.stdout,
        "progress json output should be byte-stable across repeated runs"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "progress json stderr should be stable across repeated runs"
    );
}

#[test]
fn progress_command_json_contains_expected_progress_keys() {
    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress --format json");

    assert!(
        output.status.success(),
        "progress --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("progress json should be valid");
    assert!(
        parsed.get("total_nodes").is_some(),
        "should have total_nodes"
    );
    assert!(
        parsed.get("total_edges").is_some(),
        "should have total_edges"
    );
    assert!(parsed.get("waves").is_some(), "should have waves");
    assert!(
        parsed.get("parallel_groups").is_some(),
        "should have parallel_groups"
    );
    assert!(
        parsed.get("stage_groups").is_some(),
        "should have stage_groups"
    );
    assert!(
        parsed.get("test_obligations").is_some(),
        "should have test_obligations"
    );
    assert!(
        parsed.get("topology").is_none(),
        "progress should not include topology"
    );
    assert!(
        parsed.get("labels").is_none(),
        "progress should not include labels"
    );
}

#[test]
fn progress_command_ci_json_includes_stage_groups() {
    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(ci_pipeline_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress for CI pipeline");

    assert!(
        output.status.success(),
        "progress --format json for ci failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value =
        serde_json::from_slice(&output.stdout).expect("progress --format json should emit JSON");
    let stage_groups = parsed
        .get("stage_groups")
        .and_then(Value::as_array)
        .expect("progress json should include stage_groups array");
    assert!(
        !stage_groups.is_empty(),
        "ci pipeline progress should emit non-empty stage_groups"
    );
    assert!(
        stage_groups.iter().any(|group| {
            group
                .get("stage_id")
                .and_then(Value::as_str)
                .is_some_and(|stage| stage.ends_with(":cloud_env"))
        }),
        "ci progress should include cloud_env stage group"
    );
    assert!(
        stage_groups.iter().any(|group| {
            group
                .get("stage_id")
                .and_then(Value::as_str)
                .is_some_and(|stage| stage.ends_with(":bootstrap_stage"))
        }),
        "ci progress should include explicit bootstrap_stage group"
    );
}

#[test]
fn progress_command_ci_text_renders_collapsible_stage_group_sections() {
    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(ci_pipeline_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress for ci pipeline");

    assert!(
        output.status.success(),
        "progress command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("> [collapsed] pipelines.ci.ci"));
    assert!(stdout.contains("- cloud_env:"));
    assert!(stdout.contains("- bootstrap_stage:"));
}

#[test]
fn progress_command_collection_nodes_renders_scatter_counters() {
    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(deps_file())
        .arg("--emit-collection-nodes")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress with collection nodes");

    assert!(
        output.status.success(),
        "progress command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scatter_points:"));
    assert!(stdout.contains("[0/"));
    assert!(stdout.contains("tools.deps.render_deps_toml"));
}

#[test]
fn progress_command_explicit_text_format_matches_default_output() {
    let default_output = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress default format");
    assert!(
        default_output.status.success(),
        "default progress command failed: {}",
        String::from_utf8_lossy(&default_output.stderr)
    );

    let explicit_text_output = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .arg("--format")
        .arg("text")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress --format text");
    assert!(
        explicit_text_output.status.success(),
        "progress --format text failed: {}",
        String::from_utf8_lossy(&explicit_text_output.stderr)
    );

    assert_eq!(
        default_output.stdout, explicit_text_output.stdout,
        "explicit text format should match default progress output"
    );
}

#[test]
fn progress_command_rejects_unknown_format_flag_value() {
    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .arg("--format")
        .arg("yaml")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress --format yaml");

    assert!(
        !output.status.success(),
        "progress should fail for unsupported format value"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "unsupported progress format should use usage exit code 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "Usage: daglang progress <file.dag> [--format text|json] [--emit-collection-nodes]"
        ),
        "progress unsupported format should print command usage: {stderr}"
    );
}

#[test]
fn progress_command_reports_non_zero_transport_and_lifecycle_obligations() {
    let fixture = unique_temp_file("progress_obligations");
    std::fs::write(
        &fixture,
        r#"module sample.obligations
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
resource TempFile {
  acquire {
    let path = "/tmp/file"
  }
  release {
    let done = true
  }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress on fixture");

    assert!(
        output.status.success(),
        "progress command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("service_transport_prepare_targets: 1"));
    assert!(stdout.contains("service_transport_execute_targets: 1"));
    assert!(stdout.contains("service_transport_parse_targets: 1"));
    assert!(stdout.contains("service_param_source_targets: 1"));
    assert!(stdout.contains("resource_acquire_targets: 1"));
    assert!(stdout.contains("resource_release_targets: 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn progress_command_reports_zero_service_param_source_targets_for_literal_args() {
    let fixture = unique_temp_file("progress_param_sources_zero");
    std::fs::write(
        &fixture,
        r#"module sample.literal
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(path: "README.md")
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress on fixture");

    assert!(
        output.status.success(),
        "progress command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Literal args (e.g., path: "README.md") now lower to call_literal_source
    // nodes with ServiceParamSource obligation, so the count is 1.
    assert!(stdout.contains("service_param_source_targets: 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn progress_command_interface_only_provides_has_no_release_obligation() {
    let fixture = unique_temp_file("progress_interface_provides");
    std::fs::write(
        &fixture,
        r#"module sample.provides
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress on fixture");

    assert!(
        output.status.success(),
        "progress command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("resource_provide_targets: 1"));
    assert!(stdout.contains("resource_release_targets: 0"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn obligations_command_shows_derived_obligation_summary() {
    let output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations");

    assert!(
        output.status.success(),
        "obligations command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TestObligations:"));
    assert!(stdout.contains("transport_execution_targets:"));
    assert!(stdout.contains("resource_provide_targets:"));
}

#[test]
fn obligations_command_json_format_emits_valid_json_object() {
    let output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations --format json");

    assert!(
        output.status.success(),
        "obligations --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .expect("obligations --format json should emit valid JSON");
    assert!(parsed.get("dry_run_completion_required").is_some());
    assert!(parsed.get("service_transport_prepare_targets").is_some());
}

#[test]
fn obligations_command_json_full_dsl_root_fails_on_ambiguous_resource_bindings() {
    let output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(dsl_root_dir())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations on full dsl root");

    assert!(
        !output.status.success(),
        "obligations dsl --format json should fail on ambiguous full-corpus bindings: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("lower error"),
        "full dsl obligations should fail in lower stage: {stderr}"
    );
    assert!(
        stderr.contains("compile with --profile") || stderr.contains("ambiguous"),
        "full dsl obligations should report missing profile binding or ambiguous resource: {stderr}"
    );
}

#[test]
fn obligations_command_aws_resources_reports_contract_targets() {
    let output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(aws_resources_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations for aws resources");

    assert!(
        output.status.success(),
        "obligations aws resources --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .expect("obligations aws resources should emit valid JSON");
    let interface_contracts = parsed
        .get("interface_contract_verification_targets")
        .and_then(Value::as_u64)
        .expect("expected interface_contract_verification_targets count");
    let resource_acquire = parsed
        .get("resource_acquire_targets")
        .and_then(Value::as_u64)
        .expect("expected resource_acquire_targets count");

    assert!(
        interface_contracts > 0,
        "aws resources obligations should include interface contract targets"
    );
    assert!(
        resource_acquire > 0,
        "aws resources obligations should include lifecycle acquire targets"
    );
}

#[test]
fn obligations_command_azure_resources_reports_contract_targets() {
    let output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(azure_resources_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations for azure resources");

    assert!(
        output.status.success(),
        "obligations azure resources --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .expect("obligations azure resources should emit valid JSON");
    let interface_contracts = parsed
        .get("interface_contract_verification_targets")
        .and_then(Value::as_u64)
        .expect("expected interface_contract_verification_targets count");
    let resource_acquire = parsed
        .get("resource_acquire_targets")
        .and_then(Value::as_u64)
        .expect("expected resource_acquire_targets count");

    assert!(
        interface_contracts > 0,
        "azure resources obligations should include interface contract targets"
    );
    assert!(
        resource_acquire > 0,
        "azure resources obligations should include lifecycle acquire targets"
    );
}

#[test]
fn obligations_command_explicit_text_format_matches_default_output() {
    let default_output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations default format");
    assert!(
        default_output.status.success(),
        "default obligations command failed: {}",
        String::from_utf8_lossy(&default_output.stderr)
    );

    let explicit_text_output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .arg("--format")
        .arg("text")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations --format text");
    assert!(
        explicit_text_output.status.success(),
        "obligations --format text failed: {}",
        String::from_utf8_lossy(&explicit_text_output.stderr)
    );

    assert_eq!(
        default_output.stdout, explicit_text_output.stdout,
        "explicit text format should match default obligations output"
    );
}

#[test]
fn obligations_command_reports_transport_free_graph_counts() {
    let fixture = unique_temp_file("obligations_transport_free");
    std::fs::write(
        &fixture,
        r#"module sample.obligations
fn run() -> Unit { }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations");

    assert!(
        output.status.success(),
        "obligations command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("transport_execution_targets: 0"));
    assert!(stdout.contains("pure_node_determinism_targets: 1"));
    assert!(stdout.contains("service_transport_prepare_targets: 0"));
    assert!(stdout.contains("resource_acquire_targets: 0"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn obligations_command_json_reports_transport_free_graph_counts() {
    let fixture = unique_temp_file("obligations_transport_free_json");
    std::fs::write(
        &fixture,
        r#"module sample.obligations
fn run() -> Unit { }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(&fixture)
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations --format json");

    assert!(
        output.status.success(),
        "obligations --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .expect("obligations --format json should emit valid JSON");
    assert_eq!(
        parsed
            .get("transport_execution_targets")
            .and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        parsed
            .get("pure_node_determinism_targets")
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        parsed
            .get("service_transport_prepare_targets")
            .and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        parsed
            .get("resource_acquire_targets")
            .and_then(Value::as_u64),
        Some(0)
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn progress_and_obligations_commands_share_obligation_text_output() {
    let progress_output = Command::new(daglang_bin())
        .arg("progress")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress");
    assert!(
        progress_output.status.success(),
        "progress command failed: {}",
        String::from_utf8_lossy(&progress_output.stderr)
    );

    let obligations_output = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang obligations");
    assert!(
        obligations_output.status.success(),
        "obligations command failed: {}",
        String::from_utf8_lossy(&obligations_output.stderr)
    );

    let progress_stdout = String::from_utf8_lossy(&progress_output.stdout);
    let obligations_stdout = String::from_utf8_lossy(&obligations_output.stdout);
    assert_eq!(
        obligations_block(&progress_stdout),
        obligations_stdout,
        "progress obligation section should match standalone obligations output"
    );
}

#[test]
fn obligations_command_json_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang obligations --format json");
    assert!(
        first.status.success(),
        "first obligations json command failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang obligations --format json");
    assert!(
        second.status.success(),
        "second obligations json command failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    assert_eq!(
        first.stdout, second.stdout,
        "obligations json output should be deterministic"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "obligations json stderr should be deterministic"
    );
}

#[test]
fn report_coverage_command_succeeds_when_all_stages_are_referenced() {
    let fixture = unique_temp_file("report_coverage_ok");
    std::fs::write(
        &fixture,
        r#"module sample.report_coverage_ok
fn report_entry(name: String, success: Bool) -> Bool { success }
pipeline ci {
  stage codegen { codegen_ok = true }
  stage test [after codegen] { test_ok = true }
  stage report [after test] {
    entries = [
      report_entry(name: "codegen", success: codegen_ok),
      report_entry(name: "test", success: test_ok)
    ]
  }
}
"#,
    )
    .expect("failed to write report coverage fixture");

    let output = Command::new(daglang_bin())
        .arg("report-coverage")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang report-coverage");
    assert!(
        output.status.success(),
        "report-coverage should succeed when coverage is complete: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: report coverage complete"),
        "report-coverage should report success: {stdout}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn report_coverage_command_fails_when_stage_is_missing() {
    let fixture = unique_temp_file("report_coverage_missing");
    std::fs::write(
        &fixture,
        r#"module sample.report_coverage_missing
fn report_entry(name: String, success: Bool) -> Bool { success }
pipeline ci {
  stage codegen { codegen_ok = true }
  stage test [after codegen] { test_ok = true }
  stage report [after test] {
    entries = [report_entry(name: "codegen", success: codegen_ok)]
  }
}
"#,
    )
    .expect("failed to write report coverage fixture");

    let output = Command::new(daglang_bin())
        .arg("report-coverage")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang report-coverage");
    assert!(
        !output.status.success(),
        "report-coverage should fail when coverage is incomplete"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "report-coverage should use exit code 2 on coverage lint failures"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("missing report coverage for stages: test"),
        "report-coverage should report missing stage names: {stdout}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn obligations_command_relative_directory_alias_normalized_spelling_fail_outputs_match_canonical() {
    let root = unique_temp_dir(
        "obligations_relative_directory_alias_normalized_spelling_failure_output_parity",
    );
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(root.join("nested")).expect("failed to create nested directory");
    std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .dag directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .dag directory");

    assert_single_target_command_failure_outputs_match_for_variants(
        "obligations",
        &root,
        "bundle.dag",
        &[
            ("./bundle.dag", "curdir-suffix alias"),
            ("nested/../bundle.dag", "parent-segment alias"),
            ("nested/./../bundle.dag", "parent-curdir alias"),
            ("nested/..//bundle.dag", "parent-double-separator alias"),
        ],
        &[],
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn obligations_command_json_relative_symlink_alias_normalized_spelling_fail_outputs_match_canonical(
) {
    let root = unique_temp_dir(
        "obligations_json_relative_symlink_alias_normalized_spelling_failure_output_parity",
    );
    let real_dir = root.join("real");
    std::fs::create_dir_all(root.join("nested")).expect("failed to create nested directory");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    let symlink_path = root.join("bundle_link.DAG");
    std::os::unix::fs::symlink(&real_dir, &symlink_path)
        .expect("failed to create uppercase .dag symlink");

    assert_single_target_command_failure_outputs_match_for_variants(
        "obligations",
        &root,
        "bundle_link.DAG",
        &[
            ("./bundle_link.DAG", "curdir-suffix alias"),
            ("nested/../bundle_link.DAG", "parent-segment alias"),
            ("nested/./../bundle_link.DAG", "parent-curdir alias"),
            (
                "nested/..//bundle_link.DAG",
                "parent-double-separator alias",
            ),
        ],
        &["--format", "json"],
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn show_triplets_command_shows_transport_expansion_for_makegen() {
    let output = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang show-triplets");

    assert!(
        output.status.success(),
        "show-triplets command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TransportTriplets:"));
    assert!(stdout.contains("prepare_read_makegen"));
    assert!(stdout.contains("execute_read_makegen"));
}

#[test]
fn show_triplets_command_json_format_emits_triplet_list() {
    let output = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang show-triplets --format json");

    assert!(
        output.status.success(),
        "show-triplets --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .expect("show-triplets --format json should emit valid JSON");
    let triplets = parsed
        .get("triplets")
        .and_then(Value::as_array)
        .expect("triplets should be a JSON array");
    assert!(
        !triplets.is_empty(),
        "triplet list should include at least one transport chain"
    );
}

#[test]
fn show_triplets_command_reports_none_for_transport_free_graph() {
    let fixture = unique_temp_file("show_triplets_none");
    std::fs::write(
        &fixture,
        r#"module sample.triplets
fn run() -> Unit { }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang show-triplets");

    assert!(
        output.status.success(),
        "show-triplets command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TransportTriplets:"));
    assert!(stdout.contains("(none)"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn show_triplets_command_json_reports_empty_array_for_transport_free_graph() {
    let fixture = unique_temp_file("show_triplets_json_none");
    std::fs::write(
        &fixture,
        r#"module sample.triplets
fn run() -> Unit { }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(&fixture)
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang show-triplets --format json");

    assert!(
        output.status.success(),
        "show-triplets --format json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .expect("show-triplets --format json should emit valid JSON");
    let triplets = parsed
        .get("triplets")
        .and_then(Value::as_array)
        .expect("triplets should be a JSON array");
    assert!(
        triplets.is_empty(),
        "transport-free graph should have no triplets"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn show_triplets_command_explicit_text_format_matches_default_output() {
    let default_output = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang show-triplets default format");
    assert!(
        default_output.status.success(),
        "default show-triplets command failed: {}",
        String::from_utf8_lossy(&default_output.stderr)
    );

    let explicit_text_output = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .arg("--format")
        .arg("text")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang show-triplets --format text");
    assert!(
        explicit_text_output.status.success(),
        "show-triplets --format text failed: {}",
        String::from_utf8_lossy(&explicit_text_output.stderr)
    );

    assert_eq!(
        default_output.stdout, explicit_text_output.stdout,
        "explicit text format should match default show-triplets output"
    );
}

#[test]
fn show_triplets_command_json_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang show-triplets --format json");
    assert!(
        first.status.success(),
        "first show-triplets json command failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang show-triplets --format json");
    assert!(
        second.status.success(),
        "second show-triplets json command failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    assert_eq!(
        first.stdout, second.stdout,
        "show-triplets json output should be deterministic"
    );
    assert_eq!(
        first.stderr, second.stderr,
        "show-triplets json stderr should be deterministic"
    );
}

#[test]
fn show_triplets_command_relative_directory_alias_normalized_spelling_fail_outputs_match_canonical()
{
    let root = unique_temp_dir(
        "show_triplets_relative_directory_alias_normalized_spelling_failure_output_parity",
    );
    let dag_dir = root.join("bundle.DaG");
    std::fs::create_dir_all(root.join("nested")).expect("failed to create nested directory");
    std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .DaG directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .DaG directory");

    assert_single_target_command_failure_outputs_match_for_variants(
        "show-triplets",
        &root,
        "bundle.DaG",
        &[
            ("./bundle.DaG", "curdir-suffix alias"),
            ("nested/../bundle.DaG", "parent-segment alias"),
            ("nested/./../bundle.DaG", "parent-curdir alias"),
            ("nested/..//bundle.DaG", "parent-double-separator alias"),
        ],
        &[],
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn show_triplets_command_json_relative_symlink_alias_normalized_spelling_fail_outputs_match_canonical(
) {
    let root = unique_temp_dir(
        "show_triplets_json_relative_symlink_alias_normalized_spelling_failure_output_parity",
    );
    let real_dir = root.join("real");
    std::fs::create_dir_all(root.join("nested")).expect("failed to create nested directory");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    let symlink_path = root.join("bundle_link.dag");
    std::os::unix::fs::symlink(&real_dir, &symlink_path)
        .expect("failed to create lowercase .dag symlink");

    assert_single_target_command_failure_outputs_match_for_variants(
        "show-triplets",
        &root,
        "bundle_link.dag",
        &[
            ("./bundle_link.dag", "curdir-suffix alias"),
            ("nested/../bundle_link.dag", "parent-segment alias"),
            ("nested/./../bundle_link.dag", "parent-curdir alias"),
            (
                "nested/..//bundle_link.dag",
                "parent-double-separator alias",
            ),
        ],
        &["--format", "json"],
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn viz_command_defaults_to_ascii_for_compiled_file() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz file");

    assert!(
        output.status.success(),
        "viz command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DAG daglang-compiled"));
    assert!(stdout.contains("tools.makegen::render_makefile"));
}

#[test]
fn viz_command_mermaid_format_renders_mermaid_for_compiled_file() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg(makegen_file())
        .arg("--format")
        .arg("mermaid")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz file with mermaid format");

    assert!(
        output.status.success(),
        "viz --format mermaid command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("flowchart TB"));
    assert!(stdout.contains("tools.makegen::render_makefile"));
}

#[test]
fn compile_command_reports_diagnostics_for_invalid_file() {
    let broken = unique_temp_file("broken");
    std::fs::write(&broken, "module sample.broken\nfn broken( -> String {")
        .expect("failed to write broken source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&broken)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for broken file");

    assert!(
        !output.status.success(),
        "broken source should fail compile"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}

#[test]
fn compile_command_reports_file_line_col_for_broken_file() {
    let broken_file = unique_temp_file("compile_broken_line_col");
    std::fs::write(
        &broken_file,
        "module tmp.bad\nfn broken( -> String {\n  \"oops\"\n}\n",
    )
    .expect("failed to create broken dag file");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&broken_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for broken file");

    assert!(!output.status.success(), "broken file should fail compile");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(
        stderr.contains(":2:12:"),
        "expected file:line:col in stderr, got: {stderr}"
    );

    std::fs::remove_file(broken_file).expect("failed to remove broken dag file");
}

#[test]
fn compile_command_reports_lex_diagnostic_for_unknown_character() {
    let broken_file = unique_temp_file("compile_lex_unknown_character");
    std::fs::write(&broken_file, "module tmp.lex\n$\n").expect("failed to create lex-invalid file");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&broken_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for lex-invalid file");

    assert!(
        !output.status.success(),
        "lex-invalid file should fail compile"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains("unexpected character '$'"));
    assert!(
        stderr.contains(":2:1:"),
        "expected lex diagnostic line/column in stderr, got: {stderr}"
    );

    std::fs::remove_file(broken_file).expect("failed to remove lex-invalid file");
}

#[test]
fn compile_command_directory_mode_aggregates_multiple_file_diagnostics() {
    let root = unique_temp_dir("compile_directory_mode_errors");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("broken_a.dag"), "module sample.a\nfn")
        .expect("failed to write broken_a");
    std::fs::write(root.join("broken_b.dag"), "module sample.b\nimport")
        .expect("failed to write broken_b");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail when multiple files are invalid"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains("broken_a.dag"));
    assert!(stderr.contains("broken_b.dag"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_outputs_deterministic_diagnostic_order() {
    let root = unique_temp_dir("compile_directory_mode_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("z_broken.dag"), "module sample.z\nfn")
        .expect("failed to write z_broken");
    std::fs::write(root.join("a_broken.dag"), "module sample.a\nfn")
        .expect("failed to write a_broken");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail when files are invalid"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let first_diagnostic_line = stderr
        .lines()
        .find(|line| line.contains(".dag:"))
        .expect("expected at least one diagnostic line with file path");
    assert!(
        first_diagnostic_line.contains("a_broken.dag"),
        "diagnostics should be deterministically sorted by path: {stderr}"
    );
    assert!(stderr.contains("z_broken.dag"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_diagnostic_output_is_deterministic_for_same_input() {
    let root = unique_temp_dir("compile_output_deterministic");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write z_lex");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write a_parse");

    let first = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang compile");
    assert!(!first.status.success(), "first compile run should fail");

    let second = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang compile");
    assert!(!second.status.success(), "second compile run should fail");

    assert_eq!(
        first.stderr, second.stderr,
        "compile diagnostics should be deterministic for identical inputs"
    );
    assert_eq!(
        first.stdout, second.stdout,
        "compile stdout should be deterministic for identical inputs"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&first.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_sorts_lex_diagnostics_before_parse_diagnostics() {
    let root = unique_temp_dir("compile_kind_order");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("a_parse.dag"), "module sample.parse\nfn")
        .expect("failed to write parse-error file");
    std::fs::write(root.join("z_lex.dag"), "module sample.lex\n$\n")
        .expect("failed to write lex-error file");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on mixed-error directory");

    assert!(
        !output.status.success(),
        "compile should fail when diagnostics are present"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let first_diagnostic_line = stderr
        .lines()
        .find(|line| line.contains(".dag:"))
        .expect("expected at least one diagnostic line");
    assert!(
        first_diagnostic_line.contains("z_lex.dag"),
        "lex diagnostics should sort before parse diagnostics: {stderr}"
    );
    assert!(stderr.contains("a_parse.dag"));
    assert!(stderr.contains("unexpected character '$'"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_single_file_mode_ignores_sibling_broken_files() {
    let root = unique_temp_dir("compile_single_file_ignores_sibling_broken");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    let good_file = root.join("good.dag");
    std::fs::write(&good_file, "module sample.good\nfn ok() -> Unit {}")
        .expect("failed to write good file");
    std::fs::write(root.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write broken sibling");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&good_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on single file");

    assert!(
        output.status.success(),
        "single-file compile should succeed even with sibling broken file: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(
        output.stderr.is_empty(),
        "single-file compile should not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_empty_directory_reports_lower_stage_error() {
    let root = unique_temp_dir("compile_empty_directory_lower_error");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on empty directory");

    assert!(
        !output.status.success(),
        "compile should fail for empty directory roots"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("no callable or pipeline declarations to lower"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_non_dag_only_directory_reports_lower_stage_error() {
    let root = unique_temp_dir("compile_non_dag_only_directory");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("notes.txt"), "this should be ignored")
        .expect("failed to write non-dag file");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on non-dag-only directory");

    assert!(
        !output.status.success(),
        "compile should fail with lower-stage error when no .dag files exist"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("no callable or pipeline declarations to lower"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_ignores_non_dag_files_when_dag_files_exist() {
    let root = unique_temp_dir("compile_ignore_non_dag_mixed");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit { }",
    )
    .expect("failed to write dag file");
    std::fs::write(root.join("notes.txt"), "module fake\n$").expect("failed to write txt file");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on mixed dag/non-dag directory");

    assert!(
        output.status.success(),
        "compile should succeed when dag files exist even with non-dag siblings: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim().is_empty(),
        "compile success should not emit stderr"
    );
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Compiled 1 module(s)"),
        "expected compile summary to report one dag module: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn expand_command_reports_diagnostics_for_invalid_file() {
    let broken = unique_temp_file("expand_broken");
    std::fs::write(&broken, "module sample.broken\nfn broken( -> String {")
        .expect("failed to write broken source");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&broken)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand for broken file");

    assert!(!output.status.success(), "broken source should fail expand");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}

#[test]
fn progress_command_reports_diagnostics_for_invalid_file() {
    let broken = unique_temp_file("progress_broken");
    std::fs::write(&broken, "module sample.broken\nfn broken( -> String {")
        .expect("failed to write broken source");

    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(&broken)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress for broken file");

    assert!(
        !output.status.success(),
        "broken source should fail progress"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}

#[test]
fn expand_command_reports_unresolved_service_call_typecheck_error() {
    let fixture = unique_temp_file("unresolved_service_call");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on unresolved service fixture");

    assert!(
        !output.status.success(),
        "expand should fail when service call endpoint cannot be resolved"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved service call"));
    assert!(stderr.contains("MissingStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_reports_unresolved_uses_typecheck_error() {
    let fixture = unique_temp_file("unresolved_uses");
    std::fs::write(
        &fixture,
        r#"module sample.resources
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on unresolved uses fixture");

    assert!(
        !output.status.success(),
        "expand should fail when uses target cannot be resolved"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown used resource type"));
    assert!(stderr.contains("MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_reports_unresolved_provides_typecheck_error() {
    let fixture = unique_temp_file("unresolved_provides");
    std::fs::write(
        &fixture,
        r#"module sample.resources
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on unresolved provides fixture");

    assert!(
        !output.status.success(),
        "expand should fail when provides target cannot be resolved"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown provided resource type"));
    assert!(stderr.contains("MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_shows_param_source_wiring_for_identifier_service_args() {
    let fixture = unique_temp_file("service_param_source_expand");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on fixture");

    assert!(
        output.status.success(),
        "expand command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("param_source_sample_services_run_path"));
    assert!(stdout.contains(
        "param_source_sample_services_run_path.path -> prepare_transport_sample_services_FsStorage_read.path"
    ));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_imports() {
    let root = unique_temp_dir("unresolved_import");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nimport missing.dep\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved imports"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved import"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved imports should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_definitions() {
    let root = unique_temp_dir("duplicate_definitions");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn run() -> Unit {}
func run() -> { ok: Bool } {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `run` in module `sample.main`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_interface_also_reports_ambiguous_implements() {
    let root = unique_temp_dir("duplicate_interface_ambiguous_implements");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read { input { path: String } output { body: String } }
}
interface Storage {
  capability read { input { path: String } output { body: String } }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate interface definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `Storage` in module `sample.main`"));
    assert!(stderr.contains("`FsStorage` references ambiguous interface `Storage`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-interface layering should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_service_also_reports_ambiguous_service_call() {
    let root = unique_temp_dir("duplicate_service_ambiguous_call");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate service definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `FsStorage` in module `sample.main`"));
    assert!(stderr.contains("ambiguous service call `FsStorage.read` in `run`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-service layering should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_output_fields() {
    let root = unique_temp_dir("duplicate_output_fields");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run() -> { ok: Bool, ok: Bool } {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate output fields"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate output field `ok` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_parameters() {
    let root = unique_temp_dir("duplicate_parameters");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run(a: String, a: Int) -> String { a }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate parameters"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate parameter `a` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_interface_reference() {
    let root = unique_temp_dir("ambiguous_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/first.dag"),
        "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/second.dag"),
        "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nservice FsStorage implements Storage { operation read(path: String) -> { body: String } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous interface `Storage`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous interface references should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_resource_interface_reference() {
    let root = unique_temp_dir("ambiguous_resource_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/first.dag"),
        "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/second.dag"),
        "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nresource Disk implements Storage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous resource interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` references ambiguous interface `Storage`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous interface references should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_interface_reference() {
    let root = unique_temp_dir("unresolved_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nservice FsStorage implements MissingStorage { operation read(path: String) -> { body: String } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("references unresolved interface `MissingStorage`"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved interface references should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_resource_interface_reference() {
    let root = unique_temp_dir("unresolved_resource_interface_reference");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nresource Disk implements MissingStorage { capability read { input { path: String } output { body: String } } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved resource interface reference"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` references unresolved interface `MissingStorage`"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved interface references should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_missing_resource_capability() {
    let root = unique_temp_dir("missing_resource_capability");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail when resource is missing interface capability"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(
        stderr.contains("resource `Disk` is missing capability `write` for interface `Storage`")
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_missing_service_operation() {
    let root = unique_temp_dir("missing_service_operation");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail when service is missing interface operation"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(
        stderr.contains("service `FsStorage` is missing operation `write` for interface `Storage`")
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_interface_signature_mismatch() {
    let root = unique_temp_dir("interface_signature_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: Int) -> { body: String }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on interface signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`FsStorage` does not match `Storage.read` contract"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_resource_interface_signature_mismatch() {
    let root = unique_temp_dir("resource_interface_signature_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements Storage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on resource interface signature mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("`Disk` does not match `Storage.read` contract"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_use_provide_binding_conflict() {
    let root = unique_temp_dir("use_provide_binding_conflict");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses io: Storage provides io: Storage {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on use/provide binding conflict"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("binding `io` is declared in both uses/provides in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_uses_resource_type() {
    let root = unique_temp_dir("unknown_uses_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } uses fs: MissingResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unknown uses resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown used resource type `MissingResource`"));
    assert!(
        !stderr.contains("lower error"),
        "unknown uses resource type should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_uses_resource_with_runtime_config_suffix() {
    let root = unique_temp_dir("configured_uses_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept configured uses resource type: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_uses_binding() {
    let root = unique_temp_dir("duplicate_uses_binding");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate uses binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate uses binding `fs` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_uses_resource_type() {
    let root = unique_temp_dir("ambiguous_uses_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous uses resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous used resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous uses resource types should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_resource_uses_also_reports_ambiguous_used_type() {
    let root = unique_temp_dir("duplicate_resource_uses_ambiguous");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate resource definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `SharedResource` in module `sample.main`"));
    assert!(stderr.contains("ambiguous used resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-resource uses layering should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_provides_binding() {
    let root = unique_temp_dir("duplicate_provides_binding");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage provides out: Storage {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate provides binding"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate provides binding `out` in `run`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_provides_resource_type() {
    let root = unique_temp_dir("unknown_provides_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } provides out: MissingResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unknown provides resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown provided resource type `MissingResource`"));
    assert!(
        !stderr.contains("lower error"),
        "unknown provides resource type should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_provides_resource_with_runtime_config_suffix() {
    let root = unique_temp_dir("configured_provides_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource ArtifactStore {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept configured provides resource type: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_provides_resource_type() {
    let root = unique_temp_dir("ambiguous_provides_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nresource SharedResource {}",
    )
    .expect("failed to write source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfunc run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous provides resource type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous provided resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous provides resource types should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_resource_provides_also_reports_ambiguous_provided_type()
{
    let root = unique_temp_dir("duplicate_resource_provides_ambiguous");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource { return { ok: true } }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate resource definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `SharedResource` in module `sample.main`"));
    assert!(stderr.contains("ambiguous provided resource type `SharedResource`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-resource provides layering should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_reports_call_arity_typecheck_error() {
    let fixture = unique_temp_file("call_arity");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("call arity mismatch"));
    assert!(stderr.contains("fmt"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_unknown_named_call_argument_typecheck_error() {
    let fixture = unique_temp_file("call_unknown_arg");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on unknown named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown named argument `text`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_undefined_type_typecheck_error() {
    let fixture = unique_temp_file("undefined_type");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(input: MissingType) -> String { "ok" }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on undefined type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("undefined type `MissingType"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_type_mismatch_typecheck_error() {
    let fixture = unique_temp_file("type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { return 42 }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_implicit_return_type_mismatch_typecheck_error() {
    let fixture = unique_temp_file("implicit_return_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { 42 }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on implicit return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_missing_tail_expression_type_mismatch_typecheck_error() {
    let fixture = unique_temp_file("missing_tail_expression_type_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run() -> String { let x = 42 }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail when fn has no tail expression for non-unit return type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Unit`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_no_such_field_typecheck_error() {
    let fixture = unique_temp_file("no_such_field");
    std::fs::write(
        &fixture,
        r#"module sample.types
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on missing record field access"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Record` has no field `missing`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_no_such_field_for_named_record_type() {
    let fixture = unique_temp_file("no_such_field_named_record");
    std::fs::write(
        &fixture,
        r#"module sample.types
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on missing field access for named record type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Payload` has no field `missing`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_unsatisfiable_refinement_typecheck_error() {
    let fixture = unique_temp_file("unsatisfiable_refinement");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(value: Int @range(min: 5, max: 1)) -> Int { value }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on unsatisfiable refinement"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The @range annotation in function parameter position now fails at parse
    // stage (unexpected character '@') rather than typecheck stage.
    assert!(
        stderr.contains("compile diagnostics"),
        "expected compile diagnostics in stderr: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_generic_arity_mismatch_typecheck_error() {
    let fixture = unique_temp_file("generic_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
fn run(values: Map<String>) -> Int { 1 }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Map`: expected 2, got 1"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_user_defined_generic_arity_mismatch_typecheck_error() {
    let fixture = unique_temp_file("user_defined_generic_arity_mismatch");
    std::fs::write(
        &fixture,
        r#"module sample.types
type Box<T> = T
fn run(values: Box<String, Int>) -> String { values }
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on user-defined generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Box`: expected 1, got 2"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_service_call_arity_typecheck_error() {
    let fixture = unique_temp_file("service_call_arity");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read()
  return { ok: true }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on service call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("service call arity mismatch"));
    assert!(stderr.contains("FsStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_unknown_named_service_argument_typecheck_error() {
    let fixture = unique_temp_file("service_call_unknown_arg");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(file: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on unknown named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown named argument `file`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_duplicate_named_call_argument_typecheck_error() {
    let fixture = unique_temp_file("duplicate_named_call_arg");
    std::fs::write(
        &fixture,
        r#"module sample.calls
fn fmt(value: String, mode: String) -> String { value }
fn run() -> String {
  fmt(value: "a", value: "b")
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on duplicate named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate named argument `value`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_reports_duplicate_named_service_argument_typecheck_error() {
    let fixture = unique_temp_file("duplicate_named_service_arg");
    std::fs::write(
        &fixture,
        r#"module sample.services
interface Storage {
  capability read {
    input { path: String, mode: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String, mode: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path, path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write fixture");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile");

    assert!(
        !output.status.success(),
        "compile should fail on duplicate named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate named argument `path`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_directory_mode_fails_on_call_arity_typecheck_error() {
    let root = unique_temp_dir("call_arity");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt() }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("call arity mismatch"));
    assert!(stderr.contains("fmt"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_omitting_defaulted_call_args() {
    let root = unique_temp_dir("call_default_args");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn greet(name: String, punctuation: String = "!") -> String { name }
fn run() -> String { greet(name: "hi") }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept omitted defaulted call args: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_pattern_calls_with_extra_named_wiring_args() {
    let root = unique_temp_dir("pattern_wiring_call_args");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
pattern ensure(should_act: Bool = true) -> { acted: Bool } {
  return { acted: should_act }
}
fn run() -> Bool {
  let result = ensure(check: true, action: false)
  result.acted
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept pattern calls with extra named wiring args: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_generic_fn_type_params() {
    let root = unique_temp_dir("generic_fn_type_params");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn identity<T>(value: T) -> T {
  value
}
fn relay<T>(value: T) -> T {
  identity(value: value)
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept generic fn type params: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_generic_pattern_type_params() {
    let root = unique_temp_dir("generic_pattern_type_params");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
pattern passthrough<T: Serializable>(value: T) -> { value: T } {
  return { value: value }
}
fn relay<T>(value: T) -> T {
  let result = passthrough(value: value)
  result.value
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept generic pattern type params: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_named_record_literal_returns() {
    let root = unique_temp_dir("named_record_literal_return");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type StageResult {
  success: Bool,
  skipped: Bool
}
fn result() -> StageResult {
  { success: true, skipped: false }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept named-record literal returns: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_resource_config_named_type_returns() {
    let root = unique_temp_dir("resource_config_named_return");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
resource GcsBucket {
  config {
    name: String,
    project: String
  }
}
fn gcp_dev_storage() -> GcsBucket.Config {
  { name: "gunbc-dev-artifacts", project: "gunbai-auto" }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept resource config named-type returns: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_named_call_argument_typecheck_error() {
    let root = unique_temp_dir("call_unknown_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn fmt(value: String) -> String { value }
fn run() -> String { fmt(text: "ok") }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unknown named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown named argument `text`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_named_call_argument_typecheck_error() {
    let root = unique_temp_dir("duplicate_named_call_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn fmt(value: String, mode: String) -> String { value }
fn run() -> String {
  fmt(value: "a", value: "b")
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate named call argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate named argument `value`"));
    assert!(stderr.contains("call to `fmt`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_service_call_arity_typecheck_error() {
    let root = unique_temp_dir("service_call_arity");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read()
  return { ok: true }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on service call arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("service call arity mismatch"));
    assert!(stderr.contains("FsStorage.read"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_omitting_defaulted_service_call_args() {
    let root = unique_temp_dir("service_call_default_args");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input {
      path: String,
      recursive: Bool = false
    }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String, recursive: Bool = false) -> { ok: Bool }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read(path: "/tmp")
  return { ok: response.ok }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept omitted defaulted service call args: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unknown_named_service_argument_typecheck_error() {
    let root = unique_temp_dir("service_call_unknown_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(file: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unknown named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unknown named argument `file`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_duplicate_named_service_argument_typecheck_error() {
    let root = unique_temp_dir("duplicate_named_service_arg");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
interface Storage {
  capability read {
    input { path: String, mode: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String, mode: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path, path: path)
  return { body: response.body }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate named service argument"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate named argument `path`"));
    assert!(stderr.contains("service call `FsStorage.read`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_service_call() {
    let root = unique_temp_dir("unresolved_service_call_typecheck");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved service call"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved service call `MissingStorage.read`"));
    assert!(
        !stderr.contains("lower error"),
        "directory unresolved service call should fail before lowering: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_service_call() {
    let root = unique_temp_dir("ambiguous_service_call_typecheck");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/first.dag"),
        r#"module sample.first
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
    )
    .expect("failed to write first service source");
    std::fs::write(
        root.join("sample/second.dag"),
        r#"module sample.second
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
    )
    .expect("failed to write second service source");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run(path: String) -> { body: String } {
  let response = SharedService.read(path: path)
  return { body: response.body }
}"#,
    )
    .expect("failed to write main source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous service call"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous service call `SharedService.read`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous service calls should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_ambiguous_callable_target() {
    let root = unique_temp_dir("ambiguous_callable_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/one.dag"),
        "module sample.one\nfn render(value: String) -> String { value }",
    )
    .expect("failed to write first callable source");
    std::fs::write(
        root.join("sample/two.dag"),
        "module sample.two\nfn render(value: String) -> String { value }",
    )
    .expect("failed to write second callable source");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { render(value: \"ok\") }",
    )
    .expect("failed to write main source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on ambiguous callable target"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("ambiguous call target `render`"));
    assert!(
        !stderr.contains("lower error"),
        "ambiguous callable targets should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_duplicate_callable_also_reports_ambiguous_call_target() {
    let root = unique_temp_dir("duplicate_callable_ambiguous_call_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on duplicate callable definitions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("duplicate definition `helper` in module `sample.main`"));
    assert!(stderr.contains("ambiguous call target `helper` in `run`"));
    assert!(
        !stderr.contains("lower error"),
        "duplicate-callable layering should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unresolved_callable_target() {
    let root = unique_temp_dir("unresolved_callable_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { missing(value: \"ok\") }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unresolved callable target"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unresolved call target `missing`"));
    assert!(
        !stderr.contains("lower error"),
        "unresolved callable targets should fail in typecheck stage: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_collection_intrinsic_call_targets() {
    let root = unique_temp_dir("collection_intrinsic_call_targets");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type Stage {
  success: Bool,
  skipped: Bool,
  name: String
}
fn summarize(stages: List<Stage>) -> Int {
  let passed = stages |> filter(s => s.success) |> count()
  let labels = stages |> map(s => s.name) |> join(",")
  let done = labels |> ends_with("ok")
  passed
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept collection intrinsic call targets: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_function_typed_parameter_calls() {
    let root = unique_temp_dir("function_typed_parameter_calls");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn apply(value: Int, callback: fn(Int) -> Int) -> Int {
  callback(value)
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept function-typed parameter calls: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_sum_variant_constructor_calls() {
    let root = unique_temp_dir("sum_variant_constructor_calls");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type CloudConfig
  = GcpConfig { project: String, region: String }
  | AwsConfig { region: String }

fn make_gcp() -> CloudConfig {
  GcpConfig(project: "gunbc", region: "us-central1")
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept sum variant constructor calls: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_zero_arity_variant_identifier_returns() {
    let root = unique_temp_dir("zero_arity_variant_identifier_returns");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type Environment = Dev | Ci
fn env() -> Environment {
  Dev
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept zero-arity variant identifier returns: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_lossy_match_body_without_tail_mismatch() {
    let root = unique_temp_dir("lossy_match_body");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type CloudConfig
  = GcpConfig { project: String }
  | AwsConfig { account: String }
type CloudProvider = Gcp | Aws

fn provider_of(config: CloudConfig) -> CloudProvider {
  match config {
    GcpConfig { ... } => Gcp
    AwsConfig { ... } => Aws
  }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should tolerate lossy match-body parsing without false tail mismatch: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_accepts_std_helper_intrinsic_call_targets() {
    let root = unique_temp_dir("std_helper_intrinsic_call_targets");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type DocgenSources {}

fn run(sources: DocgenSources, payload: String) -> String {
  let a = "template" |> replace_section("section", "value")
  let b = render_test_listings(sources: sources)
  let c = render_graph_structure(sources: sources)
  let d = render_source_artifacts(sources: sources)
  let e = compute_topology_diff(current: "{}", base: "{}")
  let f = render_annotated_mermaid(diff: e, topology: "{}", title: "title")
  let g = detect_runtime()
  let h = generate()
  let i = now()
  let j = build_token(
    payload: payload,
    scheme: "Bearer",
    header_name: "Authorization",
    source_id: "source",
    required_scopes: ["gist"]
  )
  a
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should accept std helper intrinsic call targets: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_type_mismatch() {
    let root = unique_temp_dir("type_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { return 42 }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_implicit_return_type_mismatch() {
    let root = unique_temp_dir("implicit_type_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { 42 }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on implicit return type mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Int`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_allows_unit_return_without_tail_expression() {
    let root = unique_temp_dir("unit_missing_tail");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit { let x = 42 }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        output.status.success(),
        "directory compile should allow missing tail for Unit return: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "Unit-return success path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "Unit-return success path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_missing_tail_expression_type_mismatch() {
    let root = unique_temp_dir("missing_tail_expression_type_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run() -> String { let x = 42 }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail when fn has no tail expression for non-unit return type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type mismatch: expected `String`, got `Unit`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_no_such_field_for_record_literal() {
    let root = unique_temp_dir("no_such_field_record_literal");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on missing field access for record literal"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Record` has no field `missing`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_no_such_field_for_named_record() {
    let root = unique_temp_dir("no_such_field_named_record");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on missing field for named record type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("type `Payload` has no field `missing`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_generic_arity_mismatch() {
    let root = unique_temp_dir("generic_arity_mismatch");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn run(values: Map<String>) -> Int { 1 }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Map`: expected 2, got 1"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_unsatisfiable_refinement() {
    let root = unique_temp_dir("unsatisfiable_refinement");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        "module sample.main\nfn run(value: Int @range(min: 9, max: 1)) -> Int { value }",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on unsatisfiable refinement"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The @range annotation in function parameter position now fails at parse
    // stage (unexpected character '@') rather than typecheck stage.
    assert!(
        stderr.contains("compile diagnostics"),
        "expected compile diagnostics in stderr: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_undefined_type_typecheck_error() {
    let root = unique_temp_dir("undefined_type");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
fn run(input: MissingType) -> String { "ok" }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on undefined type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("undefined type `MissingType"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_fails_on_user_defined_generic_arity_mismatch() {
    let root = unique_temp_dir("user_defined_generic_arity");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp dir");
    std::fs::write(
        root.join("sample/main.dag"),
        r#"module sample.main
type Box<T> = T
fn run(value: Box<String, Int>) -> String { value }
"#,
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on user-defined generic arity mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("generic arity mismatch for `Box`: expected 1, got 2"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn compile_command_directory_mode_reports_module_path_mismatch() {
    let root = unique_temp_dir("path_mismatch");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module wrong.name\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile on mismatch directory");

    assert!(
        !output.status.success(),
        "directory compile should fail on module path mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn expand_command_directory_mode_reports_module_path_mismatch() {
    let root = unique_temp_dir("expand_path_mismatch");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module wrong.name\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("expand")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang expand on mismatch directory");

    assert!(
        !output.status.success(),
        "directory expand should fail on module path mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn progress_command_directory_mode_reports_module_path_mismatch() {
    let root = unique_temp_dir("progress_path_mismatch");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module wrong.name\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("progress")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang progress on mismatch directory");

    assert!(
        !output.status.success(),
        "directory progress should fail on module path mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}
