//! **Layer:** integration
//!
//! TC1 substrate lens eta-equivalence — strict-fire executable form (§1.8 gate #11).
//!
//! V1 first slice per Pattern-A / E6-G1.a (Q-PAFS Path A ACCEPTED 2026-05-06; Q-Reification
//! Option A ratified 2026-05-07 in PR #2096; Substrate Gate A merged 2026-05-07 in PR #2079).
//! Cross-Mgr split per Evaluator E3.c (gunbc#1970): Verification authors the .dag-side η-pair +
//! lens consumer envelope; Evaluator wires lens-fold-over-`Dag` substrate-fact projection.
//!
//! Today's runner returns `NotYetImplemented` with the canonical "structural shape is valid"
//! reason (`eval_binary_dimension_report_equals_shape` in `src/v3/compiler/src/test_runner.rs`).
//! Per Director (C-modified) ratification at gunbc#828 2026-05-07: §1.8 gate #11 status STAYS
//! DECLARED on this scaffold landing; the NotYetImplemented sentinel is fail-closed-by-
//! construction (any actual implementation that runs WILL fail this assertion when E3.c lands,
//! forcing fixture upgrade). Status flips DECLARED → CONSUMER_LANDED → PASSING in one move on
//! Evaluator E3.c (gunbc#1970) merge + assertion upgrade.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str =
    include_str!("../fixtures/tc1_substrate_lens_eta_equivalence_strict_fire.dag");
const FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_strict_fire.dag";
const SUITE_NAME: &str = "tc1_substrate_lens_eta_equivalence_strict_fire_suite";

#[test]
fn tc1_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape() {
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
    assert_eq!(results.len(), 1, "strict-fire suite has exactly one claim");
    assert_eq!(
        results[0].claim_name, "tc1_eta_equivalence_executable",
        "claim name must be the §1.8 #11 canonical gate name"
    );
    // Today: shape-valid NotYetImplemented (runner waits on Evaluator E3.c / gunbc#1970).
    // When E3.c lands, this assertion flips from NotYetImplemented to Pass — that is the
    // §1.8 #11 CONSUMER_LANDED → PASSING transition without further fixture edits.
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::NotYetImplemented(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("structural shape is valid")
        ),
        "expected BinaryDimensionReportEquals shape-valid NotYetImplemented, got {:?}",
        results[0].result
    );
}

// INVARIANTS P1 / P5 — checkable receipt: this integration crate must not build if the cited
// worker brief is missing from the worktree.
const _: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../docs/briefs/r3-v-pattern-a-tc1-v1-worker.md"
));
