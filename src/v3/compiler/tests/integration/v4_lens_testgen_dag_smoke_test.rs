//! Smoke `compile_to_dag` on `src/v4/lens/testgen.dag` — T-19 carrier + rule
//! surface must parse with zero module diagnostics (IB-1 still-deer-194).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const TESTGEN_PATH: &str = "src/v4/lens/testgen.dag";

#[test]
fn v4_lens_testgen_dag_compiles() {
    match compile_to_dag(TESTGEN_DAG, TESTGEN_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{TESTGEN_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{TESTGEN_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{TESTGEN_PATH}: {other:?}"),
    }
}
