//! Smoke tests for emitted binaries.
//!
//! These tests verify that the compiler's emitted Rust code actually
//! compiles and runs. Each test exercises a generated binary in
//! --dry-run mode and asserts it exits successfully.
//!
//! This catches a class of bugs that text-level emit tests miss:
//! syntactically broken emitted code, missing imports, type
//! mismatches in generated code, etc.
//!
//! These are NOT hermetic — they run real binaries. They live in
//! `tests/` (integration tests), not alongside unit tests.

use std::process::Command;

fn cargo_bin(name: &str) -> Command {
    // Use cargo to locate the binary in the workspace
    let mut cmd = Command::new(env!("CARGO"));
    cmd.args(["run", "--bin", name, "--"]);
    cmd
}

/// Run an emitted binary with the given subcommand + --dry-run.
/// Assert it exits 0 (no crash, no panic).
fn assert_dry_run(bin: &str, subcommand: &str) {
    let output = cargo_bin(bin)
        .args([subcommand, "--dry-run"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run {bin} {subcommand}: {e}"));

    assert!(
        output.status.success(),
        "{bin} {subcommand} --dry-run failed with status {}:\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Run an emitted binary with --help. Assert it exits 0.
fn assert_help(bin: &str) {
    let output = cargo_bin(bin)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("failed to run {bin} --help: {e}"));

    assert!(
        output.status.success(),
        "{bin} --help failed with status {}:\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn gist_help() {
    assert_help("gunbc-gist");
}

#[test]
fn gist_dry_run() {
    assert_dry_run("gunbc-gist", "gist");
}

#[test]
fn gist_diff_dry_run() {
    assert_dry_run("gunbc-gist", "gist-diff");
}

#[test]
fn gist_recent_dry_run() {
    assert_dry_run("gunbc-gist", "gist-recent");
}

#[test]
fn bootstrap_help() {
    assert_help("gunbc-bootstrap");
}

#[test]
fn build_all_help() {
    assert_help("gunbc-build-all");
}

#[test]
fn readme_help() {
    assert_help("gunbc-readme");
}

#[test]
fn codegen_dag_help() {
    assert_help("gunbc-codegen-dag");
}
