//! **Layer:** integration
//!
//! TC3 strong-normalization / Pattern-A second-mover — strict-fire executable form
//! (§1.8 gate #13).
//!
//! Pairing `DimensionReport<Dag>` for baseline evaluation-step vs bounded-step /
//! termination-evidence projections per `tc3_strong_normalization_deferred.dag` staging
//! and `r3-v-pattern-a-tc3-v1-worker.md`. Unified consumer envelope:
//! `BinaryDimensionReportEquals` (audit §Pattern A Compositionality Verdict).
//!
//! Today's runner returns `NotYetImplemented` with the canonical "structural shape is valid"
//! reason (`eval_binary_dimension_report_equals_shape` in `src/v3/compiler/src/test_runner.rs`).
//! Gate #13 stays **DECLARED** at this scaffold landing; the NYI receipt is fail-closed —
//! a real equality eval **must** flip this test to `Pass` when (a)+(b) wiring lands
//! (worker brief §Implementation slices). Strict-fire **PASSING** requires bundle
//! stage (b): T-FixedPoint termination semantics (gunbc#2087) + Evaluator eval-step /
//! bounded-step producer surface.
//!
//! **Vacuity guard:** embedded `TestClaim.source` is a structurally bounded computation
//! (`succ(succ(0))`) — a multi-step evaluation whose baseline eval-step trace and
//! termination-evidence projection diverge in step structure (Pattern-A TC3 worker brief).
//!
//! **INVARIANTS P5:** The §P5(b) **single checkable per-PR receipt** (SG-0 census / pairing /
//! `ROADMAP.md` deferral) lives on **the PR description**, not in this file — see GitHub PR
//! body for authoritative dissolution bookkeeping (`INVARIANTS.md` §P5 Dispatch-Discipline
//! mechanism (b)).

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/tc3_strong_normalization_strict_fire.dag");
const FIXTURE_PATH: &str =
    "src/v3/compiler/tests/fixtures/tc3_strong_normalization_strict_fire.dag";
const SUITE_NAME: &str = "tc3_strong_normalization_strict_fire_suite";

/// Sidecar bytes for the §1.8 claim program — **must** match parsed `TestClaim.source` from the
/// `.dag` fixture (`tc3_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape`).
const CLAIM_PROGRAM_SOURCE: &str =
    include_str!("../fixtures/tc3_strong_normalization_executable.v3");
const CLAIM_PROGRAM_PATH: &str =
    "src/v3/compiler/tests/fixtures/tc3_strong_normalization_executable.v3";

const TC3_CLAIM_DATA_NAME: &str = "tc3_pattern_a_second_mover_executable_claim";

#[test]
fn tc3_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape() {
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

    let claim_decl = dag
        .declaration_by_name(TC3_CLAIM_DATA_NAME)
        .unwrap_or_else(|| panic!("missing `{TC3_CLAIM_DATA_NAME}` in {FIXTURE_PATH}"));
    let claim = TestClaimValue::from_declaration(claim_decl).unwrap_or_else(|e| {
        panic!("`{TC3_CLAIM_DATA_NAME}` should lower as TestClaim: {e}");
    });
    assert_eq!(
        claim.source, CLAIM_PROGRAM_SOURCE,
        "embedded `TestClaim.source` in {FIXTURE_PATH} must stay byte-identical to {CLAIM_PROGRAM_PATH} (single program authority per INVARIANTS P2)"
    );
    assert_eq!(
        claim.file_name, "tc3_strong_normalization_executable.v3",
        "TestClaim.file_name must match the sidecar program path used in lowering checks"
    );

    let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
        Ok(program_dag) => program_dag,
        Err(CompileError::Semantic(program_dag)) => panic!(
            "embedded TestClaim.source should lower cleanly; diagnostics: {:?}",
            program_dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("embedded TestClaim.source compile error: {other:?}"),
    };
    assert!(
        program_dag.diagnostics().is_empty(),
        "embedded claim program expected no diagnostics, got {:?}",
        program_dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let results = TestRunner::new(&dag).run_suite(SUITE_NAME);
    assert_eq!(results.len(), 1, "strict-fire suite has exactly one claim");
    assert_eq!(
        results[0].claim_name, "tc3_pattern_a_second_mover_executable",
        "claim name must be the §1.8 #13 canonical gate name"
    );
    // Today: shape-valid NotYetImplemented (runner waits on stage (b): T-FixedPoint
    // termination semantics + Evaluator eval-step producer). When (a)+(b) land, this
    // assertion flips from NotYetImplemented to Pass — that is the §1.8 #13
    // CONSUMER_LANDED → PASSING transition without further fixture edits.
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
    "/../../../docs/briefs/r3-v-pattern-a-tc3-v1-worker.md"
));
