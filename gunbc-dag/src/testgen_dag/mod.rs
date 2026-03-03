//! gunbc-dag Testgen DAG module.
//!
//! Graph builder, runtime ops, and DAG test discovery live here.
//! Mock interpretation and profile scanning are in
//! `gunbc_codegen::testgen_dag` (relocated in B5).

pub mod dag_test_discovery;
pub mod graph;
pub mod ops;

// Re-export relocated modules from core/codegen.
pub use gunbc_codegen::testgen_dag::mock_interpreter;
pub use gunbc_codegen::testgen_dag::profile_discovery;

pub use dag_test_discovery::{
    auto_testgen_for_module, build_mock_spec_from_test, build_testgen_target_def,
    compile_dag_for_test, dag_builder_call_for_module, discover_compilable_modules,
    discover_dag_tests, AutoTestgenResult, CompilableModule, DagTestTarget,
};
pub use graph::{
    build_testgen_graph, build_testgen_graph_auto, build_testgen_graph_for_test, TestgenGraphOp,
};
pub use ops::TestgenOp;
pub use gunbc_codegen::testgen_dag::{discover_profiles, profiles_for_module, DiscoveredProfile};
