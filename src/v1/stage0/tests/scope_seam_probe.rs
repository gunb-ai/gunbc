//! THROWAWAY MEASUREMENT — not for merge. Two questions, one whole-corpus prepare.
//!
//! PART A (blocking main): `test.claim.live_deploy.emit.twin_and_production_configure_
//! disjoint_tailscale_endpoints` is BUDGET-REFUSED on the required floor at "at least
//! 5001ms against 5000ms". 5001 is the INTERRUPT POINT, not the cost — the row has never
//! run to completion on the required path, so its true cost is unmeasured. `evaluation_frame`
//! applies NO budget (the cap lives in the floor runner), so calling `run_claim_measured`
//! directly runs it to completion and yields the real number.
//!   Siblings in the same module are controls: if they are also slow, the cost is
//!   module/corpus-shaped; if only this row is, it is the row's own work.
//!
//! PART B: for an enrolled expected-red identity that RUNTIME-ERRORS with
//! `no such function: <name>`, is the declaring module ABSENT FROM SCOPE (a
//! `claim_scope_for` defect) or PRESENT but missing from the interpreter's `fn_nodes`
//! index (`authored_name_at`)? Measured population: 60 of 72 distinct missing names are
//! module-scope `data`, and NONE is multiply declared.

use v1_compiler::cli_run::{
    claim_scope_for, evaluation_frame, floor_prepared_subject_exclusions, prepare_repository_once,
    run_claim_measured,
};
use v1_compiler::v1_interpreter::ExecutionMode;

#[test]
#[ignore = "whole-corpus prepare; measurement only"]
fn probe() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (prepared, _views) =
        prepare_repository_once(&roots, &floor_prepared_subject_exclusions()).expect("prepare");
    eprintln!(
        "[probe] prepared_subject_modules={}",
        prepared.graph.modules.len()
    );

    // ---------- PART A: the cost that is blocking main ----------
    let emit_entry = "test.claim.live_deploy.emit";
    match claim_scope_for(&prepared, emit_entry) {
        Ok(scope) => {
            eprintln!(
                "[probe] COST entry={emit_entry} scope_modules={}",
                scope.module_count
            );
            let frame = evaluation_frame(&scope, ExecutionMode::Hermetic, None, None);
            for f in [
                "twin_and_production_configure_disjoint_tailscale_endpoints",
                "witness_spec_targets_srv1",
                "witness_spec_listen_port_is_8080",
                "witness_apply_script_contains_systemd_and_tailscale",
            ] {
                let q = format!("{emit_entry}.{f}");
                let t0 = std::time::Instant::now();
                let (outcome, receipt) = run_claim_measured(&frame, &prepared.subject_digest, &q);
                eprintln!(
                    "[probe] COST fn={f} instant_ms={} receipt_wall_ms={} outcome={outcome:?}",
                    t0.elapsed().as_millis(),
                    receipt.wall_nanos / 1_000_000
                );
                eprintln!("[probe] COST receipt={receipt:?}");
            }
        }
        Err(e) => eprintln!("[probe] COST SCOPE-REFUSED: {e}"),
    }

    // ---------- PART B: scope membership vs declaration index ----------
    let provider = "gunbc.srv3_os_install_diagnostic";
    eprintln!(
        "[probe] provider_in_prepared_subject={}",
        prepared
            .graph
            .modules
            .iter()
            .any(|m| m.func_env.name == provider)
    );
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
                eprintln!("[probe] {label} scope_modules={}", scope.module_count);
                let frame = evaluation_frame(&scope, ExecutionMode::Hermetic, None, None);
                let (outcome, _) = run_claim_measured(&frame, &prepared.subject_digest, function);
                eprintln!("[probe] {label} outcome={outcome:?}");
            }
            Err(e) => eprintln!("[probe] {label} SCOPE-REFUSED: {e}"),
        }
    }
}
