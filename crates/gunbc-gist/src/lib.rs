//! gunbc-gist: Gist generation tool built on gunbc.
//!
//! This tool demonstrates gunbc's capabilities by implementing a
//! code context sharing workflow:
//!
//! 1. List files in a directory
//! 2. Filter files by pattern
//! 3. Read file contents
//! 4. Render as markdown
//! 5. Upload to GitHub Gist (boundary)
//!
//! The last step (CreateGist) is a boundary — it has no downstream edges,
//! so it's automatically identified as a world-write. In dry-run mode,
//! it gets intercepted and returns a mock URL.

pub mod graph;
pub mod ops;

pub use graph::build_gist_graph;
pub use ops::GistOp;
