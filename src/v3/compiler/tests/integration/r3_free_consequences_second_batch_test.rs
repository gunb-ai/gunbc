//! **Layer:** integration
//!
//! R3 T-Free-Consequences second-batch author-now/fire-later claims. The auto-loop-parallelism
//! claims (#46–#48) assert `Pass` on a **staged** scalar only: `lane2_workflow` may be installed via
//! the magic-comment harness (`src/v3/compiler/src/r3_fc_lane2_loop_witness.rs`) and read by the
//! native `auto_loop_parallelism_pending_lens` path — see `ROADMAP.md` §"R3 second-batch auto-loop
//! scaffold". The directive lines are **author attestation**, not a compiler proof of iteration
//! independence or dependence; full `Lens<Iteration-Independence> * …` composition is out of scope
//! here. Gate **#48** additionally keeps a structural `std.list.fold` program in
//! `r3_free_consequences_auto_loop_parallelism_dependence.v3` so lowering still exercises a real
//! loop body; integration tests ratchet embedded `TestClaim.source` against that file byte-for-byte.
//! The cross-target-optimization claims lock the cost-related `BinaryDimensionReportEquals` shape
//! and stay `NotYetImplemented` until cost facts land.

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_free_consequences_second_batch.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_free_consequences_second_batch.dag";
const SUITE_NAME: &str = "r3_free_consequences_second_batch_suite";
/// Byte-sync authority for gate #48 `TestClaim.source` (`include_str` ↔ embedded `.dag` string).
const GATE_48_PROGRAM_AUTHORITY: &str =
    include_str!("../fixtures/r3_free_consequences_auto_loop_parallelism_dependence.v3");
const EXPECTED_CLAIMS: [&str; 5] = [
    "auto_loop_parallelism_provable_independence_emits_parallel",
    "auto_loop_parallelism_unproven_falls_back_sequential",
    "auto_loop_parallelism_dependence_emits_sequential",
    "cross_target_optimization_constant_fold_consistent",
    "cross_target_optimization_cost_structurally_derived",
];

static SECOND_BATCH_DAG: OnceLock<Dag> = OnceLock::new();

fn second_batch_dag() -> &'static Dag {
    SECOND_BATCH_DAG.get_or_init(|| match compile_to_dag(FIXTURE_SOURCE, FIXTURE_PATH) {
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
    })
}

#[test]
fn r3_free_consequences_second_batch_reaches_expected_consumer_shapes() {
    run_on_larger_stack(|| {
        r3_free_consequences_second_batch_reaches_expected_consumer_shapes_inner()
    });
}

fn r3_free_consequences_second_batch_reaches_expected_consumer_shapes_inner() {
    let dag = second_batch_dag();
    let gate_48 = dag
        .declaration_by_name("auto_loop_parallelism_dependence_emits_sequential")
        .expect("gate #48 TestClaim declaration");
    let claim_48 = TestClaimValue::from_declaration(gate_48).expect("TestClaimValue");
    assert_eq!(
        claim_48.source,
        GATE_48_PROGRAM_AUTHORITY,
        "embedded `TestClaim.source` must match `r3_free_consequences_auto_loop_parallelism_dependence.v3` byte-for-byte"
    );

    let results = TestRunner::new(dag).run_suite(SUITE_NAME);
    assert_eq!(results.len(), EXPECTED_CLAIMS.len());

    for (idx, (result, expected_name)) in results.iter().zip(EXPECTED_CLAIMS).enumerate() {
        assert_eq!(result.claim_name, expected_name);
        if idx < 3 {
            assert!(
                matches!(&result.result, ClaimResult::Pass),
                "expected {expected_name} to pass (LensOutputEquals matches staged loop-parallelism indicator), got {:?}",
                result.result
            );
        } else {
            assert!(
                matches!(&result.result, ClaimResult::NotYetImplemented(_)),
                "expected {expected_name} to stay author-now/fire-later on BinaryDimensionReportEquals, got {:?}",
                result.result
            );
        }
    }
}
