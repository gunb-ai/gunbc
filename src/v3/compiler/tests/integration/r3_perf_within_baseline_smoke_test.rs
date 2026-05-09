//! `PerfWithinBaseline` substrate-shape smoke: `PerfBaselineMeasurement` uses
//! `p99_delta_ns`; comparator is `PerfBudgetComparisonOp::AtMostBudget` per
//! `src/v3/std/verification.dag` (PM dispatch stern-ram-58 / PR #2367).

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const FIXTURE: &str = include_str!("../fixtures/r3_perf_within_baseline_smoke.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_perf_within_baseline_smoke.dag";

#[test]
fn perf_within_baseline_smoke_suite_passes() {
    let dag = match compile_to_dag(FIXTURE, FIXTURE_PATH) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{FIXTURE_PATH}: semantic compile error: {:?}",
            dag.diagnostics()
        ),
        Err(e) => panic!("{FIXTURE_PATH}: unexpected error: {e:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "{FIXTURE_PATH}: expected no diagnostics, got {:?}",
        dag.diagnostics()
    );
    let results = TestRunner::new(&dag).run_suite("perf_within_smoke_suite");
    assert_eq!(results.len(), 1, "expected one claim");
    assert_eq!(
        results[0].result,
        ClaimResult::Pass,
        "claim {:?}",
        results[0].claim_name
    );
}
