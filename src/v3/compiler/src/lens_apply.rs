//! Bounded lens application (T-LensAPI / D1): interpret `ArrowBody::UserDefined` graphs
//! over substrate-shaped [`FieldValue`] — no whole-claim operator recognizers.

use std::collections::HashMap;

use crate::dag::{
    ArrowBody, Behavior, BranchNode, BranchPattern, Dag, DeclarationId, FieldValue,
    LiteralBits, LoopBound, OperatorKind, PortId, TransformNode, TransformTarget,
    TypeConnective,
};

/// Apply a named lens (`Arrow` + `UserDefined` body) from `lens_program` to positional
/// `inputs` (left-to-right with the arrow's formal parameters).
pub fn apply_lens_declaration(
    lens_program: &Dag,
    lens_decl_id: DeclarationId,
    inputs: &[FieldValue],
) -> Result<FieldValue, LensApplyError> {
    let decl = lens_program.declaration(lens_decl_id);
    let TypeConnective::Arrow {
        inputs: param_tys,
        output: _,
        body,
    } = &decl.connective
    else {
        return Err(LensApplyError::NotAnArrow);
    };
    let ArrowBody::UserDefined(root) = body else {
        return Err(LensApplyError::UnsupportedArrowBody);
    };
    if param_tys.len() != inputs.len() {
        return Err(LensApplyError::ArityMismatch {
            expected: param_tys.len(),
            got: inputs.len(),
        });
    }
    let Behavior::Bind(root_bind) = lens_program.node(*root) else {
        return Err(LensApplyError::MalformedLensRoot);
    };
    if root_bind.params.len() != inputs.len() {
        return Err(LensApplyError::ArityMismatch {
            expected: root_bind.params.len(),
            got: inputs.len(),
        });
    }
    let mut ctx = EvalCtx::new(lens_program);
    for (port, arg) in root_bind.params.iter().zip(inputs.iter()) {
        ctx.bind_top(*port, arg.clone());
    }
    ctx.eval_port(root_bind.value)
}

/// Structural equality on the interpreter's [`FieldValue`] surface (T-LensAPI / D2).
pub fn field_value_equal(lhs: &FieldValue, rhs: &FieldValue) -> bool {
    lhs == rhs
}

/// Build a substrate-shaped `Dag` record (only `nodes` is populated faithfully) from a
/// compiled program [`Dag`], for lenses like `named_function_count` that read `d.nodes`.
pub fn reflect_program_dag_nodes(program: &Dag) -> Result<FieldValue, LensApplyError> {
    let nodes = reflect_behavior_list(program, program.nodes())?;
    Ok(FieldValue::Record(vec![("nodes".to_string(), nodes)]))
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
    UnresolvedBranchPattern,
    UnimplementedCallable(String),
    UnimplementedLoopBound,
    BranchMiss,
    BadFieldProject,
    BadListShape,
    MissingType(&'static str),
}

struct EvalCtx<'a> {
    dag: &'a Dag,
    frames: Vec<HashMap<u32, FieldValue>>,
}

impl<'a> EvalCtx<'a> {
    fn new(dag: &'a Dag) -> Self {
        Self {
            dag,
            frames: vec![HashMap::new()],
        }
    }

    fn bind_top(&mut self, port: PortId, value: FieldValue) {
        self.frames
            .last_mut()
            .expect("root frame")
            .insert(port.raw(), value);
    }

    fn lookup(&self, port: PortId) -> Option<FieldValue> {
        self.frames
            .iter()
            .rev()
            .find_map(|m| m.get(&port.raw()).cloned())
    }

    fn bind_current(&mut self, port: PortId, value: FieldValue) {
        self.frames
            .last_mut()
            .expect("frame")
            .insert(port.raw(), value);
    }

    fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
    }

    fn pop_frame(&mut self) {
        self.frames.pop();
        if self.frames.is_empty() {
            panic!("EvalCtx: popped root frame");
        }
    }

    fn eval_port(&mut self, port: PortId) -> Result<FieldValue, LensApplyError> {
        if let Some(v) = self.lookup(port) {
            return Ok(v);
        }
        let producer = self
            .dag
            .resolve_producer_opt(&port)
            .ok_or(LensApplyError::UnresolvedPort)?;
        let out = match producer {
            Behavior::Value(v) => FieldValue::Literal(v.data.clone()),
            Behavior::Transform(t) => self.eval_transform(t)?,
            Behavior::Branch(b) => self.eval_branch(b, port)?,
            Behavior::Loop(l) => self.eval_loop(l)?,
            Behavior::Bind(b) => {
                if b.params.is_empty() {
                    self.eval_port(b.value)?
                } else {
                    return Err(LensApplyError::UnsupportedConstruct(
                        "non-root function bind as producer",
                    ));
                }
            }
        };
        self.bind_current(port, out.clone());
        Ok(out)
    }

    fn eval_transform(&mut self, t: &TransformNode) -> Result<FieldValue, LensApplyError> {
        match &t.target {
            TransformTarget::Operator(OperatorKind::Arithmetic(op)) => {
                if t.inputs.len() != 2 {
                    return Err(LensApplyError::ArityMismatch {
                        expected: 2,
                        got: t.inputs.len(),
                    });
                }
                let a = int_from_value(&self.eval_port(t.inputs[0])?)?;
                let b = int_from_value(&self.eval_port(t.inputs[1])?)?;
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
                let a = int_from_value(&self.eval_port(t.inputs[0])?)?;
                let b = int_from_value(&self.eval_port(t.inputs[1])?)?;
                let out = match op {
                    crate::dag::ComparisonOp::Eq => a == b,
                    crate::dag::ComparisonOp::Ne => a != b,
                    crate::dag::ComparisonOp::Lt => a < b,
                    crate::dag::ComparisonOp::Le => a <= b,
                    crate::dag::ComparisonOp::Gt => a > b,
                    crate::dag::ComparisonOp::Ge => a >= b,
                };
                Ok(bool_value(self.dag, out)?)
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
                let base = self.eval_port(t.inputs[0])?;
                project_field(&base, field_label)
            }
            TransformTarget::Callable(callee) => self.eval_callable(*callee, &t.inputs),
        }
    }

    fn eval_callable(
        &mut self,
        callee: DeclarationId,
        arg_ports: &[PortId],
    ) -> Result<FieldValue, LensApplyError> {
        if self.dag.std_list_fold_decl() == Some(callee) {
            return self.eval_std_fold(arg_ports);
        }
        let decl = self.dag.declaration(callee);
        let name = decl.name.clone().unwrap_or_default();
        let TypeConnective::Arrow {
            inputs,
            output: _,
            body,
        } = &decl.connective
        else {
            return Err(LensApplyError::UnimplementedCallable(format!(
                "`{name}` is not an arrow"
            )));
        };
        let ArrowBody::UserDefined(root) = body else {
            return Err(LensApplyError::UnimplementedCallable(format!(
                "`{name}` has no UserDefined body"
            )));
        };
        if inputs.len() != arg_ports.len() {
            return Err(LensApplyError::ArityMismatch {
                expected: inputs.len(),
                got: arg_ports.len(),
            });
        }
        let Behavior::Bind(b) = self.dag.node(*root) else {
            return Err(LensApplyError::MalformedLensRoot);
        };
        if b.params.len() != arg_ports.len() {
            return Err(LensApplyError::ArityMismatch {
                expected: b.params.len(),
                got: arg_ports.len(),
            });
        }
        self.push_frame();
        for (param, arg_port) in b.params.iter().zip(arg_ports.iter()) {
            let v = self.eval_port(*arg_port)?;
            self.bind_current(*param, v);
        }
        let out = self.eval_port(b.value);
        self.pop_frame();
        out
    }

    fn eval_std_fold(&mut self, arg_ports: &[PortId]) -> Result<FieldValue, LensApplyError> {
        if arg_ports.len() != 3 {
            return Err(LensApplyError::ArityMismatch {
                expected: 3,
                got: arg_ports.len(),
            });
        }
        let list = self.eval_port(arg_ports[0])?;
        let init = self.eval_port(arg_ports[1])?;
        let step_bind_port = arg_ports[2];
        let step_producer = self
            .dag
            .resolve_producer_opt(&step_bind_port)
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
        for elt in list_elements(self.dag, &list)? {
            self.push_frame();
            self.bind_current(step_bind.params[0], acc);
            self.bind_current(step_bind.params[1], elt);
            acc = self.eval_port(step_bind.value)?;
            self.pop_frame();
        }
        Ok(acc)
    }

    fn eval_branch(&mut self, b: &BranchNode, out_port: PortId) -> Result<FieldValue, LensApplyError> {
        let disc = self.eval_port(b.input)?;
        for path in &b.paths {
            let variant_id = match &path.pattern {
                BranchPattern::ResolvedVariant(id) => *id,
                BranchPattern::UnresolvedVariant { .. } => {
                    return Err(LensApplyError::UnresolvedBranchPattern);
                }
            };
            if !variant_matches(self.dag, &disc, variant_id)? {
                continue;
            }
            self.push_frame();
            if let Some(binding) = &path.binding {
                let payload = variant_payload_for_binding(self.dag, &disc, variant_id)?;
                self.bind_current(binding.payload_port, payload);
            }
            let v = self.eval_port(path.output)?;
            self.pop_frame();
            self.bind_current(out_port, v.clone());
            return Ok(v);
        }
        Err(LensApplyError::BranchMiss)
    }

    fn eval_loop(&mut self, l: &crate::dag::LoopNode) -> Result<FieldValue, LensApplyError> {
        match &l.bound {
            LoopBound::Cardinality { count } => {
                let n = int_from_value(&self.eval_port(*count)?)?;
                if n < 0 {
                    return Err(LensApplyError::TypeMismatch("negative loop bound"));
                }
                let mut acc = self.eval_port(l.init)?;
                for _ in 0..n {
                    self.push_frame();
                    self.bind_current(
                        self.dag
                            .node(l.body)
                            .as_bind()
                            .ok_or(LensApplyError::UnsupportedConstruct(
                                "loop body not bind-shaped",
                            ))?
                            .params[0],
                        acc,
                    );
                    // Loop body is a Bind(param, inner); producer chain may vary —
                    // follow `l.body` node's value port after binding accumulator.
                    let body_bind = self.dag.node(l.body).as_bind().ok_or(
                        LensApplyError::UnsupportedConstruct("loop body not a Bind"),
                    )?;
                    acc = self.eval_port(body_bind.value)?;
                    self.pop_frame();
                }
                Ok(acc)
            }
            LoopBound::Descent { .. } => Err(LensApplyError::UnimplementedLoopBound),
        }
    }
}

