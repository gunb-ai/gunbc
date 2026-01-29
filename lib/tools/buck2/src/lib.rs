//! gunbc-buck2: Buck2 file generation from Cargo.toml.
//!
//! This tool demonstrates gunbc's library-based architecture. It composes:
//! - Buck2-specific ops (parse, extract, generate)
//! - Library ops from gunbc-ops (file write, transport)
//!
//! Pipeline:
//! ```text
//! ParseCargoToml -> ExtractDeps -> GenerateBuckTargets -> PrepareFileWrite -> ExecuteTransport
//!    (buck2)        (buck2)           (buck2)               (fs)              (transport)
//! ```
//!
//! The last step (ExecuteTransport) is a boundary — it has no downstream edges,
//! so it's automatically identified as a world-write. In dry-run mode,
//! it gets intercepted and returns the generated content without writing.

pub mod graph;
pub mod ops;

pub use graph::{build_buck2_graph, Buck2GraphOp};
pub use ops::Buck2Op;
