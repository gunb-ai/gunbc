//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/languages/resolver.dag` —
//! the shared D2-resolver registry (D4). A zero-import leaf file: it must
//! lower+infer with **zero** module diagnostics (v2/v3 surface 0-diag gate).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const RESOLVER_DAG: &str = include_str!("../../../../v4/extdeps/languages/resolver.dag");
const RESOLVER_PATH: &str = "src/v4/extdeps/languages/resolver.dag";

#[test]
fn v4_extdeps_resolver_dag_compiles() {
    match compile_to_dag(RESOLVER_DAG, RESOLVER_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{RESOLVER_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{RESOLVER_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{RESOLVER_PATH}: {other:?}"),
    }
}
