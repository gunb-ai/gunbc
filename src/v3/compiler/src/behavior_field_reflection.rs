//! Behavior → [`FieldValue`] reflection (design-reflection-completeness).
//!
//! Split from `lens_apply.rs` so evaluator-owned reification can use the same spine
//! without treating `lens_apply.rs` as the runtime authority for E6-G1.a.


use crate::dag::{
    AtomPayload, Behavior, BindEmitParticipation, BindNode, BoolPortRef, BranchArm,
    BranchEmitParticipation, BranchNode, BranchPattern, BreakingShape, CardinalityBound,
    ClusterId, CreateCause, Dag, DeclarationId, EffectShape, FieldValue, HttpMethodScalar,
    IdempotentShape, KeySource, LiteralBits, LoopBound, LoopNode, NodeId, NonSingletonList,
    OperationEffect, OperatorKind, Path, PayloadBinding, PortId, TransformNode,
    TransformTarget, TypeConnective, ValueNode, WorkflowEffect,
};
use crate::diagnostics::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectError(pub &'static str);

pub type ReflectResult<T> = Result<T, ReflectError>;

fn err<T>(msg: &'static str) -> ReflectResult<T> {
    Err(ReflectError(msg))
}

fn disj_variant_ty(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
) -> ReflectResult<DeclarationId> {
    let mut decl_id = dag
        .declaration_by_name(sum_name)
        .ok_or(ReflectError("missing sum type"))?
        .id;
    const PEEL_MAX: usize = 64;
    for _ in 0..PEEL_MAX {
        let decl = dag.declaration(decl_id);
        match &decl.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                decl_id = *template;
            }
            TypeConnective::Disj { variants } => {
                return variants
                    .iter()
                    .find(|v| v.label == variant_label)
                    .map(|v| v.ty)
                    .ok_or(ReflectError("missing sum variant"));
            }
            TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name))
                if name == variant_label =>
            {
                // Single-variant enum: `Instantiation → Atom(Label)` (no `Disj` row).
                return Ok(decl_id);
            }
            _ => return err("sum type is not Disj or unit Atom"),
        }
    }
    err("Instantiation peel depth exceeded")
}

fn port_fv(p: PortId) -> FieldValue {
    FieldValue::Literal(LiteralBits::Int(i64::from(p.raw())))
}

fn node_fv(n: NodeId) -> FieldValue {
    FieldValue::Literal(LiteralBits::Int(i64::from(n.raw())))
}

fn cluster_fv(c: ClusterId) -> FieldValue {
    FieldValue::Literal(LiteralBits::Int(i64::from(c.raw())))
}

fn reflect_source_span(span: &SourceSpan) -> FieldValue {
    // Field names must match `type SourceSpan` in `dsl/std/types.dag` (`file`, `start`, `end`).
    // The Rust carrier still uses `byte_start` / `byte_end` on [`SourceSpan`].
    FieldValue::Record(vec![
        (
            "file".to_string(),
            FieldValue::Literal(LiteralBits::String(span.file.clone())),
        ),
        (
            "start".to_string(),
            FieldValue::Literal(LiteralBits::Int(i64::from(span.byte_start))),
        ),
        (
            "end".to_string(),
            FieldValue::Literal(LiteralBits::Int(i64::from(span.byte_end))),
        ),
    ])
}

fn v3_list_empty_cons_ids(dag: &Dag) -> ReflectResult<(DeclarationId, DeclarationId)> {
    let list_decl = dag.declarations().iter().find(|d| {
        d.name.as_deref() == Some("List") && matches!(d.connective, TypeConnective::Disj { .. })
    });
    let Some(list_decl) = list_decl else {
        return err("List");
    };
    let TypeConnective::Disj { variants } = &list_decl.connective else {
        return err("List");
    };
    let empty = variants
        .iter()
        .find(|v| v.label == "Empty")
        .ok_or(ReflectError("List.Empty"))?
        .ty;
    let cons = variants
        .iter()
        .find(|v| v.label == "Cons")
        .ok_or(ReflectError("List.Cons"))?
        .ty;
    Ok((empty, cons))
}

fn named_record_type_root(dag: &Dag, name: &str) -> ReflectResult<DeclarationId> {
    let mut decl_id = dag
        .declaration_by_name(name)
        .ok_or(ReflectError("missing substrate record type"))?
        .id;
    const PEEL_MAX: usize = 64;
    for _ in 0..PEEL_MAX {
        match &dag.declaration(decl_id).connective {
            TypeConnective::Conj { .. } => return Ok(decl_id),
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                decl_id = *template;
            }
            _ => return err("substrate record type is not Conj"),
        }
    }
    err("substrate record peel depth exceeded")
}

fn conj_field_ty(
    dag: &Dag,
    conj_decl_id: DeclarationId,
    label: &str,
) -> ReflectResult<DeclarationId> {
    let decl = dag.declaration(conj_decl_id);
    let TypeConnective::Conj { children } = &decl.connective else {
        return err("expected Conj");
    };
    children
        .iter()
        .find(|c| c.label == label)
        .map(|c| c.ty)
        .ok_or(ReflectError("missing Conj field"))
}

/// [`FieldValue::Variant::payload`] is **positional**: one entry per field of the
/// variant payload [`TypeConnective::Conj`], in `children` order — same contract as
/// `variant_payload_for_binding` in this file.
fn sum_variant_payload(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
    payload: Vec<FieldValue>,
) -> ReflectResult<FieldValue> {
    let ctor_ty = disj_variant_ty(dag, sum_name, variant_label)?;
    match &dag.declaration(ctor_ty).connective {
        TypeConnective::Conj { children } => {
            if children.len() != payload.len() {
                return err("sum variant payload arity mismatch");
            }
        }
        _ => {
            if payload.len() > 1 {
                return err("sum variant payload arity mismatch");
            }
        }
    }
    Ok(FieldValue::Variant {
        constructor: ctor_ty,
        payload,
    })
}

