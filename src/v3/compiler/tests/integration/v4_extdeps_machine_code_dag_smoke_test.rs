//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/languages/machine_code.dag` —
//! T-4.13 machine-code model must lower+infer with **zero** module diagnostics.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const MACHINE_CODE_DAG: &str = include_str!("../../../../v4/extdeps/languages/machine_code.dag");
const MACHINE_CODE_PATH: &str = "src/v4/extdeps/languages/machine_code.dag";

#[test]
fn v4_extdeps_machine_code_dag_compiles() {
    match compile_to_dag(MACHINE_CODE_DAG, MACHINE_CODE_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{MACHINE_CODE_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{MACHINE_CODE_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{MACHINE_CODE_PATH}: {other:?}"),
    }
}
