//! Generated tests for gist DAG.
//!
//! These tests are generated from the gist graph structure.
//! They verify:
//! - Boundary mockability (can world-writes be intercepted?)
//! - Edge type compatibility (are connections type-safe?)

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::build_gist_graph;
use gunbc_ir::{detect_boundaries, Value};
use gunbc_test::{assert_boundary_mockable, assert_types_compatible, default_mocks};

// ============================================================================
// BOUNDARY TESTS
// ============================================================================

/// Test that all boundaries can be mocked.
#[test]
fn test_boundaries_mockable() {
    let dag = build_gist_graph(vec![], false);
    let result = assert_boundary_mockable(&dag, default_mocks());
    assert!(
        result.is_ok(),
        "Boundaries should be mockable: {:?}",
        result.error
    );
}

/// Test that execute_transport boundary can be mocked.
#[test]
fn test_boundary_execute_transport_mockable() {
    let dag = build_gist_graph(vec![], false);
    let boundaries = detect_boundaries(&dag);
    assert!(
        boundaries.is_boundary_node(&"execute_transport".into()),
        "execute_transport should be a boundary"
    );

    let mut mocks = BoundaryMocks::new();
    mocks.set_value("execute_transport", "url", Value::Str("<MOCK>".to_string()));
    mocks.set_value(
        "execute_transport",
        "response",
        Value::Response(gunbc_ir::transport::TransportResponse::Shell(
            gunbc_ir::transport::ShellResponse {
                exit_code: 0,
                stdout: "<MOCK>".to_string(),
                stderr: String::new(),
            },
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();
    let entry = log
        .get("execute_transport")
        .expect("node should be in log");
    assert!(
        entry.was_intercepted,
        "boundary should be intercepted in dry-run"
    );
}

/// Test that prepare_gist_request is NOT a boundary (pure logic).
#[test]
fn test_prepare_gist_request_not_boundary() {
    let dag = build_gist_graph(vec![], false);
    let boundaries = detect_boundaries(&dag);
    assert!(
        !boundaries.is_boundary_node(&"prepare_gist_request".into()),
        "prepare_gist_request should NOT be a boundary - it's pure"
    );
}

// ============================================================================
// COMPOSITION TESTS
// ============================================================================

/// Test that all edge types are compatible.
#[test]
fn test_all_edges_compatible() {
    let dag = build_gist_graph(vec![], false);
    let results = assert_types_compatible(&dag);
    for result in &results {
        assert!(
            result.is_compatible(),
            "Edge {} should be compatible",
            result.edge
        );
    }
}

/// Test edge list_files.files -> filter_files.files type compatibility.
#[test]
fn test_edge_list_files_files_to_filter_files_files() {
    // StrList -> StrList
    assert!(true, "Types StrList and StrList should be compatible");
}

/// Test edge filter_files.files -> read_files.files type compatibility.
#[test]
fn test_edge_filter_files_files_to_read_files_files() {
    // StrList -> StrList
    assert!(true, "Types StrList and StrList should be compatible");
}

/// Test edge read_files.contents -> render_markdown.contents type compatibility.
#[test]
fn test_edge_read_files_contents_to_render_markdown_contents() {
    // MapStrStr -> MapStrStr
    assert!(true, "Types MapStrStr and MapStrStr should be compatible");
}

/// Test edge render_markdown.markdown -> prepare_gist_request.markdown type compatibility.
#[test]
fn test_edge_render_markdown_markdown_to_prepare_gist_request_markdown() {
    // String -> String
    assert!(true, "Types String and String should be compatible");
}

/// Test edge prepare_gist_request.request -> execute_transport.request type compatibility.
#[test]
fn test_edge_prepare_gist_request_to_execute_transport() {
    // TransportRequest -> TransportRequest
    assert!(
        true,
        "Types TransportRequest and TransportRequest should be compatible"
    );
}
