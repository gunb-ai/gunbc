//! **Layer:** integration
//!
//! TC1 substrate lens eta-equivalence — strict-fire executable form (§1.8 gate #11).
//!
//! V1 first slice per Pattern-A / E6-G1.a (Q-PAFS Path A ACCEPTED 2026-05-06; Q-Reification
//! Option A ratified 2026-05-07 in PR #2096; Substrate Gate A merged 2026-05-07 in PR #2079).
//! Cross-Mgr split: Verification authors the .dag-side η-pair + lens consumer envelope; Evaluator
//! wires the non-vacuous lens-fold-over-`Dag` substrate-fact projection.
//!
//! Executable receipt: `BinaryDimensionReportEquals` resolves gate #11 by comparing
//! `analyze_symbolic_cost_dimension` at two η-pair workflow roots declared in `TestClaim.source`
//! (`tc1_eta_exec_direct` vs `tc1_eta_exec_eta_expanded`). Typed refs remain
//! `DimensionReport<Tc1EtaLensObservation>` per Pattern-A envelope authority.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str =
    include_str!("../fixtures/tc1_substrate_lens_eta_equivalence_strict_fire.dag");
const FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_strict_fire.dag";
const SUITE_NAME: &str = "tc1_substrate_lens_eta_equivalence_strict_fire_suite";

#[test]
fn tc1_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape() {
    run_on_larger_stack(|| {
        tc1_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape_inner()
    });
}

fn tc1_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape_inner() {
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
    assert!(
        matches!(&results[0].result, ClaimResult::Pass),
        "expected tc1_eta_equivalence_executable to Pass (symbolic-cost dimension spine equality \
         on η-pair roots), got {:?}",
        results[0].result
    );
}

// INVARIANTS P1 / P5 — checkable receipt: this integration crate must not build if the cited
// worker brief is missing from the worktree.
const _: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../docs/briefs/r3-v-pattern-a-tc1-v1-worker.md"
));
