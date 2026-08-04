//! Hand-Rust reconciliation receipt for `coproduct_reflection::marshal_generic`
//! nullary variant-ref atom emission (review 47961 / scaffold-retirement pilot).
//!
//! Pure `.dag` structural readers over `decl_facts` body skeletons must see
//! `dissolves_to: SingleAuthority` as an `Atom` identity — without this marshal
//! arm, scaffold initializer decode collapses every real site to undecodable.

use std::rc::Rc;

use im::HashMap;
use v1_compiler::coproduct_reflection::eval_decl_facts;
use v1_compiler::v1_compiler_infer_emit_info::empty_emit_graph_info;
use v1_compiler::v1_compiler_infer_items::ResolvedGraph;
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

use crate::helpers::workspace_root;

const SPECIMEN_QN: &str = "std.bytes.bytes_seam_host_realization_marker";

fn wet_ctx() -> InterpContext {
    let graph = ResolvedGraph {
        modules: Rc::new(im::vector![]),
        item_registry: Rc::new(HashMap::new()),
        diagnostics: Rc::new(im::vector![]),
        emit_graph_info: empty_emit_graph_info(),
    };
    InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Wet)
}

fn value_contains_atom_identity(val: &Value, ctx: &InterpContext, identity: &str) -> bool {
    match val {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if ctx.sym_eq(*variant_name, "Atom")
                && fields.iter().any(|(k, v)| {
                    ctx.sym_eq(*k, "identity")
                        && matches!(v, Value::Str(s) if s.as_str() == identity)
                })
            {
                return true;
            }
            fields
                .iter()
                .any(|(_, v)| value_contains_atom_identity(v, ctx, identity))
        }
        Value::Record { fields, .. } => fields
            .iter()
            .any(|(_, v)| value_contains_atom_identity(v, ctx, identity)),
        Value::List(items) => items
            .iter()
            .any(|v| value_contains_atom_identity(v, ctx, identity)),
        _ => false,
    }
}

fn decl_fact_node_skeleton(
    ctx: &InterpContext,
    rows: &Value,
    qualified_name: &str,
) -> Option<Value> {
    let items = match rows {
        Value::List(items) => items,
        other => panic!("expected decl_facts List, got {other:?}"),
    };
    for row in items.iter() {
        let fields = match row {
            Value::Record { fields, .. } => fields.as_ref(),
            other => panic!("expected DeclFact Record, got {other:?}"),
        };
        let qn = match ctx.field(fields, "qualified_name") {
            Some(Value::Str(s)) => s.as_str(),
            other => panic!("expected qualified_name Str, got {other:?}"),
        };
        if qn != qualified_name {
            continue;
        }
        return ctx.field(fields, "node").cloned();
    }
    None
}

#[test]
fn decl_facts_data_init_skeleton_includes_nullary_variant_ref_atoms() {
    let ctx = wet_ctx();
    let roots = vec![
        workspace_root().join("dag").to_string_lossy().into_owned(),
        workspace_root()
            .join("src/v2")
            .to_string_lossy()
            .into_owned(),
    ];
    let rows = eval_decl_facts(&ctx, &roots).expect("eval_decl_facts");
    let skeleton = decl_fact_node_skeleton(&ctx, &rows, SPECIMEN_QN)
        .unwrap_or_else(|| panic!("missing decl_facts row for {SPECIMEN_QN}"));
    assert!(
        value_contains_atom_identity(&skeleton, &ctx, "SingleAuthority"),
        "scaffold disposition initializer skeleton must surface dissolves_to variant ref as Atom identity"
    );
}
