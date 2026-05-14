//! **Layer:** integration
//!
//! R3 §1.8 gate **#40** `symbolic_cost_expr_equals_executable`
//! (T-CostLens-Composition): the `SymbolicCostExprEquals` predicate must
//! be wired in the Rust `TestRunner` dispatcher as an executable arm —
//! i.e. it must NOT fall through to the generic
//! `TestPredicate::{other} is not wired in the Rust runner yet`
//! `NotYetImplemented` shell.
//!
//! Wider end-to-end pass/fail-closed receipts for the predicate live in
//! `m1_5_verification_test.rs` (`symbolic_cost_expr_equals_smoke_suite_passes`,
//! `symbolic_cost_expr_equals_countdown_demo_suite_passes`, and the
//! `symbolic_cost_expr_equals_fail_closed_*` cases). This file pins the
//! NYI-shell-retirement invariant mechanically as a fail-closed source
//! ratchet so accidental dispatch-arm retirement (which would silently
//! re-introduce the `NotYetImplemented` shell that gate #40 ratchets away
//! from) trips at test time rather than in downstream consumers.

use std::fs;
use std::path::PathBuf;

fn test_runner_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("src/v3/compiler/src/test_runner.rs");
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e} — gate #40 substrate drift?", path.display()))
}

#[test]
fn symbolic_cost_expr_equals_dispatch_arm_is_wired_in_test_runner() {
    let source = test_runner_source();
    assert!(
        source.contains("\"SymbolicCostExprEquals\" =>"),
        "R3 gate #40 (`symbolic_cost_expr_equals_executable`): \
         `SymbolicCostExprEquals` must have an explicit dispatch arm in \
         `src/v3/compiler/src/test_runner.rs`. If this fires, the executable \
         wiring landed for gate #40 has regressed back to the \
         `NotYetImplemented` shell."
    );
    assert!(
        source.contains("fn eval_symbolic_cost_expr_equals_shape"),
        "R3 gate #40: `eval_symbolic_cost_expr_equals_shape` evaluator must \
         exist in `test_runner.rs` — the dispatch arm is meaningless without \
         the executable evaluator."
    );
}

#[test]
fn symbolic_cost_expr_equals_evaluator_does_not_return_not_yet_implemented_shell() {
    let source = test_runner_source();

    let start = source
        .find("fn eval_symbolic_cost_expr_equals_shape")
        .expect("eval_symbolic_cost_expr_equals_shape must exist (see sibling test)");
    // Bound the scan at the next top-level `fn ` (post-newline) so we only
    // inspect the body of this evaluator + its tightly coupled helpers used
    // by the shape arm, not unrelated downstream evaluators.
    let tail = &source[start..];
    let end_offset = tail[1..]
        .find("\n    fn eval_symbolic_cost_expr_equals_for_bind_param")
        .map(|o| o + 1)
        .unwrap_or(tail.len());
    let body = &tail[..end_offset];

    let forbidden = "ClaimResult::NotYetImplemented(format!(\n            \"TestPredicate::";
    assert!(
        !body.contains(forbidden),
        "R3 gate #40: `eval_symbolic_cost_expr_equals_shape` must not fall \
         through to the generic `TestPredicate::{{other}} is not wired in \
         the Rust runner yet` `NotYetImplemented` shell. Found forbidden \
         spelling inside the evaluator body."
    );
}
