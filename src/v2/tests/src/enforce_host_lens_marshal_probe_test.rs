//! X-viability gate (snappy msg_b687c1a7): bypass v4 `infer()` — marshal native
//! `compile_to_resolved` output directly into v4 `InferredTree` and run
//! `run_required_lens_gates_on_subtree`. Non-termination is parse-only; infer is
//! unnecessary when native already carries `InferredNode::Resolved`.

use std::rc::Rc;
use std::time::{Duration, Instant};

use v2_compiler::cli_run::make_eval_context;
use v2_compiler::coproduct_reflection::marshal_conj_type_item;
use v2_compiler::v2_compiler_compile::compile_to_resolved;
use v2_compiler::v2_compiler_infer_items::{ItemKind, ResolvedGraph};
use v2_compiler::v2_interpreter::{run_in_context_with_args, InterpResult, Value};
use v2_compiler::v2_std_core::Node;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const LENS_PROBE_TIMEOUT: Duration = Duration::from_secs(90);

const LENS_PROBE_HARNESS: &str = r#"module test.enforce_host_lens_marshal_probe

import v4.compiler.compile {
  always_required_lenses,
  run_required_lens_gates_on_subtree
}
import v4.std.diagnostic { Accepted, Rejected }
import v4.std.logic { Bool }
import v4.std.node { Node }
import v4.test.claim.lens_common.infer_fixture {
  claim_inferred_facts,
  claim_inferred_tree
}

fn probe_lens_rejects_unit_modeling_from_marshaled_root(root: Node) -> Bool {
  let tree = claim_inferred_tree(
    root: root,
    facts: claim_inferred_facts(
      type_symbol: ^enforce_host_probe_type_sym,
      algebra_symbol: ^enforce_host_probe_algebra_sym,
      descent_symbol: ^enforce_host_probe_descent_sym
    )
  )
  match run_required_lens_gates_on_subtree(
    inferred: tree,
    lenses: always_required_lenses()
  ) {
    Rejected { diagnostics: r } =>
      r.head.reason == ^unit_modeling_flat_scalar_unit_leaf_fixable
    Accepted { value: _, diagnostics: _ } => false
  }
}

fn probe_lens_accepts_from_marshaled_root(root: Node) -> Bool {
  let tree = claim_inferred_tree(
    root: root,
    facts: claim_inferred_facts(
      type_symbol: ^enforce_host_probe_type_sym,
      algebra_symbol: ^enforce_host_probe_algebra_sym,
      descent_symbol: ^enforce_host_probe_descent_sym
    )
  )
  match run_required_lens_gates_on_subtree(
    inferred: tree,
    lenses: always_required_lenses()
  ) {
    Accepted { value: _, diagnostics: _ } => true
    Rejected { diagnostics: _ } => false
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

fn compile_probe_bundle(
    subject_path: &str,
    subject_content: &str,
) -> Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult> {
    let roots = probe_source_roots();
    let mut sources = resolve_imports_transitively_with_source_roots(
        "test/enforce_host_lens_marshal_probe.dag",
        LENS_PROBE_HARNESS,
        &roots,
    );
    for source in
        resolve_imports_transitively_with_source_roots(subject_path, subject_content, &roots)
    {
        if !sources.iter().any(|existing| existing.path == source.path) {
            sources.push(source);
        }
    }
    compile_to_resolved(Rc::new(sources))
}

fn memory_spec_root_value(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
) -> Value {
    let graph = resolved.graph.as_ref().expect("resolved probe graph");
    let ctx = make_eval_context(graph, resolved.source_indices.clone());
    let item = find_type_item(graph, "MemorySpec");
    marshal_conj_type_item(&ctx, item).expect("marshal MemorySpec to v4 Node Value")
}

fn run_probe_fn(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
    fn_name: &str,
    root: Value,
) -> InterpResult<Value> {
    let graph = resolved.graph.as_ref().expect("probe graph");
    let ctx = make_eval_context(graph, resolved.source_indices.clone());
    let args = [(Some("root".to_string()), root)];
    run_in_context_with_args(&ctx, fn_name, &args, false)
}

fn run_probe_fn_timed(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
    fn_name: &str,
    root: Value,
) -> Result<Value, String> {
    let start = Instant::now();
    let result = run_probe_fn(resolved, fn_name, root);
    let elapsed = start.elapsed();
    if elapsed > LENS_PROBE_TIMEOUT {
        return Err(format!(
            "HANG: {fn_name} exceeded {:?} (elapsed {:?})",
            LENS_PROBE_TIMEOUT, elapsed
        ));
    }
    result.map_err(|e| format!("{e}"))
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
    root: Value,
    expect: bool,
) {
    let value = run_probe_fn_timed(resolved, fn_name, root)
        .unwrap_or_else(|e| panic!("probe {fn_name}: {e}"));
    match value {
        Value::Bool(v) if v == expect => {}
        other => panic!("probe {fn_name}: expected Bool({expect}), got {other:?}"),
    }
}

fn run_bare_int_lens_probe(fn_name: &str, expect: bool) {
    let resolved = compile_probe_bundle("stage0/memory_spec_reject.dag", BARE_INT_SOURCE);
    assert_resolved_ok(&resolved);
    let root = memory_spec_root_value(&resolved);
    assert_bool_probe(&resolved, fn_name, root, expect);
}

/// Decisive PASS arm: bare-Int MemorySpec → `Rejected` with unit-modeling reason
/// through host-only marshal (no v4 `infer()`).
#[test]
#[ignore = "throwaway X-viability probe: host InferredTree marshal shape gap (no field children on Connective); unblock after adapter fix (msg_b687c1a7)"]
fn bare_int_marshaled_inferred_tree_lens_rejects_unit_modeling() {
    run_bare_int_lens_probe("probe_lens_rejects_unit_modeling_from_marshaled_root", true);
}

#[test]
#[ignore = "modeled-carrier subject pulls dsl/measure closure; run after bare-Int arm locks"]
fn modeled_carrier_marshaled_inferred_tree_lens_accepts() {
    let resolved = compile_probe_bundle("stage0/memory_spec_accept.dag", MODELED_CARRIER_SOURCE);
    assert_resolved_ok(&resolved);
    let root = memory_spec_root_value(&resolved);
    assert_bool_probe(
        &resolved,
        "probe_lens_accepts_from_marshaled_root",
        root,
        true,
    );
}
