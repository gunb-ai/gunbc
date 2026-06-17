//! Regression: the interpreter's `data`-item cache is scoped to its
//! `InterpContext` (graph-evaluation lifetime), not the process.
//!
//! The cache was previously a `thread_local!` that grew monotonically for the
//! life of the process, with a `keepalive_fns` pin defending against node
//! addresses being freed and reused by a later graph. Context scoping makes
//! both properties structural: entries cannot outlive their graph's
//! evaluation, and within one context a `data` item referenced by many runs
//! still resolves to ONE shared value.

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

/// Two graphs evaluated sequentially on one thread, each declaring a
/// same-named `data` item with a different value. Each run must see its own
/// graph's value — a process-scoped cache keyed by reused node addresses
/// could alias the first graph's entry into the second.
#[test]
fn data_cache_does_not_leak_across_graphs_on_one_thread() {
    let src_a = r#"module test.cache_scope_a
data magic: Int = 41
fn read_magic() -> Int { magic }
"#;
    let src_b = r#"module test.cache_scope_b
data magic: Int = 42
fn read_magic() -> Int { magic }
"#;
    for (src, expected) in [(src_a, 41), (src_b, 42)] {
        let resolved = resolve(src);
        let graph = resolved.graph.as_ref().expect("graph");
        // Lazy data-env (claim-run convention) so the lookup goes through the
        // data cache, not the eager initial env.
        match v1_interpreter::run_with_options(
            graph,
            resolved.source_indices.clone(),
            "read_magic",
            ExecutionMode::Wet,
            false,
        ) {
            Ok(Value::Int(n)) => assert_eq!(n, expected),
            other => panic!("expected Int({expected}), got {other:?}"),
        }
    }
}

/// Within ONE context, a `data` item read by two separate runs resolves to the
/// SAME shared value (Rc identity), not a rebuild per run — the structural
/// sharing `claim_batch` relies on across witnesses.
#[test]
fn data_value_is_shared_across_runs_in_one_context() {
    let src = r#"module test.cache_share
data xs: List<Int> = [1, 2, 3]
fn read_xs() -> List<Int> { xs }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    let first = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("first run");
    let second = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("second run");
    match (&first, &second) {
        (Value::List(a), Value::List(b)) => {
            assert!(
                Rc::ptr_eq(a, b),
                "data value must be cached and shared within one context"
            );
        }
        other => panic!("expected two Lists, got {other:?}"),
    }
}

/// Two contexts over the same graph each evaluate independently and correctly
/// (a fresh context starts with an empty cache; dropping the first releases
/// its entries).
#[test]
fn fresh_context_reevaluates_data_independently() {
    let src = r#"module test.cache_fresh
data xs: List<Int> = [7]
fn read_xs() -> List<Int> { xs }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");

    let first = {
        let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
        v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("first context")
        // ctx drops here, releasing its cache.
    };
    let ctx2 = cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let second = v1_interpreter::run_in_context(&ctx2, "read_xs", false).expect("second context");
    match (&first, &second) {
        (Value::List(a), Value::List(b)) => {
            assert_eq!(a.iter().cloned().collect::<Vec<_>>(), vec![Value::Int(7)]);
            assert_eq!(b.iter().cloned().collect::<Vec<_>>(), vec![Value::Int(7)]);
        }
        other => panic!("expected two Lists, got {other:?}"),
    }
}
