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

#![forbid(dead_code)]
pub mod graph;

#[cfg(test)]
pub mod graph_mock;

// Re-export public API
pub use graph::{
    build_gist_graph, build_read_file_body_dag, gist_signature, GistGraphOp, GistMode,
};

// Re-export the library ops for convenience
pub use gunbc_lib_gist_ops::GistOps;
pub use gunbc_lib_git_ops::GitOps;
pub use gunbc_lib_markdown::MarkdownOp;

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
