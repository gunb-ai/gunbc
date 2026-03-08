//! Testgen DAG module: discovery, mock interpretation, and auto-testgen pipeline.

pub mod dag_test_discovery;
pub mod mock_interpreter;

pub use dag_test_discovery::{
    auto_testgen_for_module, build_mock_spec_from_test, build_testgen_target_def,
    compile_dag_for_test, dag_builder_call_for_module, discover_compilable_modules,
    discover_dag_tests, find_compilable_module, output_path_for_module,
    render_auto_testgen_for_module, AutoTestgenResult, CompilableModule, DagTestTarget,
    RenderedTestgenModule,
};
