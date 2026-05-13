//! **Layer:** integration
//!
//! R3 gate #64 — reflection-closure substrate-plumbing receipt.
//!
//! This is not the canonical `substrate_gap_reflection_closure_closed` consumer.
//! It proves the live T-LensProducer-Retirement residual census is executable
//! as plumbing for the bridge-count half of the class-level closure condition.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_substrate_gap_reflection_closure.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_substrate_gap_reflection_closure.dag";
const SUITE_NAME: &str = "r3_substrate_gap_reflection_closure_suite";
const CLAIM_NAME: &str = "substrate_gap_reflection_residual_census_receipt";
const CURRENT_REFLECTION_RESIDUAL_COUNT: i64 = 2;

#[test]
fn r3_gate_64_reflection_residual_census_receipt_executes() {
    let dag = match compile_to_dag(FIXTURE_SOURCE, FIXTURE_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{FIXTURE_PATH} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {FIXTURE_PATH}: {other:?}"),
    };

    let results = TestRunner::new(&dag).run_suite(SUITE_NAME);
    let result = results
        .iter()
        .find(|result| result.claim_name == CLAIM_NAME)
        .unwrap_or_else(|| panic!("missing `{CLAIM_NAME}` in `{SUITE_NAME}` results: {results:?}"));

    match &result.result {
        ClaimResult::Pass => assert_eq!(
            CURRENT_REFLECTION_RESIDUAL_COUNT, 0,
            "`{CLAIM_NAME}` passed before the frozen live reflection residual count reached zero"
        ),
        ClaimResult::Fail(reason) => {
            assert!(
                reason.contains(&format!(
                    "lens-producer subset observed {CURRENT_REFLECTION_RESIDUAL_COUNT}"
                )),
                "`{CLAIM_NAME}` should report the live reflection residual count; got {reason:?}"
            );
        }
        ClaimResult::NotYetImplemented(reason) => {
            panic!("`{CLAIM_NAME}` receipt must be executable for R3 gate #64 plumbing, got NYI: {reason}")
        }
    }
}
