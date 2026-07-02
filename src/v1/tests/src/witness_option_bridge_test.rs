//! RESIDUAL after 5-test-migration (2026-07-02): 3 of the original 6 tests are
//! migrated to marker-discovered floor witnesses in
//! src/v2/test/claim/manual/witness_option_bridge_test.dag (map_get hit->Present,
//! miss->Absent, non-absent lookup failure -> Rejected fail-closed, plus the
//! affected_set excluded-propagation proof smoke).
//! The 3 tests below stay:
//! - match_pattern_does_not_bridge_witness_to_some_none: scans v1_interpreter.rs
//!   source text — a pinned-harness fact, dissolving with the v1 interpreter.
//! - rust_emitter_lowers_*: unit tests of the v1 Rust emitter's variant-pattern
//!   lowering — v1-EMITTER-coupled (lane ruling): they die with the Route-A
//!   emitter retirement, not before; migrating them would cement the v1 emitter.
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
