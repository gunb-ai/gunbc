//! Integration tests for gunbc-buck2.

use gunbc_buck2::build_buck2_graph;
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{FileOp, FileResponse, TransportResponse};
use gunbc_ir::Value;
use gunbc_test::assert_boundary_mockable;

/// Test that dry-run mode intercepts all transport boundaries.
#[test]
fn test_dry_run_intercepts_transport() {
    let dag = build_buck2_graph().expect("Failed to build buck2 graph");

    // Set up dry-run mode with mocks for all transport nodes
    let mut mocks = BoundaryMocks::new();

    // Mock for execute_parse_cargo_toml (reads Cargo.toml)
    mocks.set_value(
        "execute_parse_cargo_toml",
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: "Cargo.toml".to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some("[package]\nname = \"test\"".to_string()),
            exists: Some(true),
            error: None,
        })),
    );

    // Mock for execute_transport (writes BUCK file)
    mocks.set_value(
        "execute_transport",
        "written_path",
        Value::Str("/dry-run/path".to_string()),
    );
    mocks.set_value(
        "execute_transport",
        "content",
        Value::Str("mock content".to_string()),
    );
    mocks.set_value(
        "execute_transport",
        "response",
        Value::Response(TransportResponse::File(FileResponse::written(
            "/dry-run/path",
        ))),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify execute_transport was intercepted
    let entry = log
        .get("execute_transport")
        .expect("execute_transport should be in log");
    assert!(
        entry.was_intercepted,
        "execute_transport should be intercepted in dry-run"
    );

    // Verify the mock value was used
    match entry.outputs.get("written_path") {
        Some(Value::Str(path)) => assert_eq!(path, "/dry-run/path"),
        _ => panic!("expected mock path"),
    }

    // Verify prepare_file_write was NOT intercepted (it's pure)
    let prepare_entry = log
        .get("prepare_file_write")
        .expect("prepare_file_write should be in log");
    assert!(
        !prepare_entry.was_intercepted,
        "prepare_file_write should not be intercepted - it's pure"
    );
}

/// Test that the graph structure correctly identifies boundaries.
#[test]
fn test_boundary_detection() {
    let dag = build_buck2_graph().expect("Failed to build buck2 graph");

    // Verify transport nodes exist (execute_transport is a terminal boundary)
    assert!(dag.get_node(&"execute_parse_cargo_toml".into()).is_some());
    assert!(dag.get_node(&"execute_transport".into()).is_some());

    // Pure nodes should exist
    assert!(dag.get_node(&"prepare_parse_cargo_toml".into()).is_some());
    assert!(dag.get_node(&"parse_cargo_toml_result".into()).is_some());
}

/// Test that the buck2 graph passes the boundary mockable test.
#[test]
fn test_buck2_graph_boundary_mockable() {
    let dag = build_buck2_graph().expect("Failed to build buck2 graph");

    // Need proper typed mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();

    // Mock for execute_parse_cargo_toml
    mocks.set_value(
        "execute_parse_cargo_toml",
        "response",
        Value::Response(TransportResponse::File(FileResponse {
            path: "Cargo.toml".to_string(),
            operation: FileOp::Read,
            success: true,
            content: Some("[package]\nname = \"test\"".to_string()),
            exists: Some(true),
            error: None,
        })),
    );

    // Mock for execute_transport
    mocks.set_value(
        "execute_transport",
        "written_path",
        Value::Str("/mock/path".to_string()),
    );
    mocks.set_value(
        "execute_transport",
        "content",
        Value::Str("mock content".to_string()),
    );
    mocks.set_value(
        "execute_transport",
        "response",
        Value::Response(TransportResponse::File(FileResponse::written("/mock/path"))),
    );

    let result = assert_boundary_mockable(&dag, mocks);

    assert!(
        result.is_ok(),
        "Buck2 graph should be boundary-mockable: {:?}",
        result.error
    );
    assert!(result.boundary_nodes.contains(&"execute_transport".to_string()));
}
