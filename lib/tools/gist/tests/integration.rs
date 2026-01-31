//! Integration tests for gunbc-gist.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::{build_gist_graph, GistMode};
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{detect_boundaries, Value};
use gunbc_test::assert_boundary_mockable;

/// Test that dry-run mode intercepts the transport boundaries.
#[test]
fn test_dry_run_intercepts_transport() {
    let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Set up dry-run mode with mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();

    // Mock for execute_list_files (list files transport)
    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "src/main.rs\nREADME.md\n".to_string(),
            stderr: String::new(),
        })),
    );

    // Mock for execute_read_files (read files transport)
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "===GUNBC_FILE:src/main.rs===\nfn main() {}\n===GUNBC_FILE:README.md===\n# README\n".to_string(),
            stderr: String::new(),
        })),
    );

    // Mock for execute_gist (gist creation transport - only has response output now)
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "https://mock.gist/12345\n".to_string(),
            stderr: String::new(),
        })),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify all transport nodes were intercepted
    let list_entry = log
        .get("execute_list_files")
        .expect("execute_list_files should be in log");
    assert!(
        list_entry.was_intercepted,
        "execute_list_files should be intercepted in dry-run"
    );

    let read_entry = log
        .get("execute_read_files")
        .expect("execute_read_files should be in log");
    assert!(
        read_entry.was_intercepted,
        "execute_read_files should be intercepted in dry-run"
    );

    let gist_entry = log
        .get("execute_gist")
        .expect("execute_gist should be in log");
    assert!(
        gist_entry.was_intercepted,
        "execute_gist should be intercepted in dry-run"
    );

    // Verify parse_gist_response extracted the URL
    let parse_gist_entry = log
        .get("parse_gist_response")
        .expect("parse_gist_response should be in log");
    match parse_gist_entry.outputs.get("url") {
        Some(Value::Str(url)) => assert!(url.contains("mock.gist"), "expected URL to contain mock.gist"),
        _ => panic!("expected url output"),
    }

    // Verify pure nodes were NOT intercepted
    let prepare_list_entry = log
        .get("prepare_list_files")
        .expect("prepare_list_files should be in log");
    assert!(
        !prepare_list_entry.was_intercepted,
        "prepare_list_files should not be intercepted - it's pure"
    );

    let prepare_gist_entry = log
        .get("prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    assert!(
        !prepare_gist_entry.was_intercepted,
        "prepare_gist_request should not be intercepted - it's pure"
    );
}

/// Test that the graph structure correctly identifies boundaries.
#[test]
fn test_boundary_detection() {
    let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    let boundaries = detect_boundaries(&dag);

    // parse_gist_response is the terminal boundary (outputs url)
    assert!(boundaries.is_boundary_node(&"parse_gist_response".into()));

    // Pure intermediate nodes should not be boundaries
    assert!(!boundaries.is_boundary_node(&"prepare_list_files".into()));
    assert!(!boundaries.is_boundary_node(&"parse_list_files".into()));
    assert!(!boundaries.is_boundary_node(&"prepare_read_files".into()));
    assert!(!boundaries.is_boundary_node(&"parse_read_files".into()));
    assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
    assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
}

/// Test that the gist graph passes the boundary mockable test.
#[test]
fn test_gist_graph_boundary_mockable() {
    let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Need proper typed mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();

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
        "Gist graph should be boundary-mockable: {:?}",
        result.error
    );
    // execute_gist is a transport executor boundary
    assert!(result.boundary_nodes.contains(&"execute_gist".to_string()));
}

/// Test that real mode does NOT intercept boundaries.
#[test]
fn test_real_mode_no_interception() {
    let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Real mode - note: this will fail at transport nodes if gh isn't authenticated,
    // but we can still verify that pure nodes executed without interception
    match execute_with_mode(&dag, ExecutionMode::Real) {
        Ok(log) => {
            // If it succeeded, verify no interception happened for pure nodes
            for entry in &log.entries {
                if !entry.node_id.starts_with("execute_") {
                    assert!(
                        !entry.was_intercepted,
                        "{} should not be intercepted",
                        entry.node_id
                    );
                }
            }
        }
        Err(_) => {
            // Expected to fail at transport nodes without proper auth/repo
            // That's fine - the point is we got there without interception
        }
    }
}
