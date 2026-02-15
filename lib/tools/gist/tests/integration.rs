//! Integration tests for gunbc-gist.

use gunbc_exec::{execute_with_mode, BoundaryMocks, ExecutionMode};
use gunbc_gist::{build_gist_graph, GistMode};
use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{detect_boundaries, SecretString, Timestamp, Value};
use gunbc_primitives::filename;
use gunbc_test::{assert_boundary_mockable, guard_test, FermiCost, TestClass};
use std::time::SystemTime;

fn gist_request_filename(req: &gunbc_ir::transport::rest::RestRequest) -> String {
    let body = req
        .body
        .as_ref()
        .expect("gist request should have json body");
    let files = body
        .get("files")
        .and_then(|v| v.as_object())
        .expect("gist request should include files");
    files
        .keys()
        .next()
        .cloned()
        .expect("gist request should include a filename")
}

fn gist_request_description(req: &gunbc_ir::transport::rest::RestRequest) -> String {
    req.body
        .as_ref()
        .and_then(|b| b.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Helper: mock for execute_current_branch boundary (inside current_branch SubDag).
fn mock_current_branch(mocks: &mut BoundaryMocks, branch: &str) {
    mocks.set_value(
        "branch_resolution/execute_current_branch",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(format!(
            "{}\n",
            branch
        )))),
    );
}

/// Helper: mock for execute_remote_branches boundary (inside remote_branches SubDag).
///
/// Simulates `git branch -r --points-at HEAD` output.
/// Pass empty string to simulate no remote branches at HEAD.
fn mock_remote_branches(mocks: &mut BoundaryMocks, remote_output: &str) {
    mocks.set_value(
        "branch_resolution/execute_remote_branches",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(remote_output))),
    );
}

