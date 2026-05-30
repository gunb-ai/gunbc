//! **Layer:** integration
//!
//! **W2 / joint spec** `compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md` §4.2:
//! emit-host harness substrate (`emit_host.dag`, `host_run.dag`, `test_claim_falsification.dag`,
//! `nat_semiring_rung34_eval.dag`) + executable boundary `tools/emit_host_runner`.
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-38 / rung-4 host receipt path.
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` land in the same PR. **This PR (+1 census path):**
//! `v4_emit_host_harness_test.rs` — behavior-driven `run_emit_host_rust` (compile + run fixture,
//! `HostExit::Ok`, five-byte stdout parse) plus minimal `.dag` surface needles for carriers wired
//! in this PR. **Dissolution trigger:** W3 populates `nat_semiring_rung34_runtime_value_rows`
//! and wires `run_emit_host_rust` in `emit_host.dag` to invoke `tools/emit_host_runner` (removes
//! `emit_host_transport_not_wired` on the Rust row); delete this file when rung-3/4 claims +
//! generated harness replace hand-Rust probes (see `nat_semiring_rung34_eval.dag` roster comment).
//!
//! **TESTING.md:** substrate `.dag` eval remains hermetic (`run_emit_host_rust` → `Rejected` until
//! W3/CI wiring); this test exercises the Rust transport the `.dag` row models, not substrate eval.
//! The transport uses bounded child I/O (`HOST_BUILD_TIMEOUT`, `HOST_RUN_TIMEOUT`,
//! `HOST_STREAM_BYTE_CAP` in `emit_host_runner`) and isolates `CARGO_TARGET_DIR` under `work_dir`.

const EMIT_HOST_DAG: &str = include_str!("../../../../v4/extdeps/runtimes/emit_host.dag");
const HOST_RUN_DAG: &str = include_str!("../../../../v4/std/host_run.dag");
const FALSIFICATION_DAG: &str = include_str!("../../../../v4/std/test_claim_falsification.dag");

/// Minimal fixture: five stdout bytes (MVP runtime value `5` alignment).
const EMIT_HOST_FIXTURE_SOURCE: &str =
    "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[0u8; 5]); }";

#[test]
fn emit_host_runner_rust_row_builds_runs_and_parses_stdout() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_v4_emit_host_harness_{}",
        std::process::id()
    ));
    let receipt = emit_host_runner::run_emit_host_rust(EMIT_HOST_FIXTURE_SOURCE, &work_dir)
        .expect("run_emit_host_rust");
    assert!(
        matches!(receipt.exit, emit_host_runner::HostExit::Ok(_)),
        "expected successful host exit, got {:?}",
        receipt.exit
    );
    emit_host_runner::runtime_value_parse_rust(&receipt.stdout_bytes)
        .expect("runtime_value_parse_rust on fixture stdout");
}

#[test]
fn v4_host_run_logical_run_carrier_present() {
    for needle in [
        "type HostRunStdout",
        "type HostLogicalRun",
        "logical_run: Outcome<HostLogicalRun>",
        "fn host_logical_run_from_exit",
    ] {
        assert!(
            HOST_RUN_DAG.contains(needle),
            "host_run.dag missing {needle}"
        );
    }
}

#[test]
fn v4_falsification_execution_evidence_sum_present() {
    for needle in [
        "type FalsificationReceipt",
        "subject: Subj",
        "🟡 coproduct dissolution — feature:verdict-surface-execution-evidence",
        "type ExecutionEvidence",
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
fn v4_emit_host_fail_closed_transport_and_logical_run_gate() {
    for needle in [
        "emit_host_transport_not_wired",
        "match host_receipt.logical_run",
        "claim_input_root: Node",
        "expected_eval_root: Node",
    ] {
        assert!(
            EMIT_HOST_DAG.contains(needle),
            "emit_host.dag missing {needle}"
        );
    }
}
