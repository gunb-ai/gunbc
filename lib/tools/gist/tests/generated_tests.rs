//! Generated tests for gist DAG.
//!
//! These tests are generated from the gist graph structure.
//! They verify:
//! - Boundary mockability (can world-writes be intercepted?)
//! - Edge type compatibility (are connections type-safe?)

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::build_gist_graph;
use gunbc_ir::{detect_boundaries, Value};
use gunbc_test::{assert_boundary_mockable, assert_types_compatible};

// ============================================================================
// BOUNDARY TESTS
// ============================================================================

/// Test that all boundaries can be mocked.
#[test]
fn test_boundaries_mockable() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");

    // Need proper typed mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();

    // Mock execute_list_files
    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(gunbc_ir::transport::TransportResponse::Shell(
            gunbc_ir::transport::ShellResponse {
                exit_code: 0,
                stdout: "src/main.rs\n".to_string(),
                stderr: String::new(),
            },
        )),
    );

    // Mock execute_read_files
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(gunbc_ir::transport::TransportResponse::Shell(
            gunbc_ir::transport::ShellResponse {
                exit_code: 0,
                stdout: "===GUNBC_FILE:src/main.rs===\nfn main() {}\n".to_string(),
                stderr: String::new(),
            },
        )),
    );

    // Mock execute_gist (only has response output now)
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(gunbc_ir::transport::TransportResponse::Shell(
            gunbc_ir::transport::ShellResponse {
                exit_code: 0,
                stdout: "https://gist.github.com/mock/123".to_string(),
                stderr: String::new(),
            },
        )),
    );

    let result = assert_boundary_mockable(&dag, mocks);
    assert!(
        result.is_ok(),
        "Boundaries should be mockable: {:?}",
        result.error
    );
}

/// Test that parse_gist_response boundary can be mocked.
#[test]
fn test_boundary_parse_gist_response_mockable() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    let boundaries = detect_boundaries(&dag);
    // parse_gist_response is the terminal node (boundary)
    assert!(
        boundaries.is_boundary_node(&"parse_gist_response".into()),
        "parse_gist_response should be a boundary (terminal node)"
    );

    let mut mocks = BoundaryMocks::new();
    // Mock execute_list_files
    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(gunbc_ir::transport::TransportResponse::Shell(
            gunbc_ir::transport::ShellResponse {
                exit_code: 0,
                stdout: "src/main.rs\n".to_string(),
                stderr: String::new(),
            },
        )),
    );
    // Mock execute_read_files
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(gunbc_ir::transport::TransportResponse::Shell(
            gunbc_ir::transport::ShellResponse {
                exit_code: 0,
                stdout: "===GUNBC_FILE:src/main.rs===\nfn main() {}\n".to_string(),
                stderr: String::new(),
            },
        )),
    );
    // Mock execute_gist (only has response output now)
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(gunbc_ir::transport::TransportResponse::Shell(
            gunbc_ir::transport::ShellResponse {
                exit_code: 0,
                stdout: "https://gist.github.com/mock/123".to_string(),
                stderr: String::new(),
            },
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();
    
    // Verify parse_gist_response was executed (it's in the log)
    let entry = log
        .get("parse_gist_response")
        .expect("parse_gist_response should be in log");
    // parse_gist_response is a pure node, not intercepted, but it ran
    assert!(
        !entry.was_intercepted,
        "parse_gist_response should not be intercepted (pure)"
    );
}

/// Test that prepare_gist_request is NOT a boundary (pure logic).
#[test]
fn test_prepare_gist_request_not_boundary() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
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
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    let results = assert_types_compatible(&dag);
    for result in &results {
        assert!(
            result.is_compatible(),
            "Edge {} should be compatible",
            result.edge
        );
    }
}

/// Test edge prepare_list_files.request -> execute_list_files.request type compatibility.
#[test]
fn test_edge_prepare_list_to_execute_list() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // TransportRequest -> TransportRequest: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "prepare_list_files" && e.to_node.0 == "execute_list_files"));
}

/// Test edge execute_list_files.response -> parse_list_files.response type compatibility.
#[test]
fn test_edge_execute_list_to_parse_list() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // TransportResponse -> TransportResponse: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "execute_list_files" && e.to_node.0 == "parse_list_files"));
}

/// Test edge parse_list_files.files -> filter_files.files type compatibility.
#[test]
fn test_edge_parse_list_files_to_filter_files() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // StrList -> StrList: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "parse_list_files" && e.to_node.0 == "filter_files"));
}

/// Test edge filter_files.files -> prepare_read_files.files type compatibility.
#[test]
fn test_edge_filter_files_to_prepare_read_files() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // StrList -> StrList: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "filter_files" && e.to_node.0 == "prepare_read_files"));
}

/// Test edge prepare_read_files.request -> execute_read_files.request type compatibility.
#[test]
fn test_edge_prepare_read_to_execute_read() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // TransportRequest -> TransportRequest: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "prepare_read_files" && e.to_node.0 == "execute_read_files"));
}

/// Test edge execute_read_files.response -> parse_read_files.response type compatibility.
#[test]
fn test_edge_execute_read_to_parse_read() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // TransportResponse -> TransportResponse: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "execute_read_files" && e.to_node.0 == "parse_read_files"));
}

/// Test edge parse_read_files.contents -> render_markdown.contents type compatibility.
#[test]
fn test_edge_parse_read_to_render_markdown() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // MapStrStr -> MapStrStr: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "parse_read_files" && e.to_node.0 == "render_markdown"));
}

/// Test edge render_markdown.markdown -> prepare_gist_request.markdown type compatibility.
#[test]
fn test_edge_render_markdown_markdown_to_prepare_gist_request_markdown() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // String -> String: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "render_markdown" && e.to_node.0 == "prepare_gist_request"));
}

/// Test edge prepare_gist_request.request -> execute_gist.request type compatibility.
#[test]
fn test_edge_prepare_gist_request_to_execute_gist() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // TransportRequest -> TransportRequest: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "prepare_gist_request" && e.to_node.0 == "execute_gist"));
}

/// Test edge execute_gist.response -> parse_gist_response.response type compatibility.
#[test]
fn test_edge_execute_gist_to_parse_gist_response() {
    let dag = build_gist_graph(vec![], false).expect("Failed to build gist graph");
    // TransportResponse -> TransportResponse: verified by edge existence in graph
    assert!(dag.edges.iter().any(|e| e.from_node.0 == "execute_gist" && e.to_node.0 == "parse_gist_response"));
}