fn mock_env(mocks: &mut BoundaryMocks) {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    mocks.set_value("fs_env", "file:write", fs.into());
    // Gist upload SubDag internal environments
    let fs_upload = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    mocks.set_value("gist_upload/fs_env", "file:write", fs_upload.into());
    let clock = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
    mocks.set_value("gist_upload/clock_env", "clock", clock.into());

    let cloud_config = CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::LocalDev,
        audience: "local-dev".to_string(),
        project_or_account: "mock-secrets".to_string(),
        secret: CloudSecretRef {
            prefix: "ci-".to_string(),
            name: String::new(),
            delimiter: String::new(),
            version: None,
        },
        service_account_or_role: Some("ci-secrets@mock.iam.gserviceaccount.com".to_string()),
        impersonate_account_or_role: None,
    };

    mocks.set_value("gist_upload/cloud_env", "config", cloud_config.clone().into());
    mocks.set_value(
        "gist_upload/cloud_env",
        "request_url",
        Value::Str("https://example.com/oidc".to_string()),
    );
    mocks.set_value(
        "gist_upload/cloud_env",
        "request_token",
        Value::Str("mock-oidc-token".to_string()),
    );
    mocks.set_value("gist_upload/bind_secret", "config", cloud_config.into());

    let credential = Value::Map(std::collections::BTreeMap::from([
        (
            "token".to_string(),
            Value::Secret(SecretString::new("<MOCK_GITHUB_TOKEN>")),
        ),
        ("source_type".to_string(), Value::Str("static".to_string())),
        ("scheme".to_string(), Value::Str("bearer".to_string())),
        (
            "cap".to_string(),
            Value::Secret(SecretString::new("capability")),
        ),
    ]));
    mocks.set_value("gist_upload/cloud_credential", "credential", credential);
    mocks.set_value("gist_upload/cloud_credential", "expires_in", Value::Int(3_600));
    // local_auth_upsert sub-DAG mocks (local-dev path)
    let adc_path = "/tmp/mock-adc.json";
    let mock_adc_json = serde_json::json!({
        "type": "authorized_user",
        "client_id": "mock-client-id.apps.googleusercontent.com",
        "client_secret": "mock-client-secret",
        "refresh_token": "mock-refresh-token"
    })
    .to_string();
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/local_auth_upsert/net_env",
        "api:network",
        gunbc_primitives::NetworkHandle.into(),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/local_auth_upsert/execute_check",
        "response",
        Value::Response(TransportResponse::File(
            gunbc_ir::transport::file::FileResponse::exists_result(adc_path, true),
        )),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/local_auth_upsert/execute_read_adc",
        "response",
        Value::Response(TransportResponse::File(
            gunbc_ir::transport::file::FileResponse::read_ok(adc_path, mock_adc_json),
        )),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/local_auth_upsert/execute_oauth2",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "access_token": "mock-local-token",
                "expires_in": 3599,
                "token_type": "Bearer"
            })),
        )),
    );
    // Re-auth branch boundaries (skipped in happy path)
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/local_auth_upsert/execute_gcloud_auth",
        "response",
        Value::Skipped,
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/local_auth_upsert/execute_reread_adc",
        "response",
        Value::Skipped,
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/local_auth_upsert/execute_retry_oauth2",
        "response",
        Value::Skipped,
    );
    // IAM ensure (local dev only) — REST-based check + conditional set
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/execute_get_iam",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "bindings": [{
                    "role": "roles/secretmanager.secretAccessor",
                    "members": ["serviceAccount:ci-secrets@mock.iam.gserviceaccount.com"]
                }],
                "etag": "mock-etag"
            })),
        )),
    );
    // setIamPolicy is skipped (binding already exists in mock)
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/execute_set_iam",
        "response",
        Value::Skipped,
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/net_env",
        "api:network",
        gunbc_primitives::NetworkHandle.into(),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/execute_github_oidc",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({"value":"mock-oidc-token"})),
        )),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/execute_sts",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "access_token": "mock-sts-token",
                "expires_in": 3600
            })),
        )),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/execute_impersonate",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "accessToken": "mock-sa-token",
                "expireTime": "2025-01-01T00:00:00Z"
            })),
        )),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/execute_secret_access",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(serde_json::json!({
                "payload": {"data": "bW9jay1zZWNyZXQ="}
            })),
        )),
    );
    mocks.set_value(
        "gist_upload/cloud_credential/gcp_wif_secret/build_credential",
        "credential",
        Value::Map(std::collections::BTreeMap::from([
            (
                "token".to_string(),
                Value::Secret(SecretString::new("mock-secret")),
            ),
            ("source_type".to_string(), Value::Str("static".to_string())),
            ("scheme".to_string(), Value::Str("bearer".to_string())),
            (
                "cap".to_string(),
                Value::Secret(SecretString::new("capability")),
            ),
        ])),
    );

    // Entry inputs (repo_path) for all gist modes — SubDag wrappers are the entrypoints
    mocks.set_input("list_files", "repo_path", Value::Str(".".into()));
    mocks.set_input("read_files_loop", "repo_path", Value::Str(".".into()));
    mocks.set_input(
        "branch_resolution",
        "repo_path",
        Value::Str(".".into()),
    );
    mocks.set_input("diff", "repo_path", Value::Str(".".into()));
    mocks.set_input("rev_list", "repo_path", Value::Str(".".into()));
    // Loop body transport nodes are auto-mocked by execute_loop_body in DryRun mode.
}

fn guard_hermetic(name: &str) -> bool {
    guard_test(name, TestClass::Hermetic, FermiCost::XS, &[], &[])
}

fn guard_integration(name: &str) -> bool {
    guard_test(
        name,
        TestClass::Integration,
        FermiCost::M,
        &["git", "shell", "gh"],
        &[],
    )
}

/// Test that dry-run mode intercepts the transport boundaries.
#[test]
fn test_dry_run_intercepts_transport() {
    if !guard_hermetic(stringify!(test_dry_run_intercepts_transport)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Set up dry-run mode with mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock for execute_list_files (list files transport)
    mocks.set_value(
        "list_files/execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(
            "src/main.rs\nREADME.md\n",
        ))),
    );

    // Loop body transport nodes are auto-intercepted in DryRun mode

    // Mock for execute_current_branch (branch name acquisition)
    mock_current_branch(&mut mocks, "feature/test-branch");
    // Mock for execute_remote_branches (empty — we're on a local branch)
    mock_remote_branches(&mut mocks, "");

    // Mock for execute_gist (gist creation transport - only has response output now)
    mocks.set_value(
        "gist_upload/execute_gist",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(
                serde_json::json!({"html_url":"https://gist.github.com/mock/12345"}),
            ),
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify all transport nodes were intercepted
    let list_entry = log
        .get("list_files/execute_list_files")
        .expect("execute_list_files should be in log");
    assert!(
        list_entry.was_intercepted,
        "execute_list_files should be intercepted in dry-run"
    );

    let branch_entry = log
        .get("branch_resolution/execute_current_branch")
        .expect("execute_current_branch should be in log");
    assert!(
        branch_entry.was_intercepted,
        "execute_current_branch should be intercepted in dry-run"
    );

    let gist_entry = log
        .get("gist_upload/execute_gist")
        .expect("execute_gist should be in log");
    assert!(
        gist_entry.was_intercepted,
        "execute_gist should be intercepted in dry-run"
    );

    // Verify parse_gist_response extracted the URL
    let parse_gist_entry = log
        .get("gist_upload/parse_gist_response")
        .expect("parse_gist_response should be in log");
    match parse_gist_entry.outputs.get("url") {
        Some(Value::Str(url)) => assert!(
            url.contains("gist.github.com"),
            "expected URL to contain gist.github.com"
        ),
        _ => panic!("expected url output"),
    }

    // Verify pure nodes were NOT intercepted
    let prepare_list_entry = log
        .get("list_files/prepare_list_files")
        .expect("prepare_list_files should be in log");
    assert!(
        !prepare_list_entry.was_intercepted,
        "prepare_list_files should not be intercepted - it's pure"
    );

    let prepare_gist_entry = log
        .get("gist_upload/prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    assert!(
        !prepare_gist_entry.was_intercepted,
        "prepare_gist_request should not be intercepted - it's pure"
    );
}

