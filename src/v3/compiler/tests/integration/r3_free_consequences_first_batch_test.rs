//! **Layer:** integration
//!
//! R3 T-Free-Consequences first-batch author-now/fire-later claims.
//! Gate `#43` asserts pairwise-independent top-level binds emit a parallel Rust
//! schedule; gate `#44` stays fail-closed on the scalar parallelism placeholder;
//! gate `#45` asserts a Bool branch lowers to `if … else` with no `thread::scope`
//! scheduling on the arms; gate `#50` asserts one-shot pure calls do not emit
//! memo/cache scaffolding. Repeated-call memoization remains deferred.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

use crate::common::run_on_larger_stack;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/r3_free_consequences_first_batch.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_free_consequences_first_batch.dag";
const SUITE_NAME: &str = "r3_free_consequences_first_batch_suite";
const EXPECTED_CLAIMS: [&str; 5] = [
    "auto_parallelism_independent_binds_emit_parallel",
    "auto_parallelism_dependent_binds_pending_lens_fail_closed",
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
            "auto_parallelism_independent_binds_emit_parallel" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} to Pass (parallel Rust emission witness), got {:?}",
                    result.result
                );
            }
            "auto_parallelism_dependent_binds_pending_lens_fail_closed" => {
                assert!(
                    matches!(&result.result, ClaimResult::Fail(_)),
                    "expected {expected_name} to Fail (fail-closed pending parallelism lens), got {:?}",
                    result.result
                );
            }
            "auto_parallelism_branch_arms_serialize" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} to Pass (Bool branch lowers to `if … else` with no `thread::scope`), got {:?}",
                    result.result
                );
            }
            "auto_memoization_repeated_pure_call_cached" => {
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
            "auto_memoization_no_caching_for_one_shot" => {
                assert!(
                    matches!(&result.result, ClaimResult::Pass),
                    "expected {expected_name} to Pass (one-shot memoization absence witness), got {:?}",
                    result.result
                );
            }
            _ => panic!("unexpected claim name: {expected_name}"),
        }
    }
}
