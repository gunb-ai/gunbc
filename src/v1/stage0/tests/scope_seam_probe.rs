//! THROWAWAY MEASUREMENT — not for merge.
//!
//! WHAT DECIDES THE OWNER. For an enrolled expected-red identity that RUNTIME-ERRORS with
//! `no such function: <name>`, is the declaring module ABSENT FROM THE CLAIM SCOPE
//! (a `claim_scope_for` defect), or PRESENT IN SCOPE but missing from the interpreter's
//! `fn_nodes` index (a declaration-identity defect one layer down, in `authored_name_at`)?
//! Opposite repairs, different owners.
//!
//! THE DISCRIMINATING PAIR. Both modules bare-reference `srv3_install_hang_no_router_lease_ms`
//! — a module-scope `data` declared exactly once, in `gunbc.srv3_os_install_diagnostic` — and
//! NEITHER imports it. They differ only in enrollment:
//!
//!   SPECIMEN: test.claim.host_standup_assimilation_deduction — enrolled expected-red (5 ids)
//!   CONTROL:  test.claim.temporal_effect_spine               — NOT on the roster
//!
//! If the CONTROL passes, a bare cross-module `data` reference resolves fine in general, and
//! "the reference closure cannot see bare data references" is refuted as the general cause.
//! If BOTH error, the cause is the edge kind, not anything about the specimen's module.

use v1_compiler::cli_run::{
    claim_scope_for, evaluation_frame, floor_prepared_subject_exclusions, prepare_repository_once,
    run_claim_measured,
};
use v1_compiler::v1_interpreter::ExecutionMode;

#[test]
#[ignore = "whole-corpus prepare; measurement only"]
fn scope_seam_discriminating_pair() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (prepared, _views) =
        prepare_repository_once(&roots, &floor_prepared_subject_exclusions()).expect("prepare");

    // Is the provider even in the prepared subject? If not, both arms are meaningless.
    let provider = "gunbc.srv3_os_install_diagnostic";
    let in_subject = prepared
        .graph
        .modules
        .iter()
        .any(|m| m.func_env.name == provider);
    eprintln!("[scope-probe] provider_in_prepared_subject={in_subject}");

    for (label, entry, function) in [
        (
            "SPECIMEN(enrolled)",
            "test.claim.host_standup_assimilation_deduction",
            "test.claim.host_standup_assimilation_deduction.post_install_lease_row_deduces_converged_noop",
        ),
        (
            "CONTROL(not-enrolled)",
            "test.claim.temporal_effect_spine",
            "test.claim.temporal_effect_spine.srv3_stall_budget_limits_match_runbook",
        ),
    ] {
        match claim_scope_for(&prepared, entry) {
            Ok(scope) => {
                eprintln!(
                    "[scope-probe] {label} entry={entry} module_count={} ambiguous_bare_names={}",
                    scope.module_count, scope.ambiguous_bare_names
                );
                let frame = evaluation_frame(&scope, ExecutionMode::Hermetic, None, None);
                let (outcome, _) =
                    run_claim_measured(&frame, &prepared.subject_digest, function);
                eprintln!("[scope-probe] {label} outcome={outcome:?}");
            }
            Err(e) => eprintln!("[scope-probe] {label} SCOPE-REFUSED: {e}"),
        }
    }
}
