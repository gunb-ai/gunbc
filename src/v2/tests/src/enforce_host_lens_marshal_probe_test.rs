//! X-viability gate (snappy msg_b687c1a7): bypass v4 `infer()` — marshal native
//! `compile_to_resolved` output directly into v4 `InferredTree` and run
//! `run_required_lens_gates_on_subtree`. Marshaled `Value`s are interned in the
//! marshal `InterpContext`; lens evaluation must use that same context.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use v2_compiler::cli_run::{self, make_eval_context};
use v2_compiler::coproduct_reflection::marshal_conj_type_item;
use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};
use v2_compiler::v2_compiler_infer_items::{ItemKind, ResolvedGraph};
use v2_compiler::v2_interpreter::{run_in_context_with_args, InterpContext, InterpResult, Value};
use v2_compiler::v2_std_core::Node;

use crate::helpers::workspace_root;

const LENS_PROBE_TIMEOUT: Duration = Duration::from_secs(90);

const HARNESS_ENTRY: &str = "src/v4/test/claim/manual/enforce_host_lens_bridge_harness.dag";
const BARE_INT_FIXTURE: &str = "src/v4/test/fixtures/enforce_host/bare_int_memory_spec.dag";
const MODELED_CARRIER_FIXTURE: &str =
    "src/v4/test/fixtures/enforce_host/modeled_carrier_memory_spec.dag";

fn validate_source_roots() -> Vec<String> {
    let ws = workspace_root();
    vec![
        ws.join("src/v4").to_string_lossy().to_string(),
        ws.join("dsl").to_string_lossy().to_string(),
    ]
}

fn compile_probe_bundle(entry_path: &str) -> Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult> {
    let roots = validate_source_roots();
    let harness_sources = cli_run::load_sources_for_entry(&roots, HARNESS_ENTRY)
        .unwrap_or_else(|e| panic!("load harness {HARNESS_ENTRY}: {e}"));
    let subject_sources = cli_run::load_sources_for_entry(&roots, entry_path)
        .unwrap_or_else(|e| panic!("load subject {entry_path}: {e}"));
    let mut by_path: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for source in harness_sources
        .iter()
        .chain(subject_sources.iter())
    {
        by_path.insert(source.path.clone(), source.clone());
    }
    compile_to_resolved(Rc::new(by_path.into_values().collect()))
}

fn memory_spec_root_value(
    ctx: &InterpContext,
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
) -> Value {
    let graph = resolved.graph.as_ref().expect("resolved probe graph");
    let item = find_type_item(graph, "MemorySpec");
    marshal_conj_type_item(ctx, item).expect("marshal MemorySpec to v4 Node Value")
}

fn run_probe_fn(ctx: &InterpContext, fn_name: &str, root: Value) -> InterpResult<Value> {
    let args = [(Some("root".to_string()), root)];
    run_in_context_with_args(ctx, fn_name, &args, false)
}

fn run_probe_fn_timed(ctx: &InterpContext, fn_name: &str, root: Value) -> Result<Value, String> {
    let start = Instant::now();
    let result = run_probe_fn(ctx, fn_name, root);
    let elapsed = start.elapsed();
    if elapsed > LENS_PROBE_TIMEOUT {
        return Err(format!(
            "HANG: {fn_name} exceeded {:?} (elapsed {:?})",
            LENS_PROBE_TIMEOUT, elapsed
        ));
    }
    result.map_err(|e| format!("{e}"))
}

