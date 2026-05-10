//! **Layer:** integration
//!
//! R3 T-Free-Consequences second-batch author-now/fire-later claims. Gate `#47`
//! asserts sequential fallback via `LensOutputEquals` + `emit_rust` witness
//! (`r3_auto_loop_parallelism_sequential_emit_witness`); the other auto-loop
//! claims stay fail-closed on the scalar placeholder lens; the cross-target
//! optimization claims lock the cost-related `BinaryDimensionReportEquals` shape.

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_free_consequences_second_batch.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_free_consequences_second_batch.dag";
const SUITE_NAME: &str = "r3_free_consequences_second_batch_suite";
/// Suite order is still checked against `TestRunner` output; expected Pass/Fail/NYI is keyed by
/// name so reordering `claims` in the `.dag` cannot accidentally swap pass vs fail without a
/// compile-time name mismatch on the `assert_eq!(result.claim_name, expected_name)` line.
const EXPECTED_CLAIMS: [&str; 5] = [
    "auto_loop_parallelism_provable_independence_emits_parallel",
    "auto_loop_parallelism_unproven_falls_back_sequential",
    "auto_loop_parallelism_dependence_emits_sequential",
    "cross_target_optimization_constant_fold_consistent",
    "cross_target_optimization_cost_structurally_derived",
];

static SECOND_BATCH_DAG: OnceLock<Dag> = OnceLock::new();

fn second_batch_dag() -> &'static Dag {
    SECOND_BATCH_DAG.get_or_init(|| match compile_to_dag(FIXTURE_SOURCE, FIXTURE_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{FIXTURE_PATH} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {FIXTURE_PATH}: {other:?}"),
    })
}

#[test]
fn r3_free_consequences_second_batch_reaches_expected_consumer_shapes() {
    run_on_larger_stack(|| {
        r3_free_consequences_second_batch_reaches_expected_consumer_shapes_inner()
    });
}

fn r3_free_consequences_second_batch_reaches_expected_consumer_shapes_inner() {
    let results = TestRunner::new(second_batch_dag()).run_suite(SUITE_NAME);
    assert_eq!(results.len(), EXPECTED_CLAIMS.len());

    for (result, expected_name) in results.iter().zip(EXPECTED_CLAIMS) {
        assert_eq!(result.claim_name, expected_name);
        match expected_name {
            "auto_loop_parallelism_unproven_falls_back_sequential" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} to Pass (sequential emit witness — no thread::scope), got {:?}",
                    result.result
                );
            }
            "auto_loop_parallelism_provable_independence_emits_parallel"
            | "auto_loop_parallelism_dependence_emits_sequential" => {
                assert!(
                    matches!(&result.result, ClaimResult::Fail(_)),
                    "expected {expected_name} to fail closed on the pending ordinary loop-parallelism lens, got {:?}",
                    result.result
                );
            }
            "cross_target_optimization_constant_fold_consistent"
            | "cross_target_optimization_cost_structurally_derived" => {
                assert!(
                    matches!(&result.result, ClaimResult::NotYetImplemented(_)),
                    "expected {expected_name} to stay author-now/fire-later on BinaryDimensionReportEquals, got {:?}",
                    result.result
                );
            }
            _ => panic!("unexpected claim in second-batch suite: {expected_name}"),
        }
    }
}
