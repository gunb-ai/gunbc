//! Mock specification for the bootstrap tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! Three transport nodes need mocks:
//! - `execute_scan_workspace`: Scans workspace for crates
//! - `execute_makefile_transport`: Writes Makefile
//! - `execute_gitignore_transport`: Writes .gitignore
//!
//! # Input Expectations
//!
//! No inputs - bootstrap scans the workspace automatically.
//!
//! # Resource Simulations
//!
//! - File locks for Makefile and .gitignore

use crate::bootstrap::graph::build_bootstrap_graph;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_test::{extract_mock_requirements, MockSpec, NodeExample, OutputMatcher};

/// Mock specification for the bootstrap graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
pub fn bootstrap_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "bootstrap")
        // Transport: execute_scan_workspace (workspace scan)
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        // Transport: execute_makefile_transport (write Makefile)
        .transport_response(
            "execute_makefile_transport",
            "makefile_response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: true,
                content: Some("<mock-makefile>".into()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_makefile_transport response should match type")
        .boundary_str("execute_makefile_transport", "makefile_written_path", "Makefile")
        .expect("execute_makefile_transport path should match type")
        .boundary_str("execute_makefile_transport", "makefile_content", "<mock-makefile>")
        .expect("execute_makefile_transport content should match type")
        // Transport: execute_gitignore_transport (write .gitignore)
        .transport_response(
            "execute_gitignore_transport",
            "gitignore_response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Write,
                success: true,
                content: Some("<mock-gitignore>".into()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_gitignore_transport response should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_written_path", ".gitignore")
        .expect("execute_gitignore_transport path should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_content", "<mock-gitignore>")
        .expect("execute_gitignore_transport content should match type")
        // Build spec (pure terminal outputs are computed, not mocked)
        .build_unchecked()
        // Resources: file locks for both outputs
        .resource_lock("fs:Makefile")
        .resource_lock("fs:.gitignore")
        // Expected outputs: verified after DryRun execution
        .expected_output("parse_scan_result", "crate_count", Value::Int(2))
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("prepare_scan_workspace")
                .output("request", OutputMatcher::non_empty())
                .description("Prepares a workspace scan transport request"),
        )
        .node_example(
            NodeExample::new("parse_scan_result")
                .input(
                    "response",
                    Value::Response(ShellResponse::ok("crates/bar\ncrates/foo\n").into()),
                )
                .output("crate_count", OutputMatcher::exact(Value::Int(2)))
                .output(
                    "crate_names",
                    OutputMatcher::exact(Value::str_list(vec!["bar".into(), "foo".into()])),
                )
                .description("Parses shell stdout to extract sorted crate names and count"),
        )
        .node_example(
            NodeExample::new("parse_scan_result")
                .input("response", Value::Skipped)
                .output("crate_count", OutputMatcher::Any)
                .output("crate_names", OutputMatcher::Any)
                .description("Handles skipped transport response gracefully"),
        )
        .node_example(
            NodeExample::new("generate_makefile")
                .output(
                    "makefile_content",
                    OutputMatcher::contains("Generated by gunbc-makegen"),
                )
                .description("Generates Makefile content from registry"),
        )
        .node_example(
            NodeExample::new("generate_gitignore")
                .output(
                    "gitignore_content",
                    OutputMatcher::contains("Generated by gunbc-makegen"),
                )
                .description("Generates .gitignore content from build config"),
        )
        // Primitive nodes — tested in their own crates
        .skip_node_example("prepare_makefile_write")
        .skip_node_example("prepare_gitignore_write")
}

/// Mock spec for testing single file write (Makefile only).
pub fn bootstrap_mock_spec_makefile_only() -> MockSpec {
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    extract_mock_requirements(&dag, "bootstrap")
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        .transport_response(
            "execute_makefile_transport",
            "makefile_response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: true,
                content: Some("<mock>".into()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_makefile_transport response should match type")
        .boundary_str("execute_makefile_transport", "makefile_written_path", "Makefile")
        .expect("path should match type")
        .boundary_str("execute_makefile_transport", "makefile_content", "<mock>")
        .expect("content should match type")
        .transport_response(
            "execute_gitignore_transport",
            "gitignore_response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Write,
                success: false, // gitignore write skipped
                content: None,
                exists: Some(false),
                error: None,
            }),
        )
        .expect("execute_gitignore_transport response should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_written_path", "")
        .expect("path should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_content", "")
        .expect("content should match type")
        .build_unchecked()
        .resource_lock("fs:Makefile")
}

/// Mock spec for testing file system failure on Makefile.
pub fn bootstrap_mock_spec_makefile_fails() -> MockSpec {
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    extract_mock_requirements(&dag, "bootstrap")
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        .transport_response(
            "execute_makefile_transport",
            "makefile_response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: false,
                content: None,
                exists: Some(true),
                error: Some("Permission denied: Makefile is read-only".into()),
            }),
        )
        .expect("execute_makefile_transport response should match type")
        .boundary_str("execute_makefile_transport", "makefile_written_path", "")
        .expect("path should match type")
        .boundary_str("execute_makefile_transport", "makefile_content", "")
        .expect("content should match type")
        .transport_response(
            "execute_gitignore_transport",
            "gitignore_response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Write,
                success: true,
                content: Some("<mock>".into()),
                exists: Some(true),
                error: None,
            }),
        )
        .expect("execute_gitignore_transport response should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_written_path", ".gitignore")
        .expect("path should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_content", "<mock>")
        .expect("content should match type")
        .build_unchecked()
        .resource_lock_fails("fs:Makefile", "Permission denied: Makefile is read-only")
        .resource_lock("fs:.gitignore")
}

