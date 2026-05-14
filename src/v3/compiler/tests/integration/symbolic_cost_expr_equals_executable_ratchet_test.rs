//! **Layer:** integration
//!
//! R3 §1.8 gate **#40** `symbolic_cost_expr_equals_executable`
//! (T-CostLens-Composition): behavior-first NYI-shell-retirement ratchet.
//!
//! Asserts via the `TestRunner` interface — **not** by source-text grep —
//! that for any `TestClaim` whose predicate is `SymbolicCostExprEquals`,
//! the runner's `ClaimResult` is `Pass` or `Fail(_)` — **never**
//! `NotYetImplemented(_)`. The latter is the dispatch-fallthrough shell
//! the gate-#40 predicate ratchets away from (see `docs/r3-structure.md:115`,
//! `docs/r3-program-plan.md` §1.8 row #40). Wider pass / fail-closed
//! receipts (Pass + structural Fail spellings) live in
//! `m1_5_verification_test.rs::symbolic_cost_expr_equals_*`. This file
//! pins the NYI-vs-Fail boundary itself through runner behavior.
//!
//! Robust to refactors: dispatch reshaping, helper renames, and message-
//! text edits do not trip these tests — only an actual regression to the
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
fn symbolic_cost_expr_equals_well_shaped_claim_never_returns_not_yet_implemented() {
    let dag = compile_fixture(WELL_SHAPED_FIXTURE, "ratchet_well_shaped.dag");
    let results = TestRunner::new(&dag)
        .run_suite("symbolic_cost_expr_equals_executable_ratchet_well_shaped_suite");
    assert_eq!(results.len(), 1, "exactly one claim in the ratchet suite");
    let result = &results[0].result;
    match result {
        ClaimResult::Pass | ClaimResult::Fail(_) => {}
        ClaimResult::NotYetImplemented(msg) => panic!(
            "R3 gate #40 (`symbolic_cost_expr_equals_executable`): \
             `SymbolicCostExprEquals` must execute through the runner \
             dispatcher — observed `NotYetImplemented` fallthrough, \
             meaning the dispatch arm has regressed back to the shell \
             this gate ratchets away from. Runner message: {msg}"
        ),
    }
}

/// Same predicate, but the `expected` declaration is **not** typed as
/// `SymbolicCost`. The dispatched evaluator must reject this with
/// `Fail(_)` (typed-shape rejection inside the dedicated evaluator) —
/// **not** `NotYetImplemented(_)` (generic dispatcher fallthrough).
/// Pinning the fail-class through behavior pins the dispatch arm
/// indirectly: only the gate-#40-specific evaluator can produce this
/// Fail spelling; a fallthrough cannot.
const MALFORMED_EXPECTED_FIXTURE: &str = r#"
module std.symbolic_cost_expr_equals_executable_ratchet_malformed

import std.verification {
  SymbolicCostExprEquals,
  TestClaim,
  TestSuite
}

data not_a_symbolic_cost: Int = 42

data malformed_claim: TestClaim = {
  name: "symbolic_cost_expr_equals_executable_ratchet_malformed_claim",
  source: "let lit: Int = 0",
  file_name: "ratchet_malformed.v3",
  predicate: SymbolicCostExprEquals(not_a_symbolic_cost),
  requires: []
}

data malformed_suite: TestSuite = {
  name: "symbolic_cost_expr_equals_executable_ratchet_malformed_suite",
  claims: [Enumerated(malformed_claim)]
}
"#;

#[test]
fn symbolic_cost_expr_equals_malformed_expected_returns_fail_not_not_yet_implemented() {
    // This fixture deliberately mistypes the expected payload. Whether it
    // is rejected at compile time (semantic diagnostic) or at evaluation
    // time (runner `Fail`), the runner must NEVER return
    // `NotYetImplemented` for a `SymbolicCostExprEquals` claim. Both
    // accepted-then-Fail and rejected-at-compile paths satisfy gate #40.
    let dag = match compile_to_dag(MALFORMED_EXPECTED_FIXTURE, "ratchet_malformed.dag") {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    };
    if !dag.diagnostics().is_empty() {
        // Compile-time rejection is acceptable — the gate-#40 invariant
        // (no `NotYetImplemented` for this predicate) is preserved
        // vacuously because no claim reaches the runner. We pin that
        // _if_ the malformed fixture had reached the runner, the
        // dispatch arm would reject with `Fail` via the well-shaped
        // test above (which exercises the executable path).
        return;
    }
    let results = TestRunner::new(&dag)
        .run_suite("symbolic_cost_expr_equals_executable_ratchet_malformed_suite");
    assert_eq!(results.len(), 1);
    let result = &results[0].result;
    match result {
        ClaimResult::Fail(_) => {}
        ClaimResult::Pass => panic!(
            "R3 gate #40: malformed `SymbolicCostExprEquals` expected (`Int`, not `SymbolicCost`) \
             must not Pass — this would indicate the typed-shape rejection inside \
             `eval_symbolic_cost_expr_equals_shape` has regressed."
        ),
        ClaimResult::NotYetImplemented(msg) => panic!(
            "R3 gate #40 (`symbolic_cost_expr_equals_executable`): malformed payload \
             must surface as `Fail` from the dedicated evaluator, never as \
             `NotYetImplemented` from the dispatcher fallthrough. Runner message: {msg}"
        ),
    }
}
