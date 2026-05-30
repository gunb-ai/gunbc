//! Rust transport row for `v4.compiler.emit_host` (`run_emit_host_rust`).
//!
//! **Modeled authority:** `src/v4/compiler/emit_host.dag` — executable host-process boundary is
//! `tools/emit_host_runner`; substrate `.dag` assembles `EmitHostRunReceipt` from typed host facts.
//! This module is the W3 bridge exercised by integration tests until T-22 substrate eval dispatches
//! host transport directly (dissolves `emit_host_transport_not_wired`).

use emit_host_runner::{
    host_logical_run_from_exit, run_emit_host_rust, EmitHostFixtureInputs, EmitHostRunReceipt,
    HostExit,
};

/// Host-process transport: compile + run emitted Rust, returning the runner receipt.
pub fn run_emit_host_rust_transport(
    source: &str,
    inputs: &EmitHostFixtureInputs,
    work_dir: &std::path::Path,
) -> Result<EmitHostRunReceipt, emit_host_runner::HostSetupFailure> {
    run_emit_host_rust(source, inputs, work_dir)
}

/// True when the host exit witness is `Holds` (logical child succeeded).
pub fn host_exit_holds(exit: &HostExit) -> bool {
    exit.exit_holds()
}

/// Project logical stdout bytes when exit `Holds`; fail-closed otherwise.
pub fn host_stdout_bytes(exit: &HostExit, stdout_bytes: Vec<u8>) -> Option<Vec<u8>> {
    host_logical_run_from_exit(exit, stdout_bytes).map(|run| run.stdout_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use emit_host_runner::default_work_dir;

    const FIXTURE_SOURCE: &str =
        "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[0u8; 5]); }";

    #[test]
    fn bridge_runs_fixture_and_holds_exit() {
        let work_dir = default_work_dir(&format!("gunbc_emit_host_bridge_{}", std::process::id()));
        let inputs = EmitHostFixtureInputs {
            claim_input_root: "bridge_claim_input".into(),
            expected_eval_root: "bridge_expected_eval".into(),
        };
        let receipt = run_emit_host_rust_transport(FIXTURE_SOURCE, &inputs, &work_dir)
            .expect("transport");
        assert!(host_exit_holds(&receipt.exit));
        let stdout = host_stdout_bytes(&receipt.exit, receipt.stdout_bytes.clone())
            .expect("logical stdout");
        assert_eq!(stdout.len(), 5);
        emit_host_runner::runtime_value_parse_rust(&stdout).expect("parse");
    }
}
