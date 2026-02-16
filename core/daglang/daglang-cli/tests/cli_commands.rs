use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

fn unique_temp_file(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("daglang_cli_{name}_{}_{}.dag", std::process::id(), nanos))
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("daglang_cli_{name}_{}_{}", std::process::id(), nanos))
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK: parsed 42 file(s)"),
        "unexpected check output: {stdout}"
    );
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Discovered modules:"));
    assert!(stdout.contains("tools.makegen"));
    assert!(stdout.contains("std.types"));
}

#[test]
fn viz_self_renders_pipeline_mermaid() {
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("flowchart TB"));
    assert!(stdout.contains("discover_files"));
    assert!(stdout.contains("report_modules"));
}

#[test]
fn viz_self_output_is_deterministic_for_same_input() {
    let first = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run first daglang viz --self");
    assert!(first.status.success(), "first viz --self run should succeed");

    let second = Command::new(daglang_bin())
        .arg("viz")
        .arg("--self")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run second daglang viz --self");
    assert!(second.status.success(), "second viz --self run should succeed");

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
        stdout.contains("files:files"),
        "viz --self should include discover->parse files edge label"
    );
    assert!(
        stdout.contains("diagnostics:diagnostics"),
        "viz --self should include diagnostics flow edges"
    );
    assert!(
        stdout.contains("module_graph:module_graph"),
        "viz --self should include build->report module graph edge label"
    );
}

#[test]
fn check_command_reports_file_line_col_for_broken_file() {
    let broken_file = unique_temp_file("broken");
    std::fs::write(&broken_file, "module tmp.bad\nfn broken( -> String {\n  \"oops\"\n}\n")
        .expect("failed to create broken dag file");

    let output = Command::new(daglang_bin())
        .arg("check")
        .arg(&broken_file)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check for broken file");

    assert!(!output.status.success(), "broken file should fail check");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(":2:12:"),
        "expected file:line:col in stderr, got: {stderr}"
    );

    std::fs::remove_file(broken_file).expect("failed to remove broken dag file");
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
    let root_file = unique_temp_file("check_non_directory_root")
        .with_extension("txt");
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OK: parsed 1 file(s)"));

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
    assert!(
        stdout.contains("OK: parsed 0 file(s)"),
        "expected empty directory to parse zero files: {stdout}"
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
        stdout.contains("OK: parsed 0 file(s)"),
        "non-.dag files should be ignored during check: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_ignores_non_dag_files_when_dag_files_exist() {
    let root = unique_temp_dir("check_ignore_non_dag_mixed");
    std::fs::create_dir_all(&root).expect("failed to create temp dir");
    std::fs::write(root.join("good.dag"), "module sample.good\nfn ok() -> Unit {}")
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
        stdout.contains("OK: parsed 1 file(s)"),
        "expected only the .dag file to be parsed: {stdout}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_does_not_run_resolve_stage_for_unresolved_imports() {
    let root = unique_temp_dir("check_parse_only_unresolved_import");
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
        output.status.success(),
        "check should parse successfully without resolve-stage diagnostics"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("OK: parsed 1 file(s)"));
    assert!(
        stderr.trim().is_empty(),
        "check should not emit resolve diagnostics: {stderr}"
    );

    std::fs::remove_dir_all(root).expect("failed to cleanup temp dir");
}

#[test]
fn check_command_defaults_to_workspace_dsl_root() {
    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang check with default root");

    assert!(
        output.status.success(),
        "default check command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("OK: parsed 42 file(s)"),
        "default check should parse full DSL corpus"
    );
}

#[test]
fn check_command_default_root_missing_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("check_default_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");

    let output = Command::new(daglang_bin())
        .arg("check")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang check with missing default root");

    assert!(
        !output.status.success(),
        "default check should fail when cwd lacks dsl/ root"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));

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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Discovered modules:"));
    assert!(stdout.contains("std.types"));
}

#[test]
fn modules_command_default_root_missing_in_cwd_exits_nonzero() {
    let cwd = unique_temp_dir("modules_default_missing_root");
    std::fs::create_dir_all(&cwd).expect("failed to create temp cwd");

    let output = Command::new(daglang_bin())
        .arg("modules")
        .current_dir(&cwd)
        .output()
        .expect("failed to run daglang modules with missing default root");

    assert!(
        !output.status.success(),
        "default modules should fail when cwd lacks dsl/ root"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root does not exist"));

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
    std::fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
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
    let root_file = unique_temp_file("modules_non_directory_root")
        .with_extension("txt");
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

    assert!(
        !output.status.success(),
        "modules should fail when given a file path instead of directory root"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pipeline error"));
    assert!(stderr.contains("input root is not a directory"));
    assert!(
        stderr.contains(&file_path.display().to_string()),
        "single-file-root error should include offending path: {stderr}"
    );

    std::fs::remove_file(file_path).expect("failed to cleanup .dag file");
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown command"));
}

#[test]
fn placeholder_commands_remain_non_blocking() {
    for command in ["expand", "manifest", "compile"] {
        let output = Command::new(daglang_bin())
            .arg(command)
            .arg("dsl/tools/makegen.dag")
            .current_dir(workspace_root())
            .output()
            .unwrap_or_else(|err| panic!("failed to run {command}: {err}"));
        assert!(
            output.status.success(),
            "{command} placeholder command should remain non-blocking"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("TODO"),
            "{command} placeholder should make TODO status explicit"
        );
    }
}

#[test]
fn viz_without_self_is_non_blocking_placeholder() {
    let output = Command::new(daglang_bin())
        .arg("viz")
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang viz placeholder mode");

    assert!(
        output.status.success(),
        "viz placeholder mode should remain non-blocking"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("TODO"),
        "viz placeholder mode should surface TODO guidance"
    );
}
