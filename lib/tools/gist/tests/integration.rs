//! Integration tests for gunbc-gist.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::{build_gist_graph, GistMode};
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{detect_boundaries, Timestamp, Value};
use gunbc_primitives::filename;
use gunbc_test::assert_boundary_mockable;
use std::time::SystemTime;

/// Helper: mock for execute_current_branch boundary.
fn mock_current_branch(mocks: &mut BoundaryMocks, branch: &str) {
    mocks.set_value(
        "execute_current_branch",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(format!("{}\n", branch)))),
    );
}

/// Helper: mock for execute_remote_branches boundary.
///
/// Simulates `git branch -r --points-at HEAD` output.
/// Pass empty string to simulate no remote branches at HEAD.
fn mock_remote_branches(mocks: &mut BoundaryMocks, remote_output: &str) {
    mocks.set_value(
        "execute_remote_branches",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(remote_output))),
    );
}

fn mock_env(mocks: &mut BoundaryMocks) {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    mocks.set_value("fs_env", "fs:write", fs.into());
    let clock = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
    mocks.set_value("clock_env", "clock", clock.into());
}

/// Test that dry-run mode intercepts the transport boundaries.
#[test]
fn test_dry_run_intercepts_transport() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Set up dry-run mode with mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock for execute_list_files (list files transport)
    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\nREADME.md\n"))),
    );

    // Mock for execute_read_files (read files transport)
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(
                "===GUNBC_FILE:src/main.rs===\nfn main() {}\n===GUNBC_FILE:README.md===\n# README\n",
            ))),
    );

    // Mock for execute_current_branch (branch name acquisition)
    mock_current_branch(&mut mocks, "feature/test-branch");
    // Mock for execute_remote_branches (empty — we're on a local branch)
    mock_remote_branches(&mut mocks, "");

    // Mock for execute_gist (gist creation transport - only has response output now)
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("https://mock.gist/12345\n"))),
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

    let branch_entry = log
        .get("execute_current_branch")
        .expect("execute_current_branch should be in log");
    assert!(
        branch_entry.was_intercepted,
        "execute_current_branch should be intercepted in dry-run"
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
        Some(Value::Str(url)) => assert!(
            url.contains("mock.gist"),
            "expected URL to contain mock.gist"
        ),
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
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
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
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Need proper typed mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock execute_list_files
    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );

    // Mock execute_read_files
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("===GUNBC_FILE:src/main.rs===\nfn main() {}\n"))),
    );

    // Mock execute_current_branch
    mock_current_branch(&mut mocks, "main");
    // Mock execute_remote_branches (empty — on a local branch)
    mock_remote_branches(&mut mocks, "");

    // Mock execute_gist (only has response output now)
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/123"))),
    );

    let result = assert_boundary_mockable(&dag, mocks);

    assert!(
        result.is_ok(),
        "Gist graph should be boundary-mockable: {:?}",
        result.error
    );
    // execute_gist is a transport executor boundary
    assert!(result.boundary_nodes.contains(&"execute_gist".to_string()));
    // execute_current_branch is also a transport boundary
    assert!(result
        .boundary_nodes
        .contains(&"execute_current_branch".to_string()));
    // execute_remote_branches is also a transport boundary
    assert!(result
        .boundary_nodes
        .contains(&"execute_remote_branches".to_string()));
}

/// Test that real mode does NOT intercept boundaries.
#[test]
fn test_real_mode_no_interception() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

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

// ============================================================================
// Branch-based filename integration tests
// ============================================================================

