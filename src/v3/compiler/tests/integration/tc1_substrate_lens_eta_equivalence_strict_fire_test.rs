//! **Layer:** integration
//!
//! TC1 substrate lens eta-equivalence — strict-fire executable form (§1.8 gate #11).
//!
//! V1 first slice per Pattern-A / E6-G1.a (Q-PAFS Path A ACCEPTED 2026-05-06; Q-Reification
//! Option A ratified 2026-05-07 in PR #2096; Substrate Gate A merged 2026-05-07 in PR #2079).
//! Cross-Mgr split: Verification authors the .dag-side η-pair + lens consumer envelope; Evaluator
//! wires the non-vacuous lens-fold-over-`Dag` substrate-fact projection.
//!
//! Executable receipt: gate #11 **Passes** on a Path A **proxy** — workflow-root **`composed`
//! `SymbolicCost`** from [`analyze_symbolic_cost_dimension`] on the two η-pair entry binds in
//! `TestClaim.source`, **not** native `DimensionReport<Tc1EtaLensObservation>` equality (that path is
//! gunbc#1972). Typed `.dag` refs stay `DimensionReport<Tc1EtaLensObservation>` for Pattern-A shape only.
//! **Ledger authority:** §1.8 **Status** stays **DECLARED** for carrier-aligned closure; this test is a
//! **slice receipt** only (`docs/r3-program-plan.md` §1.8 row #11 Notes vs Status — INVARIANTS P5).

use v3_compiler::test_runner::{ClaimResult, TestRunner};

use crate::common::{cached_compile_to_dag, run_on_larger_stack};

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
    let dag = cached_compile_to_dag(FIXTURE_SOURCE, FIXTURE_PATH);
    assert!(
        dag.diagnostics().is_empty(),
        "{FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let results = TestRunner::new(&dag).run_suite(SUITE_NAME);
    assert_eq!(results.len(), 1, "strict-fire suite has exactly one claim");
    assert_eq!(
        results[0].claim_name, "tc1_eta_equivalence_executable",
        "claim name must be the §1.8 #11 canonical gate name"
    );
    assert!(
        matches!(&results[0].result, ClaimResult::Pass),
        "expected tc1_eta_equivalence_executable to Pass (Path A proxy: identical workflow-root \
         composed SymbolicCost via analyze_symbolic_cost_dimension on η-pair roots — not \
         Tc1EtaLensObservation report parity; see try_eval_tc1_eta_equivalence_executable rustdoc \
         / gunbc#1972), got {:?}",
        results[0].result
    );
}
