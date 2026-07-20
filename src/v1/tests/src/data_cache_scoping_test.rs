use std::sync::Arc;

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

fn resolve(src: &str) -> Arc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

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

#[test]
fn data_value_is_shared_across_runs_in_one_context() {
    let src = r#"module test.cache_share
data xs: List<Int> = [1, 2, 3]
fn read_xs() -> List<Int> { xs }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    let first = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("first run");
    let second = v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("second run");
    match (&first, &second) {
        (Value::List(a), Value::List(b)) => {
            assert!(
                Arc::ptr_eq(a, b),
                "data value must be cached and shared within one context"
            );
        }
        other => panic!("expected two Lists, got {other:?}"),
    }
}

#[test]
fn fresh_context_reevaluates_data_independently() {
    let src = r#"module test.cache_fresh
data xs: List<Int> = [7]
fn read_xs() -> List<Int> { xs }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");

    let first = {
        let ctx =
            cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
        v1_interpreter::run_in_context(&ctx, "read_xs", false).expect("first context")
    };
    let ctx2 =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let second = v1_interpreter::run_in_context(&ctx2, "read_xs", false).expect("second context");
    match (&first, &second) {
        (Value::List(a), Value::List(b)) => {
            assert_eq!(a.iter().cloned().collect::<Vec<_>>(), vec![Value::Int(7)]);
            assert_eq!(b.iter().cloned().collect::<Vec<_>>(), vec![Value::Int(7)]);
        }
        other => panic!("expected two Lists, got {other:?}"),
    }
}
