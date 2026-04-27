//! **Layer:** integration
//!
//! R1C-D — runner-side receipt that the six PB census `.dag` `TestClaim`
//! fixtures in `tests/fixtures/r1_pb_census_gates.dag` lower cleanly and
//! evaluate through `TestRunner` against the live SG-0 census authority.
//!
//! The acceptance shape (per `docs/briefs/r1c-d-t-pb-census-as-dag-worker.md`
//! §Acceptance) is **not** "every claim Pass" — three of the six gates
//! (`pb_hand_rust_at_shim_floor`, `pb_test_file_generated_from_dag`,
//! `pb_rust_tests_outside_residual_zero`) are RED today and stay RED until
//! cascade-promotion 0-floor work in the Pure Bootstrap to Zero program
//! retires the residual hand-Rust census lists. The receipt this test
//! carries is:
//!
//!   1. The fixture compiles cleanly through `compile_to_dag`.
//!   2. `TestRunner::run_suite` returns six results in declared order.
//!   3. **No** result is `ClaimResult::NotYetImplemented(_)` — proving
//!      every PB census predicate's `eval_*_shape` slice is wired in
//!      `test_runner.rs` (R1C-D Acceptance line: "Runner dispatches each
//!      fixture cleanly").
//!   4. Each result is `Pass` or `Fail` against current census state, never
//!      an evaluation error from a missing dispatch arm.
//!
//! The Rust ratchets in `tests/integration/sg0_census_test.rs` remain the
//! drift-detection authority on the underlying lists. RED `.dag` gates are
//! surfaced to the R1 Closure Manager inbox as dissolution-work-pending
//! cross-manager queue entries; pacing is the Pure Bootstrap to Zero
//! program's, not this test's.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r1_pb_census_gates.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r1_pb_census_gates.dag";
const SUITE_NAME: &str = "r1_pb_census_gates_suite";

const EXPECTED_CLAIM_NAMES: &[&str] = &[
    "pb_hand_rust_at_shim_floor",
    "lens_producer_files_remaining",
    "pb_self_compile_fixed_point",
    "pb_compiler_std_ratchet_zero",
    "pb_test_file_generated_from_dag",
    "pb_rust_tests_outside_residual_zero",
];

#[test]
fn r1c_d_pb_census_gates_suite_evaluates_through_runner() {
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
    let actual_names: Vec<&str> = results.iter().map(|r| r.claim_name.as_str()).collect();
    assert_eq!(
        actual_names, EXPECTED_CLAIM_NAMES,
        "suite `{SUITE_NAME}`: claim order must match the declared deliverable list"
    );

    // Brief acceptance line: "Runner dispatches each fixture cleanly (no
    // `NotYetImplemented` returns)." A `NotYetImplemented` here would mean
    // a PB census `eval_*_shape` slice regressed to a stub — that is the
    // exact regression this test is designed to catch.
    let unimplemented: Vec<_> = results
        .iter()
        .filter(|r| matches!(r.result, ClaimResult::NotYetImplemented(_)))
        .collect();
    assert!(
        unimplemented.is_empty(),
        "PB census predicates must dispatch to wired evaluators, not `NotYetImplemented`. Offenders:\n{unimplemented:#?}"
    );

    // Everything else should be a structural Pass/Fail outcome — never an
    // evaluation error from a missing dispatch arm or malformed payload.
    for result in &results {
        match &result.result {
            ClaimResult::Pass | ClaimResult::Fail(_) => {}
            other => panic!(
                "PB census claim `{}` must evaluate to Pass or Fail, got {:?}",
                result.claim_name, other
            ),
        }
    }
}
