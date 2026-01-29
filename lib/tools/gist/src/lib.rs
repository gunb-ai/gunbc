//! gunbc-gist: Gist generation tool built on gunbc.
//!
//! This tool demonstrates gunbc's library-based architecture. Instead of
//! defining its own ops, it composes ops from the lib crates:
//!
//! - `gunbc_lib_fs` for file operations (list, filter, read)
//! - `gunbc_lib_markdown` for markdown generation
//! - `gunbc_lib_gist_ops` for gist-specific operations
//!
//! The graph wires these library ops together into a pipeline:
//!
//! ```text
//! ListFiles -> FilterByExtension -> ReadFiles -> RenderCodeSnapshot -> PrepareRequest -> ExecuteTransport
//!    (fs)           (fs)              (fs)          (markdown)           (gist)           (transport)
//! ```
//!
//! The last step (ExecuteTransport) is a boundary — it has no downstream edges,
//! so it's automatically identified as a world-write. In dry-run mode,
//! it gets intercepted and returns a mock URL.

pub mod graph;

// Re-export for backwards compatibility
pub use graph::{build_gist_graph, GistGraphOp};

// Re-export the library ops for convenience
pub use gunbc_lib_fs::FsOp;
pub use gunbc_lib_gist_ops::GistOps;
pub use gunbc_lib_markdown::MarkdownOp;

// Legacy alias for backwards compatibility
pub type GistOp = GistGraphOp;
