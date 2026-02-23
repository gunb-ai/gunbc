#![recursion_limit = "1024"]
//! gunbc-gist: Gist generation tool built on gunbc.
//!
//! This tool composes library ops into a single parameterized graph:
//!
//! - `gunbc_lib_git_ops` for git operations (ls-files, diff)
//! - `gunbc_lib_markdown` for markdown rendering
//! - `gunbc_lib_gist_ops` for gist creation
//!
//! The graph mode is selected at build time via [`GistMode`]:
//!
//! **Snapshot** (`make gist`):
//! ```text
//! ls-files → Execute → parse → read-files → Execute → parse → render → gist
//! ```
//!
//! **Diff** (`make gist-diff`):
//! ```text
//! git-diff → Execute → parse-diff → render-diff → gist
//! ```
//!
//! Extension filtering is pushed into git via pathspecs, not separate filter nodes.
//! All I/O happens through `TransportOps::Execute` boundary nodes.
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.

#![deny(dead_code)]
pub mod graph;

pub mod graph_mock;

// Re-export public API
pub use graph::{
    build_gist_graph, build_gist_graph_with_config, build_read_file_body_dag, gist_signature,
    GistGraphOp, GistMode,
};

// Re-export the library ops for convenience
pub use gunbc_lib_gist_ops::GistOps;
pub use gunbc_lib_git_ops::GitOps;
pub use gunbc_lib_markdown::MarkdownOp;

// ============================================================================
// Tool Target Registrations
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "gist",
    crate_name = "gunbc-gist",
    description = "Create a GitHub gist from code files",
    builder = "build_gist_graph",
    args = "GistMode::Snapshot, extensions.clone(), public",
    import = "use gunbc_gist::{build_gist_graph, GistMode};",
    mock_spec = "gunbc_gist::graph_mock::gist_snapshot_mock_spec()",
    package = "gist",
    binary = "gist",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#,
    dsl_module = "gist",
    has_invocation,
    returns_result
)]
pub fn gist_snapshot_tool() {}

#[gunbc_tool_registry_macros::tool_target(
    name = "gist-diff",
    crate_name = "gunbc-gist",
    description = "Create a GitHub gist from branch diff",
    builder = "build_gist_graph",
    args = "GistMode::Diff { base_ref: base_ref.clone() }, extensions.clone(), public",
    import = "use gunbc_gist::{build_gist_graph, GistMode};",
    mock_spec = "gunbc_gist::graph_mock::gist_diff_mock_spec()",
    package = "gist",
    binary = "gist-diff",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"base_ref","type_id":"String","short":"b","default":"main","help":"Base branch for diff","make_var":"BASE"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#,
    dsl_module = "gist",
    has_invocation,
    returns_result
)]
pub fn gist_diff_tool() {}

#[gunbc_tool_registry_macros::tool_target(
    name = "gist-recent",
    crate_name = "gunbc-gist",
    description = "Create a GitHub gist from recent changes (last 7 days)",
    builder = "build_gist_graph",
    args = "GistMode::Recent, extensions.clone(), public",
    import = "use gunbc_gist::{build_gist_graph, GistMode};",
    mock_spec = "gunbc_gist::graph_mock::gist_recent_mock_spec()",
    package = "gist",
    binary = "gist-recent",
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#,
    dsl_module = "gist",
    has_invocation,
    returns_result
)]
pub fn gist_recent_tool() {}

// ============================================================================
// DagSpec Registry Helpers
// ============================================================================

/// Return DagSpec registrations originating from this crate.
pub fn dag_specs() -> Vec<&'static gunbc_testgen_registry::DagSpecDef> {
    gunbc_testgen_registry::iter_dag_specs()
        .filter(|spec| spec.origin_crate == env!("CARGO_CRATE_NAME"))
        .collect()
}
