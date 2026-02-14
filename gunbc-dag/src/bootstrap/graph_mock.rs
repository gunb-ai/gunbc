//! Mock specification for the bootstrap tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! Five transport nodes need mocks:
//! - `execute_scan_workspace`: Scans workspace for crates
//! - `execute_read_makefile`: Reads existing Makefile
//! - `execute_makefile_transport`: Writes Makefile (skippable)
//! - `execute_read_gitignore`: Reads existing .gitignore
//! - `execute_gitignore_transport`: Writes .gitignore (skippable)
//!
//! # Input Expectations
//!
//! - `check_mode`: Optional bool, defaults to false
//! - `path`: Optional string for read paths
//!
//! # Resource Simulations
//!
//! - File locks for Makefile and .gitignore

use crate::bootstrap::graph::build_bootstrap_graph;
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_primitives::filename;
use gunbc_test::{extract_mock_requirements, MockSpec, NodeExample, OutputMatcher};

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

/// Mock specification for the bootstrap graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "bootstrap",
    builder = "crate::build_bootstrap_graph().unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "bootstrap",
    output = "gunbc-dag/src/bootstrap/generated_tests.rs",
    module = "bootstrap_generated_tests",
    builder = "crate::build_bootstrap_graph().unwrap()",
    signature = "crate::bootstrap_signature()",
    tool = "bootstrap",
    flow_tests
)]
pub fn bootstrap_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "bootstrap")
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type")
        // Transport: execute_scan_workspace (workspace scan)
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        // Transport: execute_read_makefile (read existing Makefile)
        .transport_response(
            "execute_read_makefile",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock-makefile>".into()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read_makefile response should match type")
        // Transport: execute_makefile_transport (write Makefile, skippable)
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
        .boundary_str(
            "execute_makefile_transport",
            "makefile_written_path",
            "Makefile",
        )
        .expect("execute_makefile_transport path should match type")
        .boundary_str(
            "execute_makefile_transport",
            "makefile_content",
            "<mock-makefile>",
        )
        .expect("execute_makefile_transport content should match type")
        .boundary_bool("execute_makefile_transport", "skip", false)
        .expect("execute_makefile_transport skip should match type")
        .boundary_str("execute_makefile_transport", "skip_reason", "")
        .expect("execute_makefile_transport skip_reason should match type")
        // Transport: execute_read_gitignore (read existing .gitignore)
        .transport_response(
            "execute_read_gitignore",
            "response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock-gitignore>".into()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read_gitignore response should match type")
        // Transport: execute_gitignore_transport (write .gitignore, skippable)
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
        .boundary_str(
            "execute_gitignore_transport",
            "gitignore_written_path",
            ".gitignore",
        )
        .expect("execute_gitignore_transport path should match type")
        .boundary_str(
            "execute_gitignore_transport",
            "gitignore_content",
            "<mock-gitignore>",
        )
        .expect("execute_gitignore_transport content should match type")
        .boundary_bool("execute_gitignore_transport", "skip", false)
        .expect("execute_gitignore_transport skip should match type")
        .boundary_str("execute_gitignore_transport", "skip_reason", "")
        .expect("execute_gitignore_transport skip_reason should match type")
        // Build spec (pure terminal outputs are computed, not mocked)
        .build_unchecked()
        // Input mocks for DAG entry points (dangling inputs with no upstream edge)
        .input_mock(
            "prepare_read_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_read_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock(
            "prepare_write_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_write_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock("compare_makefile_content", "check_mode", Value::Bool(false))
        .input_mock(
            "compare_makefile_content",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock-makefile>".into()),
                exists: None,
                error: None,
            })),
        )
        .input_mock(
            "compare_gitignore_content",
            "check_mode",
            Value::Bool(false),
        )
        .input_mock(
            "compare_gitignore_content",
            "response",
            Value::Response(TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock-gitignore>".into()),
                exists: None,
                error: None,
            })),
        )
        // Resources: file locks for both outputs
        .resource_lock("file:Makefile")
        .resource_lock("file:.gitignore")
        // Expected outputs: verified after DryRun execution
        .expected_output("parse_scan_result", "crate_count", Value::Int(2))
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("fs_env")
                .output("file:write", OutputMatcher::Any)
                .description("Provides filesystem handle for bootstrap writes"),
        )
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
                    OutputMatcher::contains("Generated by gunbc-bootstrap"),
                )
                .description("Generates .gitignore content from build config"),
        )
        // Probe-observer: transport terminals need chain-safe observers
        .live_expected_output("execute_makefile_transport", "skip", OutputMatcher::IsBool)
        .live_expected_output("execute_gitignore_transport", "skip", OutputMatcher::IsBool)
        // Primitive nodes — tested in their own crates
        .skip_node_example("prepare_read_makefile")
        .skip_node_example("prepare_write_makefile")
        .skip_node_example("compare_makefile_content")
        .skip_node_example("prepare_read_gitignore")
        .skip_node_example("prepare_write_gitignore")
        .skip_node_example("compare_gitignore_content")
}

