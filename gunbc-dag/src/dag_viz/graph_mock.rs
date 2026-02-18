//! Mock specification for the dag-viz tool.
//!
//! Uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! - `branch_resolution/execute_current_branch`: Transport for current branch
//! - `branch_resolution/execute_remote_branches`: Transport for remote branches
//! - `gist_upload/execute_gist`: Transport for gist creation
//! - `git_show_base/execute_git_show_base`: Transport for `git show <ref>:.dag-snapshots/...`
//! - `rev_list/execute_rev_list`: Transport for `git rev-list --before=...`
//! - `execute_write_snapshot`: Transport for file write
//!
//! # Input Expectations
//!
//! - `format`: String (snapshot/diff/recent modes)
//! - `base_ref`: String (diff mode only)

use crate::dag_viz::graph::{build_dag_viz_graph, DagVizMode};
use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::transport::{FileOp, FileResponse, ShellResponse, TransportResponse};
use gunbc_ir::{SecretString, Timestamp, Value};
use gunbc_primitives::filename;
use gunbc_test::{
    extract_mock_requirements, InputConstraint, MockSpec, NodeExample, OutputMatcher,
};
use std::collections::BTreeMap;
use std::time::SystemTime;

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

fn mock_clock() -> Value {
    Timestamp::from_system_time(SystemTime::UNIX_EPOCH).into()
}

fn mock_credential() -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "token".to_string(),
        Value::Secret(SecretString::new("<MOCK_GITHUB_TOKEN>")),
    );
    map.insert("source_type".to_string(), Value::Str("static".to_string()));
    map.insert("scheme".to_string(), Value::Str("bearer".to_string()));
    map.insert(
        "cap".to_string(),
        Value::Secret(SecretString::new("capability")),
    );
    Value::Map(map)
}

fn mock_cloud_config() -> Value {
    CloudSecretConfig {
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
    }
    .into()
}

fn mock_shell_ok(stdout: &str) -> TransportResponse {
    TransportResponse::Shell(ShellResponse::ok(stdout))
}

fn mock_empty_topology_json() -> &'static str {
    r#"{"nodes":[],"edges":[]}"#
}

fn mock_gist_response_json() -> String {
    serde_json::json!({
        "id": "abc123",
        "html_url": "https://gist.github.com/mock/abc123",
        "files": {},
        "public": false
    })
    .to_string()
}

