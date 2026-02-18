//! Mock specification for the gist tool.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! Used by testgen for:
//! - Dry-run testing with realistic mock values
//! - Chain validation with other tools

use crate::graph::{build_gist_graph, GistMode};
use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::transport::gist::GITHUB_SECRET_ID;
use gunbc_ir::transport::{ShellResponse, TransportResponse};
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

fn mock_diff_response() -> &'static str {
    "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }\n"
}

fn mock_diff_files_value() -> Value {
    let mut map = BTreeMap::new();
    map.insert("src/main.rs".to_string(), mock_diff_response().to_string());
    Value::str_map(map)
}

fn mock_contents_value() -> Value {
    let mut map = BTreeMap::new();
    map.insert("src/main.rs".to_string(), "fn main() {}".to_string());
    map.insert("README.md".to_string(), "# README".to_string());
    Value::str_map(map)
}

fn mock_gist_response_json() -> String {
    serde_json::json!({
        "id": "abc123def456",
        "html_url": "https://gist.github.com/mock/abc123def456",
        "files": {},
        "public": false
    })
    .to_string()
}

/// Build a mock specification for the gist graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
///
/// # Boundary Mocks
///
/// **Snapshot mode:**
/// - `execute_list_files`: Lists files via git ls-files
/// - `read_files_loop`: Per-file reads via LoopBuilder (transport inside loop body)
/// - `execute_gist`: Creates the gist (world write)
///
/// **Diff mode:**
/// - `execute_diff`: Runs `git diff base...HEAD`
/// - `execute_gist`: Creates the gist (world write)
///
/// # Input Expectations
///
/// - `repo_path`: String (required)
/// - `base_ref`: Optional string (diff mode only)
fn gist_mock_spec(mode: &GistMode) -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_gist_graph(mode.clone(), vec![], false).expect("gist graph should build");

    // Extract typed requirements from DAG structure
    let mut reqs = extract_mock_requirements(&dag, "gist")
        // Delegate cloud_credential internal mocks to include_prefixed_runtime_mocks
        .exclude_prefix("gist_upload/cloud_credential/gcp_wif_secret")
        // Top-level environment: filesystem (for content acquisition)
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("file:write mock should match type")
        // Gist upload SubDag internal environment nodes
        .boundary("gist_upload/fs_env", "file:write", mock_fs_handle())
        .expect("gist_upload file:write mock should match type")
        .boundary("gist_upload/clock_env", "clock", mock_clock())
        .expect("clock mock should match type")
        .boundary("gist_upload/cloud_env", "config", mock_cloud_config())
        .expect("cloud_env config should match type")
        .boundary(
            "gist_upload/cloud_env",
            "request_url",
            Value::Str("https://example.com/oidc".into()),
        )
        .expect("cloud_env request_url should match type")
        .boundary(
            "gist_upload/cloud_env",
            "request_token",
            Value::Str("mock-oidc-token".into()),
        )
        .expect("cloud_env request_token should match type")
        .boundary("gist_upload/bind_secret", "config", mock_cloud_config())
        .expect("bind_secret config should match type")
        .boundary(
            "gist_upload/cloud_credential/gcp_wif_secret/build_credential",
            "credential",
            mock_credential(),
        )
        .expect("cloud_credential credential should match type")
        .boundary(
            "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
            "ok",
            Value::Bool(true),
        )
        .expect("cloud_credential ok should match type");

    // Mode-specific transport mocks
    match mode {
        GistMode::Snapshot => {
            reqs = reqs
                // execute_list_files transport response
                .transport_response(
                    "list_files/execute_list_files",
                    "response",
                    // Empty list in DryRun to avoid loop-body transport mocks.
                    TransportResponse::Shell(ShellResponse::ok("")),
                )
                .expect("execute_list_files response should match type");
        }
        GistMode::Diff { .. } => {
            reqs = reqs
                // execute_diff transport response
                .transport_response(
                    "diff/execute_diff",
                    "response",
                    TransportResponse::Shell(ShellResponse::ok(mock_diff_response())),
                )
                .expect("execute_diff response should match type");
        }
        GistMode::Recent => {
            reqs = reqs
                // execute_rev_list transport response (SHA of commit 3 days ago)
                .transport_response(
                    "rev_list/execute_rev_list",
                    "response",
                    TransportResponse::Shell(ShellResponse::ok("abc123def456\n")),
                )
                .expect("execute_rev_list response should match type")
                // execute_diff transport response
                .transport_response(
                    "diff/execute_diff",
                    "response",
                    TransportResponse::Shell(ShellResponse::ok(mock_diff_response())),
                )
                .expect("execute_diff response should match type");
        }
    }

    // Shared: current branch acquisition (inside branch_resolution SubDag)
    reqs = reqs
        .transport_response(
            "branch_resolution/execute_current_branch",
            "response",
            TransportResponse::Shell(ShellResponse::ok("main\n")),
        )
        .expect("execute_current_branch response should match type");

    // Shared: remote branch resolution (inside branch_resolution SubDag)
    reqs = reqs
        .transport_response(
            "branch_resolution/execute_remote_branches",
            "response",
            TransportResponse::Shell(ShellResponse::ok("  origin/main\n")),
        )
        .expect("execute_remote_branches response should match type");

    // Shared: gist creation (inside gist_upload SubDag)
    reqs = reqs
        .transport_response(
            "gist_upload/execute_gist",
            "response",
            TransportResponse::Rest(gunbc_ir::transport::RestResponse::ok(
                serde_json::from_str::<serde_json::Value>(&mock_gist_response_json())
                    .expect("mock gist response json should parse"),
            )),
        )
        .expect("execute_gist response should match type");

    // Terminal boundary: parse_gist_response.url (inside gist_upload SubDag)
    reqs = reqs
        .boundary_str(
            "gist_upload/parse_gist_response",
            "url",
            "https://gist.github.com/mock/abc123def456",
        )
        .expect("url mock should match type");

    // Build spec (with input expectations added via legacy API)
    let mut spec = reqs.build_unchecked().include_prefixed_runtime_mocks(
        "gist_upload/cloud_credential/gcp_wif_secret",
        &gunbc_lib_gcp_ops::graph_mock::gcp_local_mock_spec(),
    );

    spec = spec.expects_input("repo_path", InputConstraint::Any);
    // Provide a default repo_path for entrypoint injection in DryRun tests.
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
        );
    match mode {
        GistMode::Snapshot => {
            spec = spec
                .input_mock(
                    "list_files/prepare_list_files",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock("read_files_loop", "repo_path", Value::Str(".".into()));
        }
        GistMode::Diff { .. } => {
            spec = spec.input_mock("diff/prepare_diff", "repo_path", Value::Str(".".into()));
        }
        GistMode::Recent => {
            spec = spec
                .input_mock(
                    "rev_list/prepare_rev_list",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock("diff/prepare_diff", "repo_path", Value::Str(".".into()));
        }
    }
    if matches!(mode, GistMode::Diff { .. }) {
        spec = spec.expects_input("base_ref", InputConstraint::Any);
    }

    // Common node examples (present in all modes)
    spec = spec
        .node_example(
            NodeExample::new("fs_env")
                .output("file:write", OutputMatcher::Any)
                .description("Provides filesystem handle for content acquisition"),
        )
        .node_example(
            NodeExample::new("gist_upload/fs_env")
                .output("file:write", OutputMatcher::Any)
                .description("Provides filesystem handle for gist filename generation"),
        )
        .node_example(
            NodeExample::new("gist_upload/clock_env")
                .output("clock", OutputMatcher::IsInt)
                .description("Provides timestamp for gist filename generation"),
        )
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
                .output("branch", OutputMatcher::exact(Value::Str("main".into())))
                .description("Parses current branch name from git output"),
        )
        .node_example(
            NodeExample::new("branch_resolution/prepare_remote_branches")
                .input("repo_path", Value::Str(".".into()))
                .output("request", OutputMatcher::IsRequest)
                .description("Prepares git branch -r --points-at HEAD request"),
        )
        .node_example(
            NodeExample::new("branch_resolution/parse_remote_branches")
                .input(
                    "response",
                    Value::Response(ShellResponse::ok("  origin/main\n").into()),
                )
                .output(
                    "remote_branch",
                    OutputMatcher::exact(Value::Str("main".into())),
                )
                .description("Parses remote branch name from git output"),
        )
        .node_example(
            NodeExample::new("gist_upload/resolve_auth")
                .output("service", OutputMatcher::exact(Value::Str("github".into())))
                .output(
                    "secret_name",
                    OutputMatcher::exact(Value::Str(GITHUB_SECRET_ID.into())),
                )
                .output("scheme", OutputMatcher::exact(Value::Str("bearer".into())))
                .output(
                    "interactive_allowed",
                    OutputMatcher::exact(Value::Bool(true)),
                )
                .output(
                    "required_scopes",
                    OutputMatcher::exact(Value::str_list(vec!["gist:write".into()])),
                )
                .description("Resolves typed gist scope contract into auth intent"),
        )
        .node_example(
            NodeExample::new("gist_upload/prepare_gist_request")
                .input("markdown", Value::Str("# Example".into()))
                .input("branch", Value::Str("main".into()))
                .input("res:file", mock_fs_handle())
                .input("res:clock", mock_clock())
                .output("request", OutputMatcher::IsRequest)
                .description("Builds gist creation request from markdown"),
        )
        .node_example(
            NodeExample::new("gist_upload/parse_gist_response")
                .input(
                    "response",
                    Value::Response(TransportResponse::Rest(
                        gunbc_ir::transport::RestResponse::ok(
                            serde_json::from_str::<serde_json::Value>(&mock_gist_response_json())
                                .expect("mock gist response json should parse"),
                        ),
                    )),
                )
                .output("url", OutputMatcher::contains("gist.github.com"))
                .description("Extracts gist URL from response JSON"),
        )
        // Probe-observer: terminals need chain-safe observers
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
        .skip_node_example("gist_upload/cloud_env")
        .skip_node_example("gist_upload/cloud_credential")
        .skip_node_example("gist_upload/bind_secret")
        .skip_node_example("gist_upload/scope_preflight");

    // Mode-specific node examples
    match mode {
        GistMode::Snapshot => {
            spec = spec
                .skip_node_example("read_files_loop")
                .node_example(
                    NodeExample::new("list_files/prepare_list_files")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git ls-files request"),
                )
                .node_example(
                    NodeExample::new("list_files/parse_list_files")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("src/main.rs\nREADME.md\n").into()),
                        )
                        .output(
                            "files",
                            OutputMatcher::exact(Value::str_list(vec![
                                "src/main.rs".into(),
                                "README.md".into(),
                            ])),
                        )
                        .description("Parses git ls-files output into a file list"),
                )
                .node_example(
                    NodeExample::new("collect_file_contents")
                        .input(
                            "filenames",
                            Value::str_list(vec!["src/main.rs".into(), "README.md".into()]),
                        )
                        .input(
                            "contents_list",
                            Value::str_list(vec!["fn main() {}".into(), "".into()]),
                        )
                        .output(
                            "contents",
                            OutputMatcher::exact(Value::str_map({
                                let mut map = BTreeMap::new();
                                map.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                                map
                            })),
                        )
                        .description(
                            "Zips filenames + contents into a map, skipping empty content",
                        ),
                )
                .node_example(
                    NodeExample::new("render_markdown")
                        .input("contents", mock_contents_value())
                        .output("markdown", OutputMatcher::contains("# Code Snapshot"))
                        .description("Renders markdown code snapshot"),
                );
        }
        GistMode::Diff { .. } => {
            spec = spec
                .node_example(
                    NodeExample::new("diff/prepare_diff")
                        .input("repo_path", Value::Str(".".into()))
                        .input("base_ref", Value::Str("main".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git diff request"),
                )
                .node_example(
                    NodeExample::new("diff/parse_diff")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok(mock_diff_response()).into()),
                        )
                        .output("diff_files", OutputMatcher::Any)
                        .output("stats", OutputMatcher::contains("+1"))
                        .description("Parses unified diff into per-file chunks and stats"),
                )
                .node_example(
                    NodeExample::new("render_markdown")
                        .input("diff_files", mock_diff_files_value())
                        .input("stats", Value::Str("+1 -0 across 1 files".into()))
                        .output("markdown", OutputMatcher::contains("# Branch Diff"))
                        .description("Renders markdown diff snapshot"),
                );
        }
        GistMode::Recent => {
            spec = spec
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
                        .output(
                            "base_ref",
                            OutputMatcher::exact(Value::Str("abc123def456".into())),
                        )
                        .description("Parses rev-list output into base_ref"),
                )
                .node_example(
                    NodeExample::new("diff/prepare_diff")
                        .input("repo_path", Value::Str(".".into()))
                        .input("base_ref", Value::Str("abc123def456".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git diff request for recent changes"),
                )
                .node_example(
                    NodeExample::new("diff/parse_diff")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok(mock_diff_response()).into()),
                        )
                        .output("diff_files", OutputMatcher::Any)
                        .output("stats", OutputMatcher::contains("+1"))
                        .description("Parses unified diff into per-file chunks and stats"),
                )
                .node_example(
                    NodeExample::new("render_markdown")
                        .input("diff_files", mock_diff_files_value())
                        .input("stats", Value::Str("+1 -0 across 1 files".into()))
                        .output("markdown", OutputMatcher::contains("# Branch Diff"))
                        .description("Renders markdown diff snapshot"),
                );
        }
    }

    spec
}

