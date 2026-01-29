//! gunbc-buck2: Buck2 file generation from Cargo.toml.
//!
//! This tool demonstrates gunbc's capabilities by implementing a
//! build file generation workflow:
//!
//! 1. Parse Cargo.toml
//! 2. Extract dependencies
//! 3. Generate Buck2 targets
//! 4. Write BUCK file (boundary)
//!
//! The last step (WriteBuckFile) is a boundary — it has no downstream edges,
//! so it's automatically identified as a world-write. In dry-run mode,
//! it gets intercepted and returns the generated content without writing.

pub mod graph;
pub mod ops;

pub use graph::build_buck2_graph;
pub use ops::Buck2Op;
