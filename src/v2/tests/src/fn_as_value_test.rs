//! Regression: module-level fn items resolve as first-class callable values (gap-a).

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
fn fn_item_bound_and_called_via_first_class_reference() {
    let src = r#"module test.fn_as_value
fn add(a: Int, b: Int) -> Int { a + b }
fn use_via_binding() -> Int {
  let f = add
  f(a: 2, b: 3)
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
        "use_via_binding",
    ) {
        Ok(Value::Int(5)) => {}
        other => panic!("expected Int(5), got {other:?}"),
    }
}
