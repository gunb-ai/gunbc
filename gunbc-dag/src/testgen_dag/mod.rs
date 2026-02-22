//! gunbc-dag Testgen DAG module.
//!
//! DAG-based test generation from MockSpecs.
//! Named `testgen_dag` to avoid collision with the `testgen` binary name.

pub mod graph;
pub mod ops;

pub use graph::{build_testgen_graph, build_testgen_graph_for_test, TestgenGraphOp};
pub use ops::TestgenOp;

#[gunbc_testgen_registry_macros::resource_test_target(
    name = "testgen_dag",
    builder = "crate::testgen_dag::build_testgen_graph_for_test().unwrap()"
)]
pub fn testgen_dag_resource_target() {}
