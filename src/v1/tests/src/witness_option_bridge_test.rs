use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_compiler_emit_rust::emit_variant_pattern;
use v1_compiler::v1_compiler_infer_emit_info::empty_emit_graph_info;
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

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

fn run_v4_module(entry: &str, content: &str, witness_fn: &str) -> Value {
    let sources: Vec<Rc<SourceFile>> =
        resolve_imports_transitively_with_source_roots(entry, content, &v2_source_roots());
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    v1_interpreter::run(graph, resolved.source_indices.clone(), witness_fn)
        .unwrap_or_else(|e| panic!("run {witness_fn}: {e:?}"))
}

#[test]
fn match_pattern_does_not_bridge_witness_to_some_none() {
    let source = include_str!("../../stage0/src/v1_interpreter.rs");
    let match_pattern_start = source
        .find("fn match_pattern(")
        .expect("match_pattern should exist in v1_interpreter.rs");
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
fn rust_emitter_lowers_present_absent_only_for_optional_parent() {
    let info = empty_emit_graph_info();
    let empty_bindings = Rc::new(vec![]);
    let empty_path = Rc::new(vec![]);
    let empty_shared = Rc::new(std::collections::BTreeSet::new());
    let empty_indices = Rc::new(std::collections::HashMap::new());

    let non_optional = emit_variant_pattern(
        "Absent".to_string(),
        Some("NonOptional".to_string()),
        empty_bindings.clone(),
        empty_path.clone(),
        empty_shared.clone(),
        "".to_string(),
        empty_indices.clone(),
        info.clone(),
    );
    assert_eq!(
        non_optional, "NonOptional::Absent",
        "non-Optional Absent must stay on its declared enum surface"
    );

    let optional = emit_variant_pattern(
        "Absent".to_string(),
        Some("Optional".to_string()),
        empty_bindings,
        empty_path,
        empty_shared,
        "".to_string(),
        empty_indices,
        info,
    );
    assert_eq!(
        optional, "None",
        "Optional Absent must lower to Rust Option::None"
    );
}

#[test]
fn rust_emitter_lowers_holds_violates_only_for_witness_parent() {
    let info = empty_emit_graph_info();
    let empty_bindings = Rc::new(vec![]);
    let empty_path = Rc::new(vec![]);
    let empty_shared = Rc::new(std::collections::BTreeSet::new());
    let empty_indices = Rc::new(std::collections::HashMap::new());

    let non_witness = emit_variant_pattern(
        "Holds".to_string(),
        Some("NonWitness".to_string()),
        empty_bindings.clone(),
        empty_path.clone(),
        empty_shared.clone(),
        "".to_string(),
        empty_indices.clone(),
        info.clone(),
    );
    assert_eq!(
        non_witness, "NonWitness::Holds",
        "non-Witness Holds must stay on its declared enum surface"
    );

    let witness = emit_variant_pattern(
        "Holds".to_string(),
        Some("Witness".to_string()),
        empty_bindings,
        empty_path,
        empty_shared,
        "".to_string(),
        empty_indices,
        info,
    );
    assert_eq!(
        witness, "v1_rt::Witness::Holds",
        "Witness Holds must lower to the runtime Witness enum"
    );
}

#[test]
fn map_get_matches_witness_lookup_to_present_absent() {
    let src = r#"module test.witness_map_get
import v2.std.collection { Absent, Present, empty_map, map_get, map_insert }
import v2.std.diagnostic { Accepted, Rejected }

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
fn map_get_rejects_non_absent_lookup_failure() {
    let src = r#"module test.witness_map_get_rejects
import v2.std.collection { Map, Absent, Present, map_get }
import v2.std.diagnostic { Accepted, Rejected, Diagnostic, ExternalContractUnknown, Unavailable, port_locus }
import v2.std.witness { Violates }

fn custom_lookup_failure() -> Diagnostic {
  Diagnostic {
    reason: ^custom_lookup_failure,
    at: port_locus(port: ^custom_map_lookup_port),
    correction: Unavailable { reason: ExternalContractUnknown }
  }
}

fn malformed_map() -> Map<String, Int> {
  Map {
    lookup: fn(_) {
      Violates { diagnostic: custom_lookup_failure() }
    }
  }
}

fn rejects_custom_lookup_failure() -> Bool {
  match map_get(malformed_map(), "k") {
    Accepted { value: Present { value: _ }, diagnostics: _ } => false
    Accepted { value: Absent, diagnostics: _ } => false
    Rejected { diagnostics: _ } => true
  }
}
"#;
    match run_v4_module(
        "test/witness_map_get_rejects.dag",
        src,
        "rejects_custom_lookup_failure",
    ) {
        Value::Bool(true) => {}
        other => {
            panic!("expected Bool(true) from non-absent lookup failure rejection, got {other:?}")
        }
    }
}

#[test]
fn mark_excluded_no_longer_pattern_match_fails() {
    let entry = "src/v2/lens/affected_set/excluded_propagation_proof.dag";
    let content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read {entry}: {e}"));
    match run_v4_module(entry, &content, "excluded_propagation_proof_claim_holds") {
        Value::Bool(_) => {}
        other => panic!("expected Bool witness from mark_excluded path, not crash; got {other:?}"),
    }
}
