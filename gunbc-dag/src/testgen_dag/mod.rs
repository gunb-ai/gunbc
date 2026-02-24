//! gunbc-dag Testgen DAG module.
//!
//! DAG-based test generation from MockSpecs.
//! Named `testgen_dag` to avoid collision with the `testgen` binary name.

pub mod dag_test_discovery;
pub mod graph;
pub mod mock_interpreter;
pub mod ops;

pub use dag_test_discovery::{
    build_mock_spec_from_test, build_testgen_target_def, compile_dag_for_test,
    dag_builder_call_for_module, discover_dag_tests, DagTestTarget,
};
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
    provides = "**/generated_tests*.rs",
    consumes = "target/codegen/.stamp",
    returns_result
)]
pub fn testgen_tool() {}
