//! SB-c — Witness→Option pattern bridge for bootstrap map_get (B-LOOKUP-1).
//!
//! `Map.lookup` returns `Witness<V>` (`Holds`/`Violates`), while bootstrap
//! `map_get` (v4.std.collection) still matches legacy `Some`/`None` before
//! wrapping as `Present`/`Absent`. Without the bridge, affected-set
//! `mark_excluded` crashes with PatternMatchFailure on `Holds { value: ... }`.

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

/// Detection: red if Witness Holds/Violates bridging is removed from match_pattern.
#[test]
fn match_pattern_bridges_witness_holds_to_some() {
    let source = include_str!("../../stage0/src/v2_interpreter.rs");
    let match_pattern_start = source
        .find("fn match_pattern(")
        .expect("match_pattern should exist in v2_interpreter.rs");
    let match_pattern_body = &source[match_pattern_start..];
    assert!(
        match_pattern_body.contains(r#"variant_name == "Holds" && name == "Some""#),
        "map_get bootstrap must bridge Witness Holds to Some pattern (B-LOOKUP-1)."
    );
    assert!(
        match_pattern_body.contains(r#"variant_name == "Violates""#),
        "map_get bootstrap must bridge Witness Violates to None pattern (B-LOOKUP-1)."
    );
}

#[test]
fn witness_holds_matches_some_pattern_at_runtime() {
    let src = r#"module test.witness_bridge
type Witness<T>
  = Holds { value: T }
  | Violates { diagnostic: Int }

fn probe_holds() -> Int {
  match Holds { value: 42 } {
    Some { value: v } => v
    None => 0
  }
}

fn probe_violates() -> Int {
  match Violates { diagnostic: 1 } {
    Some { value: v } => v
    None => 99
  }
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(graph, resolved.source_indices.clone(), "probe_holds") {
        Ok(Value::Int(42)) => {}
        other => panic!("expected Int(42) from Holds→Some bridge, got {other:?}"),
    }
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "probe_violates") {
        Ok(Value::Int(99)) => {}
        other => panic!("expected Int(99) from Violates→None bridge, got {other:?}"),
    }
}