/// Build a mock specification for the dag-viz graph.
fn dag_viz_mock_spec(mode: &DagVizMode) -> MockSpec {
    let dag = build_dag_viz_graph(mode.clone()).expect("dag-viz graph should build");

    let mut reqs = extract_mock_requirements(&dag, "dag-viz")
        // Delegate cloud_credential internal mocks to include_prefixed_runtime_mocks
        .exclude_prefix("gist_upload/cloud_credential/gcp_wif_secret")
        // Top-level filesystem environment
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type");

    match mode {
        DagVizMode::Snapshot => {
            // Gist upload SubDag internal environments
            reqs = reqs
                .boundary("gist_upload/fs_env", "file:write", mock_fs_handle())
                .expect("gist_upload fs_env should match type")
                .boundary("gist_upload/clock_env", "clock", mock_clock())
                .expect("gist_upload clock_env should match type")
                .boundary("gist_upload/cloud_env", "config", mock_cloud_config())
                .expect("gist_upload cloud_env config should match type")
                .boundary(
                    "gist_upload/cloud_env",
                    "request_url",
                    Value::Str("https://example.com/oidc".into()),
                )
                .expect("gist_upload cloud_env request_url should match type")
                .boundary(
                    "gist_upload/cloud_env",
                    "request_token",
                    Value::Str("mock-oidc-token".into()),
                )
                .expect("gist_upload cloud_env request_token should match type")
                .boundary("gist_upload/bind_secret", "config", mock_cloud_config())
                .expect("gist_upload bind_secret config should match type")
                .boundary(
                    "gist_upload/cloud_credential/gcp_wif_secret/build_credential",
                    "credential",
                    mock_credential(),
                )
                .expect("gist_upload cloud_credential credential should match type")
                .boundary(
                    "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
                    "ok",
                    Value::Bool(true),
                )
                .expect("gist_upload cloud_credential ok should match type");

            // Branch resolution transports
            reqs = reqs
                .transport_response(
                    "branch_resolution/execute_current_branch",
                    "response",
                    mock_shell_ok("main\n"),
                )
                .expect("branch_resolution current_branch response should match")
                .transport_response(
                    "branch_resolution/execute_remote_branches",
                    "response",
                    mock_shell_ok("  origin/main\n"),
                )
                .expect("branch_resolution remote_branches response should match");

            // Gist upload transport
            reqs = reqs
                .transport_response(
                    "gist_upload/execute_gist",
                    "response",
                    TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(
                        serde_json::from_str::<serde_json::Value>(&mock_gist_response_json())
                            .expect("mock gist response json should parse"),
                    )),
                )
                .expect("gist_upload execute_gist response should match");

            // Local save + browser open transports
            reqs = reqs
                .transport_response(
                    "local_save/execute_local_save",
                    "response",
                    TransportResponse::File(FileResponse {
                        path: "target/dag-viz/dag-visualization.html".into(),
                        operation: FileOp::Write,
                        success: true,
                        content: None,
                        exists: Some(true),
                        error: None,
                    }),
                )
                .expect("local_save response should match")
                .transport_response(
                    "browser_open/execute_browser_open",
                    "response",
                    mock_shell_ok(""),
                )
                .expect("browser_open response should match");
        }
        DagVizMode::Diff { .. } => {
            // Gist upload SubDag internal environments
            reqs = reqs
                .boundary("gist_upload/fs_env", "file:write", mock_fs_handle())
                .expect("gist_upload fs_env should match type")
                .boundary("gist_upload/clock_env", "clock", mock_clock())
                .expect("gist_upload clock_env should match type")
                .boundary("gist_upload/cloud_env", "config", mock_cloud_config())
                .expect("gist_upload cloud_env config should match type")
                .boundary(
                    "gist_upload/cloud_env",
                    "request_url",
                    Value::Str("https://example.com/oidc".into()),
                )
                .expect("gist_upload cloud_env request_url should match type")
                .boundary(
                    "gist_upload/cloud_env",
                    "request_token",
                    Value::Str("mock-oidc-token".into()),
                )
                .expect("gist_upload cloud_env request_token should match type")
                .boundary("gist_upload/bind_secret", "config", mock_cloud_config())
                .expect("gist_upload bind_secret config should match type")
                .boundary(
                    "gist_upload/cloud_credential/gcp_wif_secret/build_credential",
                    "credential",
                    mock_credential(),
                )
                .expect("gist_upload cloud_credential credential should match type")
                .boundary(
                    "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
                    "ok",
                    Value::Bool(true),
                )
                .expect("gist_upload cloud_credential ok should match type");

            // Branch resolution transports
            reqs = reqs
                .transport_response(
                    "branch_resolution/execute_current_branch",
                    "response",
                    mock_shell_ok("main\n"),
                )
                .expect("branch_resolution current_branch response should match")
                .transport_response(
                    "branch_resolution/execute_remote_branches",
                    "response",
                    mock_shell_ok("  origin/main\n"),
                )
                .expect("branch_resolution remote_branches response should match");

            // Git show base topology transport
            reqs = reqs
                .transport_response(
                    "git_show_base/execute_git_show_base",
                    "response",
                    mock_shell_ok(mock_empty_topology_json()),
                )
                .expect("git_show_base response should match");

            // Gist upload transport
            reqs = reqs
                .transport_response(
                    "gist_upload/execute_gist",
                    "response",
                    TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(
                        serde_json::from_str::<serde_json::Value>(&mock_gist_response_json())
                            .expect("mock gist response json should parse"),
                    )),
                )
                .expect("gist_upload execute_gist response should match");
        }
        DagVizMode::Recent => {
            // Gist upload SubDag internal environments
            reqs = reqs
                .boundary("gist_upload/fs_env", "file:write", mock_fs_handle())
                .expect("gist_upload fs_env should match type")
                .boundary("gist_upload/clock_env", "clock", mock_clock())
                .expect("gist_upload clock_env should match type")
                .boundary("gist_upload/cloud_env", "config", mock_cloud_config())
                .expect("gist_upload cloud_env config should match type")
                .boundary(
                    "gist_upload/cloud_env",
                    "request_url",
                    Value::Str("https://example.com/oidc".into()),
                )
                .expect("gist_upload cloud_env request_url should match type")
                .boundary(
                    "gist_upload/cloud_env",
                    "request_token",
                    Value::Str("mock-oidc-token".into()),
                )
                .expect("gist_upload cloud_env request_token should match type")
                .boundary("gist_upload/bind_secret", "config", mock_cloud_config())
                .expect("gist_upload bind_secret config should match type")
                .boundary(
                    "gist_upload/cloud_credential/gcp_wif_secret/build_credential",
                    "credential",
                    mock_credential(),
                )
                .expect("gist_upload cloud_credential credential should match type")
                .boundary(
                    "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
                    "ok",
                    Value::Bool(true),
                )
                .expect("gist_upload cloud_credential ok should match type");

            // Branch resolution transports
            reqs = reqs
                .transport_response(
                    "branch_resolution/execute_current_branch",
                    "response",
                    mock_shell_ok("main\n"),
                )
                .expect("branch_resolution current_branch response should match")
                .transport_response(
                    "branch_resolution/execute_remote_branches",
                    "response",
                    mock_shell_ok("  origin/main\n"),
                )
                .expect("branch_resolution remote_branches response should match");

            // Rev-list transport
            reqs = reqs
                .transport_response(
                    "rev_list/execute_rev_list",
                    "response",
                    mock_shell_ok("abc123def456\n"),
                )
                .expect("rev_list response should match");

            // Git show base topology transport
            reqs = reqs
                .transport_response(
                    "git_show_base/execute_git_show_base",
                    "response",
                    mock_shell_ok(mock_empty_topology_json()),
                )
                .expect("git_show_base response should match");

            // Gist upload transport
            reqs = reqs
                .transport_response(
                    "gist_upload/execute_gist",
                    "response",
                    TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(
                        serde_json::from_str::<serde_json::Value>(&mock_gist_response_json())
                            .expect("mock gist response json should parse"),
                    )),
                )
                .expect("gist_upload execute_gist response should match");
        }
        DagVizMode::SaveSnapshot => {
            reqs = reqs
                .transport_response(
                    "execute_write_snapshot",
                    "response",
                    TransportResponse::File(FileResponse {
                        path: ".dag-snapshots/workspace.json".into(),
                        operation: FileOp::Write,
                        success: true,
                        content: Some(mock_empty_topology_json().to_string()),
                        exists: Some(true),
                        error: None,
                    }),
                )
                .expect("write_snapshot response should match");
        }
    }

    // Terminal boundary mocks: outputs that are DAG sinks
    match mode {
        DagVizMode::Snapshot => {
            reqs = reqs
                .boundary_str(
                    "gist_upload/parse_gist_response",
                    "url",
                    "https://gist.github.com/mock/abc123",
                )
                .expect("gist_upload parse_gist_response.url mock should match type")
                .boundary(
                    "browser_open/parse_browser_open",
                    "opened",
                    Value::Bool(true),
                )
                .expect("parse_browser_open.opened mock should match type");
        }
        DagVizMode::Diff { .. } | DagVizMode::Recent => {
            reqs = reqs
                .boundary_str(
                    "gist_upload/parse_gist_response",
                    "url",
                    "https://gist.github.com/mock/abc123",
                )
                .expect("gist_upload parse_gist_response.url mock should match type");
        }
        DagVizMode::SaveSnapshot => {
            reqs = reqs
                .boundary_str(
                    "parse_write_result",
                    "summary",
                    "Saved DAG topology snapshot to .dag-snapshots/workspace.json (0 workflows, 0 total nodes)",
                )
                .expect("parse_write_result.summary mock should match type");
        }
    }

    let mut spec = reqs.build_unchecked();

    // Only include cloud credential runtime mocks for modes that use gist_upload
    if !matches!(mode, DagVizMode::SaveSnapshot) {
        spec = spec.include_prefixed_runtime_mocks(
            "gist_upload/cloud_credential/gcp_wif_secret",
            &gunbc_lib_gcp_ops::graph_mock::gcp_local_mock_spec(),
        );
    }

    // Input mocks for entry points
    match mode {
        DagVizMode::Snapshot => {
            spec = spec
                .input_mock(
                    "branch_resolution/prepare_current_branch",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock(
                    "branch_resolution/prepare_remote_branches",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock("render_snapshot", "format", Value::Str("html".into()))
                .expects_input("repo_path", InputConstraint::Any)
                .expects_input("format", InputConstraint::Any);
        }
        DagVizMode::Diff { .. } => {
            spec = spec
                .input_mock(
                    "branch_resolution/prepare_current_branch",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock(
                    "branch_resolution/prepare_remote_branches",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock("diff_and_render", "base_ref", Value::Str("main".into()))
                .expects_input("repo_path", InputConstraint::Any)
                .expects_input("format", InputConstraint::Any)
                .expects_input("base_ref", InputConstraint::Any);
        }
        DagVizMode::Recent => {
            spec = spec
                .input_mock(
                    "branch_resolution/prepare_current_branch",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock(
                    "branch_resolution/prepare_remote_branches",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock(
                    "rev_list/prepare_rev_list",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .expects_input("repo_path", InputConstraint::Any)
                .expects_input("format", InputConstraint::Any);
        }
        DagVizMode::SaveSnapshot => {
            // No entry points for save-snapshot
        }
    }

    // Node examples for verification
    spec = spec
        .node_example(
            NodeExample::new("fs_env")
                .output("file:write", OutputMatcher::Any)
                .description("Provides filesystem handle"),
        )
        .node_example(
            NodeExample::new("build_topology")
                .output("topology_json", OutputMatcher::non_empty())
                .output("node_count", OutputMatcher::IntGe(0))
                .description("Builds workspace DAG and extracts topology"),
        );

    // Mode-specific node examples / skips
    match mode {
        DagVizMode::Snapshot => {
            spec = spec
                // Branch resolution (inside SubDag)
                .node_example(
                    NodeExample::new("branch_resolution/prepare_current_branch")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-parse request for current branch"),
                )
                .node_example(
                    NodeExample::new("branch_resolution/parse_current_branch")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("main\n").into()),
                        )
                        .output("branch", OutputMatcher::non_empty())
                        .description("Parses branch name from git rev-parse response"),
                )
                // Tool-specific pure nodes
                .node_example(
                    NodeExample::new("render_snapshot")
                        .input(
                            "topology_json",
                            Value::Str(mock_empty_topology_json().into()),
                        )
                        .input("branch", Value::Str("main".into()))
                        .input("format", Value::Str("html".into()))
                        .output("content", OutputMatcher::non_empty())
                        .output("ext", OutputMatcher::non_empty())
                        .description("Renders topology as HTML or markdown"),
                )
                // Gist upload (inside SubDag)
                .node_example(
                    NodeExample::new("gist_upload/prepare_gist_request")
                        .input("markdown", Value::Str("<html>mock</html>".into()))
                        .input("branch", Value::Str("main".into()))
                        .input("res:file", mock_fs_handle())
                        .input("res:clock", mock_clock())
                        .output("request", OutputMatcher::IsRequest)
                        .description("Builds gist creation request from content"),
                )
                .node_example(
                    NodeExample::new("gist_upload/parse_gist_response")
                        .input(
                            "response",
                            Value::Response(TransportResponse::Rest(
                                gunbc_ir::transport::RestResponse::ok(
                                    serde_json::from_str::<serde_json::Value>(
                                        &mock_gist_response_json(),
                                    )
                                    .expect("mock gist response json should parse"),
                                ),
                            )),
                        )
                        .output("url", OutputMatcher::contains("gist.github.com"))
                        .description("Extracts gist URL from response"),
                )
                // Local save + browser open
                .node_example(
                    NodeExample::new("local_save/prepare_local_save")
                        .input("content", Value::Str("<html>mock</html>".into()))
                        .input("ext", Value::Str("html".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares file write request for local HTML"),
                )
                .node_example(
                    NodeExample::new("local_save/parse_local_save")
                        .input(
                            "response",
                            Value::Response(TransportResponse::File(FileResponse {
                                path: "target/dag-viz/dag-visualization.html".into(),
                                operation: FileOp::Write,
                                success: true,
                                content: None,
                                exists: Some(true),
                                error: None,
                            })),
                        )
                        .output("file_path", OutputMatcher::non_empty())
                        .description("Extracts local file path from write response"),
                )
                .node_example(
                    NodeExample::new("browser_open/prepare_browser_open")
                        .input(
                            "file_path",
                            Value::Str("target/dag-viz/dag-visualization.html".into()),
                        )
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares xdg-open/open command for browser"),
                )
                .node_example(
                    NodeExample::new("browser_open/parse_browser_open")
                        .input("response", Value::Response(ShellResponse::ok("").into()))
                        .output("opened", OutputMatcher::IsBool)
                        .description("Confirms browser open succeeded"),
                )
                .live_expected_output(
                    "gist_upload/parse_gist_response",
                    "url",
                    OutputMatcher::NonEmpty,
                )
                .live_expected_output(
                    "browser_open/parse_browser_open",
                    "opened",
                    OutputMatcher::IsBool,
                )
                .live_expected_output(
                    "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
                    "ok",
                    OutputMatcher::IsBool,
                )
                .skip_node_example("gist_upload/fs_env")
                .skip_node_example("gist_upload/clock_env")
                .skip_node_example("gist_upload/cloud_env")
                .skip_node_example("gist_upload/cloud_credential")
                .skip_node_example("gist_upload/bind_secret")
                .skip_node_example("gist_upload/scope_preflight")
                .skip_node_example("gist_upload/resolve_auth")
                .skip_node_example("branch_resolution/prepare_remote_branches")
                .skip_node_example("branch_resolution/parse_remote_branches");
        }
        DagVizMode::Diff { .. } => {
            spec = spec
                // Branch resolution (inside SubDag)
                .node_example(
                    NodeExample::new("branch_resolution/prepare_current_branch")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-parse request for current branch"),
                )
                .node_example(
                    NodeExample::new("branch_resolution/parse_current_branch")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("main\n").into()),
                        )
                        .output("branch", OutputMatcher::non_empty())
                        .description("Parses branch name from git rev-parse response"),
                )
                // Git show (still a triplet)
                .node_example(
                    NodeExample::new("git_show_base/prepare_git_show_base")
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git show request for base topology"),
                )
                .node_example(
                    NodeExample::new("git_show_base/parse_git_show_base")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok(mock_empty_topology_json()).into()),
                        )
                        .output("content", OutputMatcher::non_empty())
                        .description("Parses git show response into file content"),
                )
                .node_example(
                    NodeExample::new("parse_base_topology")
                        .input("content", Value::Str(mock_empty_topology_json().into()))
                        .output("topology_json", OutputMatcher::non_empty())
                        .description("Validates content as DagTopology JSON"),
                )
                .node_example(
                    NodeExample::new("diff_and_render")
                        .input(
                            "current_json",
                            Value::Str(mock_empty_topology_json().into()),
                        )
                        .input("base_json", Value::Str(mock_empty_topology_json().into()))
                        .input("branch", Value::Str("main".into()))
                        .input("base_ref", Value::Str("main".into()))
                        .output("content", OutputMatcher::non_empty())
                        .description("Diffs topologies and renders as markdown"),
                )
                // Gist upload (inside SubDag)
                .node_example(
                    NodeExample::new("gist_upload/prepare_gist_request")
                        .input("markdown", Value::Str("# Mock diff".into()))
                        .input("branch", Value::Str("main".into()))
                        .input("res:file", mock_fs_handle())
                        .input("res:clock", mock_clock())
                        .output("request", OutputMatcher::IsRequest)
                        .description("Builds gist creation request from content"),
                )
                .node_example(
                    NodeExample::new("gist_upload/parse_gist_response")
                        .input(
                            "response",
                            Value::Response(TransportResponse::Rest(
                                gunbc_ir::transport::RestResponse::ok(
                                    serde_json::from_str::<serde_json::Value>(
                                        &mock_gist_response_json(),
                                    )
                                    .expect("mock gist response json should parse"),
                                ),
                            )),
                        )
                        .output("url", OutputMatcher::contains("gist.github.com"))
                        .description("Extracts gist URL from response"),
                )
                .live_expected_output(
                    "gist_upload/parse_gist_response",
                    "url",
                    OutputMatcher::NonEmpty,
                )
                .live_expected_output(
                    "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
                    "ok",
                    OutputMatcher::IsBool,
                )
                .skip_node_example("gist_upload/fs_env")
                .skip_node_example("gist_upload/clock_env")
                .skip_node_example("gist_upload/cloud_env")
                .skip_node_example("gist_upload/cloud_credential")
                .skip_node_example("gist_upload/bind_secret")
                .skip_node_example("gist_upload/scope_preflight")
                .skip_node_example("gist_upload/resolve_auth")
                .skip_node_example("branch_resolution/prepare_remote_branches")
                .skip_node_example("branch_resolution/parse_remote_branches");
        }
        DagVizMode::Recent => {
            spec = spec
                // Branch resolution (inside SubDag)
                .node_example(
                    NodeExample::new("branch_resolution/prepare_current_branch")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-parse request for current branch"),
                )
                .node_example(
                    NodeExample::new("branch_resolution/parse_current_branch")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("main\n").into()),
                        )
                        .output("branch", OutputMatcher::non_empty())
                        .description("Parses branch name from git rev-parse response"),
                )
                .node_example(
                    NodeExample::new("rev_list/prepare_rev_list")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-list request for recent commit"),
                )
                .node_example(
                    NodeExample::new("rev_list/parse_rev_list")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("abc123def456\n").into()),
                        )
                        .output("base_ref", OutputMatcher::non_empty())
                        .description("Parses commit hash from rev-list response"),
                )
                // Git show (still a triplet)
                .node_example(
                    NodeExample::new("git_show_base/prepare_git_show_base")
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git show request for base topology"),
                )
                .node_example(
                    NodeExample::new("git_show_base/parse_git_show_base")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok(mock_empty_topology_json()).into()),
                        )
                        .output("content", OutputMatcher::non_empty())
                        .description("Parses git show response into file content"),
                )
                .node_example(
                    NodeExample::new("parse_base_topology")
                        .input("content", Value::Str(mock_empty_topology_json().into()))
                        .output("topology_json", OutputMatcher::non_empty())
                        .description("Validates content as DagTopology JSON"),
                )
                .node_example(
                    NodeExample::new("diff_and_render")
                        .input(
                            "current_json",
                            Value::Str(mock_empty_topology_json().into()),
                        )
                        .input("base_json", Value::Str(mock_empty_topology_json().into()))
                        .input("branch", Value::Str("main".into()))
                        .input("base_ref", Value::Str("abc123def456".into()))
                        .output("content", OutputMatcher::non_empty())
                        .description("Diffs topologies and renders as markdown"),
                )
                // Gist upload (inside SubDag)
                .node_example(
                    NodeExample::new("gist_upload/prepare_gist_request")
                        .input("markdown", Value::Str("# Mock diff".into()))
                        .input("branch", Value::Str("main".into()))
                        .input("res:file", mock_fs_handle())
                        .input("res:clock", mock_clock())
                        .output("request", OutputMatcher::IsRequest)
                        .description("Builds gist creation request from content"),
                )
                .node_example(
                    NodeExample::new("gist_upload/parse_gist_response")
                        .input(
                            "response",
                            Value::Response(TransportResponse::Rest(
                                gunbc_ir::transport::RestResponse::ok(
                                    serde_json::from_str::<serde_json::Value>(
                                        &mock_gist_response_json(),
                                    )
                                    .expect("mock gist response json should parse"),
                                ),
                            )),
                        )
                        .output("url", OutputMatcher::contains("gist.github.com"))
                        .description("Extracts gist URL from response"),
                )
                .live_expected_output(
                    "gist_upload/parse_gist_response",
                    "url",
                    OutputMatcher::NonEmpty,
                )
                .live_expected_output(
                    "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
                    "ok",
                    OutputMatcher::IsBool,
                )
                .skip_node_example("gist_upload/fs_env")
                .skip_node_example("gist_upload/clock_env")
                .skip_node_example("gist_upload/cloud_env")
                .skip_node_example("gist_upload/cloud_credential")
                .skip_node_example("gist_upload/bind_secret")
                .skip_node_example("gist_upload/scope_preflight")
                .skip_node_example("gist_upload/resolve_auth")
                .skip_node_example("branch_resolution/prepare_remote_branches")
                .skip_node_example("branch_resolution/parse_remote_branches");
        }
        DagVizMode::SaveSnapshot => {
            spec = spec
                .node_example(
                    NodeExample::new("prepare_write_snapshot")
                        .input(
                            "topology_json",
                            Value::Str(mock_empty_topology_json().into()),
                        )
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares file write request for topology JSON"),
                )
                .node_example(
                    NodeExample::new("parse_write_result")
                        .input(
                            "response",
                            Value::Response(TransportResponse::File(FileResponse {
                                path: ".dag-snapshots/workspace.json".into(),
                                operation: FileOp::Write,
                                success: true,
                                content: None,
                                exists: Some(true),
                                error: None,
                            })),
                        )
                        .input("node_count", Value::Int(0))
                        .input("total_node_count", Value::Int(0))
                        .output("summary", OutputMatcher::contains("Saved DAG topology"))
                        .description("Parses file write response into summary"),
                )
                .live_expected_output("parse_write_result", "summary", OutputMatcher::NonEmpty);
        }
    }

    spec
}

