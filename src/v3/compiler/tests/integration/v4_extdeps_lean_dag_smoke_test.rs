//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/languages/lean.dag` — B-2 / L-4
//! Lean LanguageModel must lower+infer with **zero** module diagnostics.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const LEAN_DAG: &str = include_str!("../../../../v4/extdeps/languages/lean.dag");
const LEAN_PATH: &str = "src/v4/extdeps/languages/lean.dag";

#[test]
fn v4_extdeps_lean_dag_compiles() {
    match compile_to_dag(LEAN_DAG, LEAN_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{LEAN_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{LEAN_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{LEAN_PATH}: {other:?}"),
    }
}
