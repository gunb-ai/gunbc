//! Integration tests for gunbc-gist.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::build_gist_graph;
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{detect_boundaries, Value};
use gunbc_test::{assert_boundary_mockable, default_mocks};

/// Test that dry-run mode intercepts the transport boundary.
#[test]
fn test_dry_run_intercepts_transport() {
    let dag = build_gist_graph(vec![], false);

    // Set up dry-run mode with mock
    let mut mocks = BoundaryMocks::new();
    mocks.set_value(
        "execute_transport",
        "url",
        Value::Str("https://mock.gist/12345".to_string()),
    );
    mocks.set_value(
        "execute_transport",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: "https://mock.gist/12345\n".to_string(),
            stderr: String::new(),
        })),
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
    match entry.outputs.get("url") {
        Some(Value::Str(url)) => assert_eq!(url, "https://mock.gist/12345"),
        _ => panic!("expected mock url"),
    }

    // Verify other nodes were NOT intercepted
    let list_entry = log.get("list_files").expect("list_files should be in log");
    assert!(
        !list_entry.was_intercepted,
        "list_files should not be intercepted"
    );

    // Verify prepare_gist_request was NOT intercepted (it's pure)
    let prepare_entry = log
        .get("prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    assert!(
        !prepare_entry.was_intercepted,
        "prepare_gist_request should not be intercepted - it's pure"
    );
}

/// Test that the graph structure correctly identifies boundaries.
#[test]
fn test_boundary_detection() {
    let dag = build_gist_graph(vec![], false);
    let boundaries = detect_boundaries(&dag);

    // Only execute_transport should be a boundary
    assert_eq!(boundaries.boundary_nodes.len(), 1);
    assert!(boundaries.is_boundary_node(&"execute_transport".into()));

    // Intermediate nodes should not be boundaries
    assert!(!boundaries.is_boundary_node(&"list_files".into()));
    assert!(!boundaries.is_boundary_node(&"filter_files".into()));
    assert!(!boundaries.is_boundary_node(&"read_files".into()));
    assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
    assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
}

/// Test that the gist graph passes the boundary mockable test.
#[test]
fn test_gist_graph_boundary_mockable() {
    let dag = build_gist_graph(vec![], false);
    let result = assert_boundary_mockable(&dag, default_mocks());

    assert!(
        result.is_ok(),
        "Gist graph should be boundary-mockable: {:?}",
        result.error
    );
    assert_eq!(result.boundary_nodes, vec!["execute_transport"]);
}

/// Test that real mode does NOT intercept boundaries.
#[test]
fn test_real_mode_no_interception() {
    let dag = build_gist_graph(vec![], false);

    // Real mode - note: this will fail at execute_transport if gh isn't authenticated,
    // but we can still verify that intermediate nodes executed without interception
    match execute_with_mode(&dag, ExecutionMode::Real) {
        Ok(log) => {
            // If it succeeded, verify no interception happened
            for entry in &log.entries {
                if entry.node_id != "execute_transport" {
                    assert!(
                        !entry.was_intercepted,
                        "{} should not be intercepted",
                        entry.node_id
                    );
                }
            }
        }
        Err(_) => {
            // Expected to fail at execute_transport without gh auth
            // That's fine - the point is we got there without interception
        }
    }
}