fn peel_to_optional_cardinality_decl(
    dag: &Dag,
    mut ty: DeclarationId,
) -> ReflectResult<DeclarationId> {
    const PEEL_MAX: usize = 64;
    for _ in 0..PEEL_MAX {
        match &dag.declaration(ty).connective {
            TypeConnective::Cardinality(p) if p.bound() == CardinalityBound::AtMostOne => {
                return Ok(ty);
            }
            TypeConnective::Instantiation { template, .. } => ty = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => ty = *next,
            _ => return err("expected optional T? cardinality"),
        }
    }
    err("optional cardinality peel depth exceeded")
}

fn optional_some_none_constructor_ids(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
) -> ReflectResult<(DeclarationId, DeclarationId)> {
    let disj_id = dag
        .optional_match_disj(cardinality_decl_id)
        .ok_or(ReflectError("missing optional Some/None sum"))?;
    let decl = dag.declaration(disj_id);
    let TypeConnective::Disj { variants } = &decl.connective else {
        return err("optional match row is not Disj");
    };
    let some_ty = variants
        .iter()
        .find(|v| v.label == "Some")
        .ok_or(ReflectError("Some"))?
        .ty;
    let none_ty = variants
        .iter()
        .find(|v| v.label == "None")
        .ok_or(ReflectError("None"))?
        .ty;
    Ok((some_ty, none_ty))
}

/// Optional `T?` using the same `Some` / `None` sum as inference (`Cardinality` → match disj).
fn reflect_optional_sum<T>(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
    opt: Option<T>,
    mut reflect_some: impl FnMut(&Dag, T) -> ReflectResult<FieldValue>,
) -> ReflectResult<FieldValue> {
    let (some_ty, none_ty) = optional_some_none_constructor_ids(dag, cardinality_decl_id)?;
    Ok(match opt {
        None => FieldValue::Variant {
            constructor: none_ty,
            payload: vec![],
        },
        Some(x) => FieldValue::Variant {
            constructor: some_ty,
            payload: vec![reflect_some(dag, x)?],
        },
    })
}

fn reflect_port_id_list(dag: &Dag, ports: &[PortId]) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    let mut tail = FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    };
    for p in ports.iter().rev() {
        tail = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![port_fv(*p), tail],
        };
    }
    Ok(tail)
}

fn reflect_string_list_spine(dag: &Dag, strings: &[String]) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    let mut tail = FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    };
    for s in strings.iter().rev() {
        tail = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![FieldValue::Literal(LiteralBits::String(s.clone())), tail],
        };
    }
    Ok(tail)
}

fn reflect_optional_declaration_id(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
    opt: Option<DeclarationId>,
) -> ReflectResult<FieldValue> {
    reflect_optional_sum(dag, cardinality_decl_id, opt, |_d, id| {
        Ok(FieldValue::Reference(id))
    })
}

fn reflect_unit_variant(dag: &Dag, sum_name: &str, label: &str) -> ReflectResult<FieldValue> {
    sum_variant_payload(dag, sum_name, label, vec![])
}

fn reflect_arithmetic_op(dag: &Dag, op: crate::dag::ArithmeticOp) -> ReflectResult<FieldValue> {
    let label = match op {
        crate::dag::ArithmeticOp::Add => "Add",
        crate::dag::ArithmeticOp::Sub => "Sub",
        crate::dag::ArithmeticOp::Mul => "Mul",
        crate::dag::ArithmeticOp::Div => "Div",
    };
    reflect_unit_variant(dag, "ArithmeticOp", label)
}

fn reflect_comparison_op(dag: &Dag, op: crate::dag::ComparisonOp) -> ReflectResult<FieldValue> {
    let label = match op {
        crate::dag::ComparisonOp::Eq => "Eq",
        crate::dag::ComparisonOp::Ne => "Ne",
        crate::dag::ComparisonOp::Lt => "Lt",
        crate::dag::ComparisonOp::Le => "Le",
        crate::dag::ComparisonOp::Gt => "Gt",
        crate::dag::ComparisonOp::Ge => "Ge",
    };
    reflect_unit_variant(dag, "ComparisonOp", label)
}

fn reflect_logical_op(dag: &Dag, op: crate::dag::LogicalOp) -> ReflectResult<FieldValue> {
    let label = match op {
        crate::dag::LogicalOp::And => "And",
        crate::dag::LogicalOp::Or => "Or",
    };
    reflect_unit_variant(dag, "LogicalOp", label)
}

fn reflect_operator_kind(dag: &Dag, k: &OperatorKind) -> ReflectResult<FieldValue> {
    match k {
        OperatorKind::Arithmetic(op) => {
            let id = disj_variant_ty(dag, "OperatorKind", "Arithmetic")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_arithmetic_op(dag, *op)?],
            })
        }
        OperatorKind::Comparison(op) => {
            let id = disj_variant_ty(dag, "OperatorKind", "Comparison")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_comparison_op(dag, *op)?],
            })
        }
        OperatorKind::Logical(op) => {
            let id = disj_variant_ty(dag, "OperatorKind", "Logical")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_logical_op(dag, *op)?],
            })
        }
    }
}

fn reflect_http_method_scalar(dag: &Dag, m: HttpMethodScalar) -> ReflectResult<FieldValue> {
    let label = match m {
        HttpMethodScalar::Get => "GET",
        HttpMethodScalar::Post => "POST",
        HttpMethodScalar::Put => "PUT",
        HttpMethodScalar::Patch => "PATCH",
        HttpMethodScalar::Delete => "DELETE",
        HttpMethodScalar::Head => "HEAD",
        HttpMethodScalar::Options => "OPTIONS",
    };
    reflect_unit_variant(dag, "HttpMethod", label)
}

