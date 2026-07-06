use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
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
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

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
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    match v1_interpreter::run_in_context(&ctx, "build", false) {
        Ok(Value::Int(4)) => {}
        other => panic!("expected Int(4), got {other:?}"),
    }
    let counters = ctx.mutation_counters_snapshot();
    assert_eq!(counters.map_insert_calls, 4);
    assert_eq!(counters.map_insert_entries_copied, 0);
}

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
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    match v1_interpreter::run_in_context(&ctx, "build", false) {
        Ok(Value::Int(5)) => {}
        other => panic!("expected Int(5), got {other:?}"),
    }
    let counters = ctx.mutation_counters_snapshot();
    assert_eq!(counters.list_concat_calls, 2);
    assert_eq!(counters.list_concat_items_copied, 0);
    assert_eq!(counters.list_push_calls, 0, "merge must not leak into push");
}

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
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

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

#[test]
fn retained_accounting_counts_shared_structure_once() {
    let src = r#"module test.stats_shared
data xs: List<Int> = [1, 2, 3]
fn read_xs() -> List<Int> { xs }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    let first = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("first run");
    let second = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("second run");

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
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

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
