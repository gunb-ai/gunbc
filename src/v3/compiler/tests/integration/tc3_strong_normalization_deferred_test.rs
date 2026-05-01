//! **Layer:** integration
//!
//! TC3 strong-normalization stage-(a) fixture — `BinaryDimensionReportEquals` consumer
//! for `tc3_strong_normalization_deferred.dag` (R3 Verification); runner evaluation NYI
//! until `DimensionReport<C>` production lands. Full theorem witnessing awaits T-FixedPoint.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE_SOURCE: &str = include_str!("../fixtures/tc3_strong_normalization_deferred.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/tc3_strong_normalization_deferred.dag";
const SUITE_NAME: &str = "tc3_strong_normalization_suite";

#[test]
fn tc3_strong_normalization_suite_shape_valid_nyi_at_head() {
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
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].claim_name,
        "tc3_strong_normalization_substrate_introduced"
    );
    assert!(
        matches!(
            &results[0].result,
            ClaimResult::NotYetImplemented(reason)
                if reason.contains("BinaryDimensionReportEquals")
                    && reason.contains("structural shape is valid")
        ),
        "expected TC3 unified predicate to pass shape validation with NYI evaluation, got {:?}",
        results[0].result
    );
}
