//! **Layer:** integration
//!
//! R3 gate #8 — `self_gen_non_test_zero` executable receipt.
//!
//! The gate is the conjunction of the Self-Generation-0 T-PB-A non-test Rust census and
//! hand-authored scaffold-fragment census reaching zero. This receipt keeps the
//! state-check load-bearing while residuals remain: it must execute through the
//! `.dag` `TestRunner` path and consume the live census instead of remaining a
//! paper-only row in `docs/r3-program-plan.md`.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

use crate::self_gen0_census_test::{
    expected_hand_authored_fragments_count, expected_hand_authored_non_test_count,
};

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_self_gen_non_test_zero.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_self_gen_non_test_zero.dag";
const SUITE_NAME: &str = "r3_self_gen_non_test_zero_suite";
const NON_TEST_CLAIM: &str = "self_gen_non_test_zero_rust";
const FRAGMENTS_CLAIM: &str = "self_gen_non_test_zero_fragments";

#[test]
fn r3_gate_8_self_gen_non_test_zero_claims_execute_against_live_census() {
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

    assert_live_census_result(
        &results,
        NON_TEST_CLAIM,
        "expected_hand_authored_non_test",
        expected_hand_authored_non_test_count(),
    );
    assert_live_census_result(
        &results,
        FRAGMENTS_CLAIM,
        "expected_hand_authored_fragments",
        expected_hand_authored_fragments_count(),
    );
}

fn assert_live_census_result(
    results: &[v3_compiler::test_runner::ClaimEvaluation],
    claim_name: &str,
    list_constant: &str,
    expected_count: usize,
) {
    let result = results
        .iter()
        .find(|result| result.claim_name == claim_name)
        .unwrap_or_else(|| panic!("missing `{claim_name}` in `{SUITE_NAME}` results: {results:?}"));

    match &result.result {
        ClaimResult::Pass => assert_eq!(
            expected_count, 0,
            "`{claim_name}` unexpectedly passed while `{list_constant}` still has \
             {expected_count} live Self-Generation-0 entries"
        ),
        ClaimResult::Fail(reason) => {
            let expected_reason =
                format!("CensusBoundCheck `{list_constant}` observed {expected_count}, bound 0");
            assert!(
                expected_count > 0,
                "`{claim_name}` failed even though `{list_constant}` is empty: {reason:?}"
            );
            assert_eq!(
                reason, &expected_reason,
                "`{claim_name}` should execute CensusBoundCheck against live `{list_constant}`"
            );
        }
        ClaimResult::NotYetImplemented(reason) => {
            panic!("`{claim_name}` must be executable for R3 gate #8, got NYI: {reason}")
        }
    }
}
