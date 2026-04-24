//! Bounded interpreter for applying user-authored lens functions (T-LensAPI / D1).
//!
//! Walks `ArrowBody::UserDefined` behavior graphs (`Bind` / `Transform` / `Branch` /
//! `Loop`) over substrate-shaped [`FieldValue`] inputs — not pattern-matching on ad-hoc
//! operator shapes for whole claims.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BranchPattern, Dag, DeclarationId, FieldValue,
    LiteralBits, LoopBound, OperatorKind, PortId, TransformTarget, TypeConnective,
    ValueBody,
};

/// Apply `lens_decl` (must name an `Arrow` with `UserDefined` body) from `lens_program`
/// to `input`, returning the result as structural [`FieldValue`] (typically `Int`).
pub fn apply_lens_declaration(
    lens_program: &Dag,
    lens_decl_id: DeclarationId,
    input: &FieldValue,
) -> Result<FieldValue, LensApplyError> {
    let decl = lens_program.declaration(lens_decl_id);
    let TypeConnective::Arrow {
        inputs,
        output: _,
        body,
    } = &decl.connective
    else {
        return Err(LensApplyError::NotAnArrow);
    };
    let ArrowBody::UserDefined(root) = body else {
        return Err(LensApplyError::UnsupportedArrowBody);
    };
    if inputs.len() != 1 {
        return Err(LensApplyError::ArityMismatch {
            expected: 1,
            got: inputs.len(),
        });
    }
    let Behavior::Bind(root_bind) = lens_program.node(*root) else {
        return Err(LensApplyError::MalformedLensRoot);
    };
    if root_bind.params.len() != 1 {
        return Err(LensApplyError::ArityMismatch {
            expected: 1,
            got: root_bind.params.len(),
        });
    }
    let mut env = PortEnv::new();
    env.bind(root_bind.params[0], input.clone());
    eval_port_value(lens_program, &mut env, root_bind.value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LensApplyError {
    NotAnArrow,
    UnsupportedArrowBody,
    ArityMismatch { expected: usize, got: usize },
    MalformedLensRoot,
    UnsupportedConstruct(&'static str),
    TypeMismatch(&'static str),
    UnresolvedPort,
    UnimplementedCallable(String),
    UnimplementedLoopBound,
    BranchMiss,
    BadFieldProject,
    BadListShape,
}

struct PortEnv {
    values: HashMap<u32, FieldValue>,
}

impl PortEnv {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn bind(&mut self, port: PortId, value: FieldValue) {
        self.values.insert(port.raw(), value);
    }

    fn get(&self, port: PortId) -> Option<&FieldValue> {
        self.values.get(&port.raw())
    }
}

fn eval_port_value(
    dag: &Dag,
    env: &mut PortEnv,
    port: PortId,
) -> Result<FieldValue, LensApplyError> {
    if let Some(existing) = env.get(port) {
        return Ok(existing.clone());
    }
    let producer = dag
        .resolve_producer_opt(&port)
        .ok_or(LensApplyError::UnresolvedPort)?;
    let out = match producer {
        Behavior::Value(v) => FieldValue::Literal(v.data.clone()),
        Behavior::Transform(t) => eval_transform(dag, env, t)?,
        Behavior::Branch(b) => eval_branch(dag, env, b, port)?,
        Behavior::Loop(l) => eval_loop(dag, env, l)?,
        Behavior::Bind(b) => {
            if b.params.is_empty() {
                eval_port_value(dag, env, b.value)?
            } else {
                return Err(LensApplyError::UnsupportedConstruct(
                    "nested function bind as producer",
                ));
            }
        }
    };
    env.bind(port, out.clone());
    Ok(out)
}

fn eval_transform(
    dag: &Dag,
    env: &mut PortEnv,
    t: &crate::dag::TransformNode,
) -> Result<FieldValue, LensApplyError> {
    match &t.target {
        TransformTarget::Operator(OperatorKind::Arithmetic(op)) => {
            if t.inputs.len() != 2 {
                return Err(LensApplyError::ArityMismatch {
                    expected: 2,
                    got: t.inputs.len(),
                });
            }
            let a = int_from_value(&eval_port_value(dag, env, t.inputs[0])?)?;
            let b = int_from_value(&eval_port_value(dag, env, t.inputs[1])?)?;
            let n = match op {
                crate::dag::ArithmeticOp::Add => a + b,
                crate::dag::ArithmeticOp::Sub => a - b,
                crate::dag::ArithmeticOp::Mul => a * b,
                crate::dag::ArithmeticOp::Div => {
                    if b == 0 {
                        return Err(LensApplyError::TypeMismatch("division by zero"));
                    }
                    a / b
                }
            };
            Ok(FieldValue::Literal(LiteralBits::Int(n)))
        }
        TransformTarget::Operator(OperatorKind::Comparison(op)) => {
            if t.inputs.len() != 2 {
                return Err(LensApplyError::ArityMismatch {
                    expected: 2,
                    got: t.inputs.len(),
                });
            }
            let a = int_from_value(&eval_port_value(dag, env, t.inputs[0])?)?;
            let b = int_from_value(&eval_port_value(dag, env, t.inputs[1])?)?;
            let out = match op {
                crate::dag::ComparisonOp::Eq => a == b,
                crate::dag::ComparisonOp::Ne => a != b,
                crate::dag::ComparisonOp::Lt => a < b,
                crate::dag::ComparisonOp::Le => a <= b,
                crate::dag::ComparisonOp::Gt => a > b,
                crate::dag::ComparisonOp::Ge => a >= b,
            };
            Ok(bool_value(dag, out))
        }
        TransformTarget::Operator(OperatorKind::Logical(_)) => Err(
            LensApplyError::UnsupportedConstruct("logical operator in lens apply"),
        ),
        TransformTarget::FieldProject {
            field_label,
            field_child: _,
        } => {
            if t.inputs.len() != 1 {
                return Err(LensApplyError::ArityMismatch {
                    expected: 1,
                    got: t.inputs.len(),
                });
            }
            let base = eval_port_value(dag, env, t.inputs[0])?;
            project_field(&base, field_label)
        }
        TransformTarget::Callable(decl_id) => eval_callable(dag, env, *decl_id, &t.inputs),
    }
}

fn eval_callable(
    dag: &Dag,
    env: &mut PortEnv,
    callee: DeclarationId,
    arg_ports: &[PortId],
) -> Result<FieldValue, LensApplyError> {
    if let Some(fold_id) = dag.std_list_fold_decl() {
        if callee == fold_id {
            return eval_std_fold(dag, env, arg_ports);
        }
    }
    let decl = dag.declaration(callee);
    let name = decl.name.clone().unwrap_or_default();
    let TypeConnective::Arrow {
        inputs,
        output: _,
        body,
    } = &decl.connective
    else {
        return Err(LensApplyError::UnimplementedCallable(format!(
            "callee `{}` is not an arrow",
            name
        )));
    };
    let ArrowBody::UserDefined(root) = body else {
        return Err(LensApplyError::UnimplementedCallable(format!(
            "callee `{}` has no UserDefined body (likely std scaffold)",
            name
        )));
    };
    if inputs.len() != arg_ports.len() {
        return Err(LensApplyError::ArityMismatch {
            expected: inputs.len(),
            got: arg_ports.len(),
        });
    }
    let Behavior::Bind(b) = dag.node(*root) else {
        return Err(LensApplyError::MalformedLensRoot);
    };
    if b.params.len() != arg_ports.len() {
        return Err(LensApplyError::ArityMismatch {
            expected: b.params.len(),
            got: arg_ports.len(),
        });
    }
    let mut inner = PortEnv::new();
    for (param, arg_port) in b.params.iter().zip(arg_ports.iter()) {
        let v = eval_port_value(dag, env, *arg_port)?;
        inner.bind(*param, v);
    }
    // Inline callee body ports may reference outer `env`; delegate by
    // temporarily layering: for ports not in inner, fall back to outer.
    eval_port_value_layered(dag, env, &mut inner, b.value)
}

/// Like `eval_port_value` but reads free ports from `outer` after `inner`.
fn eval_port_value_layered(
    dag: &Dag,
    outer: &PortEnv,
    inner: &mut PortEnv,
    port: PortId,
) -> Result<FieldValue, LensApplyError> {
    if let Some(v) = inner.get(port) {
        return Ok(v.clone());
    }
    if let Some(v) = outer.get(port) {
        return Ok(v.clone());
    }
    let producer = dag
        .resolve_producer_opt(&port)
        .ok_or(LensApplyError::UnresolvedPort)?;
    let out = match producer {
        Behavior::Value(v) => FieldValue::Literal(v.data.clone()),
        Behavior::Transform(t) => eval_transform_layered(dag, outer, inner, t)?,
        Behavior::Branch(b) => eval_branch_layered(dag, outer, inner, b, port)?,
        Behavior::Loop(l) => eval_loop_layered(dag, outer, inner, l)?,
        Behavior::Bind(b) => {
            if b.params.is_empty() {
                eval_port_value_layered(dag, outer, inner, b.value)?
            } else {
                return Err(LensApplyError::UnsupportedConstruct(
                    "nested function bind inside callable",
                ));
            }
        }
    };
    inner.bind(port, out.clone());
    Ok(out)
}

fn eval_transform_layered(
    dag: &Dag,
    outer: &PortEnv,
    inner: &mut PortEnv,
    t: &crate::dag::TransformNode,
) -> Result<FieldValue, LensApplyError> {
    match &t.target {
        TransformTarget::Operator(OperatorKind::Arithmetic(op)) => {
            let a = int_from_value(&eval_port_value_layered(dag, outer, inner, t.inputs[0])?)?;
            let b = int_from_value(&eval_port_value_layered(dag, outer, inner, t.inputs[1])?)?;
            let n = match op {
                crate::dag::ArithmeticOp::Add => a + b,
                crate::dag::ArithmeticOp::Sub => a - b,
                crate::dag::ArithmeticOp::Mul => a * b,
                crate::dag::ArithmeticOp::Div => {
                    if b == 0 {
                        return Err(LensApplyError::TypeMismatch("division by zero"));
                    }
                    a / b
                }
            };
            Ok(FieldValue::Literal(LiteralBits::Int(n)))
        }
        TransformTarget::Operator(OperatorKind::Comparison(op)) => {
            let a = int_from_value(&eval_port_value_layered(dag, outer, inner, t.inputs[0])?)?;
            let b = int_from_value(&eval_port_value_layered(dag, outer, inner, t.inputs[1])?)?;
            let out = match op {
                crate::dag::ComparisonOp::Eq => a == b,
                crate::dag::ComparisonOp::Ne => a != b,
                crate::dag::ComparisonOp::Lt => a < b,
                crate::dag::ComparisonOp::Le => a <= b,
                crate::dag::ComparisonOp::Gt => a > b,
                crate::dag::ComparisonOp::Ge => a >= b,
            };
            Ok(bool_value(dag, out))
        }
        TransformTarget::Operator(OperatorKind::Logical(_)) => Err(
            LensApplyError::UnsupportedConstruct("logical operator in lens apply"),
        ),
        TransformTarget::FieldProject {
            field_label,
            field_child: _,
        } => {
            let base = eval_port_value_layered(dag, outer, inner, t.inputs[0])?;
            project_field(&base, field_label)
        }
        TransformTarget::Callable(decl_id) => {
            eval_callable_layered(dag, outer, inner, *decl_id, &t.inputs)
        }
    }
}

fn eval_callable_layered(
    dag: &Dag,
    outer: &PortEnv,
    inner: &mut PortEnv,
    callee: DeclarationId,
    arg_ports: &[PortId],
) -> Result<FieldValue, LensApplyError> {
    if let Some(fold_id) = dag.std_list_fold_decl() {
        if callee == fold_id {
            let list = eval_port_value_layered(dag, outer, inner, arg_ports[0])?;
            let init = eval_port_value_layered(dag, outer, inner, arg_ports[1])?;
            let step_bind_port = arg_ports[2];
            return eval_fold_with_step_port(dag, outer, inner, list, init, step_bind_port);
        }
    }
    let decl = dag.declaration(callee);
    let name = decl.name.clone().unwrap_or_default();
    let TypeConnective::Arrow {
        inputs,
        output: _,
        body,
    } = &decl.connective
    else {
        return Err(LensApplyError::UnimplementedCallable(format!(
            "callee `{}` is not an arrow",
            name
        )));
    };
    let ArrowBody::UserDefined(root) = body else {
        return Err(LensApplyError::UnimplementedCallable(format!(
            "callee `{}` has no UserDefined body",
            name
        )));
    };
    let Behavior::Bind(b) = dag.node(*root) else {
        return Err(LensApplyError::MalformedLensRoot);
    };
    let mut callee_env = PortEnv::new();
    for (param, arg_port) in b.params.iter().zip(arg_ports.iter()) {
        let v = eval_port_value_layered(dag, outer, inner, *arg_port)?;
        callee_env.bind(*param, v);
    }
    eval_port_value_merged(dag, outer, inner, &mut callee_env, b.value)
}

fn eval_port_value_merged(
    dag: &Dag,
    outer: &PortEnv,
    mid: &PortEnv,
    inner: &mut PortEnv,
    port: PortId,
) -> Result<FieldValue, LensApplyError> {
    if let Some(v) = inner.get(port) {
        return Ok(v.clone());
    }
    if let Some(v) = mid.get(port) {
        return Ok(v.clone());
    }
    if let Some(v) = outer.get(port) {
        return Ok(v.clone());
    }
    let producer = dag
        .resolve_producer_opt(&port)
        .ok_or(LensApplyError::UnresolvedPort)?;
    let out = match producer {
        Behavior::Value(v) => FieldValue::Literal(v.data.clone()),
        Behavior::Transform(t) => eval_transform_merged(dag, outer, mid, inner, t)?,
        Behavior::Branch(b) => eval_branch_merged(dag, outer, mid, inner, b, port)?,
        Behavior::Loop(l) => eval_loop_merged(dag, outer, mid, inner, l)?,
        Behavior::Bind(b) => {
            if b.params.is_empty() {
                eval_port_value_merged(dag, outer, mid, inner, b.value)?
            } else {
                return Err(LensApplyError::UnsupportedConstruct(
                    "nested function bind (merged)",
                ));
            }
        }
    };
    inner.bind(port, out.clone());
    Ok(out)
}

fn eval_transform_merged(
    dag: &Dag,
    outer: &PortEnv,
    mid: &PortEnv,
    inner: &mut PortEnv,
    t: &crate::dag::TransformNode,
) -> Result<FieldValue, LensApplyError> {
    match &t.target {
        TransformTarget::Operator(OperatorKind::Arithmetic(op)) => {
            let a = int_from_value(&eval_port_value_merged(
                dag, outer, mid, inner, t.inputs[0],
            )?)?;
            let b = int_from_value(&eval_port_value_merged(
                dag, outer, mid, inner, t.inputs[1],
            )?)?;
            let n = match op {
                crate::dag::ArithmeticOp::Add => a + b,
                crate::dag::ArithmeticOp::Sub => a - b,
                crate::dag::ArithmeticOp::Mul => a * b,
                crate::dag::ArithmeticOp::Div => {
                    if b == 0 {
                        return Err(LensApplyError::TypeMismatch("division by zero"));
                    }
                    a / b
                }
            };
            Ok(FieldValue::Literal(LiteralBits::Int(n)))
        }
        TransformTarget::Operator(OperatorKind::Comparison(op)) => {
            let a = int_from_value(&eval_port_value_merged(
                dag, outer, mid, inner, t.inputs[0],
            )?)?;
            let b = int_from_value(&eval_port_value_merged(
                dag, outer, mid, inner, t.inputs[1],
            )?)?;
            let out = match op {
                crate::dag::ComparisonOp::Eq => a == b,
                crate::dag::ComparisonOp::Ne => a != b,
                crate::dag::ComparisonOp::Lt => a < b,
                crate::dag::ComparisonOp::Le => a <= b,
                crate::dag::ComparisonOp::Gt => a > b,
                crate::dag::ComparisonOp::Ge => a >= b,
            };
            Ok(bool_value(dag, out))
        }
        TransformTarget::Operator(OperatorKind::Logical(_)) => Err(
            LensApplyError::UnsupportedConstruct("logical operator in lens apply"),
        ),
        TransformTarget::FieldProject {
            field_label,
            field_child: _,
        } => {
            let base = eval_port_value_merged(dag, outer, mid, inner, t.inputs[0])?;
            project_field(&base, field_label)
        }
        TransformTarget::Callable(decl_id) => {
            eval_callable_merged(dag, outer, mid, inner, *decl_id, &t.inputs)
        }
    }
}

fn eval_callable_merged(
    dag: &Dag,
    outer: &PortEnv,
    mid: &PortEnv,
    inner: &mut PortEnv,
    callee: DeclarationId,
    arg_ports: &[PortId],
) -> Result<FieldValue, LensApplyError> {
    if let Some(fold_id) = dag.std_list_fold_decl() {
        if callee == fold_id {
            let list = eval_port_value_merged(dag, outer, mid, inner, arg_ports[0])?;
            let init = eval_port_value_merged(dag, outer, mid, inner, arg_ports[1])?;
            let step = arg_ports[2];
            return eval_fold_with_step_port_merged(dag, outer, mid, inner, list, init, step);
        }
    }
    let decl = dag.declaration(callee);
    let name = decl.name.clone().unwrap_or_default();
    let TypeConnective::Arrow {
        inputs,
        output: _,
        body,
    } = &decl.connective
    else {
        return Err(LensApplyError::UnimplementedCallable(format!(
            "callee `{}` is not an arrow",
            name
        )));
    };
    let ArrowBody::UserDefined(root) = body else {
        return Err(LensApplyError::UnimplementedCallable(format!(
            "callee `{}` has no UserDefined body",
            name
        )));
    };
    let Behavior::Bind(b) = dag.node(*root) else {
        return Err(LensApplyError::MalformedLensRoot);
    };
    let mut callee_env = PortEnv::new();
    for (param, arg_port) in b.params.iter().zip(arg_ports.iter()) {
        let v = eval_port_value_merged(dag, outer, mid, inner, *arg_port)?;
        callee_env.bind(*param, v);
    }
    eval_port_value_merged(dag, outer, mid, &mut callee_env, b.value)
}

fn eval_std_fold(
    dag: &Dag,
    env: &mut PortEnv,
    arg_ports: &[PortId],
) -> Result<FieldValue, LensApplyError> {
    if arg_ports.len() != 3 {
        return Err(LensApplyError::ArityMismatch {
            expected: 3,
            got: arg_ports.len(),
        });
    }
    let list = eval_port_value(dag, env, arg_ports[0])?;
    let init = eval_port_value(dag, env, arg_ports[1])?;
    eval_fold_with_step_port(dag, env, &mut PortEnv::new(), list, init, arg_ports[2])
}

fn eval_fold_with_step_port(
    dag: &Dag,
    outer: &PortEnv,
    inner: &mut PortEnv,
    list: FieldValue,
    init: FieldValue,
    step_bind_port: PortId,
) -> Result<FieldValue, LensApplyError> {
    let step_producer = dag
        .resolve_producer_opt(step_bind_port)
        .ok_or(LensApplyError::UnresolvedPort)?;
    let Behavior::Bind(step_bind) = step_producer else {
        return Err(LensApplyError::UnsupportedConstruct(
            "fold step is not a Bind",
        ));
    };
    if step_bind.params.len() != 2 {
        return Err(LensApplyError::ArityMismatch {
            expected: 2,
            got: step_bind.params.len(),
        });
    }
    let mut acc = init;
    for elt in list_elements(&list)? {
        let mut step_env = PortEnv::new();
        step_env.bind(step_bind.params[0], acc);
        step_env.bind(step_bind.params[1], elt);
        acc = eval_port_value_layered(dag, outer, &mut step_env, step_bind.value)?;
    }
    Ok(acc)
}

fn eval_fold_with_step_port_merged(
    dag: &Dag,
    outer: &PortEnv,
    mid: &PortEnv,
    inner: &mut PortEnv,
    list: FieldValue,
    init: FieldValue,
    step_bind_port: PortId,
) -> Result<FieldValue, LensApplyError> {
    let step_producer = dag
        .resolve_producer_opt(step_bind_port)
        .ok_or(LensApplyError::UnresolvedPort)?;
    let Behavior::Bind(step_bind) = step_producer else {
        return Err(LensApplyError::UnsupportedConstruct(
            "fold step is not a Bind",
        ));
    };
    if step_bind.params.len() != 2 {
        return Err(LensApplyError::ArityMismatch {
            expected: 2,
            got: step_bind.params.len(),
        });
    }
    let mut acc = init;
    for elt in list_elements(&list)? {
        let mut step_env = PortEnv::new();
        step_env.bind(step_bind.params[0], acc);
        step_env.bind(step_bind.params[1], elt);
        acc = eval_port_value_merged(dag, outer, mid, &mut step_env, step_bind.value)?;
    }
    Ok(acc)
}

fn list_elements(list: &FieldValue) -> Result<Vec<FieldValue>, LensApplyError> {
    let mut out = Vec::new();
    let mut cur = list;
    loop {
        let (label, payload) = variant_parts(cur)?;
        match label.as_str() {
            "Empty" => break,
            "Cons" => {
                if payload.len() != 2 {
                    return Err(LensApplyError::BadListShape);
                }
                out.push(payload[0].clone());
                cur = &payload[1];
            }
            _ => return Err(LensApplyError::BadListShape),
        }
    }
    Ok(out)
}

fn variant_parts(value: &FieldValue) -> Result<(String, &[FieldValue]), LensApplyError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(LensApplyError::BadListShape);
    };
    let label = constructor_label(*constructor).ok_or(LensApplyError::BadListShape)?;
    Ok((label, payload.as_slice()))
}

fn constructor_label(id: DeclarationId) -> Option<String> {
    let dag = Dag::new();
    dag.declarations().iter().find_map(|decl| match &decl.connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .find(|v| v.ty == id)
            .map(|v| v.label.clone()),
        _ => None,
    })
}

// Fix: constructor_label must use the lens dag, not Dag::new(). This is a bug in the draft.
