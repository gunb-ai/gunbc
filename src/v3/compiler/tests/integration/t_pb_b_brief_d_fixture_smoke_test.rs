//! **Layer:** integration
//!
//! T-PB-B Brief D — compile-only smoke for draft `TestClaim` / `TestSuite` fixtures under
//! `tests/fixtures/t_pb_b_brief_d/`. These files exercise `std.verification` shapes from real
//! v3 source; they are **not** a `pb_*` gate and do not replace existing Rust tests.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

fn assert_lowers_without_diagnostics(source: &'static str, file: &'static str) {
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
fn t_pb_b_brief_d_pipeline_smoke_fixture_lowers_cleanly() {
    assert_lowers_without_diagnostics(
        include_str!("../fixtures/t_pb_b_brief_d/pipeline_smoke.v3"),
        "fixtures/t_pb_b_brief_d/pipeline_smoke.v3",
    );
}

#[test]
fn t_pb_b_brief_d_contract_diagnostic_smoke_fixture_lowers_cleanly() {
    assert_lowers_without_diagnostics(
        include_str!("../fixtures/t_pb_b_brief_d/contract_diagnostic_smoke.v3"),
        "fixtures/t_pb_b_brief_d/contract_diagnostic_smoke.v3",
    );
}

#[test]
fn t_pb_b_brief_d_contract_port_cost_smoke_fixture_lowers_cleanly() {
    assert_lowers_without_diagnostics(
        include_str!("../fixtures/t_pb_b_brief_d/contract_port_cost_smoke.v3"),
        "fixtures/t_pb_b_brief_d/contract_port_cost_smoke.v3",
    );
}
