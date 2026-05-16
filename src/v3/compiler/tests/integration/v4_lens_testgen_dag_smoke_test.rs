//! Smoke `compile_to_dag` on T-19 artifacts under `src/v4/` (IB-1 still-deer-194).
//! Self-contained modules only — `compile_to_dag` does not yet load peer
//! `import v4.std.*` files; see `src/v4/lens/testgen.dag` header.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const TESTGEN_PATH: &str = "src/v4/lens/testgen.dag";

const MANUAL_MANIFEST_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/t19_manual_anchor_manifest.dag");
const MANUAL_MANIFEST_PATH: &str = "src/v4/test/claim/manual/t19_manual_anchor_manifest.dag";

fn assert_compile_empty(source: &str, path: &'static str) {
    match compile_to_dag(source, path) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{path}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{path}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{path}: {other:?}"),
    }
}

#[test]
fn v4_lens_testgen_dag_compiles() {
    assert_compile_empty(TESTGEN_DAG, TESTGEN_PATH);
}

#[test]
fn v4_test_claim_manual_t19_manifest_dag_compiles() {
    assert_compile_empty(MANUAL_MANIFEST_DAG, MANUAL_MANIFEST_PATH);
}