fn int_from_value(v: &FieldValue) -> Result<i64, LensApplyError> {
    match v {
        FieldValue::Literal(LiteralBits::Int(n)) => Ok(*n),
        _ => Err(LensApplyError::TypeMismatch("expected Int literal")),
    }
}

fn bool_value(dag: &Dag, b: bool) -> Result<FieldValue, LensApplyError> {
    let bool_decl = dag
        .declaration_by_name("Bool")
        .ok_or(LensApplyError::MissingType("Bool"))?;
    let TypeConnective::Disj { variants } = &bool_decl.connective else {
        return Err(LensApplyError::MissingType("Bool shape"));
    };
    let label = if b { "True" } else { "False" };
    let id = variants
        .iter()
        .find(|v| v.label == label)
        .ok_or(LensApplyError::MissingType("Bool variant"))?
        .ty;
    Ok(FieldValue::Variant {
        constructor: id,
        payload: vec![],
    })
}

fn project_field(base: &FieldValue, label: &str) -> Result<FieldValue, LensApplyError> {
    let FieldValue::Record(fields) = base else {
        return Err(LensApplyError::BadFieldProject);
    };
    fields
        .iter()
        .find(|(l, _)| l == label)
        .map(|(_, v)| v.clone())
        .ok_or(LensApplyError::BadFieldProject)
}

