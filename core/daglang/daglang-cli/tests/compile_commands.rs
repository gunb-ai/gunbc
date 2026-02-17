// Test infrastructure: filesystem access for test fixtures
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

fn makegen_file() -> PathBuf {
    workspace_root().join("dsl/tools/makegen.dag")
}

fn unique_temp_file(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "daglang_cli_compile_cmd_{name}_{}_{}.dag",
        std::process::id(),
        nanos
    ))
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "daglang_cli_compile_cmd_dir_{name}_{}_{}",
        std::process::id(),
        nanos
    ))
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

fn run_single_target_command_with_optional_trailing_slash(
    command_name: &str,
    root: &Path,
    input: &str,
) -> (Output, Output) {
    let plain = Command::new(daglang_bin())
        .arg(command_name)
        .arg(input)
        .current_dir(root)
        .output()
        .expect("failed to run plain single-target command invocation");
    let trailing = Command::new(daglang_bin())
        .arg(command_name)
        .arg(format!("{input}/"))
        .current_dir(root)
        .output()
        .expect("failed to run trailing-slash single-target command invocation");
    (plain, trailing)
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
    if let Some(snippet) = nested_diagnostic_snippet {
        assert!(
            !stderr.contains(snippet),
            "{input} should fail before parsing nested files: {stderr}"
        );
    }
    assert_no_stage_failures(&stderr);
}