/// Mock spec for testing complete write failure.
pub fn bootstrap_mock_spec_all_fail() -> MockSpec {
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    extract_mock_requirements(&dag, "bootstrap")
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        .transport_response(
            "execute_makefile_transport",
            "makefile_response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Write,
                success: false,
                content: None,
                exists: Some(true),
                error: Some("Permission denied".into()),
            }),
        )
        .expect("execute_makefile_transport response should match type")
        .boundary_str("execute_makefile_transport", "makefile_written_path", "")
        .expect("path should match type")
        .boundary_str("execute_makefile_transport", "makefile_content", "")
        .expect("content should match type")
        .transport_response(
            "execute_gitignore_transport",
            "gitignore_response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Write,
                success: false,
                content: None,
                exists: Some(true),
                error: Some("Permission denied".into()),
            }),
        )
        .expect("execute_gitignore_transport response should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_written_path", "")
        .expect("path should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_content", "")
        .expect("content should match type")
        .build_unchecked()
        .resource_lock_fails("fs:Makefile", "Permission denied")
        .resource_lock_fails("fs:.gitignore", "Permission denied")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Mock spec tests (Pattern B - mock value properties)
    // ========================================================================

    #[test]
    fn test_mock_spec_has_transport_mocks() {
        let spec = bootstrap_mock_spec();
        // All three transport mocks should be present
        assert!(spec
            .get_transport_mock("execute_scan_workspace", "response")
            .is_some());
        assert!(spec
            .get_transport_mock("execute_makefile_transport", "makefile_response")
            .is_some());
        assert!(spec
            .get_transport_mock("execute_gitignore_transport", "gitignore_response")
            .is_some());
    }

    #[test]
    fn test_mock_spec_has_expected_output() {
        let spec = bootstrap_mock_spec();
        let has_expected = spec
            .expected_outputs
            .iter()
            .any(|e| e.node == "parse_scan_result" && e.port == "crate_count");
        assert!(has_expected);
    }

    #[test]
    fn test_makefile_fails_spec() {
        let spec = bootstrap_mock_spec_makefile_fails();
        let makefile = spec.get_resource("fs:Makefile").unwrap();
        let gitignore = spec.get_resource("fs:.gitignore").unwrap();

        assert!(matches!(
            makefile.acquire(),
            gunbc_test::ResourceAcquireResult::Failed(_)
        ));
        assert!(matches!(
            gitignore.acquire(),
            gunbc_test::ResourceAcquireResult::Acquired
        ));
    }

    #[test]
    fn test_typed_builder_rejects_wrong_slot() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let reqs = extract_mock_requirements(&dag, "bootstrap");

        // Try to set a mock for a non-existent node
        let result = reqs.boundary_str("nonexistent_node", "port", "value");
        assert!(result.is_err());
    }
}
