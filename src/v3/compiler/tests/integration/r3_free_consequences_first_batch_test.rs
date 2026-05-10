//! **Layer:** integration
//!
//! R3 T-Free-Consequences first-batch author-now/fire-later claims.
//! Gate `#43` asserts pairwise-independent top-level binds emit a parallel Rust schedule;
//! gate `#44` asserts dependent binds schedule sequentially via the bind-cluster witness;
//! gate `#45` asserts a Bool branch lowers to `if … else` with no `thread::scope` scheduling on
//! the arms. Auto-memoization claims lock the `BinaryDimensionReportEquals` shape.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_free_consequences_first_batch.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_free_consequences_first_batch.dag";
const SUITE_NAME: &str = "r3_free_consequences_first_batch_suite";
const EXPECTED_CLAIMS: [&str; 5] = [
    "auto_parallelism_independent_binds_emit_parallel",
    "auto_parallelism_dependent_binds_emit_sequential",
    "auto_parallelism_branch_arms_serialize",
    "auto_memoization_repeated_pure_call_cached",
    "auto_memoization_no_caching_for_one_shot",
];

#[test]
fn r3_free_consequences_first_batch_reaches_unified_predicate_shape() {
    run_on_larger_stack(|| {
        r3_free_consequences_first_batch_reaches_unified_predicate_shape_inner()
    });
}

fn r3_free_consequences_first_batch_reaches_unified_predicate_shape_inner() {
    let dag = match compile_to_dag(FIXTURE_SOURCE, FIXTURE_PATH) {
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
    };

    let results = TestRunner::new(&dag).run_suite(SUITE_NAME);
    assert_eq!(results.len(), EXPECTED_CLAIMS.len());

    for (result, expected_name) in results.iter().zip(EXPECTED_CLAIMS) {
        assert_eq!(result.claim_name, expected_name);
        match expected_name {
            "auto_parallelism_independent_binds_emit_parallel"
            | "auto_parallelism_dependent_binds_emit_sequential" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} (R3 gates #43/#44) to Pass, got {:?}",
                    result.result
                );
            }
            "auto_parallelism_branch_arms_serialize" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} (R3 gate #45) to Pass, got {:?}",
                    result.result
                );
            }
            _ => {
                assert!(
                    matches!(
                        &result.result,
                        ClaimResult::NotYetImplemented(reason)
                            if reason.contains("BinaryDimensionReportEquals")
                                && reason.contains("structural shape is valid")
                    ),
                    "expected {expected_name} to reach BinaryDimensionReportEquals deferred path, got {:?}",
                    result.result
                );
            }
        }
    }
}
