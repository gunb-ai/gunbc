//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/languages/ptx.dag` —
//! T-4.14 PTX model must lower+infer with **zero** module diagnostics.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const PTX_DAG: &str = include_str!("../../../../v4/extdeps/languages/ptx.dag");
const PTX_PATH: &str = "src/v4/extdeps/languages/ptx.dag";

#[test]
fn v4_extdeps_ptx_dag_compiles() {
    match compile_to_dag(PTX_DAG, PTX_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{PTX_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{PTX_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{PTX_PATH}: {other:?}"),
    }
}
