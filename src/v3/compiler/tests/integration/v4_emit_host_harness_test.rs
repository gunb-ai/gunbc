//! **Layer:** integration
//!
//! Structural ratchet for W2 emit-host harness substrate (`emit_host.dag`, `host_run.dag`,
//! `test_claim_falsification.dag`, `nat_semiring_rung34_eval.dag`, `tools/emit_host_runner`).
//! Execution of full nat_semiring emit-vs-eval remains blocked on W1 round-trip + W3 claims.

const EMIT_HOST_DAG: &str = include_str!("../../../../v4/extdeps/runtimes/emit_host.dag");
const HOST_RUN_DAG: &str = include_str!("../../../../v4/std/host_run.dag");
const FALSIFICATION_DAG: &str = include_str!("../../../../v4/std/test_claim_falsification.dag");
const VERDICT_DAG: &str = include_str!("../../../../v4/std/verdict.dag");
const EVAL_DAG: &str = include_str!("../../../../v4/compiler/05_eval.dag");
const NAT_RUNG34_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/nat_semiring_rung34_eval.dag");

#[test]
fn v4_host_run_carriers_present() {
    for needle in [
        "type EmitHostRunReceipt",
        "type HostExit",
        "type HostRunStdout",
        "type HostLogicalRun",
        "logical_run: Outcome<HostLogicalRun>",
        "fn host_logical_run_from_exit",
        "type ByteString",
        "type ValueDiff",
    ] {
        assert!(
            HOST_RUN_DAG.contains(needle),
            "host_run.dag missing {needle}"
        );
    }
}

#[test]
fn v4_falsification_contract_present() {
    for needle in [
        "type FalsificationReceipt",
        "type ExecutionEvidence",
        "type InterpreterTrace",
        "subject: TestClaimEvalSubject",
        "evidence: ExecutionEvidence",
        "Host { receipt: EmitHostRunReceipt }",
        "Interpreter { trace: InterpreterTrace }",
        "EvidenceNone",
    ] {
        assert!(
            FALSIFICATION_DAG.contains(needle),
            "test_claim_falsification.dag missing {needle}"
        );
    }
}

#[test]
fn v4_verdict_s_t_parameter_and_fail_falsification() {
    assert!(
        VERDICT_DAG.contains("type Verdict<S, T>")
            && VERDICT_DAG.contains("falsification: Optional<FalsificationReceipt<S, T>>")
            && VERDICT_DAG.contains("fn verdict_combine<S, T>")
            && VERDICT_DAG.contains("fn verdict_fail"),
        "verdict.dag must use Verdict<S, T> with parameterized falsification"
    );
    assert!(
        EVAL_DAG.contains("verdict: Verdict<S, A>"),
        "TestClaimRun.verdict must widen to Verdict<S, A>"
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
        "fn run_test_claim_emit_vs_eval_for_claim",
        "claim_input_root: Node",
        "expected_eval_root: Node",
        "evidence: Host { receipt: host_receipt }",
        "match host_receipt.logical_run",
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
