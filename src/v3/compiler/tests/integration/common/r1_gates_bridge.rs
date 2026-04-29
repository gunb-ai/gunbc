//! Shared receipts for `tests/fixtures/r1_gates.dag` R1C-B / T-P0 gate (structural
//! `p0_repeat_string_correct`).
//!
//! Keeps a single implementation for suite names + compile path so `test_runner_test` and
//! feature-specific modules (e.g. P0 oracle) cannot drift.

use std::path::PathBuf;

use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

/// Loads `r1_gates.dag` and asserts `p0_repeat_string_correct_gate` evaluates one `Pass` claim.
pub fn assert_p0_repeat_string_correct_gate_passes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate_path = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let gate_source = std::fs::read_to_string(&gate_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", gate_path.display()));
    let dag = match compile_to_dag(&gate_source, "src/v3/compiler/tests/fixtures/r1_gates.dag") {
        Ok(d) => d,
        Err(CompileError::Semantic(err_dag)) => panic!(
            "r1_gates.dag semantic errors: {:?}",
            err_dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(e) => panic!("r1_gates.dag: {e:?}"),
    };
    let results = TestRunner::new(&dag).run_suite("p0_repeat_string_correct_gate");
    assert_eq!(results.len(), 1, "{results:?}");
    assert_eq!(
        results[0].result,
        ClaimResult::Pass,
        "expected Pass on p0_repeat_string_correct, got {:?}",
        results[0]
    );
}
