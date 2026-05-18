//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/languages/python.dag` —
//! T-4 Python primitive scaffold must lower+infer with **zero** module
//! diagnostics after the D2-REV scalar-tower fact-bundle Phase-3 rework
//! (flat `PythonScalar` coproduct; per-primitive grounding rows gated).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const PYTHON_DAG: &str = include_str!("../../../../v4/extdeps/languages/python.dag");
const PYTHON_PATH: &str = "src/v4/extdeps/languages/python.dag";

#[test]
fn v4_extdeps_python_dag_compiles() {
    match compile_to_dag(PYTHON_DAG, PYTHON_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{PYTHON_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{PYTHON_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{PYTHON_PATH}: {other:?}"),
    }
}
