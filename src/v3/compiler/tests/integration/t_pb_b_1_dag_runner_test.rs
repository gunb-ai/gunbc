//! **Layer:** integration
//!
//! T-PB-B-1 runner wiring — proves `TestRunner` evaluates the landed
//! `src/v3/compiler/tests/dag/*.dag` `TestSuite` modules through the same
//! `compile_to_dag` entrypoint as the rest of the integration harness. This is
//! line-item (1) of the pre–Rust-deletion checklist in
//! `docs/briefs/r1-testgen-manager.md` (Hand-off → Self-hosting): runner path
//! accepts the landed layout and `requires: []` lowers to a shape the runner
//! consumes. Still **not** a `pb_*` gate and still not a Rust-deletion signal.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

fn lower(source: &'static str, file: &'static str) -> Dag {
    match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected compile error for {file}: {other:?}"),
    }
}

fn run_suite_all_pass(dag: &Dag, suite_name: &str) {
    let results = TestRunner::new(dag).run_suite(suite_name);
    assert!(
        !results.is_empty(),
        "suite `{suite_name}` should contain at least one claim"
    );
    assert!(
        results
            .iter()
            .all(|result| result.result == ClaimResult::Pass),
        "suite `{suite_name}` should pass every claim, got {results:?}"
    );
}

#[test]
fn t_pb_b_1_pipeline_smoke_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_pb_b_1_pipeline_smoke.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_pipeline_smoke.dag",
    );
    run_suite_all_pass(&dag, "suite_pipeline_pipe_unary");
}

#[test]
fn t_pb_b_1_contract_diagnostic_smoke_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_pb_b_1_contract_diagnostic_smoke.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_contract_diagnostic_smoke.dag",
    );
    run_suite_all_pass(&dag, "suite_contract_diagnostic_negatives");
}

#[test]
fn t_pb_b_1_contract_port_cost_suite_passes_through_runner() {
    let dag = lower(
        include_str!("../dag/t_pb_b_1_contract_port_cost.dag"),
        "src/v3/compiler/tests/dag/t_pb_b_1_contract_port_cost.dag",
    );
    run_suite_all_pass(&dag, "suite_contract_port_and_cost");
}
