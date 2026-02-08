//! Mock specification for the clippy tool.
//!
//! This file declares the mocks used by testgen for the clippy DAG.
//! The clippy upsert uses transport triplets:
//! prepare_check → execute_check → parse_check
//! prepare_install → execute_install → parse_install
//! prepare_resolve → execute_resolve → parse_resolve

use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_test::{InputConstraint, MockSpec};

/// Mock specification for the clippy DAG.
///
/// Transport execute nodes are mocked with ShellResponses.
/// The check response indicates tool exists, so install is skipped.
/// The resolve response returns a successful run result.
#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "clippy",
    builder = "crate::build_clippy_graph_lint_all()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    skip,
    builder = "crate::build_clippy_graph_lint_all()"
)]
pub fn clippy_mock_spec() -> MockSpec {
    MockSpec::new("clippy")
        // Transport mocks: mock the execute nodes with ShellResponses
        .boundary(
            "execute_check",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok("clippy 0.1.0"))),
        )
        .boundary(
            "execute_install",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok(""))),
        )
        .boundary(
            "execute_resolve",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok(""))),
        )
        // Boundary outputs: parse_resolve.result is the final DAG output
        .boundary(
            "parse_resolve",
            "result",
            Value::Map(
                vec![
                    ("success".to_string(), Value::Bool(true)),
                    ("exit_code".to_string(), Value::Int(0)),
                    ("stdout".to_string(), Value::Str(String::new())),
                    ("stderr".to_string(), Value::Str(String::new())),
                ]
                .into_iter()
                .collect(),
            ),
        )
        // Entry inputs (unit trigger) for isolated DAG execution.
        .input_mock("prepare_check", "trigger", Value::Unit)
        // Document the expected external input.
        .expects_input("trigger", InputConstraint::Any)
        // Skip node examples (these nodes are exercised via DAG-level tests).
        .skip_node_example("resource_gate")
        .skip_node_example("prepare_check")
        .skip_node_example("execute_check")
        .skip_node_example("parse_check")
        .skip_node_example("prepare_install")
        .skip_node_example("execute_install")
        .skip_node_example("parse_install")
        .skip_node_example("prepare_resolve")
        .skip_node_example("execute_resolve")
        .skip_node_example("parse_resolve")
}
