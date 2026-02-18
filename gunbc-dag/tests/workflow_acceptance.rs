//! Workflow acceptance-contract tests.
//!
//! These tests pin the intended CI workflow command contracts so refactors do
//! not silently reintroduce duplicate compile paths or drift from verify mode.

use gunbc_dag::build_ci_graph;
use gunbc_exec::{execute_single_node, lower, ExecutionMode};
use gunbc_ir::transport::{ShellRequest, TransportRequest};
use gunbc_ir::Value;
use std::collections::HashMap;

fn bool_inputs(items: &[(&str, bool)]) -> HashMap<String, Value> {
    items
        .iter()
        .map(|(k, v)| ((*k).to_string(), Value::Bool(*v)))
        .collect()
}

fn prepare_shell_request(
    node: &str,
    inputs: HashMap<String, Value>,
) -> Result<ShellRequest, Box<dyn std::error::Error>> {
    let dag = build_ci_graph()?;
    let lowered = lower(&dag)?;
    let outputs = execute_single_node(&lowered.dag, node, inputs, ExecutionMode::Real)?;
    let request = outputs
        .get("request")
        .and_then(|v| v.as_request())
        .ok_or_else(|| format!("node '{node}' did not produce request output"))?;

    match request {
        TransportRequest::Shell(shell) => Ok(shell),
        other => Err(format!("node '{node}' produced non-shell request: {other:?}").into()),
    }
}

fn prepare_outputs(
    node: &str,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let dag = build_ci_graph()?;
    let lowered = lower(&dag)?;
    Ok(execute_single_node(
        &lowered.dag,
        node,
        inputs,
        ExecutionMode::Real,
    )?)
}

#[test]
fn ci_build_stage_compiles_tests_without_running_them() {
    let shell = prepare_shell_request(
        "build/prepare_build",
        bool_inputs(&[("prep_success", true), ("testgen_success", true)]),
    )
    .expect("prepare_build should produce shell request");

    assert_eq!(shell.command, "cargo");
    assert_eq!(shell.args, vec!["test", "--no-run"]);
    assert_eq!(shell.env.get("RUSTFLAGS"), Some(&"-D warnings".to_string()));
}

#[test]
fn ci_test_stage_runs_tests_after_build() {
    let shell = prepare_shell_request("test/prepare_test", bool_inputs(&[("build_success", true)]))
        .expect("prepare_test should produce shell request");

    assert_eq!(shell.command, "cargo");
    assert_eq!(shell.args, vec!["test"]);
    assert!(!shell.args.iter().any(|arg| arg == "--no-run"));
}

#[test]
fn ci_test_stage_skips_when_build_fails() {
    let outputs = prepare_outputs(
        "test/prepare_test",
        bool_inputs(&[("build_success", false)]),
    )
    .expect("prepare_test should execute");

    assert_eq!(outputs.get("skip").and_then(|v| v.as_bool()), Some(true));
    assert!(!outputs.contains_key("request"));
    assert!(outputs
        .get("skip_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("build failure"));
}

#[test]
fn ci_guardrail_stage_runs_disallowed_methods_and_resource_purity_checks() {
    let shell = prepare_shell_request(
        "guardrail_check/prepare_guardrail_check",
        bool_inputs(&[("testgen_success", true), ("pragma_success", true)]),
    )
    .expect("prepare_guardrail_check should produce shell request");

    assert_eq!(shell.command, "bash");
    assert_eq!(shell.args.first().map(String::as_str), Some("-lc"));
    let command = shell
        .args
        .get(1)
        .expect("bash -lc command should be present");
    assert!(
        !command.contains("tools/check-disallowed-methods.sh"),
        "guardrail should no longer invoke deprecated shell script"
    );
    assert!(command.contains("cargo test -p gunbc-dag --test resource_purity_checks"));
}

#[test]
fn ci_guardrail_stage_skips_when_upstream_fails() {
    let outputs = prepare_outputs(
        "guardrail_check/prepare_guardrail_check",
        bool_inputs(&[("testgen_success", false), ("pragma_success", true)]),
    )
    .expect("prepare_guardrail_check should execute");

    assert_eq!(outputs.get("skip").and_then(|v| v.as_bool()), Some(true));
    assert!(!outputs.contains_key("request"));
    assert!(outputs
        .get("skip_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("testgen failure"));
}

#[test]
fn ci_verify_stage_uses_verify_mode_commands() {
    let shell = prepare_shell_request(
        "verify_makegen_check/prepare_verify_makegen_check",
        bool_inputs(&[
            ("prep_success", true),
            ("bootstrap_success", true),
            ("testgen_success", true),
            ("pragma_success", true),
        ]),
    )
    .expect("prepare_verify_makegen_check should produce shell request");

    assert_eq!(shell.command, "cargo");
    assert_eq!(
        shell.args,
        vec![
            "run",
            "-p",
            "gunbc-dag",
            "--bin",
            "gunbc-makegen",
            "--",
            "--mode=verify",
        ]
    );
    assert_eq!(shell.env.get("RUSTFLAGS"), Some(&"-D warnings".to_string()));
}

#[test]
fn ci_verify_stage_skips_when_prep_fails() {
    let outputs = prepare_outputs(
        "verify_makegen_check/prepare_verify_makegen_check",
        bool_inputs(&[
            ("prep_success", false),
            ("bootstrap_success", true),
            ("testgen_success", true),
            ("pragma_success", true),
        ]),
    )
    .expect("prepare_verify_makegen_check should execute");

    assert_eq!(outputs.get("skip").and_then(|v| v.as_bool()), Some(true));
    assert!(!outputs.contains_key("request"));
    assert!(outputs
        .get("skip_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("codegen failure"));
}
