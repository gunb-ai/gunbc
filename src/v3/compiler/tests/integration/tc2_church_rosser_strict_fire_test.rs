//! **Layer:** integration
//!
//! TC2 Church-Rosser / strategy-order — strict-fire executable form (§1.8 gate #12).
//!
//! Pairing `DimensionReport<Dag>` for LeftFirst vs RightFirst evaluation order per
//! `tc2_evaluation_order_independence_deferred.dag` staging and
//! `r3-v-pattern-a-tc2-v1-worker.md`. Unified consumer envelope: `BinaryDimensionReportEquals`.
//!
//! Today's runner returns `NotYetImplemented` with the canonical "structural shape is valid"
//! reason (`eval_binary_dimension_report_equals_shape` in `src/v3/compiler/src/test_runner.rs`).
//! Gate #12 stays **DECLARED** at this scaffold until Evaluator + substrate produce comparable
//! reports; the NYI receipt is fail-closed — a real equality eval **must** flip this test to
//! `Pass` when wiring lands (worker brief §Implementation slices).
//!
//! **Vacuity guard:** embedded `TestClaim.source` is a binary Transform application with two
//! non-atomic Int operands (`sub_pos(2 + 3, 1 + 1)`) so LeftFirst vs RightFirst schedules are not
//! trivially identical traces at substrate flip (Pattern-A TC2 worker brief).
//!
//! **INVARIANTS P5:** The §P5(b) **single checkable per-PR receipt** (SG-0 census / pairing /
//! `ROADMAP.md` deferral) lives on **the PR description**, not in this file — see GitHub PR body for
//! authoritative dissolution bookkeeping (`INVARIANTS.md` §P5 Dispatch-Discipline mechanism (b)).

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/tc2_church_rosser_strict_fire.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/tc2_church_rosser_strict_fire.dag";
const SUITE_NAME: &str = "tc2_church_rosser_strict_fire_suite";

/// Byte-identical to `TestClaim.source` in `tc2_church_rosser_strict_fire.dag` (canonical program authority).
const CLAIM_PROGRAM_SOURCE: &str = include_str!("../fixtures/tc2_church_rosser_executable.v3");
const CLAIM_PROGRAM_PATH: &str = "src/v3/compiler/tests/fixtures/tc2_church_rosser_executable.v3";

#[test]
fn tc2_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape() {
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
        results[0].claim_name, "tc2_church_rosser_executable",
        "claim name must be the §1.8 #12 canonical gate name"
    );
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

#[test]
fn tc2_executable_claim_source_lowers_without_diagnostics() {
    let dag = match compile_to_dag(CLAIM_PROGRAM_SOURCE, CLAIM_PROGRAM_PATH) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{CLAIM_PROGRAM_PATH}: embedded claim program should lower cleanly; got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {CLAIM_PROGRAM_PATH}: {other:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "{CLAIM_PROGRAM_PATH}: expected empty diagnostics on §1.8 claim program"
    );
}
