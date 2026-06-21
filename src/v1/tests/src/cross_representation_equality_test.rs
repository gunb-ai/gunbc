//! Fail-closed cross-representation `==` (DESIGN §5; operator: `==` fail-closed,
//! 2026-06-20).
//!
//! Every v2 primitive is modeled as a coproduct and realized as a native
//! `Value`; where a bridge is missing the two representations silently disagree.
//! `nat_add(a, b)` folds via `nat_cata(n: a, zero: b, ..)`, so it builds the
//! non-canonical hybrid `Succ^a(Int(b))` — a `Variant` chain with a native `Int`
//! leaf. Comparing that to a native `Int` funnels through `Value::eq`'s
//! `_ => false` arm and silently returns `false` — a §5 fail-open
//! (`nat_add(85, 32) == 117` → `false`).
//!
//! The fix makes such a comparison a typed, located error at the `eval_binop`
//! `BinOp::Eq`/`Ne` seam (leaving `Value::eq` infallible for `CanonKey`). This
//! is the discriminating witness for that safety property: the forks below go
//! RED (a silent `false`/`true`) the instant the guard regresses, while genuine
//! inequalities and reconciled forks must keep returning a plain bool.

use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpError, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

/// Receipts from the operator's diagnosis (2026-06-20), authored as individual
/// witness functions so each can be run and classified independently.
const RECEIPTS_SOURCE: &str = r#"
module test.xrepr

import v2.std.logic { Bool }
import v2.std.nat { Nat, Succ, Zero, nat_add }
import v2.std.algebra { Cons, Empty }

// --- forks: must FAIL CLOSED (typed error), never a silent bool ---
fn fork_nat_add_eq_int() -> Bool { nat_add(a: 85, b: 32) == 117 }
fn fork_nat_add_1_1_eq_2() -> Bool { nat_add(a: 1, b: 1) == 2 }
fn fork_nat_add_noncanon() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 100, b: 17) }
fn fork_list_element() -> Bool { [nat_add(a: 1, b: 1)] == [2] }
fn fork_ne_path() -> Bool { nat_add(a: 1, b: 1) != 2 }

// --- controls: reconciled / native, must stay Bool(true) ---
fn ok_native_int_eq() -> Bool { (85 + 32) == 117 }
fn ok_nat_add_reflexive() -> Bool { nat_add(a: 85, b: 32) == nat_add(a: 85, b: 32) }
fn ok_list_freemonoid_reconciled() -> Bool {
  [1, 2, 3] == Cons { head: 1, tail: Cons { head: 2, tail: Cons { head: 3, tail: Empty } } }
}

// --- genuine differences: must stay Bool(false), NOT error (no false positives) ---
fn diff_int() -> Bool { 1 == 2 }
fn diff_variant_name() -> Bool { (Succ { prev: Zero }) == Zero }
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
fn cross_representation_forks_fail_closed() {
    with_receipts_ctx(|ctx| {
        for f in [
            "fork_nat_add_eq_int",
            "fork_nat_add_1_1_eq_2",
            "fork_nat_add_noncanon",
            "fork_list_element",
            "fork_ne_path",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Err(InterpError::CrossRepresentationEquality { .. }) => {}
                other => panic!(
                    "{f}: expected Err(CrossRepresentationEquality) — a silent bool here is the \
                     §5 fail-open this guard closes; got {other:?}"
                ),
            }
        }
    });
}

#[test]
fn reconciled_and_native_equality_still_true() {
    with_receipts_ctx(|ctx| {
        for f in [
            "ok_native_int_eq",
            "ok_nat_add_reflexive",
            "ok_list_freemonoid_reconciled",
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(true)) => {}
                other => panic!("{f}: expected Bool(true), got {other:?}"),
            }
        }
    });
}

#[test]
fn genuine_inequalities_stay_false_not_errors() {
    with_receipts_ctx(|ctx| {
        for f in ["diff_int", "diff_variant_name", "diff_list_elements"] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Bool(false)) => {}
                other => panic!(
                    "{f}: expected Bool(false) — the guard must flag only representation \
                     straddles, not genuine differences; got {other:?}"
                ),
            }
        }
    });
}
