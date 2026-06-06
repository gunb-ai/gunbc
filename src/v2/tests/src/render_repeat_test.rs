//! Regression: dsl/std/render.dag repeat_string must repeat n times (P0-A).

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

#[test]
fn repeat_string_and_indent_text_semantics_via_interpreter() {
    let src = r#"module test.repeat_string_regression
import std.render { repeat_string }
fn repeat_string_returns_n_copies() -> String { repeat_string(s: "x", n: 3) }
// Same shape as indent_text's pad + text, without string interpolation (interpreter
// does not expand `"{pad}{text}"` templates yet).
fn pads_then_text_like_indent() -> String {
  let pad = repeat_string(s: " ", n: 2)
  concat(pad, "a")
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "repeat_string_returns_n_copies",
    ) {
        Ok(Value::Str(s)) => assert_eq!(s, "xxx"),
        other => panic!("expected Str(\"xxx\"), got {other:?}"),
    }

    match v2_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "pads_then_text_like_indent",
    ) {
        Ok(Value::Str(s)) => assert_eq!(s, "  a"),
        other => panic!("expected Str(\"  a\"), got {other:?}"),
    }
}
