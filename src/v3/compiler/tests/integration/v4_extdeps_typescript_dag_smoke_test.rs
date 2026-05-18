//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/languages/typescript.dag` —
//! T-4 TypeScript primitive scaffold must lower+infer with **zero** module
//! diagnostics while the fact-bundle Phase-3 rework is gated.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const TYPESCRIPT_DAG: &str = include_str!("../../../../v4/extdeps/languages/typescript.dag");
const TYPESCRIPT_PATH: &str = "src/v4/extdeps/languages/typescript.dag";

#[test]
fn v4_extdeps_typescript_dag_compiles() {
    match compile_to_dag(TYPESCRIPT_DAG, TYPESCRIPT_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{TYPESCRIPT_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{TYPESCRIPT_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{TYPESCRIPT_PATH}: {other:?}"),
    }
}
