//! gunbc-dag Testgen DAG module.
//!
//! DAG-based test generation from MockSpecs.
//! Named `testgen_dag` to avoid collision with the `testgen` binary name.

pub mod graph;
pub mod ops;

pub mod graph_mock;

pub use graph::{build_testgen_graph, build_testgen_graph_for_test, TestgenGraphOp};
pub use ops::TestgenOp;

#[cfg(test)]
#[allow(unused_variables)]
mod generated_tests {
    include!("generated_tests.rs");
}
