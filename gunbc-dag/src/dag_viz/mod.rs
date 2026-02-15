//! gunbc-dag DAG visualization module.
//!
//! Visualizes DAG topology as interactive HTML or static Mermaid markdown.
//!
//! The tool composes workspace introspection, git operations, and rendering
//! into a DAG graph. The mode is selected at build time via [`DagVizMode`]:
//!
//! **Snapshot** (`make dag-viz`):
//! ```text
//! BuildTopology → RenderSnapshot → PrepareGist → Execute → ParseGist
//! ```
//!
//! **Diff** (`make dag-viz-diff`):
//! ```text
//! BuildTopology + GitShow(base) → DiffTopologies → RenderDiff → Gist
//! ```
//!
//! **Recent** (`make dag-viz-recent`):
//! ```text
//! BuildTopology + RevList → GitShow → DiffTopologies → RenderDiff → Gist
//! ```
//!
//! **SaveSnapshot** (`make dag-snapshot`):
//! ```text
//! BuildTopology → PrepareWrite → Execute → ParseWrite
//! ```
//!
//! All I/O happens through `TransportOps::Execute` boundary nodes.
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.

pub mod graph;
pub mod graph_mock;

// Re-export public API
pub use graph::{build_dag_viz_graph, dag_viz_signature, DagVizGraphOp, DagVizMode};

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "dag-viz",
    crate_name = "gunbc-dag",
    description = "Visualize DAG topology as interactive HTML",
    builder = "build_dag_viz_graph",
    args = "DagVizMode::Snapshot",
    import = "use gunbc_dag::dag_viz::{build_dag_viz_graph, DagVizMode};",
    mock_spec = "gunbc_dag::dag_viz::graph_mock::dag_viz_snapshot_mock_spec()",
    package = "dag",
    binary = "dag-viz",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path","make_var":"REPO"},{"port_name":"format","type_id":"String","short":"f","default":"html","help":"Output format: html (default) or md","make_var":"FMT"}]"#,
    has_invocation,
    returns_result
)]
pub fn dag_viz_snapshot_tool() {}

#[gunbc_tool_registry_macros::tool_target(
    name = "dag-viz-diff",
    crate_name = "gunbc-dag",
    description = "Visualize DAG topology diff vs base branch",
    builder = "build_dag_viz_graph",
    args = "DagVizMode::Diff { base_ref: base_ref.clone() }",
    import = "use gunbc_dag::dag_viz::{build_dag_viz_graph, DagVizMode};",
    mock_spec = "gunbc_dag::dag_viz::graph_mock::dag_viz_diff_mock_spec()",
    package = "dag",
    binary = "dag-viz-diff",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path","make_var":"REPO"},{"port_name":"format","type_id":"String","short":"f","default":"html","help":"Output format: html (default) or md","make_var":"FMT"},{"port_name":"base_ref","type_id":"String","short":"b","default":"main","help":"Base branch for diff","make_var":"BASE"}]"#,
    has_invocation,
    returns_result
)]
pub fn dag_viz_diff_tool() {}

#[gunbc_tool_registry_macros::tool_target(
    name = "dag-viz-recent",
    crate_name = "gunbc-dag",
    description = "Visualize DAG topology changes from last 3 days",
    builder = "build_dag_viz_graph",
    args = "DagVizMode::Recent",
    import = "use gunbc_dag::dag_viz::{build_dag_viz_graph, DagVizMode};",
    mock_spec = "gunbc_dag::dag_viz::graph_mock::dag_viz_recent_mock_spec()",
    package = "dag",
    binary = "dag-viz-recent",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path","make_var":"REPO"},{"port_name":"format","type_id":"String","short":"f","default":"html","help":"Output format: html (default) or md","make_var":"FMT"}]"#,
    has_invocation,
    returns_result
)]
pub fn dag_viz_recent_tool() {}

#[gunbc_tool_registry_macros::tool_target(
    name = "dag-snapshot",
    crate_name = "gunbc-dag",
    description = "Save DAG topology snapshot to .dag-snapshots/workspace.json",
    builder = "build_dag_viz_graph",
    args = "DagVizMode::SaveSnapshot",
    import = "use gunbc_dag::dag_viz::{build_dag_viz_graph, DagVizMode};",
    mock_spec = "gunbc_dag::dag_viz::graph_mock::dag_viz_save_snapshot_mock_spec()",
    package = "dag",
    binary = "dag-snapshot",
    entrypoints = r#"[]"#,
    has_invocation,
    returns_result
)]
pub fn dag_snapshot_tool() {}

// ============================================================================
// Generated Tests (from `make testgen`)
// ============================================================================

#[cfg(test)]
mod generated_tests_snapshot {
    include!("generated_tests_snapshot.rs");
}

#[cfg(test)]
mod generated_tests_diff {
    include!("generated_tests_diff.rs");
}

#[cfg(test)]
mod generated_tests_recent {
    include!("generated_tests_recent.rs");
}

#[cfg(test)]
mod generated_tests_snapshot_save {
    include!("generated_tests_snapshot_save.rs");
}
