//! Mock specification for the clippy tool.
//!
//! This file declares the mocks used by testgen for the clippy DAG.
//! The clippy upsert is represented as a flat DAG: check → create → resolve.

use gunbc_ir::Value;
use gunbc_test::{InputConstraint, MockSpec};
use std::collections::BTreeMap;

fn mock_cli_result() -> Value {
    let mut map = BTreeMap::new();
    map.insert("success".to_string(), Value::Bool(true));
    map.insert("exit_code".to_string(), Value::Int(0));
    map.insert("stdout".to_string(), Value::Str(String::new()));
    map.insert("stderr".to_string(), Value::Str(String::new()));
    Value::Map(map)
}

/// Mock specification for the clippy DAG.
///
/// The check node is mocked to return `exists = true` so the create node
/// is skipped during DryRun. The resolve node returns a mock CliResult.
#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "clippy",
    builder = "crate::build_clippy_graph_lint_all()",
)]
#[gunbc_testgen_registry_macros::testgen_target(skip, builder = "crate::build_clippy_graph_lint_all()")]
pub fn clippy_mock_spec() -> MockSpec {
    MockSpec::new("clippy")
        // Mock check.exists so create is skipped.
        .boundary("check", "exists", Value::Bool(true))
        // Mock resolve.result so the DAG has a concrete output.
        .boundary("resolve", "result", mock_cli_result())
        // Entry inputs (unit trigger) for isolated DAG execution.
        .input_mock("check", "trigger", Value::Unit)
        .input_mock("create", "trigger", Value::Unit)
        .input_mock("resolve", "trigger", Value::Unit)
        // Document the expected external input.
        .expects_input("trigger", InputConstraint::Any)
        // Skip node examples (these nodes are exercised via DAG-level tests).
        .skip_node_example("resource_gate")
        .skip_node_example("check")
        .skip_node_example("create")
        .skip_node_example("resolve")
}
