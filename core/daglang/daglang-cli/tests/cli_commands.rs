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