fn reflect_create_cause(dag: &Dag, c: &CreateCause) -> ReflectResult<FieldValue> {
    match c {
        CreateCause::PostAlways => {
            sum_variant_payload(dag, "CreateCause", "PostAlways", vec![])
        }
        CreateCause::KeylessFallback { method } => sum_variant_payload(
            dag,
            "CreateCause",
            "KeylessFallback",
            vec![reflect_http_method_scalar(dag, *method)?],
        ),
    }
}

fn reflect_key_source(dag: &Dag, ks: &KeySource) -> ReflectResult<FieldValue> {
    match ks {
        KeySource::PathParam { param } => sum_variant_payload(
            dag,
            "KeySource",
            "PathParam",
            vec![FieldValue::Literal(LiteralBits::String(param.clone()))],
        ),
        KeySource::InputField { field } => sum_variant_payload(
            dag,
            "KeySource",
            "InputField",
            vec![FieldValue::Literal(LiteralBits::String(field.clone()))],
        ),
        KeySource::CompositeKey { fields } => sum_variant_payload(
            dag,
            "KeySource",
            "CompositeKey",
            vec![reflect_string_list_spine(dag, fields)?],
        ),
    }
}

fn reflect_idempotent_shape(dag: &Dag, s: &IdempotentShape) -> ReflectResult<FieldValue> {
    match s {
        IdempotentShape::ReadEffect => {
            sum_variant_payload(dag, "IdempotentShape", "ReadEffect", vec![])
        }
        IdempotentShape::UpsertEffect { key_source } => sum_variant_payload(
            dag,
            "IdempotentShape",
            "UpsertEffect",
            vec![reflect_key_source(dag, key_source)?],
        ),
        IdempotentShape::DeleteEffect { key_source } => sum_variant_payload(
            dag,
            "IdempotentShape",
            "DeleteEffect",
            vec![reflect_key_source(dag, key_source)?],
        ),
    }
}

fn reflect_breaking_shape(dag: &Dag, s: &BreakingShape) -> ReflectResult<FieldValue> {
    match s {
        BreakingShape::CreateEffect { cause } => sum_variant_payload(
            dag,
            "BreakingShape",
            "CreateEffect",
            vec![reflect_create_cause(dag, cause)?],
        ),
        BreakingShape::AppendEffect => {
            sum_variant_payload(dag, "BreakingShape", "AppendEffect", vec![])
        }
    }
}

fn reflect_effect_shape(dag: &Dag, s: &EffectShape) -> ReflectResult<FieldValue> {
    match s {
        EffectShape::IsIdempotent(inner) => sum_variant_payload(
            dag,
            "EffectShape",
            "IsIdempotent",
            vec![reflect_idempotent_shape(dag, inner)?],
        ),
        EffectShape::IsBreaking(inner) => sum_variant_payload(
            dag,
            "EffectShape",
            "IsBreaking",
            vec![reflect_breaking_shape(dag, inner)?],
        ),
    }
}

fn reflect_operation_effect(dag: &Dag, op: &OperationEffect) -> ReflectResult<FieldValue> {
    Ok(FieldValue::Record(vec![
        (
            "operation_name".to_string(),
            FieldValue::Literal(LiteralBits::String(op.operation_name.clone())),
        ),
        ("shape".to_string(), reflect_effect_shape(dag, &op.shape)?),
    ]))
}

fn reflect_operation_effect_vec_spine(
    dag: &Dag,
    ops: &[OperationEffect],
) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    let mut tail = FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    };
    for op in ops.iter().rev() {
        let head = reflect_operation_effect(dag, op)?;
        tail = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![head, tail],
        };
    }
    Ok(tail)
}

fn reflect_bool_port_ref(_dag: &Dag, r: BoolPortRef) -> ReflectResult<FieldValue> {
    Ok(FieldValue::Record(vec![(
        "port".to_string(),
        port_fv(r.port_id()),
    )]))
}

fn reflect_workflow_effect(dag: &Dag, wf: &WorkflowEffect) -> ReflectResult<FieldValue> {
    match wf {
        WorkflowEffect::LinearEffect { ops } => sum_variant_payload(
            dag,
            "WorkflowEffect",
            "LinearEffect",
            vec![reflect_operation_effect_vec_spine(dag, ops)?],
        ),
        WorkflowEffect::BranchEffect { arms } => sum_variant_payload(
            dag,
            "WorkflowEffect",
            "BranchEffect",
            vec![reflect_non_singleton_branch_arms(dag, arms)?],
        ),
        WorkflowEffect::LoopEffect { body } => sum_variant_payload(
            dag,
            "WorkflowEffect",
            "LoopEffect",
            vec![reflect_workflow_effect(dag, body)?],
        ),
        WorkflowEffect::ParallelEffect { branches } => sum_variant_payload(
            dag,
            "WorkflowEffect",
            "ParallelEffect",
            vec![reflect_non_singleton_workflow_branches(dag, branches)?],
        ),
    }
}

fn reflect_branch_arm_vec_spine(dag: &Dag, arms: &[BranchArm]) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    let mut tail = FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    };
    for arm in arms.iter().rev() {
        tail = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![reflect_branch_arm(dag, arm)?, tail],
        };
    }
    Ok(tail)
}

fn reflect_non_singleton_branch_arms(
    dag: &Dag,
    arms: &NonSingletonList<BranchArm>,
) -> ReflectResult<FieldValue> {
    let rest_spine = reflect_branch_arm_vec_spine(dag, &arms.rest)?;
    Ok(FieldValue::Record(vec![
        ("first".to_string(), reflect_branch_arm(dag, &arms.first)?),
        ("second".to_string(), reflect_branch_arm(dag, &arms.second)?),
        ("rest".to_string(), rest_spine),
    ]))
}

