//! Throwaway X-viability gate (snappy msg_be7834be): native `compile_to_resolved` →
//! marshal MemorySpec `CoreNode` → interpreted v4 `infer()` → `run_required_lens_gates`.
//!
//! Decisive in one run: bare-Int must Reject `^unit_modeling_flat_scalar_unit_leaf_fixable`;
//! modeled `ByteSize` carrier must Accept. Termination alone is insufficient.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use v2_compiler::cli_run::make_eval_context;
use v2_compiler::coproduct_reflection::marshal_conj_type_item;
use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};
use v2_compiler::v2_compiler_infer_items::{ItemKind, ResolvedGraph};
use v2_compiler::v2_interpreter::{run_in_context_with_args, InterpResult, Value};
use v2_compiler::v2_std_core::Node;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const INFER_PROBE_TIMEOUT: Duration = Duration::from_secs(90);

const PROBE_HARNESS: &str = r#"module test.enforce_host_infer_native_probe

import v4.compiler.compile {
  always_required_lenses,
  run_required_lens_gates_on_subtree
}
import v4.compiler.infer { infer }
import v4.std.diagnostic { Accepted, Rejected }
import v4.std.logic { Bool }
import v4.std.node { Node }

fn probe_infer_terminates(tree: Node) -> Bool {
  match infer(tree: tree) {
    Accepted { value: _, diagnostics: _ } => true
    Rejected { diagnostics: _ } => true
  }
}

fn probe_lens_rejects_unit_modeling(tree: Node) -> Bool {
  match infer(tree: tree) {
    Rejected { diagnostics: _ } => false
    Accepted { value: inferred, diagnostics: _ } =>
      match run_required_lens_gates_on_subtree(
        inferred: inferred,
        lenses: always_required_lenses()
      ) {
        Rejected { diagnostics: r } =>
          r.head.reason == ^unit_modeling_flat_scalar_unit_leaf_fixable
        Accepted { value: _, diagnostics: _ } => false
      }
  }
}

fn probe_lens_accepts(tree: Node) -> Bool {
  match infer(tree: tree) {
    Rejected { diagnostics: _ } => false
    Accepted { value: inferred, diagnostics: _ } =>
      match run_required_lens_gates_on_subtree(
        inferred: inferred,
        lenses: always_required_lenses()
      ) {
        Accepted { value: _, diagnostics: _ } => true
        Rejected { diagnostics: _ } => false
      }
  }
}
"#;

const BARE_INT_SOURCE: &str = r#"module stage0.real_source.unit_modeling.reject

type MemorySpec {
  ram_bytes: Int
}
"#;

const MODELED_CARRIER_SOURCE: &str = r#"module stage0.real_source.unit_modeling.accept

import std.measure { ByteSize }

type MemorySpec {
  ram_bytes: ByteSize
}
"#;

fn probe_source_roots() -> Vec<std::path::PathBuf> {
    let ws = workspace_root();
    vec![ws.join("src/v4"), ws.join("dsl")]
}

fn merge_sources(mut left: Vec<Rc<SourceFile>>, right: Vec<Rc<SourceFile>>) -> Vec<Rc<SourceFile>> {
    let mut seen = HashMap::new();
    let mut out = Vec::new();
    for s in left.drain(..).chain(right) {
        if seen.insert(s.path.clone(), ()).is_none() {
            out.push(s);
        }
    }
    out
}

fn probe_sources(subject_path: &str, subject_content: &str) -> Vec<Rc<SourceFile>> {
    let harness = resolve_imports_transitively_with_source_roots(
        "test/enforce_host_infer_native_probe.dag",
        PROBE_HARNESS,
        &probe_source_roots(),
    );
    let subject = resolve_imports_transitively_with_source_roots(
        subject_path,
        subject_content,
        &probe_source_roots(),
    );
    merge_sources(harness, subject)
}

fn find_type_item<'a>(graph: &'a ResolvedGraph, type_name: &str) -> &'a Rc<Node> {
    let info = graph
        .item_registry
        .values()
        .find(|info| info.kind == ItemKind::TypeItem && info.name == type_name)
        .unwrap_or_else(|| panic!("{type_name} not in item_registry"));
    graph
        .modules
        .iter()
        .flat_map(|m| m.items.iter())
        .find(|item| {
            graph
                .item_registry
                .get(&item.name)
                .is_some_and(|i| i.kind == ItemKind::TypeItem && i.name == info.name)
        })
        .unwrap_or_else(|| panic!("{type_name} type item node missing"))
}

fn compile_probe_graph(
    subject_path: &str,
    subject_content: &str,
) -> Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult> {
    compile_to_resolved(Rc::new(probe_sources(subject_path, subject_content)))
}

fn memory_spec_tree_value(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
) -> Value {
    let graph = resolved
        .graph
        .as_ref()
        .expect("resolved graph for probe sources");
    let ctx = make_eval_context(graph, resolved.source_indices.clone());
    let item = find_type_item(graph, "MemorySpec");
    marshal_conj_type_item(&ctx, item).expect("marshal MemorySpec to v4 Node Value")
}

fn run_probe_fn(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
    fn_name: &str,
    tree: Value,
) -> InterpResult<Value> {
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = make_eval_context(graph, resolved.source_indices.clone());
    let args = [(Some("tree".to_string()), tree)];
    run_in_context_with_args(&ctx, fn_name, &args, false)
}

fn run_probe_fn_timed(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
    fn_name: &str,
    tree: Value,
) -> Result<Value, String> {
    let start = Instant::now();
    let result = run_probe_fn(resolved, fn_name, tree);
    let elapsed = start.elapsed();
    if elapsed > INFER_PROBE_TIMEOUT {
        return Err(format!(
            "HANG: {fn_name} exceeded {:?} (elapsed {:?})",
            INFER_PROBE_TIMEOUT, elapsed
        ));
    }
    result.map_err(|e| format!("{e}"))
}

fn assert_resolved_ok(resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved probe graph, diagnostics: {msgs:?}"
    );
}

fn assert_bool_probe(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
    fn_name: &str,
    tree: Value,
    expect: bool,
) {
    let value = run_probe_fn_timed(resolved, fn_name, tree)
        .unwrap_or_else(|e| panic!("probe {fn_name}: {e}"));
    match value {
        Value::Bool(v) if v == expect => {}
        other => panic!("probe {fn_name}: expected Bool({expect}), got {other:?}"),
    }
}

#[test]
fn infer_native_node_terminates_on_bare_int_memory_spec() {
    let resolved = compile_probe_graph("stage0/memory_spec_reject.dag", BARE_INT_SOURCE);
    assert_resolved_ok(&resolved);
    let tree = memory_spec_tree_value(&resolved);
    assert_bool_probe(&resolved, "probe_infer_terminates", tree, true);
}

#[test]
fn bare_int_native_infer_lens_chain_rejects_unit_modeling() {
    let resolved = compile_probe_graph("stage0/memory_spec_reject.dag", BARE_INT_SOURCE);
    assert_resolved_ok(&resolved);
    let tree = memory_spec_tree_value(&resolved);
    assert_bool_probe(&resolved, "probe_lens_rejects_unit_modeling", tree, true);
}

#[test]
fn modeled_carrier_native_infer_lens_chain_accepts() {
    let resolved = compile_probe_graph("stage0/memory_spec_accept.dag", MODELED_CARRIER_SOURCE);
    assert_resolved_ok(&resolved);
    let tree = memory_spec_tree_value(&resolved);
    assert_bool_probe(&resolved, "probe_lens_accepts", tree, true);
}
