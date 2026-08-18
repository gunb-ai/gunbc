//! Scoped required-floor path for compute_board admission witnesses — discriminates
//! claim_batch (entry closure) from claim_scope_for + evaluation_frame (PreparedSubject).

use v1_compiler::cli_run::{
    claim_scope_for, evaluation_frame, floor_prepared_subject_exclusions, prepare_repository_once,
    run_claim_measured, ClaimOutcome,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn scoped_floor_outcome(function: &str) -> ClaimOutcome {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let prepared =
        prepare_repository_once(&roots, &floor_prepared_subject_exclusions()).expect("prepare");
    let scope =
        claim_scope_for(&prepared, "test.claim.compute_board_admission_witness").expect("scope");
    let frame = evaluation_frame(&scope, ExecutionMode::Hermetic, None, None);
    let (outcome, _) = run_claim_measured(&frame, &prepared.subject_digest, function);
    outcome
}

#[test]
#[ignore = "whole-corpus prepare; run locally before pushing compute_board matcher fixes"]
fn scoped_floor_two_components_sharing_identity_refuse() {
    let outcome = scoped_floor_outcome(
        "test.claim.compute_board_admission_witness.w_two_components_sharing_an_identity_refuse",
    );
    assert_eq!(
        outcome,
        ClaimOutcome::Pass,
        "scoped floor path: {outcome:?}"
    );
}

#[test]
#[ignore]
fn scoped_floor_coherent_fixture_is_admitted() {
    let outcome = scoped_floor_outcome(
        "test.claim.compute_board_admission_witness.w_a_coherent_fixture_is_admitted",
    );
    assert_eq!(
        outcome,
        ClaimOutcome::Pass,
        "scoped floor path: {outcome:?}"
    );
}