fn variant_matches(
    dag: &Dag,
    value: &FieldValue,
    variant_ty: DeclarationId,
) -> Result<bool, LensApplyError> {
    let FieldValue::Variant { constructor, .. } = value else {
        return Ok(false);
    };
    Ok(*constructor == variant_ty)
}

fn variant_payload_for_binding(
    dag: &Dag,
    value: &FieldValue,
    variant_ty: DeclarationId,
) -> Result<FieldValue, LensApplyError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(LensApplyError::BadFieldProject);
    };
    if *constructor != variant_ty {
        return Err(LensApplyError::BadFieldProject);
    }
    if payload.len() == 1 {
        return Ok(payload[0].clone());
    }
    let conj = dag.declaration(variant_ty);
    let TypeConnective::Conj { children } = &conj.connective else {
        // Unit-like variant (e.g. `True` / `False` / `Empty`).
        return Ok(FieldValue::Record(vec![]));
    };
    if payload.len() != children.len() {
        return Err(LensApplyError::BadFieldProject);
    }
    let fields: Vec<_> = children
        .iter()
        .zip(payload.iter())
        .map(|(c, v)| (c.label.clone(), v.clone()))
        .collect();
    Ok(FieldValue::Record(fields))
}

fn list_elements(dag: &Dag, list: &FieldValue) -> Result<Vec<FieldValue>, LensApplyError> {
    let mut out = Vec::new();
    let mut cur = list;
    loop {
        let FieldValue::Variant {
            constructor,
            payload,
        } = cur
        else {
            return Err(LensApplyError::BadListShape);
        };
        let label = variant_label(dag, *constructor).ok_or(LensApplyError::BadListShape)?;
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

fn variant_label(dag: &Dag, variant_id: DeclarationId) -> Option<String> {
    dag.declarations().iter().find_map(|decl| match &decl.connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .find(|variant| variant.ty == variant_id)
            .map(|variant| variant.label.clone()),
        _ => None,
    })
}

fn v3_list_empty_cons_ids(dag: &Dag) -> Result<(DeclarationId, DeclarationId), LensApplyError> {
    let list_decl = dag.declarations().iter().find(|d| {
        d.name.as_deref() == Some("List") && matches!(d.connective, TypeConnective::Disj { .. })
    });
    let Some(list_decl) = list_decl else {
        return Err(LensApplyError::MissingType("List"));
    };
    let TypeConnective::Disj { variants } = &list_decl.connective else {
        return Err(LensApplyError::MissingType("List"));
    };
    let empty = variants
        .iter()
        .find(|v| v.label == "Empty")
        .ok_or(LensApplyError::MissingType("List.Empty"))?
        .ty;
    let cons = variants
        .iter()
        .find(|v| v.label == "Cons")
        .ok_or(LensApplyError::MissingType("List.Cons"))?
        .ty;
    Ok((empty, cons))
}

fn behavior_variant_id(dag: &Dag, label: &str) -> Result<DeclarationId, LensApplyError> {
    let decl = dag
        .declaration_by_name("Behavior")
        .ok_or(LensApplyError::MissingType("Behavior"))?;
    let TypeConnective::Disj { variants } = &decl.connective else {
        return Err(LensApplyError::MissingType("Behavior"));
    };
    variants
        .iter()
        .find(|v| v.label == label)
        .map(|v| v.ty)
        .ok_or(LensApplyError::MissingType("Behavior variant"))
}

fn reflect_behavior_list(dag: &Dag, nodes: &[Behavior]) -> Result<FieldValue, LensApplyError> {
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

fn reflect_behavior(dag: &Dag, behavior: &Behavior) -> Result<FieldValue, LensApplyError> {
    match behavior {
        Behavior::Value(v) => {
            let id = behavior_variant_id(dag, "Value")?;
            let payload = FieldValue::Record(vec![
                (
                    "payload".to_string(),
                    FieldValue::Literal(v.data.clone()),
                ),
                (
                    "result_port".to_string(),
                    FieldValue::Literal(LiteralBits::Int(i64::from(v.output.raw()))),
                ),
            ]);
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![payload],
            })
        }
        Behavior::Transform(t) => {
            let id = behavior_variant_id(dag, "Transform")?;
            let payload = FieldValue::Record(vec![(
                "result_port".to_string(),
                FieldValue::Literal(LiteralBits::Int(i64::from(t.output.raw()))),
            )]);
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![payload],
            })
        }
        Behavior::Branch(b) => {
            let id = behavior_variant_id(dag, "Branch")?;
            let payload = FieldValue::Record(vec![(
                "result_port".to_string(),
                FieldValue::Literal(LiteralBits::Int(i64::from(b.output.raw()))),
            )]);
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![payload],
            })
        }
        Behavior::Loop(l) => {
            let id = behavior_variant_id(dag, "Loop")?;
            let payload = FieldValue::Record(vec![(
                "result_port".to_string(),
                FieldValue::Literal(LiteralBits::Int(i64::from(l.output.raw()))),
            )]);
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![payload],
            })
        }
        Behavior::Bind(b) => {
            let id = behavior_variant_id(dag, "Bind")?;
            let record = bindnode_record(b);
            Ok(FieldValue::Variant {
                constructor: id,
                payload: vec![record],
            })
        }
    }
}