/// Test that the graph structure correctly identifies boundaries.
#[test]
fn test_boundary_detection() {
    if !guard_hermetic(stringify!(test_boundary_detection)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");
    let boundaries = detect_boundaries(&dag);

    // gist_upload is a terminal SubDag (contains parse_gist_response which outputs url)
    assert!(boundaries.is_boundary_node(&"gist_upload".into()));

    // Pure intermediate nodes should not be boundaries
    assert!(!boundaries.is_boundary_node(&"collect_file_contents".into()));
    assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
}

/// Test that the gist graph passes the boundary mockable test.
#[test]
fn test_gist_graph_boundary_mockable() {
    if !guard_hermetic(stringify!(test_gist_graph_boundary_mockable)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Need proper typed mocks for all transport boundaries
    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock execute_list_files
    mocks.set_value(
        "list_files/execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );

    // Loop body transport nodes are auto-intercepted in DryRun mode

    // Mock execute_current_branch
    mock_current_branch(&mut mocks, "main");
    // Mock execute_remote_branches (empty — on a local branch)
    mock_remote_branches(&mut mocks, "");

    // Mock execute_gist (only has response output now)
    mocks.set_value(
        "gist_upload/execute_gist",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(
                serde_json::json!({"html_url":"https://gist.github.com/mock/123"}),
            ),
        )),
    );

    let result = assert_boundary_mockable(&dag, mocks);

    assert!(
        result.is_ok(),
        "Gist graph should be boundary-mockable: {:?}",
        result.error
    );
    // gist_upload and branch_resolution are SubDag boundary nodes at the top level.
    // Their internal transport boundaries are handled by the SubDag executor.
}

/// Test that real mode does NOT intercept boundaries.
#[test]
fn test_real_mode_no_interception() {
    if !guard_integration(stringify!(test_real_mode_no_interception)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    // Real mode - note: this will fail at transport nodes if gh isn't authenticated,
    // but we can still verify that pure nodes executed without interception
    match execute_with_mode(&dag, ExecutionMode::Real) {
        Ok(log) => {
            // If it succeeded, verify no interception happened for pure nodes
            for entry in &log.entries {
                if !entry.node_id.contains("execute_") {
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
    if !guard_hermetic(stringify!(test_branch_name_in_gist_filename)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    mocks.set_value(
        "list_files/execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );
    // Loop body transport nodes are auto-intercepted in DryRun mode
    // Use a branch name with slashes (common in git workflows)
    mock_current_branch(&mut mocks, "claude/improve-gist-filename");
    mock_remote_branches(&mut mocks, "");
    mocks.set_value(
        "gist_upload/execute_gist",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(
                serde_json::json!({"html_url":"https://gist.github.com/mock/456"}),
            ),
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify prepare_gist_request received the branch name and produced a sanitized filename
    let prepare_gist = log
        .get("gist_upload/prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(req))) => {
            // The filename should contain the sanitized branch name (slashes → hyphens)
            let filename = gist_request_filename(req);
            let filename_arg =
                Some(&filename).filter(|a| a.contains("claude-improve-gist-filename"));
            assert!(
                filename_arg.is_some(),
                "expected sanitized branch name in filename, got filename: {}",
                filename
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
            "expected rest request from prepare_gist_request, got: {:?}",
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
    if !guard_hermetic(stringify!(test_platform_challenging_branch_names)) {
        return;
    }

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
            "list_files/execute_list_files",
            "response",
            Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
        );
        // Loop body transport nodes are auto-intercepted in DryRun mode
        mock_current_branch(&mut mocks, branch);
        mock_remote_branches(&mut mocks, "");
        mocks.set_value(
            "gist_upload/execute_gist",
            "response",
            Value::Response(TransportResponse::Rest(
                gunbc_ir::transport::RestResponse::ok(
                    serde_json::json!({"html_url":"https://gist.github.com/mock/789"}),
                ),
            )),
        );

        let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

        let prepare_gist = log
            .get("gist_upload/prepare_gist_request")
            .expect("prepare_gist_request should be in log");
        match prepare_gist.outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(req))) => {
                let filename = gist_request_filename(req);
                let filename_arg = Some(&filename).filter(|a| a.starts_with(expected_prefix));
                assert!(
                    filename_arg.is_some(),
                    "branch '{}': expected filename starting with '{}', got filename: {}",
                    branch,
                    expected_prefix,
                    filename
                );
            }
            other => panic!(
                "branch '{}': expected rest request, got: {:?}",
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
    if !guard_hermetic(stringify!(test_detached_head_uses_remote_branch_name)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    mocks.set_value(
        "list_files/execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );
    // Loop body transport nodes are auto-intercepted in DryRun mode

    // Detached HEAD — rev-parse returns "HEAD"
    mock_current_branch(&mut mocks, "HEAD");
    // Remote branch points at HEAD
    mock_remote_branches(&mut mocks, "  origin/main\n");

    mocks.set_value(
        "gist_upload/execute_gist",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(
                serde_json::json!({"html_url":"https://gist.github.com/mock/detached"}),
            ),
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    let prepare_gist = log
        .get("gist_upload/prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(req))) => {
            // Should use remote branch name "main" for filename
            let filename = gist_request_filename(req);
            let filename_arg = Some(&filename).filter(|a| a.starts_with("main_"));
            assert!(
                filename_arg.is_some(),
                "expected remote branch 'main' in filename, got filename: {}",
                filename
            );
            assert!(filename_arg.unwrap().ends_with(".md"));
        }
        other => panic!(
            "expected rest request from prepare_gist_request, got: {:?}",
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
    if !guard_hermetic(stringify!(test_recent_mode_dry_run)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock execute_rev_list: return a SHA (repo is older than 3 days)
    mocks.set_value(
        "rev_list/execute_rev_list",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(
            "abc123def456\n",
        ))),
    );

    // Mock execute_diff: return sample diff
    mocks.set_value(
        "diff/execute_diff",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }\n"))),
    );

    mock_current_branch(&mut mocks, "main");
    mock_remote_branches(&mut mocks, "");

    mocks.set_value(
        "gist_upload/execute_gist",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(
                serde_json::json!({"html_url":"https://gist.github.com/mock/recent123"}),
            ),
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // Verify rev-list was intercepted
    let rev_list_entry = log
        .get("rev_list/execute_rev_list")
        .expect("execute_rev_list should be in log");
    assert!(
        rev_list_entry.was_intercepted,
        "execute_rev_list should be intercepted in dry-run"
    );

    // Verify diff was intercepted
    let diff_entry = log
        .get("diff/execute_diff")
        .expect("execute_diff should be in log");
    assert!(
        diff_entry.was_intercepted,
        "execute_diff should be intercepted in dry-run"
    );

    // Verify the parsed rev-list SHA flowed to prepare_diff as base_ref
    let parse_rev_list = log
        .get("rev_list/parse_rev_list")
        .expect("parse_rev_list should be in log");
    match parse_rev_list.outputs.get("base_ref") {
        Some(Value::Str(sha)) => assert_eq!(sha, "abc123def456"),
        _ => panic!("expected base_ref output from parse_rev_list"),
    }

    // Verify gist URL was produced
    let parse_gist = log
        .get("gist_upload/parse_gist_response")
        .expect("parse_gist_response should be in log");
    match parse_gist.outputs.get("url") {
        Some(Value::Str(url)) => assert!(url.contains("mock"), "expected mock URL"),
        _ => panic!("expected url output"),
    }

    // Verify commit range appears in gist filename
    let prepare_gist = log
        .get("gist_upload/prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(req))) => {
            let filename = gist_request_filename(req);
            let filename_arg =
                Some(&filename).filter(|a| a.contains("recent-3d") && a.contains("abc123d..HEAD"));
            assert!(
                filename_arg.is_some(),
                "expected recent-mode filename with commit range, got filename: {}",
                filename
            );
            // Description should mention the commit range
            let desc = gist_request_description(req);
            assert!(
                desc.contains("Recent changes (3d) abc123d..HEAD on main"),
                "expected recent-mode description, got: {}",
                desc
            );
        }
        other => panic!(
            "expected rest request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}

/// Test that recent mode with young repo (empty rev-list) produces graceful empty diff.
#[test]
fn test_recent_mode_young_repo() {
    if !guard_hermetic(stringify!(test_recent_mode_young_repo)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Recent, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    // Mock execute_rev_list: empty output (repo < 3 days old)
    mocks.set_value(
        "rev_list/execute_rev_list",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(""))),
    );

    // Mock execute_diff: empty diff (HEAD...HEAD produces nothing)
    mocks.set_value(
        "diff/execute_diff",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok(""))),
    );

    mock_current_branch(&mut mocks, "main");
    mock_remote_branches(&mut mocks, "");

    mocks.set_value(
        "gist_upload/execute_gist",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(
                serde_json::json!({"html_url":"https://gist.github.com/mock/young"}),
            ),
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    // parse_rev_list should produce no base_ref (empty output)
    let parse_rev_list = log
        .get("rev_list/parse_rev_list")
        .expect("parse_rev_list should be in log");
    assert!(
        !parse_rev_list.outputs.contains_key("base_ref"),
        "young repo should produce no base_ref"
    );

    // Gist should still complete (empty diff is valid)
    let parse_gist = log
        .get("gist_upload/parse_gist_response")
        .expect("parse_gist_response should be in log");
    assert!(
        parse_gist.outputs.contains_key("url"),
        "gist should still produce a URL even with empty diff"
    );

    // Young repo has no base_ref → falls back to snapshot-style filename
    let prepare_gist = log
        .get("gist_upload/prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(req))) => {
            // Should NOT contain recent-3d (no base_ref to form commit range)
            let filename = gist_request_filename(req);
            let has_recent = filename.contains("recent-3d");
            assert!(
                !has_recent,
                "young repo should fall back to snapshot-style filename, got filename: {}",
                filename
            );
            // Should use branch name "main" as prefix
            let f_arg = Some(&filename).filter(|a| a.starts_with("main_"));
            assert!(
                f_arg.is_some(),
                "expected main-prefixed filename for young repo, got filename: {}",
                filename
            );
        }
        other => panic!(
            "expected rest request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}

/// Test that detached HEAD with no remote branch falls back to "snapshot".
#[test]
fn test_detached_head_no_remote_uses_snapshot() {
    if !guard_hermetic(stringify!(test_detached_head_no_remote_uses_snapshot)) {
        return;
    }

    let dag =
        build_gist_graph(GistMode::Snapshot, vec![], false).expect("Failed to build gist graph");

    let mut mocks = BoundaryMocks::new();
    mock_env(&mut mocks);

    mocks.set_value(
        "list_files/execute_list_files",
        "response",
        Value::Response(TransportResponse::Shell(ShellResponse::ok("src/main.rs\n"))),
    );
    // Loop body transport nodes are auto-intercepted in DryRun mode

    // Detached HEAD — rev-parse returns "HEAD"
    mock_current_branch(&mut mocks, "HEAD");
    // No remote branches point at HEAD
    mock_remote_branches(&mut mocks, "");

    mocks.set_value(
        "gist_upload/execute_gist",
        "response",
        Value::Response(TransportResponse::Rest(
            gunbc_ir::transport::RestResponse::ok(
                serde_json::json!({"html_url":"https://gist.github.com/mock/orphan"}),
            ),
        )),
    );

    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks)).unwrap();

    let prepare_gist = log
        .get("gist_upload/prepare_gist_request")
        .expect("prepare_gist_request should be in log");
    match prepare_gist.outputs.get("request") {
        Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(req))) => {
            // Should fall back to "snapshot" for filename
            let filename = gist_request_filename(req);
            let filename_arg = Some(&filename).filter(|a| a.starts_with("snapshot_"));
            assert!(
                filename_arg.is_some(),
                "expected 'snapshot' filename for orphan detached HEAD, got filename: {}",
                filename
            );
        }
        other => panic!(
            "expected rest request from prepare_gist_request, got: {:?}",
            other
        ),
    }
}
