//! Mock specification for the makegen tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! - `execute_read_makegen`: Transport node that reads the existing Makefile
//!   - `response`: TransportResponse (file read result)
//! - `execute_makegen_transport`: Transport node that writes the Makefile (skippable)
//!   - `makegen_response`: TransportResponse
//!   - `makegen_written_path`: Path where Makefile was written
//!   - `makegen_content`: The generated Makefile content
//!   - `skip`: Bool (from compare_makegen_content)
//!   - `skip_reason`: String
//!
//! # Input Expectations
//!
//! - `path`: String path for Makefile generation
//! - `check_mode`: Optional bool, defaults to false

use crate::makegen::graph::build_makegen_graph;
use crate::WorkspaceBinary;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{CargoInvocation, Value};
use gunbc_primitives::filename;
use gunbc_test::{
    extract_mock_requirements, InputConstraint, MockSpec, NodeExample, OutputMatcher,
};

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

/// Mock specification for the makegen graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
///
/// Both transport mocks are required. Pure terminal outputs (load_registry.tool_count,
/// load_registry.tool_names) are computed during DryRun execution.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "makegen",
    builder = "crate::build_makegen_graph().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "makegen",
    output = "gunbc-dag/src/makegen/generated_tests.rs",
    module = "makegen_generated_tests",
    builder = "crate::build_makegen_graph().unwrap()",
    signature = "crate::makegen_signature()",
    tool = "makegen",
    flow_tests
)]
pub fn makegen_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "makegen")
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs_env should match type")
        // Transport: execute_read (file read) - mock the read response
        .transport_response(
            "execute_read_makegen",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some(mock_makefile_content()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read response should match type")
        // Transport: execute_makegen_transport (file write) - mock the write response
        .transport_response(
            "execute_makegen_transport",
            "makegen_response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: true,
                content: Some(mock_makefile_content()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_makegen_transport response should match type")
        .boundary_str(
            "execute_makegen_transport",
            "makegen_written_path",
            "Makefile",
        )
        .expect("execute_makegen_transport written_path should match type")
        .boundary_str(
            "execute_makegen_transport",
            "makegen_content",
            &mock_makefile_content(),
        )
        .expect("execute_makegen_transport content should match type")
        .boundary_bool("execute_makegen_transport", "skip", true)
        .expect("execute_write skip should match type")
        .boundary_str(
            "execute_makegen_transport",
            "skip_reason",
            "content is fresh — write skipped",
        )
        .expect("execute_write skip_reason should match type")
        // Build spec (pure terminal outputs are computed, not mocked)
        .build_unchecked()
        // Input mocks for DAG entry points (dangling inputs with no upstream edge)
        .input_mock(
            "prepare_read_makegen",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_write_makegen",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock("compare_makegen_content", "check_mode", Value::Bool(false))
        // Input expectations (via legacy API post-build)
        .expects_input("path", InputConstraint::Any)
        .expects_input("check_mode", InputConstraint::Any)
        // Resource: file write lock
        .resource_lock("fs:Makefile")
        // Expected outputs for verification
        .expected_output(
            "load_registry",
            "tool_count",
            Value::Int(
                crate::makegen::registry::ToolRegistry::default_registry()
                    .tools
                    .len() as i64,
            ),
        )
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("fs_env")
                .output("fs:write", OutputMatcher::Any)
                .description("Provides filesystem handle for Makefile writes"),
        )
        .node_example(
            NodeExample::new("load_registry")
                .output("tool_count", OutputMatcher::IntGe(2))
                .output("tool_names", OutputMatcher::non_empty())
                .description("Default registry loads with expected tools"),
        )
        .node_example(
            NodeExample::new("render_makefile")
                .output("makefile_content", OutputMatcher::contains("gist"))
                .description("Rendered Makefile contains gist target"),
        )
        // Primitive nodes — tested in their own crates
        .skip_node_example("prepare_read_makegen")
        .skip_node_example("prepare_write_makegen")
        .skip_node_example("compare_makegen_content")
}

/// Mock spec for testing no-change scenario.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn makegen_mock_spec_no_change() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    extract_mock_requirements(&dag, "makegen")
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs_env should match type")
        .transport_response(
            "execute_read_makegen",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some(mock_makefile_content()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read_makegen response should match type")
        .transport_response(
            "execute_makegen_transport",
            "makegen_response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: true,
                content: Some(mock_makefile_content()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_makegen_transport response should match type")
        .boundary_str(
            "execute_makegen_transport",
            "makegen_written_path",
            "Makefile",
        )
        .expect("execute_makegen_transport written_path should match type")
        .boundary_str(
            "execute_makegen_transport",
            "makegen_content",
            &mock_makefile_content(),
        )
        .expect("execute_makegen_transport content should match type")
        .boundary_bool("execute_makegen_transport", "skip", true)
        .expect("execute_makegen_transport skip should match type")
        .boundary_str(
            "execute_makegen_transport",
            "skip_reason",
            "content is fresh — write skipped",
        )
        .expect("execute_makegen_transport skip_reason should match type")
        .build_unchecked()
        .input_mock(
            "prepare_read_makegen",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_write_makegen",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock("compare_makegen_content", "check_mode", Value::Bool(false))
        .expects_input("path", InputConstraint::Any)
        .expects_input("check_mode", InputConstraint::Any)
}

/// Mock spec for testing file system failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn makegen_mock_spec_fs_fails() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    extract_mock_requirements(&dag, "makegen")
        .boundary("fs_env", "fs:write", mock_fs_handle())
        .expect("fs_env should match type")
        .transport_response(
            "execute_read_makegen",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file or directory".into()),
            }),
        )
        .expect("execute_read_makegen response should match type")
        .transport_response(
            "execute_makegen_transport",
            "makegen_response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: false,
                content: None,
                exists: Some(true),
                error: Some("Permission denied: Makefile is read-only".to_string()),
            }),
        )
        .expect("execute_makegen_transport response should match type")
        .boundary_str("execute_makegen_transport", "makegen_written_path", "")
        .expect("execute_makegen_transport written_path should match type")
        .boundary_str("execute_makegen_transport", "makegen_content", "")
        .expect("execute_makegen_transport content should match type")
        .boundary_bool("execute_makegen_transport", "skip", false)
        .expect("execute_makegen_transport skip should match type")
        .boundary_str("execute_makegen_transport", "skip_reason", "")
        .expect("execute_makegen_transport skip_reason should match type")
        .build_unchecked()
        .input_mock(
            "prepare_read_makegen",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_write_makegen",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock("compare_makegen_content", "check_mode", Value::Bool(false))
        .expects_input("path", InputConstraint::Any)
        .expects_input("check_mode", InputConstraint::Any)
        .resource_lock_fails("fs:Makefile", "Permission denied: Makefile is read-only")
}

/// Generate mock Makefile content.
fn mock_makefile_content() -> String {
    let gist = CargoInvocation::standalone("gist").command();
    let deps = CargoInvocation::standalone("deps").command();
    let makegen = WorkspaceBinary::Makegen.command();
    format!(
        "# Generated by gunbc-makegen\n\
         # DO NOT EDIT\n\
         \n\
         .PHONY: gist deps makegen\n\
         \n\
         gist:\n\
         \t@{gist} -- $(if $(REPO),--repo $(REPO))\n\
         \n\
         deps:\n\
         \t@{deps} -- $(if $(MANIFEST),--manifest $(MANIFEST))\n\
         \n\
         makegen:\n\
         \t@{makegen} -- $(if $(OUTPUT),--output $(OUTPUT))\n"
    )
}
