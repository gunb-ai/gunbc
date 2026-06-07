//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/frameworks/react.dag` —
//! T-4.7 React framework substrate must lower+infer with **zero** module
//! diagnostics (same 0-diag gate as `v4_extdeps_typescript_dag_smoke_test`).
//!
//! **Shape (E2 B-DELETE exemplar):** `v4_extdeps_react_dag_compiles` is the sole
//! hand-Rust receipt here — the **0-diag** gate. **7 A-class** behavioral receipts
//! are discriminating `.dag` witnesses in
//! `src/v4/test/claim/extdeps_react/structural_receipts.dag` (mutation-proven).
//! All **5 B-class** declaration-shape receipts deleted (operator 2026-06-07
//! tightened keep-bar): cited-but-hand-copied arm sets are mirrors, not independent
//! oracles; no external manifest-as-data; no keeper met all three keep criteria.
//!
//! **P5 receipt (INVARIANTS §P5(b)):** Explicit deferral ROADMAP.md § "Nine lanes" row
//! **T-PB-B** / `pb_rust_tests_outside_residual_zero` (ROADMAP.md:74). Dissolves when
//! the 0-diag gate ports to `.dag` TestClaim or generated harness coverage.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const REACT_DAG: &str = include_str!("../../../../v4/extdeps/frameworks/react.dag");
const REACT_PATH: &str = "src/v4/extdeps/frameworks/react.dag";

#[test]
fn v4_extdeps_react_dag_compiles() {
    match compile_to_dag(REACT_DAG, REACT_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{REACT_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{REACT_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{REACT_PATH}: {other:?}"),
    }
}
