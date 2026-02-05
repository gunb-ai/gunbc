//! Mock specification for the makegen tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! - `execute_transport`: Transport node that writes the Makefile
//!   - `response`: TransportResponse
//!   - `written_path`: Path where Makefile was written
//!   - `content`: The generated Makefile content
//!
//! # Input Expectations
//!
//! - `output_path`: Optional string, defaults to "Makefile"

use crate::makegen::graph::build_makegen_graph;
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::{CargoInvocation, Value};
use gunbc_test::{extract_mock_requirements, InputConstraint, MockSpec, NodeExample, OutputMatcher};

/// Mock specification for the makegen graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
///
/// Only transport mocks are required. Pure terminal outputs (load_registry.tool_count,
/// load_registry.tool_names) are computed during DryRun execution.
pub fn makegen_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "makegen")
        // Transport: execute_transport (file write) - all outputs need mocks
        .transport_response(
            "execute_transport",
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
        .expect("execute_transport response should match type")
        .boundary_str("execute_transport", "written_path", "Makefile")
        .expect("execute_transport written_path should match type")
        .boundary_str("execute_transport", "content", &mock_makefile_content())
        .expect("execute_transport content should match type")
        // Build spec (pure terminal outputs are computed, not mocked)
        .build_unchecked()
        // Input expectations (via legacy API post-build)
        .expects_input("output_path", InputConstraint::Any)
        // Resource: file write lock
        .resource_lock("fs:Makefile")
        // Expected outputs for verification
        .expected_output("load_registry", "tool_count", Value::Int(7))
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
        .skip_node_example("prepare_file_write")
}

/// Mock spec for testing no-change scenario.
pub fn makegen_mock_spec_no_change() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    extract_mock_requirements(&dag, "makegen")
        .transport_response(
            "execute_transport",
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
        .expect("execute_transport response should match type")
        .boundary_str("execute_transport", "written_path", "Makefile")
        .expect("execute_transport written_path should match type")
        .boundary_str("execute_transport", "content", &mock_makefile_content())
        .expect("execute_transport content should match type")
        .build_unchecked()
        .expects_input("output_path", InputConstraint::Any)
}

/// Mock spec for testing file system failure.
pub fn makegen_mock_spec_fs_fails() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_makegen_graph().expect("makegen graph should build");

    extract_mock_requirements(&dag, "makegen")
        .transport_response(
            "execute_transport",
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
        .expect("execute_transport response should match type")
        .boundary_str("execute_transport", "written_path", "")
        .expect("execute_transport written_path should match type")
        .boundary_str("execute_transport", "content", "")
        .expect("execute_transport content should match type")
        .build_unchecked()
        .expects_input("output_path", InputConstraint::Any)
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
        let reqs = extract_mock_requirements(&dag, "makegen");

        // Try to set a mock for a non-existent node
        let result = reqs.boundary_str("nonexistent_node", "port", "value");
        assert!(result.is_err());
    }
}
