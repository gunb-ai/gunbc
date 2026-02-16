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
