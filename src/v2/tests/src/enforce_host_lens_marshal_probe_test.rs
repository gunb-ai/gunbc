//! X-viability gate (snappy msg_b687c1a7): bypass v4 `infer()` — marshal native
//! `compile_to_resolved` output directly into v4 `InferredTree` and run
//! `run_required_lens_gates_on_subtree`. Marshaled `Value`s are interned in the
//! marshal `InterpContext`; lens evaluation must use that same context.
//!
//! Production transport: `gunbc validate` + `enforce_host_validate` (shared helper).

use std::rc::Rc;

use v2_compiler::coproduct_reflection::marshal_conj_type_item;
use v2_compiler::enforce_host_validate::{validate_marshal_lens, MarshalLensVerdict, ValidateOutcome};
use v2_compiler::v2_compiler_compile;
use v2_compiler::v2_interpreter::{InterpContext, Value};
use v2_compiler::v2_std_core::Node;

use crate::helpers::workspace_root;

const HARNESS_ENTRY: &str = v2_compiler::enforce_host_validate::DEFAULT_HARNESS_ENTRY;
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

fn compile_probe_bundle(
    relative_entry: &str,
) -> Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult> {
    let roots = validate_source_roots();
    let ws = workspace_root();
    let harness_path = ws.join(HARNESS_ENTRY).to_string_lossy().to_string();
    let subject_path = ws.join(relative_entry).to_string_lossy().to_string();
    let harness_sources = v2_compiler::cli_run::load_sources_for_entry(&roots, &harness_path)
        .unwrap_or_else(|e| panic!("load harness: {e}"));
    let subject_sources = v2_compiler::cli_run::load_sources_for_entry(&roots, &subject_path)
        .unwrap_or_else(|e| panic!("load subject: {e}"));
    let mut by_path = std::collections::HashMap::new();
    for source in harness_sources.iter().chain(subject_sources.iter()) {
        by_path.insert(source.path.clone(), source.clone());
    }
    v2_compiler_compile::compile_to_resolved(Rc::new(by_path.into_values().collect()))
}

fn probe_eval_context(
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
) -> InterpContext {
    let graph = resolved.graph.as_ref().expect("probe graph");
    v2_compiler::cli_run::make_eval_context(graph, resolved.source_indices.clone())
}

fn memory_spec_root_value(
    ctx: &InterpContext,
    resolved: &Rc<v2_compiler::v2_compiler_compile::ResolvedPipelineResult>,
) -> Value {
    let graph = resolved.graph.as_ref().expect("resolved probe graph");
    let item = find_type_item(graph, "MemorySpec");
    marshal_conj_type_item(ctx, item).expect("marshal MemorySpec to v4 Node Value")
}

fn find_type_item<'a>(
    graph: &'a v2_compiler::v2_compiler_compile::ResolvedGraph,
    type_name: &str,
) -> &'a Rc<Node> {
    use v2_compiler::v2_compiler_infer_items::ItemKind;
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

fn is_free_monoid_carrier(ctx: &InterpContext, value: &Value) -> bool {
    match value {
        Value::Variant { variant_name, .. } => {
            ctx.sym_eq(*variant_name, "Empty") || ctx.sym_eq(*variant_name, "Cons")
        }
        Value::List(_) => false,
        _ => false,
    }
}

fn free_monoid_carrier_tag(ctx: &InterpContext, value: &Value) -> &'static str {
    match value {
        Value::List(_) => "Value::List",
        Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Empty") => "Empty",
        Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Cons") => "Cons",
        Value::Variant { .. } => "Variant(non-monoid)",
        _ => "other",
    }
}

fn free_monoid_elems<'a>(ctx: &InterpContext, value: &'a Value) -> Result<Vec<&'a Value>, String> {
    let mut out = Vec::new();
    let mut cur = value;
    loop {
        match cur {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                let head = ctx
                    .field(fields, "head")
                    .ok_or_else(|| "Cons without `head` field".to_string())?;
                out.push(head);
                cur = ctx
                    .field(fields, "tail")
                    .ok_or_else(|| "Cons without `tail` field".to_string())?;
            }
            Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Empty") => {
                return Ok(out);
            }
            Value::List(items) => {
                out.extend(items.iter());
                return Ok(out);
            }
            other => {
                return Err(format!(
                    "expected FreeMonoid (Cons/Empty), got {}",
                    value_type_label(ctx, other)
                ))
            }
        }
    }
}

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
    let edges = free_monoid_elems(ctx, children).unwrap_or_else(|e| {
        panic!(
            "{path}.children: expected FreeMonoid Cons/Empty, got {} ({e})",
            value_type_label(ctx, children)
        )
    });
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

fn run_shared_validate(fixture: &str, expect: MarshalLensVerdict) {
    let ws = workspace_root();
    let roots = validate_source_roots();
    match validate_marshal_lens(&ws, &roots, HARNESS_ENTRY, fixture) {
        ValidateOutcome::Pass(verdict) => assert_eq!(verdict, expect),
        ValidateOutcome::Fail { reason } => panic!("validate failed: {reason}"),
    }
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
    let edges = free_monoid_elems(&ctx, children).expect("children monoid");
    assert_eq!(edges.len(), 1, "MemorySpec has one field edge");
    let Value::Record {
        fields: edge_fields,
        ..
    } = edges[0]
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

/// Decisive PASS arm: bare-Int MemorySpec → `Rejected` with unit-modeling reason
/// through host-only marshal (no v4 `infer()`).
#[test]
fn bare_int_marshaled_inferred_tree_lens_rejects_unit_modeling() {
    run_shared_validate(BARE_INT_FIXTURE, MarshalLensVerdict::Rejected);
}

/// Bisect (snappy msg_3e3c99ad): marshaled root `Node.children` must be substrate
/// FreeMonoid (`Empty`/`Cons`), not host `Value::List`.
#[test]
fn marshaled_root_children_use_free_monoid_carrier() {
    let resolved = compile_probe_bundle(MODELED_CARRIER_FIXTURE);
    assert_resolved_ok(&resolved);
    let ctx = probe_eval_context(&resolved);
    let root = memory_spec_root_value(&ctx, &resolved);
    let Value::Record { fields, .. } = &root else {
        unreachable!()
    };
    let children = fields.get(&ctx.sym("children")).expect("children");
    assert!(
        is_free_monoid_carrier(&ctx, children),
        "marshaled Node.children carrier: {}",
        free_monoid_carrier_tag(&ctx, children)
    );
}

/// Decisive PASS arm: modeled ByteSize MemorySpec → dual-lens Accepted through
/// host-only marshal (no v4 `infer()`).
#[test]
fn modeled_carrier_marshaled_inferred_tree_lens_accepts() {
    run_shared_validate(MODELED_CARRIER_FIXTURE, MarshalLensVerdict::Accepted);
}
