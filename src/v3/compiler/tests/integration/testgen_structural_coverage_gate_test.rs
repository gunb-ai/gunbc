//! **Layer:** integration
//!
//! R1 T-TestGen gate `testgen_structural_coverage` (ROADMAP.md:51, `[ext]`):
//! hand-authored `TestClaim` data in `tests/fixtures/r1_gates.dag` runs through
//! `TestRunner::run_suite` and evaluates a structural predicate (`PortHasState`)
//! against the compiled fixture DAG.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const R1_GATES_SOURCE: &str = include_str!("../fixtures/r1_gates.dag");

fn compile_clean(source: &str, file: &str) -> v3_compiler::dag::Dag {
    match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{file} should compile cleanly, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(err) => panic!("{file} should compile cleanly, got {err:?}"),
    }
}

#[test]
fn testgen_structural_coverage_gate_passes() {
    let dag = compile_clean(
        R1_GATES_SOURCE,
        "src/v3/compiler/tests/fixtures/r1_gates.dag",
    );
    let results = TestRunner::new(&dag).run_suite("testgen_structural_coverage_suite");

    assert_eq!(
        results.len(),
        1,
        "expected one claim in testgen_structural_coverage_suite"
    );
    assert_eq!(results[0].claim_name, "testgen_structural_coverage");
    assert_eq!(results[0].result, ClaimResult::Pass);
}
