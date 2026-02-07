//! Mock specification for the pragma tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! Six transport nodes need mocks:
//! - `execute_read_clippy`: Reads existing clippy.toml
//! - `execute_clippy_transport`: Writes clippy.toml (skippable)
//! - `execute_read_allowlist`: Reads existing allowlist
//! - `execute_allowlist_transport`: Writes allowlist (skippable)
//! - `execute_read_policy`: Reads existing lint policy
//! - `execute_policy_transport`: Writes lint policy (skippable)

use crate::pragma::graph::build_pragma_graph;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_primitives::filename;
use gunbc_test::{extract_mock_requirements, MockSpec, NodeExample, OutputMatcher};

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

/// Mock specification for the pragma graph.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "pragma",
    builder = "crate::build_pragma_graph().unwrap()",
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "pragma",
    output = "gunbc-dag/src/pragma/generated_tests.rs",
    module = "pragma_generated_tests",
    builder = "crate::build_pragma_graph().unwrap()",
    signature = "crate::pragma_signature()",
    flow_tests
)]
pub fn pragma_mock_spec() -> MockSpec {
    let dag = build_pragma_graph().expect("pragma graph should build");

    extract_mock_requirements(&dag, "pragma")
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs_env should match type")
        // Transport: execute_read_clippy
        .transport_response(
            "execute_read_clippy",
            "response",
            TransportResponse::File(FileResponse {
                path: "clippy.toml".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock-clippy>".into()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read_clippy response should match type")
        // Transport: execute_clippy_transport
        .transport_response(
            "execute_clippy_transport",
            "clippy_response",
            TransportResponse::File(FileResponse {
                path: "clippy.toml".into(),
                operation: FileOp::Write,
                success: true,
                content: Some("<mock-clippy>".into()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_clippy_transport response should match type")
        .boundary_str("execute_clippy_transport", "clippy_written_path", "clippy.toml")
        .expect("execute_clippy_transport path should match type")
        .boundary_str("execute_clippy_transport", "clippy_content", "<mock-clippy>")
        .expect("execute_clippy_transport content should match type")
        .boundary_bool("execute_clippy_transport", "skip", false)
        .expect("execute_clippy_transport skip should match type")
        .boundary_str("execute_clippy_transport", "skip_reason", "")
        .expect("execute_clippy_transport skip_reason should match type")
        // Transport: execute_read_allowlist
        .transport_response(
            "execute_read_allowlist",
            "response",
            TransportResponse::File(FileResponse {
                path: "tools/disallowed-methods-allowlist.txt".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock-allowlist>".into()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read_allowlist response should match type")
        // Transport: execute_allowlist_transport
        .transport_response(
            "execute_allowlist_transport",
            "allowlist_response",
            TransportResponse::File(FileResponse {
                path: "tools/disallowed-methods-allowlist.txt".into(),
                operation: FileOp::Write,
                success: true,
                content: Some("<mock-allowlist>".into()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_allowlist_transport response should match type")
        .boundary_str("execute_allowlist_transport", "allowlist_written_path", "tools/disallowed-methods-allowlist.txt")
        .expect("execute_allowlist_transport path should match type")
        .boundary_str("execute_allowlist_transport", "allowlist_content", "<mock-allowlist>")
        .expect("execute_allowlist_transport content should match type")
        .boundary_bool("execute_allowlist_transport", "skip", false)
        .expect("execute_allowlist_transport skip should match type")
        .boundary_str("execute_allowlist_transport", "skip_reason", "")
        .expect("execute_allowlist_transport skip_reason should match type")
        // Transport: execute_read_policy
        .transport_response(
            "execute_read_policy",
            "response",
            TransportResponse::File(FileResponse {
                path: "tools/pragma-lint-policy.txt".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock-policy>".into()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read_policy response should match type")
        // Transport: execute_policy_transport
        .transport_response(
            "execute_policy_transport",
            "policy_response",
            TransportResponse::File(FileResponse {
                path: "tools/pragma-lint-policy.txt".into(),
                operation: FileOp::Write,
                success: true,
                content: Some("<mock-policy>".into()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_policy_transport response should match type")
        .boundary_str("execute_policy_transport", "policy_written_path", "tools/pragma-lint-policy.txt")
        .expect("execute_policy_transport path should match type")
        .boundary_str("execute_policy_transport", "policy_content", "<mock-policy>")
        .expect("execute_policy_transport content should match type")
        .boundary_bool("execute_policy_transport", "skip", false)
        .expect("execute_policy_transport skip should match type")
        .boundary_str("execute_policy_transport", "skip_reason", "")
        .expect("execute_policy_transport skip_reason should match type")
        // Build spec
        .build_unchecked()
        // Input mocks for DAG entry points
        .input_mock("prepare_read_clippy", "path", Value::Str("clippy.toml".into()))
        .input_mock("prepare_read_allowlist", "path", Value::Str("tools/disallowed-methods-allowlist.txt".into()))
        .input_mock("prepare_read_policy", "path", Value::Str("tools/pragma-lint-policy.txt".into()))
        .input_mock("prepare_write_clippy", "path", Value::Str("clippy.toml".into()))
        .input_mock("prepare_write_allowlist", "path", Value::Str("tools/disallowed-methods-allowlist.txt".into()))
        .input_mock("prepare_write_policy", "path", Value::Str("tools/pragma-lint-policy.txt".into()))
        .input_mock("compare_clippy_content", "check_mode", Value::Bool(false))
        .input_mock("compare_allowlist_content", "check_mode", Value::Bool(false))
        .input_mock("compare_policy_content", "check_mode", Value::Bool(false))
        // Resources: file locks for all outputs
        .resource_lock("fs:clippy.toml")
        .resource_lock("fs:tools/disallowed-methods-allowlist.txt")
        .resource_lock("fs:tools/pragma-lint-policy.txt")
        // Node I/O examples
        .node_example(
            NodeExample::new("fs_env")
                .output("fs:write", OutputMatcher::Any)
                .description("Provides filesystem handle for pragma writes"),
        )
        .node_example(
            NodeExample::new("render_clippy")
                .output("content", OutputMatcher::contains("disallowed-methods"))
                .description("Renders clippy.toml with disallowed methods config"),
        )
        .node_example(
            NodeExample::new("render_allowlist")
                .output("content", OutputMatcher::contains("Generated by gunbc-pragma"))
                .description("Renders disallowed-methods allowlist"),
        )
        .node_example(
            NodeExample::new("render_policy")
                .output("content", OutputMatcher::contains("Generated by gunbc-pragma"))
                .description("Renders pragma lint policy"),
        )
        // Primitive nodes — tested in their own crates
        .skip_node_example("prepare_read_clippy")
        .skip_node_example("prepare_write_clippy")
        .skip_node_example("compare_clippy_content")
        .skip_node_example("prepare_read_allowlist")
        .skip_node_example("prepare_write_allowlist")
        .skip_node_example("compare_allowlist_content")
        .skip_node_example("prepare_read_policy")
        .skip_node_example("prepare_write_policy")
        .skip_node_example("compare_policy_content")
}
