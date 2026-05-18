//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/compiler/01_tokenize.dag` — keeps the CP-1
//! tokenize stage (`LexRules` / `Outcome<TokenStream>`) on the v2 `compile` graph.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const TOKENIZE_DAG: &str = include_str!("../../../../v4/compiler/01_tokenize.dag");
const TOKENIZE_PATH: &str = "src/v4/compiler/01_tokenize.dag";

#[test]
fn v4_compiler_tokenize_dag_compiles() {
    match compile_to_dag(TOKENIZE_DAG, TOKENIZE_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{TOKENIZE_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{TOKENIZE_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{TOKENIZE_PATH}: {other:?}"),
    }
}