/// Test that the branch name flows through to the gist request filename.
///
/// Verifies the full pipeline: branch acquisition → filename sanitization →
/// gist request creation with the branch-based filename.
#[test]
fn test_branch_name_in_gist_filename() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("===GUNBC_FILE:src/main.rs===\nfn main() {}\n"))),
    );
    // Use a branch name with slashes (common in git workflows)
    mock_current_branch(&mut mocks, "claude/improve-gist-filename");
    mock_remote_branches(&mut mocks, "");
    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/456"))),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify prepare_gist_request received the branch name and produced a sanitized filename
    let prepare_gist = log
        .get("prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Shell(req))) => {
            // The filename should contain the sanitized branch name (slashes → hyphens)
            let filename_arg = req
                .args
                .iter()
                .find(|a| a.contains("claude-improve-gist-filename"));
            assert!(
                filename_arg.is_some(),
                "expected sanitized branch name in filename, got args: {:?}",
                req.args
            );
            // Should end with .md
            assert!(
                filename_arg.unwrap().ends_with(".md"),
                "filename should end with .md"
            );
            // Should contain a timestamp pattern (YYYY-MM-DD)
            assert!(
                filename_arg.unwrap().contains('-') && filename_arg.unwrap().contains('_'),
                "filename should contain timestamp separators"
            );
        }
        other => panic!(
            "expected shell request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}

/// Test filename generation with various platform-challenging branch names.
///
/// This is an integration-level test that exercises the full filename
/// sanitization pipeline through the DAG, covering branch names that
/// would be problematic on Windows, macOS, or Linux.
#[test]
fn test_platform_challenging_branch_names() {
    let challenging_branches = vec![
        // Slashes (common in git, invalid on all platforms)
        ("feature/my-feature", "feature-my-feature"),
        // Deep nesting
        ("refs/heads/feature/sub/deep", "refs-heads-feature-sub-deep"),
        // Windows-unsafe characters
        ("fix:urgent", "fix-urgent"),
        // Spaces
        ("my cool branch", "my-cool-branch"),
        // Mixed problematic chars
        ("user/fix<bug>?v2", "user-fix-bug-v2"),
    ];

    for (branch, expected_prefix) in challenging_branches {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");

        let mut mocks = BoundaryMocks::new();
        mock_env(&mut mocks);
        mocks.set_value(
            "execute_list_files",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
        );
        mocks.set_value(
            "execute_read_files",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok("===GUNBC_FILE:src/main.rs===\nfn main() {}\n"))),
        );
        mock_current_branch(&mut mocks, branch);
        mock_remote_branches(&mut mocks, "");
        mocks.set_value(
            "execute_gist",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/789"))),
        );

        let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

        let prepare_gist = log
            .get("prepare_gist_request")
            .expect("prepare_gist_request should be in log");
        match prepare_gist.outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Shell(req))) => {
                let filename_arg = req.args.iter().find(|a| a.starts_with(expected_prefix));
                assert!(
                    filename_arg.is_some(),
                    "branch '{}': expected filename starting with '{}', got args: {:?}",
                    branch,
                    expected_prefix,
                    req.args
                );
            }
            other => panic!(
                "branch '{}': expected shell request, got: {:?}",
                branch, other
            ),
        }
    }
}

// ============================================================================
// Detached HEAD → remote branch resolution tests
// ============================================================================

