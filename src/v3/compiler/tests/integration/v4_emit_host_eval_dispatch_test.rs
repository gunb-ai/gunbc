//! **Layer:** integration
//!
//! T-22 eval dispatch behavior receipts for `run_emit_host_rust` substrate intercept
//! (`emit_host_eval.rs`) and B3 `run_host_process` / SignedI32Le transport. Complements
//! `v4_emit_host_harness_test.rs` (bridge + surface); this file ratchets runner fail-closed
//! contracts the eval hook must preserve when mapping into `emit_host_receipt_from_source`.
//!
//! **P5 receipt (Mechanism (b)):** `EXPECTED_HAND_AUTHORED_TEST` row in `sg0_census_test.rs`;
//! lane `T-PB-B` / `pb_rust_tests_outside_residual_zero`. Dissolution: substrate-only authority.

use emit_host_runner::{
    default_work_dir, run_emit_host_rust, run_host_process, runtime_value_parse_signed_i32_le,
    EmitHostFixtureInputs, HostLogicalFailure, TS_HOST_TRANSPORT_MVP1_IDENTITY,
};

const FIXTURE_SOURCE_PASS: &str =
    "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[0u8; 5]); }";

const FIXTURE_SOURCE_NONZERO: &str = "fn main() { std::process::exit(1); }";

const TS_ADD_FIXTURE: &str = "function add(x: number, y: number): number { return x + y; }\n";

fn mvp2_inputs() -> EmitHostFixtureInputs {
    EmitHostFixtureInputs {
        claim_input_root: "eval_dispatch_claim".to_string(),
        expected_eval_root: "eval_dispatch_expected".to_string(),
    }
}

#[test]
fn emit_host_eval_dispatch_runner_pass_fixture_holds_and_logical_run_projects() {
    let work_dir = default_work_dir(&format!(
        "gunbc_emit_host_eval_dispatch_pass_{}",
        std::process::id()
    ));
    let receipt = run_emit_host_rust(FIXTURE_SOURCE_PASS, &mvp2_inputs().transport(), &work_dir)
        .expect("host setup");
    assert!(
        receipt.exit.exit_holds(),
        "eval dispatch must preserve Holds exit for pass fixture, got {:?}",
        receipt.exit
    );
    assert!(
        emit_host_runner::host_logical_run_from_exit(&receipt.exit, receipt.stdout_bytes.clone())
            .is_some(),
        "logical_run substrate rule: stdout only when exit Holds"
    );
}

#[test]
fn emit_host_eval_dispatch_runner_nonzero_exit_denies_logical_run_projection() {
    let work_dir = default_work_dir(&format!(
        "gunbc_emit_host_eval_dispatch_fail_{}",
        std::process::id()
    ));
    let receipt = run_emit_host_rust(
        FIXTURE_SOURCE_NONZERO,
        &mvp2_inputs().transport(),
        &work_dir,
    )
    .expect("host setup");
    assert!(!receipt.exit.exit_holds(), "nonzero fixture must not Hold");
    assert!(
        emit_host_runner::host_logical_run_from_exit(&receipt.exit, receipt.stdout_bytes.clone())
            .is_none(),
        "logical_run must be fail-closed when exit Violates"
    );
    assert!(matches!(
        receipt.exit.outcome,
        emit_host_runner::HostExitOutcome::Accepted(emit_host_runner::ExitWitness::Violates(
            HostLogicalFailure::ExitedNonzero { .. }
        ))
    ));
}

#[test]
fn emit_host_eval_dispatch_run_host_process_ts_add_stdout_is_five() {
    let work_dir = default_work_dir(&format!(
        "gunbc_b3_run_host_process_add_{}",
        std::process::id()
    ));
    let receipt = run_host_process(
        TS_HOST_TRANSPORT_MVP1_IDENTITY,
        TS_ADD_FIXTURE,
        &mvp2_inputs().transport(),
        &work_dir,
    )
    .expect("host setup");
    assert!(
        receipt.exit.exit_holds(),
        "B3 TS transport must Hold for add fixture, got {:?}",
        receipt.exit
    );
    let parsed = runtime_value_parse_signed_i32_le(&receipt.stdout_bytes).expect("i32 le parse");
    assert_eq!(parsed, 5, "add(2,3) harness must emit SignedI32Le 5");
}