fn reflect_branch_arm(dag: &Dag, arm: &BranchArm) -> ReflectResult<FieldValue> {
    Ok(FieldValue::Record(vec![
        (
            "condition".to_string(),
            reflect_bool_port_ref(dag, arm.bool_port())?,
        ),
        (
            "body".to_string(),
            reflect_workflow_effect(dag, arm.body())?,
        ),
    ]))
}

fn reflect_workflow_effect_vec_spine(
    dag: &Dag,
    items: &[Box<WorkflowEffect>],
) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    let mut tail = FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    };
    for item in items.iter().rev() {
        tail = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![reflect_workflow_effect(dag, item.as_ref())?, tail],
        };
    }
    Ok(tail)
}

fn reflect_non_singleton_workflow_branches(
    dag: &Dag,
    branches: &NonSingletonList<Box<WorkflowEffect>>,
) -> ReflectResult<FieldValue> {
    let rest_spine = reflect_workflow_effect_vec_spine(dag, &branches.rest)?;
    Ok(FieldValue::Record(vec![
        (
            "first".to_string(),
            reflect_workflow_effect(dag, branches.first.as_ref())?,
        ),
        (
            "second".to_string(),
            reflect_workflow_effect(dag, branches.second.as_ref())?,
        ),
        ("rest".to_string(), rest_spine),
    ]))
}

fn reflect_optional_workflow_effect(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
    opt: Option<&WorkflowEffect>,
) -> ReflectResult<FieldValue> {
    reflect_optional_sum(dag, cardinality_decl_id, opt.cloned(), |d, w| {
        reflect_workflow_effect(d, &w)
    })
}

fn reflect_optional_branch_emit(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
    opt: Option<BranchEmitParticipation>,
) -> ReflectResult<FieldValue> {
    reflect_optional_sum(dag, cardinality_decl_id, opt, |d, p| match p {
        BranchEmitParticipation::UserMatch => {
            reflect_unit_variant(d, "BranchEmitParticipation", "UserMatch")
        }
    })
}

fn reflect_optional_bind_emit(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
    opt: Option<BindEmitParticipation>,
) -> ReflectResult<FieldValue> {
    reflect_optional_sum(dag, cardinality_decl_id, opt, |d, p| match p {
        BindEmitParticipation::UserCallable => {
            reflect_unit_variant(d, "BindEmitParticipation", "UserCallable")
        }
    })
}

fn reflect_transform_target(dag: &Dag, t: &TransformTarget) -> ReflectResult<FieldValue> {
    match t {
        TransformTarget::Callable(callee) => sum_variant_payload(
            dag,
            "TransformTarget",
            "Callable",
            vec![FieldValue::Reference(*callee)],
        ),
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            let fp_conj = disj_variant_ty(dag, "TransformTarget", "FieldProject")?;
            let field_child_card = peel_to_optional_cardinality_decl(
                dag,
                conj_field_ty(dag, fp_conj, "field_child")?,
            )?;
            sum_variant_payload(
                dag,
                "TransformTarget",
                "FieldProject",
                vec![
                    FieldValue::Literal(LiteralBits::String(field_label.clone())),
                    reflect_optional_declaration_id(dag, field_child_card, *field_child)?,
                ],
            )
        }
        TransformTarget::Operator(op) => sum_variant_payload(
            dag,
            "TransformTarget",
            "Operator",
            vec![reflect_operator_kind(dag, op)?],
        ),
    }
}

fn reflect_branch_pattern(dag: &Dag, p: &BranchPattern) -> ReflectResult<FieldValue> {
    match p {
        BranchPattern::UnresolvedVariant { name, span } => sum_variant_payload(
            dag,
            "BranchPattern",
            "UnresolvedVariant",
            vec![
                FieldValue::Literal(LiteralBits::String(name.clone())),
                reflect_source_span(span),
            ],
        ),
        BranchPattern::ResolvedVariant(decl) => sum_variant_payload(
            dag,
            "BranchPattern",
            "ResolvedVariant",
            vec![FieldValue::Reference(*decl)],
        ),
    }
}

fn reflect_optional_payload_binding(
    dag: &Dag,
    opt: Option<&PayloadBinding>,
) -> ReflectResult<FieldValue> {
    let path = named_record_type_root(dag, "BranchPath")?;
    let binding_card =
        peel_to_optional_cardinality_decl(dag, conj_field_ty(dag, path, "binding")?)?;
    reflect_optional_sum(dag, binding_card, opt.cloned(), |_d, b| {
        Ok(FieldValue::Record(vec![
            (
                "binding_name".to_string(),
                FieldValue::Literal(LiteralBits::String(b.binding_name.clone())),
            ),
            ("payload_port".to_string(), port_fv(b.payload_port)),
        ]))
    })
}

fn reflect_branch_path(dag: &Dag, p: &Path) -> ReflectResult<FieldValue> {
    Ok(FieldValue::Record(vec![
        ("body".to_string(), node_fv(p.body)),
        ("result_port".to_string(), port_fv(p.result_port())),
        (
            "pattern".to_string(),
            reflect_branch_pattern(dag, &p.pattern)?,
        ),
        (
            "binding".to_string(),
            reflect_optional_payload_binding(dag, p.binding.as_ref())?,
        ),
    ]))
}

fn reflect_branch_paths(dag: &Dag, paths: &[Path]) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    let mut tail = FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    };
    for p in paths.iter().rev() {
        let head = reflect_branch_path(dag, p)?;
        tail = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![head, tail],
        };
    }
    Ok(tail)
}

