//! Optional surface dissolution for bootstrap map_get (B-LOOKUP-1).
//!
//! `Map.lookup` returns `Witness<V>` (`Holds`/`Violates`), while bootstrap
//! `map_get` (v4.std.collection) now matches that Witness surface directly
//! before wrapping as canonical `Present`/`Absent`.

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

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

fn run_v4_module(entry: &str, content: &str, witness_fn: &str) -> Value {
    let sources: Vec<Rc<SourceFile>> =
        resolve_imports_transitively_with_source_roots(entry, content, &v4_source_roots());
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    v2_interpreter::run(graph, resolved.source_indices.clone(), witness_fn)
        .unwrap_or_else(|e| panic!("run {witness_fn}: {e:?}"))
}

/// Detection: red if the deleted Witness→Some/None spelling bridge returns.
#[test]
fn match_pattern_does_not_bridge_witness_to_some_none() {
    let source = include_str!("../../stage0/src/v2_interpreter.rs");
    let match_pattern_start = source
        .find("fn match_pattern(")
        .expect("match_pattern should exist in v2_interpreter.rs");
    let match_pattern_body = &source[match_pattern_start..];
    assert!(
        !match_pattern_body.contains(r#"variant_name == "Holds" && name == "Some""#),
        "Witness Holds must not be accepted as legacy Some."
    );
    assert!(
        !match_pattern_body.contains(r#"variant_name == "Violates" && (name == "None""#),
        "Witness Violates must not be accepted as legacy None."
    );
}

#[test]
fn map_get_matches_witness_lookup_to_present_absent() {
    let src = r#"module test.witness_map_get
import v4.std.collection { Absent, Present, empty_map, map_get, map_insert }
import v4.std.diagnostic { Accepted, Rejected }

fn found() -> Bool {
  let m = empty_map() |> map_insert("k", 42)
  match map_get(m, "k") {
    Accepted { value: Present { value: v }, diagnostics: _ } => v == 42
    Accepted { value: Absent, diagnostics: _ } => false
    Rejected { diagnostics: _ } => false
  }
}

fn missing() -> Bool {
  let m = empty_map()
  match map_get(m, "k") {
    Accepted { value: Present { value: _ }, diagnostics: _ } => false
    Accepted { value: Absent, diagnostics: _ } => true
    Rejected { diagnostics: _ } => false
  }
}
"#;
    match run_v4_module("test/witness_map_get.dag", src, "found") {
        Value::Bool(true) => {}
        other => panic!("expected Bool(true) from map_get on Witness Holds, got {other:?}"),
    }
    match run_v4_module("test/witness_map_get.dag", src, "missing") {
        Value::Bool(true) => {}
        other => panic!("expected Bool(true) from map_get on Witness Violates, got {other:?}"),
    }
}

#[test]
fn mark_excluded_no_longer_pattern_match_fails() {
    let entry = "src/v4/test/claim/lens_affected_set/excluded_propagation_proof.dag";
    let content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read {entry}: {e}"));
    // Returns Bool (true or false) — the pre-fix crash was PatternMatchFailure on Holds.
    match run_v4_module(entry, &content, "excluded_propagation_proof_claim_holds") {
        Value::Bool(_) => {}
        other => panic!("expected Bool witness from mark_excluded path, not crash; got {other:?}"),
    }
}