/// Mock spec for testing single file write (Makefile only).
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn bootstrap_mock_spec_makefile_only() -> MockSpec {
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    extract_mock_requirements(&dag, "bootstrap")
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type")
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        .transport_response(
            "execute_read_makefile",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file".into()),
            }),
        )
        .expect("execute_read_makefile response should match type")
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
        .boundary_str(
            "execute_makefile_transport",
            "makefile_written_path",
            "Makefile",
        )
        .expect("path should match type")
        .boundary_str("execute_makefile_transport", "makefile_content", "<mock>")
        .expect("content should match type")
        .boundary_bool("execute_makefile_transport", "skip", false)
        .expect("skip should match type")
        .boundary_str("execute_makefile_transport", "skip_reason", "")
        .expect("skip_reason should match type")
        .transport_response(
            "execute_read_gitignore",
            "response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file".into()),
            }),
        )
        .expect("execute_read_gitignore response should match type")
        .transport_response(
            "execute_gitignore_transport",
            "gitignore_response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Write,
                success: false,
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
        .boundary_bool("execute_gitignore_transport", "skip", false)
        .expect("skip should match type")
        .boundary_str("execute_gitignore_transport", "skip_reason", "")
        .expect("skip_reason should match type")
        .build_unchecked()
        .input_mock(
            "prepare_read_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_read_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock(
            "prepare_write_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_write_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock("compare_makefile_content", "check_mode", Value::Bool(false))
        .input_mock(
            "compare_gitignore_content",
            "check_mode",
            Value::Bool(false),
        )
        .resource_lock("file:Makefile")
}

/// Mock spec for testing file system failure on Makefile.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn bootstrap_mock_spec_makefile_fails() -> MockSpec {
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    extract_mock_requirements(&dag, "bootstrap")
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type")
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        .transport_response(
            "execute_read_makefile",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file".into()),
            }),
        )
        .expect("execute_read_makefile response should match type")
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
        .boundary_bool("execute_makefile_transport", "skip", false)
        .expect("skip should match type")
        .boundary_str("execute_makefile_transport", "skip_reason", "")
        .expect("skip_reason should match type")
        .transport_response(
            "execute_read_gitignore",
            "response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("<mock>".into()),
                exists: None,
                error: None,
            }),
        )
        .expect("execute_read_gitignore response should match type")
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
        .boundary_str(
            "execute_gitignore_transport",
            "gitignore_written_path",
            ".gitignore",
        )
        .expect("path should match type")
        .boundary_str("execute_gitignore_transport", "gitignore_content", "<mock>")
        .expect("content should match type")
        .boundary_bool("execute_gitignore_transport", "skip", false)
        .expect("skip should match type")
        .boundary_str("execute_gitignore_transport", "skip_reason", "")
        .expect("skip_reason should match type")
        .build_unchecked()
        .input_mock(
            "prepare_read_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_read_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock(
            "prepare_write_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_write_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock("compare_makefile_content", "check_mode", Value::Bool(false))
        .input_mock(
            "compare_gitignore_content",
            "check_mode",
            Value::Bool(false),
        )
        .resource_lock_fails("file:Makefile", "Permission denied: Makefile is read-only")
        .resource_lock("file:.gitignore")
}

/// Mock spec for testing complete write failure.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn bootstrap_mock_spec_all_fail() -> MockSpec {
    let dag = build_bootstrap_graph().expect("bootstrap graph should build");

    extract_mock_requirements(&dag, "bootstrap")
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type")
        .transport_response(
            "execute_scan_workspace",
            "response",
            TransportResponse::Shell(ShellResponse::ok("crates/bar\ncrates/foo\n")),
        )
        .expect("execute_scan_workspace response should match type")
        .transport_response(
            "execute_read_makefile",
            "response",
            TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file".into()),
            }),
        )
        .expect("execute_read_makefile response should match type")
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
        .boundary_bool("execute_makefile_transport", "skip", false)
        .expect("skip should match type")
        .boundary_str("execute_makefile_transport", "skip_reason", "")
        .expect("skip_reason should match type")
        .transport_response(
            "execute_read_gitignore",
            "response",
            TransportResponse::File(FileResponse {
                path: ".gitignore".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file".into()),
            }),
        )
        .expect("execute_read_gitignore response should match type")
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
        .boundary_bool("execute_gitignore_transport", "skip", false)
        .expect("skip should match type")
        .boundary_str("execute_gitignore_transport", "skip_reason", "")
        .expect("skip_reason should match type")
        .build_unchecked()
        .input_mock(
            "prepare_read_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_read_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock(
            "prepare_write_makefile",
            "path",
            Value::Str("Makefile".into()),
        )
        .input_mock(
            "prepare_write_gitignore",
            "path",
            Value::Str(".gitignore".into()),
        )
        .input_mock("compare_makefile_content", "check_mode", Value::Bool(false))
        .input_mock(
            "compare_gitignore_content",
            "check_mode",
            Value::Bool(false),
        )
        .resource_lock_fails("file:Makefile", "Permission denied")
        .resource_lock_fails("file:.gitignore", "Permission denied")
}
