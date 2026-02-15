//! Mock specification for the dag-viz tool.
//!
//! Uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Boundary Mocks
//!
//! - `execute_current_branch`: Transport node for `git rev-parse --abbrev-ref HEAD`
//! - `execute_gist`: Transport node for `gh gist create`
//! - `execute_git_show_base`: Transport node for `git show <ref>:.dag-snapshots/...`
//! - `execute_rev_list`: Transport node for `git rev-list --before=...`
//! - `execute_write_snapshot`: Transport node for file write
//!
//! # Input Expectations
//!
//! - `format`: String (snapshot/diff/recent modes)
//! - `base_ref`: String (diff mode only)

use crate::dag_viz::graph::{build_dag_viz_graph, DagVizMode};
use gunbc_ir::transport::{
    FileOp, FileResponse, ShellResponse, TransportResponse,
};
use gunbc_ir::Value;
use gunbc_primitives::filename;
use gunbc_test::{extract_mock_requirements, InputConstraint, MockSpec, NodeExample, OutputMatcher};

fn mock_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

fn mock_shell_ok(stdout: &str) -> TransportResponse {
    TransportResponse::Shell(ShellResponse::ok(stdout))
}

fn mock_empty_topology_json() -> &'static str {
    r#"{"nodes":[],"edges":[]}"#
}

