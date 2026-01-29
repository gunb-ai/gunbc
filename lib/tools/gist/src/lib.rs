//! gunbc-gist: Gist generation tool built on gunbc.
//!
//! This tool demonstrates gunbc's primitive-based architecture.
//! It composes primitives and library ops:
//!
//! - `gunbc_primitives` for file operations (ListFiles, ReadFiles)
//! - `gunbc_lib_markdown` for markdown generation
//! - `gunbc_lib_gist_ops` for gist-specific operations
//!
//! The graph wires these ops together into a pipeline:
//!
//! ```text
//! ListFiles -> FilterByExtension -> ReadFiles -> RenderCodeSnapshot -> PrepareRequest -> ExecuteTransport
//! (primitive)    (local)         (primitive)      (markdown)           (gist)           (transport)
//! ```
//!
//! The last step (ExecuteTransport) is a boundary — it has no downstream edges,
//! so it's automatically identified as a world-write. In dry-run mode,
//! it gets intercepted and returns a mock URL.
//!
//! # Mock Specifications
//!
//! Mock specs are in `graph_mock.rs` for test generation.

pub mod graph;

#[cfg(test)]
pub mod graph_mock;

// Re-export for backwards compatibility
pub use graph::{build_gist_graph, GistGraphOp};

// Re-export the library ops for convenience
pub use gunbc_lib_gist_ops::GistOps;
pub use gunbc_lib_markdown::MarkdownOp;

// Legacy alias for backwards compatibility
pub type GistOp = GistGraphOp;
