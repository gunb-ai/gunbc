//! gunbc-dag Build module.
//!
//! Local development build pipeline with DAG progress visualization.
//! Wraps cargo build, test, and clippy in a progress-tracked DAG.

pub mod graph;
pub mod ops;

pub use crate::dsl_builder::build_build_graph_dsl;
pub use graph::{build_build_graph, build_signature, BuildGraphOp};
pub use ops::BuildOp;