/// Mock spec for snapshot mode.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-viz-snapshot",
    output = "gunbc-dag/src/dag_viz/generated_tests_snapshot.rs",
    module = "dag_viz_snapshot_generated_tests",
    builder = "crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::Snapshot).unwrap()",
    signature = "crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::Snapshot)",
    tool = "dag-viz"
)]
pub fn dag_viz_snapshot_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::Snapshot)
}

/// Mock spec for diff mode.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-viz-diff",
    output = "gunbc-dag/src/dag_viz/generated_tests_diff.rs",
    module = "dag_viz_diff_generated_tests",
    builder = r#"crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::Diff { base_ref: "main".to_string() }).unwrap()"#,
    signature = r#"crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::Diff { base_ref: "main".to_string() })"#,
    tool = "dag-viz-diff"
)]
pub fn dag_viz_diff_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::Diff {
        base_ref: "main".to_string(),
    })
}

/// Mock spec for recent mode.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-viz-recent",
    output = "gunbc-dag/src/dag_viz/generated_tests_recent.rs",
    module = "dag_viz_recent_generated_tests",
    builder = "crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::Recent).unwrap()",
    signature = "crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::Recent)",
    tool = "dag-viz-recent"
)]
pub fn dag_viz_recent_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::Recent)
}

/// Mock spec for save-snapshot mode.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-snapshot",
    output = "gunbc-dag/src/dag_viz/generated_tests_snapshot_save.rs",
    module = "dag_snapshot_generated_tests",
    builder = "crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::SaveSnapshot).unwrap()",
    signature = "crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::SaveSnapshot)",
    tool = "dag-snapshot"
)]
pub fn dag_viz_save_snapshot_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::SaveSnapshot)
}