/// Mock spec for snapshot mode (default gist).
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gist-snapshot",
    builder = "crate::build_gist_graph(crate::GistMode::Snapshot, vec![], false).unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "gist-snapshot",
    output = "lib/tools/gist/src/generated_tests_snapshot.rs",
    module = "gist_snapshot_generated_tests",
    builder = "crate::build_gist_graph(crate::GistMode::Snapshot, vec![], false).unwrap()",
    signature = "crate::gist_signature(&crate::GistMode::Snapshot)",
    tool = "gist",
    window_max_nodes = 1
)]
pub fn gist_snapshot_mock_spec() -> MockSpec {
    gist_mock_spec(&GistMode::Snapshot)
}

/// Mock spec for diff mode (gist-diff).
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gist-diff",
    builder = r#"crate::build_gist_graph(crate::GistMode::Diff { base_ref: "main".to_string() }, vec![], false).unwrap()"#
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "gist-diff",
    output = "lib/tools/gist/src/generated_tests_diff.rs",
    module = "gist_diff_generated_tests",
    builder = r#"crate::build_gist_graph(crate::GistMode::Diff { base_ref: "main".to_string() }, vec![], false).unwrap()"#,
    signature = r#"crate::gist_signature(&crate::GistMode::Diff { base_ref: "main".to_string() })"#,
    tool = "gist-diff",
    window_max_nodes = 1
)]
pub fn gist_diff_mock_spec() -> MockSpec {
    gist_mock_spec(&GistMode::Diff {
        base_ref: "main".to_string(),
    })
}

/// Mock spec for recent mode (gist-recent).
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "gist-recent",
    builder = "crate::build_gist_graph(crate::GistMode::Recent, vec![], false).unwrap()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "gist-recent",
    output = "lib/tools/gist/src/generated_tests_recent.rs",
    module = "gist_recent_generated_tests",
    builder = "crate::build_gist_graph(crate::GistMode::Recent, vec![], false).unwrap()",
    signature = "crate::gist_signature(&crate::GistMode::Recent)",
    tool = "gist-recent",
    window_max_nodes = 1
)]
pub fn gist_recent_mock_spec() -> MockSpec {
    gist_mock_spec(&GistMode::Recent)
}

/// Mock spec for testing gist with file system lock simulation.
///
/// Use this when testing tools that acquire file locks before reading.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn gist_mock_spec_with_fs_lock() -> MockSpec {
    gist_mock_spec(&GistMode::Snapshot).resource_lock("file:read")
}

/// Mock spec for testing lease expiration scenarios.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn gist_mock_spec_lease_expires() -> MockSpec {
    gist_mock_spec(&GistMode::Snapshot).resource_lease_expires("github:api_token", 5000)
}