fn reflect_loop_bound(dag: &Dag, b: &LoopBound) -> ReflectResult<FieldValue> {
    match b {
        LoopBound::Cardinality { count } => {
            sum_variant_payload(dag, "LoopBound", "Cardinality", vec![port_fv(*count)])
        }
        LoopBound::Descent { cluster, measure } => sum_variant_payload(
            dag,
            "LoopBound",
            "Descent",
            vec![cluster_fv(*cluster), port_fv(*measure)],
        ),
    }
}

fn behavior_variant_id(dag: &Dag, label: &str) -> ReflectResult<DeclarationId> {
    disj_variant_ty(dag, "Behavior", label)
}

pub fn reflect_behavior(dag: &Dag, behavior: &Behavior) -> ReflectResult<FieldValue> {
    match behavior {
        Behavior::Value(v) => reflect_value(dag, v),
        Behavior::Transform(t) => reflect_transform(dag, t),
        Behavior::Branch(b) => reflect_branch(dag, b),
        Behavior::Loop(l) => reflect_loop(dag, l),
        Behavior::Bind(b) => reflect_bind(dag, b),
    }
}

fn reflect_value(dag: &Dag, v: &ValueNode) -> ReflectResult<FieldValue> {
    let id = behavior_variant_id(dag, "Value")?;
    let vn = named_record_type_root(dag, "ValueNode")?;
    let lane2_card =
        peel_to_optional_cardinality_decl(dag, conj_field_ty(dag, vn, "lane2_workflow")?)?;
    let lane2 = reflect_optional_workflow_effect(dag, lane2_card, v.lane2_workflow())?;
    let payload = FieldValue::Record(vec![
        ("id".to_string(), node_fv(v.id)),
        ("payload".to_string(), FieldValue::Literal(v.data.clone())),
        ("result_port".to_string(), port_fv(v.output)),
        ("span".to_string(), reflect_source_span(&v.span)),
        ("lane2_workflow".to_string(), lane2),
    ]);
    Ok(FieldValue::Variant {
        constructor: id,
        payload: vec![payload],
    })
}

fn reflect_transform(dag: &Dag, t: &TransformNode) -> ReflectResult<FieldValue> {
    let id = behavior_variant_id(dag, "Transform")?;
    let inputs = reflect_port_id_list(dag, &t.inputs)?;
    let payload = FieldValue::Record(vec![
        ("id".to_string(), node_fv(t.id)),
        (
            "target".to_string(),
            reflect_transform_target(dag, &t.target)?,
        ),
        ("inputs".to_string(), inputs),
        ("result_port".to_string(), port_fv(t.output)),
        ("span".to_string(), reflect_source_span(&t.span)),
    ]);
    Ok(FieldValue::Variant {
        constructor: id,
        payload: vec![payload],
    })
}

fn reflect_branch(dag: &Dag, b: &BranchNode) -> ReflectResult<FieldValue> {
    let id = behavior_variant_id(dag, "Branch")?;
    let paths = reflect_branch_paths(dag, &b.paths)?;
    let br = named_record_type_root(dag, "BranchNode")?;
    let emit_card =
        peel_to_optional_cardinality_decl(dag, conj_field_ty(dag, br, "emit_participation")?)?;
    let emit = reflect_optional_branch_emit(dag, emit_card, b.emit_participation())?;
    let payload = FieldValue::Record(vec![
        ("id".to_string(), node_fv(b.id)),
        ("input".to_string(), port_fv(b.input)),
        ("paths".to_string(), paths),
        ("result_port".to_string(), port_fv(b.output)),
        ("span".to_string(), reflect_source_span(&b.span)),
        ("emit_participation".to_string(), emit),
    ]);
    Ok(FieldValue::Variant {
        constructor: id,
        payload: vec![payload],
    })
}

fn reflect_loop(dag: &Dag, l: &LoopNode) -> ReflectResult<FieldValue> {
    let id = behavior_variant_id(dag, "Loop")?;
    let bound = reflect_loop_bound(dag, &l.bound)?;
    let payload = FieldValue::Record(vec![
        ("id".to_string(), node_fv(l.id)),
        ("source".to_string(), port_fv(l.source)),
        ("init".to_string(), port_fv(l.init)),
        ("body".to_string(), node_fv(l.body)),
        ("bound".to_string(), bound),
        ("result_port".to_string(), port_fv(l.output)),
        ("span".to_string(), reflect_source_span(&l.span)),
    ]);
    Ok(FieldValue::Variant {
        constructor: id,
        payload: vec![payload],
    })
}

fn reflect_bind(dag: &Dag, b: &BindNode) -> ReflectResult<FieldValue> {
    let id = behavior_variant_id(dag, "Bind")?;
    let params = reflect_port_id_list(dag, &b.params)?;
    let bn = named_record_type_root(dag, "BindNode")?;
    let lane2_card =
        peel_to_optional_cardinality_decl(dag, conj_field_ty(dag, bn, "lane2_workflow")?)?;
    let emit_card =
        peel_to_optional_cardinality_decl(dag, conj_field_ty(dag, bn, "emit_participation")?)?;
    let lane2 = reflect_optional_workflow_effect(dag, lane2_card, b.lane2_workflow())?;
    let emit = reflect_optional_bind_emit(dag, emit_card, b.emit_participation())?;
    let payload = FieldValue::Record(vec![
        ("id".to_string(), node_fv(b.id)),
        (
            "name".to_string(),
            FieldValue::Literal(LiteralBits::String(b.name.clone())),
        ),
        ("result_port".to_string(), port_fv(b.value)),
        ("params".to_string(), params),
        ("span".to_string(), reflect_source_span(&b.span)),
        ("lane2_workflow".to_string(), lane2),
        ("emit_participation".to_string(), emit),
    ]);
    Ok(FieldValue::Variant {
        constructor: id,
        payload: vec![payload],
    })
}

