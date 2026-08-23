//! THROWAWAY MEASUREMENT — not for merge.
//!
//! Question: for a witness that runtime-errors with `no such function: <data name>`,
//! is the declaring module ABSENT FROM THE CLAIM SCOPE (a claim_scope_for defect), or
//! PRESENT IN SCOPE but missing from the interpreter's `fn_nodes` index (a declaration-
//! identity defect one layer down)? These have different owners and opposite repairs.
//!
//! Specimen: `test.claim.temporal_effect_spine` bare-references
//! `srv3_install_hang_no_router_lease_ms`, a module-scope `data` declared exactly once,
//! in `gunbc.srv3_os_install_diagnostic`, which it does not import.

use v1_compiler::cli_run::{
    claim_scope_for, evaluation_frame, floor_prepared_subject_exclusions, prepare_repository_once,
    run_claim_measured,
};
use v1_compiler::v1_interpreter::ExecutionMode;

#[test]
#[ignore = "whole-corpus prepare; measurement only"]
fn scope_seam_specimen() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (prepared, _views) =
        prepare_repository_once(&roots, &floor_prepared_subject_exclusions()).expect("prepare");

    let entry = "test.claim.temporal_effect_spine";
    let scope = claim_scope_for(&prepared, entry).expect("scope");
    eprintln!(
        "[scope-probe] scope_identity_len={} module_count={} ambiguous_bare_names={}",
        scope.scope_identity.len(),
        scope.module_count,
        scope.ambiguous_bare_names
    );

    let frame = evaluation_frame(&scope, ExecutionMode::Hermetic, None, None);

    // The specimen: plain bare ExprVar references, no call, no record literal.
    let (outcome, _) = run_claim_measured(
        &frame,
        &prepared.subject_digest,
        "test.claim.temporal_effect_spine.srv3_stall_budget_limits_match_runbook",
    );
    eprintln!("[scope-probe] SPECIMEN outcome={outcome:?}");

    // POSITIVE CONTROL: a claim in the same module that does not reach across this edge.
    // If this also errors, the probe is measuring something broader than the seam.
    let (control, _) = run_claim_measured(
        &frame,
        &prepared.subject_digest,
        "test.claim.temporal_effect_spine.stall_cause_fold_router_absent_dominates_silent_installer",
    );
    eprintln!("[scope-probe] CONTROL outcome={control:?}");
}
