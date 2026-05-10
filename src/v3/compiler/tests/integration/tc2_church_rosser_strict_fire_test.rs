//! **Layer:** integration
//!
//! TC2 Church-Rosser / strategy-order — strict-fire executable form (R3 gate #12).
//!
//! Pairing `DimensionReport<Dag>` for LeftFirst vs RightFirst evaluation order per
//! `tc2_evaluation_order_independence_deferred.dag` staging and
//! `r3-v-pattern-a-tc2-v1-worker.md`. Unified consumer envelope: `BinaryDimensionReportEquals`.
//!
//! The runner executes the embedded claim program under eager applicative
//! `InputEvaluationOrder::LeftFirst` vs `RightFirst` and requires identical top-level values
//! (confluence slice). Other `BinaryDimensionReportEquals` claims remain NYI at the generic
//! `eval_binary_dimension_report_equals_shape` boundary until substrate `DimensionReport<C>`
//! producers land.
//!
//! **Vacuity guard:** embedded `TestClaim.source` is a binary Transform application with two
//! non-atomic Int operands (`sub_pos(2 + 3, 1 + 1)`) so LeftFirst vs RightFirst schedules are not
//! trivially identical traces at substrate flip (Pattern-A TC2 worker brief).
//!
//! **INVARIANTS P5:** The P5(b) **single checkable per-PR receipt** (SG-0 census / pairing /
//! `ROADMAP.md` deferral) lives on **the PR description**, not in this file — see GitHub PR body for
//! authoritative dissolution bookkeeping (`INVARIANTS.md` P5 Dispatch-Discipline mechanism (b)).

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/tc2_church_rosser_strict_fire.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/tc2_church_rosser_strict_fire.dag";
const SUITE_NAME: &str = "tc2_church_rosser_strict_fire_suite";

/// Sidecar bytes for the gate #12 claim program — **must** match parsed `TestClaim.source` from the
/// `.dag` fixture (`tc2_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape`).
const CLAIM_PROGRAM_SOURCE: &str = include_str!("../fixtures/tc2_church_rosser_executable.v3");
const CLAIM_PROGRAM_PATH: &str = "src/v3/compiler/tests/fixtures/tc2_church_rosser_executable.v3";

const TC2_CLAIM_DATA_NAME: &str = "tc2_church_rosser_executable_claim";

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

    let claim_decl = dag
        .declaration_by_name(TC2_CLAIM_DATA_NAME)
        .unwrap_or_else(|| panic!("missing `{TC2_CLAIM_DATA_NAME}` in {FIXTURE_PATH}"));
    let claim = TestClaimValue::from_declaration(claim_decl).unwrap_or_else(|e| {
        panic!("`{TC2_CLAIM_DATA_NAME}` should lower as TestClaim: {e}");
    });
    assert_eq!(
        claim.source, CLAIM_PROGRAM_SOURCE,
        "embedded `TestClaim.source` in {FIXTURE_PATH} must stay byte-identical to {CLAIM_PROGRAM_PATH} (single program authority per INVARIANTS P2)"
    );
    assert_eq!(
        claim.file_name, "tc2_church_rosser_executable.v3",
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
        results[0].claim_name, "tc2_church_rosser_executable",
        "claim name must be the gate #12 canonical gate name"
    );
    assert_eq!(
        results[0].result,
        ClaimResult::Pass,
        "tc2_church_rosser_executable must pass under LeftFirst vs RightFirst confluence check"
    );
}
