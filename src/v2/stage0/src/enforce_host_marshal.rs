//! Enforce-host record (Conj) marshaling: v2 resolved type item → v4 substrate `Node` Value.
//! Host-only adapter for the X-viability bridge (native `compile_to_resolved` → lens).
//! Not coproduct-arm reflection — Conj field edges only.

use std::collections::HashMap;
use std::rc::Rc;

use crate::v2_interpreter::{InterpContext, InterpError, InterpResult, Value};
use crate::v2_std_core::{authored_name_at, field_node_type_expr, inferred_to_node, Connective, Node};

fn nullary_connective_variant(ctx: &InterpContext, name: &str) -> Value {
    Value::Variant {
        type_name: ctx.sym("Connective"),
        variant_name: ctx.sym(name),
        fields: Rc::new(HashMap::new()),
    }
}

fn atom_connective_variant(ctx: &InterpContext, identity: &str) -> Value {
    Value::Variant {
        type_name: ctx.sym("Connective"),
        variant_name: ctx.sym("Atom"),
        fields: Rc::new(HashMap::from([(
            ctx.sym("identity"),
            Value::Str(identity.to_string()),
        )])),
    }
}

fn node_kind_type_node(ctx: &InterpContext, connective: Value) -> Value {
    Value::Variant {
        type_name: ctx.sym("NodeKind"),
        variant_name: ctx.sym("TypeNode"),
        fields: Rc::new(HashMap::from([(ctx.sym("connective"), connective)])),
    }
}

fn synthetic_occurrence(ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("NodeOccurrenceId"),
        variant_name: ctx.sym("SyntheticOccurrence"),
        fields: Rc::new(HashMap::new()),
    }
}

fn node_record(ctx: &InterpContext, kind: Value, children: Vec<Value>) -> Value {
    Value::Record {
        type_name: ctx.sym("Node"),
        fields: Rc::new(HashMap::from([
            (ctx.sym("kind"), kind),
            (
                ctx.sym("children"),
                crate::v2_interpreter::list_value(children),
            ),
            (ctx.sym("occurrence_id"), synthetic_occurrence(ctx)),
        ])),
    }
}

fn edge_named(ctx: &InterpContext, name: &str, target: Value) -> Value {
    Value::Record {
        type_name: ctx.sym("Edge"),
        fields: Rc::new(HashMap::from([
            (
                ctx.sym("label"),
                Value::Variant {
                    type_name: ctx.sym("EdgeLabel"),
                    variant_name: ctx.sym("Named"),
                    fields: Rc::new(HashMap::from([(
                        ctx.sym("name"),
                        Value::Str(name.to_string()),
                    )])),
                },
            ),
            (ctx.sym("target"), target),
        ])),
    }
}

fn type_expr_authored_name(ctx: &InterpContext, type_expr: &Rc<Node>) -> String {
    let si = ctx.source_indices();
    let name = authored_name_at(si.clone(), type_expr.clone());
    if !name.is_empty() {
        return name;
    }
    if !type_expr.name.is_empty() {
        return type_expr.name.clone();
    }
    if let Some(inferred) = type_expr
        .inferred
        .as_ref()
        .and_then(|inf| inferred_to_node(inf.clone()))
    {
        let inferred_name = authored_name_at(si.clone(), inferred.clone());
        if !inferred_name.is_empty() {
            return inferred_name;
        }
        if !inferred.name.is_empty() {
            return inferred.name.clone();
        }
    }
    String::new()
}

/// Map v2 authored kernel spellings to v4 resolve binding symbols (dag language model K-1).
fn kernel_binding_symbol(authored: &str) -> &str {
    match authored {
        "Int" => "dag_binding_type_int",
        other => other,
    }
}

fn marshal_kernel_type_expr_ref(ctx: &InterpContext, type_expr: &Rc<Node>) -> InterpResult<Value> {
    let name = type_expr_authored_name(ctx, type_expr);
    if name.is_empty() {
        return Err(InterpError::TypeError {
            msg: "marshal_kernel_type_expr_ref: empty authored type name".to_string(),
        });
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(
            ctx,
            atom_connective_variant(ctx, kernel_binding_symbol(&name)),
        ),
        vec![],
    ))
}

/// Marshal a resolved record (Conj) type item to substrate `Node` with Named field edges.
pub fn marshal_conj_type_item(ctx: &InterpContext, item: &Rc<Node>) -> InterpResult<Value> {
    if item.connective != Connective::Conj {
        return Err(InterpError::TypeError {
            msg: "marshal_conj_type_item: type is not a record (Conj)".to_string(),
        });
    }
    let si = ctx.source_indices();
    let mut edges = Vec::with_capacity(item.children.len());
    for field in item.children.iter() {
        let field_name = authored_name_at(si.clone(), field.clone());
        let type_expr = field
            .inferred
            .as_ref()
            .and_then(|inf| inferred_to_node(inf.clone()))
            .unwrap_or_else(|| field_node_type_expr(field.clone()));
        let target = marshal_kernel_type_expr_ref(ctx, &type_expr)?;
        edges.push(edge_named(ctx, &field_name, target));
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Conj")),
        edges,
    ))
}
