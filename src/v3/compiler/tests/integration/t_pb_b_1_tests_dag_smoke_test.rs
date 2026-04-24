//! **Layer:** integration
//!
//! T-PB-B-1 — `src/v3/compiler/tests/dag/*.dag` compile smoke. Landed `.dag` `TestClaim`
//! modules for the eventual Testgen runner; **does not** evaluate predicates or assert `pb_*`.
//! Coordinate with Testgen manager before deleting overlapping Rust tests.
//!
//! `assert_dag_module_lowers_cleanly` accepts `Ok(dag)` and `Err(CompileError::Semantic(dag))`
//! then requires empty module diagnostics — lowering shape only, not `TestPredicate` proof
//! (see `docs/briefs/t-pb-b-1.md` *Compile-smoke caveat*; revisit when the runner evaluates
//! `FailsWithDiagnostic` for real).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

fn assert_dag_module_lowers_cleanly(source: &'static str, file: &'static str) {
    let dag = match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected compile error for {file}: {other:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "{file} should lower without diagnostics, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn t_pb_b_1_pipeline_smoke_dag_lowers_cleanly() {
    assert_dag_module_lowers_cleanly(
        include_str!("../dag/t_pb_b_1_pipeline_smoke.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_pipeline_smoke.dag",
    );
}

#[test]
fn t_pb_b_1_contract_diagnostic_smoke_dag_lowers_cleanly() {
    assert_dag_module_lowers_cleanly(
        include_str!("../dag/t_pb_b_1_contract_diagnostic_smoke.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_contract_diagnostic_smoke.dag",
    );
}

#[test]
fn t_pb_b_1_contract_port_cost_dag_lowers_cleanly() {
    assert_dag_module_lowers_cleanly(
        include_str!("../dag/t_pb_b_1_contract_port_cost.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_contract_port_cost.dag",
    );
}
