//! Shared receipts for `tests/fixtures/r1_gates.dag` R1C-B / T-P0 gates (`p0_repeat_string_correct` +
//! `p0_repeat_string_space_pad_correct`, host sentinel / REST alignment suite).

use std::path::PathBuf;

use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

fn load_r1_gates_dag() -> v3_compiler::Dag {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate_path = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let gate_source = std::fs::read_to_string(&gate_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", gate_path.display()));
    match compile_to_dag(&gate_source, "src/v3/compiler/tests/fixtures/r1_gates.dag") {
        Ok(d) => d,
        Err(CompileError::Semantic(err_dag)) => panic!(
            "r1_gates.dag semantic errors: {:?}",
            err_dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(e) => panic!("r1_gates.dag: {e:?}"),
    }
}

/// Loads `r1_gates.dag` and asserts `p0_repeat_string_correct_gate` passes on every claim.
pub fn assert_p0_repeat_string_correct_gate_passes() {
    let dag = load_r1_gates_dag();
    let results = TestRunner::new(&dag).run_suite("p0_repeat_string_correct_gate");
    assert_eq!(results.len(), 2, "{results:?}");
    for r in &results {
        assert_eq!(
            r.result,
            ClaimResult::Pass,
            "expected Pass on {}, got {:?}",
            r.claim_name,
            r
        );
    }
}

/// `p0_no_fabrication_sentinel` + `p0_rest_ops_aligned` (`ExecuteCommand` host scripts).
pub fn assert_p0_host_sentinel_and_rest_gate_passes() {
    let dag = load_r1_gates_dag();
    let results = TestRunner::new(&dag).run_suite("p0_host_sentinel_and_rest_gate");
    assert_eq!(results.len(), 2, "{results:?}");
    for r in &results {
        assert_eq!(
            r.result,
            ClaimResult::Pass,
            "expected Pass on {}: {:?}",
            r.claim_name,
            r
        );
    }
}
