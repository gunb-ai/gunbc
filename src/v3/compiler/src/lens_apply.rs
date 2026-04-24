//! Bounded lens application (T-LensAPI / D1): interpret `ArrowBody::UserDefined` graphs
//! over substrate-shaped [`FieldValue`] — no whole-claim operator recognizers.

use std::collections::HashMap;

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BindNode, BranchNode, BranchPattern, Dag, Declaration,
    DeclarationId, FieldValue, LiteralBits, OperatorKind, PortId, TransformNode, TransformTarget,
    TypeConnective, ValueBody,
};
use crate::diagnostics::SourceSpan;
use crate::infer_helpers::resolve_template_argument_value;

fn is_fold_instantiation(dag: &Dag, decl: &Declaration) -> bool {
    matches!(
        &decl.connective,
        TypeConnective::Instantiation { template, .. }
            if dag.std_list_fold_decl() == Some(*template)
    )
}

/// Skip the monomorphized `std.list.fold` → [`eval_fold_step`] fast path for transforms whose
/// source site lives in `v3.std.algebra` (staged mirror: `dsl/std/algebra.dag`).
///
/// That module lowers `fold` over `List<SymbolicCost>` — including list-shaped accumulators —
/// with step bodies threading dominance / `normalize`. The bounded R1 interpreter only certifies
/// the small `named_function_count` fragment (e.g. `Int` acc, `Behavior` elements). Using the
/// fast path on algebra folds risks silent divergence from substrate semantics; skipping routes
/// through [`LensInterpreter::eval_callable`], which rejects monomorphized `Instantiation {{
/// template: fold }}` until a fuller interpreter lands (fail-closed vs wrong answers).
///
/// **Dissolution trigger:** replace this filter with a structural predicate on lowering facts
/// (R1-certified step shape, or third `Transform` operand for the step so arity matches
/// [`eval_std_fold`]) and delete this helper.
fn fold_site_skips_d1_monomorph_list_fold_path(span: &SourceSpan) -> bool {
    // `SourceSpan.file` is normalized to forward slashes in the lowering paths we exercise;
    // do not guess Windows separators until a span fact proves backslashes appear here.
    span.file.as_str().ends_with("std/algebra.dag")
}

const LENS_APPLY_TYPE_WALK_DEPTH: usize = 32;

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
        | TypeConnective::Cardinality { .. } => false,
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
                return match dag.node(*root) {
                    Behavior::Bind(b) if b.params.len() >= 2 => Some(b),
                    _ => None,
                };
            }
            _ => return None,
        }
    }
    None
}

/// Primary path: walk `Instantiation.arguments` on the monomorphized fold callee — substrate
/// `DeclarationId` keyed to the template's callable formal — then resolve the step `Arrow` root
/// `Bind`. Matches builder/runtime arity: the step is not a `Transform` input port.
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

    let mut ambiguous_fallback: Vec<&BindNode> = Vec::new();
    for arg in arguments {
        let resolved = resolve_template_argument_value(&depth_budget, arguments, arg.value);
        let Some(b) = monomorph_callable_bind_root(dag, resolved) else {
            continue;
        };
        if formals.contains(&arg.parameter) {
            return Some(b);
        }
        if !ambiguous_fallback
            .iter()
            .any(|existing| existing.id == b.id)
        {
            ambiguous_fallback.push(b);
        }
    }
    (ambiguous_fallback.len() == 1).then(|| ambiguous_fallback[0])
}

