//! gunbc-dag doc generation module.
//!
//! Generates documentation with live code excerpts and test indices.

pub mod graph;
pub mod ops;

pub use graph::{build_docgen_graph, DocgenGraphOp};
pub use ops::DocgenOp;
