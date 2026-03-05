//! gunbc-dag testgen support.
//!
//! The `.dag` entrypoint for `tools/testgen.dag` owns orchestration.
//! This module keeps discovery/test helpers plus the narrow Rust bridges
//! that the DSL still calls through.

pub mod dag_test_discovery;

// Re-export relocated modules from core/codegen.
pub use gunbc_codegen::testgen_dag::mock_interpreter;
pub use gunbc_codegen::testgen_dag::profile_discovery;

pub use dag_test_discovery::{
    auto_testgen_for_module, build_mock_spec_from_test, build_testgen_target_def,
    compile_dag_for_test, dag_builder_call_for_module, discover_compilable_modules,
    discover_dag_tests, find_compilable_module, output_path_for_module,
    render_auto_testgen_for_module, AutoTestgenResult, CompilableModule, DagTestTarget,
    RenderedTestgenModule,
};
pub use gunbc_codegen::testgen_dag::{discover_profiles, profiles_for_module, DiscoveredProfile};