pub fn reflect_behavior_list(dag: &Dag, nodes: &[Behavior]) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    let mut tail = FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    };
    for behavior in nodes.iter().rev() {
        let head = reflect_behavior(dag, behavior)?;
        tail = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![head, tail],
        };
    }
    Ok(tail)
}

#[cfg(test)]
mod reflection_tests {
    use super::*;
    use crate::compile_to_dag;
    use crate::dag::{
        Behavior, DeclarationId, FieldValue, LiteralBits, LoopBound, TransformTarget,
        TypeConnective,
    };

    fn compile(src: &str, file: &str) -> Dag {
        match compile_to_dag(src, file) {
            Ok(d) => {
                assert!(d.diagnostics().is_empty(), "{file}: {:?}", d.diagnostics());
                d
            }
            Err(e) => panic!("compile {file}: {e:?}"),
        }
    }

    fn behavior_inner_record(fv: &FieldValue) -> &[(String, FieldValue)] {
        let FieldValue::Variant { payload, .. } = fv else {
            panic!("expected Behavior variant, got {fv:?}");
        };
        assert_eq!(payload.len(), 1, "Behavior variant payload");
        let FieldValue::Record(fields) = &payload[0] else {
            panic!("expected inner record");
        };
        fields.as_slice()
    }

    fn record_get<'a>(rec: &'a [(String, FieldValue)], key: &str) -> &'a FieldValue {
        rec.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("missing field `{key}` in {rec:?}"))
    }

    fn list_spine_len(dag: &Dag, list: &FieldValue) -> usize {
        let (empty_id, _) = v3_list_empty_cons_ids(dag).expect("List ids");
        let mut n = 0;
        let mut cur = list;
        loop {
            let FieldValue::Variant {
                constructor,
                payload,
            } = cur
            else {
                panic!("expected List spine");
            };
            if *constructor == empty_id {
                break;
            }
            assert_eq!(payload.len(), 2, "Cons payload");
            n += 1;
            cur = &payload[1];
        }
        n
    }

    fn loop_bound_variant_label(dag: &Dag, variant_constructor_ty: DeclarationId) -> String {
        let decl = dag
            .declaration_by_name("LoopBound")
            .expect("LoopBound declaration");
        let TypeConnective::Disj { variants } = &decl.connective else {
            panic!("LoopBound not a Disj");
        };
        variants
            .iter()
            .find(|v| v.ty == variant_constructor_ty)
            .map(|v| v.label.clone())
            .unwrap_or_else(|| {
                panic!("constructor {variant_constructor_ty:?} not a LoopBound variant");
            })
    }

    fn optional_carrier_variant_label(dag: &Dag, fv: &FieldValue) -> String {
        let FieldValue::Variant { constructor, .. } = fv else {
            panic!("expected optional carrier variant, got {fv:?}");
        };
        dag.declarations()
            .iter()
            .find_map(|decl| match &decl.connective {
                TypeConnective::Disj { variants } => variants
                    .iter()
                    .find(|v| v.ty == *constructor)
                    .map(|v| v.label.clone()),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!("constructor {constructor:?} not a sum variant payload ty")
            })
    }

    /// Peel empty `Instantiation` wrappers to the underlying `Conj` (same contract as
    /// `substrate_reflection::named_record_type_root`).
    fn named_conj_root_id(dag: &Dag, substrate_record_name: &str) -> DeclarationId {
        let mut decl_id = dag
            .declaration_by_name(substrate_record_name)
            .unwrap_or_else(|| panic!("missing declaration `{substrate_record_name}`"))
            .id;
        const PEEL_MAX: usize = 64;
        for _ in 0..PEEL_MAX {
            match &dag.declaration(decl_id).connective {
                TypeConnective::Conj { .. } => return decl_id,
                TypeConnective::Instantiation {
                    template,
                    arguments,
                } if arguments.is_empty() => {
                    decl_id = *template;
                }
                _ => panic!("`{substrate_record_name}` did not peel to a Conj"),
            }
        }
        panic!("peel depth exceeded for `{substrate_record_name}`");
    }

    /// Ratchet: reflected `Record` field **order and labels** match the named substrate `Conj`.
    fn assert_record_matches_named_substrate_conj(
        dag: &Dag,
        rec: &[(String, FieldValue)],
        substrate_record_name: &str,
    ) {
        let conj_id = named_conj_root_id(dag, substrate_record_name);
        let TypeConnective::Conj { children } = &dag.declaration(conj_id).connective else {
            panic!("`{substrate_record_name}` not a Conj after peel");
        };
        assert_eq!(
            rec.len(),
            children.len(),
            "`{substrate_record_name}`: reflected field count vs declared Conj arity"
        );
        for (i, ((ref_label, _), decl_field)) in rec.iter().zip(children.iter()).enumerate() {
            assert_eq!(
                ref_label, &decl_field.label,
                "`{substrate_record_name}`: field {i} label mismatch (reflection vs substrate.dag)"
            );
        }
    }

    /// Ratchet: `FieldValue::Variant` payload slot count matches the declared variant payload
    /// shape (`Conj` children when the constructor is a record; otherwise 0–1 like
    /// `sum_variant_payload`).
    fn assert_sum_variant_payload_matches_substrate(
        dag: &Dag,
        sum_decl_name: &str,
        fv: &FieldValue,
    ) {
        let FieldValue::Variant {
            constructor,
            payload,
        } = fv
        else {
            panic!("expected sum variant, got {fv:?}");
        };
        let ctor_ty = *constructor;
        let ctor_decl = dag.declaration(ctor_ty);
        let variant_label = {
            let sum_decl = dag
                .declaration_by_name(sum_decl_name)
                .unwrap_or_else(|| panic!("missing sum `{sum_decl_name}`"));
            let TypeConnective::Disj { variants } = &sum_decl.connective else {
                panic!("`{sum_decl_name}` not a Disj");
            };
            variants
                .iter()
                .find(|v| v.ty == ctor_ty)
                .map(|v| v.label.as_str())
                .unwrap_or_else(|| {
                    panic!("constructor {ctor_ty:?} is not a variant of `{sum_decl_name}`")
                })
        };
        match &ctor_decl.connective {
            TypeConnective::Conj { children } => {
                assert_eq!(
                    payload.len(),
                    children.len(),
                    "`{sum_decl_name}`::{variant_label}: payload arity vs declared Conj"
                );
            }
            _ => {
                assert!(
                    payload.len() <= 1,
                    "`{sum_decl_name}`::{variant_label}: non-Conj constructor allows at most one payload; got {}",
                    payload.len()
                );
            }
        }
    }

    #[test]
    fn reflection_optional_fields_use_some_none_not_list() {
        let src = "let x: Int = 7\n";
        let file = "reflect_optional_carrier_value.v3";
        let dag = compile(src, file);
        let v = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Value(v) if v.span.file == file => Some(v),
                _ => None,
            })
            .expect("Value node");
        let fv = reflect_behavior(&dag, &Behavior::Value(v.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        let lane2 = record_get(rec, "lane2_workflow");
        assert_eq!(optional_carrier_variant_label(&dag, lane2), "None");

        let src_bind = "fn g(x: Int) -> Int = x + 1\n";
        let file_bind = "reflect_optional_carrier_bind.v3";
        let dag_b = compile(src_bind, file_bind);
        let b = dag_b
            .nodes()
            .iter()
            .find_map(|beh| match beh {
                Behavior::Bind(b) if b.span.file == file_bind && !b.params.is_empty() => {
                    Some(b)
                }
                _ => None,
            })
            .expect("Bind node");
        let fv_b = reflect_behavior(&dag_b, &Behavior::Bind(b.clone())).expect("reflect bind");
        let rec_b = behavior_inner_record(&fv_b);
        let emit = record_get(rec_b, "emit_participation");
        assert_eq!(optional_carrier_variant_label(&dag_b, emit), "Some");
    }

    #[test]
    fn reflection_value_includes_all_substrate_fields() {
        let src = "let x: Int = 7\n";
        let file = "reflect_value.v3";
        let dag = compile(src, file);
        let v = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Value(v) if v.span.file == file => Some(v),
                _ => None,
            })
            .expect("Value node");
        let fv = reflect_behavior(&dag, &Behavior::Value(v.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        for key in ["id", "payload", "result_port", "span", "lane2_workflow"] {
            let _ = record_get(rec, key);
        }
    }

    #[test]
    fn reflection_transform_includes_target_inputs_and_span() {
        let src = "fn f(a: Int, b: Int) -> Int = a + b\n";
        let file = "reflect_transform.v3";
        let dag = compile(src, file);
        let t = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Transform(t) if t.span.file == file => Some(t),
                _ => None,
            })
            .expect("Transform node");
        let fv = reflect_behavior(&dag, &Behavior::Transform(t.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        for key in ["id", "target", "inputs", "result_port", "span"] {
            let _ = record_get(rec, key);
        }
    }

    #[test]
    fn reflection_bind_includes_params_lane2_emit() {
        let src = "fn g(x: Int) -> Int = x + 1\n";
        let file = "reflect_bind.v3";
        let dag = compile(src, file);
        let b = dag
            .nodes()
            .iter()
            .find_map(|beh| match beh {
                Behavior::Bind(b) if b.span.file == file && !b.params.is_empty() => Some(b),
                _ => None,
            })
            .expect("Bind node");
        let fv = reflect_behavior(&dag, &Behavior::Bind(b.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        for key in [
            "id",
            "name",
            "result_port",
            "params",
            "span",
            "lane2_workflow",
            "emit_participation",
        ] {
            let _ = record_get(rec, key);
        }
    }

    #[test]
    fn reflection_branch_three_arms_totality() {
        let src = "\
type Color = Red | Green | Blue
fn classify(h: Color) -> Int = match h { Red => 0, Green => 1, Blue => 2 }
";
        let file = "reflect_branch_three.v3";
        let dag = compile(src, file);
        let br = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Branch(br) if br.span.file == file => Some(br),
                _ => None,
            })
            .expect("Branch");
        assert_eq!(br.paths.len(), 3);
        let fv = reflect_behavior(&dag, &Behavior::Branch(br.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        assert_record_matches_named_substrate_conj(&dag, rec, "BranchNode");
        let paths = record_get(rec, "paths");
        assert_eq!(list_spine_len(&dag, paths), 3);
        let FieldValue::Variant { payload, .. } = paths else {
            panic!("paths list");
        };
        assert_eq!(payload.len(), 2);
        let FieldValue::Record(arm_fields) = &payload[0] else {
            panic!("first arm record");
        };
        for key in ["body", "result_port", "pattern", "binding"] {
            let _ = record_get(arm_fields, key);
        }
    }

    #[test]
    fn reflection_loop_bound_coproduct_cardinality_vs_descent() {
        let src_single = "\
fn count(n: Int) -> Int = if n == 0 then 0 else 1 + count(n - 1)
let _: Int = count(1)
";
        let file_c = "reflect_loop_card.v3";
        let dag_c = compile(src_single, file_c);
        let lp_c = dag_c
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Loop(l) if l.span.file == file_c => Some(l),
                _ => None,
            })
            .expect("cardinality loop");
        assert!(matches!(lp_c.bound, LoopBound::Cardinality { .. }));
        let fv_c = reflect_behavior(&dag_c, &Behavior::Loop(lp_c.clone())).expect("reflect");
        let rec_c = behavior_inner_record(&fv_c);
        assert_record_matches_named_substrate_conj(&dag_c, rec_c, "LoopNode");
        let bound_c = record_get(rec_c, "bound");
        let FieldValue::Variant {
            constructor: c_ty, ..
        } = bound_c
        else {
            panic!("LoopBound variant");
        };
        assert_eq!(loop_bound_variant_label(&dag_c, *c_ty), "Cardinality");

        let src_mutual = "\
fn even(n: Int) -> Bool = if n == 0 then true else odd(n - 1)
fn odd(n: Int) -> Bool = if n == 0 then false else even(n - 1)
";
        let file_d = "reflect_loop_desc.v3";
        let dag_d = compile(src_mutual, file_d);
        let lp_d = dag_d
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Loop(l)
                    if l.span.file == file_d
                        && matches!(l.bound, LoopBound::Descent { .. }) =>
                {
                    Some(l)
                }
                _ => None,
            })
            .expect("descent loop");
        let fv_d = reflect_behavior(&dag_d, &Behavior::Loop(lp_d.clone())).expect("reflect");
        let rec_d = behavior_inner_record(&fv_d);
        assert_record_matches_named_substrate_conj(&dag_d, rec_d, "LoopNode");
        let bound_d = record_get(rec_d, "bound");
        let FieldValue::Variant {
            constructor: d_ty, ..
        } = bound_d
        else {
            panic!("LoopBound variant");
        };
        assert_eq!(loop_bound_variant_label(&dag_d, *d_ty), "Descent");
    }

    #[test]
    fn reflection_value_inner_record_matches_value_node_conj_decl() {
        let src = "let x: Int = 7\n";
        let file = "reflect_schema_value.v3";
        let dag = compile(src, file);
        let v = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Value(v) if v.span.file == file => Some(v),
                _ => None,
            })
            .expect("Value node");
        let fv = reflect_behavior(&dag, &Behavior::Value(v.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        assert_record_matches_named_substrate_conj(&dag, rec, "ValueNode");
    }

    #[test]
    fn reflection_source_span_record_matches_types_dag_conj_decl() {
        let src = "let x: Int = 7\n";
        let file = "reflect_schema_source_span.v3";
        let dag = compile(src, file);
        let v = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Value(v) if v.span.file == file => Some(v),
                _ => None,
            })
            .expect("Value node");
        let fv = reflect_behavior(&dag, &Behavior::Value(v.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        let span_rec = match record_get(rec, "span") {
            FieldValue::Record(fields) => fields.as_slice(),
            other => panic!("expected span record, got {other:?}"),
        };
        assert_record_matches_named_substrate_conj(&dag, span_rec, "SourceSpan");
        assert_eq!(
            record_get(span_rec, "start"),
            &FieldValue::Literal(LiteralBits::Int(i64::from(v.span.byte_start)))
        );
        assert_eq!(
            record_get(span_rec, "end"),
            &FieldValue::Literal(LiteralBits::Int(i64::from(v.span.byte_end)))
        );
    }

    #[test]
    fn reflection_transform_inner_record_matches_transform_node_conj_decl() {
        let src = "fn f(a: Int, b: Int) -> Int = a + b\n";
        let file = "reflect_schema_transform.v3";
        let dag = compile(src, file);
        let t = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Transform(t) if t.span.file == file => Some(t),
                _ => None,
            })
            .expect("Transform node");
        let fv = reflect_behavior(&dag, &Behavior::Transform(t.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        assert_record_matches_named_substrate_conj(&dag, rec, "TransformNode");
    }

    #[test]
    fn reflection_bind_inner_record_matches_bind_node_conj_decl() {
        let src = "fn g(x: Int) -> Int = x + 1\n";
        let file = "reflect_schema_bind.v3";
        let dag = compile(src, file);
        let b = dag
            .nodes()
            .iter()
            .find_map(|beh| match beh {
                Behavior::Bind(b) if b.span.file == file && !b.params.is_empty() => Some(b),
                _ => None,
            })
            .expect("Bind node");
        let fv = reflect_behavior(&dag, &Behavior::Bind(b.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        assert_record_matches_named_substrate_conj(&dag, rec, "BindNode");
    }

    #[test]
    fn reflection_transform_target_callable_variant_matches_substrate_payload_shape() {
        let src = "fn id(x: Int) -> Int = x\nlet _: Int = id(1)\n";
        let file = "reflect_schema_callable.v3";
        let dag = compile(src, file);
        let t = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Transform(t) if t.span.file == file => match &t.target {
                    TransformTarget::Callable(_) => Some(t),
                    _ => None,
                },
                _ => None,
            })
            .expect("Callable transform");
        let fv = reflect_behavior(&dag, &Behavior::Transform(t.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        let target = record_get(rec, "target");
        assert_sum_variant_payload_matches_substrate(&dag, "TransformTarget", target);
    }

    #[test]
    fn reflection_transform_target_field_project_variant_matches_substrate_payload_shape() {
        let src = "\
type Point { x: Int y: Int }
fn get_x(point: Point) -> Int = point.x
";
        let file = "reflect_schema_field_proj.v3";
        let dag = compile(src, file);
        let t = dag
            .nodes()
            .iter()
            .find_map(|b| match b {
                Behavior::Transform(t) if t.span.file == file => match &t.target {
                    TransformTarget::FieldProject { .. } => Some(t),
                    _ => None,
                },
                _ => None,
            })
            .expect("FieldProject transform");
        let fv = reflect_behavior(&dag, &Behavior::Transform(t.clone())).expect("reflect");
        let rec = behavior_inner_record(&fv);
        let target = record_get(rec, "target");
        assert_sum_variant_payload_matches_substrate(&dag, "TransformTarget", target);
    }
}
