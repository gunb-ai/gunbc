//! **Layer:** integration
//!
//! R3 T-Free-Consequences first-batch author-now/fire-later claims. The
//! auto-parallelism claims exercise the ordinary lens-data path and stay
//! fail-closed because parallelism is not a Dimension instance; auto-memoization
//! locks the cost-related `BinaryDimensionReportEquals` shape.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

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

    for (idx, (result, expected_name)) in results.iter().zip(EXPECTED_CLAIMS).enumerate() {
        assert_eq!(result.claim_name, expected_name);
        if idx < 3 {
            assert!(
                matches!(
                    &result.result,
                    ClaimResult::Fail(reason)
                        if reason.contains("expected 1")
                            && reason.contains("computed 0")
                            && reason.contains("auto_parallelism_pending_lens")
                ),
                "expected {expected_name} to fail closed on the pending ordinary parallelism lens, got {:?}",
                result.result
            );
        } else {
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

fn run_on_larger_stack<T>(f: impl FnOnce() -> T + Send + 'static) -> T
where
    T: Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(f)
        .expect("spawn larger-stack integration thread")
        .join()
        .expect("larger-stack integration thread panicked")
}
