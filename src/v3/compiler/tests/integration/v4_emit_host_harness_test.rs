//! **Layer:** integration
//!
//! Structural ratchet for W2 emit-host harness substrate (`emit_host.dag`, `host_run.dag`,
//! `nat_semiring_rung34_eval.dag`, `tools/emit_host_runner`). Execution of full nat_semiring
//! emit-vs-eval remains blocked on W1 round-trip + W3 claims; this test proves symbols and
//! host-transport alignment only.

const EMIT_HOST_DAG: &str = include_str!("../../../../v4/extdeps/runtimes/emit_host.dag");
const HOST_RUN_DAG: &str = include_str!("../../../../v4/std/host_run.dag");
const VERDICT_DAG: &str = include_str!("../../../../v4/std/verdict.dag");
const NAT_RUNG34_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/nat_semiring_rung34_eval.dag");

#[test]
fn v4_host_run_carriers_present() {
    for needle in [
        "type EmitHostRunReceipt",
        "type HostExit",
        "type ByteString",
        "type FalsificationReceipt",
        "type ValueDiff",
    ] {
        assert!(
            HOST_RUN_DAG.contains(needle),
            "host_run.dag missing {needle}"
        );
    }
}

#[test]
fn v4_verdict_fail_carries_optional_falsification() {
    assert!(
        VERDICT_DAG.contains("falsification: Optional<FalsificationReceipt<T>>")
            && VERDICT_DAG.contains("fn verdict_fail"),
        "verdict.dag must extend Fail with optional FalsificationReceipt"
    );
}

#[test]
fn v4_emit_host_harness_surface_present() {
    for needle in [
        "fn runtime_value_parse",
        "fn runtime_value_parse_rust",
        "fn run_emit_host",
        "fn run_emit_host_rust",
        "fn run_test_claim_emit_vs_eval",
        "type FalsificationReceipt",
        "stdout_bytes: ByteString",
    ] {
        assert!(
            EMIT_HOST_DAG.contains(needle),
            "emit_host.dag missing {needle}"
        );
    }
}

#[test]
fn v4_nat_semiring_rung34_runner_present() {
    for needle in [
        "fn run_nat_semiring_rung34_eval",
        "fn nat_semiring_rung3_gate",
        "fn nat_semiring_rung4_gate",
        "fn nat_semiring_rungs_34_closed",
        "nat_semiring_rung34_runtime_value_rows",
    ] {
        assert!(
            NAT_RUNG34_DAG.contains(needle),
            "nat_semiring_rung34_eval.dag missing {needle}"
        );
    }
}

#[test]
fn emit_host_runner_rust_row_runtime_value_parse() {
    assert!(emit_host_runner::runtime_value_parse_rust(&[0u8; 5]).is_ok());
    assert!(emit_host_runner::runtime_value_parse_rust(&[1, 2]).is_err());
}
