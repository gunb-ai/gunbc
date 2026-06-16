//! Phase-0 measurement harness (ctrl#1533): the interpreter counts the copy
//! work its collection primitives perform, and can produce a sharing-aware
//! byte accounting of what a context retains.
//!
//! These tests pin the measurement semantics, not the performance. Under the
//! phase-2 persistent carriers, native-carrier updates share structure and
//! copy NOTHING — the *_entries_copied/_items_copied counters must read 0
//! (the triangular-number copy term phase 0 measured is gone; what remains
//! countable is the FreeMonoid chain flatten). Shared structure must be
//! accounted exactly once.

use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

fn resolve(src: &str) -> Rc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

/// Building a 4-entry map by successive inserts performs 4 persistent
/// updates and copies ZERO entries — the triangular-number term (0+1+2+3 = 6
/// under the ephemeral carrier) is gone. This is the phase-2 receipt.
#[test]
fn fold_built_map_copies_no_entries() {
    let src = r#"module test.stats_map
fn build() -> Int {
  let m = empty_map() |> map_insert("a", 1) |> map_insert("b", 2) |> map_insert("c", 3) |> map_insert("d", 4)
  m.length()
}
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone());

    match v1_interpreter::run_in_context(&ctx, "build", false) {
        Ok(Value::Int(4)) => {}
        other => panic!("expected Int(4), got {other:?}"),
    }
    let counters = ctx.mutation_counters_snapshot();
    assert_eq!(counters.map_insert_calls, 4);
    assert_eq!(counters.map_insert_entries_copied, 0);
}

/// Concat of native lists is persistent RRB concatenation: both calls are
/// counted as merges, but no items are copied (under the ephemeral carrier
/// [1,2,3]⊕[4] copied 4 and the 4-list⊕[5] copied 5 — triangular growth).
#[test]
fn list_concat_copies_no_native_items() {
    let src = r#"module test.stats_list
fn build() -> Int {
  let a = [1, 2, 3].concat([4])
  let b = a.concat([5])
  b.length()
}
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone());

    match v1_interpreter::run_in_context(&ctx, "build", false) {
        Ok(Value::Int(5)) => {}
        other => panic!("expected Int(5), got {other:?}"),
    }
    let counters = ctx.mutation_counters_snapshot();
    assert_eq!(counters.list_concat_calls, 2);
    assert_eq!(counters.list_concat_items_copied, 0);
    assert_eq!(counters.list_push_calls, 0, "merge must not leak into push");
}

/// Appending an atomic element is a push regardless of dispatch surface, and
/// a persistent push_back copies none of the receiver's elements. Same
/// primitive, one bucket — never split between `list_push` and `list_concat`
/// by method-vs-builtin path.
#[test]
fn atomic_append_counts_as_push_not_concat() {
    let src = r#"module test.stats_push
fn build() -> Int {
  let a = [1, 2, 3].append("x")
  a.length()
}
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone());

    match v1_interpreter::run_in_context(&ctx, "build", false) {
        Ok(Value::Int(4)) => {}
        other => panic!("expected Int(4), got {other:?}"),
    }
    let counters = ctx.mutation_counters_snapshot();
    assert_eq!(counters.list_push_calls, 1);
    assert_eq!(counters.list_push_items_copied, 0);
    assert_eq!(
        counters.list_concat_calls, 0,
        "push must not leak into concat"
    );
}

/// A cached `data` value reached from two roots is one unique allocation plus
/// a sharing hit — never double-counted bytes.
#[test]
fn retained_accounting_counts_shared_structure_once() {
    let src = r#"module test.stats_shared
data xs: List<Int> = [1, 2, 3]
fn read_xs() -> List<Int> { xs }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone());

    let first = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("first run");
    let second = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("second run");

    // Both results and the data cache point at ONE list allocation: the walk
    // must report exactly one unique List and at least two sharing hits.
    let acc = ctx.account_retained_memory(&[&first, &second]);
    let list = acc.per_variant.get("List").expect("List accounted");
    assert_eq!(list.unique_allocations, 1);
    assert!(
        list.shared_references >= 2,
        "expected >=2 sharing hits, got {}",
        list.shared_references
    );
    assert_eq!(
        list.heap_bytes,
        (3 * std::mem::size_of::<Value>()) as u64,
        "3-slot list buffer"
    );
}

/// Symbol interning (#4799) receipt, pinned at the table: a name interned
/// twice yields the SAME `Symbol`, is retained once (`distinct == 1`), and the
/// repeat call is a hit — the `String` allocation a per-occurrence carrier
/// would have made and interning elides. The accounting invariant
/// `calls == distinct + hits` always holds.
#[test]
fn interner_dedups_repeated_name() {
    let mut interner = v1_interpreter::SymbolInterner::default();
    let a = interner.intern("VariantName");
    let b = interner.intern("VariantName");
    let c = interner.intern("OtherName");
    assert_eq!(a, b, "same name interns to the same symbol");
    assert_ne!(a, c, "distinct names get distinct symbols");

    let stats = interner.stats();
    assert_eq!(stats.calls, 3);
    assert_eq!(stats.distinct, 2);
    assert_eq!(stats.hits, 1, "the repeat lookup is an avoided allocation");
    assert_eq!(
        stats.calls,
        stats.distinct + stats.hits,
        "calls = distinct + hits invariant"
    );
}

/// Running a witness interns its identity names (binding keys, every variable
/// reference resolves through `ctx.sym(name)`), and the same names recur
/// across the evaluation, so the context's interner accumulates hits — the
/// live #4799 dedup signal the `claim_batch` `[interp-stats]` report surfaces.
/// Here `x` is read four times: each read interns `"x"`, so at least three of
/// those are hits. The accounting invariant must hold over a real evaluation,
/// not just a hand-built table.
#[test]
fn witness_evaluation_produces_intern_hits() {
    let src = r#"module test.stats_intern
fn build() -> Int {
  let x = 1
  x + x + x + x
}
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone());

    match v1_interpreter::run_in_context(&ctx, "build", false) {
        Ok(Value::Int(4)) => {}
        other => panic!("expected Int(4), got {other:?}"),
    }

    let stats = ctx.interner_stats_snapshot();
    assert!(stats.calls > 0, "evaluation interned identity names");
    assert!(
        stats.hits >= 3,
        "the four reads of `x` dedup to >=3 hits, got {stats:?}"
    );
    assert_eq!(
        stats.calls,
        stats.distinct + stats.hits,
        "calls = distinct + hits invariant over a real evaluation"
    );
}
