//! Mock specification for the makegen tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! - `execute_read`: Transport node that reads the existing Makefile
//!   - `response`: TransportResponse (file read result)
//! - `execute_write`: Transport node that writes the Makefile (skippable)
//!   - `response`: TransportResponse
//!   - `written_path`: Path where Makefile was written
//!   - `content`: The generated Makefile content
//!   - `skip`: Bool (from compare_content)
//!   - `skip_reason`: String
//!
//! # Input Expectations
//!
//! - `output_path`: Optional string, defaults to "Makefile"
//! - `check_mode`: Optional bool, defaults to false

use crate::makegen::graph::build_makegen_graph;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{CargoInvocation, Value};
use gunbc_test::{extract_mock_requirements, InputConstraint, MockSpec, NodeExample, OutputMatcher};

/// Mock specification for the makegen graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
///
/// Both transport mocks are required. Pure terminal outputs (load_registry.tool_count,
/// load_registry.tool_names) are computed during DryRun execution.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "makegen",
    output = "gunbc-dag/src/makegen/generated_tests.rs",
    module = "makegen_generated_tests",
    builder = "crate::build_makegen_graph().unwrap()",
    signature = "crate::makegen_signature()",
    flow_tests
)]
pub fn makegen_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "makegen")
        // Transport: execute_read (file read) - mock the read response
        .transport_response(
            "execute_read",
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
        // Transport: execute_write (file write) - mock the write response
        .transport_response(
            "execute_write",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: true,
                content: Some(mock_makefile_content()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_write response should match type")
        .boundary_str("execute_write", "written_path", "Makefile")
        .expect("execute_write written_path should match type")
        .boundary_str("execute_write", "content", &mock_makefile_content())
        .expect("execute_write content should match type")
        .boundary_bool("execute_write", "skip", true)
        .expect("execute_write skip should match type")
        .boundary_str("execute_write", "skip_reason", "content is fresh — write skipped")
        .expect("execute_write skip_reason should match type")
        // Build spec (pure terminal outputs are computed, not mocked)
        .build_unchecked()
        // Input mocks for DAG entry points (dangling inputs with no upstream edge)
        .input_mock("prepare_file_read", "path", Value::Str("Makefile".into()))
        .input_mock("prepare_file_write", "output_path", Value::Str("Makefile".into()))
        .input_mock("compare_content", "check_mode", Value::Bool(false))
        // Input expectations (via legacy API post-build)
        .expects_input("output_path", InputConstraint::Any)
        .expects_input("check_mode", InputConstraint::Any)
        // Resource: file write lock
        .resource_lock("fs:Makefile")
        // Expected outputs for verification
        .expected_output("load_registry", "tool_count", Value::Int(9))
        // Node I/O examples: verify pure node behavior
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
        .skip_node_example("prepare_file_read")
        .skip_node_example("prepare_file_write")
        .skip_node_example("compare_content")
}

/// Mock spec for testing no-change scenario.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn makegen_mock_spec_no_change() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    extract_mock_requirements(&dag, "makegen")
        .transport_response(
            "execute_read",
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
        .transport_response(
            "execute_write",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: true,
                content: Some(mock_makefile_content()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_write response should match type")
        .boundary_str("execute_write", "written_path", "Makefile")
        .expect("execute_write written_path should match type")
        .boundary_str("execute_write", "content", &mock_makefile_content())
        .expect("execute_write content should match type")
        .boundary_bool("execute_write", "skip", true)
        .expect("execute_write skip should match type")
        .boundary_str("execute_write", "skip_reason", "content is fresh — write skipped")
        .expect("execute_write skip_reason should match type")
        .build_unchecked()
        .input_mock("prepare_file_read", "path", Value::Str("Makefile".into()))
        .input_mock("prepare_file_write", "output_path", Value::Str("Makefile".into()))
        .input_mock("compare_content", "check_mode", Value::Bool(false))
        .expects_input("output_path", InputConstraint::Any)
        .expects_input("check_mode", InputConstraint::Any)
}

/// Mock spec for testing file system failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn makegen_mock_spec_fs_fails() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    extract_mock_requirements(&dag, "makegen")
        .transport_response(
            "execute_read",
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
        .expect("execute_read response should match type")
        .transport_response(
            "execute_write",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: false,
                content: None,
                exists: Some(true),
                error: Some("Permission denied: Makefile is read-only".to_string()),
            }),
        )
        .expect("execute_write response should match type")
        .boundary_str("execute_write", "written_path", "")
        .expect("execute_write written_path should match type")
        .boundary_str("execute_write", "content", "")
        .expect("execute_write content should match type")
        .boundary_bool("execute_write", "skip", false)
        .expect("execute_write skip should match type")
        .boundary_str("execute_write", "skip_reason", "")
        .expect("execute_write skip_reason should match type")
        .build_unchecked()
        .input_mock("prepare_file_read", "path", Value::Str("Makefile".into()))
        .input_mock("prepare_file_write", "output_path", Value::Str("Makefile".into()))
        .input_mock("compare_content", "check_mode", Value::Bool(false))
        .expects_input("output_path", InputConstraint::Any)
        .expects_input("check_mode", InputConstraint::Any)
        .resource_lock_fails("fs:Makefile", "Permission denied: Makefile is read-only")
}

/// Generate mock Makefile content.
fn mock_makefile_content() -> String {
    let gist = CargoInvocation::standalone("gist").command();
    let deps = CargoInvocation::standalone("deps").command();
    let makegen = CargoInvocation::composed("makegen", "dag").command();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_builder_rejects_wrong_slot() {
        let dag = build_makegen_graph().expect("graph should build");
        gunbc_test::assert_typed_builder_rejects_invalid_slot(&dag, "makegen");
    }
}
