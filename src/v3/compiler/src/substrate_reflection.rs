//! Complete structural reflection of computation-substrate [`Behavior`] nodes into
//! lens-input [`FieldValue`] per `docs/design-reflection-completeness.md` (LOCKED).
//! Reflection is static: no execution, no branch-arm selection, no loop iteration.

use crate::dag::{
    Behavior, BindEmitParticipation, BindNode, BoolPortRef, BranchArm, BranchEmitParticipation,
    BranchNode, BranchPattern, BreakingShape, ClusterId, CreateCause, Dag, DeclarationId,
    EffectShape, ElementRef, FieldValue, HttpMethodScalar, IdempotentShape, KeySource,
    LiteralBits, LoopBound, LoopNode, NonSingletonList, OperationEffect, OperatorKind, Path,
    PortId, SourceSpan, TransformNode, TransformTarget, TypeConnective, ValueNode,
    WorkflowEffect,
};
use crate::dag::{NodeId, TransformRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectError(pub &'static str);

pub type ReflectResult<T> = Result<T, ReflectError>;

fn err<T>(msg: &'static str) -> ReflectResult<T> {
    Err(ReflectError(msg))
}

fn disj_variant_ty(dag: &Dag, sum_name: &str, variant_label: &str) -> ReflectResult<DeclarationId> {
    let decl = dag
        .declaration_by_name(sum_name)
        .ok_or(ReflectError("missing sum type"))?;
    let TypeConnective::Disj { variants } = &decl.connective else {
        return err("sum type is not Disj");
    };
    variants
        .iter()
        .find(|v| v.label == variant_label)
        .map(|v| v.ty)
        .ok_or(ReflectError("missing sum variant"))
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
    FieldValue::Record(vec![
        ("file".to_string(), FieldValue::Literal(LiteralBits::String(span.file.clone()))),
        (
            "byte_start".to_string(),
            FieldValue::Literal(LiteralBits::Int(i64::from(span.byte_start))),
        ),
        (
            "byte_end".to_string(),
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

/// Optional `T?` as the substrate `List` sum (`Empty` | `Cons`) — two-variant present/absent
/// carrier per design-reflection-completeness §4.1 (not a bool flag).
fn reflect_optional_list_spine<T>(
    dag: &Dag,
    opt: Option<T>,
    mut reflect_some: impl FnMut(&Dag, T) -> ReflectResult<FieldValue>,
) -> ReflectResult<FieldValue> {
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    Ok(match opt {
        None => FieldValue::Variant {
            constructor: empty_id,
            payload: vec![],
        },
        Some(x) => FieldValue::Variant {
            constructor: cons_id,
            payload: vec![
                reflect_some(dag, x)?,
                FieldValue::Variant {
                    constructor: empty_id,
                    payload: vec![],
                },
            ],
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
            payload: vec![
                FieldValue::Literal(LiteralBits::String(s.clone())),
                tail,
            ],
        };
    }
    Ok(tail)
}

fn reflect_optional_declaration_id(
    dag: &Dag,
    opt: Option<DeclarationId>,
) -> ReflectResult<FieldValue> {
    reflect_optional_list_spine(dag, opt, |d, id| {
        Ok(FieldValue::Reference(id))
    })
}

fn reflect_unit_variant(dag: &Dag, sum_name: &str, label: &str) -> ReflectResult<FieldValue> {
    let id = disj_variant_ty(dag, sum_name, label)?;
    Ok(FieldValue::Variant {
        constructor: id,
        payload: vec![],
    })
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
            let id = disj_variant_ty(dag, "CreateCause", "PostAlways")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![],
            })
        }
        CreateCause::KeylessFallback { method } => {
            let id = disj_variant_ty(dag, "CreateCause", "KeylessFallback")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_http_method_scalar(dag, *method)?],
            })
        }
    }
}

fn reflect_key_source(dag: &Dag, ks: &KeySource) -> ReflectResult<FieldValue> {
    match ks {
        KeySource::PathParam { param } => {
            let id = disj_variant_ty(dag, "KeySource", "PathParam")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Literal(LiteralBits::String(param.clone()))],
            })
        }
        KeySource::InputField { field } => {
            let id = disj_variant_ty(dag, "KeySource", "InputField")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Literal(LiteralBits::String(field.clone()))],
            })
        }
        KeySource::CompositeKey { fields } => {
            let id = disj_variant_ty(dag, "KeySource", "CompositeKey")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_string_list_spine(dag, fields)?],
            })
        }
    }
}