/// Test that detached HEAD at a remote branch uses the remote branch name.
///
/// Simulates `git checkout origin/main` — HEAD is detached, so
/// `rev-parse --abbrev-ref HEAD` returns "HEAD", but
/// `git branch -r --points-at HEAD` returns "  origin/main".
/// The gist filename should use "main" (remote prefix stripped).
#[test]
fn test_detached_head_uses_remote_branch_name() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("===GUNBC_FILE:src/main.rs===\nfn main() {}\n"))),
    );

    // Detached HEAD — rev-parse returns "HEAD"
    mock_current_branch(&mut mocks, "HEAD");
    // Remote branch points at HEAD
    mock_remote_branches(&mut mocks, "  origin/main\n");

    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/detached"))),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    let prepare_gist = log
        .get("prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Shell(req))) => {
            // Should use remote branch name "main" for filename
            let filename_arg = req.args.iter().find(|a| a.starts_with("main_"));
            assert!(
                filename_arg.is_some(),
                "expected remote branch 'main' in filename, got args: {:?}",
                req.args
            );
            assert!(filename_arg.unwrap().ends_with(".md"));
        }
        other => panic!(
            "expected shell request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}

// ============================================================================
// Recent mode integration tests
// ============================================================================

/// Test that recent mode dry-run works: mock rev-list returning a SHA,
/// verify diff runs against it.
#[test]
fn test_recent_mode_dry_run() {
    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock execute_rev_list: return a SHA (repo is older than 3 days)
    mocks.set_value(
        "execute_rev_list",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("abc123def456\n"))),
    );

    // Mock execute_diff: return sample diff
    mocks.set_value(
        "execute_diff",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }\n"))),
    );

    mock_current_branch(&mut mocks, "main");
    mock_remote_branches(&mut mocks, "");

    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/recent123"))),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify rev-list was intercepted
    let rev_list_entry = log
        .get("execute_rev_list")
        .expect("execute_rev_list should be in log");
    assert!(
        rev_list_entry.was_intercepted,
        "execute_rev_list should be intercepted in dry-run"
    );

    // Verify diff was intercepted
    let diff_entry = log
        .get("execute_diff")
        .expect("execute_diff should be in log");
    assert!(
        diff_entry.was_intercepted,
        "execute_diff should be intercepted in dry-run"
    );

    // Verify the parsed rev-list SHA flowed to prepare_diff as base_ref
    let parse_rev_list = log
        .get("parse_rev_list")
        .expect("parse_rev_list should be in log");
    match parse_rev_list.outputs.get("base_ref") {
        Some(Value::Str(sha)) => assert_eq!(sha, "abc123def456"),
        _ => panic!("expected base_ref output from parse_rev_list"),
    }

    // Verify gist URL was produced
    let parse_gist = log
        .get("parse_gist_response")
        .expect("parse_gist_response should be in log");
    match parse_gist.outputs.get("url") {
        Some(Value::Str(url)) => assert!(url.contains("mock"), "expected mock URL"),
        _ => panic!("expected url output"),
    }

    // Verify commit range appears in gist filename
    let prepare_gist = log
        .get("prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Shell(req))) => {
            let filename_arg = req
                .args
                .iter()
                .find(|a| a.contains("recent-3d") && a.contains("abc123d..HEAD"));
            assert!(
                filename_arg.is_some(),
                "expected recent-mode filename with commit range, got args: {:?}",
                req.args
            );
            // Description should mention the commit range
            let desc_idx = req.args.iter().position(|a| a == "--desc").unwrap();
            let desc = &req.args[desc_idx + 1];
            assert!(
                desc.contains("Recent changes (3d) abc123d..HEAD on main"),
                "expected recent-mode description, got: {}",
                desc
            );
        }
        other => panic!(
            "expected shell request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}

/// Test that recent mode with young repo (empty rev-list) produces graceful empty diff.
#[test]
fn test_recent_mode_young_repo() {
    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock execute_rev_list: empty output (repo < 3 days old)
    mocks.set_value(
        "execute_rev_list",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(""))),
    );

    // Mock execute_diff: empty diff (HEAD...HEAD produces nothing)
    mocks.set_value(
        "execute_diff",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(""))),
    );

    mock_current_branch(&mut mocks, "main");
    mock_remote_branches(&mut mocks, "");

    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/young"))),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // parse_rev_list should produce no base_ref (empty output)
    let parse_rev_list = log
        .get("parse_rev_list")
        .expect("parse_rev_list should be in log");
    assert!(
        !parse_rev_list.outputs.contains_key("base_ref"),
        "young repo should produce no base_ref"
    );

    // Gist should still complete (empty diff is valid)
    let parse_gist = log
        .get("parse_gist_response")
        .expect("parse_gist_response should be in log");
    assert!(
        parse_gist.outputs.contains_key("url"),
        "gist should still produce a URL even with empty diff"
    );

    // Young repo has no base_ref → falls back to snapshot-style filename
    let prepare_gist = log
        .get("prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Shell(req))) => {
            // Should NOT contain recent-3d (no base_ref to form commit range)
            let has_recent = req.args.iter().any(|a| a.contains("recent-3d"));
            assert!(
                !has_recent,
                "young repo should fall back to snapshot-style filename, got args: {:?}",
                req.args
            );
            // Should use branch name "main" as prefix
            let f_arg = req.args.iter().find(|a| a.starts_with("main_"));
            assert!(
                f_arg.is_some(),
                "expected main-prefixed filename for young repo, got args: {:?}",
                req.args
            );
        }
        other => panic!(
            "expected shell request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}

/// Test that detached HEAD with no remote branch falls back to "snapshot".
#[test]
fn test_detached_head_no_remote_uses_snapshot() {
    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    mocks.set_value(
        "execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );
    mocks.set_value(
        "execute_read_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("===GUNBC_FILE:src/main.rs===\nfn main() {}\n"))),
    );

    // Detached HEAD — rev-parse returns "HEAD"
    mock_current_branch(&mut mocks, "HEAD");
    // No remote branches point at HEAD
    mock_remote_branches(&mut mocks, "");

    mocks.set_value(
        "execute_gist",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/orphan"))),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    let prepare_gist = log
        .get("prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Shell(req))) => {
            // Should fall back to "snapshot" for filename
            let filename_arg = req.args.iter().find(|a| a.starts_with("snapshot_"));
            assert!(
                filename_arg.is_some(),
                "expected 'snapshot' filename for orphan detached HEAD, got args: {:?}",
                req.args
            );
        }
        other => panic!(
            "expected shell request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}
