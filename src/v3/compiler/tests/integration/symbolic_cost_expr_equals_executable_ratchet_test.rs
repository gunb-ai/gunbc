//! **Layer:** integration
//!
//! R3 §1.8 gate **#40** `symbolic_cost_expr_equals_executable`
//! (T-CostLens-Composition): behavior-first NYI-shell-retirement ratchet.
//!
//! Asserts via the `TestRunner` interface — **not** by source-text grep —
//! that a well-shaped `TestClaim` with predicate `SymbolicCostExprEquals`
//! produces `ClaimResult::Pass` and **never** `ClaimResult::NotYetImplemented(_)`.
//! The latter is the dispatch-fallthrough shell the gate-#40 predicate
//! ratchets away from (see `docs/r3-structure.md:115`,
//! `docs/r3-program-plan.md` §1.8 row #40). Wider pass / fail-closed
//! receipts (typed-shape Fail paths under `validate_symbolic_cost_ref`,
//! type + value mismatch spellings) live in
//! `m1_5_verification_test.rs::symbolic_cost_expr_equals_fail_closed_*`
//! and the smoke / countdown demo suites — those exercise the dedicated
//! evaluator's typed-shape rejection paths.
//!
//! Robust to refactors: dispatch reshaping, helper renames, and message-
//! text edits do not trip this test — only an actual regression to the
//! `NotYetImplemented` dispatcher fallthrough does.

use v3_compiler::compile_to_dag;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

/// Minimal well-shaped fixture: should evaluate to `Pass`. If the
/// dispatch arm regresses, the runner falls through to the generic
/// `TestPredicate::{other} is not wired in the Rust runner yet`
/// shell and emits `NotYetImplemented` — which this test forbids.
const WELL_SHAPED_FIXTURE: &str = r#"
module std.symbolic_cost_expr_equals_executable_ratchet_well_shaped

import std.verification {
  SymbolicCostExprEquals,
  TestClaim,
  TestSuite
}
import std.algebra { SymbolicCost }

data expected_cost: SymbolicCost = ConstantCost(0)

data well_shaped_claim: TestClaim = {
  name: "symbolic_cost_expr_equals_executable_ratchet_well_shaped_claim",
  source: "let lit: Int = 7",
  file_name: "ratchet_well_shaped.v3",
  predicate: SymbolicCostExprEquals(expected_cost),
  requires: []
}

data well_shaped_suite: TestSuite = {
  name: "symbolic_cost_expr_equals_executable_ratchet_well_shaped_suite",
  claims: [Enumerated(well_shaped_claim)]
}
"#;

fn compile_fixture(src: &str, file: &str) -> v3_compiler::dag::Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "gate #40 ratchet fixture must compile cleanly: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "gate #40 ratchet fixture should not produce semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("gate #40 ratchet fixture compile error: {other:?}"),
    }
}

#[test]
fn symbolic_cost_expr_equals_well_shaped_claim_passes_and_never_returns_not_yet_implemented() {
    let dag = compile_fixture(WELL_SHAPED_FIXTURE, "ratchet_well_shaped.dag");
    let results = TestRunner::new(&dag)
        .run_suite("symbolic_cost_expr_equals_executable_ratchet_well_shaped_suite");
    assert_eq!(results.len(), 1, "exactly one claim in the ratchet suite");
    let result = &results[0].result;
    match result {
        ClaimResult::Pass => {}
        ClaimResult::NotYetImplemented(msg) => panic!(
            "R3 gate #40 (`symbolic_cost_expr_equals_executable`): \
             `SymbolicCostExprEquals` must execute through the runner \
             dispatcher — observed `NotYetImplemented` fallthrough, \
             meaning the dispatch arm has regressed back to the shell \
             this gate ratchets away from. Runner message: {msg}"
        ),
        ClaimResult::Fail(msg) => panic!(
            "R3 gate #40: well-shaped `SymbolicCostExprEquals` ratchet \
             must Pass — observed `Fail`, indicating the dedicated \
             evaluator regressed on the well-shaped path. Runner \
             message: {msg}"
        ),
    }
}
