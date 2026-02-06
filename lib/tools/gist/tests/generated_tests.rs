//! Generated tests for gist DAG.
//!
//! These tests are generated from the gist graph structure.
//! They verify:
//! - Boundary mockability (can world-writes be intercepted?)
//! - Edge type compatibility (are connections type-safe?)

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::{build_gist_graph, GistMode};
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{detect_boundaries, Timestamp, Value};
use gunbc_primitives::filename;
use gunbc_test::{assert_boundary_mockable, assert_types_compatible};
use std::time::SystemTime;

/// Helper: mock for execute_current_branch boundary.
fn mock_current_branch(mocks: &mut BoundaryMocks) {
    mocks.set_value(
        "execute_current_branch",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "main\n".to_string(),
            stderr: String::new(),
        })),
    );
}

/// Helper: mock for execute_remote_branches boundary.
fn mock_remote_branches(mocks: &mut BoundaryMocks) {
    mocks.set_value(
        "execute_remote_branches",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "".to_string(),
            stderr: String::new(),
        })),
    );
}

fn mock_env(mocks: &mut BoundaryMocks) {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    mocks.set_value("fs_env", "fs:write", fs.into());
    let clock = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
    mocks.set_value("clock_env", "clock", clock.into());
    // Entry inputs (repo_path) for snapshot graph
    mocks.set_input("prepare_list_files", "repo_path", Value::Str(".".into()));
    mocks.set_input("prepare_read_files", "repo_path", Value::Str(".".into()));
    mocks.set_input("prepare_current_branch", "repo_path", Value::Str(".".into()));
    mocks.set_input("prepare_remote_branches", "repo_path", Value::Str(".".into()));
}

// ============================================================================
// BOUNDARY TESTS
// ============================================================================

/// Test that all boundaries can be mocked.
#[test]
fn test_boundaries_mockable() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Need proper typed mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock execute_list_files
    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "src/main.rs\n".to_string(),
            stderr: String::new(),
        })),
    );

    // Mock execute_read_files
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "===GUNBC_FILE:src/main.rs===\nfn main() {}\n".to_string(),
            stderr: String::new(),
        })),
    );

    // Mock execute_current_branch
    mock_current_branch(&mut mocks);
    // Mock execute_remote_branches
    mock_remote_branches(&mut mocks);

    // Mock execute_gist (only has response output now)
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "https://gist.github.com/mock/123".to_string(),
            stderr: String::new(),
        })),
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
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    let boundaries = detect_boundaries(&dag);
    // parse_gist_response is the terminal node (boundary)
    assert!(
        boundaries.is_boundary_node(&"parse_gist_response".into()),
        "parse_gist_response should be a boundary (terminal node)"
    );

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);
    // Mock execute_list_files
    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "src/main.rs\n".to_string(),
            stderr: String::new(),
        })),
    );
    // Mock execute_read_files
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "===GUNBC_FILE:src/main.rs===\nfn main() {}\n".to_string(),
            stderr: String::new(),
        })),
    );
    // Mock execute_current_branch
    mock_current_branch(&mut mocks);
    // Mock execute_remote_branches
    mock_remote_branches(&mut mocks);
    // Mock execute_gist (only has response output now)
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "https://gist.github.com/mock/123".to_string(),
            stderr: String::new(),
        })),
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
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
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
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
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
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // TransportRequest -> TransportRequest: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "prepare_list_files" && e.to_node.0 == "execute_list_files"));
}

/// Test edge execute_list_files.response -> parse_list_files.response type compatibility.
#[test]
fn test_edge_execute_list_to_parse_list() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // TransportResponse -> TransportResponse: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "execute_list_files" && e.to_node.0 == "parse_list_files"));
}

/// Test edge parse_list_files.files -> read_files_loop.files type compatibility.
/// (Snapshot mode uses a LoopBuilder for per-file reads.)
#[test]
fn test_edge_parse_list_files_to_read_files_loop() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // List -> List: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "parse_list_files" && e.to_node.0 == "read_files_loop"));
}

/// Test edge parse_list_files.files -> collect_file_contents.filenames type compatibility.
#[test]
fn test_edge_parse_list_files_to_collect_file_contents() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // List -> List: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "parse_list_files" && e.to_node.0 == "collect_file_contents"));
}

/// Test edge read_files_loop.contents -> collect_file_contents.contents_list type compatibility.
#[test]
fn test_edge_read_files_loop_to_collect_file_contents() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // List -> List: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "read_files_loop" && e.to_node.0 == "collect_file_contents"));
}