fn probe_eval_context(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
) -> InterpContext {
    let graph = resolved.graph.as_ref().expect("probe graph");
    make_eval_context(graph, resolved.source_indices.clone())
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

fn assert_bool_probe(ctx: &InterpContext, fn_name: &str, root: Value, expect: bool) {
    let value =
        run_probe_fn_timed(ctx, fn_name, root).unwrap_or_else(|e| panic!("probe {fn_name}: {e}"));
    match value {
        Value::Bool(v) if v == expect => {}
        other => panic!("probe {fn_name}: expected Bool({expect}), got {other:?}"),
    }
}

fn value_type_label(ctx: &InterpContext, value: &Value) -> String {
    match value {
        Value::Record { type_name, .. } => ctx.resolve(*type_name),
        Value::Variant {
            type_name,
            variant_name,
            ..
        } => format!(
            "{}::{}",
            ctx.resolve(*type_name),
            ctx.resolve(*variant_name)
        ),
        Value::List(_) => "List".to_string(),
        Value::Str(_) => "String".to_string(),
        Value::Bool(_) => "Bool".to_string(),
        other => format!("{other:?}"),
    }
}

/// Walk marshaled host `Value`s and assert the v4 `Node`/`Edge` skeleton the lens
/// roster expects (TypeNode kind + `List<Edge>` children, each target a `Node`).
fn assert_v4_node_tree(ctx: &InterpContext, value: &Value, path: &str) {
    let Value::Record { type_name, fields } = value else {
        panic!(
            "{path}: expected Node record, got {}",
            value_type_label(ctx, value)
        );
    };
    assert_eq!(ctx.resolve(*type_name), "Node", "{path}: wrong record type");
    let kind = fields
        .get(&ctx.sym("kind"))
        .unwrap_or_else(|| panic!("{path}: missing kind"));
    let Value::Variant {
        type_name: kind_type,
        variant_name: kind_variant,
        ..
    } = kind
    else {
        panic!(
            "{path}.kind: expected NodeKind variant, got {}",
            value_type_label(ctx, kind)
        );
    };
    assert_eq!(ctx.resolve(*kind_type), "NodeKind", "{path}.kind.type");
    assert_eq!(
        ctx.resolve(*kind_variant),
        "TypeNode",
        "{path}.kind.variant"
    );
    let children = fields
        .get(&ctx.sym("children"))
        .unwrap_or_else(|| panic!("{path}: missing children"));
    let Value::List(edges) = children else {
        panic!(
            "{path}.children: expected list, got {}",
            value_type_label(ctx, children)
        );
    };
    for (index, edge) in edges.iter().enumerate() {
        assert_v4_edge(ctx, edge, &format!("{path}.children[{index}]"));
    }
}

fn assert_v4_edge(ctx: &InterpContext, value: &Value, path: &str) {
    let Value::Record { type_name, fields } = value else {
        panic!(
            "{path}: expected Edge record, got {}",
            value_type_label(ctx, value)
        );
    };
    assert_eq!(ctx.resolve(*type_name), "Edge", "{path}: wrong record type");
    let target = fields
        .get(&ctx.sym("target"))
        .unwrap_or_else(|| panic!("{path}: missing target"));
    assert_v4_node_tree(ctx, target, &format!("{path}.target"));
}

#[test]
fn marshaled_memory_spec_has_v4_node_edge_skeleton() {
    let resolved = compile_probe_bundle(BARE_INT_FIXTURE);
    assert_resolved_ok(&resolved);
    let ctx = probe_eval_context(&resolved);
    let root = memory_spec_root_value(&ctx, &resolved);
    assert_v4_node_tree(&ctx, &root, "MemorySpec");
    let Value::Record { fields, .. } = &root else {
        unreachable!()
    };
    let kind = fields.get(&ctx.sym("kind")).expect("kind");
    let Value::Variant {
        fields: kind_fields,
        ..
    } = kind
    else {
        unreachable!()
    };
    let connective = kind_fields.get(&ctx.sym("connective")).expect("connective");
    assert_eq!(
        value_type_label(&ctx, connective),
        "Connective::Conj",
        "root carrier connective"
    );
    let children = fields.get(&ctx.sym("children")).expect("children");
    let Value::List(edges) = children else {
        unreachable!()
    };
    assert_eq!(edges.len(), 1, "MemorySpec has one field edge");
    let Value::Record {
        fields: edge_fields,
        ..
    } = &edges[0]
    else {
        unreachable!()
    };
    let label = edge_fields.get(&ctx.sym("label")).expect("label");
    let Value::Variant {
        variant_name,
        fields: label_fields,
        ..
    } = label
    else {
        unreachable!()
    };
    assert_eq!(ctx.resolve(*variant_name), "Named");
    let name = label_fields.get(&ctx.sym("name")).expect("name");
    assert!(matches!(name, Value::Str(s) if s == "ram_bytes"));
    let target = edge_fields.get(&ctx.sym("target")).expect("target");
    let Value::Record {
        fields: target_fields,
        ..
    } = target
    else {
        unreachable!()
    };
    let target_kind = target_fields.get(&ctx.sym("kind")).expect("target.kind");
    let Value::Variant {
        fields: target_kind_fields,
        ..
    } = target_kind
    else {
        unreachable!()
    };
    let atom = target_kind_fields
        .get(&ctx.sym("connective"))
        .expect("target.kind.connective");
    let Value::Variant {
        variant_name: atom_variant,
        fields: atom_fields,
        ..
    } = atom
    else {
        unreachable!()
    };
    assert_eq!(ctx.resolve(*atom_variant), "Atom");
    let identity = atom_fields.get(&ctx.sym("identity")).expect("identity");
    assert!(
        matches!(identity, Value::Str(s) if s == "dag_binding_type_int"),
        "Int leaf should marshal to kernel binding symbol, got {identity:?}"
    );
}

fn run_bare_int_lens_probe(fn_name: &str, expect: bool) {
    let resolved = compile_probe_bundle(BARE_INT_FIXTURE);
    assert_resolved_ok(&resolved);
    let ctx = probe_eval_context(&resolved);
    let root = memory_spec_root_value(&ctx, &resolved);
    assert_bool_probe(&ctx, fn_name, root, expect);
}

/// Decisive PASS arm: bare-Int MemorySpec → `Rejected` with unit-modeling reason
/// through host-only marshal (no v4 `infer()`).
#[test]
fn bare_int_marshaled_inferred_tree_lens_rejects_unit_modeling() {
    run_bare_int_lens_probe("probe_lens_rejects_unit_modeling_from_marshaled_root", true);
}

#[test]
fn modeled_carrier_marshaled_inferred_tree_lens_accepts() {
    let resolved = compile_probe_bundle(MODELED_CARRIER_FIXTURE);
    assert_resolved_ok(&resolved);
    let ctx = probe_eval_context(&resolved);
    let root = memory_spec_root_value(&ctx, &resolved);
    assert_bool_probe(&ctx, "probe_lens_accepts_from_marshaled_root", root, true);
}