fn reflect_idempotent_shape(dag: &Dag, s: &IdempotentShape) -> ReflectResult<FieldValue> {
    match s {
        IdempotentShape::ReadEffect => {
            let id = disj_variant_ty(dag, "IdempotentShape", "ReadEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![],
            })
        }
        IdempotentShape::UpsertEffect { key_source } => {
            let id = disj_variant_ty(dag, "IdempotentShape", "UpsertEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_key_source(dag, key_source)?],
            })
        }
        IdempotentShape::DeleteEffect { key_source } => {
            let id = disj_variant_ty(dag, "IdempotentShape", "DeleteEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_key_source(dag, key_source)?],
            })
        }
    }
}

fn reflect_breaking_shape(dag: &Dag, s: &BreakingShape) -> ReflectResult<FieldValue> {
    match s {
        BreakingShape::CreateEffect { cause } => {
            let id = disj_variant_ty(dag, "BreakingShape", "CreateEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_create_cause(dag, cause)?],
            })
        }
        BreakingShape::AppendEffect => {
            let id = disj_variant_ty(dag, "BreakingShape", "AppendEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![],
            })
        }
    }
}

fn reflect_effect_shape(dag: &Dag, s: &EffectShape) -> ReflectResult<FieldValue> {
    match s {
        EffectShape::IsIdempotent(inner) => {
            let id = disj_variant_ty(dag, "EffectShape", "IsIdempotent")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_idempotent_shape(dag, inner)?],
            })
        }
        EffectShape::IsBreaking(inner) => {
            let id = disj_variant_ty(dag, "EffectShape", "IsBreaking")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_breaking_shape(dag, inner)?],
            })
        }
    }
}

fn reflect_element_ref_operation_effect(
    dag: &Dag,
    r: ElementRef<OperationEffect>,
) -> ReflectResult<FieldValue> {
    Ok(FieldValue::Record(vec![(
        "index".to_string(),
        FieldValue::Literal(LiteralBits::Int(i64::try_from(r.index_of()).map_err(|_| {
            ReflectError("ElementRef index overflow")
        })?)),
    )]))
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

fn reflect_operation_effect_vec_spine(dag: &Dag, ops: &[OperationEffect]) -> ReflectResult<FieldValue> {
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

fn reflect_bool_port_ref(dag: &Dag, r: BoolPortRef) -> ReflectResult<FieldValue> {
    Ok(FieldValue::Record(vec![(
        "port".to_string(),
        port_fv(r.port_id()),
    )]))
}

fn reflect_transform_ref(dag: &Dag, r: TransformRef) -> ReflectResult<FieldValue> {
    Ok(FieldValue::Record(vec![(
        "node".to_string(),
        node_fv(r.node_id()),
    )]))
}

fn reflect_workflow_effect(dag: &Dag, wf: &WorkflowEffect) -> ReflectResult<FieldValue> {
    match wf {
        WorkflowEffect::LinearEffect { ops } => {
            let id = disj_variant_ty(dag, "WorkflowEffect", "LinearEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_operation_effect_vec_spine(dag, ops)?],
            })
        }
        WorkflowEffect::BranchEffect { arms } => {
            let id = disj_variant_ty(dag, "WorkflowEffect", "BranchEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_non_singleton_branch_arms(dag, arms)?],
            })
        }
        WorkflowEffect::LoopEffect { body } => {
            let id = disj_variant_ty(dag, "WorkflowEffect", "LoopEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_workflow_effect(dag, body)?],
            })
        }
        WorkflowEffect::ParallelEffect { branches } => {
            let id = disj_variant_ty(dag, "WorkflowEffect", "ParallelEffect")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_non_singleton_workflow_branches(dag, branches)?],
            })
        }
    }
}

fn reflect_non_singleton_branch_arms(
    dag: &Dag,
    arms: &NonSingletonList<BranchArm>,
) -> ReflectResult<FieldValue> {
    let mut rest_spine = reflect_operation_effect_vec_spine(dag, &[])?;
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    for arm in arms.rest.iter().rev() {
        let b = reflect_branch_arm(dag, arm)?;
        rest_spine = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![b, rest_spine],
        };
    }
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

