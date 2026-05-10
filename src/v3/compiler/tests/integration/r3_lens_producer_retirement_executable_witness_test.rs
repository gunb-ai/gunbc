//! **Layer:** integration
//!
//! R3 gate #66 — `lens_producer_retirement_executable_witness`.
//!
//! This is the executable receipt for the current T-LensProducer-Retirement
//! state-check: the `.dag` PB census claim is runnable through `TestRunner`, and
//! it observes the live lens-producer residual set instead of deferring to a
//! paper-only receipt. The gate turns green when the three named producer
//! surfaces retire and this claim returns `Pass`.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r1_pb_census_gates.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r1_pb_census_gates.dag";
const SUITE_NAME: &str = "r1_pb_census_gates_suite";
const CLAIM_NAME: &str = "lens_producer_files_remaining";
const CURRENT_RESIDUAL_COUNT: i64 = 3;

#[test]
fn r3_gate_66_lens_producer_retirement_claim_executes_against_live_census() {
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
        ClaimResult::Pass => {}
        ClaimResult::Fail(reason) => {
            assert!(
                reason.contains(&format!(
                    "lens-producer subset observed {CURRENT_RESIDUAL_COUNT}"
                )),
                "`{CLAIM_NAME}` should report the live lens-producer residual count; got {reason:?}"
            );
        }
        ClaimResult::NotYetImplemented(reason) => {
            panic!("`{CLAIM_NAME}` must be executable for R3 gate #66, got NYI: {reason}")
        }
    }
}
