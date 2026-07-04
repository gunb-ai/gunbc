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

#[test]
fn recursive_call_env_chain_bounded_by_lexical_nesting() {
    let src = r#"module test.lexical_base_env
fn countdown(n: Int) -> Int {
  if n <= 0 { 0 }
  else { countdown(n: n - 1) }
}
fn run() -> Int { countdown(n: 200) }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    match v1_interpreter::run_in_context(&ctx, "run", false) {
        Ok(Value::Int(0)) => {}
        other => panic!("expected Int(0), got {other:?}"),
    }
    let peak = v1_interpreter::call_env_depth_peak_snapshot();
    assert!(
        peak <= 3,
        "recursive calls must extend a fixed lexical base (peak chain depth {peak}, want <= 3)"
    );
}

#[test]
fn default_param_eval_still_reads_caller_lexical_scope() {
    let src = r#"module test.lexical_default
fn pick(x: Int = offset) -> Int { x }
fn run() -> Int {
  let offset = 7
  pick()
}
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    match v1_interpreter::run_in_context(&ctx, "run", false) {
        Ok(Value::Int(7)) => {}
        other => panic!("expected Int(7), got {other:?}"),
    }
}

#[test]
fn closure_capture_unaffected_by_lexical_base_env() {
    let src = r#"module test.lexical_closure
fn run() -> Int {
  let base = 10
  let add = (n: Int) -> base + n
  add(5)
}
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    match v1_interpreter::run_in_context(&ctx, "run", false) {
        Ok(Value::Int(15)) => {}
        other => panic!("expected Int(15), got {other:?}"),
    }
}

#[test]
fn recursive_global_data_reads_stay_correct() {
    let src = r#"module test.lexical_data
data step: Int = 1
fn countdown(n: Int) -> Int {
  if n <= 0 { 0 }
  else { countdown(n: n - step) }
}
fn run() -> Int { countdown(n: 5) }
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    match v1_interpreter::run_in_context(&ctx, "run", true) {
        Ok(Value::Int(0)) => {}
        other => panic!("expected Int(0), got {other:?}"),
    }
    let peak = v1_interpreter::call_env_depth_peak_snapshot();
    assert!(
        peak <= 3,
        "eager data env + recursion must not grow call chain with depth (peak {peak})"
    );
}