fn assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
    command_name: &str,
    root: &Path,
    input: &str,
    expected_target: &Path,
    nested_diagnostic_snippet: Option<&str>,
) {
    let (plain, trailing) =
        run_single_target_command_with_optional_trailing_slash(command_name, root, input);
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
    assert!(stdout.contains("Compiled 1 module(s)"));
    assert!(stdout.contains("target/generated/rust/main.rs"));
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

#[test]
fn compile_command_directory_named_dag_extension_is_treated_as_invalid_single_file_target() {
    let root = unique_temp_dir("compile_directory_named_dag_extension");
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .dag directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .dag directory");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "bundle.dag",
        &dag_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_directory_named_dag_extension_with_nested_errors_still_fails_as_single_file_target(
) {
    let root = unique_temp_dir("compile_directory_named_dag_extension_errors");
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .dag directory root");
    let broken_file = dag_dir.join("broken.dag");
    std::fs::write(&broken_file, "module sample.broken\nfn")
        .expect("failed to write malformed source in .dag directory");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "bundle.dag",
        &dag_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_directory_named_uppercase_dag_extension_is_treated_as_invalid_single_file_target(
) {
    let root = unique_temp_dir("compile_directory_named_uppercase_dag_extension");
    let dag_dir = root.join("bundle.DAG");
    std::fs::create_dir_all(dag_dir.join("sample"))
        .expect("failed to create .DAG directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .DAG directory");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "bundle.DAG",
        &dag_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_directory_named_mixed_case_dag_extension_is_treated_as_invalid_single_file_target(
) {
    let root = unique_temp_dir("compile_directory_named_mixed_case_dag_extension");
    let dag_dir = root.join("bundle.DaG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory root");
    std::fs::write(dag_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in .DaG directory");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "bundle.DaG",
        &dag_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_uppercase_dag_directory_is_treated_as_invalid_single_file_target(
) {
    let root = unique_temp_dir("compile_curdir_suffix_uppercase_dag_directory");
    let dag_dir = root.join("bundle.DAG");
    std::fs::create_dir_all(dag_dir.join("sample"))
        .expect("failed to create .DAG directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .DAG directory");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "./bundle.DAG",
        &dag_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_mixed_case_dag_directory_is_treated_as_invalid_single_file_target(
) {
    let root = unique_temp_dir("compile_curdir_suffix_mixed_case_dag_directory");
    let dag_dir = root.join("bundle.DaG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory root");
    std::fs::write(dag_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in .DaG directory");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "./bundle.DaG",
        &dag_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_symlink_directory_named_dag_extension_is_treated_as_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_symlink_directory_named_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.dag");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .dag directory symlink");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "link.dag",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_symlink_directory_named_mixed_case_dag_extension_is_treated_as_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_symlink_directory_named_mixed_case_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DaG");
    std::fs::create_dir_all(&real_dir).expect("failed to create real directory root");
    std::fs::write(real_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create mixed-case .DaG directory symlink");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "link.DaG",
        &link_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_symlink_directory_named_uppercase_dag_extension_is_treated_as_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_symlink_directory_named_uppercase_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DAG");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create uppercase .DAG directory symlink");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "link.DAG",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_curdir_suffix_symlink_named_dag_extension_is_invalid_single_file_target() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_curdir_suffix_symlink_named_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.dag");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create lowercase .dag directory symlink");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "./link.dag",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_curdir_suffix_symlink_named_uppercase_dag_extension_is_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_curdir_suffix_symlink_named_uppercase_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DAG");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create uppercase .DAG directory symlink");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "./link.DAG",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn compile_command_curdir_suffix_symlink_named_mixed_case_dag_extension_is_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("compile_curdir_suffix_symlink_named_mixed_case_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DaG");
    std::fs::create_dir_all(&real_dir).expect("failed to create real directory root");
    std::fs::write(real_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create mixed-case .DaG directory symlink");

    assert_dag_suffixed_directory_is_invalid_single_file_target(
        &root,
        "./link.DaG",
        &link_dir,
        Some("broken.dag:2:3"),
    );

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
        stderr.contains(&format!(
            "failed to read {}",
            dangling_link.display()
        )),
        "dangling-target compile diagnostics should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_relative_and_absolute_missing_roots_are_equivalent() {
    let root = unique_temp_dir("compile_relative_absolute_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let relative = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run relative missing-root compile");
    assert!(
        !relative.status.success(),
        "relative missing-root compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&missing)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute missing-root compile");
    assert!(
        !absolute.status.success(),
        "absolute missing-root compile should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "missing-root diagnostic should contain normalized absolute path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_relative_and_absolute_non_directory_roots_are_equivalent() {
    let root = unique_temp_dir("compile_relative_absolute_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let file_path = root.join("input.txt");
    std::fs::write(&file_path, "not a directory").expect("failed to write non-directory root");

    let relative = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run relative non-directory-root compile");
    assert!(
        !relative.status.success(),
        "relative non-directory-root compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&file_path)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute non-directory-root compile");
    assert!(
        !absolute.status.success(),
        "absolute non-directory-root compile should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&file_path.display().to_string()),
        "non-directory-root diagnostic should contain normalized absolute path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_missing_root_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_segment_missing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg("../missing_root")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-segment missing-root compile");
    assert!(
        !parent_segment.status.success(),
        "parent-segment missing-root compile should fail"
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
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_segment.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "parent-segment missing-root diagnostic should normalize to absolute path: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_trailing_slash_missing_root_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_segment_trailing_missing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let parent_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("../missing_root/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-trailing missing-root compile");
    assert!(
        !parent_trailing.status.success(),
        "parent-trailing missing-root compile should fail"
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
        parent_trailing.stdout, absolute.stdout,
        "parent-trailing and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        parent_trailing.stderr, absolute.stderr,
        "parent-trailing and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_trailing.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "parent-trailing missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_missing_root_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_double_separator_missing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let parent_double = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//missing_root")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double missing-root compile");
    assert!(
        !parent_double.status.success(),
        "parent-double missing-root compile should fail"
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
        parent_double.stdout, absolute.stdout,
        "parent-double and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        parent_double.stderr, absolute.stderr,
        "parent-double and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "parent-double missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_trailing_slash_missing_root_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_double_trailing_missing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let parent_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//missing_root/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double-trailing missing-root compile");
    assert!(
        !parent_double_trailing.status.success(),
        "parent-double-trailing missing-root compile should fail"
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
        parent_double_trailing.stdout, absolute.stdout,
        "parent-double-trailing and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        parent_double_trailing.stderr, absolute.stderr,
        "parent-double-trailing and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double_trailing.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "parent-double-trailing missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_non_directory_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_segment_non_directory_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg("../input.txt")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-segment non-directory-root compile");
    assert!(
        !parent_segment.status.success(),
        "parent-segment non-directory-root compile should fail"
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
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_segment.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "parent-segment non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_trailing_slash_non_directory_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_segment_trailing_non_directory_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let parent_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("../input.txt/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-trailing non-directory-root compile");
    assert!(
        !parent_trailing.status.success(),
        "parent-trailing non-directory-root compile should fail"
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
        parent_trailing.stdout, absolute.stdout,
        "parent-trailing and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        parent_trailing.stderr, absolute.stderr,
        "parent-trailing and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_trailing.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "parent-trailing non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_non_directory_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_double_non_directory_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let parent_double = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//input.txt")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double non-directory-root compile");
    assert!(
        !parent_double.status.success(),
        "parent-double non-directory-root compile should fail"
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
        parent_double.stdout, absolute.stdout,
        "parent-double and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        parent_double.stderr, absolute.stderr,
        "parent-double and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "parent-double non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_trailing_slash_non_directory_root_matches_absolute_output()
{
    let root = unique_temp_dir("compile_parent_double_trailing_non_directory_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let parent_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//input.txt/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double-trailing non-directory-root compile");
    assert!(
        !parent_double_trailing.status.success(),
        "parent-double-trailing non-directory-root compile should fail"
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
        parent_double_trailing.stdout, absolute.stdout,
        "parent-double-trailing and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        parent_double_trailing.stderr, absolute.stderr,
        "parent-double-trailing and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double_trailing.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "parent-double-trailing non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let curdir = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment missing-root compile");
    assert!(
        !curdir.status.success(),
        "curdir-segment missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir.stdout,
        "plain and curdir missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir.stderr,
        "plain and curdir missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-segment missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_trailing_slash_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_trailing_slash_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing_root/")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing missing-root compile");
    assert!(
        !curdir_segment_trailing.status.success(),
        "curdir-segment-trailing missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_trailing.stdout,
        "plain and curdir-segment-trailing missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_trailing.stderr,
        "plain and curdir-segment-trailing missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-segment-trailing missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_trailing_slash_missing_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_trailing_slash_missing_root",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root/")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-trailing-slash missing-root compile",
        );
    assert!(
        !dot_double_separator_curdir_segment_trailing_slash.status.success(),
        "dot-double-separator-curdir-segment-trailing-slash missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_trailing_slash.stdout,
        "plain and dot-double-separator-curdir-segment-trailing-slash missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_trailing_slash.stderr,
        "plain and dot-double-separator-curdir-segment-trailing-slash missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-curdir-segment-trailing-slash missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_suffix_missing_root_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_segment_suffix_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root/./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run dot-double-separator-curdir-segment-suffix missing-root compile",
        );
    assert!(
        !dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_segment_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-suffix missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_segment_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-suffix missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-curdir-segment-suffix missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_curdir_suffix_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix missing-root compile");
    assert!(
        !dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix.stdout,
        "plain and dot-double-separator-curdir-suffix missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix.stderr,
        "plain and dot-double-separator-curdir-suffix missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-curdir-suffix missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_suffix_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing_root/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix missing-root compile");
    assert!(
        !curdir_suffix.status.success(),
        "curdir-suffix missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-suffix missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_suffix_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_suffix_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing_root/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix missing-root compile");
    assert!(
        !curdir_segment_suffix.status.success(),
        "curdir-segment-suffix missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_suffix.stdout,
        "plain and curdir-segment-suffix missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_suffix.stderr,
        "plain and curdir-segment-suffix missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-segment-suffix missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_double_separator_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_suffix_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing_root//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator missing-root compile");
    assert!(
        !curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_suffix_double_separator.stdout,
        "plain and curdir-suffix-double-separator missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix_double_separator.stderr,
        "plain and curdir-suffix-double-separator missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-suffix-double-separator missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_suffix_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_double_separator_suffix_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing_root//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator-suffix missing-root compile");
    assert!(
        !curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator_suffix.stdout,
        "plain and curdir-segment-double-separator-suffix missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator_suffix.stderr,
        "plain and curdir-segment-double-separator-suffix missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-segment-double-separator-suffix missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_suffix_missing_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_suffix_missing_root",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator-suffix missing-root compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-curdir-segment-double-separator-suffix missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing_root//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator missing-root compile");
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator.stdout,
        "plain and curdir-segment-double-separator missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator.stderr,
        "plain and curdir-segment-double-separator missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "curdir-segment-double-separator missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_missing_root_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_segment_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator missing-root compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator.status.success(),
        "dot-double-separator-curdir-segment-double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-curdir-segment-double-separator missing-root diagnostic should normalize to absolute path: {stderr}"
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
    assert!(!plain.status.success(), "plain missing-root compile should fail");

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
fn compile_command_double_separator_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let double_sep = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root//")
        .current_dir(&root)
        .output()
        .expect("failed to run double-separator missing-root compile");
    assert!(
        !double_sep.status.success(),
        "double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, double_sep.stdout,
        "plain and double-separator missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, double_sep.stderr,
        "plain and double-separator missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "double-separator missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_trailing_slash_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_trailing_slash_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash missing-root compile");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, trailing_slash.stdout,
        "plain and trailing-slash missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing_slash.stderr,
        "plain and trailing-slash missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "trailing-slash missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_relative_and_absolute_missing_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("compile_relative_absolute_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let relative = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative missing single-file compile");
    assert!(
        !relative.status.success(),
        "relative missing single-file compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&missing)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute missing single-file compile");
    assert!(
        !absolute.status.success(),
        "absolute missing single-file compile should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "missing single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_trailing_slash_missing_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_trailing_slash_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash missing single-file compile");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, trailing_slash.stdout,
        "plain and trailing-slash missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing_slash.stderr,
        "plain and trailing-slash missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "trailing-slash missing single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_missing_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let curdir = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment missing single-file compile");
    assert!(
        !curdir.status.success(),
        "curdir-segment missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir.stdout,
        "plain and curdir missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir.stderr,
        "plain and curdir missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "curdir-segment missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_trailing_slash_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_segment_trailing_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing missing single-file compile");
    assert!(
        !curdir_segment_trailing.status.success(),
        "curdir-segment-trailing missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_trailing.stdout,
        "plain and curdir-segment-trailing missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_trailing.stderr,
        "plain and curdir-segment-trailing missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "curdir-segment-trailing missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_trailing_slash_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_trailing_missing_single_file_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-trailing-slash missing single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_trailing_slash.status.success(),
        "dot-double-separator-curdir-segment-trailing-slash missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_trailing_slash.stdout,
        "plain and dot-double-separator-curdir-segment-trailing-slash missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_trailing_slash.stderr,
        "plain and dot-double-separator-curdir-segment-trailing-slash missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator-curdir-segment-trailing-slash missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_suffix_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_suffix_missing_single_file_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-suffix missing single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_segment_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-suffix missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_segment_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-suffix missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator-curdir-segment-suffix missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_missing_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_suffix_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix missing single-file compile");
    assert!(
        !dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix.stdout,
        "plain and dot-double-separator-curdir-suffix missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix.stderr,
        "plain and dot-double-separator-curdir-suffix missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator-curdir-suffix missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_missing_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_suffix_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix missing single-file compile");
    assert!(
        !curdir_suffix.status.success(),
        "curdir-suffix missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "curdir-suffix missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_suffix_missing_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_suffix_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix missing single-file compile");
    assert!(
        !curdir_segment_suffix.status.success(),
        "curdir-segment-suffix missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_suffix.stdout,
        "plain and curdir-segment-suffix missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_suffix.stderr,
        "plain and curdir-segment-suffix missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "curdir-segment-suffix missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_double_separator_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_suffix_double_separator_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator missing single-file compile");
    assert!(
        !curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_suffix_double_separator.stdout,
        "plain and curdir-suffix-double-separator missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix_double_separator.stderr,
        "plain and curdir-suffix-double-separator missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "curdir-suffix-double-separator missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_suffix_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_curdir_segment_double_separator_suffix_missing_single_file_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix missing single-file compile",
        );
    assert!(
        !curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator_suffix.stdout,
        "plain and curdir-segment-double-separator-suffix missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator_suffix.stderr,
        "plain and curdir-segment-double-separator-suffix missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "curdir-segment-double-separator-suffix missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_suffix_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_suffix_missing_single_file_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator-suffix missing single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator-curdir-segment-double-separator-suffix missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_segment_double_separator_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./missing.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator missing single-file compile");
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator.stdout,
        "plain and curdir-segment-double-separator missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator.stderr,
        "plain and curdir-segment-double-separator missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "curdir-segment-double-separator missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_missing_single_file_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator missing single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator.status.success(),
        "dot-double-separator-curdir-segment-double-separator missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator-curdir-segment-double-separator missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_relative_and_absolute_invalid_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("compile_relative_absolute_invalid_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let relative = Command::new(daglang_bin())
        .arg("compile")
        .arg("invalid.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative invalid single-file compile");
    assert!(
        !relative.status.success(),
        "relative invalid single-file compile should fail"
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&invalid_target)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute invalid single-file compile");
    assert!(
        !absolute.status.success(),
        "absolute invalid single-file compile should fail"
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&relative.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_trailing_slash_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_trailing_invalid_single_file_target");
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

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg("invalid.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing invalid single-file compile");
    assert!(
        !trailing_slash.status.success(),
        "trailing invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, trailing_slash.stdout,
        "plain and trailing invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing_slash.stderr,
        "plain and trailing invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "trailing invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_invalid_single_file_target");
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

    let curdir = Command::new(daglang_bin())
        .arg("compile")
        .arg("./invalid.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir invalid single-file compile");
    assert!(
        !curdir.status.success(),
        "curdir invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir.stdout,
        "plain and curdir invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir.stderr,
        "plain and curdir invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_segment_trailing_invalid_single_file_target");
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

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("./invalid.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing invalid single-file compile");
    assert!(
        !curdir_segment_trailing.status.success(),
        "curdir-segment-trailing invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_trailing.stdout,
        "plain and curdir-segment-trailing invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_trailing.stderr,
        "plain and curdir-segment-trailing invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir-segment-trailing invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_trailing_slash_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_trailing_invalid_single_file_target",
    );
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

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-trailing-slash invalid single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_trailing_slash.status.success(),
        "dot-double-separator-curdir-segment-trailing-slash invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_trailing_slash.stdout,
        "plain and dot-double-separator-curdir-segment-trailing-slash invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_trailing_slash.stderr,
        "plain and dot-double-separator-curdir-segment-trailing-slash invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator-curdir-segment-trailing-slash invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_suffix_invalid_single_file_target",
    );
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

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-suffix invalid single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_segment_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-suffix invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_segment_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-suffix invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator-curdir-segment-suffix invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_suffix_invalid_single_file_target");
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

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix invalid single-file compile");
    assert!(
        !dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix.stdout,
        "plain and dot-double-separator-curdir-suffix invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix.stderr,
        "plain and dot-double-separator-curdir-suffix invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator-curdir-suffix invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_suffix_invalid_single_file_target");
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

    let curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./invalid.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix invalid single-file compile");
    assert!(
        !curdir_suffix.status.success(),
        "curdir-suffix invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir-suffix invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_suffix_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_suffix_invalid_single_file_target");
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

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./invalid.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix invalid single-file compile");
    assert!(
        !curdir_segment_suffix.status.success(),
        "curdir-segment-suffix invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_suffix.stdout,
        "plain and curdir-segment-suffix invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_suffix.stderr,
        "plain and curdir-segment-suffix invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir-segment-suffix invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_suffix_double_separator_invalid_single_file_target");
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

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./invalid.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator invalid single-file compile");
    assert!(
        !curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_suffix_double_separator.stdout,
        "plain and curdir-suffix-double-separator invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix_double_separator.stderr,
        "plain and curdir-suffix-double-separator invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir-suffix-double-separator invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_curdir_segment_double_separator_suffix_invalid_single_file_target",
    );
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

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./invalid.dag//./.")
        .current_dir(&root)
        .output()
        .expect(
            "failed to run curdir-segment-double-separator-suffix invalid single-file compile",
        );
    assert!(
        !curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator_suffix.stdout,
        "plain and curdir-segment-double-separator-suffix invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator_suffix.stderr,
        "plain and curdir-segment-double-separator-suffix invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir-segment-double-separator-suffix invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_suffix_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_suffix_invalid_single_file_target",
    );
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

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator-suffix invalid single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator-curdir-segment-double-separator-suffix invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_segment_double_separator_invalid_single_file_target");
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

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./invalid.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator invalid single-file compile");
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator.stdout,
        "plain and curdir-segment-double-separator invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator.stderr,
        "plain and curdir-segment-double-separator invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "curdir-segment-double-separator invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_invalid_single_file_target",
    );
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

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator invalid single-file compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator.status.success(),
        "dot-double-separator-curdir-segment-double-separator invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator-curdir-segment-double-separator invalid single-file diagnostic should include normalized absolute path: {stderr}"
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
fn compile_command_parent_segment_missing_single_file_target_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_segment_missing_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg("../missing.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-segment missing single-file compile");
    assert!(
        !parent_segment.status.success(),
        "parent-segment missing single-file compile should fail"
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
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_segment.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "parent-segment missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_trailing_slash_missing_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_segment_trailing_missing_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let parent_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("../missing.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-trailing missing single-file compile");
    assert!(
        !parent_trailing.status.success(),
        "parent-trailing missing single-file compile should fail"
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
        parent_trailing.stdout, absolute.stdout,
        "parent-trailing and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        parent_trailing.stderr, absolute.stderr,
        "parent-trailing and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_trailing.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "parent-trailing missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_missing_single_file_target_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_double_missing_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let parent_double = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//missing.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double missing single-file compile");
    assert!(
        !parent_double.status.success(),
        "parent-double missing single-file compile should fail"
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
        parent_double.stdout, absolute.stdout,
        "parent-double and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        parent_double.stderr, absolute.stderr,
        "parent-double and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "parent-double missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_trailing_slash_missing_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_double_trailing_missing_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let parent_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//missing.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double-trailing missing single-file compile");
    assert!(
        !parent_double_trailing.status.success(),
        "parent-double-trailing missing single-file compile should fail"
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
        parent_double_trailing.stdout, absolute.stdout,
        "parent-double-trailing and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        parent_double_trailing.stderr, absolute.stderr,
        "parent-double-trailing and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double_trailing.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "parent-double-trailing missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

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
fn compile_command_parent_segment_trailing_slash_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_segment_trailing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let parent_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("../dsl/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-trailing root compile");
    assert!(
        parent_trailing.status.success(),
        "parent-trailing root compile should succeed: {}",
        String::from_utf8_lossy(&parent_trailing.stderr)
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
        parent_trailing.stdout, absolute.stdout,
        "parent-trailing and absolute root compile stdout should match"
    );
    assert_eq!(
        parent_trailing.stderr, absolute.stderr,
        "parent-trailing and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_trailing.stderr));

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
fn compile_command_parent_double_separator_trailing_slash_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_double_trailing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let parent_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//dsl/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double-trailing root compile");
    assert!(
        parent_double_trailing.status.success(),
        "parent-double-trailing root compile should succeed: {}",
        String::from_utf8_lossy(&parent_double_trailing.stderr)
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
        parent_double_trailing.stdout, absolute.stdout,
        "parent-double-trailing and absolute root compile stdout should match"
    );
    assert_eq!(
        parent_double_trailing.stderr, absolute.stderr,
        "parent-double-trailing and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_double_trailing.stderr));

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
fn compile_command_parent_segment_trailing_slash_single_file_target_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_segment_trailing_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root child");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root sample");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let parent_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("../sample/main.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-trailing single-file compile");
    assert!(
        parent_trailing.status.success(),
        "parent-trailing single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_trailing.stderr)
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
        parent_trailing.stdout, absolute.stdout,
        "parent-trailing and absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_trailing.stderr, absolute.stderr,
        "parent-trailing and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_trailing.stderr));

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
fn compile_command_parent_double_separator_trailing_slash_single_file_target_matches_absolute_output()
{
    let root = unique_temp_dir("compile_parent_double_trailing_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root child");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root sample");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let parent_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//sample/main.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double-trailing single-file compile");
    assert!(
        parent_double_trailing.status.success(),
        "parent-double-trailing single-file compile should succeed: {}",
        String::from_utf8_lossy(&parent_double_trailing.stderr)
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
        parent_double_trailing.stdout, absolute.stdout,
        "parent-double-trailing and absolute single-file compile stdout should match"
    );
    assert_eq!(
        parent_double_trailing.stderr, absolute.stderr,
        "parent-double-trailing and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&parent_double_trailing.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_segment_missing_root_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_curdir_segment_missing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././missing_root")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-segment missing-root compile");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment missing-root compile should fail"
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
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_segment.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "parent-curdir-segment missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_double_separator_missing_root_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_curdir_double_missing_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let parent_curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././missing_root//")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-double missing-root compile");
    assert!(
        !parent_curdir_double.status.success(),
        "parent-curdir-double missing-root compile should fail"
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
        parent_curdir_double.stdout, absolute.stdout,
        "parent-curdir-double and absolute missing-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_double.stderr, absolute.stderr,
        "parent-curdir-double and absolute missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_double.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "parent-curdir-double missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

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
fn compile_command_parent_curdir_segment_non_directory_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_segment_non_directory_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././input.txt")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-segment non-directory-root compile");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment non-directory-root compile should fail"
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
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_segment.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "parent-curdir-segment non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_double_separator_non_directory_root_matches_absolute_output() {
    let root = unique_temp_dir("compile_parent_curdir_double_non_directory_root");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let parent_curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././input.txt//")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-double non-directory-root compile");
    assert!(
        !parent_curdir_double.status.success(),
        "parent-curdir-double non-directory-root compile should fail"
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
        parent_curdir_double.stdout, absolute.stdout,
        "parent-curdir-double and absolute non-directory-root compile stdout should match"
    );
    assert_eq!(
        parent_curdir_double.stderr, absolute.stderr,
        "parent-curdir-double and absolute non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_double.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "parent-curdir-double non-directory-root diagnostic should normalize to absolute path: {stderr}"
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
fn compile_command_parent_curdir_segment_missing_single_file_target_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_curdir_segment_missing_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././missing.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-segment missing single-file compile");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment missing single-file compile should fail"
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
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_segment.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "parent-curdir-segment missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_double_separator_missing_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_curdir_double_missing_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let parent_curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././missing.dag//")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-double missing single-file compile");
    assert!(
        !parent_curdir_double.status.success(),
        "parent-curdir-double missing single-file compile should fail"
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
        parent_curdir_double.stdout, absolute.stdout,
        "parent-curdir-double and absolute missing single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_double.stderr, absolute.stderr,
        "parent-curdir-double and absolute missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_double.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "parent-curdir-double missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

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
fn compile_command_parent_segment_invalid_single_file_target_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_segment_invalid_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let parent_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg("../invalid.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-segment invalid single-file compile");
    assert!(
        !parent_segment.status.success(),
        "parent-segment invalid single-file compile should fail"
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
        parent_segment.stdout, absolute.stdout,
        "parent-segment and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        parent_segment.stderr, absolute.stderr,
        "parent-segment and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_segment.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "parent-segment invalid single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_segment_trailing_slash_invalid_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_segment_trailing_invalid_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let parent_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("../invalid.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-trailing invalid single-file compile");
    assert!(
        !parent_trailing.status.success(),
        "parent-trailing invalid single-file compile should fail"
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
        parent_trailing.stdout, absolute.stdout,
        "parent-trailing and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        parent_trailing.stderr, absolute.stderr,
        "parent-trailing and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_trailing.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "parent-trailing invalid single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_invalid_single_file_target_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_double_invalid_single_file_target");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let parent_double = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//invalid.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double invalid single-file compile");
    assert!(
        !parent_double.status.success(),
        "parent-double invalid single-file compile should fail"
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
        parent_double.stdout, absolute.stdout,
        "parent-double and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        parent_double.stderr, absolute.stderr,
        "parent-double and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "parent-double invalid single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_double_separator_trailing_slash_invalid_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_double_trailing_invalid_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let parent_double_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("..//invalid.dag/")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-double-trailing invalid single-file compile");
    assert!(
        !parent_double_trailing.status.success(),
        "parent-double-trailing invalid single-file compile should fail"
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
        parent_double_trailing.stdout, absolute.stdout,
        "parent-double-trailing and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        parent_double_trailing.stderr, absolute.stderr,
        "parent-double-trailing and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_double_trailing.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "parent-double-trailing invalid single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_segment_invalid_single_file_target_is_normalized_and_equivalent() {
    let root = unique_temp_dir("compile_parent_curdir_segment_invalid_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let parent_curdir_segment = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././invalid.dag")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-segment invalid single-file compile");
    assert!(
        !parent_curdir_segment.status.success(),
        "parent-curdir-segment invalid single-file compile should fail"
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
        parent_curdir_segment.stdout, absolute.stdout,
        "parent-curdir-segment and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_segment.stderr, absolute.stderr,
        "parent-curdir-segment and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_segment.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "parent-curdir-segment invalid single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_parent_curdir_double_separator_invalid_single_file_target_is_normalized_and_equivalent(
) {
    let root = unique_temp_dir("compile_parent_curdir_double_invalid_single_file");
    std::fs::create_dir_all(root.join("child")).expect("failed to create temp root");
    let invalid_target = root.join("invalid.dag");
    std::fs::create_dir_all(&invalid_target)
        .expect("failed to create invalid single-file target directory");

    let parent_curdir_double = Command::new(daglang_bin())
        .arg("compile")
        .arg(".././invalid.dag//")
        .current_dir(root.join("child"))
        .output()
        .expect("failed to run parent-curdir-double invalid single-file compile");
    assert!(
        !parent_curdir_double.status.success(),
        "parent-curdir-double invalid single-file compile should fail"
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
        parent_curdir_double.stdout, absolute.stdout,
        "parent-curdir-double and absolute invalid single-file compile stdout should match"
    );
    assert_eq!(
        parent_curdir_double.stderr, absolute.stderr,
        "parent-curdir-double and absolute invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&parent_curdir_double.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "parent-curdir-double invalid single-file diagnostic should normalize to absolute path: {stderr}"
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
fn compile_command_curdir_segment_trailing_slash_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_trailing_single_file_target");
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

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("./sample/main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing single-file compile");
    assert!(
        curdir_segment_trailing.status.success(),
        "curdir-segment-trailing single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_trailing.stdout,
        "plain and curdir-segment-trailing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_trailing.stderr,
        "plain and curdir-segment-trailing single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_trailing_slash_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_trailing_single_file_target",
    );
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

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-trailing-slash single-file compile");
    assert!(
        dot_double_separator_curdir_segment_trailing_slash.status.success(),
        "dot-double-separator-curdir-segment-trailing-slash single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_trailing_slash.stdout,
        "plain and dot-double-separator-curdir-segment-trailing-slash single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_trailing_slash.stderr,
        "plain and dot-double-separator-curdir-segment-trailing-slash single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_suffix_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_segment_suffix_single_file_target");
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

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-suffix single-file compile");
    assert!(
        dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_segment_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-suffix single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_segment_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-suffix single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_suffix_single_file_target");
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

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix single-file compile");
    assert!(
        dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix.stdout,
        "plain and dot-double-separator-curdir-suffix single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix.stderr,
        "plain and dot-double-separator-curdir-suffix single-file compile stderr should match"
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
fn compile_command_curdir_segment_suffix_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_suffix_single_file_target");
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

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./sample/main.dag/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix single-file compile");
    assert!(
        curdir_segment_suffix.status.success(),
        "curdir-segment-suffix single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_suffix.stdout,
        "plain and curdir-segment-suffix single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_suffix.stderr,
        "plain and curdir-segment-suffix single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_curdir_suffix_double_separator_single_file_target");
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

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./sample/main.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator single-file compile");
    assert!(
        curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix_double_separator.stdout,
        "plain and curdir-suffix-double-separator single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix_double_separator.stderr,
        "plain and curdir-suffix-double-separator single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_suffix_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_curdir_segment_double_separator_suffix_single_file_target");
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

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./sample/main.dag//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator-suffix single-file compile");
    assert!(
        curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator_suffix.stdout,
        "plain and curdir-segment-double-separator-suffix single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator_suffix.stderr,
        "plain and curdir-segment-double-separator-suffix single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_suffix_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_suffix_single_file_target",
    );
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

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator-suffix single-file compile");
    assert!(
        dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator_suffix.stderr)
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_single_file_target_matches_plain_relative_output()
{
    let root = unique_temp_dir("compile_curdir_segment_double_separator_single_file_target");
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

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./sample/main.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator single-file compile");
    assert!(
        curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator single-file compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator.stdout,
        "plain and curdir-segment-double-separator single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator.stderr,
        "plain and curdir-segment-double-separator single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_single_file_target",
    );
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

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator single-file compile");
    assert!(
        dot_double_separator_curdir_segment_double_separator.status.success(),
        "dot-double-separator-curdir-segment-double-separator single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_non_directory_root_matches_plain_relative_output()
{
    let root = unique_temp_dir("compile_curdir_segment_double_separator_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./input.txt//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator non-directory-root compile");
    assert!(
        !curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator.stdout,
        "plain and curdir-segment-double-separator non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator.stderr,
        "plain and curdir-segment-double-separator non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "curdir-segment-double-separator non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_non_directory_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_non_directory_root",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator non-directory-root compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator.status.success(),
        "dot-double-separator-curdir-segment-double-separator non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-curdir-segment-double-separator non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_suffix_non_directory_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_double_separator_suffix_non_directory_root",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator-suffix non-directory-root compile");
    assert!(
        !dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-double-separator-suffix non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-curdir-segment-double-separator-suffix non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_trailing_slash_non_directory_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_trailing_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("./input.txt/")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing non-directory-root compile");
    assert!(
        !curdir_segment_trailing.status.success(),
        "curdir-segment-trailing non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, curdir_segment_trailing.stdout,
        "plain and curdir-segment-trailing non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_trailing.stderr,
        "plain and curdir-segment-trailing non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "curdir-segment-trailing non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_trailing_slash_non_directory_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_trailing_non_directory_root",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-trailing-slash non-directory-root compile");
    assert!(
        !dot_double_separator_curdir_segment_trailing_slash.status.success(),
        "dot-double-separator-curdir-segment-trailing-slash non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_trailing_slash.stdout,
        "plain and dot-double-separator-curdir-segment-trailing-slash non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_trailing_slash.stderr,
        "plain and dot-double-separator-curdir-segment-trailing-slash non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-curdir-segment-trailing-slash non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_suffix_non_directory_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_segment_suffix_non_directory_root",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-suffix non-directory-root compile");
    assert!(
        !dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_segment_suffix.stdout,
        "plain and dot-double-separator-curdir-segment-suffix non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_segment_suffix.stderr,
        "plain and dot-double-separator-curdir-segment-suffix non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-curdir-segment-suffix non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_non_directory_root_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_suffix_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix non-directory-root compile");
    assert!(
        !dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix.stdout,
        "plain and dot-double-separator-curdir-suffix non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix.stderr,
        "plain and dot-double-separator-curdir-suffix non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-curdir-suffix non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_trailing_slash_non_directory_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt/")
        .current_dir(&root)
        .output()
        .expect("failed to run trailing-slash non-directory-root compile");
    assert!(
        !trailing_slash.status.success(),
        "trailing-slash non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, trailing_slash.stdout,
        "plain and trailing-slash non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, trailing_slash.stderr,
        "plain and trailing-slash non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "trailing-slash non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

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
fn compile_command_relative_and_absolute_single_file_targets_are_equivalent() {
    let root = unique_temp_dir("compile_relative_absolute_single_file_target");
    std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
    let source = root.join("sample/main.dag");
    std::fs::write(&source, "module sample.main\nfn run() -> Unit { }")
        .expect("failed to write single-file source");

    let relative = Command::new(daglang_bin())
        .arg("compile")
        .arg("sample/main.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run relative single-file compile");
    assert!(
        relative.status.success(),
        "relative single-file compile should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&source)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute single-file compile");
    assert!(
        absolute.status.success(),
        "absolute single-file compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute single-file compile stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&relative.stderr));
    assert!(
        String::from_utf8_lossy(&relative.stdout).contains("Compiled 1 module(s)"),
        "single-file compile summary missing: {}",
        String::from_utf8_lossy(&relative.stdout)
    );

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
fn compile_command_relative_and_absolute_roots_are_equivalent() {
    let root = unique_temp_dir("compile_relative_absolute_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_root = root.join("dsl");

    let relative = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl")
        .current_dir(&root)
        .output()
        .expect("failed to run relative-root compile");
    assert!(
        relative.status.success(),
        "relative-root compile should succeed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("compile")
        .arg(&absolute_root)
        .current_dir(&root)
        .output()
        .expect("failed to run absolute-root compile");
    assert!(
        absolute.status.success(),
        "absolute-root compile should succeed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute root compile stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&relative.stderr));

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
fn compile_command_absolute_curdir_segment_double_separator_root_matches_canonical_absolute_output() {
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
        &curdir_segment_double_separator.stderr
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
fn compile_command_absolute_curdir_suffix_double_separator_root_matches_canonical_absolute_output() {
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
        &curdir_suffix_double_separator.stderr
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
        &curdir_segment_double_separator_suffix.stderr
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
        &parent_curdir_segment_trailing.stderr
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
        &parent_curdir_segment_double.stderr
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_double_separator_trailing_slash_root_matches_canonical_absolute_output(
) {
    let root =
        unique_temp_dir("compile_absolute_parent_curdir_segment_double_trailing_slash_root");
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
        .expect("failed to run parent-curdir-segment-double-separator-trailing absolute root compile");
    assert!(
        parent_curdir_segment_double_separator_trailing.status.success(),
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
        &parent_curdir_segment_double_separator_trailing.stderr
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
fn compile_command_absolute_parent_segment_double_separator_root_matches_canonical_absolute_output() {
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
        &parent_segment_double_trailing.stderr
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
        &curdir_segment_double_separator.stderr
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
        &curdir_suffix_double_separator.stderr
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
        &curdir_segment_double_separator_suffix.stderr
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
    let absolute_parent_curdir_segment_trailing_target =
        PathBuf::from(format!("{}/anchor/.././dsl/sample/main.dag/", root.display()));

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
        &parent_curdir_segment_trailing.stderr
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_curdir_segment_double_separator_single_file_target_matches_canonical_output(
) {
    let root = unique_temp_dir("compile_absolute_parent_curdir_segment_double_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_curdir_segment_double_target =
        PathBuf::from(format!("{}/anchor/.././dsl/sample//main.dag", root.display()));
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
        &parent_curdir_segment_double.stderr
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
    let absolute_parent_curdir_segment_double_separator_trailing_target =
        PathBuf::from(format!("{}/anchor//.././dsl/sample/main.dag/", root.display()));

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
        &parent_curdir_segment_double_separator_trailing.stderr
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
    let absolute_parent_segment_double_trailing_target =
        PathBuf::from(format!("{}/dsl/sample/..//sample/main.dag/", root.display()));
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
        &parent_segment_double_trailing.stderr
    ));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_absolute_parent_segment_trailing_slash_single_file_target_matches_canonical_output()
{
    let root = unique_temp_dir("compile_absolute_parent_segment_trailing_single_file");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    write_minimal_directory_compile_fixture(&root);
    let absolute_parent_segment_trailing_target =
        PathBuf::from(format!("{}/", root.join("dsl/sample/../sample/main.dag").display()));
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
            PathBuf::from(format!("{}/", root.join("anchor/../missing_root").display())),
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
        (
            "parent_curdir_segment",
            root.join("anchor/.././input.txt"),
        ),
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
            PathBuf::from(format!("{}/dsl/sample/..//sample/missing.dag", root.display())),
        ),
        (
            "parent_trailing",
            PathBuf::from(format!("{}/", root.join("dsl/sample/../sample/missing.dag").display())),
        ),
        (
            "parent_double_separator_trailing",
            PathBuf::from(format!("{}/dsl/sample/..//sample/missing.dag/", root.display())),
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
fn compile_command_curdir_segment_trailing_slash_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_trailing_root");
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

    let curdir_segment_trailing = Command::new(daglang_bin())
        .arg("compile")
        .arg("./dsl/")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-trailing root compile");
    assert!(
        curdir_segment_trailing.status.success(),
        "curdir-segment-trailing root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_trailing.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_trailing.stdout,
        "plain-relative and curdir-segment-trailing root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_trailing.stderr,
        "plain-relative and curdir-segment-trailing root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_trailing_slash_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_curdir_segment_trailing_root");
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

    let dot_double_separator_curdir_segment_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-trailing-slash root compile");
    assert!(
        dot_double_separator_curdir_segment_trailing_slash.status.success(),
        "dot-double-separator-curdir-segment-trailing-slash root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_trailing_slash.stderr)
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_trailing_slash.stdout,
        "plain-relative and dot-double-separator-curdir-segment-trailing-slash root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_trailing_slash.stderr,
        "plain-relative and dot-double-separator-curdir-segment-trailing-slash root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_suffix_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_curdir_segment_suffix_root");
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

    let dot_double_separator_curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-suffix root compile");
    assert!(
        dot_double_separator_curdir_segment_suffix.status.success(),
        "dot-double-separator-curdir-segment-suffix root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_segment_suffix.stdout,
        "plain-relative and dot-double-separator-curdir-segment-suffix root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_segment_suffix.stderr,
        "plain-relative and dot-double-separator-curdir-segment-suffix root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_curdir_suffix_root");
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

    let dot_double_separator_curdir_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl/.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix root compile");
    assert!(
        dot_double_separator_curdir_suffix.status.success(),
        "dot-double-separator-curdir-suffix root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix.stdout,
        "plain-relative and dot-double-separator-curdir-suffix root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix.stderr,
        "plain-relative and dot-double-separator-curdir-suffix root compile stderr should match"
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
fn compile_command_curdir_segment_suffix_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_suffix_root");
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

    let curdir_segment_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./dsl/./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-suffix root compile");
    assert!(
        curdir_segment_suffix.status.success(),
        "curdir-segment-suffix root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_suffix.stdout,
        "plain-relative and curdir-segment-suffix root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_suffix.stderr,
        "plain-relative and curdir-segment-suffix root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_suffix_double_separator_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_suffix_double_separator_root");
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

    let curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./dsl//.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-suffix-double-separator root compile");
    assert!(
        curdir_suffix_double_separator.status.success(),
        "curdir-suffix-double-separator root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_suffix_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix_double_separator.stdout,
        "plain-relative and curdir-suffix-double-separator root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix_double_separator.stderr,
        "plain-relative and curdir-suffix-double-separator root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_suffix_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_double_separator_suffix_root");
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

    let curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg("./dsl//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator-suffix root compile");
    assert!(
        curdir_segment_double_separator_suffix.status.success(),
        "curdir-segment-double-separator-suffix root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator_suffix.stdout,
        "plain-relative and curdir-segment-double-separator-suffix root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator_suffix.stderr,
        "plain-relative and curdir-segment-double-separator-suffix root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_suffix_root_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_segment_double_separator_suffix_root");
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

    let dot_double_separator_curdir_segment_double_separator_suffix = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl//./.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator-suffix root compile");
    assert!(
        dot_double_separator_curdir_segment_double_separator_suffix
            .status
            .success(),
        "dot-double-separator-curdir-segment-double-separator-suffix root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator_suffix.stderr)
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator_suffix.stdout,
        "plain-relative and dot-double-separator-curdir-segment-double-separator-suffix root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator_suffix.stderr,
        "plain-relative and dot-double-separator-curdir-segment-double-separator-suffix root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_double_separator_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_curdir_suffix_double_separator_root");
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

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl//.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix-double-separator root compile");
    assert!(
        dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix_double_separator.stdout,
        "plain-relative and dot-double-separator-curdir-suffix-double-separator root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix_double_separator.stderr,
        "plain-relative and dot-double-separator-curdir-suffix-double-separator root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_double_separator_missing_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_curdir_suffix_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root//.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix-double-separator missing-root compile");
    assert!(
        !dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix_double_separator.stdout,
        "plain and dot-double-separator-curdir-suffix-double-separator missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix_double_separator.stderr,
        "plain and dot-double-separator-curdir-suffix-double-separator missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-curdir-suffix-double-separator missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_double_separator_non_directory_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_suffix_double_separator_non_directory_root",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt//.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix-double-separator non-directory-root compile");
    assert!(
        !dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix_double_separator.stdout,
        "plain and dot-double-separator-curdir-suffix-double-separator non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix_double_separator.stderr,
        "plain and dot-double-separator-curdir-suffix-double-separator non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-curdir-suffix-double-separator non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root =
        unique_temp_dir("compile_dot_double_separator_curdir_suffix_double_separator_single_file");
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

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix-double-separator single-file compile");
    assert!(
        dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_suffix_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix_double_separator.stdout,
        "plain and dot-double-separator-curdir-suffix-double-separator single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix_double_separator.stderr,
        "plain and dot-double-separator-curdir-suffix-double-separator single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_double_separator_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_suffix_double_separator_missing_single_file_target",
    );
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix-double-separator missing single-file compile");
    assert!(
        !dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix_double_separator.stdout,
        "plain and dot-double-separator-curdir-suffix-double-separator missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix_double_separator.stderr,
        "plain and dot-double-separator-curdir-suffix-double-separator missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator-curdir-suffix-double-separator missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_suffix_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir(
        "compile_dot_double_separator_curdir_suffix_double_separator_invalid_single_file_target",
    );
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

    let dot_double_separator_curdir_suffix_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag//.")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-suffix-double-separator invalid single-file compile");
    assert!(
        !dot_double_separator_curdir_suffix_double_separator
            .status
            .success(),
        "dot-double-separator-curdir-suffix-double-separator invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_curdir_suffix_double_separator.stdout,
        "plain and dot-double-separator-curdir-suffix-double-separator invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_curdir_suffix_double_separator.stderr,
        "plain and dot-double-separator-curdir-suffix-double-separator invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator-curdir-suffix-double-separator invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

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
fn compile_command_dot_double_separator_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator missing-root compile");
    assert!(
        !dot_double_separator.status.success(),
        "dot-double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator.stdout,
        "plain and dot-double-separator missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator.stderr,
        "plain and dot-double-separator missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_non_directory_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator non-directory-root compile");
    assert!(
        !dot_double_separator.status.success(),
        "dot-double-separator non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator.stdout,
        "plain and dot-double-separator non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator.stderr,
        "plain and dot-double-separator non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

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
fn compile_command_dot_double_separator_missing_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator missing single-file compile");
    assert!(
        !dot_double_separator.status.success(),
        "dot-double-separator missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator.stdout,
        "plain and dot-double-separator missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator.stderr,
        "plain and dot-double-separator missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_invalid_single_file_target_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_invalid_single_file_target");
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

    let dot_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator invalid single-file compile");
    assert!(
        !dot_double_separator.status.success(),
        "dot-double-separator invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator.stdout,
        "plain and dot-double-separator invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator.stderr,
        "plain and dot-double-separator invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_double_separator_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_double_separator_root");
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

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator root compile");
    assert!(
        dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_double_separator.stdout,
        "plain-relative and dot-double-separator-double-separator root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_double_separator.stderr,
        "plain-relative and dot-double-separator-double-separator root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_double_separator_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_double_separator_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator missing-root compile");
    assert!(
        !dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_double_separator.stdout,
        "plain and dot-double-separator-double-separator missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_double_separator.stderr,
        "plain and dot-double-separator-double-separator missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-double-separator missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_double_separator_non_directory_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_double_separator_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator non-directory-root compile");
    assert!(
        !dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_double_separator.stdout,
        "plain and dot-double-separator-double-separator non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_double_separator.stderr,
        "plain and dot-double-separator-double-separator non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-double-separator non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_double_separator_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_double_separator_single_file_target");
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

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator single-file compile");
    assert!(
        dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_double_separator.stdout,
        "plain and dot-double-separator-double-separator single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_double_separator.stderr,
        "plain and dot-double-separator-double-separator single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_double_separator_missing_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_double_separator_missing_single_file_target");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing.dag");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing.dag")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing single-file compile");
    assert!(
        !plain.status.success(),
        "plain missing single-file compile should fail"
    );

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator missing single-file compile");
    assert!(
        !dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator missing single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_double_separator.stdout,
        "plain and dot-double-separator-double-separator missing single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_double_separator.stderr,
        "plain and dot-double-separator-double-separator missing single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", missing.display())),
        "dot-double-separator-double-separator missing single-file diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_double_separator_invalid_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_double_separator_invalid_single_file_target");
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

    let dot_double_separator_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//invalid.dag//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-double-separator invalid single-file compile");
    assert!(
        !dot_double_separator_double_separator.status.success(),
        "dot-double-separator-double-separator invalid single-file compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_double_separator.stdout,
        "plain and dot-double-separator-double-separator invalid single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_double_separator.stderr,
        "plain and dot-double-separator-double-separator invalid single-file compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&format!("failed to read {}", invalid_target.display())),
        "dot-double-separator-double-separator invalid single-file diagnostic should include normalized absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_trailing_slash_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_trailing_slash_root");
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

    let dot_double_separator_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-trailing-slash root compile");
    assert!(
        dot_double_separator_trailing_slash.status.success(),
        "dot-double-separator-trailing-slash root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_trailing_slash.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_trailing_slash.stdout,
        "plain-relative and dot-double-separator-trailing-slash root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_trailing_slash.stderr,
        "plain-relative and dot-double-separator-trailing-slash root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_trailing_slash_missing_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_dot_double_separator_trailing_slash_missing_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let missing = root.join("missing_root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("missing_root")
        .current_dir(&root)
        .output()
        .expect("failed to run plain missing-root compile");
    assert!(!plain.status.success(), "plain missing-root compile should fail");

    let dot_double_separator_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//missing_root/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-trailing-slash missing-root compile");
    assert!(
        !dot_double_separator_trailing_slash.status.success(),
        "dot-double-separator-trailing-slash missing-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_trailing_slash.stdout,
        "plain and dot-double-separator-trailing-slash missing-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_trailing_slash.stderr,
        "plain and dot-double-separator-trailing-slash missing-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&missing.display().to_string()),
        "dot-double-separator-trailing-slash missing-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_trailing_slash_non_directory_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_trailing_slash_non_directory_root");
    std::fs::create_dir_all(&root).expect("failed to create temp root");
    let root_file = root.join("input.txt");
    std::fs::write(&root_file, "not a directory").expect("failed to write non-directory root");

    let plain = Command::new(daglang_bin())
        .arg("compile")
        .arg("input.txt")
        .current_dir(&root)
        .output()
        .expect("failed to run plain non-directory-root compile");
    assert!(
        !plain.status.success(),
        "plain non-directory-root compile should fail"
    );

    let dot_double_separator_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//input.txt/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-trailing-slash non-directory-root compile");
    assert!(
        !dot_double_separator_trailing_slash.status.success(),
        "dot-double-separator-trailing-slash non-directory-root compile should fail"
    );

    assert_eq!(
        plain.stdout, dot_double_separator_trailing_slash.stdout,
        "plain and dot-double-separator-trailing-slash non-directory-root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_trailing_slash.stderr,
        "plain and dot-double-separator-trailing-slash non-directory-root compile stderr should match"
    );
    let stderr = String::from_utf8_lossy(&plain.stderr);
    assert!(
        stderr.contains(&root_file.display().to_string()),
        "dot-double-separator-trailing-slash non-directory-root diagnostic should normalize to absolute path: {stderr}"
    );
    assert_no_stage_failures(&stderr);

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_trailing_slash_single_file_target_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_trailing_slash_single_file_target");
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

    let dot_double_separator_trailing_slash = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//sample/main.dag/")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-trailing-slash single-file compile");
    assert!(
        dot_double_separator_trailing_slash.status.success(),
        "dot-double-separator-trailing-slash single-file compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_trailing_slash.stderr)
    );

    assert_eq!(
        plain.stdout, dot_double_separator_trailing_slash.stdout,
        "plain and dot-double-separator-trailing-slash single-file compile stdout should match"
    );
    assert_eq!(
        plain.stderr, dot_double_separator_trailing_slash.stderr,
        "plain and dot-double-separator-trailing-slash single-file compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_curdir_segment_double_separator_root_matches_plain_relative_output() {
    let root = unique_temp_dir("compile_curdir_segment_double_separator_root");
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

    let curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg("./dsl//")
        .current_dir(&root)
        .output()
        .expect("failed to run curdir-segment-double-separator root compile");
    assert!(
        curdir_segment_double_separator.status.success(),
        "curdir-segment-double-separator root compile should succeed: {}",
        String::from_utf8_lossy(&curdir_segment_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_segment_double_separator.stdout,
        "plain-relative and curdir-segment-double-separator root compile stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_segment_double_separator.stderr,
        "plain-relative and curdir-segment-double-separator root compile stderr should match"
    );
    assert_no_stage_failures(&String::from_utf8_lossy(&plain.stderr));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn compile_command_dot_double_separator_curdir_segment_double_separator_root_matches_plain_relative_output(
) {
    let root = unique_temp_dir("compile_dot_double_separator_curdir_segment_double_separator_root");
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

    let dot_double_separator_curdir_segment_double_separator = Command::new(daglang_bin())
        .arg("compile")
        .arg(".//dsl//")
        .current_dir(&root)
        .output()
        .expect("failed to run dot-double-separator-curdir-segment-double-separator root compile");
    assert!(
        dot_double_separator_curdir_segment_double_separator.status.success(),
        "dot-double-separator-curdir-segment-double-separator root compile should succeed: {}",
        String::from_utf8_lossy(&dot_double_separator_curdir_segment_double_separator.stderr)
    );

    assert_eq!(
        plain.stdout,
        dot_double_separator_curdir_segment_double_separator.stdout,
        "plain-relative and dot-double-separator-curdir-segment-double-separator root compile stdout should match"
    );
    assert_eq!(
        plain.stderr,
        dot_double_separator_curdir_segment_double_separator.stderr,
        "plain-relative and dot-double-separator-curdir-segment-double-separator root compile stderr should match"
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
fn compile_command_single_file_duplicate_callable_does_not_report_ambiguous_call_target() {
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
        !stderr.contains("ambiguous call target `helper`"),
        "single-file relaxed mode should not report ambiguous call-target diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-callable path should not fail in lowering: {stderr}"
    );

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_duplicate_service_does_not_report_ambiguous_service_call() {
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
        !stderr.contains("ambiguous service call `FsStorage.read`"),
        "single-file relaxed mode should not report ambiguous service-call diagnostics: {stderr}"
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
fn compile_command_single_file_duplicate_resource_uses_does_not_report_ambiguous_used_type() {
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
    assert!(stderr.contains(
        "duplicate definition `SharedResource` in module `sample.single`"
    ));
    assert!(
        !stderr.contains("ambiguous used resource type `SharedResource`"),
        "single-file relaxed mode should suppress ambiguous used resource diagnostics: {stderr}"
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
fn compile_command_single_file_duplicate_resource_provides_does_not_report_ambiguous_provided_type() {
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
    assert!(stderr.contains(
        "duplicate definition `SharedResource` in module `sample.single`"
    ));
    assert!(
        !stderr.contains("ambiguous provided resource type `SharedResource`"),
        "single-file relaxed mode should suppress ambiguous provided resource diagnostics: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "single-file relaxed duplicate-resource provides path should not fail in lowering: {stderr}"
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
    assert!(stderr.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

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
    assert!(stderr.contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

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
fn compile_command_single_file_unresolved_service_call_reports_lower_error() {
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
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved service call"));
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
fn compile_command_single_file_unresolved_uses_reports_lower_error() {
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
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved used resource"));
    assert!(stderr.contains("fs: MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn compile_command_single_file_unresolved_provides_reports_lower_error() {
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
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved provided resource"));
    assert!(stderr.contains("out: MissingResource"));

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
fn compile_command_single_file_allows_unresolved_imports() {
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

    assert!(
        output.status.success(),
        "single-file compile should tolerate unresolved imports: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "relaxed unresolved-import path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "relaxed unresolved-import path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

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
fn compile_command_single_file_allows_unresolved_call_targets() {
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

    assert!(
        output.status.success(),
        "single-file compile should tolerate unresolved call targets: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("typecheck errors"),
        "relaxed unresolved-call path should not emit typecheck failures: {stderr}"
    );
    assert!(
        !stderr.contains("lower error"),
        "relaxed unresolved-call path should not emit lower-stage failures: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));

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
fn manifest_command_shows_derived_progress_manifest() {
    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ProgressManifest:"));
    assert!(stdout.contains("total_nodes:"));
    assert!(stdout.contains("waves:"));
    assert!(stdout.contains("TestObligations:"));
    assert!(stdout.contains("service_transport_prepare_targets:"));
    assert!(stdout.contains("service_param_source_targets:"));
    assert!(stdout.contains("resource_provide_targets:"));
}

#[test]
fn manifest_command_reports_non_zero_transport_and_lifecycle_obligations() {
    let fixture = unique_temp_file("manifest_obligations");
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
        .arg("manifest")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on fixture");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
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
fn manifest_command_reports_zero_service_param_source_targets_for_literal_args() {
    let fixture = unique_temp_file("manifest_param_sources_zero");
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
        .arg("manifest")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on fixture");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("service_param_source_targets: 0"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn manifest_command_interface_only_provides_has_no_release_obligation() {
    let fixture = unique_temp_file("manifest_interface_provides");
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
        .arg("manifest")
        .arg(&fixture)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on fixture");

    assert!(
        output.status.success(),
        "manifest command failed: {}",
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
fn manifest_and_obligations_commands_share_obligation_text_output() {
    let manifest_output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest");
    assert!(
        manifest_output.status.success(),
        "manifest command failed: {}",
        String::from_utf8_lossy(&manifest_output.stderr)
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

    let manifest_stdout = String::from_utf8_lossy(&manifest_output.stdout);
    let obligations_stdout = String::from_utf8_lossy(&obligations_output.stdout);
    assert_eq!(
        obligations_block(&manifest_stdout),
        obligations_stdout,
        "manifest obligation section should match standalone obligations output"
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
fn obligations_command_curdir_suffix_target_matches_plain_relative_output() {
    let plain = Command::new(daglang_bin())
        .arg("obligations")
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run plain-target daglang obligations");
    assert!(
        plain.status.success(),
        "plain-target obligations command failed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_suffix = Command::new(daglang_bin())
        .arg("obligations")
        .arg("./dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run curdir-suffix-target daglang obligations");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix-target obligations command failed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix obligations stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix obligations stderr should match"
    );
}

#[test]
fn obligations_command_relative_and_absolute_targets_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("obligations")
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-target daglang obligations");
    assert!(
        relative.status.success(),
        "relative-target obligations command failed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-target daglang obligations");
    assert!(
        absolute.status.success(),
        "absolute-target obligations command failed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute obligations stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute obligations stderr should match"
    );
}

#[test]
fn obligations_command_json_curdir_suffix_target_matches_plain_relative_output() {
    let plain = Command::new(daglang_bin())
        .arg("obligations")
        .arg("dsl/tools/makegen.dag")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run plain-target daglang obligations --format json");
    assert!(
        plain.status.success(),
        "plain-target obligations --format json failed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_suffix = Command::new(daglang_bin())
        .arg("obligations")
        .arg("./dsl/tools/makegen.dag")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run curdir-suffix-target daglang obligations --format json");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix-target obligations --format json failed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix obligations json stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix obligations json stderr should match"
    );
}

#[test]
fn obligations_command_json_relative_and_absolute_targets_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("obligations")
        .arg("dsl/tools/makegen.dag")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-target daglang obligations --format json");
    assert!(
        relative.status.success(),
        "relative-target obligations --format json failed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("obligations")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-target daglang obligations --format json");
    assert!(
        absolute.status.success(),
        "absolute-target obligations --format json failed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute obligations json stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute obligations json stderr should match"
    );
}

#[test]
fn obligations_command_directory_named_dag_extension_is_invalid_single_file_target() {
    let root = unique_temp_dir("obligations_directory_named_dag_extension");
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .dag directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .dag directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "bundle.dag",
        &dag_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn obligations_command_directory_named_uppercase_dag_extension_is_invalid_single_file_target() {
    let root = unique_temp_dir("obligations_directory_named_uppercase_dag_extension");
    let dag_dir = root.join("bundle.DAG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory root");
    std::fs::write(dag_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in .DAG directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "bundle.DAG",
        &dag_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn obligations_command_curdir_suffix_directory_named_dag_extension_is_invalid_single_file_target() {
    let root = unique_temp_dir("obligations_curdir_suffix_directory_named_dag_extension");
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .dag directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .dag directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "./bundle.dag",
        &dag_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn obligations_command_curdir_suffix_directory_named_mixed_case_dag_extension_is_invalid_single_file_target(
) {
    let root =
        unique_temp_dir("obligations_curdir_suffix_directory_named_mixed_case_dag_extension");
    let dag_dir = root.join("bundle.DaG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory root");
    std::fs::write(dag_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in .DaG directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "./bundle.DaG",
        &dag_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn obligations_command_symlink_directory_named_dag_extension_is_invalid_single_file_target() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("obligations_symlink_directory_named_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.dag");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .dag directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "link.dag",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn obligations_command_symlink_directory_named_mixed_case_dag_extension_is_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root =
        unique_temp_dir("obligations_symlink_directory_named_mixed_case_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DaG");
    std::fs::create_dir_all(&real_dir).expect("failed to create real directory root");
    std::fs::write(real_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .DaG directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "link.DaG",
        &link_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn obligations_command_curdir_suffix_symlink_named_dag_extension_is_invalid_single_file_target() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("obligations_curdir_suffix_symlink_named_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.dag");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .dag directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "./link.dag",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn obligations_command_curdir_suffix_symlink_named_uppercase_dag_extension_is_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root =
        unique_temp_dir("obligations_curdir_suffix_symlink_named_uppercase_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DAG");
    std::fs::create_dir_all(&real_dir).expect("failed to create real directory root");
    std::fs::write(real_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .DAG directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "obligations",
        &root,
        "./link.DAG",
        &link_dir,
        Some("broken.dag:2:3"),
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
    assert!(triplets.is_empty(), "transport-free graph should have no triplets");

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
fn show_triplets_command_curdir_suffix_target_matches_plain_relative_output() {
    let plain = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run plain-target daglang show-triplets");
    assert!(
        plain.status.success(),
        "plain-target show-triplets command failed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_suffix = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg("./dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run curdir-suffix-target daglang show-triplets");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix-target show-triplets command failed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix show-triplets stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix show-triplets stderr should match"
    );
}

#[test]
fn show_triplets_command_relative_and_absolute_targets_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-target daglang show-triplets");
    assert!(
        relative.status.success(),
        "relative-target show-triplets command failed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-target daglang show-triplets");
    assert!(
        absolute.status.success(),
        "absolute-target show-triplets command failed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute show-triplets stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute show-triplets stderr should match"
    );
}

#[test]
fn show_triplets_command_json_curdir_suffix_target_matches_plain_relative_output() {
    let plain = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg("dsl/tools/makegen.dag")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run plain-target daglang show-triplets --format json");
    assert!(
        plain.status.success(),
        "plain-target show-triplets --format json failed: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let curdir_suffix = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg("./dsl/tools/makegen.dag")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run curdir-suffix-target daglang show-triplets --format json");
    assert!(
        curdir_suffix.status.success(),
        "curdir-suffix-target show-triplets --format json failed: {}",
        String::from_utf8_lossy(&curdir_suffix.stderr)
    );

    assert_eq!(
        plain.stdout, curdir_suffix.stdout,
        "plain and curdir-suffix show-triplets json stdout should match"
    );
    assert_eq!(
        plain.stderr, curdir_suffix.stderr,
        "plain and curdir-suffix show-triplets json stderr should match"
    );
}

#[test]
fn show_triplets_command_json_relative_and_absolute_targets_are_equivalent() {
    let relative = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg("dsl/tools/makegen.dag")
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run relative-target daglang show-triplets --format json");
    assert!(
        relative.status.success(),
        "relative-target show-triplets --format json failed: {}",
        String::from_utf8_lossy(&relative.stderr)
    );

    let absolute = Command::new(daglang_bin())
        .arg("show-triplets")
        .arg(makegen_file())
        .arg("--format")
        .arg("json")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run absolute-target daglang show-triplets --format json");
    assert!(
        absolute.status.success(),
        "absolute-target show-triplets --format json failed: {}",
        String::from_utf8_lossy(&absolute.stderr)
    );

    assert_eq!(
        relative.stdout, absolute.stdout,
        "relative and absolute show-triplets json stdout should match"
    );
    assert_eq!(
        relative.stderr, absolute.stderr,
        "relative and absolute show-triplets json stderr should match"
    );
}

#[test]
fn show_triplets_command_directory_named_dag_extension_is_invalid_single_file_target() {
    let root = unique_temp_dir("show_triplets_directory_named_dag_extension");
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .dag directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .dag directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "bundle.dag",
        &dag_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn show_triplets_command_directory_named_mixed_case_dag_extension_is_invalid_single_file_target() {
    let root = unique_temp_dir("show_triplets_directory_named_mixed_case_dag_extension");
    let dag_dir = root.join("bundle.DaG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DaG directory root");
    std::fs::write(dag_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in .DaG directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "bundle.DaG",
        &dag_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn show_triplets_command_curdir_suffix_directory_named_dag_extension_is_invalid_single_file_target(
) {
    let root = unique_temp_dir("show_triplets_curdir_suffix_directory_named_dag_extension");
    let dag_dir = root.join("bundle.dag");
    std::fs::create_dir_all(dag_dir.join("sample")).expect("failed to create .dag directory root");
    std::fs::write(
        dag_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in .dag directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "./bundle.dag",
        &dag_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn show_triplets_command_curdir_suffix_directory_named_uppercase_dag_extension_is_invalid_single_file_target(
) {
    let root =
        unique_temp_dir("show_triplets_curdir_suffix_directory_named_uppercase_dag_extension");
    let dag_dir = root.join("bundle.DAG");
    std::fs::create_dir_all(&dag_dir).expect("failed to create .DAG directory root");
    std::fs::write(dag_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in .DAG directory");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "./bundle.DAG",
        &dag_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn show_triplets_command_symlink_directory_named_dag_extension_is_invalid_single_file_target() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("show_triplets_symlink_directory_named_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.dag");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .dag directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "link.dag",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn show_triplets_command_symlink_directory_named_uppercase_dag_extension_is_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root =
        unique_temp_dir("show_triplets_symlink_directory_named_uppercase_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DAG");
    std::fs::create_dir_all(&real_dir).expect("failed to create real directory root");
    std::fs::write(real_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .DAG directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "link.DAG",
        &link_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn show_triplets_command_curdir_suffix_symlink_named_dag_extension_is_invalid_single_file_target()
{
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("show_triplets_curdir_suffix_symlink_named_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.dag");
    std::fs::create_dir_all(real_dir.join("sample")).expect("failed to create real directory root");
    std::fs::write(
        real_dir.join("sample/main.dag"),
        "module sample.main\nfn run() -> Unit {}",
    )
    .expect("failed to write valid source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .dag directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "./link.dag",
        &link_dir,
        None,
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[cfg(unix)]
#[test]
fn show_triplets_command_curdir_suffix_symlink_named_mixed_case_dag_extension_is_invalid_single_file_target(
) {
    use std::os::unix::fs::symlink;

    let root =
        unique_temp_dir("show_triplets_curdir_suffix_symlink_named_mixed_case_dag_extension");
    let real_dir = root.join("real");
    let link_dir = root.join("link.DaG");
    std::fs::create_dir_all(&real_dir).expect("failed to create real directory root");
    std::fs::write(real_dir.join("broken.dag"), "module sample.broken\nfn")
        .expect("failed to write malformed source in real directory");
    symlink(&real_dir, &link_dir).expect("failed to create .DaG directory symlink");

    assert_single_target_command_treats_dag_directory_as_invalid_single_file_target(
        "show-triplets",
        &root,
        "./link.DaG",
        &link_dir,
        Some("broken.dag:2:3"),
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
}

#[test]
fn viz_command_renders_mermaid_for_compiled_file() {
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

    assert!(!output.status.success(), "broken source should fail compile");
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

    assert!(!output.status.success(), "lex-invalid file should fail compile");
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
    std::fs::write(root.join("sample/main.dag"), "module sample.main\nfn run() -> Unit { }")
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
    assert!(stderr.trim().is_empty(), "compile success should not emit stderr");
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
fn manifest_command_reports_diagnostics_for_invalid_file() {
    let broken = unique_temp_file("manifest_broken");
    std::fs::write(&broken, "module sample.broken\nfn broken( -> String {")
        .expect("failed to write broken source");

    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(&broken)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest for broken file");

    assert!(!output.status.success(), "broken source should fail manifest");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}

#[test]
fn expand_command_reports_unresolved_service_call_lower_error() {
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
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved service call"));
    assert!(stderr.contains("MissingStorage.read"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_reports_unresolved_uses_lower_error() {
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
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved used resource"));
    assert!(stderr.contains("fs: MissingResource"));

    std::fs::remove_file(fixture).expect("failed to cleanup fixture");
}

#[test]
fn expand_command_reports_unresolved_provides_lower_error() {
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
    assert_lower_stage_failure(&stderr);
    assert!(stderr.contains("lower error: unresolved provided resource"));
    assert!(stderr.contains("out: MissingResource"));

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
    assert!(stderr.contains("resource `Disk` is missing capability `write` for interface `Storage`"));

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
    assert!(stderr.contains("service `FsStorage` is missing operation `write` for interface `Storage`"));

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
fn compile_command_directory_mode_duplicate_resource_provides_also_reports_ambiguous_provided_type() {
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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unsatisfiable refinement on `Int`: range min 5 exceeds max 1"));

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
    assert_typecheck_stage_failure(&stderr);
    assert!(stderr.contains("unsatisfiable refinement on `Int`: range min 9 exceeds max 1"));

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
fn manifest_command_directory_mode_reports_module_path_mismatch() {
    let root = unique_temp_dir("manifest_path_mismatch");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(
        root.join("main.dag"),
        "module wrong.name\nfn run() -> Unit {}",
    )
    .expect("failed to write source");

    let output = Command::new(daglang_bin())
        .arg("manifest")
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang manifest on mismatch directory");

    assert!(
        !output.status.success(),
        "directory manifest should fail on module path mismatch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_stage_failures(&stderr);
    assert!(stderr.contains("module path mismatches"));
    assert!(stderr.contains("declared `wrong.name`"));

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}
