use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled 1 module(s)"));
    assert!(stdout.contains("target/generated/rust/main.rs"));
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ProgressManifest:"));
    assert!(stdout.contains("total_nodes:"));
    assert!(stdout.contains("waves:"));
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
    assert!(stderr.contains("compile diagnostics"));
    assert!(stderr.contains(":2:"));

    std::fs::remove_file(broken).expect("failed to remove temp broken source");
}
