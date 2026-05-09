//! Bounded lens application (T-LensAPI / D1): interpret `ArrowBody::UserDefined` graphs
//! over substrate-shaped [`FieldValue`] — no whole-claim operator recognizers.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use num_bigint::BigInt;

use crate::dag::{
    literal_bits_int, ArrowBody, AtomPayload, Behavior, BindNode, BranchNode, BranchPattern, Dag,
    Declaration, DeclarationId, FieldValue, LiteralBits, OperatorKind, PortId, TransformNode,
    TransformTarget, TypeConnective, ValueBody,
};
use crate::infer_helpers::resolve_template_argument_value;

fn is_fold_instantiation(dag: &Dag, decl: &Declaration) -> bool {
    matches!(
        &decl.connective,
        TypeConnective::Instantiation { template, .. }
            if dag.std_list_fold_decl() == Some(*template)
    )
}

const LENS_APPLY_TYPE_WALK_DEPTH: usize = 32;

/// Structural eligibility for the monomorphized `std.list.fold` → [`eval_fold_step`] fast path.
///
/// `eval_fold_step` / `eval_std_fold` are type-erased over accumulator and element shape: they
/// bind acc/elt as opaque [`FieldValue`] to the step `Bind`'s last two params and walk the step
/// body via [`EvalCtx::eval_port`]. The interpretability constraint therefore lives on the step
/// body's transitive [`Behavior`] reachability, not on the resolved type arguments. A step body
/// is eligible iff every reachable producer is one the bounded interpreter can evaluate:
/// [`Behavior::Loop`], non-`UserDefined` [`ArrowBody`], parameterized `Bind` producers, and
/// logical operators are all rejected as fail-closed (matching the interpreter's own
/// [`LensApplyError::UnimplementedLoopBound`] / [`LensApplyError::UnsupportedConstruct`] paths).
///
/// **Dissolution trigger:** delete this predicate when `eval_loop` and the rejected
/// `Behavior::Bind` / `Logical` paths gain bounded semantics — at that point every fold step
/// body is interpretable and the eligibility gate is vacuous.
fn step_body_eligible_for_bounded_eval(dag: &Dag, step_bind: &BindNode) -> bool {
    let mut visited: HashSet<DeclarationId> = HashSet::new();
    eligibility_walk_port(dag, step_bind.value, &mut visited, 0)
}

fn eligibility_walk_port(
    dag: &Dag,
    port: PortId,
    visited: &mut HashSet<DeclarationId>,
    depth: usize,
) -> bool {
    if depth >= LENS_APPLY_TYPE_WALK_DEPTH {
        return false;
    }
    // Parameter ports (acc/elt of the step `Bind`, formals of any callable we walk into)
    // have no producer — they're bound by the interpreter at evaluation time. Treat as
    // eligible; only producer-backed nodes contribute interpretability constraints.
    let Some(producer) = dag.resolve_producer_opt(&port) else {
        return true;
    };
    match producer {
        Behavior::Value(_) => true,
        Behavior::Transform(t) => eligibility_walk_transform(dag, t, visited, depth + 1),
        Behavior::Branch(b) => eligibility_walk_branch(dag, b, visited, depth + 1),
        Behavior::Loop(_) => false,
        Behavior::Bind(b) => {
            if !b.params.is_empty() {
                return false;
            }
            eligibility_walk_port(dag, b.value, visited, depth + 1)
        }
    }
}

fn eligibility_walk_transform(
    dag: &Dag,
    t: &TransformNode,
    visited: &mut HashSet<DeclarationId>,
    depth: usize,
) -> bool {
    for input in &t.inputs {
        if !eligibility_walk_port(dag, *input, visited, depth + 1) {
            return false;
        }
    }
    match &t.target {
        TransformTarget::Operator(OperatorKind::Logical(_)) => false,
        TransformTarget::Operator(_) | TransformTarget::FieldProject { .. } => true,
        TransformTarget::Callable(callee) => {
            eligibility_walk_callable(dag, *callee, visited, depth + 1)
        }
    }
}

fn eligibility_walk_branch(
    dag: &Dag,
    b: &BranchNode,
    visited: &mut HashSet<DeclarationId>,
    depth: usize,
) -> bool {
    if !eligibility_walk_port(dag, b.input, visited, depth + 1) {
        return false;
    }
    for path in &b.paths {
        match &path.pattern {
            BranchPattern::ResolvedVariant(_) => {}
            BranchPattern::UnresolvedVariant { .. } => return false,
        }
        if !eligibility_walk_port(dag, path.output, visited, depth + 1) {
            return false;
        }
    }
    true
}

fn eligibility_walk_callable(
    dag: &Dag,
    callee: DeclarationId,
    visited: &mut HashSet<DeclarationId>,
    depth: usize,
) -> bool {
    if depth >= LENS_APPLY_TYPE_WALK_DEPTH {
        return false;
    }
    if !visited.insert(callee) {
        return true;
    }
    let decl = dag.declaration(callee);
    match &decl.connective {
        TypeConnective::Instantiation { template, .. } => {
            // Nested `std.list.fold` instantiations: the interpreter dispatches the same
            // bounded path recursively, so eligibility is conditional on the inner step body.
            // For now any nested fold is treated as ineligible — its step body would need to
            // be located via `find_fold_step_bind_via_instantiation`, which is the same shape
            // recursion this predicate already walks at the outer call site.
            if dag.std_list_fold_decl() == Some(*template) {
                return false;
            }
            eligibility_walk_callable(dag, *template, visited, depth + 1)
        }
        TypeConnective::Arrow { body, .. } => {
            let ArrowBody::UserDefined(root) = body else {
                return false;
            };
            let b = (*root).bind(dag);
            eligibility_walk_port(dag, b.value, visited, depth + 1)
        }
        _ => false,
    }
}

