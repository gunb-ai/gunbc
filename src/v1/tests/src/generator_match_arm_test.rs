use std::sync::Arc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

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

#[test]
fn record_match_pattern_includes_value_record_arm() {
    let source = include_str!("../../stage0/src/v1_interpreter.rs");
    let match_pattern_start = source
        .find("fn match_pattern(")
        .expect("match_pattern should exist in v1_interpreter.rs");
    let match_pattern_body = &source[match_pattern_start..];
    assert!(
        match_pattern_body.contains("Value::Record { type_name, fields } =>"),
        "Generator/record destructuring must route through match_pattern Value::Record arm \
         (ctrl#1476 B3 Gap-1)."
    );
}

#[test]
fn record_destructuring_match_binds_fields_at_runtime() {
    let src = r#"module test.gen_match_arm
type Pair { left: Int, right: Int }

fn seven() -> Int {
  match Pair { left: 3, right: 4 } {
    Pair { left: a, right: b } => a + b
  }
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "seven") {
        Ok(Value::Int(7)) => {}
        other => panic!("expected Int(7) from record destructuring match, got {other:?}"),
    }
}
