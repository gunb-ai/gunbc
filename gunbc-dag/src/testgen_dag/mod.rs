//! gunbc-dag Testgen DAG module.
//!
//! DAG-based test generation from MockSpecs.
//! Named `testgen_dag` to avoid collision with the `testgen` binary name.

pub mod graph;
pub mod ops;

pub use graph::{build_testgen_graph, build_testgen_graph_for_test, TestgenGraphOp};
pub use ops::TestgenOp;

// ============================================================================
// Tool Target Registration
// ============================================================================

#[gunbc_tool_registry_macros::tool_target(
    name = "testgen",
    crate_name = "gunbc-testgen",
    description = "Generate tests from DAG mock specifications",
    builder = "build_testgen_graph_for_test",
    outputs = "**/generated_tests*.rs",
    returns_result
)]
pub fn testgen_tool() {}
