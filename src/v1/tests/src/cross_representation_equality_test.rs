//! Numeric-tower grounding: native form == modeled form (DESIGN §1/§2/§7,
//! model↔realization fork). Discriminating witness for the grounding that
//! dissolves the cross-representation `==` straddle.
//!
//! `Nat` is **modeled** as the coproduct `Zero | Succ { prev }` (`v2.std.nat`)
//! and **realized** as a native `Value::Int`. Before grounding, the read bridge
//! mapped `Int → Zero/Succ` (so `match` worked) but *construction* of
//! `Succ { prev }` built a `Variant`, so `nat_add(a, b)` — which folds via
//! `nat_cata(n: a, zero: b, ..)` — produced the hybrid `Succ^a(Int(b))`. Comparing
//! that to a native `Int` funnelled through `Value::eq`'s `_ => false` arm and
//! silently disagreed: a §5 fail-open the operator first closed with a typed
//! `CrossRepresentationEquality` guard (2026-06-20), then dissolved at the root.
//!
//! The root fix grounds the construction side too — `Zero` is realized as
//! `Int(0)` and `Succ { prev: Int(k) }` as `Int(k + 1)` — so a `Nat` value is
//! never a coproduct `Variant` and the native form *is* the modeled form. The
//! former forks now **reconcile** to a plain `Bool`: this is the discriminating
//! witness for that grounding. If construction regresses to building a `Variant`,
//! every reconciling case goes RED (a typed straddle error from the still-armed
//! backstop guard, or — were that guard also removed — a silent wrong bool),
//! while the genuine-difference cases must keep returning plain bools.

use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

/// Receipts authored as individual witness functions so each can be run and
/// classified independently.
const RECEIPTS_SOURCE: &str = r#"
module test.xrepr

import v2.std.logic { Bool }
import v2.std.nat { Nat, Succ, Zero, nat_add }
import v2.std.algebra { Cons, Empty }

// --- grounded reconciliation: native form == modeled form, plain Bool(true) ---
// (these were the §5 fail-open forks before the numeric tower was grounded)
fn reconciles_nat_add_eq_int() -> Bool { nat_add(a: 85, b: 32) == 117 }
fn reconciles_nat_add_1_1_eq_2() -> Bool { nat_add(a: 1, b: 1) == 2 }
fn reconciles_nat_add_noncanon() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 100, b: 17) }
fn reconciles_list_element() -> Bool { [nat_add(a: 1, b: 1)] == [2] }
fn reconciles_succ_zero_eq_one() -> Bool { (Succ { prev: Zero }) == 1 }
fn reconciles_zero_eq_int() -> Bool { Zero == 0 }
fn reconciles_ne_path_false() -> Bool { nat_add(a: 1, b: 1) != 2 }

// --- other reconciled / native controls, must stay Bool(true) ---
fn ok_native_int_eq() -> Bool { (85 + 32) == 117 }
fn ok_nat_add_reflexive() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 85, b: 32) }
fn ok_list_freemonoid_reconciled() -> Bool {
  [1, 2, 3] == Cons { head: 1, tail: Cons { head: 2, tail: Cons { head: 3, tail: Empty } } }
}

// --- genuine differences: must stay Bool(false), NOT error (no false positives) ---
fn diff_int() -> Bool { 1 == 2 }
fn diff_succ_zero_vs_zero() -> Bool { (Succ { prev: Zero }) == Zero }
fn diff_nat_add() -> Bool { nat_add(a: 1, b: 1) == 3 }
fn diff_list_elements() -> Bool { [1, 2] == [1, 3] }
"#;

fn assert_resolved(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "receipts source should resolve cleanly, got {:?} (graph present: {})",
        msgs,
        resolved.graph.is_some(),
    );
}

/// Resolve the receipts once and run `body` against the eval context, keeping the
/// resolved graph alive for the duration of the call (the context borrows it).
fn with_receipts_ctx<R>(body: impl FnOnce(&v1_interpreter::InterpContext) -> R) -> R {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dsl")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", RECEIPTS_SOURCE, &roots);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    body(&ctx)
}

#[test]
fn grounded_numeric_tower_reconciles() {
    with_receipts_ctx(|ctx| {
        for f in [
            "reconciles_nat_add_eq_int",
            "reconciles_nat_add_1_1_eq_2",
            "reconciles_nat_add_noncanon",
            "reconciles_list_element",
            "reconciles_succ_zero_eq_one",
            "reconciles_zero_eq_int",
            "ok_native_int_eq",
            "ok_nat_add_reflexive",
            "ok_list_freemonoid_reconciled",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(true)) => {}
                other => panic!(
                    "{f}: expected Bool(true) — grounding makes the native form equal the \
                     modeled form, so this reconciles; a straddle error or wrong bool here is \
                     the regression this witness guards; got {other:?}"
                ),
            }
        }
        // `!=` over a reconciled pair: equal, so `!=` is false.
        match v1_interpreter::run_in_context(ctx, "reconciles_ne_path_false", false) {
            Ok(Value::Bool(false)) => {}
            other => panic!("reconciles_ne_path_false: expected Bool(false), got {other:?}"),
        }
    });
}

#[test]
fn genuine_inequalities_stay_false_not_errors() {
    with_receipts_ctx(|ctx| {
        for f in [
            "diff_int",
            "diff_succ_zero_vs_zero",
            "diff_nat_add",
            "diff_list_elements",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(false)) => {}
                other => panic!(
                    "{f}: expected Bool(false) — grounding reconciles representations without \
                     turning genuine differences into errors; got {other:?}"
                ),
            }
        }
    });
}