fn bindnode_record(b: &crate::dag::BindNode) -> FieldValue {
    FieldValue::Record(vec![(
        "name".to_string(),
        FieldValue::Literal(LiteralBits::String(b.name.clone())),
    )])
}

/// Evaluate `(a ⊕ b) ⊕ c` vs `a ⊕ (b ⊕ c)` for a binary `Int` lens using [`apply_lens_declaration`].
pub fn int_associativity_holds(
    program_dag: &Dag,
    lens_decl_id: DeclarationId,
    a: i64,
    b: i64,
    c: i64,
) -> Result<bool, LensApplyError> {
    let int = |n: i64| FieldValue::Literal(LiteralBits::Int(n));
    let left_ab = apply_lens_declaration(program_dag, lens_decl_id, &[int(a), int(b)])?;
    let left = apply_lens_declaration(program_dag, lens_decl_id, &[left_ab, int(c)])?;
    let right_bc = apply_lens_declaration(program_dag, lens_decl_id, &[int(b), int(c)])?;
    let right = apply_lens_declaration(program_dag, lens_decl_id, &[int(a), right_bc])?;
    Ok(field_value_equal(&left, &right))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_to_dag;

    #[test]
    fn named_function_count_on_trivial_program() {
        let src = include_str!("../../../lenses/named_function_count.dag");
        let lens_dag =
            compile_to_dag(src, "src/v3/lenses/named_function_count.dag").expect("lens compiles");
        let prog = compile_to_dag("let x: Int = 1", "lens_apply_prog.v3").expect("prog compiles");
        let lens_id = lens_dag
            .declaration_by_name("named_function_count")
            .expect("named_function_count")
            .id;
        let input = reflect_program_dag_nodes(&prog).expect("reflect");
        let out = apply_lens_declaration(&lens_dag, lens_id, &[input]).expect("apply");
        assert_eq!(out, FieldValue::Literal(LiteralBits::Int(1)));
    }

    #[test]
    fn int_add_lens_associativity_sample() {
        let src = r#"module w
fn lens_composition_op(a: Int, b: Int) -> Int = a + b
"#;
        let dag = compile_to_dag(src, "assoc.v3").expect("compiles");
        let id = dag.declaration_by_name("lens_composition_op").unwrap().id;
        assert!(int_associativity_holds(&dag, id, 2, 3, 5).expect("assoc"));
    }
}
