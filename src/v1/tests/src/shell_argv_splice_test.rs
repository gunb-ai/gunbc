//! Regression: `push_shell_argv_tokens` must splice `List<String>` / FreeMonoid<Str>
//! element-wise (cargo extra_args, clippy lint_args). Collapsing to one token breaks gates.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

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

fn run_list_fn(src: &str, function: &str) -> (Value, InterpContext) {
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    let ctx = InterpContext::new(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let val = match v1_interpreter::run(graph, resolved.source_indices.clone(), function) {
        Ok(v) => v,
        other => panic!("expected value from {function}, got {other:?}"),
    };
    (val, ctx)
}

#[test]
fn shell_argv_splices_freemonoid_str_list_elementwise() {
    let src = r#"module test.shell_argv_splice
fn fmt_extra_args() -> List<String> {
  ["--all", "--check"]
}
"#;
    let (val, ctx) = run_list_fn(src, "fmt_extra_args");
    let argv = v1_interpreter::shell_argv_tokens_for_test(val, &ctx)
        .expect("shell argv materialization");
    assert_eq!(
        argv,
        vec!["--all".to_string(), "--check".to_string()],
        "FreeMonoid<Str> must splice to N argv tokens, not one mashed token"
    );
}

#[test]
fn shell_argv_splices_clippy_lint_args_elementwise() {
    let src = r#"module test.shell_argv_clippy
fn lint_args() -> List<String> {
  ["-D", "warnings"]
}
"#;
    let (val, ctx) = run_list_fn(src, "lint_args");
    let argv = v1_interpreter::shell_argv_tokens_for_test(val, &ctx)
        .expect("shell argv materialization");
    assert_eq!(
        argv,
        vec!["-D".to_string(), "warnings".to_string()],
        "clippy lint_args must not collapse to '-Dwarnings'"
    );
}
