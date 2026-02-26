//! gunbc-dag Testgen DAG module.
//!
//! DAG-based test generation from MockSpecs.
//! Named `testgen_dag` to avoid collision with the `testgen` binary name.

pub mod dag_test_discovery;
pub mod graph;
pub mod mock_interpreter;
pub mod ops;
pub mod profile_discovery;

pub use dag_test_discovery::{
    auto_testgen_for_module, build_mock_spec_from_test, build_testgen_target_def,
    compile_dag_for_test, dag_builder_call_for_module, discover_compilable_modules,
    discover_dag_tests, AutoTestgenResult, CompilableModule, DagTestTarget,
};
pub use graph::{
    build_testgen_graph, build_testgen_graph_auto, build_testgen_graph_for_test, TestgenGraphOp,
};
pub use ops::TestgenOp;
pub use profile_discovery::{discover_profiles, profiles_for_module, DiscoveredProfile};