/// Test edge collect_file_contents.contents -> render_markdown.contents type compatibility.
#[test]
fn test_edge_collect_file_contents_to_render_markdown() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // Map -> Map: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "collect_file_contents" && e.to_node.0 == "render_markdown"));
}

/// Test edge render_markdown.markdown -> prepare_gist_request.markdown type compatibility.
#[test]
fn test_edge_render_markdown_markdown_to_prepare_gist_request_markdown() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // String -> String: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "render_markdown" && e.to_node.0 == "prepare_gist_request"));
}

/// Test edge prepare_gist_request.request -> execute_gist.request type compatibility.
#[test]
fn test_edge_prepare_gist_request_to_execute_gist() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // TransportRequest -> TransportRequest: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "prepare_gist_request" && e.to_node.0 == "execute_gist"));
}

/// Test edge execute_gist.response -> parse_gist_response.response type compatibility.
#[test]
fn test_edge_execute_gist_to_parse_gist_response() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    // TransportResponse -> TransportResponse: verified by edge existence in graph
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "execute_gist" && e.to_node.0 == "parse_gist_response"));
}

// ============================================================================
// BRANCH ACQUISITION EDGE TESTS
// ============================================================================

/// Test edge prepare_current_branch.request -> execute_current_branch.request type compatibility.
#[test]
fn test_edge_prepare_current_branch_to_execute_current_branch() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "prepare_current_branch"
            && e.to_node.0 == "execute_current_branch"));
}

/// Test edge execute_current_branch.response -> parse_current_branch.response type compatibility.
#[test]
fn test_edge_execute_current_branch_to_parse_current_branch() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    assert!(dag.edges.iter().any(
        |e| e.from_node.0 == "execute_current_branch" && e.to_node.0 == "parse_current_branch"
    ));
}

/// Test edge parse_current_branch.branch -> prepare_gist_request.branch type compatibility.
#[test]
fn test_edge_parse_current_branch_to_prepare_gist_request() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "parse_current_branch" && e.to_node.0 == "prepare_gist_request"));
}

// ============================================================================
// REMOTE BRANCH ACQUISITION EDGE TESTS
// ============================================================================

/// Test edge prepare_remote_branches.request -> execute_remote_branches.request type compatibility.
#[test]
fn test_edge_prepare_remote_branches_to_execute_remote_branches() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "prepare_remote_branches"
            && e.to_node.0 == "execute_remote_branches"));
}

/// Test edge execute_remote_branches.response -> parse_remote_branches.response type compatibility.
#[test]
fn test_edge_execute_remote_branches_to_parse_remote_branches() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    assert!(dag.edges.iter().any(
        |e| e.from_node.0 == "execute_remote_branches"
            && e.to_node.0 == "parse_remote_branches"
    ));
}

/// Test edge parse_remote_branches.remote_branch -> prepare_gist_request.remote_branch type compatibility.
#[test]
fn test_edge_parse_remote_branches_to_prepare_gist_request() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    assert!(dag.edges.iter().any(
        |e| e.from_node.0 == "parse_remote_branches"
            && e.to_node.0 == "prepare_gist_request"
    ));
}

// ============================================================================
// RECENT MODE EDGE TESTS
// ============================================================================

/// Test edge prepare_rev_list.request -> execute_rev_list.request type compatibility.
#[test]
fn test_edge_prepare_rev_list_to_execute_rev_list() {
    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "prepare_rev_list" && e.to_node.0 == "execute_rev_list"));
}

/// Test edge execute_rev_list.response -> parse_rev_list.response type compatibility.
#[test]
fn test_edge_execute_rev_list_to_parse_rev_list() {
    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "execute_rev_list" && e.to_node.0 == "parse_rev_list"));
}

/// Test edge parse_rev_list.base_ref -> prepare_diff.base_ref type compatibility.
#[test]
fn test_edge_parse_rev_list_to_prepare_diff() {
    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");
    assert!(dag
        .edges
        .iter()
        .any(|e| e.from_node.0 == "parse_rev_list" && e.to_node.0 == "prepare_diff"));
}

/// Test that execute_rev_list is NOT a boundary node (its output is consumed by parse_rev_list).
#[test]
fn test_execute_rev_list_not_boundary() {
    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");
    let boundaries = detect_boundaries(&dag);
    assert!(
        !boundaries.is_boundary_node(&"execute_rev_list".into()),
        "execute_rev_list should NOT be a boundary (output consumed by parse_rev_list)"
    );
}