/// Locate the `|acc, x|` step closure lowered as a two-parameter `Bind` for this `fold` site.
///
/// Uses only [`find_fold_step_bind_via_instantiation`]: template-argument substrate edges from
/// the monomorphized `std.list.fold` `Instantiation` (the step is not a runtime `Transform` input
/// port when arity collapses to list + init). No span-overlap recovery — that would not be a
/// declared substrate dependency (Facts Flow Forward).
///
/// **Dissolution:** attach the step as an explicit `Transform` input / behavior edge and delete
/// the `Instantiation`-walk indirection when lowering guarantees a direct edge.
fn find_fold_step_bind(dag: &Dag, fold_callable_id: DeclarationId) -> Option<&BindNode> {
    find_fold_step_bind_via_instantiation(dag, fold_callable_id)
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

/// Build a substrate-shaped `Dag` record (only `nodes` is populated faithfully) from a
/// compiled program [`Dag`], for lenses like `named_function_count` that read `d.nodes`.
///
/// **Scaffold:** reflection is **lossy** relative to `std.substrate.Behavior` — e.g.
/// `Transform` records expose only `result_port` (no `target` / `inputs`); `Branch` /
/// `Loop` omit `paths`, bounds, and bodies. Lenses that only need `Bind.name` or a
/// shallow node list are supported; do not rely on full structural fidelity.
///
/// **Dissolution trigger:** dissolve this Rust-side mirror once `.dag` values on disk
/// (or an equivalent canonical carrier) are consumed directly by the interpreter
/// without a hand-rolled `Behavior` → [`FieldValue`] reflection pass.
///
/// `source_file` limits nodes to those authored in that compilation unit (the merged
/// bootstrap graph also lives in the same [`Dag`]).
pub fn reflect_program_dag_nodes_in_file(
    program: &Dag,
    source_file: &str,
) -> Result<FieldValue, LensApplyError> {
    let nodes: Vec<Behavior> = program
        .nodes()
        .iter()
        .filter(|b| behavior_source_file(b) == source_file)
        .cloned()
        .collect();
    let nodes = reflect_behavior_list(program, &nodes)?;
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
    MissingValueBody,
}

/// Lower a declaration [`ValueBody`] into the structural [`FieldValue`] carrier used by the
/// lens interpreter (references keep their [`DeclarationId`] edges from the fixture DAG).
pub fn field_value_from_value_body(
    _fixture_dag: &Dag,
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
                if is_fold_instantiation(self.dag, decl)
                    && t.inputs.len() == 2
                    && !fold_site_skips_d1_monomorph_list_fold_path(&t.span)
                {
                    let list = self.eval_port(t.inputs[0])?;
                    let init = self.eval_port(t.inputs[1])?;
                    let step_bind = find_fold_step_bind(self.dag, *callee).ok_or(
                        LensApplyError::UnsupportedConstruct(
                            "monomorphized fold: could not locate step closure Bind via \
                             Instantiation arguments (no span overlap fallback)",
                        ),
                    )?;
                    return self.eval_fold_step(list, init, step_bind);
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
                Ok(FieldValue::Literal(LiteralBits::Int(n)))
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
                    ) => match op {
                        crate::dag::ComparisonOp::Eq => a == b,
                        crate::dag::ComparisonOp::Ne => a != b,
                        crate::dag::ComparisonOp::Lt => a < b,
                        crate::dag::ComparisonOp::Le => a <= b,
                        crate::dag::ComparisonOp::Gt => a > b,
                        crate::dag::ComparisonOp::Ge => a >= b,
                    },
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
        FieldValue::Literal(LiteralBits::Int(n)) => Ok(*n),
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
    _dag: &Dag,
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

/// See [`reflect_program_dag_nodes_in_file`] scaffold note — lossy `Behavior` spine.
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

/// Lossy mirror of one [`Behavior`] (see [`reflect_program_dag_nodes_in_file`]).
fn reflect_behavior(dag: &Dag, behavior: &Behavior) -> Result<FieldValue, LensApplyError> {
    match behavior {
        Behavior::Value(v) => {
            let id = behavior_variant_id(dag, "Value")?;
            let payload = FieldValue::Record(vec![
                ("payload".to_string(), FieldValue::Literal(v.data.clone())),
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
        let input =
            reflect_program_dag_nodes_in_file(&prog, "lens_apply_prog.v3").expect("reflect");
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
        assert!(
            int_associativity_holds_all_triples(&dag, id, ASSOCIATIVITY_WITNESS_TRIPLES)
                .expect("assoc witness"),
            "Int `+` lens must pass every ASSOCIATIVITY_WITNESS_TRIPLES entry"
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
                FieldValue::Literal(LiteralBits::Int(i64::MAX)),
                FieldValue::Literal(LiteralBits::Int(1)),
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
        assert_eq!(out, FieldValue::Literal(LiteralBits::Int(99)));
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
        let via_inst =
            super::find_fold_step_bind_via_instantiation(&dag, *callee).expect("inst path");
        let combined = super::find_fold_step_bind(&dag, *callee).expect("find_fold_step_bind");
        assert_eq!(combined.id, via_inst.id);
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
            apply_lens_declaration(&dag, run_even, &[FieldValue::Literal(LiteralBits::Int(1))])
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
        let err = apply_lens_declaration(&dag, id, &[FieldValue::Literal(LiteralBits::Int(1))])
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
    fn branch_miss_when_scrutinee_matches_no_arm() {
        let src = r#"module m
fn p(b: Bool) -> Int = match b { True => 1, False => 0 }
"#;
        let dag = compile_to_dag(src, "branch_miss.v3").expect("compiles");
        let id = dag.declaration_by_name("p").unwrap().id;
        let err = apply_lens_declaration(&dag, id, &[FieldValue::Literal(LiteralBits::Int(42))])
            .expect_err("Int is not True/False");
        assert!(matches!(err, LensApplyError::BranchMiss), "{err:?}");
    }
}
