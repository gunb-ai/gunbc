//! Integration tests for gunbc-buck2.

use gunbc_buck2::build_buck2_graph;
use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::{FileResponse, TransportResponse};
use gunbc_ir::{detect_boundaries, Value};
use gunbc_test::{assert_boundary_mockable, default_mocks};

/// Test that dry-run mode intercepts the transport boundary.
#[test]
fn test_dry_run_intercepts_transport() {
    let dag = build_buck2_graph().expect("Failed to build buck2 graph");

    // Set up dry-run mode with mock
    let mut mocks = BoundaryMocks::new();
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
    let boundaries = detect_boundaries(&dag);

    // Only execute_transport should be a boundary
    assert_eq!(boundaries.boundary_nodes.len(), 1);
    assert!(boundaries.is_boundary_node(&"execute_transport".into()));

    // Intermediate nodes should not be boundaries
    assert!(!boundaries.is_boundary_node(&"parse_cargo_toml".into()));
    assert!(!boundaries.is_boundary_node(&"extract_deps".into()));
    assert!(!boundaries.is_boundary_node(&"generate_targets".into()));
    assert!(!boundaries.is_boundary_node(&"prepare_file_write".into()));
}

/// Test that the buck2 graph passes the boundary mockable test.
#[test]
fn test_buck2_graph_boundary_mockable() {
    let dag = build_buck2_graph().expect("Failed to build buck2 graph");
    let result = assert_boundary_mockable(&dag, default_mocks());

    assert!(
        result.is_ok(),
        "Buck2 graph should be boundary-mockable: {:?}",
        result.error
    );
    assert_eq!(result.boundary_nodes, vec!["execute_transport"]);
}