fn declaration_is_callable_type(dag: &Dag, current: DeclarationId, depth: usize) -> bool {
    if depth >= LENS_APPLY_TYPE_WALK_DEPTH {
        return false;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Arrow { .. } => true,
        TypeConnective::Instantiation { template, .. } => {
            declaration_is_callable_type(dag, *template, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            declaration_is_callable_type(dag, *next, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::TypeParam(_))
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Cardinality(_) => false,
    }
}

/// Arrow formals of `std.list.fold` whose types are callable — the step `f` slot.
fn fold_template_callable_formals(dag: &Dag, fold_template: DeclarationId) -> Vec<DeclarationId> {
    let decl = dag.declaration(fold_template);
    let TypeConnective::Arrow { inputs, .. } = &decl.connective else {
        return Vec::new();
    };
    inputs
        .iter()
        .copied()
        .filter(|&i| declaration_is_callable_type(dag, i, 0))
        .collect()
}

/// Peel `Instantiation` carriers (same contract as [`LensInterpreter::eval_callable`]) until an
/// `Arrow` with `UserDefined` body; return the root `Bind` when it has ≥2 params (step closure).
fn monomorph_callable_bind_root(dag: &Dag, mut decl_id: DeclarationId) -> Option<&BindNode> {
    for _ in 0..LENS_APPLY_TYPE_WALK_DEPTH {
        let decl = dag.declaration(decl_id);
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => {
                if dag.std_list_fold_decl() == Some(*template) {
                    return None;
                }
                decl_id = *template;
            }
            TypeConnective::Arrow { body, .. } => {
                let ArrowBody::UserDefined(root) = body else {
                    return None;
                };
                let b = (*root).bind(dag);
                return (b.params.len() >= 2).then_some(b);
            }
            _ => return None,
        }
    }
    None
}

/// Locate the `|acc, x|` step closure lowered as a two-parameter `Bind` for this `fold` site.
///
/// Walk `Instantiation.arguments` on the monomorphized fold callee — substrate `DeclarationId`
/// keyed to the template's callable formal — then resolve the step `Arrow` root `Bind`. Matches
/// builder/runtime arity: the step is not a `Transform` input port. No span-overlap recovery —
/// that would not be a declared substrate dependency (Facts Flow Forward).
///
/// **Dissolution:** attach the step as an explicit `Transform` input / behavior edge and delete
/// this `Instantiation`-walk indirection when lowering guarantees a direct edge.
fn find_fold_step_bind_via_instantiation(
    dag: &Dag,
    fold_callable_id: DeclarationId,
) -> Option<&BindNode> {
    let decl = dag.declaration(fold_callable_id);
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &decl.connective
    else {
        return None;
    };
    if dag.std_list_fold_decl() != Some(*template) {
        return None;
    }
    let formals = fold_template_callable_formals(dag, *template);
    let depth_budget = LENS_APPLY_TYPE_WALK_DEPTH as i64;

    for arg in arguments {
        let resolved = resolve_template_argument_value(&depth_budget, arguments, arg.value);
        let Some(b) = monomorph_callable_bind_root(dag, resolved) else {
            continue;
        };
        if formals.contains(&arg.parameter) {
            return Some(b);
        }
    }
    None
}

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
    let root_bind = (*root).bind(lens_program);
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

/// Build a substrate-shaped `Dag` record (only `nodes` is populated faithfully) from a
/// compiled program [`Dag`], for lenses like `named_function_count` that read `d.nodes`.
///
/// Reflection is **complete** per [`docs/design-reflection-completeness.md`](../../docs/design-reflection-completeness.md):
/// every substrate-declared field on each [`Behavior`] variant (and nested carriers such as
/// [`WorkflowEffect`]) projects into [`FieldValue`] with no per-consumer narrowing and no
/// execution semantics (structural `NodeId` / `PortId` references are not followed into
/// callee bodies).
///
/// `source_file` limits nodes to those authored in that compilation unit (the merged
/// bootstrap graph also lives in the same [`Dag`]).
///
/// **`id_space` (INVARIANTS P2):** `List` / `Behavior` variant [`DeclarationId`]s in the
/// returned [`FieldValue`] are taken from `id_space` — the same [`Dag`] you pass to
/// [`apply_lens_declaration`] for the lens under test. Shapes still come from `program`'s
/// lowered [`Behavior`] nodes (filtered by `source_file`). Pass `program` for both when the
/// lens and claim program share one compile; pass the canonical lens `Dag` as `id_space`
/// when applying `named_function_count` from the same bytes as `build.rs` splices while
/// reflecting nodes from the claim program.
pub fn reflect_program_dag_nodes_in_file(
    program: &Dag,
    source_file: &str,
    id_space: &Dag,
) -> Result<FieldValue, LensApplyError> {
    let nodes: Vec<Behavior> = program
        .nodes()
        .iter()
        .filter(|b| behavior_source_file(b) == source_file)
        .cloned()
        .collect();
    let nodes = substrate_reflection::reflect_behavior_list(id_space, &nodes)?;
    Ok(FieldValue::Record(vec![("nodes".to_string(), nodes)]))
}

/// Empty `std.list` spine (`Empty` variant) in the substrate shape expected by `fold`.
pub fn empty_substrate_list_value(dag: &Dag) -> Result<FieldValue, LensApplyError> {
    let (empty_id, _) = v3_list_empty_cons_ids(dag)?;
    Ok(FieldValue::Variant {
        constructor: empty_id,
        payload: vec![],
    })
}

fn behavior_source_file(behavior: &Behavior) -> &str {
    match behavior {
        Behavior::Value(v) => v.span.file.as_str(),
        Behavior::Transform(t) => t.span.file.as_str(),
        Behavior::Branch(b) => b.span.file.as_str(),
        Behavior::Loop(l) => l.span.file.as_str(),
        Behavior::Bind(b) => b.span.file.as_str(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LensApplyError {
    NotAnArrow,
    UnsupportedArrowBody,
    ArityMismatch {
        expected: usize,
        got: usize,
    },
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
    MissingValueBody,
    /// PR-E: reserved for fold-driver paths that are not yet delegated to
    /// [`apply_lens_declaration`] (e.g. `DimensionReport` / lens-instance carriers).
    /// Per Q-Reification Option-A (Dag IS the reflected program), the historical
    /// `fold_lens_over_reflected_program` reflect-then-apply driver has been retired;
    /// callers go through [`apply_lens_declaration`] directly.
    UnimplementedLensFold,
    /// Substrate → [`FieldValue`] reflection failed (missing sum/variant wiring in id_space).
    SubstrateReflect(&'static str),
}

impl From<substrate_reflection::ReflectError> for LensApplyError {
    fn from(e: substrate_reflection::ReflectError) -> Self {
        LensApplyError::SubstrateReflect(e.0)
    }
}

/// Lower a declaration [`ValueBody`] into the structural [`FieldValue`] carrier used by the
/// lens interpreter.
///
/// `fixture_dag` is the graph that will own [`DeclarationId`] edges once structural bodies carry
/// resolvable `Reference` payloads through this helper; today only scalar + shallow structural
/// clone paths are implemented (no `fixture_dag` lookup yet).
pub fn field_value_from_value_body(
    #[allow(unused_variables)] fixture_dag: &Dag,
    body: &ValueBody,
) -> Result<FieldValue, LensApplyError> {
    match body {
        ValueBody::Scalar(bits) => Ok(FieldValue::Literal(bits.clone())),
        ValueBody::Structural { fields } => {
            let mut out = Vec::with_capacity(fields.len());
            for (label, fv) in fields {
                out.push((label.clone(), fv.clone()));
            }
            Ok(FieldValue::Record(out))
        }
        ValueBody::List(values) => Ok(FieldValue::List(values.clone())),
        ValueBody::Map(entries) => Ok(FieldValue::Map(entries.clone())),
        ValueBody::Unparsed(_) => Err(LensApplyError::UnsupportedConstruct(
            "unparsed declaration value body",
        )),
    }
}

/// Frame-based evaluation context for D1 lens interpretation.
///
/// **Complexity / cloning:** port lookups use [`FieldValue::clone`] and list walks
/// clone elements deliberately — bounded R1 gate scope; revisit if lenses iterate
/// over large program DAGs (e.g. T-LaneE).
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
                // Only top-level lens roots (`apply_lens_declaration`) and fold step
                // binds (`eval_fold_step` / `eval_std_fold`) supply parameters; inner
                // parameterized `Bind` producers are not interpreted yet.
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
            TransformTarget::Callable(callee) => {
                let decl = self.dag.declaration(*callee);
                if is_fold_instantiation(self.dag, decl) && t.inputs.len() == 2 {
                    if let Some(step_bind) =
                        find_fold_step_bind_via_instantiation(self.dag, *callee)
                    {
                        if step_body_eligible_for_bounded_eval(self.dag, step_bind) {
                            let list = self.eval_port(t.inputs[0])?;
                            let init = self.eval_port(t.inputs[1])?;
                            return self.eval_fold_step(list, init, step_bind);
                        }
                    }
                }
                self.eval_callable(*callee, &t.inputs)
            }
            TransformTarget::Operator(OperatorKind::Arithmetic(op)) => {
                if t.inputs.len() != 2 {
                    return Err(LensApplyError::ArityMismatch {
                        expected: 2,
                        got: t.inputs.len(),
                    });
                }
                let a = int_from_value(&self.eval_port(t.inputs[0])?)?;
                let b = int_from_value(&self.eval_port(t.inputs[1])?)?;
                let n = apply_arithmetic_int(*op, a, b)?;
                Ok(FieldValue::Literal(LiteralBits::Int(n.to_string())))
            }
            TransformTarget::Operator(OperatorKind::Comparison(op)) => {
                if t.inputs.len() != 2 {
                    return Err(LensApplyError::ArityMismatch {
                        expected: 2,
                        got: t.inputs.len(),
                    });
                }
                let lhs = self.eval_port(t.inputs[0])?;
                let rhs = self.eval_port(t.inputs[1])?;
                let out = match (&lhs, &rhs) {
                    (
                        FieldValue::Literal(LiteralBits::Int(a)),
                        FieldValue::Literal(LiteralBits::Int(b)),
                    ) => {
                        let ai = BigInt::from_str(a).map_err(|_| {
                            LensApplyError::TypeMismatch("expected decimal Int literal")
                        })?;
                        let bi = BigInt::from_str(b).map_err(|_| {
                            LensApplyError::TypeMismatch("expected decimal Int literal")
                        })?;
                        match op {
                            crate::dag::ComparisonOp::Eq => ai == bi,
                            crate::dag::ComparisonOp::Ne => ai != bi,
                            crate::dag::ComparisonOp::Lt => ai < bi,
                            crate::dag::ComparisonOp::Le => ai <= bi,
                            crate::dag::ComparisonOp::Gt => ai > bi,
                            crate::dag::ComparisonOp::Ge => ai >= bi,
                        }
                    }
                    (
                        FieldValue::Literal(LiteralBits::String(a)),
                        FieldValue::Literal(LiteralBits::String(b)),
                    ) => match op {
                        crate::dag::ComparisonOp::Eq => a == b,
                        crate::dag::ComparisonOp::Ne => a != b,
                        _ => {
                            return Err(LensApplyError::UnsupportedConstruct(
                                "string comparison beyond Eq/Ne",
                            ));
                        }
                    },
                    _ => {
                        return Err(LensApplyError::UnsupportedConstruct(
                            "comparison operands must both be Int literals or both String literals",
                        ));
                    }
                };
                Ok(FieldValue::Literal(LiteralBits::Bool(out)))
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
        }
    }

    fn eval_callable(
        &mut self,
        callee: DeclarationId,
        arg_ports: &[PortId],
    ) -> Result<FieldValue, LensApplyError> {
        let decl = self.dag.declaration(callee);
        if self.dag.std_list_fold_decl() == Some(callee) {
            return self.eval_std_fold(arg_ports);
        }
        if let TypeConnective::Instantiation { template, .. } = &decl.connective {
            // Monomorphized user functions (`count_named_bind`, …) lower as `Instantiation`
            // carriers; interpretation follows the generic template `Arrow`.
            if self.dag.std_list_fold_decl() != Some(*template) {
                return self.eval_callable(*template, arg_ports);
            }
        }
        let name = decl.name.clone().unwrap_or_default();
        let TypeConnective::Arrow {
            inputs,
            output: _,
            body,
        } = &decl.connective
        else {
            return Err(LensApplyError::UnimplementedCallable(format!(
                "`{name}` (id={}) is not an evaluated callable arrow",
                callee.raw()
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
        let b = (*root).bind(self.dag);
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

    fn eval_fold_step(
        &mut self,
        list: FieldValue,
        init: FieldValue,
        step_bind: &BindNode,
    ) -> Result<FieldValue, LensApplyError> {
        let n = step_bind.params.len();
        if n < 2 {
            return Err(LensApplyError::ArityMismatch {
                expected: 2,
                got: n,
            });
        }
        // Monomorphized `fold` lowers the `|acc, x|` lambda with possible leading
        // synthesized parameters; the accumulator and element are always the last two
        // formal parameter ports.
        let acc_param = step_bind.params[n - 2];
        let elt_param = step_bind.params[n - 1];
        let mut acc = init;
        for elt in list_elements(self.dag, &list)? {
            self.push_frame();
            self.bind_current(acc_param, acc);
            self.bind_current(elt_param, elt);
            acc = self.eval_port(step_bind.value)?;
            self.pop_frame();
        }
        Ok(acc)
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
        let n = step_bind.params.len();
        if n < 2 {
            return Err(LensApplyError::ArityMismatch {
                expected: 2,
                got: n,
            });
        }
        let acc_param = step_bind.params[n - 2];
        let elt_param = step_bind.params[n - 1];
        let mut acc = init;
        for elt in list_elements(self.dag, &list)? {
            self.push_frame();
            self.bind_current(acc_param, acc);
            self.bind_current(elt_param, elt);
            acc = self.eval_port(step_bind.value)?;
            self.pop_frame();
        }
        Ok(acc)
    }

    fn eval_branch(
        &mut self,
        b: &BranchNode,
        out_port: PortId,
    ) -> Result<FieldValue, LensApplyError> {
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

    /// Loop bodies are intentionally uninterpreted in D1.
    ///
    /// **Fail-closed receipt:** both `LoopBound::Cardinality` (single-fn recursion)
    /// and `LoopBound::Descent` (mutual-recursion clusters) return
    /// [`LensApplyError::UnimplementedLoopBound`] until iteration semantics land in
    /// this interpreter.
    fn eval_loop(&mut self, _l: &crate::dag::LoopNode) -> Result<FieldValue, LensApplyError> {
        Err(LensApplyError::UnimplementedLoopBound)
    }
}

fn int_from_value(v: &FieldValue) -> Result<i64, LensApplyError> {
    match v {
        FieldValue::Literal(LiteralBits::Int(s)) => s
            .parse::<i64>()
            .map_err(|_| LensApplyError::TypeMismatch("expected Int literal in i64 range")),
        _ => Err(LensApplyError::TypeMismatch("expected Int literal")),
    }
}

/// Checked `Int` arithmetic for the D1 lens interpreter (INVARIANTS P3: no wrapping `+`/`-`/`*`/`/`
/// that could fabricate results or panic in debug under `LensOutputEquals` / gate paths).
fn apply_arithmetic_int(
    op: crate::dag::ArithmeticOp,
    a: i64,
    b: i64,
) -> Result<i64, LensApplyError> {
    const OVERFLOW: LensApplyError = LensApplyError::UnsupportedConstruct("integer overflow");
    match op {
        crate::dag::ArithmeticOp::Add => a.checked_add(b).ok_or(OVERFLOW),
        crate::dag::ArithmeticOp::Sub => a.checked_sub(b).ok_or(OVERFLOW),
        crate::dag::ArithmeticOp::Mul => a.checked_mul(b).ok_or(OVERFLOW),
        crate::dag::ArithmeticOp::Div => {
            if b == 0 {
                return Err(LensApplyError::TypeMismatch("division by zero"));
            }
            a.checked_div(b).ok_or(OVERFLOW)
        }
    }
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
    match value {
        FieldValue::Variant { constructor, .. } => Ok(*constructor == variant_ty),
        FieldValue::Literal(LiteralBits::Bool(b)) => {
            let expected_ty = dag
                .bool_runtime_variant_id(*b)
                .ok_or(LensApplyError::MissingType("Bool variant"))?;
            Ok(expected_ty == variant_ty)
        }
        _ => Ok(false),
    }
}

fn variant_payload_for_binding(
    dag: &Dag,
    value: &FieldValue,
    variant_ty: DeclarationId,
) -> Result<FieldValue, LensApplyError> {
    if let FieldValue::Literal(LiteralBits::Bool(_)) = value {
        if !variant_matches(dag, value, variant_ty)? {
            return Err(LensApplyError::BadFieldProject);
        }
        let conj = dag.declaration(variant_ty);
        return match &conj.connective {
            TypeConnective::Conj { children } if children.is_empty() => {
                Ok(FieldValue::Record(vec![]))
            }
            TypeConnective::Conj { .. } => Err(LensApplyError::UnsupportedConstruct(
                "Bool match arm with payload fields is not supported for Bool literal scrutinee",
            )),
            _ => Ok(FieldValue::Record(vec![])),
        };
    }
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

/// Uncons a substrate `List` spine into owned elements (clones each head).
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

// perf: linear scan over all declarations per variant id — fine for R1 gate DAGs;
// revisit if lenses iterate over large programs (prefer declaration-indexed lookup).
fn variant_label(dag: &Dag, variant_id: DeclarationId) -> Option<String> {
    dag.declarations()
        .iter()
        .find_map(|decl| match &decl.connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .find(|variant| variant.ty == variant_id)
                .map(|variant| variant.label.clone()),
            _ => None,
        })
}

// perf: linear scan for `List` disj — same bounded-gate rationale as `variant_label`.
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

/// Fixed `(a, b, c)` triples for R1 `AlgebraicLaw(Associativity, …)` operational checks.
///
/// A **single** triple can pass for operations that are not associative (coincidence). The
/// runner requires **every** triple here to satisfy [`int_associativity_holds`] — still not a
/// quantified substrate law proof, but a materially stronger witness than one sample. Magnitudes
/// stay modest so `Int` addition-style lenses do not overflow during witness application.
pub const ASSOCIATIVITY_WITNESS_TRIPLES: &[(i64, i64, i64)] = &[
    (2, 3, 5),
    (0, 1, 99),
    (-3, 7, 2),
    (-1, 0, 1),
    (1, 1, 2),
    (5, 0, 3),
    (10, -4, 7),
    (100, 200, 300),
];

/// Fixed `(a, b)` pairs for PR-B.3 `AlgebraicLaw(Commutativity, …)` operational checks.
///
/// These are runner witnesses, not substrate law proofs. The sample table lives next to the
/// associativity table so law-runner scaffolds share one bounded Int witness authority until
/// first-class substrate law witnesses dissolve the runner-side sample checks.
pub const COMMUTATIVITY_WITNESS_PAIRS: &[(i64, i64)] = &[
    (2, 3),
    (0, 99),
    (-3, 7),
    (-1, 0),
    (1, 1),
    (5, 0),
    (10, -4),
    (100, 200),
];

/// Samples for `AlgebraicLaw(Identity, …)` operational checks: **every** value must appear in at
/// least one [`COMMUTATIVITY_WITNESS_PAIRS`] coordinate so identity witnessing stays materially
/// aligned with the commutativity witness authority (still not a substrate law proof).
pub const IDENTITY_WITNESS_SAMPLES: &[i64] = &[-4, -3, -1, 0, 1, 2, 3, 5, 7, 10, 99, 100, 200];

/// Candidate identity elements searched left-to-right; exactly one must satisfy
/// `e ⊕ a = a` and `a ⊕ e = a` for every sample in [`IDENTITY_WITNESS_SAMPLES`]. Multiple matches
/// fail closed (`Ok(false)`) so incidental finite-table coincidences cannot certify an ambiguous op.
///
/// **High-end redundancy:** keep **at least two** entries **strictly greater** than
/// `max(IDENTITY_WITNESS_SAMPLES)` (currently `200`). With only one such candidate, a binary `Int`
/// lens behaving like `min` can satisfy the finite witness with a unique “top-like” constant even
/// though `Int` has no lattice top (INVARIANTS P1/P3; codex PR #2394). Several candidates also sit
/// below `min(samples)` so `max`-like ops typically hit the same multiplicity fail-closure.
pub const IDENTITY_WITNESS_CANDIDATES: &[i64] = &[
    -300, -200, -100, -99, -10, -7, -5, -4, -3, -2, -1, 0, 1, 2, 3, 5, 7, 10, 99, 100, 200, 250,
    300, 400,
];

/// Evaluate `(a ⊕ b) ⊕ c` vs `a ⊕ (b ⊕ c)` for a binary `Int` lens using [`apply_lens_declaration`].
pub fn int_associativity_holds(
    program_dag: &Dag,
    lens_decl_id: DeclarationId,
    a: i64,
    b: i64,
    c: i64,
) -> Result<bool, LensApplyError> {
    let int = |n: i64| FieldValue::Literal(literal_bits_int(n));
    let left_ab = apply_lens_declaration(program_dag, lens_decl_id, &[int(a), int(b)])?;
    let left = apply_lens_declaration(program_dag, lens_decl_id, &[left_ab, int(c)])?;
    let right_bc = apply_lens_declaration(program_dag, lens_decl_id, &[int(b), int(c)])?;
    let right = apply_lens_declaration(program_dag, lens_decl_id, &[int(a), right_bc])?;
    Ok(left == right)
}

/// True iff [`int_associativity_holds`] succeeds for every triple in `triples`.
pub fn int_associativity_holds_all_triples(
    program_dag: &Dag,
    lens_decl_id: DeclarationId,
    triples: &[(i64, i64, i64)],
) -> Result<bool, LensApplyError> {
    for &(a, b, c) in triples {
        if !int_associativity_holds(program_dag, lens_decl_id, a, b, c)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// True iff exactly one `e` in `candidates` satisfies left/right identity against every `samples`
/// entry via [`apply_lens_declaration`].
pub fn int_identity_witness_holds(
    program_dag: &Dag,
    lens_decl_id: DeclarationId,
    samples: &[i64],
    candidates: &[i64],
) -> Result<bool, LensApplyError> {
    let int = |n: i64| FieldValue::Literal(LiteralBits::Int(n));
    let mut matching = 0_i32;
    for &e in candidates {
        let ev = int(e);
        let mut ok = true;
        for &a in samples {
            let left = apply_lens_declaration(program_dag, lens_decl_id, &[ev.clone(), int(a)])?;
            let right = apply_lens_declaration(program_dag, lens_decl_id, &[int(a), ev.clone()])?;
            if left != int(a) || right != int(a) {
                ok = false;
                break;
            }
        }
        if ok {
            matching += 1;
            if matching > 1 {
                return Ok(false);
            }
        }
    }
    Ok(matching == 1)
}

const _: () = assert!(IDENTITY_WITNESS_SAMPLES.len() > 1);
const _: () = assert!(IDENTITY_WITNESS_CANDIDATES.len() > 1);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_to_dag;
    use crate::dag::{Behavior, LoopBound};

    #[test]
    fn named_function_count_on_trivial_program() {
        let src = include_str!("../../lenses/named_function_count.dag");
        let lens_dag =
            compile_to_dag(src, "src/v3/lenses/named_function_count.dag").expect("lens compiles");
        let prog = compile_to_dag("let x: Int = 1", "lens_apply_prog.v3").expect("prog compiles");
        let lens_id = lens_dag
            .declaration_by_name("named_function_count")
            .expect("named_function_count")
            .id;
        let input = reflect_program_dag_nodes_in_file(&prog, "lens_apply_prog.v3", &lens_dag)
            .expect("reflect");
        let out = apply_lens_declaration(&lens_dag, lens_id, &[input]).expect("apply");
        assert_eq!(out, FieldValue::Literal(literal_bits_int(1)));
    }

    #[test]
    fn identity_witness_samples_align_with_commutativity_witness_authority() {
        for &s in IDENTITY_WITNESS_SAMPLES {
            assert!(
                COMMUTATIVITY_WITNESS_PAIRS
                    .iter()
                    .any(|&(a, b)| a == s || b == s),
                "every IDENTITY_WITNESS_SAMPLES entry must appear in COMMUTATIVITY_WITNESS_PAIRS \
                 (doc invariant on IDENTITY_WITNESS_SAMPLES); missing sample {s}"
            );
        }
    }

    #[test]
    fn identity_witness_candidate_table_has_redundant_high_and_low_extents() {
        let sample_min = *IDENTITY_WITNESS_SAMPLES.iter().min().unwrap();
        let sample_max = *IDENTITY_WITNESS_SAMPLES.iter().max().unwrap();
        let highs: Vec<i64> = IDENTITY_WITNESS_CANDIDATES
            .iter()
            .copied()
            .filter(|&c| c > sample_max)
            .collect();
        let lows: Vec<i64> = IDENTITY_WITNESS_CANDIDATES
            .iter()
            .copied()
            .filter(|&c| c < sample_min)
            .collect();
        assert!(
            highs.len() >= 2,
            "need ≥2 candidates strictly above sample max ({sample_max}) so min-like lenses cannot \
             mint a unique false identity; got {highs:?}"
        );
        assert!(
            lows.len() >= 2,
            "need ≥2 candidates strictly below sample min ({sample_min}) for symmetric fail-closure \
             skew; got {lows:?}"
        );
    }

    #[test]
    fn int_add_lens_associativity_sample() {
        let src = r#"module w
fn lens_composition_op(a: Int, b: Int) -> Int = a + b
"#;
        let dag = compile_to_dag(src, "assoc.v3").expect("compiles");
        let id = dag.declaration_by_name("lens_composition_op").unwrap().id;
        assert!(
            int_associativity_holds_all_triples(&dag, id, ASSOCIATIVITY_WITNESS_TRIPLES)
                .expect("assoc witness"),
            "Int `+` lens must pass every ASSOCIATIVITY_WITNESS_TRIPLES entry"
        );
    }

    #[test]
    fn int_add_lens_identity_witness_finds_zero_uniquely() {
        let src = r#"module w
fn lens_composition_op(a: Int, b: Int) -> Int = a + b
"#;
        let dag = compile_to_dag(src, "id_add.v3").expect("compiles");
        let id = dag.declaration_by_name("lens_composition_op").unwrap().id;
        assert!(
            super::int_identity_witness_holds(
                &dag,
                id,
                IDENTITY_WITNESS_SAMPLES,
                IDENTITY_WITNESS_CANDIDATES
            )
            .expect("identity witness"),
            "Int `+` lens identity must be uniquely 0 on the bounded witness tables"
        );
    }

    #[test]
    fn int_mul_lens_identity_witness_finds_one_uniquely() {
        let src = r#"module w
fn lens_mul_op(a: Int, b: Int) -> Int = a * b
"#;
        let dag = compile_to_dag(src, "id_mul.v3").expect("compiles");
        let id = dag.declaration_by_name("lens_mul_op").unwrap().id;
        assert!(
            super::int_identity_witness_holds(
                &dag,
                id,
                IDENTITY_WITNESS_SAMPLES,
                IDENTITY_WITNESS_CANDIDATES
            )
            .expect("identity witness"),
            "Int `*` lens identity must be uniquely 1 on the bounded witness tables"
        );
    }

    #[test]
    fn int_min_lens_identity_witness_fails_closed_on_ambiguous_top_candidates() {
        let src = r#"module w
fn lens_min_op(a: Int, b: Int) -> Int = if a < b then a else b
"#;
        let dag = compile_to_dag(src, "id_min.v3").expect("compiles");
        let id = dag.declaration_by_name("lens_min_op").unwrap().id;
        assert!(
            !super::int_identity_witness_holds(
                &dag,
                id,
                IDENTITY_WITNESS_SAMPLES,
                IDENTITY_WITNESS_CANDIDATES
            )
            .expect("identity witness"),
            "binary Int min has no identity on bounded samples; bounded witness must fail closed"
        );
    }

    #[test]
    fn int_add_overflow_returns_lens_apply_error_not_wrapped_value() {
        let src = r#"module m
fn f(a: Int, b: Int) -> Int = a + b
"#;
        let dag = compile_to_dag(src, "int_ovf.v3").expect("compiles");
        let id = dag.declaration_by_name("f").unwrap().id;
        let err = apply_lens_declaration(
            &dag,
            id,
            &[
                FieldValue::Literal(literal_bits_int(i64::MAX)),
                FieldValue::Literal(literal_bits_int(1)),
            ],
        )
        .expect_err("overflow must not yield a wrapped Int");
        assert!(
            matches!(
                err,
                super::LensApplyError::UnsupportedConstruct("integer overflow")
            ),
            "{err:?}"
        );
    }

    #[test]
    fn empty_fold_returns_init() {
        let src = r#"module m
import std.list { List, fold }

fn sum(xs: List<Int>) -> Int =
  fold(xs, 99, |acc, x| acc + x)
"#;
        let dag = compile_to_dag(src, "empty_fold.v3").expect("compiles");
        let sum_id = dag.declaration_by_name("sum").expect("sum").id;
        let empty = empty_substrate_list_value(&dag).expect("empty list");
        let out = apply_lens_declaration(&dag, sum_id, &[empty]).expect("fold empty");
        assert_eq!(out, FieldValue::Literal(literal_bits_int(99)));
    }

    #[test]
    fn monomorphized_fold_step_bind_recovered_via_instantiation_arguments() {
        let src = r#"module m
import std.list { List, fold }

fn sum(xs: List<Int>) -> Int =
  fold(xs, 99, |acc, x| acc + x)
"#;
        let dag = compile_to_dag(src, "mono_fold_bind_paths.v3").expect("compiles");
        let fold_transform = dag
            .nodes()
            .iter()
            .find_map(|n| {
                let Behavior::Transform(t) = n else {
                    return None;
                };
                let TransformTarget::Callable(callee) = &t.target else {
                    return None;
                };
                super::is_fold_instantiation(&dag, dag.declaration(*callee)).then_some(t)
            })
            .expect("monomorphized fold transform");
        let TransformTarget::Callable(callee) = &fold_transform.target else {
            unreachable!();
        };
        super::find_fold_step_bind_via_instantiation(&dag, *callee).expect("fold step bind");
    }

    #[test]
    fn fold_step_lookup_requires_template_formal_edge() {
        let src = r#"module m
import std.list { List, fold }

fn sum(xs: List<Int>) -> Int =
  fold(xs, 99, |acc, x| acc + x)
"#;
        let mut dag = compile_to_dag(src, "mono_fold_bind_no_formal_edge.v3").expect("compiles");
        let fold_callable = dag
            .nodes()
            .iter()
            .find_map(|n| {
                let Behavior::Transform(t) = n else {
                    return None;
                };
                let TransformTarget::Callable(callee) = &t.target else {
                    return None;
                };
                super::is_fold_instantiation(&dag, dag.declaration(*callee)).then_some(*callee)
            })
            .expect("monomorphized fold transform");
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &dag.declaration(fold_callable).connective
        else {
            panic!("fold callable should be an Instantiation");
        };
        let formals = super::fold_template_callable_formals(&dag, *template);
        let depth_budget = LENS_APPLY_TYPE_WALK_DEPTH as i64;
        let callable_arg_index = arguments
            .iter()
            .position(|arg| {
                let resolved = resolve_template_argument_value(&depth_budget, arguments, arg.value);
                super::monomorph_callable_bind_root(&dag, resolved).is_some()
                    && formals.contains(&arg.parameter)
            })
            .expect("callable fold argument");
        let non_formal_parameter = arguments
            .iter()
            .map(|arg| arg.parameter)
            .find(|parameter| !formals.contains(parameter))
            .expect("non-callable fold formal parameter");

        let TypeConnective::Instantiation { arguments, .. } =
            &mut dag.declaration_mut(fold_callable).connective
        else {
            panic!("fold callable should remain an Instantiation");
        };
        arguments[callable_arg_index].parameter = non_formal_parameter;

        assert!(
            super::find_fold_step_bind_via_instantiation(&dag, fold_callable).is_none(),
            "unique callable candidates must not be accepted without the template-formal edge"
        );
    }

    #[test]
    fn comparison_operator_returns_bool_literal_not_sum_variant() {
        let src = r#"module m
fn truth() -> Bool = 1 == 1
"#;
        let dag = compile_to_dag(src, "truth.v3").expect("compiles");
        let id = dag.declaration_by_name("truth").unwrap().id;
        let out = apply_lens_declaration(&dag, id, &[]).expect("apply");
        assert_eq!(out, FieldValue::Literal(LiteralBits::Bool(true)));
    }

    #[test]
    fn bool_literal_scrutinee_selects_disj_match_arm() {
        let src = r#"module m
fn p(b: Bool) -> Int = match b { True => 1, False => 0 }
"#;
        let dag = compile_to_dag(src, "bool_lit_match.v3").expect("compiles");
        let id = dag.declaration_by_name("p").unwrap().id;
        let out = apply_lens_declaration(&dag, id, &[FieldValue::Literal(LiteralBits::Bool(true))])
            .expect("apply");
        assert_eq!(out, FieldValue::Literal(literal_bits_int(1)));
    }

    #[test]
    fn mutual_recursion_lens_hits_unimplemented_loop_for_descent_cluster() {
        let src = r#"module m
fn even(n: Int) -> Bool = if n == 0 then true else odd(n - 1)
fn odd(n: Int) -> Bool = if n == 0 then false else even(n - 1)
fn run_even(n: Int) -> Bool = even(n)
"#;
        let dag = compile_to_dag(src, "mutual_lens.v3").expect("compiles");
        assert!(
            dag.nodes()
                .iter()
                .filter_map(Behavior::as_loop)
                .any(|l| { matches!(l.bound, LoopBound::Descent { .. }) }),
            "expected at least one LoopBound::Descent (mutual cluster)"
        );
        let run_even = dag.declaration_by_name("run_even").expect("run_even").id;
        let err =
            apply_lens_declaration(&dag, run_even, &[FieldValue::Literal(literal_bits_int(1))])
                .expect_err("loop interpretation is unimplemented");
        assert!(
            matches!(err, LensApplyError::UnimplementedLoopBound),
            "expected UnimplementedLoopBound, got {err:?}"
        );
    }

    #[test]
    fn apply_lens_declaration_arity_mismatch() {
        let src = r#"module m
fn f(a: Int, b: Int) -> Int = a + b
"#;
        let dag = compile_to_dag(src, "arity.v3").expect("compiles");
        let id = dag.declaration_by_name("f").unwrap().id;
        let err = apply_lens_declaration(&dag, id, &[FieldValue::Literal(literal_bits_int(1))])
            .expect_err("wrong arity");
        assert!(
            matches!(
                err,
                LensApplyError::ArityMismatch {
                    expected: 2,
                    got: 1
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn eligible_fold_runs_through_bounded_path() {
        // Step body is `acc + x` — no Loop, no unsupported construct in transitive call graph.
        // The structural eligibility predicate must approve and the bounded fast path must
        // produce the sum.
        let src = r#"module m
import std.list { List, fold }

fn sum(xs: List<Int>) -> Int =
  fold(xs, 0, |acc, x| acc + x)
"#;
        let dag = compile_to_dag(src, "eligible_fold.v3").expect("compiles");
        let sum_id = dag.declaration_by_name("sum").expect("sum").id;
        let (empty_id, cons_id) = super::v3_list_empty_cons_ids(&dag).expect("list ids");
        let int = |n: i64| FieldValue::Literal(literal_bits_int(n));
        let mut list = FieldValue::Variant {
            constructor: empty_id,
            payload: vec![],
        };
        for n in [3i64, 2, 1] {
            list = FieldValue::Variant {
                constructor: cons_id,
                payload: vec![int(n), list],
            };
        }
        let out = apply_lens_declaration(&dag, sum_id, &[list]).expect("eligible fold runs");
        assert_eq!(out, int(6));
    }

    #[test]
    fn ineligible_fold_with_recursive_step_skips_bounded_path() {
        // `helper` is self-recursive → lowers to a `LoopNode` in the step body's transitive
        // call graph. The structural eligibility predicate must reject; the bounded fast path
        // is bypassed; `eval_callable` rejects the monomorphized `Instantiation { template:
        // fold }` with `UnimplementedCallable` (fail-closed, vs running and hitting
        // `UnimplementedLoopBound` deeper in).
        let src = r#"module m
import std.list { List, fold }

fn helper(n: Int) -> Int = if n == 0 then 0 else helper(n - 1) + 1
fn sum(xs: List<Int>) -> Int =
  fold(xs, 0, |acc, x| acc + helper(x))
"#;
        let dag = compile_to_dag(src, "ineligible_fold.v3").expect("compiles");
        let sum_id = dag.declaration_by_name("sum").expect("sum").id;
        let (_, cons_id) = super::v3_list_empty_cons_ids(&dag).expect("list ids");
        let empty = empty_substrate_list_value(&dag).expect("empty list");
        let one = FieldValue::Variant {
            constructor: cons_id,
            payload: vec![FieldValue::Literal(literal_bits_int(1)), empty],
        };
        let err = apply_lens_declaration(&dag, sum_id, &[one]).expect_err("ineligible step body");
        assert!(
            matches!(err, LensApplyError::UnimplementedCallable(_)),
            "{err:?}"
        );
    }

    #[test]
    fn branch_miss_when_scrutinee_matches_no_arm() {
        let src = r#"module m
fn p(b: Bool) -> Int = match b { True => 1, False => 0 }
"#;
        let dag = compile_to_dag(src, "branch_miss.v3").expect("compiles");
        let id = dag.declaration_by_name("p").unwrap().id;
        let err = apply_lens_declaration(&dag, id, &[FieldValue::Literal(literal_bits_int(42))])
            .expect_err("Int is not True/False");
        assert!(matches!(err, LensApplyError::BranchMiss), "{err:?}");
    }
}

// Substrate ↔ `FieldValue` structural reflection (design-reflection-completeness).
// SG-0: folded under this file — the census forbids a new sibling `substrate_reflection.rs`.
mod substrate_reflection {
    //! Complete structural reflection of computation-substrate [`Behavior`] nodes into
    //! lens-input [`FieldValue`] per `docs/design-reflection-completeness.md` (LOCKED).
    //! Reflection is static: no execution, no branch-arm selection, no loop iteration.

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
        FieldValue::Literal(LiteralBits::Int(i64::from(p.raw()).to_string()))
    }

    fn node_fv(n: NodeId) -> FieldValue {
        FieldValue::Literal(LiteralBits::Int(i64::from(n.raw()).to_string()))
    }

    fn cluster_fv(c: ClusterId) -> FieldValue {
        FieldValue::Literal(LiteralBits::Int(i64::from(c.raw()).to_string()))
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
                FieldValue::Literal(LiteralBits::Int(i64::from(span.byte_start).to_string())),
            ),
            (
                "end".to_string(),
                FieldValue::Literal(LiteralBits::Int(i64::from(span.byte_end).to_string())),
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
                &FieldValue::Literal(LiteralBits::Int(i64::from(v.span.byte_start).to_string()))
            );
            assert_eq!(
                record_get(span_rec, "end"),
                &FieldValue::Literal(LiteralBits::Int(i64::from(v.span.byte_end).to_string()))
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
}