fn reflect_non_singleton_workflow_branches(
    dag: &Dag,
    branches: &NonSingletonList<Box<WorkflowEffect>>,
) -> ReflectResult<FieldValue> {
    let mut rest_spine = {
        let (empty_id, _cons_id) = v3_list_empty_cons_ids(dag)?;
        FieldValue::Variant {
            constructor: empty_id,
            payload: vec![],
        }
    };
    let (empty_id, cons_id) = v3_list_empty_cons_ids(dag)?;
    for b in branches.rest.iter().rev() {
        let head = reflect_workflow_effect(dag, b.as_ref())?;
        rest_spine = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![head, rest_spine],
        };
    }
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
    opt: Option<&WorkflowEffect>,
) -> ReflectResult<FieldValue> {
    reflect_optional_list_spine(dag, opt.map(|w| w.clone()), |d, w| reflect_workflow_effect(d, &w))
}

fn reflect_optional_branch_emit(
    dag: &Dag,
    opt: Option<BranchEmitParticipation>,
) -> ReflectResult<FieldValue> {
    reflect_optional_list_spine(dag, opt, |d, p| match p {
        BranchEmitParticipation::UserMatch => {
            let id = disj_variant_ty(d, "BranchEmitParticipation", "UserMatch")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![],
            })
        }
    })
}

fn reflect_optional_bind_emit(
    dag: &Dag,
    opt: Option<BindEmitParticipation>,
) -> ReflectResult<FieldValue> {
    reflect_optional_list_spine(dag, opt, |d, p| match p {
        BindEmitParticipation::UserCallable => {
            let id = disj_variant_ty(d, "BindEmitParticipation", "UserCallable")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![],
            })
        }
    })
}

fn reflect_transform_target(dag: &Dag, t: &TransformTarget) -> ReflectResult<FieldValue> {
    match t {
        TransformTarget::Callable(callee) => {
            let id = disj_variant_ty(dag, "TransformTarget", "Callable")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Reference(*callee)],
            })
        }
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            let id = disj_variant_ty(dag, "TransformTarget", "FieldProject")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Record(vec![
                    (
                        "field_label".to_string(),
                        FieldValue::Literal(LiteralBits::String(field_label.clone())),
                    ),
                    (
                        "field_child".to_string(),
                        reflect_optional_declaration_id(dag, *field_child)?,
                    ),
                ])],
            })
        }
        TransformTarget::Operator(op) => {
            let id = disj_variant_ty(dag, "TransformTarget", "Operator")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![reflect_operator_kind(dag, op)?],
            })
        }
    }
}

fn reflect_branch_pattern(dag: &Dag, p: &BranchPattern) -> ReflectResult<FieldValue> {
    match p {
        BranchPattern::UnresolvedVariant { name, span } => {
            let id = disj_variant_ty(dag, "BranchPattern", "UnresolvedVariant")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Record(vec![
                    ("name".to_string(), FieldValue::Literal(LiteralBits::String(name.clone()))),
                    ("span".to_string(), reflect_source_span(span)),
                ])],
            })
        }
        BranchPattern::ResolvedVariant(decl) => {
            let id = disj_variant_ty(dag, "BranchPattern", "ResolvedVariant")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Reference(*decl)],
            })
        }
    }
}

fn reflect_optional_payload_binding(
    dag: &Dag,
    opt: Option<&crate::dag::PayloadBinding>,
) -> ReflectResult<FieldValue> {
    reflect_optional_list_spine(dag, opt.cloned(), |d, b| {
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
        ("pattern".to_string(), reflect_branch_pattern(dag, &p.pattern)?),
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
            let id = disj_variant_ty(dag, "LoopBound", "Cardinality")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Record(vec![(
                    "count".to_string(),
                    port_fv(*count),
                )])],
            })
        }
        LoopBound::Descent { cluster } => {
            let id = disj_variant_ty(dag, "LoopBound", "Descent")?;
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![FieldValue::Record(vec![(
                    "cluster".to_string(),
                    cluster_fv(*cluster),
                )])],
            })
        }
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
    let lane2 = reflect_optional_workflow_effect(dag, v.lane2_workflow().map(|w| w))?;
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
        ("target".to_string(), reflect_transform_target(dag, &t.target)?),
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
    let emit = reflect_optional_branch_emit(dag, b.emit_participation())?;
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
    let lane2 = reflect_optional_workflow_effect(dag, b.lane2_workflow().map(|w| w))?;
    let emit = reflect_optional_bind_emit(dag, b.emit_participation())?;
    let payload = FieldValue::Record(vec![
        ("id".to_string(), node_fv(b.id)),
        ("name".to_string(), FieldValue::Literal(LiteralBits::String(b.name.clone()))),
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
