use std::rc::Rc;

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

#[test]
fn repeat_string_and_indent_text_semantics_via_interpreter() {
    let src = r#"module test.repeat_string_regression
import std.render_repeat_string_bootstrap { repeat_string }
fn repeat_string_returns_n_copies() -> String { repeat_string(s: "x", n: 3) }
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

    match v1_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "repeat_string_returns_n_copies",
    ) {
        Ok(Value::Str(s)) => assert_eq!(s, "xxx"),
        other => panic!("expected Str(\"xxx\"), got {other:?}"),
    }

    match v1_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "pads_then_text_like_indent",
    ) {
        Ok(Value::Str(s)) => assert_eq!(s, "  a"),
        other => panic!("expected Str(\"  a\"), got {other:?}"),
    }
}