/// Build a mock specification for the dag-viz graph.
fn dag_viz_mock_spec(mode: &DagVizMode) -> MockSpec {
    let dag = build_dag_viz_graph(mode.clone()).expect("dag-viz graph should build");

    let mut reqs = extract_mock_requirements(&dag, "dag-viz")
        .boundary("fs_env", "file:write", mock_fs_handle())
        .expect("fs_env should match type");

    match mode {
        DagVizMode::Snapshot => {
            reqs = reqs
                .transport_response(
                    "execute_current_branch",
                    "response",
                    mock_shell_ok("main\n"),
                )
                .expect("current_branch response should match")
                .transport_response(
                    "execute_gist",
                    "response",
                    mock_shell_ok("https://gist.github.com/mock/abc123\n"),
                )
                .expect("gist response should match")
                .transport_response(
                    "execute_local_save",
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
                    "execute_browser_open",
                    "response",
                    mock_shell_ok(""),
                )
                .expect("browser_open response should match");
        }
        DagVizMode::Diff { .. } => {
            reqs = reqs
                .transport_response(
                    "execute_current_branch",
                    "response",
                    mock_shell_ok("main\n"),
                )
                .expect("current_branch response should match")
                .transport_response(
                    "execute_git_show_base",
                    "response",
                    mock_shell_ok(mock_empty_topology_json()),
                )
                .expect("git_show_base response should match")
                .transport_response(
                    "execute_gist",
                    "response",
                    mock_shell_ok("https://gist.github.com/mock/abc123\n"),
                )
                .expect("gist response should match");
        }
        DagVizMode::Recent => {
            reqs = reqs
                .transport_response(
                    "execute_current_branch",
                    "response",
                    mock_shell_ok("main\n"),
                )
                .expect("current_branch response should match")
                .transport_response(
                    "execute_rev_list",
                    "response",
                    mock_shell_ok("abc123def456\n"),
                )
                .expect("rev_list response should match")
                .transport_response(
                    "execute_git_show_base",
                    "response",
                    mock_shell_ok(mock_empty_topology_json()),
                )
                .expect("git_show_base response should match")
                .transport_response(
                    "execute_gist",
                    "response",
                    mock_shell_ok("https://gist.github.com/mock/abc123\n"),
                )
                .expect("gist response should match");
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
                    "parse_gist",
                    "url",
                    "https://gist.github.com/mock/abc123",
                )
                .expect("parse_gist.url mock should match type")
                .boundary(
                    "parse_browser_open",
                    "opened",
                    Value::Bool(true),
                )
                .expect("parse_browser_open.opened mock should match type");
        }
        DagVizMode::Diff { .. } | DagVizMode::Recent => {
            reqs = reqs
                .boundary_str(
                    "parse_gist",
                    "url",
                    "https://gist.github.com/mock/abc123",
                )
                .expect("parse_gist.url mock should match type");
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

    // Input mocks for entry points
    match mode {
        DagVizMode::Snapshot => {
            spec = spec
                .input_mock(
                    "prepare_current_branch",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock(
                    "render_snapshot",
                    "format",
                    Value::Str("html".into()),
                )
                .expects_input("repo_path", InputConstraint::Any)
                .expects_input("format", InputConstraint::Any);
        }
        DagVizMode::Diff { .. } => {
            spec = spec
                .input_mock(
                    "prepare_current_branch",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock(
                    "diff_and_render",
                    "base_ref",
                    Value::Str("main".into()),
                )
                .expects_input("repo_path", InputConstraint::Any)
                .expects_input("format", InputConstraint::Any)
                .expects_input("base_ref", InputConstraint::Any);
        }
        DagVizMode::Recent => {
            spec = spec
                .input_mock(
                    "prepare_current_branch",
                    "repo_path",
                    Value::Str(".".into()),
                )
                .input_mock(
                    "prepare_rev_list",
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
                // Git ops — tested in gunbc-lib-git-ops crate
                .node_example(
                    NodeExample::new("prepare_current_branch")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-parse request for current branch"),
                )
                .node_example(
                    NodeExample::new("parse_current_branch")
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
                        .input("topology_json", Value::Str(mock_empty_topology_json().into()))
                        .input("branch", Value::Str("main".into()))
                        .input("format", Value::Str("html".into()))
                        .output("content", OutputMatcher::non_empty())
                        .output("ext", OutputMatcher::non_empty())
                        .description("Renders topology as HTML or markdown"),
                )
                .node_example(
                    NodeExample::new("prepare_gist")
                        .input("content", Value::Str("<html>mock</html>".into()))
                        .input("branch", Value::Str("main".into()))
                        .input("ext", Value::Str("html".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares gist upload shell request"),
                )
                .node_example(
                    NodeExample::new("parse_gist")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("https://gist.github.com/mock/abc123\n").into()),
                        )
                        .output("url", OutputMatcher::contains("gist.github.com"))
                        .description("Extracts gist URL from response"),
                )
                // Local save + browser open
                .node_example(
                    NodeExample::new("prepare_local_save")
                        .input("content", Value::Str("<html>mock</html>".into()))
                        .input("ext", Value::Str("html".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares file write request for local HTML"),
                )
                .node_example(
                    NodeExample::new("parse_local_save")
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
                    NodeExample::new("prepare_browser_open")
                        .input("file_path", Value::Str("target/dag-viz/dag-visualization.html".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares xdg-open/open command for browser"),
                )
                .node_example(
                    NodeExample::new("parse_browser_open")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("").into()),
                        )
                        .output("opened", OutputMatcher::IsBool)
                        .description("Confirms browser open succeeded"),
                )
                .live_expected_output("parse_gist", "url", OutputMatcher::NonEmpty)
                .live_expected_output("parse_browser_open", "opened", OutputMatcher::IsBool);
        }
        DagVizMode::Diff { .. } => {
            spec = spec
                // Git ops
                .node_example(
                    NodeExample::new("prepare_current_branch")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-parse request for current branch"),
                )
                .node_example(
                    NodeExample::new("parse_current_branch")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("main\n").into()),
                        )
                        .output("branch", OutputMatcher::non_empty())
                        .description("Parses branch name from git rev-parse response"),
                )
                // Tool-specific pure nodes
                .node_example(
                    NodeExample::new("prepare_git_show_base")
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git show request for base topology"),
                )
                .node_example(
                    NodeExample::new("parse_git_show_base")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok(mock_empty_topology_json()).into()),
                        )
                        .output("topology_json", OutputMatcher::non_empty())
                        .description("Parses git show response into topology JSON"),
                )
                .node_example(
                    NodeExample::new("diff_and_render")
                        .input("current_json", Value::Str(mock_empty_topology_json().into()))
                        .input("base_json", Value::Str(mock_empty_topology_json().into()))
                        .input("branch", Value::Str("main".into()))
                        .input("base_ref", Value::Str("main".into()))
                        .output("content", OutputMatcher::non_empty())
                        .description("Diffs topologies and renders as markdown"),
                )
                .node_example(
                    NodeExample::new("prepare_gist")
                        .input("content", Value::Str("# Mock diff".into()))
                        .input("branch", Value::Str("main".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares gist upload shell request"),
                )
                .node_example(
                    NodeExample::new("parse_gist")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("https://gist.github.com/mock/abc123\n").into()),
                        )
                        .output("url", OutputMatcher::contains("gist.github.com"))
                        .description("Extracts gist URL from response"),
                )
                .live_expected_output("parse_gist", "url", OutputMatcher::NonEmpty);
        }
        DagVizMode::Recent => {
            spec = spec
                // Git ops
                .node_example(
                    NodeExample::new("prepare_current_branch")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-parse request for current branch"),
                )
                .node_example(
                    NodeExample::new("parse_current_branch")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("main\n").into()),
                        )
                        .output("branch", OutputMatcher::non_empty())
                        .description("Parses branch name from git rev-parse response"),
                )
                .node_example(
                    NodeExample::new("prepare_rev_list")
                        .input("repo_path", Value::Str(".".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git rev-list request for recent commit"),
                )
                .node_example(
                    NodeExample::new("parse_rev_list")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("abc123def456\n").into()),
                        )
                        .output("base_ref", OutputMatcher::non_empty())
                        .description("Parses commit hash from rev-list response"),
                )
                // Tool-specific pure nodes
                .node_example(
                    NodeExample::new("prepare_git_show_base")
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares git show request for base topology"),
                )
                .node_example(
                    NodeExample::new("parse_git_show_base")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok(mock_empty_topology_json()).into()),
                        )
                        .output("topology_json", OutputMatcher::non_empty())
                        .description("Parses git show response into topology JSON"),
                )
                .node_example(
                    NodeExample::new("diff_and_render")
                        .input("current_json", Value::Str(mock_empty_topology_json().into()))
                        .input("base_json", Value::Str(mock_empty_topology_json().into()))
                        .input("branch", Value::Str("main".into()))
                        .input("base_ref", Value::Str("abc123def456".into()))
                        .output("content", OutputMatcher::non_empty())
                        .description("Diffs topologies and renders as markdown"),
                )
                .node_example(
                    NodeExample::new("prepare_gist")
                        .input("content", Value::Str("# Mock diff".into()))
                        .input("branch", Value::Str("main".into()))
                        .output("request", OutputMatcher::IsRequest)
                        .description("Prepares gist upload shell request"),
                )
                .node_example(
                    NodeExample::new("parse_gist")
                        .input(
                            "response",
                            Value::Response(ShellResponse::ok("https://gist.github.com/mock/abc123\n").into()),
                        )
                        .output("url", OutputMatcher::contains("gist.github.com"))
                        .description("Extracts gist URL from response"),
                )
                .live_expected_output("parse_gist", "url", OutputMatcher::NonEmpty);
        }
        DagVizMode::SaveSnapshot => {
            spec = spec
                .node_example(
                    NodeExample::new("prepare_write_snapshot")
                        .input("topology_json", Value::Str(mock_empty_topology_json().into()))
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
