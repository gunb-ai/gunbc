// v3 compiler — M0 substrate skeleton.
//
// Pipeline (target end state for M0):
//   source text -> tokenize -> parse -> lower to L1 behaviors -> infer -> Dag
//
// Fail-closed compile boundary (invariant C-8):
//   compile_to_dag returns Ok(Dag) ONLY when the diagnostic table
//   is empty. Any semantic errors (type mismatches, unresolved
//   names, arity errors, etc.) surface as Err(CompileError::Semantic(dag))
//   — the dag is still handed back so the caller can inspect the
//   diagnostics, but the Result variant is Err.
//
//   Structural errors (tokenize/parse) surface as their own variants
//   because they occur before a Dag exists. G5: no TypeError variant
//   on CompileError — type errors live on the Dag, not in the Err
//   payload.

pub mod complexity_lattice;
pub mod dag;
pub mod diagnostics;
mod enforced_lens_application;
pub mod lens_t_las_carrier;
pub mod pb_method_template_projection;
pub mod pb_method_template_projection_dag_emit;
mod regen_bootstrap_emit;
pub mod regen_tokenize;

/// SG-0 producer-owned generated-file manifest.
///
/// `GENERATED_FILES` is the workspace-relative path list of every
/// `.rs` file under `src/v3/compiler` that is produced by a codegen
/// authority. The list is emitted by `build.rs` at build time; the
/// literal is reviewed there. Two consumers today: the `regen_*`
/// binaries (they assert their output path is in the list before
/// writing) and the SG-0 census test (it uses the list as the sole
/// generated/hand-authored partition — no content-marker scanning).
pub mod generated_files {
    include!(concat!(env!("OUT_DIR"), "/v3_generated_files.rs"));
}

pub mod emit;
pub mod emit_rust;
pub mod emit_rust_bin_shim;
pub mod omni_shape_b_openapi;
pub mod process_exit;
pub mod self_host_receipt_p0;
pub mod evaluator {
    //! E2 evaluator frame helpers.
    //!
    //! This module is the narrow Rust realization of the existing
    //! `EvalFrame { bindings: Map<PortId, Value> }` and
    //! `EvalStateStack { frames: List<EvalFrame> }` substrate carriers. The
    //! value payload is generic on purpose: E2 owns binding-scope behavior,
    //! not a new observable `Value` carrier. Later body-evaluator slices plug
    //! in the runtime value representation without changing the frame rules
    //! here.
    //!
    //! Dissolution target: when the `.dag` evaluator body implementation owns
    //! frame mutation directly, this host helper should shrink to generated or
    //! substrate-backed calls instead of becoming a parallel evaluator runtime.

    use std::collections::HashMap;
    use std::str::FromStr;

    use num_bigint::BigInt;

    use crate::dag::{
        literal_bits_int, ArithmeticOp, ArrowBody, Behavior, BranchNode, BranchPattern, ClusterId,
        ComparisonOp, Dag, DeclarationId, LiteralBits, LoopBound, NodeId, OperatorKind, Path,
        PortId, TransformNode, TransformTarget, TypeConnective,
    };

    /// Rust mirror of the substrate runtime `Value` carrier in
    /// `src/v3/std/runtime.dag`.
    ///
    /// **Dissolution receipt: TERMINAL.** This is not a second value authority:
    /// it has the same five inhabitants as the `.dag` carrier and exists so the
    /// Rust eager evaluator can return typed runtime data until generated
    /// substrate-backed calls replace this host helper.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Value {
        LiteralValue(LiteralBits),
        RecordValue(Vec<NamedField>),
        VariantValue {
            tag: DeclarationId,
            payload: Box<Value>,
        },
        NodeRef(NodeId),
        CardinalityValue(LoopBound),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NamedField {
        pub label: String,
        pub value: Value,
    }

    /// Rust evaluator consumer mirror of `std.termination::StrictEvidence`.
    ///
    /// The `.dag` carrier is the authority; this mirror exists only because
    /// the eager Rust evaluator cannot yet call generated std block bodies.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum StrictEvidence {
        Strict,
    }

    /// Rust evaluator consumer mirror of `std.termination::NonStrictEvidence`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NonStrictEvidence {
        NonIncreasing,
        DescentUnknown,
    }

    /// Rust evaluator consumer mirror of `std.termination::DescentResidual`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DescentResidual {
        EvidenceUnknown(NonStrictEvidence),
        EvidenceIncomplete,
    }

    /// Rust evaluator consumer mirror of
    /// `std.termination::DescentExecutionProof`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DescentExecutionProof {
        pub cluster: ClusterId,
        pub port: PortId,
        pub per_path: HashMap<String, StrictEvidence>,
    }

    fn descent_execution_proof(
        _dag: &Dag,
        _cluster: ClusterId,
        _port: PortId,
    ) -> Result<DescentExecutionProof, DescentResidual> {
        Err(DescentResidual::EvidenceIncomplete)
    }

    /// Rust mirror of the PR-A.3 eager-baseline strategy carrier.
    ///
    /// **Dissolution receipt: TERMINAL at PR-A.3 eager-baseline scope.** The
    /// public evaluator boundary carries strategy now so downstream slices
    /// cannot erase it, but the mirror stays identical to `runtime.dag`:
    /// exactly `ApplicativeOrder / LeftFirst`. Additional inhabitants must land
    /// with substrate carriers and executable evaluator rules.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum EvalStrategy {
        ApplicativeOrder { input_order: InputEvaluationOrder },
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum InputEvaluationOrder {
        LeftFirst,
    }

    /// **Dissolution receipt: TERMINAL.** Typed fail-closed outcomes for
    /// the body evaluator: missing-substrate cases (`MissingNode`,
    /// `UnboundPort`), E3 transform
    /// operand / arity diagnostics (`TransformArityMismatch`,
    /// `UnsupportedTransformTarget`, `BadTransformOperands`), Branch
    /// resolution / shape / payload-frame errors (E4), and frame
    /// discipline propagation (`FrameError`). E5 adds loop-bound
    /// diagnostics from the readiness audit: descent residual,
    /// non-integer cardinality, and negative cardinality. Adding any
    /// further variant is a STOP+PING per the E0 brief — either route the
    /// underlying gap through P1 or extend PR-B.1's fail-closed catalog
    /// first.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum EvalError {
        MissingNode {
            node: NodeId,
        },
        UnboundPort {
            port: PortId,
        },
        TransformArityMismatch {
            expected: usize,
            got: usize,
        },
        UnsupportedTransformTarget {
            kind: &'static str,
        },
        BadTransformOperands {
            reason: &'static str,
        },
        /// E4 fail-closed: a `BranchPath.pattern` is still
        /// `UnresolvedVariant` at evaluation time. Resolution must lower
        /// `UnresolvedVariant` to `ResolvedVariant(DeclarationId)` before
        /// the body evaluator runs. Reaching this variant is a resolution
        /// gap, not a runtime case.
        BranchUnresolvedVariant {
            node: NodeId,
            name: String,
        },
        /// E4 fail-closed: scrutinee `Value` is not `VariantValue`, so no
        /// `BranchPattern::ResolvedVariant(tag)` can match. Substrate +
        /// inference guarantee totality on well-typed programs; reaching
        /// this is an invariant violation, not a runtime case.
        BranchScrutineeShape {
            node: NodeId,
        },
        /// E4 fail-closed: scrutinee is a `VariantValue { tag, .. }` but
        /// no `BranchPath` carries `pattern: ResolvedVariant(decl)` with
        /// `decl == tag`. Same invariant-violation framing.
        BranchNoMatchingArm {
            node: NodeId,
            tag: DeclarationId,
        },
        /// E5 fail-closed: `LoopBound::Descent` execution requires
        /// termination evidence and is explicitly deferred by the E5
        /// readiness audit. The `measure` port is carried so diagnostics
        /// identify the runtime value that would need descent evidence.
        LoopBoundDescentResidual {
            node: NodeId,
            cluster: crate::dag::ClusterId,
            measure: PortId,
        },
        /// E5 fail-closed: cardinality loops accept only a non-negative
        /// runtime `Int` literal as the bounded iteration witness.
        LoopCardinalityNonInteger {
            node: NodeId,
            count: PortId,
        },
        /// E5 fail-closed: negative counts are not cardinality witnesses.
        LoopCardinalityNegative {
            node: NodeId,
            count: PortId,
            value: i64,
        },
        /// E5 fail-closed: count is non-negative but cannot fit in the
        /// host iteration counter without truncation.
        LoopCardinalityTooLarge {
            node: NodeId,
            count: PortId,
            value: i64,
        },
        /// E4 / E2 propagation: an `EvalFrameError` produced during
        /// frame discipline (push/pop balance, duplicate bind, unbound
        /// port). Carries the underlying `EvalFrameError` for
        /// diagnostic locality.
        FrameError(EvalFrameError),
    }

    impl From<EvalFrameError> for EvalError {
        fn from(err: EvalFrameError) -> Self {
            EvalError::FrameError(err)
        }
    }

    pub fn eval_value(value: &crate::dag::ValueNode) -> Value {
        Value::LiteralValue(value.data.clone())
    }

    fn reify_bool_literal_for_branch_scrutinee(dag: &Dag, value: Value) -> Value {
        match value {
            Value::LiteralValue(LiteralBits::Bool(b)) => dag
                .bool_runtime_variant_id(b)
                .map(|tag| Value::VariantValue {
                    tag,
                    payload: Box::new(Value::RecordValue(Vec::new())),
                })
                .unwrap_or(Value::LiteralValue(LiteralBits::Bool(b))),
            other => other,
        }
    }

    pub fn eval_port(
        dag: &Dag,
        port: PortId,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        let Some(port_ref) = dag.port_opt(&port) else {
            return Err(EvalError::UnboundPort { port });
        };
        if let Some(producer) = port_ref.produced_by {
            return eval_node(dag, producer, state, strategy);
        }
        state
            .lookup(port)
            .cloned()
            .map_err(|_| EvalError::UnboundPort { port })
    }

    pub fn eval_node(
        dag: &Dag,
        node: NodeId,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        match strategy {
            EvalStrategy::ApplicativeOrder {
                input_order: InputEvaluationOrder::LeftFirst,
            } => {}
        }
        match dag.node_opt(&node).ok_or(EvalError::MissingNode { node })? {
            Behavior::Value(value) => Ok(eval_value(value)),
            Behavior::Transform(t) => eval_transform_node(dag, t, state, strategy),
            Behavior::Branch(branch) => eval_branch(dag, branch.clone(), state, strategy),
            Behavior::Loop(loop_node) => eval_loop(dag, loop_node.clone(), state, strategy),
            Behavior::Bind(bind) => eval_bind(dag, bind.clone(), state, strategy),
        }
    }

    /// PR-E E4: evaluate a `Branch` node per PR-B.1 §B.1.3 — eager
    /// scrutinee evaluation, exact `ResolvedVariant` tag match,
    /// payload binding in a fresh frame, body evaluation through
    /// `eval_node`. Frame push/pop is balanced on both success and
    /// diagnostic paths so the stack invariant survives errors.
    pub fn eval_branch(
        dag: &Dag,
        branch: BranchNode,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        let scrutinee = reify_bool_literal_for_branch_scrutinee(
            dag,
            eval_port(dag, branch.input, state, strategy)?,
        );
        let (tag, payload) = match scrutinee {
            Value::VariantValue { tag, payload } => (tag, payload),
            _ => return Err(EvalError::BranchScrutineeShape { node: branch.id }),
        };
        let path = select_branch_path(&branch, tag)?;
        state.push_frame(EvalFrame::empty());
        let body_result =
            eval_branch_body_in_pushed_frame(dag, branch.id, &path, *payload, state, strategy);
        let pop_result = state.pop_frame();
        // The body result is authoritative; only surface a frame
        // error if the body otherwise succeeded.
        match (body_result, pop_result) {
            (Ok(value), Ok(_)) => Ok(value),
            (Err(err), _) => Err(err),
            (Ok(_), Err(frame_err)) => Err(EvalError::from(frame_err)),
        }
    }

    fn select_branch_path(branch: &BranchNode, tag: DeclarationId) -> Result<Path, EvalError> {
        // Pass 1 (Fail-Closed / C-8): every `BranchPath.pattern` must be
        // `ResolvedVariant` before evaluation proceeds. Any
        // `UnresolvedVariant` — even on a path that wouldn't have been
        // selected — is a substrate-resolution invariant violation and
        // surfaces before arm matching, so a matching early arm cannot
        // mask a later unresolved arm.
        for path in &branch.paths {
            if let BranchPattern::UnresolvedVariant { name, .. } = &path.pattern {
                return Err(EvalError::BranchUnresolvedVariant {
                    node: branch.id,
                    name: name.clone(),
                });
            }
        }
        // Pass 2: select the arm whose `ResolvedVariant.tag` matches.
        for path in &branch.paths {
            if let BranchPattern::ResolvedVariant(decl) = &path.pattern {
                if *decl == tag {
                    return Ok(path.clone());
                }
            }
        }
        Err(EvalError::BranchNoMatchingArm {
            node: branch.id,
            tag,
        })
    }

    fn eval_branch_body_in_pushed_frame(
        dag: &Dag,
        branch_id: NodeId,
        path: &Path,
        payload: Value,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        if let Some(binding) = &path.binding {
            state.bind_top(binding.payload_port, payload)?;
        }
        // `lower.rs` uses `path.body == branch.id` as the
        // **producerless-arm sentinel**: the arm's result port has no
        // producer node (returns an existing or payload-bound port
        // directly), so the lowerer points body back at the Branch
        // node itself. Skip body evaluation in that case — recursing
        // through `eval_node` would re-enter `eval_branch` on the
        // same node and either loop forever or drop the authoritative
        // `path.output` fact. The arm's value is read directly from
        // `path.output` via `eval_port`.
        if path.body != branch_id {
            // Otherwise: evaluate the body for its frame-binding side
            // effects (E6 Bind, future E3 Transform). Body diagnostic
            // propagates through `?`. The body's local return value
            // is intentionally discarded — the arm's authoritative
            // value is at `path.output`, per
            // `BranchPath.output: PortId` in the substrate; an arm
            // whose `output` is an existing or payload-bound port
            // would drop that fact if the evaluator returned the
            // body's local result instead.
            let _ = eval_node(dag, path.body, state, strategy)?;
        }
        eval_port(dag, path.output, state, strategy)
    }

    /// PR-E E5: bounded eager loop execution. `LoopBound::Cardinality`
    /// evaluates by count. `LoopBound::Descent` is wired through the
    /// `descent_execution_proof` proof hook, but the live default hook remains
    /// fail-closed with `EvidenceIncomplete` until the substrate producer
    /// emits per-path strict evidence. Tests inject that proof hook to exercise
    /// the consumer success and residual paths.
    pub fn eval_loop(
        dag: &Dag,
        loop_node: crate::dag::LoopNode,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        eval_loop_with_descent_execution_proof(
            dag,
            loop_node,
            state,
            strategy,
            descent_execution_proof,
        )
    }

    fn eval_loop_with_descent_execution_proof(
        dag: &Dag,
        loop_node: crate::dag::LoopNode,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
        proof_fn: impl Fn(&Dag, ClusterId, PortId) -> Result<DescentExecutionProof, DescentResidual>,
    ) -> Result<Value, EvalError> {
        let mut acc = eval_port(dag, loop_node.init, state, strategy)?;
        let count = match &loop_node.bound {
            LoopBound::Cardinality { count } => {
                decode_loop_cardinality_count(dag, loop_node.id, *count, state, strategy)?
            }
            LoopBound::Descent { cluster, measure } => {
                let proof = proof_fn(dag, *cluster, *measure).map_err(|_| {
                    EvalError::LoopBoundDescentResidual {
                        node: loop_node.id,
                        cluster: *cluster,
                        measure: *measure,
                    }
                })?;
                discharge_descent_obligation(dag, loop_node.id, *cluster, *measure, proof)?;
                // Current E2 consumer scope discharges the termination
                // obligation and executes one descent step. Body-to-convergence
                // semantics belong to the follow-on descent producer/runtime.
                1
            }
        };
        for _ in 0..count {
            state.push_frame(EvalFrame::empty());
            let body_result = eval_loop_iteration_body(dag, &loop_node, acc, state, strategy);
            let pop_result = state.pop_frame();
            match (body_result, pop_result) {
                (Ok(next), Ok(_)) => acc = next,
                (Err(err), _) => return Err(err),
                (Ok(_), Err(frame_err)) => return Err(EvalError::from(frame_err)),
            }
        }
        Ok(acc)
    }

    fn discharge_descent_obligation(
        dag: &Dag,
        node: NodeId,
        cluster: ClusterId,
        measure: PortId,
        proof: DescentExecutionProof,
    ) -> Result<(), EvalError> {
        if proof.cluster == cluster
            && proof.port == measure
            && dag.cluster(cluster).intra_cluster_calls.iter().all(|call| {
                proof
                    .per_path
                    .contains_key(&descent_proof_path_key(call.transform))
            })
        {
            Ok(())
        } else {
            Err(EvalError::LoopBoundDescentResidual {
                node,
                cluster,
                measure,
            })
        }
    }

    fn descent_proof_path_key(transform: crate::dag::TransformRef) -> String {
        transform.node_id().raw().to_string()
    }

    fn eval_loop_iteration_body(
        dag: &Dag,
        loop_node: &crate::dag::LoopNode,
        acc: Value,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        state.bind_top(loop_node.source, acc)?;
        eval_node(dag, loop_node.body, state, strategy)
    }

    fn decode_loop_cardinality_count(
        dag: &Dag,
        node: NodeId,
        count: PortId,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<usize, EvalError> {
        match eval_port(dag, count, state, strategy)? {
            Value::LiteralValue(LiteralBits::Int(n)) if n >= 0 => {
                usize::try_from(n).map_err(|_| EvalError::LoopCardinalityTooLarge {
                    node,
                    count,
                    value: n,
                })
            }
            Value::LiteralValue(LiteralBits::Int(n)) => Err(EvalError::LoopCardinalityNegative {
                node,
                count,
                value: n,
            }),
            _ => Err(EvalError::LoopCardinalityNonInteger { node, count }),
        }
    }

    /// PR-E Bind/callable-entry prerequisite: evaluate the body port in a
    /// fresh callable frame populated from the caller-visible parameter
    /// bindings. Argument evaluation / callable dispatch stays outside this
    /// slice; callers must have already registered `BindNode.params` in an
    /// outer frame.
    pub fn eval_bind(
        dag: &Dag,
        bind: crate::dag::BindNode,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        if bind.params.is_empty() {
            return eval_port(dag, bind.value, state, strategy);
        }
        let bindings = bind
            .params
            .iter()
            .map(|param| {
                state
                    .lookup(*param)
                    .cloned()
                    .map(|value| (*param, value))
                    .map_err(EvalError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
        state.push_frame(EvalFrame::empty());
        let body_result = eval_bind_body_in_pushed_frame(dag, &bind, bindings, state, strategy);
        let pop_result = state.pop_frame();
        match (body_result, pop_result) {
            (Ok(value), Ok(_)) => Ok(value),
            (Err(err), _) => Err(err),
            (Ok(_), Err(frame_err)) => Err(EvalError::from(frame_err)),
        }
    }

    fn eval_bind_body_in_pushed_frame(
        dag: &Dag,
        bind: &crate::dag::BindNode,
        bindings: Vec<(PortId, Value)>,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        for (param, value) in bindings {
            state.bind_top(param, value)?;
        }
        eval_port(dag, bind.value, state, strategy)
    }

    fn eval_transform_node(
        dag: &Dag,
        t: &TransformNode,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        let mut operands = Vec::with_capacity(t.inputs.len());
        for port in &t.inputs {
            operands.push(eval_port(dag, *port, state, strategy)?);
        }
        match &t.target {
            TransformTarget::Operator(OperatorKind::Arithmetic(op)) => {
                if operands.len() != 2 {
                    return Err(EvalError::TransformArityMismatch {
                        expected: 2,
                        got: operands.len(),
                    });
                }
                let a = expect_int_literal(&operands[0])?;
                let b = expect_int_literal(&operands[1])?;
                // `Div` is totalized as `Result<T, DivError>` on the transform port in inference.
                // The host `Value` carrier has no std `Result` projection yet — fail closed here
                // instead of routing `/` through the same `i64` success path as `+`/`-`/`*`.
                if matches!(op, ArithmeticOp::Div) {
                    return Err(EvalError::UnsupportedTransformTarget {
                        kind: "ArithmeticDiv",
                    });
                }
                let n = apply_arithmetic_int_ring_only(*op, a, b)?;
                Ok(Value::LiteralValue(LiteralBits::Int(n)))
            }
            TransformTarget::Operator(OperatorKind::Comparison(op)) => {
                if operands.len() != 2 {
                    return Err(EvalError::TransformArityMismatch {
                        expected: 2,
                        got: operands.len(),
                    });
                }
                let out = match (&operands[0], &operands[1]) {
                    (
                        Value::LiteralValue(LiteralBits::Int(a)),
                        Value::LiteralValue(LiteralBits::Int(b)),
                    ) => match op {
                        ComparisonOp::Eq => a == b,
                        ComparisonOp::Ne => a != b,
                        ComparisonOp::Lt => a < b,
                        ComparisonOp::Le => a <= b,
                        ComparisonOp::Gt => a > b,
                        ComparisonOp::Ge => a >= b,
                    },
                    (
                        Value::LiteralValue(LiteralBits::Bool(a)),
                        Value::LiteralValue(LiteralBits::Bool(b)),
                    ) => match op {
                        ComparisonOp::Eq => a == b,
                        ComparisonOp::Ne => a != b,
                        ComparisonOp::Lt => a < b,
                        ComparisonOp::Le => a <= b,
                        ComparisonOp::Gt => a > b,
                        ComparisonOp::Ge => a >= b,
                    },
                    (
                        Value::LiteralValue(LiteralBits::String(a)),
                        Value::LiteralValue(LiteralBits::String(b)),
                    ) => match op {
                        ComparisonOp::Eq => a == b,
                        ComparisonOp::Ne => a != b,
                        _ => {
                            return Err(EvalError::BadTransformOperands {
                                reason: "string comparison beyond Eq/Ne",
                            });
                        }
                    },
                    _ => {
                        return Err(EvalError::BadTransformOperands {
                            reason:
                                "comparison operands must be Int literals, Bool literals, or String literals (Eq/Ne only for String)",
                        });
                    }
                };
                Ok(Value::LiteralValue(LiteralBits::Bool(out)))
            }
            TransformTarget::Operator(OperatorKind::Logical(_)) => {
                Err(EvalError::UnsupportedTransformTarget { kind: "Logical" })
            }
            TransformTarget::FieldProject {
                field_label,
                field_child: _,
            } => {
                if operands.len() != 1 {
                    return Err(EvalError::TransformArityMismatch {
                        expected: 1,
                        got: operands.len(),
                    });
                }
                let carrier = &operands[0];
                let Value::RecordValue(fields) = carrier else {
                    return Err(EvalError::BadTransformOperands {
                        reason: "FieldProject carrier must be a RecordValue",
                    });
                };
                let Some(field) = fields.iter().find(|f| &f.label == field_label) else {
                    return Err(EvalError::BadTransformOperands {
                        reason: "FieldProject label not present on RecordValue carrier",
                    });
                };
                Ok(field.value.clone())
            }
            TransformTarget::Callable(callee_decl) => {
                let callee_decl = *callee_decl;
                let connective = &dag.declaration(callee_decl).connective;
                if let TypeConnective::Arrow { body, .. } = connective {
                    let ArrowBody::UserDefined(bind_id) = body else {
                        return Err(EvalError::UnsupportedTransformTarget {
                            kind: "Callable (non-UserDefined body)",
                        });
                    };
                    let bind_node_id = bind_id.node_id();
                    let Behavior::Bind(bind) = dag.node(bind_node_id) else {
                        return Err(EvalError::MissingNode { node: bind_node_id });
                    };
                    let bind = bind.clone();
                    if operands.len() != bind.params.len() {
                        return Err(EvalError::TransformArityMismatch {
                            expected: bind.params.len(),
                            got: operands.len(),
                        });
                    }
                    let bindings: Vec<(PortId, Value)> =
                        bind.params.iter().copied().zip(operands).collect();
                    state.push_frame(EvalFrame::empty());
                    let body_result =
                        eval_callable_body_in_pushed_frame(dag, &bind, bindings, state, strategy);
                    let pop_result = state.pop_frame();
                    return match (body_result, pop_result) {
                        (Ok(value), Ok(_)) => Ok(value),
                        (Err(err), _) => Err(err),
                        (Ok(_), Err(frame_err)) => Err(EvalError::from(frame_err)),
                    };
                }

                // E6-G0d: non-Arrow `Callable` targets lowered as record/variant constructors.
                let variant_tag_template = match &dag.declaration(callee_decl).connective {
                    TypeConnective::Instantiation { template, .. } => *template,
                    _ => callee_decl,
                };
                if crate::lower::declaration_is_disj_variant_arm(dag, variant_tag_template) {
                    let Some(fields) =
                        crate::lower::eval_constructor_variant_payload_fields(dag, callee_decl)
                    else {
                        return Err(EvalError::BadTransformOperands {
                            reason: "variant constructor payload fields could not be resolved",
                        });
                    };
                    if operands.len() != fields.len() {
                        return Err(EvalError::TransformArityMismatch {
                            expected: fields.len(),
                            got: operands.len(),
                        });
                    }
                    // Match `infer::resolve_payload_binding_type`: a single Conj field labeled
                    // `_0` uses `PayloadBindingResolution::Direct` — the match arm's payload port
                    // is typed as the inner `T`, not a record shape. Runtime `VariantValue.payload`
                    // must carry `T` directly so `eval_branch` binding agrees with inference.
                    let payload: Value = if fields.len() == 1 && fields[0].0 == "_0" {
                        operands
                            .into_iter()
                            .next()
                            .expect("length checked against fields.len()")
                    } else {
                        Value::RecordValue(
                            fields
                                .into_iter()
                                .zip(operands)
                                .map(|((label, _), value)| NamedField { label, value })
                                .collect(),
                        )
                    };
                    return Ok(Value::VariantValue {
                        tag: variant_tag_template,
                        payload: Box::new(payload),
                    });
                }

                if let Some(labels) =
                    crate::lower::constructor_record_field_labels(dag, callee_decl)
                {
                    if operands.len() != labels.len() {
                        return Err(EvalError::TransformArityMismatch {
                            expected: labels.len(),
                            got: operands.len(),
                        });
                    }
                    let record_fields: Vec<NamedField> = labels
                        .into_iter()
                        .zip(operands)
                        .map(|(label, value)| NamedField { label, value })
                        .collect();
                    return Ok(Value::RecordValue(record_fields));
                }

                Err(EvalError::BadTransformOperands {
                    reason: "Callable target declaration is not an Arrow type",
                })
            }
        }
    }

    fn eval_callable_body_in_pushed_frame(
        dag: &Dag,
        bind: &crate::dag::BindNode,
        bindings: Vec<(PortId, Value)>,
        state: &mut EvalStateStack<Value>,
        strategy: &EvalStrategy,
    ) -> Result<Value, EvalError> {
        for (param, value) in bindings {
            state.bind_top(param, value)?;
        }
        eval_port(dag, bind.value, state, strategy)
    }

    fn expect_int_literal(value: &Value) -> Result<i64, EvalError> {
        match value {
            Value::LiteralValue(LiteralBits::Int(n)) => Ok(*n),
            _ => Err(EvalError::BadTransformOperands {
                reason: "expected Int literal",
            }),
        }
    }

    /// Checked `Int` ring ops only (`Add` / `Sub` / `Mul`). `Div` is rejected
    /// at the [`eval_transform_node`] call site so this helper never collapses
    /// division through the same `i64` carrier as ring success values.
    fn apply_arithmetic_int_ring_only(op: ArithmeticOp, a: i64, b: i64) -> Result<i64, EvalError> {
        const OVERFLOW: EvalError = EvalError::BadTransformOperands {
            reason: "integer overflow",
        };
        match op {
            ArithmeticOp::Add => a.checked_add(b).ok_or(OVERFLOW),
            ArithmeticOp::Sub => a.checked_sub(b).ok_or(OVERFLOW),
            ArithmeticOp::Mul => a.checked_mul(b).ok_or(OVERFLOW),
            ArithmeticOp::Div => Err(EvalError::UnsupportedTransformTarget {
                kind: "ArithmeticDiv",
            }),
        }
    }

    pub fn evaluate_body(
        dag: &Dag,
        entry: NodeId,
        state: &mut EvalStateStack<Value>,
        strategy: EvalStrategy,
    ) -> Result<Value, EvalError> {
        eval_node(dag, entry, state, &strategy)
    }

    /// **Dissolution receipt: TERMINAL.** The three variants are distinct
    /// fail-closed outcomes for the E2 frame boundary.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum EvalFrameError {
        EmptyStateStack,
        DuplicateBinding { port: PortId },
        UnboundPort { port: PortId },
    }

    /// One evaluator binding scope.
    ///
    /// `HashMap<PortId, V>` is the Rust realization of the substrate
    /// `Map<PortId, Value>` finite partial-function discipline. Do not replace
    /// this with a duplicate-admitting `List<EvalBinding>` shape.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct EvalFrame<V> {
        bindings: HashMap<PortId, V>,
    }

    impl<V> EvalFrame<V> {
        pub fn empty() -> Self {
            Self {
                bindings: HashMap::new(),
            }
        }

        pub fn from_bindings(
            bindings: impl IntoIterator<Item = (PortId, V)>,
        ) -> Result<Self, EvalFrameError> {
            let mut frame = Self::empty();
            for (port, value) in bindings {
                frame.bind(port, value)?;
            }
            Ok(frame)
        }

        pub fn bind(&mut self, port: PortId, value: V) -> Result<(), EvalFrameError> {
            if self.bindings.contains_key(&port) {
                return Err(EvalFrameError::DuplicateBinding { port });
            }
            self.bindings.insert(port, value);
            Ok(())
        }

        pub fn lookup_local(&self, port: PortId) -> Option<&V> {
            self.bindings.get(&port)
        }
    }

    /// Evaluator frame stack. The final element is the innermost / top frame.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct EvalStateStack<V> {
        frames: Vec<EvalFrame<V>>,
    }

    impl<V> EvalStateStack<V> {
        pub fn with_root_frame(frame: EvalFrame<V>) -> Self {
            Self {
                frames: vec![frame],
            }
        }

        pub fn from_outer_to_inner(frames: Vec<EvalFrame<V>>) -> Self {
            Self { frames }
        }

        pub fn push_frame(&mut self, frame: EvalFrame<V>) {
            self.frames.push(frame);
        }

        pub fn pop_frame(&mut self) -> Result<EvalFrame<V>, EvalFrameError> {
            self.frames.pop().ok_or(EvalFrameError::EmptyStateStack)
        }

        pub fn lookup(&self, port: PortId) -> Result<&V, EvalFrameError> {
            self.frames
                .iter()
                .rev()
                .find_map(|frame| frame.lookup_local(port))
                .ok_or(EvalFrameError::UnboundPort { port })
        }

        pub fn bind_top(&mut self, port: PortId, value: V) -> Result<(), EvalFrameError> {
            self.frames
                .last_mut()
                .ok_or(EvalFrameError::EmptyStateStack)?
                .bind(port, value)
        }

        pub fn frames_outer_to_inner(&self) -> &[EvalFrame<V>] {
            &self.frames
        }
    }

    #[cfg(test)]
    mod tests {
        use std::collections::HashMap;

        use super::NamedField;
        use super::{
            descent_proof_path_key, eval_loop_with_descent_execution_proof, eval_node, eval_port,
            eval_value, evaluate_body, DescentExecutionProof, DescentResidual, EvalError,
            EvalFrame, EvalFrameError, EvalStateStack, EvalStrategy, InputEvaluationOrder,
            NonStrictEvidence, StrictEvidence, Value,
        };
        use crate::compile_to_dag;
        use crate::dag::{
            ArithmeticOp, ArrowBody, Behavior, BranchPattern, Cluster, ComparisonOp, Dag,
            DeclarationId, IntraClusterCall, LiteralBits, LogicalOp, LoopBound, MemberDescent,
            NodeId, NonEmptyList, NonSingletonList, OperatorKind, Path, PayloadBinding, PortId,
            TransformTarget, TypeConnective,
        };
        use crate::diagnostics::SourceSpan;

        fn span() -> SourceSpan {
            SourceSpan::new("evaluator_frame.unit", 0, 1)
        }

        fn ports(count: usize) -> Vec<PortId> {
            let mut dag = Dag::new();
            (0..count)
                .map(|i| dag.push_value(LiteralBits::Int(i as i64), span()))
                .collect()
        }

        fn node_for_port(dag: &Dag, port: PortId) -> NodeId {
            dag.resolve_producer_opt(&port).expect("producer").id()
        }

        fn descent_loop_fixture(body_value: i64) -> (Dag, crate::dag::LoopNode) {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(0), span());
            let body_output = dag.push_value(LiteralBits::Int(body_value), span());
            let body = node_for_port(&dag, body_output);
            let bind = dag.push_bind("descent_member", init, vec![source, init], span());
            let param0 = dag.param_of(bind, 0).expect("param 0");
            let param1 = dag.param_of(bind, 1).expect("param 1");
            let transform_output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Sub)),
                vec![source, init],
                span(),
            );
            let transform = dag
                .as_transform_ref(node_for_port(&dag, transform_output))
                .expect("transform ref");
            let cluster = dag.push_cluster(Cluster {
                members: NonSingletonList::from_vec(vec![
                    MemberDescent { param: param0 },
                    MemberDescent { param: param1 },
                ])
                .expect("non-singleton"),
                intra_cluster_calls: NonEmptyList::from_vec(vec![IntraClusterCall { transform }])
                    .expect("non-empty"),
            });
            let output = dag.push_loop(
                source,
                init,
                body,
                LoopBound::Descent {
                    cluster,
                    measure: source,
                },
                span(),
            );
            let entry = node_for_port(&dag, output);
            let loop_node = dag.node(entry).as_loop().expect("loop node").clone();
            (dag, loop_node)
        }

        fn empty_state() -> EvalStateStack<Value> {
            EvalStateStack::with_root_frame(EvalFrame::empty())
        }

        fn eager_strategy() -> EvalStrategy {
            EvalStrategy::ApplicativeOrder {
                input_order: InputEvaluationOrder::LeftFirst,
            }
        }

        fn bind_node_id_for_fn(dag: &Dag, fn_name: &str) -> NodeId {
            let decl = dag
                .declaration_by_name(fn_name)
                .unwrap_or_else(|| panic!("missing fn `{fn_name}`"));
            let TypeConnective::Arrow { body, .. } = &decl.connective else {
                panic!("`{fn_name}` must be Arrow");
            };
            let ArrowBody::UserDefined(bind_id) = body else {
                panic!("`{fn_name}` must be UserDefined");
            };
            bind_id.node_id()
        }

        fn first_resolved_variant_tag_in_bind_body(dag: &Dag, fn_name: &str) -> DeclarationId {
            let entry = bind_node_id_for_fn(dag, fn_name);
            let Behavior::Bind(bind) = dag.node(entry) else {
                panic!("expected Bind");
            };
            let mut stack = vec![bind.value];
            while let Some(port) = stack.pop() {
                let Some(producer) = dag.port(port).produced_by else {
                    continue;
                };
                match dag.node(producer) {
                    Behavior::Branch(branch) => {
                        for path in &branch.paths {
                            if let BranchPattern::ResolvedVariant(decl) = &path.pattern {
                                return *decl;
                            }
                        }
                    }
                    Behavior::Bind(inner) => stack.push(inner.value),
                    _ => {}
                }
            }
            panic!("no ResolvedVariant pattern in `{fn_name}` body");
        }

        fn template_decl_id_for_callable_constructor_in_fn(
            dag: &Dag,
            fn_name: &str,
        ) -> DeclarationId {
            let entry = bind_node_id_for_fn(dag, fn_name);
            let Behavior::Bind(bind) = dag.node(entry) else {
                panic!("expected Bind");
            };
            let mut stack = vec![bind.value];
            while let Some(port) = stack.pop() {
                let Some(producer) = dag.port(port).produced_by else {
                    continue;
                };
                match dag.node(producer) {
                    Behavior::Transform(t) => {
                        if let TransformTarget::Callable(callee) = t.target {
                            return match &dag.declaration(callee).connective {
                                TypeConnective::Instantiation { template, .. } => *template,
                                _ => callee,
                            };
                        }
                    }
                    Behavior::Bind(inner) => stack.push(inner.value),
                    _ => {}
                }
            }
            panic!("no TransformTarget::Callable in `{fn_name}` body");
        }

        #[test]
        fn e6_g0d_variant_constructor_executes_to_variant_value() {
            let src = "type MaybeInt = Some { value: Int } | None\n\
                        fn pack(x: Int) -> MaybeInt = Some { value: x }\n";
            let dag = compile_to_dag(src, "e6_g0d_variant.v3").expect("compile");
            let expected_tag = template_decl_id_for_callable_constructor_in_fn(&dag, "pack");
            let entry = bind_node_id_for_fn(&dag, "pack");
            let Behavior::Bind(bind) = dag.node(entry) else {
                panic!("bind");
            };
            let x_port = bind.params[0];
            let frame =
                EvalFrame::from_bindings([(x_port, Value::LiteralValue(LiteralBits::Int(42)))])
                    .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let value = evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("eval");
            let Value::VariantValue { tag, payload } = value else {
                panic!("expected VariantValue, got {value:?}");
            };
            assert_eq!(tag, expected_tag);
            let Value::RecordValue(fields) = *payload else {
                panic!("expected record payload");
            };
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].label, "value");
            assert_eq!(fields[0].value, Value::LiteralValue(LiteralBits::Int(42)));
        }

        #[test]
        fn e6_g0d_record_constructor_preserves_declaration_field_order() {
            let src = "type R { a: Int, b: Int }\n\
                        fn mk() -> R = { b: 7, a: 5 }\n";
            let dag = compile_to_dag(src, "e6_g0d_record.v3").expect("compile");
            let entry = bind_node_id_for_fn(&dag, "mk");
            let mut state = empty_state();
            let value = evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("eval");
            let Value::RecordValue(fields) = value else {
                panic!("expected RecordValue, got {value:?}");
            };
            assert_eq!(
                fields,
                vec![
                    NamedField {
                        label: "a".to_string(),
                        value: Value::LiteralValue(LiteralBits::Int(5)),
                    },
                    NamedField {
                        label: "b".to_string(),
                        value: Value::LiteralValue(LiteralBits::Int(7)),
                    },
                ]
            );
        }

        #[test]
        fn e6_g0d_generic_variant_constructor_tag_matches_resolved_variant_pattern() {
            let src = "type Maybe<T> = Some(T) | None\n\
                        fn pack_int(x: Int) -> Maybe<Int> = Some(x)\n\
                        fn probe(x: Int) -> Int = match pack_int(x) { Some(p) => p, None => 0 }\n";
            let dag = compile_to_dag(src, "e6_g0d_gen_variant.v3").expect("compile");
            let expected_tag = first_resolved_variant_tag_in_bind_body(&dag, "probe");
            let entry = bind_node_id_for_fn(&dag, "pack_int");
            let Behavior::Bind(bind) = dag.node(entry) else {
                panic!("bind");
            };
            let x_port = bind.params[0];
            let frame =
                EvalFrame::from_bindings([(x_port, Value::LiteralValue(LiteralBits::Int(99)))])
                    .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let value = evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("eval");
            let Value::VariantValue { tag, payload } = value else {
                panic!("expected VariantValue");
            };
            assert_eq!(
                tag, expected_tag,
                "tag must be template variant id (matches BranchPattern::ResolvedVariant), not an anonymous Instantiation id"
            );
            assert_eq!(
                *payload,
                Value::LiteralValue(LiteralBits::Int(99)),
                "positional `Some(T)` payload must be Direct-shaped (infer `PayloadBindingResolution::Direct`), not RecordValue(_0)"
            );
        }

        #[test]
        fn e6_g0d_positional_variant_payload_round_trips_through_direct_binding_arm() {
            let src = "type Boxed<T> = Box(T) | Empty\n\
                        fn mk(x: Int) -> Boxed<Int> = Box(x)\n\
                        fn bump(x: Int) -> Int = match mk(x) { Box(p) => p + 1, Empty => 0 }\n";
            let dag = compile_to_dag(src, "e6_g0d_positional_direct.v3").expect("compile");
            let entry = bind_node_id_for_fn(&dag, "bump");
            let Behavior::Bind(bind) = dag.node(entry) else {
                panic!("bind");
            };
            let x_port = bind.params[0];
            let frame =
                EvalFrame::from_bindings([(x_port, Value::LiteralValue(LiteralBits::Int(5)))])
                    .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let value = evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("eval");
            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(6)));
        }

        #[test]
        fn e6_g0d_generic_record_constructor_preserves_template_field_order() {
            let src = "type PairAB<A, B> { first: A, second: B }\n\
                        fn mk() -> PairAB<Int, Bool> = { second: true, first: 3 }\n";
            let dag = compile_to_dag(src, "e6_g0d_gen_record.v3").expect("compile");
            let entry = bind_node_id_for_fn(&dag, "mk");
            let mut state = empty_state();
            let value = evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("eval");
            let Value::RecordValue(fields) = value else {
                panic!("expected RecordValue");
            };
            assert_eq!(
                fields,
                vec![
                    NamedField {
                        label: "first".to_string(),
                        value: Value::LiteralValue(LiteralBits::Int(3)),
                    },
                    NamedField {
                        label: "second".to_string(),
                        value: Value::LiteralValue(LiteralBits::Bool(true)),
                    },
                ]
            );
        }

        #[test]
        fn e6_g0d_nullary_variant_constructor_empty_record_payload() {
            let src = "type U = Z | S { n: Int }\n\
                        fn zed() -> U = Z\n";
            let dag = compile_to_dag(src, "e6_g0d_nullary.v3").expect("compile");
            let expected_tag = template_decl_id_for_callable_constructor_in_fn(&dag, "zed");
            let entry = bind_node_id_for_fn(&dag, "zed");
            let mut state = empty_state();
            let value = evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("eval");
            let Value::VariantValue { tag, payload } = value else {
                panic!("expected VariantValue");
            };
            assert_eq!(tag, expected_tag);
            let Value::RecordValue(fields) = *payload else {
                panic!("expected record payload");
            };
            assert!(fields.is_empty());
        }

        #[test]
        fn e6_g0d_variant_constructor_round_trips_through_branch_match() {
            let src = "type MaybeInt = Some { value: Int } | None\n\
                        fn pack(x: Int) -> MaybeInt = Some { value: x }\n\
                        fn unpack(x: Int) -> Int = match pack(x) { Some { value: v } => v, None => 0 }\n";
            let dag = compile_to_dag(src, "e6_g0d_roundtrip.v3").expect("compile");
            let entry = bind_node_id_for_fn(&dag, "unpack");
            let Behavior::Bind(bind) = dag.node(entry) else {
                panic!("bind");
            };
            let x_port = bind.params[0];
            let frame =
                EvalFrame::from_bindings([(x_port, Value::LiteralValue(LiteralBits::Int(11)))])
                    .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let value = evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("eval");
            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(11)));
        }

        #[test]
        fn eval_value_constructs_literal_runtime_value() {
            let mut dag = Dag::new();
            let output = dag.push_value(LiteralBits::String("ready".to_string()), span());
            let value_node = dag
                .resolve_producer_opt(&output)
                .and_then(Behavior::as_value)
                .expect("value producer");

            let value = eval_value(value_node);

            assert_eq!(
                value,
                Value::LiteralValue(LiteralBits::String("ready".to_string()))
            );
        }

        #[test]
        fn eval_node_dispatches_value_behavior_to_literal_runtime_value() {
            let mut dag = Dag::new();
            let output = dag.push_value(LiteralBits::String("ready".to_string()), span());
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("value evaluates");

            assert_eq!(
                value,
                Value::LiteralValue(LiteralBits::String("ready".to_string()))
            );
        }

        #[test]
        fn eval_port_prefers_dag_producer_over_frame_binding() {
            let mut dag = Dag::new();
            let port = dag.push_value(LiteralBits::Int(1), span());
            let frame = EvalFrame::from_bindings([(
                port,
                Value::LiteralValue(LiteralBits::String("shadow".to_string())),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = eval_port(&dag, port, &mut state, &strategy).expect("producer wins");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(1)));
        }

        #[test]
        fn eval_port_uses_innermost_frame_binding_for_producerless_port() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let port = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let outer = EvalFrame::from_bindings([(
                port,
                Value::LiteralValue(LiteralBits::String("outer".to_string())),
            )])
            .expect("outer frame");
            let inner = EvalFrame::from_bindings([(
                port,
                Value::LiteralValue(LiteralBits::String("inner".to_string())),
            )])
            .expect("inner frame");
            let mut state = EvalStateStack::from_outer_to_inner(vec![outer, inner]);
            let strategy = eager_strategy();

            let value = eval_port(&dag, port, &mut state, &strategy).expect("bound port");

            assert_eq!(
                value,
                Value::LiteralValue(LiteralBits::String("inner".to_string()))
            );
        }

        #[test]
        fn eval_port_falls_back_to_producer_value() {
            let mut dag = Dag::new();
            let port = dag.push_value(LiteralBits::Bool(true), span());
            let mut state = empty_state();
            let strategy = eager_strategy();

            let value = eval_port(&dag, port, &mut state, &strategy).expect("producer value");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Bool(true)));
        }

        #[test]
        fn eval_port_reports_unbound_port_when_no_frame_or_producer_resolves() {
            let mut source = Dag::new();
            let stale_port = source.push_value(LiteralBits::Int(1), span());
            let empty = Dag::new();
            let mut state = empty_state();
            let strategy = eager_strategy();

            let err =
                eval_port(&empty, stale_port, &mut state, &strategy).expect_err("unbound port");

            assert_eq!(err, EvalError::UnboundPort { port: stale_port });
        }

        #[test]
        fn eval_port_rejects_frame_binding_for_port_absent_from_dag() {
            let mut source = Dag::new();
            let stale_port = source.push_value(LiteralBits::Int(1), span());
            let empty = Dag::new();
            let frame = EvalFrame::from_bindings([(
                stale_port,
                Value::LiteralValue(LiteralBits::String("stale".to_string())),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let err =
                eval_port(&empty, stale_port, &mut state, &strategy).expect_err("unbound port");

            assert_eq!(err, EvalError::UnboundPort { port: stale_port });
        }

        #[test]
        fn evaluate_body_delegates_to_eval_node_shell() {
            let mut dag = Dag::new();
            let output = dag.push_value(LiteralBits::Int(8), span());
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let value =
                evaluate_body(&dag, entry, &mut state, eager_strategy()).expect("value evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(8)));
        }

        #[test]
        fn missing_entry_node_fails_closed() {
            let mut source = Dag::new();
            let output = source.push_value(LiteralBits::Int(1), span());
            let stale_entry = node_for_port(&source, output);
            let empty = Dag::new();
            let mut state = empty_state();

            let err = evaluate_body(&empty, stale_entry, &mut state, eager_strategy())
                .expect_err("missing node");

            assert_eq!(err, EvalError::MissingNode { node: stale_entry });
        }

        #[test]
        fn transform_arithmetic_add_evaluates() {
            let mut dag = Dag::new();
            let lhs = dag.push_value(LiteralBits::Int(1), span());
            let rhs = dag.push_value(LiteralBits::Int(2), span());
            let output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![lhs, rhs],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("transform evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(3)));
        }

        #[test]
        fn transform_arithmetic_sub_evaluates() {
            let mut dag = Dag::new();
            let lhs = dag.push_value(LiteralBits::Int(10), span());
            let rhs = dag.push_value(LiteralBits::Int(3), span());
            let output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Sub)),
                vec![lhs, rhs],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let value = eval_node(&dag, entry, &mut state, &eager_strategy()).expect("sub");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(7)));
        }

        #[test]
        fn transform_arithmetic_checked_overflow_fails_closed() {
            let mut dag = Dag::new();
            let lhs = dag.push_value(LiteralBits::Int(i64::MAX), span());
            let rhs = dag.push_value(LiteralBits::Int(1), span());
            let output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![lhs, rhs],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err = eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("overflow");

            assert_eq!(
                err,
                EvalError::BadTransformOperands {
                    reason: "integer overflow",
                }
            );
        }

        // PR-E E4: branch helpers — bridge a tag declaration name into the
        // bootstrap fixture so `Value::VariantValue { tag, .. }` carries an
        // honest `DeclarationId`, not a hand-rolled stub. The branch
        // evaluator only does `decl == tag` equality, so any two distinct
        // real declarations (`Bool` / `Int`) are sufficient as tag stand-ins
        // without introducing variant-discovery infrastructure that doesn't
        // belong in this slice.
        fn declaration_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
            dag.declaration_by_name(name)
                .unwrap_or_else(|| panic!("declaration `{name}` missing from fixture"))
                .id
        }

        fn variant(tag: DeclarationId, payload: Value) -> Value {
            Value::VariantValue {
                tag,
                payload: Box::new(payload),
            }
        }

        // E4 §B.1.3: caller-supplied `VariantValue` scrutinee at the
        // branch input port matches the path whose `ResolvedVariant.tag`
        // equals the runtime tag; the body's value is returned.
        #[test]
        fn eval_branch_selects_resolved_variant_by_tag() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let some_tag = declaration_id_by_name(&dag, "Bool");
            let other_tag = declaration_id_by_name(&dag, "Int");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            // Bodies are `Behavior::Value` nodes (E1-supported); Bind /
            // Transform bodies wait on E3 / E6.
            let some_arm_output = dag.push_value(LiteralBits::Int(7), span());
            let some_arm_body = node_for_port(&dag, some_arm_output);
            let other_arm_output = dag.push_value(LiteralBits::Int(13), span());
            let other_arm_body = node_for_port(&dag, other_arm_output);
            let output = dag.push_branch(
                scrutinee,
                vec![
                    Path {
                        body: some_arm_body,
                        output: some_arm_output,
                        pattern: BranchPattern::ResolvedVariant(some_tag),
                        binding: None,
                    },
                    Path {
                        body: other_arm_body,
                        output: other_arm_output,
                        pattern: BranchPattern::ResolvedVariant(other_tag),
                        binding: None,
                    },
                ],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                variant(some_tag, Value::LiteralValue(LiteralBits::Bool(true))),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("branch evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(7)));
        }

        // E4 Bool alignment: Bool literal scrutinees are reified through
        // `Dag::bool_runtime_variant_id`, so branch dispatch compares the same
        // `True` / `False` declaration ids that inference resolved on patterns.
        #[test]
        fn eval_branch_reifies_bool_literal_scrutinee_to_disj_variant_id() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let true_tag = dag
                .bool_runtime_variant_id(true)
                .expect("Bool.True variant id");
            let false_tag = dag
                .bool_runtime_variant_id(false)
                .expect("Bool.False variant id");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let true_output = dag.push_value(LiteralBits::Int(1), span());
            let true_body = node_for_port(&dag, true_output);
            let false_output = dag.push_value(LiteralBits::Int(0), span());
            let false_body = node_for_port(&dag, false_output);
            let output = dag.push_branch(
                scrutinee,
                vec![
                    Path {
                        body: true_body,
                        output: true_output,
                        pattern: BranchPattern::ResolvedVariant(true_tag),
                        binding: None,
                    },
                    Path {
                        body: false_body,
                        output: false_output,
                        pattern: BranchPattern::ResolvedVariant(false_tag),
                        binding: None,
                    },
                ],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                Value::LiteralValue(LiteralBits::Bool(true)),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("branch evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(1)));
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_branch_reifies_true_bool_literal_scrutinee_to_true_arm() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let true_tag = dag
                .bool_runtime_variant_id(true)
                .expect("Bool.True variant id");
            let false_tag = dag
                .bool_runtime_variant_id(false)
                .expect("Bool.False variant id");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let true_output = dag.push_value(LiteralBits::Int(11), span());
            let true_body = node_for_port(&dag, true_output);
            let false_output = dag.push_value(LiteralBits::Int(22), span());
            let false_body = node_for_port(&dag, false_output);
            let output = dag.push_branch(
                scrutinee,
                vec![
                    Path {
                        body: true_body,
                        output: true_output,
                        pattern: BranchPattern::ResolvedVariant(true_tag),
                        binding: None,
                    },
                    Path {
                        body: false_body,
                        output: false_output,
                        pattern: BranchPattern::ResolvedVariant(false_tag),
                        binding: None,
                    },
                ],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                Value::LiteralValue(LiteralBits::Bool(true)),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("branch evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(11)));
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_branch_reifies_false_bool_literal_scrutinee_to_false_arm() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let true_tag = dag
                .bool_runtime_variant_id(true)
                .expect("Bool.True variant id");
            let false_tag = dag
                .bool_runtime_variant_id(false)
                .expect("Bool.False variant id");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let true_output = dag.push_value(LiteralBits::Int(1), span());
            let true_body = node_for_port(&dag, true_output);
            let false_output = dag.push_value(LiteralBits::Int(0), span());
            let false_body = node_for_port(&dag, false_output);
            let output = dag.push_branch(
                scrutinee,
                vec![
                    Path {
                        body: true_body,
                        output: true_output,
                        pattern: BranchPattern::ResolvedVariant(true_tag),
                        binding: None,
                    },
                    Path {
                        body: false_body,
                        output: false_output,
                        pattern: BranchPattern::ResolvedVariant(false_tag),
                        binding: None,
                    },
                ],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                Value::LiteralValue(LiteralBits::Bool(false)),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("branch evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(0)));
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_branch_fails_closed_when_bool_reification_authority_is_missing() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let true_tag = dag
                .bool_runtime_variant_id(true)
                .expect("Bool.True variant id");
            let bool_decl = dag
                .declaration_by_name("Bool")
                .expect("Bool declaration")
                .id;
            dag.declaration_mut(bool_decl).name = Some("TruthValue".to_string());

            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let arm_output = dag.push_value(LiteralBits::Int(1), span());
            let arm_body = node_for_port(&dag, arm_output);
            let output = dag.push_branch(
                scrutinee,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::ResolvedVariant(true_tag),
                    binding: None,
                }],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                Value::LiteralValue(LiteralBits::Bool(true)),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let err = eval_node(&dag, entry, &mut state, &strategy)
                .expect_err("missing Bool reification authority rejected");

            assert_eq!(err, EvalError::BranchScrutineeShape { node: entry });
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        // E4 §B.1.3: payload binding registers on the freshly-pushed
        // frame and does not leak past `pop_frame`. Bodies that *read*
        // the bound payload (Bind / Transform forms) wait on E3 / E6;
        // this test verifies the frame-discipline scaffolding the body
        // would observe by checking pre/post stack state through a
        // shadow-port outer binding that the inner frame's lookup chain
        // walks past.
        #[test]
        fn eval_branch_binds_payload_in_fresh_frame_for_body() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let some_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let payload_port = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            // Body returns a literal — verifies the path body is
            // dispatched through `eval_node` after the frame push and
            // payload bind. Bodies that consume the payload via
            // `eval_port` wait on E3 / E6 supporting Bind / Transform.
            let arm_output = dag.push_value(LiteralBits::Int(0), span());
            let arm_body = node_for_port(&dag, arm_output);
            let output = dag.push_branch(
                scrutinee,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::ResolvedVariant(some_tag),
                    binding: Some(PayloadBinding {
                        binding_name: "p".to_string(),
                        payload_port,
                    }),
                }],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                variant(
                    some_tag,
                    Value::LiteralValue(LiteralBits::String("hello".to_string())),
                ),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("branch evaluates");

            // Body is a literal; the test's primary assertion is the
            // post-evaluation frame discipline below. (Body return value
            // is the literal, confirming `eval_node` dispatched into the
            // path body.)
            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(0)));
            // After branch evaluation, the pushed frame is popped; only
            // the original root frame remains, which never bound the
            // payload port. Per Facts-Flow-Forward, the payload binding
            // was scoped to the body frame and does not leak into the
            // caller's stack.
            assert_eq!(state.frames_outer_to_inner().len(), 1);
            assert!(state.frames_outer_to_inner()[0]
                .lookup_local(payload_port)
                .is_none());
        }

        // E4 §B.1.3 fail-closed: `UnresolvedVariant` reaching evaluation
        // is a resolution gap, not a runtime case — diagnose the path's
        // declared name so a downstream consumer can repoint.
        #[test]
        fn eval_branch_fails_closed_on_unresolved_variant() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let some_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let arm_output = dag.push_value(LiteralBits::Int(1), span());
            let arm_body = node_for_port(&dag, arm_output);
            let output = dag.push_branch(
                scrutinee,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Drift".to_string(),
                        span: span(),
                    },
                    binding: None,
                }],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                variant(some_tag, Value::LiteralValue(LiteralBits::Bool(true))),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let err = eval_node(&dag, entry, &mut state, &strategy)
                .expect_err("unresolved variant rejected");

            assert_eq!(
                err,
                EvalError::BranchUnresolvedVariant {
                    node: entry,
                    name: "Drift".to_string(),
                }
            );
            // Stack invariant: balanced even on the diagnostic path.
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        // E4 §B.1.3 fail-closed: scrutinee must be `VariantValue` for
        // `ResolvedVariant` matching to be well-defined.
        #[test]
        fn eval_branch_fails_closed_on_non_variant_scrutinee() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let some_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let arm_output = dag.push_value(LiteralBits::Int(1), span());
            let arm_body = node_for_port(&dag, arm_output);
            let output = dag.push_branch(
                scrutinee,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::ResolvedVariant(some_tag),
                    binding: None,
                }],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame =
                EvalFrame::from_bindings([(scrutinee, Value::LiteralValue(LiteralBits::Int(42)))])
                    .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let err = eval_node(&dag, entry, &mut state, &strategy)
                .expect_err("non-variant scrutinee rejected");

            assert_eq!(err, EvalError::BranchScrutineeShape { node: entry });
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        // E4 §B.1.3 fail-closed: scrutinee tag with no matching path is
        // an inference / substrate invariant violation at runtime.
        #[test]
        fn eval_branch_fails_closed_on_no_matching_arm() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let path_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee_tag = declaration_id_by_name(&dag, "Int");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let arm_output = dag.push_value(LiteralBits::Int(1), span());
            let arm_body = node_for_port(&dag, arm_output);
            let output = dag.push_branch(
                scrutinee,
                // Only the `path_tag` (`Bool`) arm; scrutinee carries
                // `scrutinee_tag` (`Int`) so no path matches and the
                // evaluator must fail closed on no-matching-arm.
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::ResolvedVariant(path_tag),
                    binding: None,
                }],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                variant(scrutinee_tag, Value::LiteralValue(LiteralBits::Bool(false))),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let err = eval_node(&dag, entry, &mut state, &strategy)
                .expect_err("no matching arm rejected");

            assert_eq!(
                err,
                EvalError::BranchNoMatchingArm {
                    node: entry,
                    tag: scrutinee_tag,
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        // E4 sanity: reachable through the public `evaluate_body` entry.
        #[test]
        fn evaluate_body_dispatches_branch_through_eval_node() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let some_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let arm_output = dag.push_value(LiteralBits::Int(99), span());
            let arm_body = node_for_port(&dag, arm_output);
            let output = dag.push_branch(
                scrutinee,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::ResolvedVariant(some_tag),
                    binding: None,
                }],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                variant(some_tag, Value::LiteralValue(LiteralBits::Bool(true))),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = evaluate_body(&dag, entry, &mut state, strategy).expect("branch evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(99)));
        }

        // E4 / Facts-Flow-Forward: the arm value is the value at
        // `BranchPath.output`, not the body's local return. When
        // `path.output` is a payload-bound port (the arm "returns its
        // payload"), the evaluator must read that port via `eval_port`
        // after running the body for its side effects, not return the
        // body node's own value. Constructed so the body is a Value
        // node returning a *different* literal than the payload — if
        // the evaluator returned the body's value the test would see
        // the wrong literal.
        #[test]
        fn eval_branch_returns_value_at_path_output_not_body_value() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let some_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let payload_port = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            // Body returns a sentinel literal; if the evaluator wrongly
            // returned this directly, the test would fail.
            let body_output = dag.push_value(LiteralBits::Int(0xdead), span());
            let arm_body = node_for_port(&dag, body_output);
            let payload_value = Value::LiteralValue(LiteralBits::Int(7));
            let output = dag.push_branch(
                scrutinee,
                vec![Path {
                    body: arm_body,
                    // Authoritative arm value comes from `payload_port`,
                    // not from `body_output`.
                    output: payload_port,
                    pattern: BranchPattern::ResolvedVariant(some_tag),
                    binding: Some(PayloadBinding {
                        binding_name: "p".to_string(),
                        payload_port,
                    }),
                }],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame =
                EvalFrame::from_bindings([(scrutinee, variant(some_tag, payload_value.clone()))])
                    .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("branch evaluates");

            assert_eq!(value, payload_value);
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        // E4 / Facts-Flow-Forward: producerless-arm sentinel.
        // `lower.rs:6133-6134, 6265` lowers arms whose result port has
        // no producer node by setting `path.body == branch.id` —
        // there's no body to execute; the value is at `path.output`
        // already (e.g. payload-bound or an existing port from outside
        // the branch). `eval_branch` must detect this sentinel and
        // skip body evaluation, otherwise it would re-enter
        // `eval_branch` on the same node and either loop or drop the
        // authoritative `path.output` fact. Constructed via
        // crate-private `alloc_node_id` / `push_node` so `path.body`
        // can equal the branch's own id (the public `push_branch`
        // builder asserts body existence before allocating its id, so
        // it cannot construct this sentinel directly).
        #[test]
        fn eval_branch_handles_producerless_arm_sentinel() {
            use crate::dag::BranchNode;
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let some_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let payload_port = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            // Manual branch construction so `path.body == branch_id`,
            // mirroring the `producer_of(...).unwrap_or(branch_id)`
            // sentinel from `lower.rs`.
            let branch_id = dag.alloc_node_id();
            let branch_output = dag.alloc_port(Some(branch_id));
            dag.push_node(Behavior::Branch(BranchNode {
                id: branch_id,
                input: scrutinee,
                paths: vec![Path {
                    body: branch_id, // sentinel: no producer for the arm result
                    output: payload_port,
                    pattern: BranchPattern::ResolvedVariant(some_tag),
                    binding: Some(PayloadBinding {
                        binding_name: "p".to_string(),
                        payload_port,
                    }),
                }],
                output: branch_output,
                span: span(),
                emit_participation: None,
            }));
            let payload_value = Value::LiteralValue(LiteralBits::Int(42));
            let frame =
                EvalFrame::from_bindings([(scrutinee, variant(some_tag, payload_value.clone()))])
                    .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            // If eval_branch recursed into eval_node(branch_id), it would
            // re-enter the same Branch infinitely; the test passing at
            // all confirms the sentinel detection.
            let value =
                eval_node(&dag, branch_id, &mut state, &strategy).expect("branch evaluates");

            assert_eq!(value, payload_value);
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        // E4 §B.1.3 fail-closed: every `BranchPath.pattern` must be
        // `ResolvedVariant` before evaluation proceeds. A matching
        // early arm cannot mask a later `UnresolvedVariant` —
        // unresolved-substrate state is a Fail-Closed (C-8) violation
        // regardless of which arm would have been selected.
        #[test]
        fn eval_branch_fails_closed_on_late_unresolved_arm_even_if_earlier_arm_matches() {
            let mut dag = Dag::std_fixture_bootstrap_snapshot();
            let matching_tag = declaration_id_by_name(&dag, "Bool");
            let scrutinee = dag.alloc_port_with_shape(dag.bool_shape().expect("Bool shape"));
            let early_output = dag.push_value(LiteralBits::Int(7), span());
            let early_body = node_for_port(&dag, early_output);
            let late_output = dag.push_value(LiteralBits::Int(13), span());
            let late_body = node_for_port(&dag, late_output);
            let output = dag.push_branch(
                scrutinee,
                vec![
                    // Early arm matches the scrutinee tag — without the
                    // pre-pass this would be returned and the late
                    // unresolved arm would slip through.
                    Path {
                        body: early_body,
                        output: early_output,
                        pattern: BranchPattern::ResolvedVariant(matching_tag),
                        binding: None,
                    },
                    Path {
                        body: late_body,
                        output: late_output,
                        pattern: BranchPattern::UnresolvedVariant {
                            name: "LateDrift".to_string(),
                            span: span(),
                        },
                        binding: None,
                    },
                ],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let frame = EvalFrame::from_bindings([(
                scrutinee,
                variant(matching_tag, Value::LiteralValue(LiteralBits::Bool(true))),
            )])
            .expect("frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let strategy = eager_strategy();

            let err = eval_node(&dag, entry, &mut state, &strategy)
                .expect_err("late unresolved arm rejected");

            assert_eq!(
                err,
                EvalError::BranchUnresolvedVariant {
                    node: entry,
                    name: "LateDrift".to_string(),
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_zero_iterations_returns_init_without_body_eval() {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(9), span());
            let zero = dag.push_value(LiteralBits::Int(0), span());
            let one = dag.push_value(LiteralBits::Int(1), span());
            let bad_body_output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Div)),
                vec![source, one],
                span(),
            );
            let bad_body = node_for_port(&dag, bad_body_output);
            let output = dag.push_loop(
                source,
                init,
                bad_body,
                LoopBound::Cardinality { count: zero },
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();
            let strategy = eager_strategy();

            let value = eval_node(&dag, entry, &mut state, &strategy).expect("zero loop");

            assert_eq!(
                value,
                Value::LiteralValue(LiteralBits::Int(9)),
                "zero iterations must not evaluate the unsupported body"
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_cardinality_threads_accumulator_through_body() {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(0), span());
            let count = dag.push_value(LiteralBits::Int(3), span());
            let one = dag.push_value(LiteralBits::Int(1), span());
            let body_output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![source, one],
                span(),
            );
            let body = node_for_port(&dag, body_output);
            let output =
                dag.push_loop(source, init, body, LoopBound::Cardinality { count }, span());
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let value =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect("loop evaluates");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(3)));
            assert_eq!(state.frames_outer_to_inner().len(), 1);
            assert_eq!(
                state.lookup(source),
                Err(EvalFrameError::UnboundPort { port: source }),
                "iteration accumulator binding must not leak"
            );
        }

        #[test]
        fn eval_loop_cardinality_missing_count_fails_closed() {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(0), span());
            let missing_count = dag.alloc_port(None);
            let body_output = dag.push_value(LiteralBits::Int(1), span());
            let body = node_for_port(&dag, body_output);
            let output = dag.push_loop(
                source,
                init,
                body,
                LoopBound::Cardinality {
                    count: missing_count,
                },
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("missing count");

            assert_eq!(
                err,
                EvalError::UnboundPort {
                    port: missing_count
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_cardinality_non_integer_count_fails_closed() {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(0), span());
            let count = dag.push_value(LiteralBits::String("three".to_string()), span());
            let body_output = dag.push_value(LiteralBits::Int(1), span());
            let body = node_for_port(&dag, body_output);
            let output =
                dag.push_loop(source, init, body, LoopBound::Cardinality { count }, span());
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err = eval_node(&dag, entry, &mut state, &eager_strategy())
                .expect_err("non-integer count");

            assert_eq!(
                err,
                EvalError::LoopCardinalityNonInteger { node: entry, count }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_cardinality_negative_count_fails_closed() {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(0), span());
            let count = dag.push_value(LiteralBits::Int(-1), span());
            let body_output = dag.push_value(LiteralBits::Int(1), span());
            let body = node_for_port(&dag, body_output);
            let output =
                dag.push_loop(source, init, body, LoopBound::Cardinality { count }, span());
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("negative count");

            assert_eq!(
                err,
                EvalError::LoopCardinalityNegative {
                    node: entry,
                    count,
                    value: -1,
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[cfg(target_pointer_width = "32")]
        #[test]
        fn eval_loop_cardinality_count_too_large_for_usize_fails_closed() {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(0), span());
            let too_large = i64::from(u32::MAX) + 1;
            let count = dag.push_value(LiteralBits::Int(too_large), span());
            let body_output = dag.push_value(LiteralBits::Int(1), span());
            let body = node_for_port(&dag, body_output);
            let output =
                dag.push_loop(source, init, body, LoopBound::Cardinality { count }, span());
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("count too large");

            assert_eq!(
                err,
                EvalError::LoopCardinalityTooLarge {
                    node: entry,
                    count,
                    value: too_large,
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_descent_bound_with_strict_per_path_proof_succeeds() {
            let (dag, loop_node) = descent_loop_fixture(7);
            let mut state = empty_state();
            let strategy = eager_strategy();
            let LoopBound::Descent { cluster, measure } = loop_node.bound else {
                panic!("descent bound");
            };

            let value = eval_loop_with_descent_execution_proof(
                &dag,
                loop_node,
                &mut state,
                &strategy,
                |_, proof_cluster, proof_measure| {
                    assert_eq!(proof_cluster, cluster);
                    assert_eq!(proof_measure, measure);
                    Ok(DescentExecutionProof {
                        cluster: proof_cluster,
                        port: proof_measure,
                        per_path: HashMap::from([(
                            descent_proof_path_key(
                                dag.cluster(cluster).intra_cluster_calls.first.transform,
                            ),
                            StrictEvidence::Strict,
                        )]),
                    })
                },
            )
            .expect("strict descent proof discharges loop bound");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(7)));
            assert_eq!(state.frames_outer_to_inner().len(), 1);
            assert_eq!(
                state.lookup(measure),
                Err(EvalFrameError::UnboundPort { port: measure }),
                "descent iteration binding must not leak"
            );
        }

        #[test]
        fn eval_loop_descent_bound_incomplete_per_path_coverage_fails_closed() {
            let (dag, loop_node) = descent_loop_fixture(1);
            let mut state = empty_state();
            let strategy = eager_strategy();
            let node = loop_node.id;
            let LoopBound::Descent { cluster, measure } = loop_node.bound else {
                panic!("descent bound");
            };

            let err = eval_loop_with_descent_execution_proof(
                &dag,
                loop_node,
                &mut state,
                &strategy,
                |_, proof_cluster, proof_measure| {
                    Ok(DescentExecutionProof {
                        cluster: proof_cluster,
                        port: proof_measure,
                        per_path: HashMap::new(),
                    })
                },
            )
            .expect_err("missing intra-cluster call coverage fails closed");

            assert_eq!(
                err,
                EvalError::LoopBoundDescentResidual {
                    node,
                    cluster,
                    measure,
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_descent_bound_evidence_incomplete_fails_closed() {
            let (dag, loop_node) = descent_loop_fixture(1);
            let mut state = empty_state();
            let strategy = eager_strategy();
            let node = loop_node.id;
            let LoopBound::Descent { cluster, measure } = loop_node.bound else {
                panic!("descent bound");
            };

            let err = eval_loop_with_descent_execution_proof(
                &dag,
                loop_node,
                &mut state,
                &strategy,
                |_, _, _| Err(DescentResidual::EvidenceIncomplete),
            )
            .expect_err("incomplete descent proof fails closed");

            assert_eq!(
                err,
                EvalError::LoopBoundDescentResidual {
                    node,
                    cluster,
                    measure,
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_descent_bound_evidence_unknown_non_increasing_fails_closed() {
            let (dag, loop_node) = descent_loop_fixture(1);
            let mut state = empty_state();
            let strategy = eager_strategy();
            let node = loop_node.id;
            let LoopBound::Descent { cluster, measure } = loop_node.bound else {
                panic!("descent bound");
            };

            let err = eval_loop_with_descent_execution_proof(
                &dag,
                loop_node,
                &mut state,
                &strategy,
                |_, _, _| {
                    Err(DescentResidual::EvidenceUnknown(
                        NonStrictEvidence::NonIncreasing,
                    ))
                },
            )
            .expect_err("non-increasing descent proof fails closed");

            assert_eq!(
                err,
                EvalError::LoopBoundDescentResidual {
                    node,
                    cluster,
                    measure,
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_descent_bound_evidence_unknown_descent_unknown_fails_closed() {
            let (dag, loop_node) = descent_loop_fixture(1);
            let mut state = empty_state();
            let strategy = eager_strategy();
            let node = loop_node.id;
            let LoopBound::Descent { cluster, measure } = loop_node.bound else {
                panic!("descent bound");
            };

            let err = eval_loop_with_descent_execution_proof(
                &dag,
                loop_node,
                &mut state,
                &strategy,
                |_, _, _| {
                    Err(DescentResidual::EvidenceUnknown(
                        NonStrictEvidence::DescentUnknown,
                    ))
                },
            )
            .expect_err("unknown descent proof fails closed");

            assert_eq!(
                err,
                EvalError::LoopBoundDescentResidual {
                    node,
                    cluster,
                    measure,
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_loop_restores_stack_after_body_diagnostic() {
            let mut dag = Dag::new();
            let source = dag.alloc_port(None);
            let init = dag.push_value(LiteralBits::Int(0), span());
            let count = dag.push_value(LiteralBits::Int(1), span());
            let one = dag.push_value(LiteralBits::Int(1), span());
            let bad_body_output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Div)),
                vec![source, one],
                span(),
            );
            let bad_body = node_for_port(&dag, bad_body_output);
            let output = dag.push_loop(
                source,
                init,
                bad_body,
                LoopBound::Cardinality { count },
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("body diagnostic");

            assert_eq!(
                err,
                EvalError::UnsupportedTransformTarget {
                    kind: "ArithmeticDiv",
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
            assert_eq!(
                state.lookup(source),
                Err(EvalFrameError::UnboundPort { port: source }),
                "failed iteration frame must be popped"
            );
        }

        // E3 Transform — preserved from main during merge.

        #[test]
        fn transform_arithmetic_div_unsupported_until_result_carrier_eval_lands() {
            let mut dag = Dag::new();
            let lhs = dag.push_value(LiteralBits::Int(8), span());
            let rhs = dag.push_value(LiteralBits::Int(2), span());
            let output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Div)),
                vec![lhs, rhs],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err = eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("div");

            assert_eq!(
                err,
                EvalError::UnsupportedTransformTarget {
                    kind: "ArithmeticDiv",
                }
            );
        }

        #[test]
        fn transform_comparison_int_evaluates() {
            let mut dag = Dag::new();
            let lhs = dag.push_value(LiteralBits::Int(2), span());
            let rhs = dag.push_value(LiteralBits::Int(3), span());
            let output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Lt)),
                vec![lhs, rhs],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let value = eval_node(&dag, entry, &mut state, &eager_strategy()).expect("cmp");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Bool(true)));
        }

        #[test]
        fn transform_comparison_bool_matches_rust_total_order() {
            let mut dag = Dag::new();
            let lhs = dag.push_value(LiteralBits::Bool(false), span());
            let rhs = dag.push_value(LiteralBits::Bool(true), span());
            let output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Lt)),
                vec![lhs, rhs],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let value = eval_node(&dag, entry, &mut state, &eager_strategy()).expect("bool lt");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Bool(true)));
        }

        #[test]
        fn transform_logical_operator_unsupported_in_e3_slice() {
            let mut dag = Dag::new();
            let lhs = dag.push_value(LiteralBits::Bool(true), span());
            let rhs = dag.push_value(LiteralBits::Bool(false), span());
            let output = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
                vec![lhs, rhs],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err = eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("logical");

            assert_eq!(
                err,
                EvalError::UnsupportedTransformTarget { kind: "Logical" }
            );
        }

        #[test]
        fn transform_field_project_non_record_carrier_fails_closed() {
            // E6-G0c: FieldProject now executes; non-record carrier must
            // fail closed with a typed BadTransformOperands rather than
            // returning an unrelated value.
            let mut dag = Dag::new();
            let v = dag.push_value(LiteralBits::Int(1), span());
            let output = dag.push_transform(
                TransformTarget::FieldProject {
                    field_label: "x".to_string(),
                    field_child: None,
                },
                vec![v],
                span(),
            );
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err = eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("project");

            assert_eq!(
                err,
                EvalError::BadTransformOperands {
                    reason: "FieldProject carrier must be a RecordValue",
                }
            );
        }

        #[test]
        fn transform_callable_non_arrow_decl_fails_closed() {
            // E6-G0c: Callable now executes for Arrow-bodied user
            // functions; a non-Arrow declaration (e.g., Bool's Disj
            // connective) must fail closed with a typed
            // BadTransformOperands rather than the prior blanket
            // UnsupportedTransformTarget.
            let mut dag = Dag::new();
            let target_decl = dag.declaration_by_name("Bool").expect("Bool").id;
            let output = dag.push_transform(TransformTarget::Callable(target_decl), vec![], span());
            let entry = node_for_port(&dag, output);
            let mut state = empty_state();

            let err = eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("callable");

            assert_eq!(
                err,
                EvalError::BadTransformOperands {
                    reason: "Callable target declaration is not an Arrow type",
                }
            );
        }

        #[test]
        fn eval_bind_value_binding_returns_body_port_value() {
            let mut dag = Dag::new();
            let value = dag.push_value(LiteralBits::Bool(false), span());
            let entry = dag.push_bind("flag", value, Vec::new(), span());
            let mut state = empty_state();
            let strategy = eager_strategy();

            let out = eval_node(&dag, entry, &mut state, &strategy).expect("bind evaluates");

            assert_eq!(out, Value::LiteralValue(LiteralBits::Bool(false)));
            assert!(matches!(dag.node(entry), Behavior::Bind(_)));
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_bind_copies_callable_param_into_fresh_frame_for_body() {
            let mut dag = Dag::new();
            let param = dag.alloc_port(None);
            let one = dag.push_value(LiteralBits::Int(1), span());
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![param, one],
                span(),
            );
            let entry = dag.push_bind("increment", body, vec![param], span());
            let caller =
                EvalFrame::from_bindings([(param, Value::LiteralValue(LiteralBits::Int(41)))])
                    .expect("caller frame");
            let mut state = EvalStateStack::with_root_frame(caller);

            let out = eval_node(&dag, entry, &mut state, &eager_strategy())
                .expect("callable bind evaluates");

            assert_eq!(out, Value::LiteralValue(LiteralBits::Int(42)));
            assert_eq!(state.frames_outer_to_inner().len(), 1);
            assert_eq!(
                state.lookup(param),
                Ok(&Value::LiteralValue(LiteralBits::Int(41))),
                "caller binding remains in the outer frame"
            );
        }

        #[test]
        fn eval_bind_body_result_can_be_parameter_port() {
            let mut dag = Dag::new();
            let param = dag.alloc_port(None);
            let entry = dag.push_bind("identity", param, vec![param], span());
            let caller = EvalFrame::from_bindings([(
                param,
                Value::LiteralValue(LiteralBits::String("arg".to_string())),
            )])
            .expect("caller frame");
            let mut state = EvalStateStack::with_root_frame(caller);

            let out = eval_node(&dag, entry, &mut state, &eager_strategy()).expect("identity bind");

            assert_eq!(
                out,
                Value::LiteralValue(LiteralBits::String("arg".to_string()))
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_bind_duplicate_param_fails_closed_and_restores_stack() {
            let mut dag = Dag::new();
            let param = dag.alloc_port(None);
            let entry = dag.push_bind("dup", param, vec![param, param], span());
            let caller =
                EvalFrame::from_bindings([(param, Value::LiteralValue(LiteralBits::Int(1)))])
                    .expect("caller frame");
            let mut state = EvalStateStack::with_root_frame(caller);

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("duplicate param");

            assert_eq!(
                err,
                EvalError::FrameError(EvalFrameError::DuplicateBinding { port: param })
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        // ---- E6-G0c: Callable / FieldProject execution ----

        fn alloc_arrow_decl_with_bind(
            dag: &mut Dag,
            bind_node: NodeId,
            arity: usize,
        ) -> DeclarationId {
            // Wrap an existing Bind body in an Arrow declaration so the
            // evaluator's Callable arm can find it via
            // `dag.declaration(decl).connective`. `arity` matches the
            // bind's parameter count; `inputs` holds `arity` Int decls
            // so the builder's `callable_runtime_arity` agrees with the
            // operand count at the call site.
            let int_decl = dag.int_shape().expect("Int decl in bootstrap").declaration;
            let bind_id =
                crate::dag::BindNodeId::from_bind_node(dag, bind_node).expect("bind node id");
            let id = dag.alloc_declaration_id();
            dag.push_declaration(crate::dag::Declaration {
                id,
                name: Some("user_callable".to_string()),
                connective: crate::dag::TypeConnective::Arrow {
                    inputs: vec![int_decl; arity],
                    output: int_decl,
                    body: crate::dag::ArrowBody::UserDefined(bind_id),
                },
                type_params: Vec::new(),
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: None,
                span: span(),
            });
            id
        }

        #[test]
        fn transform_field_project_returns_named_field_value() {
            // Lift the RecordValue carrier directly into the caller frame
            // — the FieldProject arm reads from the projected carrier's
            // Value shape, not from a Record-lowering port path.
            let mut dag = Dag::new();
            let carrier_port = dag.alloc_port(None);
            let carrier_value = Value::RecordValue(vec![
                crate::evaluator::NamedField {
                    label: "x".to_string(),
                    value: Value::LiteralValue(LiteralBits::Int(7)),
                },
                crate::evaluator::NamedField {
                    label: "y".to_string(),
                    value: Value::LiteralValue(LiteralBits::Int(11)),
                },
            ]);
            let frame =
                EvalFrame::from_bindings([(carrier_port, carrier_value)]).expect("caller frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let output = dag.push_transform(
                TransformTarget::FieldProject {
                    field_label: "y".to_string(),
                    field_child: None,
                },
                vec![carrier_port],
                span(),
            );
            let entry = node_for_port(&dag, output);

            let value = eval_node(&dag, entry, &mut state, &eager_strategy()).expect("y projects");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(11)));
        }

        #[test]
        fn transform_field_project_missing_label_fails_closed() {
            let mut dag = Dag::new();
            let carrier_port = dag.alloc_port(None);
            let carrier_value = Value::RecordValue(vec![crate::evaluator::NamedField {
                label: "only".to_string(),
                value: Value::LiteralValue(LiteralBits::Int(1)),
            }]);
            let frame =
                EvalFrame::from_bindings([(carrier_port, carrier_value)]).expect("caller frame");
            let mut state = EvalStateStack::with_root_frame(frame);
            let output = dag.push_transform(
                TransformTarget::FieldProject {
                    field_label: "missing".to_string(),
                    field_child: None,
                },
                vec![carrier_port],
                span(),
            );
            let entry = node_for_port(&dag, output);

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("missing label");

            assert_eq!(
                err,
                EvalError::BadTransformOperands {
                    reason: "FieldProject label not present on RecordValue carrier",
                }
            );
        }

        // (FieldProject / Callable arity mismatches are unrepresentable
        // at construction: `Dag::push_transform`'s builder asserts arity
        // matches `field_child`/`callable_runtime_arity`. The runtime
        // arity guards in the evaluator are defense-in-depth for future
        // builders that bypass that assertion; they are not exercisable
        // through the standard construction path. No runtime-only arity
        // tests here.)

        #[test]
        fn transform_callable_executes_user_function_body() {
            // E6-G0c positive: build `fn user_callable(n) = n + n` as an
            // Arrow declaration with UserDefined body. Invoke through
            // TransformTarget::Callable with operand 21; expect 42.
            let mut dag = Dag::new();
            let param = dag.alloc_port(None);
            let two_n = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![param, param],
                span(),
            );
            let bind_node = dag.push_bind("user_callable", two_n, vec![param], span());
            let callee_decl = alloc_arrow_decl_with_bind(&mut dag, bind_node, /* arity */ 1);
            let twenty_one = dag.push_value(LiteralBits::Int(21), span());
            let call_output = dag.push_transform(
                TransformTarget::Callable(callee_decl),
                vec![twenty_one],
                span(),
            );
            let entry = node_for_port(&dag, call_output);
            let mut state = empty_state();

            let value =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect("user callable");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(42)));
            assert_eq!(state.frames_outer_to_inner().len(), 1, "frame popped");
        }

        #[test]
        fn transform_callable_evaluates_inputs_left_to_right() {
            // Build `fn sub(a, b) = a - b` and invoke `sub(10, 3)` →
            // expect 7. The argument-evaluation order is left-to-right;
            // Sub is non-commutative, so a swap would surface a swap to
            // -7. Pinning the orientation is enough for a left-first
            // ordering check at this layer.
            let mut dag = Dag::new();
            let a = dag.alloc_port(None);
            let b = dag.alloc_port(None);
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Sub)),
                vec![a, b],
                span(),
            );
            let bind_node = dag.push_bind("sub", body, vec![a, b], span());
            let callee_decl = alloc_arrow_decl_with_bind(&mut dag, bind_node, /* arity */ 2);
            let ten = dag.push_value(LiteralBits::Int(10), span());
            let three = dag.push_value(LiteralBits::Int(3), span());
            let call_output = dag.push_transform(
                TransformTarget::Callable(callee_decl),
                vec![ten, three],
                span(),
            );
            let entry = node_for_port(&dag, call_output);
            let mut state = empty_state();

            let value = eval_node(&dag, entry, &mut state, &eager_strategy()).expect("sub");

            assert_eq!(value, Value::LiteralValue(LiteralBits::Int(7)));
        }

        #[test]
        fn eval_bind_unbound_param_fails_closed_without_pushing_frame() {
            let mut dag = Dag::new();
            let param = dag.alloc_port(None);
            let entry = dag.push_bind("missing", param, vec![param], span());
            let mut state = empty_state();

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("unbound param");

            assert_eq!(
                err,
                EvalError::FrameError(EvalFrameError::UnboundPort { port: param })
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
        }

        #[test]
        fn eval_bind_restores_stack_after_body_diagnostic() {
            let mut dag = Dag::new();
            let param = dag.alloc_port(None);
            let one = dag.push_value(LiteralBits::Int(1), span());
            let bad_body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Div)),
                vec![param, one],
                span(),
            );
            let entry = dag.push_bind("bad", bad_body, vec![param], span());
            let caller =
                EvalFrame::from_bindings([(param, Value::LiteralValue(LiteralBits::Int(8)))])
                    .expect("caller frame");
            let mut state = EvalStateStack::with_root_frame(caller);

            let err =
                eval_node(&dag, entry, &mut state, &eager_strategy()).expect_err("body diagnostic");

            assert_eq!(
                err,
                EvalError::UnsupportedTransformTarget {
                    kind: "ArithmeticDiv",
                }
            );
            assert_eq!(state.frames_outer_to_inner().len(), 1);
            assert_eq!(
                state.lookup(param),
                Ok(&Value::LiteralValue(LiteralBits::Int(8)))
            );
        }

        #[test]
        fn lookup_walks_innermost_frame_first() {
            let ids = ports(2);
            let outer = EvalFrame::from_bindings([(ids[0], "outer"), (ids[1], "outer-only")])
                .expect("outer frame");
            let inner = EvalFrame::from_bindings([(ids[0], "inner")]).expect("inner frame");
            let stack = EvalStateStack::from_outer_to_inner(vec![outer, inner]);

            assert_eq!(stack.lookup(ids[0]), Ok(&"inner"));
            assert_eq!(stack.lookup(ids[1]), Ok(&"outer-only"));
        }

        #[test]
        fn bind_top_writes_only_the_innermost_frame() {
            let ids = ports(1);
            let outer = EvalFrame::from_bindings([(ids[0], "outer")]).expect("outer frame");
            let inner = EvalFrame::empty();
            let mut stack = EvalStateStack::from_outer_to_inner(vec![outer, inner]);

            stack.bind_top(ids[0], "inner").expect("bind top");

            assert_eq!(stack.lookup(ids[0]), Ok(&"inner"));
            assert_eq!(
                stack.frames_outer_to_inner()[0].lookup_local(ids[0]),
                Some(&"outer")
            );
            assert_eq!(
                stack.frames_outer_to_inner()[1].lookup_local(ids[0]),
                Some(&"inner")
            );
        }

        #[test]
        fn bind_top_rejects_duplicate_binding_in_current_frame() {
            let ids = ports(1);
            let mut stack =
                EvalStateStack::with_root_frame(EvalFrame::from_bindings([(ids[0], 1)]).unwrap());

            let err = stack.bind_top(ids[0], 2).expect_err("duplicate rejected");

            assert_eq!(err, EvalFrameError::DuplicateBinding { port: ids[0] });
            assert_eq!(stack.lookup(ids[0]), Ok(&1));
        }

        #[test]
        fn lookup_reports_unbound_port_after_full_stack_walk() {
            let ids = ports(2);
            let stack =
                EvalStateStack::with_root_frame(EvalFrame::from_bindings([(ids[0], 1)]).unwrap());

            let err = stack.lookup(ids[1]).expect_err("unbound rejected");

            assert_eq!(err, EvalFrameError::UnboundPort { port: ids[1] });
        }

        #[test]
        fn bind_top_reports_empty_state_stack() {
            let ids = ports(1);
            let mut stack: EvalStateStack<i64> = EvalStateStack::from_outer_to_inner(Vec::new());

            let err = stack.bind_top(ids[0], 1).expect_err("empty stack rejected");

            assert_eq!(err, EvalFrameError::EmptyStateStack);
        }
    }
}
mod int_literal_ranges;
/// T-LensAPI D1: bounded lens interpreter over substrate-shaped [`FieldValue`]
/// (see module docs in `lens_apply.rs`).
pub mod lens_apply;
pub mod lens_testgen;

/// Effect-enumeration lens. Authority lives in
/// `src/v3/lenses/effect_enumeration.dag`; the Rust projection is
/// auto-emitted into `src/v3/compiler/src/lens_effect_enumeration_generated.rs`
/// and wrapped here so callers use `v3_compiler::lens_effect_enumeration`.
/// Editing the lens means editing the `.dag` and regenerating the checked-in
/// projection in the same change.
pub mod lens_effect_enumeration {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_effect_enumeration_generated.rs");
    }

    pub use generated::{
        enumerate_effects, CoverageGap, EffectEnumerationReport, EffectFact, RedundantReadError,
        StructuralEffectShape, TransactionalPattern,
    };
}

/// Unused-parameters lens. Authority lives in `src/v3/lenses/unused_parameters.dag`;
/// the Rust projection is emitted into `lens_unused_parameters_generated.rs` and
/// wrapped here inline (same host pattern as `lens_cost` / `lens_provenance`).
pub mod lens_unused_parameters {
    use crate::dag::{NodeId, PortId};
    use crate::diagnostics::SourceSpan;
    use crate::Dag;

    mod generated {
        #![allow(
            dead_code,
            unused_imports,
            unused_parens,
            unused_variables,
            clippy::clone_on_copy,
            clippy::collapsible_else_if
        )]

        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_unused_parameters_generated.rs");
    }

    #[derive(Debug, Clone, Default)]
    pub struct UnusedParametersConfig {}

    #[derive(Debug, Clone)]
    pub struct UnusedParameter {
        pub function: NodeId,
        pub parameter: PortId,
        pub parameter_index: usize,
        pub function_span: SourceSpan,
    }

    pub struct UnusedParametersLens<'a> {
        dag: &'a Dag,
    }

    impl<'a> UnusedParametersLens<'a> {
        pub fn new(dag: &'a Dag) -> Self {
            Self { dag }
        }

        pub fn query(&self, _config: &UnusedParametersConfig) -> Vec<UnusedParameter> {
            generated::check(self.dag)
                .into_iter()
                .map(|violation| UnusedParameter {
                    function: violation.function,
                    parameter: violation.parameter,
                    parameter_index: usize::try_from(violation.parameter_index)
                        .expect("compiled lens should emit non-negative parameter indexes"),
                    function_span: violation.function_span,
                })
                .collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{UnusedParametersConfig, UnusedParametersLens};
        use crate::dag::{
            BranchPattern, Dag, LiteralBits, LoopBound, Path, PortId, TransformTarget,
        };
        use crate::diagnostics::SourceSpan;
        use crate::operators::{ArithmeticOp, ComparisonOp, OperatorKind};

        const DIRECT_DAG_FILE: &str = "lens_unused_parameters.unit";

        fn span() -> SourceSpan {
            SourceSpan::new(DIRECT_DAG_FILE, 0, 0)
        }

        fn unused_parameter_indexes(dag: &Dag) -> Vec<usize> {
            let lens = UnusedParametersLens::new(dag);
            let mut indexes: Vec<_> = lens
                .query(&UnusedParametersConfig::default())
                .into_iter()
                .map(|violation| violation.parameter_index)
                .collect();
            indexes.sort_unstable();
            indexes
        }

        fn int_value(dag: &mut Dag, value: i64) -> PortId {
            dag.push_value(LiteralBits::Int(value), span())
        }

        fn add(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
            dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![lhs, rhs],
                span(),
            )
        }

        fn gt(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
            dag.push_transform(
                TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Gt)),
                vec![lhs, rhs],
                span(),
            )
        }

        fn int_params(dag: &mut Dag, count: usize) -> Vec<PortId> {
            let int_shape = dag.int_shape().expect("bootstrap Dag has Int");
            (0..count)
                .map(|_| dag.alloc_port_with_shape(int_shape))
                .collect()
        }

        fn producer_or_bind(dag: &mut Dag, name: &str, output: PortId) -> crate::dag::NodeId {
            dag.port(output)
                .produced_by
                .unwrap_or_else(|| dag.push_bind(name, output, Vec::new(), span()))
        }

        fn bind_arm(dag: &mut Dag, name: &str, output: PortId) -> Path {
            Path {
                body: producer_or_bind(dag, name, output),
                output,
                pattern: BranchPattern::UnresolvedVariant {
                    name: name.to_string(),
                    span: span(),
                },
                binding: None,
            }
        }

        fn function_dag<F>(name: &str, param_count: usize, build_body: F) -> Dag
        where
            F: FnOnce(&mut Dag, &[PortId]) -> PortId,
        {
            let mut dag = Dag::new();
            let params = int_params(&mut dag, param_count);
            let value = build_body(&mut dag, &params);
            dag.push_bind(name, value, params, span());
            dag
        }

        #[test]
        fn unused_params_empty_for_function_using_every_parameter() {
            let dag = function_dag("add", 2, |dag, params| add(dag, params[0], params[1]));

            assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
        }

        #[test]
        fn unused_params_reports_single_unused_parameter() {
            let dag = function_dag("first", 2, |_dag, params| params[0]);

            assert_eq!(unused_parameter_indexes(&dag), vec![1]);
        }

        #[test]
        fn unused_params_reports_all_parameters_for_constant_body() {
            let dag = function_dag("always_one", 3, |dag, _params| int_value(dag, 1));

            assert_eq!(unused_parameter_indexes(&dag), vec![0, 1, 2]);
        }

        #[test]
        fn unused_params_skips_value_bindings() {
            let mut dag = Dag::new();
            let lhs = int_value(&mut dag, 1);
            let rhs = int_value(&mut dag, 2);
            let value = add(&mut dag, lhs, rhs);
            dag.push_bind("x", value, Vec::new(), span());

            assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
        }

        #[test]
        fn unused_params_handles_branch_in_body() {
            let dag = function_dag("pick", 2, |dag, params| {
                let zero = int_value(dag, 0);
                let cond = gt(dag, params[0], zero);
                let then_path = bind_arm(dag, "then_arm", params[0]);
                let else_path = bind_arm(dag, "else_arm", params[1]);
                dag.push_branch(cond, vec![then_path, else_path], span())
            });

            assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
        }

        #[test]
        fn unused_params_bootstrap_baseline_is_empty() {
            let dag = Dag::new();
            assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
        }

        #[test]
        fn unused_params_reports_unused_in_branch_body() {
            let dag = function_dag("always_a", 2, |dag, params| {
                let zero = int_value(dag, 0);
                let cond = gt(dag, params[0], zero);
                let then_path = bind_arm(dag, "then_arm", params[0]);
                let else_path = bind_arm(dag, "else_arm", params[0]);
                dag.push_branch(cond, vec![then_path, else_path], span())
            });

            assert_eq!(unused_parameter_indexes(&dag), vec![1]);
        }

        #[test]
        fn unused_params_descends_into_loop_body_for_recursive_calls() {
            let dag = function_dag("count_down", 2, |dag, params| {
                let body_output = add(dag, params[0], params[1]);
                let count = int_value(dag, 1);
                let body = producer_or_bind(dag, "loop_body", body_output);
                dag.push_loop(
                    params[0],
                    params[1],
                    body,
                    LoopBound::Cardinality { count },
                    span(),
                )
            });

            assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
        }

        #[test]
        fn unused_params_loop_body_descent_finds_param_only_used_in_recursion() {
            let dag = function_dag("count_down", 2, |dag, params| {
                let body_output = add(dag, params[0], params[1]);
                let init = int_value(dag, 0);
                let count = int_value(dag, 1);
                let body = producer_or_bind(dag, "loop_body", body_output);
                dag.push_loop(
                    params[0],
                    init,
                    body,
                    LoopBound::Cardinality { count },
                    span(),
                )
            });

            assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
        }
    }
}

/// DB-8 / m1_3 / R1C-E: shared `PROGRAM_FIXTURES` + reflected harness table.
pub mod emit_rust_roundtrip_fixtures;
pub mod post_emit_verifier;
pub mod r1c_e_gates;
pub mod test_runner;
pub mod serialize {
    use crate::dag::{Behavior, Dag};
    use crate::diagnostics::Diagnostic;

    include!("serialize_generated.rs");
}
pub mod types {
    use crate::dag::DeclarationId;

    include!("types_generated.rs");
}
pub mod parse_surface {
    use crate::diagnostics::SourceSpan;
    use crate::operators::OperatorKind;

    include!("parse_surface_generated.rs");
}

pub use regen_bootstrap_emit::{render_bootstrap_generated_rs, render_bootstrap_std_generated_rs};

/// Operator symbols and `algebra_field_name` projections.
///
/// **Single authority:** `src/v3/compiler/operators.dag` → `operators_generated.rs`
/// (see the generated file header; do not add a parallel hand-written
/// `operators.rs` — reviewers sometimes misread the crate layout that way).
pub mod operators {
    pub use crate::dag::{ArithmeticOp, ComparisonOp, LogicalOp, OperatorKind};

    mod generated {
        #![allow(
            dead_code,
            unused_imports,
            unused_parens,
            unused_variables,
            clippy::clone_on_copy,
            clippy::collapsible_else_if
        )]

        use crate::dag::{ArithmeticOp, ComparisonOp, LogicalOp, OperatorKind};

        include!("operators_generated.rs");
    }

    pub use generated::{algebra_field_name, from_symbol, symbol};
}

/// SG-2c grammar-tables prototype (SG-2c-1 binary ops, SG-2c-2 item-keyword
/// dispatch, SG-2c-3 type-RHS boundary keywords, SG-2c-4 brackets, SG-2c-5 soft
/// keyword idents, SG-2c-6/7 `parse_primary` prefix + atom cluster).
/// Authority: `src/v3/compiler/parse_tables.dag`.
/// The generated Rust projection is emitted by `regen_parse_tables` and consumed
/// from `parse_parser_body.txt` so the parser no longer open-codes token → operator
/// matches or keyword membership tables for top-level type lookahead. Full parser authority (SG-2c
/// proper) is blocked on recursive list-body emission over `List<Token>`; see
/// `parse_tables.dag` header for the dissolution trigger.
pub mod parse_tables {
    #![allow(
        dead_code,
        unused_imports,
        unused_parens,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]

    include!("parse_tables_generated.rs");
}

/// Complexity lens. The authority lives in `src/v3/lenses/complexity.dag`;
/// the generated surface now returns `Lookup<ComplexitySummary>` via
/// `complexity_of`. `cost_of` below is a compatibility adapter for older
/// structural-depth test-runner paths; new consumers should read
/// `complexity_of`.
pub mod lens_cost {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if,
        clippy::double_parens,
        clippy::eq_op,
        clippy::large_enum_variant
    )]
    mod generated {
        // Regen can emit a redundant paren around some `Hit(...)` payload
        // subexpressions (`Hit((1 + n))` vs `Hit(1 + n)`) — relax until emission
        // drops one stable layer of grouping.
        use crate::complexity_lattice::complexity_enforcement_budget_dominates as asymptotic_dominates;
        use crate::dag::*;
        use crate::diagnostics::*;
        use crate::lens_t_las_carrier::{
            EnforceableLens, Lens, LensEnforcement, Monoid, OptionalDiagnostic,
        };
        use crate::Witness;

        include!("complexity_lens_generated.rs");
    }

    pub use generated::{
        complexity_enforcement_project, complexity_enforcement_violates, complexity_of,
        complexity_summary_work_class_consistent, Certainty, ComplexityEntry, ComplexitySummary,
        DominanceOutcome,
    };
    pub type ComplexityLookup = crate::dag::Lookup<ComplexitySummary>;

    #[cfg(test)]
    mod complexity_summary_work_class_consistent_tests {
        use super::{complexity_summary_work_class_consistent, Certainty, ComplexitySummary};
        use crate::dag::{AsymptoticClass, PortId, SizeVariable, SymbolicCost};

        #[test]
        fn rejects_under_reported_asymptotic_class_class_log_vs_classified_linear_work() {
            let summary = ComplexitySummary {
                work: SymbolicCost::LinearCost {
                    _0: SizeVariable {
                        source_port: PortId::test_raw(1),
                        display_name: None,
                    },
                },
                span: SymbolicCost::ConstantCost { _0: 0 },
                asymptotic_class: AsymptoticClass::ClassLog,
                work_certainty: Certainty::Proven,
                span_certainty: Certainty::Proven,
            };
            assert!(
                !complexity_summary_work_class_consistent(&summary),
                "stored ClassLog must not cover classified ClassLinear work"
            );
        }

        #[test]
        fn accepts_stored_class_covering_classified_work_class_linear_covers_log_work() {
            let summary = ComplexitySummary {
                work: SymbolicCost::LogCost {
                    _0: SizeVariable {
                        source_port: PortId::test_raw(2),
                        display_name: None,
                    },
                },
                span: SymbolicCost::ConstantCost { _0: 0 },
                asymptotic_class: AsymptoticClass::ClassLinear,
                work_certainty: Certainty::Proven,
                span_certainty: Certainty::Proven,
            };
            assert!(complexity_summary_work_class_consistent(&summary));
        }
    }

    /// Back-compat projection for pre-`ComplexitySummary` callers that still
    /// assert the old structural depth integer. This is not the behavioral
    /// completion surface.
    pub type CostLookup = crate::dag::Lookup<i64>;

    pub fn cost_of(dag: &crate::dag::Dag, port: &crate::dag::PortId) -> CostLookup {
        legacy_lookup_cost(&legacy_compute_costs(dag), port)
    }

    fn legacy_compute_costs(dag: &crate::dag::Dag) -> Vec<(crate::dag::PortId, CostLookup)> {
        dag.nodes()
            .iter()
            .fold(legacy_seed_bind_params(dag.nodes()), |mut acc, behavior| {
                acc.insert(0, legacy_entry_for(dag, &acc, behavior));
                acc
            })
    }

    fn legacy_seed_bind_params(
        nodes: &[crate::dag::Behavior],
    ) -> Vec<(crate::dag::PortId, CostLookup)> {
        let mut out = Vec::new();
        for behavior in nodes {
            if let crate::dag::Behavior::Bind(bind) = behavior {
                for param in &bind.params {
                    out.insert(0, (*param, CostLookup::Hit(0)));
                }
            }
        }
        out
    }

    fn legacy_entry_for(
        _dag: &crate::dag::Dag,
        acc: &[(crate::dag::PortId, CostLookup)],
        behavior: &crate::dag::Behavior,
    ) -> (crate::dag::PortId, CostLookup) {
        use crate::dag::Behavior;
        match behavior {
            Behavior::Value(v) => (v.result_port(), CostLookup::Hit(0)),
            Behavior::Transform(t) => (
                t.result_port(),
                legacy_add_one(&legacy_sum_costs(acc, &t.inputs)),
            ),
            Behavior::Branch(b) => (
                b.result_port(),
                legacy_add_one(&legacy_add_cost(
                    &legacy_lookup_cost(acc, &b.input),
                    &legacy_max_path_cost(acc, &b.paths),
                )),
            ),
            Behavior::Loop(l) => (
                l.result_port(),
                legacy_add_one(&legacy_add_cost(
                    &legacy_lookup_cost(acc, &l.source),
                    &legacy_lookup_cost(acc, &l.init),
                )),
            ),
            Behavior::Bind(bind) => (
                bind.result_port(),
                legacy_lookup_cost(acc, &bind.result_port()),
            ),
        }
    }

    fn legacy_sum_costs(
        acc: &[(crate::dag::PortId, CostLookup)],
        ports: &[crate::dag::PortId],
    ) -> CostLookup {
        ports.iter().fold(CostLookup::Hit(0), |sum, port_id| {
            legacy_add_cost(&sum, &legacy_lookup_cost(acc, port_id))
        })
    }

    fn legacy_max_path_cost(
        acc: &[(crate::dag::PortId, CostLookup)],
        paths: &[crate::dag::Path],
    ) -> CostLookup {
        paths.iter().fold(CostLookup::Hit(0), |best, path| {
            legacy_max_cost(&best, &legacy_lookup_cost(acc, &path.result_port()))
        })
    }

    fn legacy_lookup_cost(
        acc: &[(crate::dag::PortId, CostLookup)],
        port_id: &crate::dag::PortId,
    ) -> CostLookup {
        acc.iter()
            .find(|(port, _)| port == port_id)
            .map(|(_, cost)| cost.clone())
            .unwrap_or(CostLookup::Miss)
    }

    fn legacy_add_one(c: &CostLookup) -> CostLookup {
        match c {
            CostLookup::Miss => CostLookup::Miss,
            CostLookup::Hit(n) => CostLookup::Hit(n + 1),
        }
    }

    fn legacy_add_cost(a: &CostLookup, b: &CostLookup) -> CostLookup {
        match a {
            CostLookup::Miss => CostLookup::Miss,
            CostLookup::Hit(x) => match b {
                CostLookup::Miss => CostLookup::Miss,
                CostLookup::Hit(y) => CostLookup::Hit(x + y),
            },
        }
    }

    fn legacy_max_cost(a: &CostLookup, b: &CostLookup) -> CostLookup {
        match a {
            CostLookup::Miss => CostLookup::Miss,
            CostLookup::Hit(x) => match b {
                CostLookup::Miss => CostLookup::Miss,
                CostLookup::Hit(y) => CostLookup::Hit((*x).max(*y)),
            },
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{cost_of, CostLookup};
        use crate::dag::{
            ArithmeticOp, BranchPattern, Dag, LiteralBits, LoopBound, OperatorKind, Path, PortId,
            TransformTarget,
        };
        use crate::diagnostics::SourceSpan;

        fn span() -> SourceSpan {
            SourceSpan::new("<lens-cost-test>", 0, 0)
        }

        fn expect_found(lookup: CostLookup) -> i64 {
            match lookup {
                CostLookup::Hit(c) => c,
                CostLookup::Miss => panic!("expected Hit, got Miss"),
            }
        }

        fn assert_cost(dag: &Dag, port: PortId, expected: i64) {
            assert_eq!(expect_found(cost_of(dag, &port)), expected);
        }

        fn int_value(dag: &mut Dag, value: i64) -> PortId {
            dag.push_value(LiteralBits::Int(value), span())
        }

        fn add(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
            dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![lhs, rhs],
                span(),
            )
        }

        fn bind_arm(dag: &mut Dag, name: &str, output: PortId) -> Path {
            Path {
                body: dag.push_bind(name, output, Vec::new(), span()),
                output,
                pattern: BranchPattern::UnresolvedVariant {
                    name: name.to_string(),
                    span: span(),
                },
                binding: None,
            }
        }

        #[test]
        fn value_port_has_zero_cost() {
            let mut dag = Dag::new();
            let port = dag.push_value(LiteralBits::Int(7), span());
            assert_cost(&dag, port, 0);
        }

        #[test]
        fn transform_adds_one_to_sum_of_input_costs() {
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let sum = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            assert_cost(&dag, sum, 1);
        }

        #[test]
        fn chained_transforms_accumulate_through_input_edges() {
            // (1 + 2) + 3: outer transform = 1 + (inner=1) + (literal=0) = 2.
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let c = dag.push_value(LiteralBits::Int(3), span());
            let inner = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            let outer = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![inner, c],
                span(),
            );
            assert_cost(&dag, outer, 2);
        }

        #[test]
        fn branch_cost_is_one_plus_input_plus_max_of_path_outputs() {
            // cond=Bool(true) [0], arm=Int(1) [0] → branch = 1 + 0 + 0 = 1.
            let mut dag = Dag::new();
            let cond = dag.push_value(LiteralBits::Bool(true), span());
            let arm_output = int_value(&mut dag, 1);
            let arm_body = dag.push_bind("arm", arm_output, Vec::new(), span());
            let branch = dag.push_branch(
                cond,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Only".to_string(),
                        span: span(),
                    },
                    binding: None,
                }],
                span(),
            );
            assert_cost(&dag, branch, 1);
        }

        #[test]
        fn branch_cost_uses_max_not_sum_across_paths() {
            let mut dag = Dag::new();
            let cond = dag.push_value(LiteralBits::Bool(true), span());
            let cheap = int_value(&mut dag, 20);
            let forty = int_value(&mut dag, 40);
            let fifty = int_value(&mut dag, 50);
            let pricey = add(&mut dag, forty, fifty);
            let sixty = int_value(&mut dag, 60);
            let pricier = add(&mut dag, pricey, sixty);
            let paths = vec![
                bind_arm(&mut dag, "cheap_arm", cheap),
                bind_arm(&mut dag, "pricier_arm", pricier),
            ];
            let branch = dag.push_branch(cond, paths, span());

            // branch = 1 + cond(0) + max(cheap=0, pricier=((40+50)+60)=2)
            assert_cost(&dag, branch, 3);
        }

        #[test]
        fn loop_cost_is_one_plus_source_plus_init() {
            let mut dag = Dag::new();
            let source = dag.push_value(LiteralBits::Int(4), span());
            let init = dag.push_value(LiteralBits::Int(0), span());
            let body_output = dag.push_value(LiteralBits::Int(0), span());
            let body = dag.push_bind("loop_body", body_output, Vec::new(), span());
            let loop_port = dag.push_loop(
                source,
                init,
                body,
                LoopBound::Cardinality { count: source },
                span(),
            );
            assert_cost(&dag, loop_port, 1);
        }

        #[test]
        fn bind_cost_tracks_body_value_cost() {
            // let x = 1 + 2: bind.value is the Add transform (cost 1).
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            let _ = dag.push_bind("x", body, Vec::new(), span());
            assert_cost(&dag, body, 1);
        }

        #[test]
        fn bind_params_seed_as_zero_cost_and_body_costs_accumulate() {
            // fn double(x) = x + x: body transform reads x twice. Each param
            // port is seeded to cost 0, so body cost = 1 + 0 + 0 = 1.
            let mut dag = Dag::new();
            let int_shape = dag.int_shape().expect("bootstrap Int");
            let x = dag.alloc_port_with_shape(int_shape);
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![x, x],
                span(),
            );
            let _ = dag.push_bind("double", body, vec![x], span());

            // Parameter ports look up against the seeded entries.
            assert_cost(&dag, x, 0);
            assert_cost(&dag, body, 1);
        }
    }
}

/// Symbolic-cost lens (Lane 2 Stage 2d / DB-7). Authority lives in
/// `src/v3/lenses/cost.dag`; the Rust projection is auto-emitted
/// into `src/v3/compiler/src/lens_cost_symbolic_generated.rs` and
/// re-exported so callers use `v3_compiler::lens_cost_symbolic::*`.
///
/// The `SymbolicCost` + `SizeVariable` carriers live in
/// `src/v3/compiler/src/dag.rs` rather than the generated module
/// because they're declared in `src/v3/std/algebra.dag`, which
/// `emit_rust_module`'s `is_bootstrap_file` filter excludes from
/// type emission. The hand-maintained Rust mirror adjacent to
/// `Behavior` / `LoopBound` follows the same substrate-ownership
/// pattern the other bootstrap-resident types use.
pub mod lens_cost_symbolic {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_cost_symbolic_generated.rs");
    }

    pub use generated::{
        compute_symbolic_costs, symbolic_cost_of, transform_cost_for_target, CostBasisDeclaration,
        CostBasisKind, SymbolicCostEntry,
    };
    /// Rust projection of the shared `v3.std.lookup::Lookup` carrier
    /// at `SymbolicCost`. Alias (not a second sum type) — the lens now
    /// returns `Lookup<SymbolicCost>` directly; this name stays for
    /// embedder stability, mirroring `lens_cost::CostLookup`.
    pub type SymbolicCostLookup = crate::dag::Lookup<crate::dag::SymbolicCost>;
}

pub mod memory_peak_cost;

/// `cost_target_realization.dag` `.dag`-tier consumer of the
/// `declaration_by_name` substrate accessor (T-CostLens-Composition
/// Slice 1a.1; gunb-ai/gunbc#2141 ε scope per gunbc#2181 ratification).
/// Six per-category meta-type `Declaration?` resolvers covering all
/// `*Realization` carriers declared in `src/v3/std/emit_model.dag`
/// (TypeRealization / CallableRealization / OperatorRealization /
/// BehaviorRealization / TypeInstantiationRealization / PatternRealization).
pub mod lens_cost_target_realization {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;

        include!("lens_cost_target_realization_generated.rs");
    }

    pub use generated::{
        behavior_realization_meta, callable_realization_meta, operator_realization_meta,
        pattern_realization_meta, type_instantiation_realization_meta, type_realization_meta,
    };
}

/// Provenance lens. The authority lives in
/// `src/v3/lenses/provenance.dag`; the Rust projection is auto-emitted
/// into `src/v3/compiler/src/lens_provenance_generated.rs` and wrapped
/// here as a module so callers use `v3_compiler::lens_provenance`.
/// Editing the lens means editing the `.dag` — there is no hand-written
/// implementation on this crate side.
///
/// Only `Origin` and `origin_of` are re-exported. The generated module
/// also declares internal helper carriers (`PortLookup`,
/// `BehaviorLookup`) and their `find_*` / `behavior_id` walkers, which
/// exist solely because the substrate still exposes `Dag.ports` /
/// `Dag.nodes` as linear lists. Those helpers are bounded scaffolding
/// that dissolves when the substrate grows total keyed `port(id)` /
/// `node(id)` accessors — keeping them crate-private now prevents the
/// tracked-scaffold from leaking into `v3_compiler::lens_provenance`'s
/// public surface and attracting downstream consumers.
pub mod lens_provenance {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_provenance_generated.rs");
    }

    pub use generated::{origin_of, Origin};

    #[cfg(test)]
    mod tests {
        use super::{origin_of, Origin};
        use crate::dag::{
            ArithmeticOp, BranchPattern, Dag, LiteralBits, LoopBound, OperatorKind, Path,
            TransformTarget,
        };
        use crate::diagnostics::SourceSpan;

        fn span() -> SourceSpan {
            SourceSpan::new("<lens-provenance-test>", 0, 0)
        }

        fn label(origin: &Origin) -> &'static str {
            match origin {
                Origin::NoProducer => "NoProducer",
                Origin::MissingPort => "MissingPort",
                Origin::MissingBehavior => "MissingBehavior",
                Origin::Source { .. } => "Source",
                Origin::Computed { .. } => "Computed",
                Origin::Selected { .. } => "Selected",
                Origin::Accumulated { .. } => "Accumulated",
            }
        }

        #[test]
        fn unproduced_parameter_port_reports_no_producer() {
            let mut dag = Dag::new();
            let int_shape = dag.int_shape().expect("bootstrap Int");
            let param = dag.alloc_port_with_shape(int_shape);
            assert_eq!(label(&origin_of(&dag, &param)), "NoProducer");
        }

        #[test]
        fn value_port_reports_source_origin() {
            let mut dag = Dag::new();
            let port = dag.push_value(LiteralBits::Int(1), span());
            assert_eq!(label(&origin_of(&dag, &port)), "Source");
        }

        #[test]
        fn transform_port_reports_computed_origin() {
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let sum = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            assert_eq!(label(&origin_of(&dag, &sum)), "Computed");
        }

        #[test]
        fn branch_port_reports_selected_origin() {
            let mut dag = Dag::new();
            let cond = dag.push_value(LiteralBits::Bool(true), span());
            let arm_output = dag.push_value(LiteralBits::Int(1), span());
            let arm_body = dag.push_bind("arm", arm_output, Vec::new(), span());
            let branch = dag.push_branch(
                cond,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Only".to_string(),
                        span: span(),
                    },
                    binding: None,
                }],
                span(),
            );
            assert_eq!(label(&origin_of(&dag, &branch)), "Selected");
        }

        #[test]
        fn loop_port_reports_accumulated_origin() {
            let mut dag = Dag::new();
            let source = dag.push_value(LiteralBits::Int(4), span());
            let init = dag.push_value(LiteralBits::Int(0), span());
            let body_output = dag.push_value(LiteralBits::Int(0), span());
            let body = dag.push_bind("loop_body", body_output, Vec::new(), span());
            let loop_port = dag.push_loop(
                source,
                init,
                body,
                LoopBound::Cardinality { count: source },
                span(),
            );
            assert_eq!(label(&origin_of(&dag, &loop_port)), "Accumulated");
        }

        #[test]
        fn bind_value_origin_is_its_producer_not_the_bind_itself() {
            // `let x = 1 + 2` — bind.value is the transform output, so
            // origin_of(bind.value) walks through to the Transform producer
            // and reports Computed. A Bind's own output is only reached
            // when something references the Bind node directly.
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            let _ = dag.push_bind("x", body, Vec::new(), span());
            assert_eq!(label(&origin_of(&dag, &body)), "Computed");
        }
    }
}

/// Structural-resolution lens. The authority lives in
/// `src/v3/lenses/structural_resolution.dag`; the Rust projection is
/// auto-emitted into `src/v3/compiler/src/lens_structural_resolution_generated.rs`
/// and wrapped here as a module so callers use
/// `v3_compiler::lens_structural_resolution`. Editing the lens means
/// editing the `.dag` — there is no hand-written implementation on
/// this crate side.
///
/// Detects leaked `ArrowBody::Pending` in the final Dag.
/// Defense-in-depth regression pin for the R13 fix (see the `.dag`
/// source for the full detection rule and disposal trigger).
pub mod lens_structural_resolution {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_structural_resolution_generated.rs");
    }

    pub use generated::{check, name_keyed_references, NameKeyedReference, UnresolvedArrowBody};

    #[cfg(test)]
    mod tests {
        use super::{check, name_keyed_references, NameKeyedReference, UnresolvedArrowBody};
        use crate::dag::{ArrowBody, AtomPayload, Declaration, DeclarationId, TypeConnective};
        use crate::diagnostics::SourceSpan;
        use crate::{compile_to_dag, Dag};

        fn span() -> SourceSpan {
            SourceSpan::new("<lens-structural-resolution-test>", 0, 0)
        }

        fn inject_named_pending_arrow(
            dag: &mut Dag,
            name: &str,
            output_type: DeclarationId,
        ) -> DeclarationId {
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: Some(name.to_string()),
                connective: TypeConnective::Arrow {
                    inputs: Vec::new(),
                    output: output_type,
                    body: ArrowBody::Pending,
                },
                type_params: Vec::new(),
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: None,
                span: span(),
            });
            id
        }

        fn inject_anonymous_pending_arrow(
            dag: &mut Dag,
            output_type: DeclarationId,
        ) -> DeclarationId {
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Arrow {
                    inputs: Vec::new(),
                    output: output_type,
                    body: ArrowBody::Pending,
                },
                type_params: Vec::new(),
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: None,
                span: span(),
            });
            id
        }

        fn inject_name_keyed_reference(dag: &mut Dag, target: DeclarationId) -> DeclarationId {
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Atom(AtomPayload::ResolvedByName(target)),
                type_params: Vec::new(),
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: None,
                span: span(),
            });
            id
        }

        fn violations(dag: &Dag) -> Vec<UnresolvedArrowBody> {
            check(dag)
        }

        fn name_keyed(dag: &Dag) -> Vec<NameKeyedReference> {
            name_keyed_references(dag)
        }

        #[test]
        fn lens_flags_named_arrow_pending_injected_into_dag() {
            let mut dag = Dag::new();
            let int_output = dag.int_shape().expect("bootstrap Dag has Int").declaration;
            let decl_id = inject_named_pending_arrow(&mut dag, "leaked_fn", int_output);

            let found = violations(&dag);
            assert_eq!(
                found.len(),
                1,
                "expected exactly one violation, got {}: {:?}",
                found.len(),
                found
            );
            assert_eq!(found[0].declaration, decl_id);
            assert_eq!(found[0].name, "leaked_fn");
        }

        #[test]
        fn lens_silent_on_empty_bootstrap_dag() {
            let dag = Dag::new();
            let found = violations(&dag);
            assert!(
                found.is_empty(),
                "bootstrap Dag must produce zero violations (algebra arrows are anonymous), got {}: {:?}",
                found.len(),
                found
            );
        }

        #[test]
        fn lens_flags_anonymous_arrow_pending_injected_into_dag() {
            let mut dag = Dag::new();
            let int_output = dag.int_shape().expect("bootstrap Dag has Int").declaration;
            let decl_id = inject_anonymous_pending_arrow(&mut dag, int_output);

            let found = violations(&dag);
            assert_eq!(
                found.len(),
                1,
                "expected exactly one anonymous violation, got {}: {:?}",
                found.len(),
                found
            );
            assert_eq!(found[0].declaration, decl_id);
            assert_eq!(found[0].name, "<anonymous>");
        }

        #[test]
        fn lens_survives_co_existing_injected_and_compiled_declarations() {
            let mut dag =
                compile_to_dag("fn good(x: Int) -> Int = x + 1", "user.v3").expect("compiles");
            let int_output = dag.int_shape().expect("Int shape").declaration;
            let leak_id = inject_named_pending_arrow(&mut dag, "leaked", int_output);
            let found = violations(&dag);
            assert_eq!(
                found.len(),
                1,
                "expected exactly one violation amid real declarations, got {}: {:?}",
                found.len(),
                found
            );
            assert_eq!(found[0].declaration, leak_id);
            assert_eq!(found[0].name, "leaked");
        }

        #[test]
        fn lens_flags_injected_name_keyed_reference() {
            let mut dag = Dag::new();
            let int_id = dag.int_shape().expect("bootstrap Dag has Int").declaration;
            let site_id = inject_name_keyed_reference(&mut dag, int_id);

            let found = name_keyed(&dag);
            let injected = found
                .iter()
                .find(|entry| entry.declaration == site_id)
                .unwrap_or_else(|| {
                    panic!(
                        "expected injected site in name-keyed references, got {}: {:?}",
                        found.len(),
                        found
                    )
                });
            assert_eq!(injected.resolved_to, int_id);
        }
    }
}

mod bootstrap;

pub use bootstrap::BOOTSTRAP_FIXTURE_PATH_KEYS;

#[cfg(feature = "bootstrap-regen-fresh")]
mod bootstrap_regen_fresh;

/// Fresh tokenize/parse/lower bootstrap used **only** by `regen_bootstrap`
/// (`--features bootstrap-regen-fresh`). Default `v3-compiler` builds omit this
/// module; production callers load snapshots via `Dag::new()`.
#[cfg(feature = "bootstrap-regen-fresh")]
pub use bootstrap_regen_fresh::{
    compile_full_bootstrap_dag_from_std_seed,
    compile_full_bootstrap_without_parse_surface_dag_from_std_seed, compile_std_bootstrap_dag,
};

mod cost_basis_declaration;
mod dimension;
mod infer;

/// SG-4 prep: first .dag-authority slice of `infer.rs`. Authority
/// lives in `src/v3/lenses/infer_helpers.dag`; the Rust projection is
/// auto-emitted into `src/v3/compiler/src/infer_helpers_generated.rs`
/// and consumed by `infer.rs` via
/// `crate::infer_helpers::behavior_output_port`. Editing the helper
/// means editing the `.dag` — there is no hand-written implementation
/// on this crate side. SG-6 owns folding the standalone regen driver
/// and relocating extracted helper modules out of `lenses/` once the
/// consolidated regen target lands.
pub(crate) mod infer_helpers {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::SourceSpan;

        include!("infer_helpers_generated.rs");
    }

    pub(crate) use generated::{
        behavior_output_port, behavior_span, normalize_instantiation_arguments,
        payload_binding_span, push_template_argument_binding, resolve_template_argument_value,
        template_argument_value, template_arguments_match as generated_template_arguments_match,
        NormalizedInstantiationArgs, TemplateArgumentBinding, TemplateArgumentsMatch,
    };

    #[cfg(test)]
    pub(crate) use generated::filter_non_self_template_arguments;
}

/// SG-3g-d: `.dag`-authority `expr_span` / `item_span` for surface nodes plus
/// `pattern_binding_names` (see `lenses/lower_helpers.dag`).
/// Consumed from `lower.rs`; `parse_generated.rs` keeps its own `&SourceSpan` helper for
/// parser-local span fusion without cloning.
pub(crate) mod lower_helpers {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::diagnostics::SourceSpan;
        use crate::parse_surface;
        use crate::parse_surface::{SurfaceExpr, SurfaceItem, SurfacePattern};

        include!("lower_helpers_generated.rs");
    }

    pub(crate) use generated::{expr_span, item_span, pattern_binding_names};

    #[cfg(test)]
    mod tests {
        use super::{expr_span, item_span, pattern_binding_names};
        use crate::diagnostics::SourceSpan;
        use crate::parse_surface::{SurfaceExpr, SurfaceItem, SurfacePattern, SurfacePatternField};

        #[test]
        fn expr_span_matches_variant_span_field() {
            let span = SourceSpan::new("t.v3", 10, 20);
            let e = SurfaceExpr::Literal {
                value: crate::parse_surface::SurfaceLiteral::Int(1),
                span: span.clone(),
            };
            assert_eq!(expr_span(&e), span);
        }

        #[test]
        fn item_span_matches_item_shape() {
            let item_span_value = SourceSpan::new("t.v3", 30, 40);
            let expr_span_value = SourceSpan::new("t.v3", 50, 60);

            assert_eq!(
                item_span(&SurfaceItem::Let {
                    name: "x".into(),
                    type_ann: None,
                    expr: SurfaceExpr::Literal {
                        value: crate::parse_surface::SurfaceLiteral::Int(1),
                        span: expr_span_value.clone(),
                    },
                }),
                expr_span_value
            );
            assert_eq!(
                item_span(&SurfaceItem::Fn {
                    name: "f".into(),
                    type_params: vec![],
                    params: vec![],
                    return_type: crate::parse_surface::SurfaceType::Named {
                        name: "Int".into(),
                        span: item_span_value.clone(),
                    },
                    body: SurfaceExpr::Literal {
                        value: crate::parse_surface::SurfaceLiteral::Int(1),
                        span: expr_span_value,
                    },
                    span: item_span_value.clone(),
                }),
                item_span_value
            );
        }

        #[test]
        fn pattern_binding_names_match_pattern_shape() {
            let span = SourceSpan::new("t.v3", 10, 20);
            assert_eq!(
                pattern_binding_names(&SurfacePattern::BareVariant {
                    name: "None".into(),
                    span: span.clone(),
                }),
                Vec::<String>::new()
            );
            assert_eq!(
                pattern_binding_names(&SurfacePattern::VariantWith {
                    name: "Some".into(),
                    binding: "value".into(),
                    span: span.clone(),
                }),
                vec!["value".to_string()]
            );
            assert_eq!(
                pattern_binding_names(&SurfacePattern::VariantFields {
                    name: "Pair".into(),
                    fields: vec![
                        SurfacePatternField {
                            name: "left".into(),
                            binding: "x".into(),
                            span: span.clone(),
                        },
                        SurfacePatternField {
                            name: "right".into(),
                            binding: "y".into(),
                            span: span.clone(),
                        },
                    ],
                    span,
                }),
                vec!["x".to_string(), "y".to_string()]
            );
        }
    }
}

/// Back-compat module path for the Stage 2b idempotency lens.
///
/// The dedicated `lens_idempotency.rs` wrapper retired once the native-Dag
/// bridge collapsed to a single re-export. Keep the module name as an API alias
/// until callers move to the crate-root `analyze_workflow` export.
pub mod lens_idempotency {
    pub use crate::workflow_idempotency::analyze_workflow;
}
// Surface pipeline for this crate (not workspace-root `src/tokenize.rs` / `src/parse.rs`):
// `tokenize.dag` → `regen_tokenize` → `tokenize_generated.rs`,
// `parse_parser_body.txt` → `regen_parse` → `parse_generated.rs` (`parse` module),
// then hand-authored `lower.rs` consumes `parse::SurfaceItem` (including `inhabits`).
mod lower;
#[path = "parse_generated.rs"]
mod parse;
mod pipeline_authority;
mod regen_parse_emit;
mod regen_parse_tables_emit;
#[allow(
    dead_code,
    unused_imports,
    unused_parens,
    unused_variables,
    clippy::clone_on_copy,
    clippy::collapsible_else_if
)]
#[path = "tokenize_generated.rs"]
mod tokenize;

pub use regen_parse_emit::{render_parse_generated_rs, RenderParseGeneratedError};
pub use regen_parse_tables_emit::{
    render_parse_tables_generated_rs, RenderParseTablesGeneratedError,
};
pub(crate) mod variant_payload {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if,
        clippy::cmp_owned,
        clippy::large_enum_variant
    )]
    mod generated {
        use crate::dag::*;

        include!("variant_payload_generated.rs");
    }

    pub(crate) use generated::{
        variant_payload_shape, VariantPayloadShape, VariantPayloadShapeLookup,
    };
}
pub(crate) mod workflow_idempotency;
pub(crate) mod workflow_parallelism;

pub use cost_basis_declaration::{
    try_build_per_write_log_cost_basis_declaration, CostBasisDeclarationBuildError,
};
pub use dag::{Dag, NodeId};
pub use diagnostics::{Diagnostic, SourceSpan, LAYER1_DIAGNOSTIC_KIND_LABELS};
pub use emit::{EmitDispatchError, EmitMode, EmitTarget, EmittedSource};
pub use emit_rust::EmitError;
/// Lane 2 Stage 2b — supported public surface: [`analyze_workflow`] is the
/// primary entry; [`report_unsupported_workflow_variant`] and
/// [`lane2_workflow_idempotency_report`] are additionally exported so
/// `emit_rust_module(idempotency.dag)` output can link in rustc round-trip
/// tests. Composition helpers such as `compose_operation_effects` /
/// `operation_to_breaker` are **not** re-exported: naming and algebra authority
/// live in `src/v3/std/effects.dag`, and the Rust bridge must not become a
/// parallel public implementation surface beyond these std.effects mirrors.
pub use workflow_idempotency::analyze_workflow;
pub use workflow_idempotency::{
    lane2_workflow_idempotency_report, report_unsupported_workflow_variant,
};
/// Lane 2 Stage 2e — parallel composition safety (`ParallelEffect`); see DB-20.
pub use workflow_parallelism::analyze_parallelism;

/// Lane 2 Stage 2f — DB-3 dimension abstraction (`std/dimensions.dag` types;
/// `analyze_symbolic_cost_dimension` is the first migrated lens path).
pub use dimension::{
    analyze_complexity, analyze_symbolic_cost_dimension, behavior_spine_in_node_order,
    DimensionReport, Witness,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageSnapshotKind {
    Surface,
    Text,
    Dag,
}

#[derive(Debug, Clone)]
pub struct StageSnapshot {
    pub stage: String,
    pub kind: StageSnapshotKind,
    pub bytes: Vec<u8>,
    pub dag: Option<Dag>,
}

#[derive(Debug)]
pub enum StageSnapshotError {
    Compile(Box<CompileError>),
    Emit(Box<emit_rust::EmitError>),
    Pipeline(String),
}

#[derive(Debug)]
pub struct FixedPointMismatch {
    pub stage: String,
    pub detail: String,
}

/// Test-only hook: tokenize a source string. Used by the
/// `real_stdlib_parse_smoke` integration test to verify the parser
/// accepts production `dsl/std/*.dag` files before bootstrap migration.
#[doc(hidden)]
pub fn tokenize_for_test(source: &str, file: &str) -> Result<Vec<tokenize::Token>, Diagnostic> {
    tokenize::tokenize(source, file)
}

#[doc(hidden)]
pub fn token_is_kw_fn_for_test(token: &tokenize::Token) -> bool {
    matches!(token.kind, tokenize::TokenKind::KwFn)
}

#[doc(hidden)]
pub fn token_is_ident_for_test(token: &tokenize::Token, spelling: &str) -> bool {
    matches!(&token.kind, tokenize::TokenKind::Ident(s) if s == spelling)
}

/// Test-only hook: parse a token stream into a surface module.
#[doc(hidden)]
pub fn parse_for_test(
    tokens: &[tokenize::Token],
    file: &str,
) -> Result<parse::SurfaceModule, Diagnostic> {
    parse::parse(tokens, file)
}

/// Test-only hook: top-level `let` binding names in source order.
#[doc(hidden)]
pub fn surface_top_level_let_names_for_test(module: &parse::SurfaceModule) -> Vec<String> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            parse::SurfaceItem::Let { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// Test hook: pipeline stage identifiers in `compile { ... }` order in
/// `pipeline.dag` — the same ordering as `materialize_pipeline_realizations`.
#[doc(hidden)]
pub fn pipeline_compile_order_stage_names() -> Result<Vec<String>, String> {
    pipeline_authority::pipeline_compile_order_names(&dag::Dag::new())
}

/// Top-level compile failure. Distinguishes three structural
/// categories of failure by phase of the pipeline where they occurred.
///
/// **Dissolution receipt: TERMINAL.** Three variants, each with a
/// structurally distinct payload:
/// - `Tokenize(Diagnostic)`: tokenization produced a single diagnostic;
///   no Dag exists yet, so no Dag payload.
/// - `Parse(Diagnostic)`: parsing produced a single diagnostic; no Dag
///   exists yet.
/// - `Semantic(Dag)`: lowering/inference produced one or more
///   diagnostics; the Dag exists and carries them in its diagnostic
///   table, so it's handed back as the payload for caller inspection.
///
/// The three variants correspond to three structurally different
/// failure states (no-Dag-yet with a diagnostic vs Dag-with-
/// diagnostic-table). Pattern 2 (variant-is-data) fails because the
/// payloads are different types. Pattern 3 (algebraic-form) doesn't
/// apply — these are failure phases, not algebraic operations.
///
/// Guardrail G5: there is no `TypeError` variant. Type errors are
/// data on the Dag via the diagnostic table, not fields on the
/// error type. `Semantic(Dag)` is a handoff, not a classification of
/// what went wrong — the caller reads `dag.diagnostics()` for
/// specifics. This is what "fail-closed at the boundary" means in
/// practice: a successful compile returns `Ok(Dag)` with an empty
/// diagnostic table; a failed compile returns `Err(Semantic(Dag))`
/// with a non-empty one. There is no third outcome.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum CompileError {
    Tokenize(diagnostics::Diagnostic),
    Parse(diagnostics::Diagnostic),
    /// Semantic errors. The Dag is included so callers can inspect
    /// `dag.diagnostics()` to see what went wrong. `Err(Semantic(_))`
    /// means: the compile reached infer, some (>=1) diagnostics were
    /// produced, and the result is not usable.
    Semantic(Dag),
}

// `result_large_err`: clippy flags `Result<Dag, CompileError>`
// because `CompileError::Semantic(Dag)` carries a `Dag` payload
// (~264 bytes after the M1(3) PR-B-unwind R1 added the realization
// meta cache). Boxing the Dag would touch every pattern-match
// against `CompileError::Semantic` in the test suite, and the
// payload is on the cold failure path where the indirection would
// matter less than the API churn. Targeted `allow` on the function
// signature only — the rest of the crate keeps the lint enforced.
fn is_lenses_complexity_authority_module(module: &parse::SurfaceModule) -> bool {
    use crate::parse::SurfaceItem;
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::Module { path, .. }
                if path.len() >= 2 && path[0] == "lenses" && path[1] == "complexity"
        )
    })
}

fn needs_complexity_lens_authority_prepended(module: &parse::SurfaceModule) -> bool {
    use crate::parse::SurfaceItem;
    if is_lenses_complexity_authority_module(module) {
        return false;
    }
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::Import { path, .. }
                if path.len() >= 2 && path[0] == "lenses" && path[1] == "complexity"
        )
    })
}

#[allow(clippy::result_large_err)]
pub fn compile_to_dag(source: &str, file: &str) -> Result<Dag, CompileError> {
    let tokens = tokenize::tokenize(source, file).map_err(CompileError::Tokenize)?;
    let surface = parse::parse(&tokens, file).map_err(CompileError::Parse)?;
    let mut dag = if needs_complexity_lens_authority_prepended(&surface) {
        lower::lower_prepending_complexity_lens_authority(&surface)
    } else {
        lower::lower(&surface)
    };
    infer::infer(&mut dag);
    if dag.diagnostics().is_empty() {
        Ok(dag)
    } else {
        Err(CompileError::Semantic(dag))
    }
}

/// Structural witness for integer literal range narrowing: the
/// `IntegerAlgebra` and `TargetCarrier` variant payload [`dag::DeclarationId`]s
/// used to match rows in `rust_pilot_primitives`, derived from std
/// `OrderedRing<C>` / `Semiring<C>` by declaration identity (no template-name
/// string routing).
pub fn integer_literal_routing_witness(
    dag: &Dag,
    declaration: dag::DeclarationId,
) -> Option<(dag::DeclarationId, dag::DeclarationId)> {
    int_literal_ranges::integer_routing_witness_for_decl(dag, declaration)
        .map(|w| (w.algebra_variant_ty, w.carrier_variant_ty))
}

/// Lower `src/v3/std/parse_surface.dag` for codegen (`regen_parse`, SG-2 staging tests).
///
/// Unlike [`compile_to_dag`], this starts from a bootstrap Dag that omits the embedded
/// `parse_surface.dag` staged fixture so the fresh parse is first-of-name and can be
/// lowered without duplicate-declaration diagnostics.
#[allow(clippy::result_large_err)]
fn compile_onto_parse_surface_free_bootstrap(
    source: &str,
    file: &str,
) -> Result<Dag, CompileError> {
    let tokens = tokenize::tokenize(source, file).map_err(CompileError::Tokenize)?;
    let surface = parse::parse(&tokens, file).map_err(CompileError::Parse)?;
    let mut dag = Dag::new_without_parse_surface_staged_fixture_bootstrap();
    let user_start = dag.declarations().len();
    lower::lower_into(&mut dag, &surface);
    lower::finalize_strict_user_lower_range(&mut dag, user_start);
    infer::infer(&mut dag);
    if dag.diagnostics().is_empty() {
        Ok(dag)
    } else {
        Err(CompileError::Semantic(dag))
    }
}

#[allow(clippy::result_large_err)]
pub fn compile_parse_surface_std_authority_dag(
    source: &str,
    file: &str,
) -> Result<Dag, CompileError> {
    compile_onto_parse_surface_free_bootstrap(source, file)
}

/// PB-1-a generated snapshot helper: load the committed std-fixture
/// bootstrap snapshot without re-running tokenize/parse/lower.
pub fn generated_std_bootstrap_dag() -> Dag {
    Dag::std_fixture_bootstrap_snapshot()
}

pub fn generated_full_bootstrap_dag() -> Dag {
    Dag::new()
}

pub fn generated_full_bootstrap_without_parse_surface_dag() -> Dag {
    Dag::new_without_parse_surface_staged_fixture_bootstrap()
}

pub fn default_fixed_point_source() -> &'static str {
    "let x: Int = 1 + 2\nlet y: Int = x + 3\n"
}

pub fn compile_stage_snapshots(
    source: &str,
    file: &str,
) -> Result<Vec<StageSnapshot>, StageSnapshotError> {
    let pipeline_dag = Dag::new();
    if !pipeline_dag.diagnostics().is_empty() {
        return Err(StageSnapshotError::Compile(Box::new(
            CompileError::Semantic(pipeline_dag),
        )));
    }
    let pipeline = pipeline_authority::ordered_pipeline_stages(&pipeline_dag)
        .map_err(StageSnapshotError::Pipeline)?;

    let tokens = tokenize::tokenize(source, file)
        .map_err(CompileError::Tokenize)
        .map_err(|error| StageSnapshotError::Compile(Box::new(error)))?;
    let surface = parse::parse(&tokens, file)
        .map_err(CompileError::Parse)
        .map_err(|error| StageSnapshotError::Compile(Box::new(error)))?;
    let parse_bytes = format!("{surface:#?}").into_bytes();

    let mut lower_dag = lower::lower(&surface);
    let lower_snapshot = lower_dag.clone();
    let lower_bytes = serialize::serialize_dag(&lower_snapshot);

    infer::infer(&mut lower_dag);
    if !lower_dag.diagnostics().is_empty() {
        return Err(StageSnapshotError::Compile(Box::new(
            CompileError::Semantic(lower_dag.clone()),
        )));
    }

    let infer_snapshot = lower_dag.clone();
    let infer_bytes = serialize::serialize_dag(&infer_snapshot);
    let emitted = emit::emit(&lower_dag, EmitTarget::Rust)
        .map(|source| source.text)
        .map_err(|error| match error {
            emit::EmitDispatchError::Core(error) => StageSnapshotError::Emit(Box::new(error)),
            emit::EmitDispatchError::Python(_) => {
                unreachable!("EmitTarget::Rust cannot yield a Python emission error")
            }
        })?;

    let mut snapshots = Vec::with_capacity(pipeline.len());
    for stage in pipeline {
        let (kind, bytes, dag) = match stage.stage_name.as_str() {
            "parse" => (StageSnapshotKind::Surface, parse_bytes.clone(), None),
            "lower" => (
                StageSnapshotKind::Dag,
                lower_bytes.clone(),
                Some(lower_snapshot.clone()),
            ),
            "infer" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "compute_ownership" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "lens_complexity" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "emit" => (StageSnapshotKind::Text, emitted.clone().into_bytes(), None),
            other => {
                return Err(StageSnapshotError::Pipeline(format!(
                    "pipeline stage `{other}` has no Rust snapshot implementation"
                )));
            }
        };

        if !snapshot_kind_matches(stage.snapshot_kind, kind) {
            return Err(StageSnapshotError::Pipeline(format!(
                "pipeline stage `{}` declares snapshot kind {:?} but Rust produced {:?}",
                stage.stage_name, stage.snapshot_kind, kind
            )));
        }

        snapshots.push(StageSnapshot {
            stage: stage.stage_name,
            kind,
            bytes,
            dag,
        });
    }

    Ok(snapshots)
}

pub fn compare_stage_snapshots(
    lhs: &[StageSnapshot],
    rhs: &[StageSnapshot],
) -> Result<(), FixedPointMismatch> {
    if lhs.len() != rhs.len() {
        return Err(FixedPointMismatch {
            stage: "pipeline".to_string(),
            detail: format!(
                "stage count mismatch: pass1 has {}, pass2 has {}",
                lhs.len(),
                rhs.len()
            ),
        });
    }

    for (left, right) in lhs.iter().zip(rhs.iter()) {
        if left.stage != right.stage {
            return Err(FixedPointMismatch {
                stage: "pipeline".to_string(),
                detail: format!(
                    "stage order mismatch: pass1 has `{}`, pass2 has `{}`",
                    left.stage, right.stage
                ),
            });
        }
        if left.kind != right.kind {
            return Err(FixedPointMismatch {
                stage: left.stage.clone(),
                detail: format!(
                    "snapshot kind mismatch at stage `{}`: pass1={:?}, pass2={:?}",
                    left.stage, left.kind, right.kind
                ),
            });
        }
        if left.bytes == right.bytes {
            continue;
        }

        let detail = match (&left.dag, &right.dag) {
            (Some(lhs_dag), Some(rhs_dag)) => serialize::first_difference(lhs_dag, rhs_dag)
                .map(|diff| diff.detail)
                .unwrap_or_else(|| first_differing_line(&left.bytes, &right.bytes)),
            _ => first_differing_line(&left.bytes, &right.bytes),
        };
        return Err(FixedPointMismatch {
            stage: left.stage.clone(),
            detail,
        });
    }

    Ok(())
}

fn snapshot_kind_matches(
    declared: pipeline_authority::PipelineSnapshotKind,
    actual: StageSnapshotKind,
) -> bool {
    matches!(
        (declared, actual),
        (
            pipeline_authority::PipelineSnapshotKind::Surface,
            StageSnapshotKind::Surface
        ) | (
            pipeline_authority::PipelineSnapshotKind::Dag,
            StageSnapshotKind::Dag
        ) | (
            pipeline_authority::PipelineSnapshotKind::Text,
            StageSnapshotKind::Text
        )
    )
}

fn first_differing_line(lhs: &[u8], rhs: &[u8]) -> String {
    let lhs = String::from_utf8_lossy(lhs);
    let rhs = String::from_utf8_lossy(rhs);
    for (idx, (left, right)) in lhs.lines().zip(rhs.lines()).enumerate() {
        if left != right {
            return format!(
                "first differing line {}: pass1=`{}`, pass2=`{}`",
                idx + 1,
                left,
                right
            );
        }
    }
    format!(
        "snapshot byte-length mismatch: pass1={} bytes, pass2={} bytes",
        lhs.len(),
        rhs.len()
    )
}
#[cfg(test)]
mod tokenize_ascii_parity_tests {
    use super::tokenize::{byte_matches, ScannerCharClass};

    #[test]
    fn ascii_byte_class_predicates_match_std_unicode_ascii_boundary() {
        for byte in 0u8..=127 {
            assert_eq!(
                byte_matches(byte, ScannerCharClass::Whitespace),
                byte.is_ascii_whitespace(),
                "tokenizer whitespace predicate diverged at byte {byte:#04x}"
            );

            assert_eq!(
                byte_matches(byte, ScannerCharClass::Digit),
                byte.is_ascii_digit(),
                "tokenizer digit predicate diverged at byte {byte:#04x}"
            );

            assert_eq!(
                byte_matches(byte, ScannerCharClass::IdentStart),
                (byte.is_ascii_alphabetic() || byte == b'_'),
                "tokenizer ident-start predicate diverged at byte {byte:#04x}"
            );

            assert_eq!(
                byte_matches(byte, ScannerCharClass::IdentContinue),
                (byte.is_ascii_alphanumeric() || byte == b'_'),
                "tokenizer ident-continue predicate diverged at byte {byte:#04x}"
            );
        }
    }
}

/// Signed decimal literals (`-` immediately followed by ASCII digits) for
/// Coercion-Fold interval lowers (`src/v3/spec/rust.dag` Slice B rows).
#[cfg(test)]
mod signed_decimal_int_literal_tests {
    use super::tokenize::{tokenize, TokenKind};

    #[test]
    fn minus_digits_lexes_as_single_int_lit_i32_min() {
        let tokens =
            tokenize("-2147483648", "signed_decimal_int_literal_tests.v3").expect("tokenize");
        assert!(
            matches!(tokens.first().map(|t| &t.kind), Some(TokenKind::IntLit(n)) if *n == -2147483648),
            "expected `-2147483648` as one token; got {:?}",
            tokens.first().map(|t| &t.kind)
        );
        assert!(
            matches!(tokens.get(1).map(|t| &t.kind), Some(TokenKind::Eof)),
            "expected EOF after literal; got {:?}",
            tokens.get(1).map(|t| &t.kind)
        );
    }

    #[test]
    fn minus_digits_lexes_as_single_int_lit_i64_min() {
        let tokens =
            tokenize("-9223372036854775808", "signed_decimal_i64_min.v3").expect("tokenize");
        assert!(
            matches!(tokens.first().map(|t| &t.kind), Some(TokenKind::IntLit(n)) if *n == i64::MIN),
            "expected i64::MIN as one token; got {:?}",
            tokens.first().map(|t| &t.kind)
        );
        assert!(
            matches!(tokens.get(1).map(|t| &t.kind), Some(TokenKind::Eof)),
            "expected EOF after literal; got {:?}",
            tokens.get(1).map(|t| &t.kind)
        );
    }

    #[test]
    fn infix_minus_without_whitespace_stays_binary_minus() {
        let tokens = tokenize("1-1", "infix_minus.v3").expect("tokenize");
        assert!(tokens.len() >= 4, "expected literal, minus, literal, EOF");
        assert!(matches!(&tokens[0].kind, TokenKind::IntLit(1)));
        assert!(matches!(&tokens[1].kind, TokenKind::Minus));
        assert!(matches!(&tokens[2].kind, TokenKind::IntLit(1)));
        assert!(matches!(&tokens[3].kind, TokenKind::Eof));
    }

    #[test]
    fn ident_minus_digit_without_whitespace_stays_binary_minus() {
        let tokens = tokenize("x-1", "ident_minus.v3").expect("tokenize");
        assert!(tokens.len() >= 4, "expected ident, minus, literal, EOF");
        assert!(matches!(
            &tokens[0].kind,
            TokenKind::Ident(x) if x == "x"
        ));
        assert!(matches!(&tokens[1].kind, TokenKind::Minus));
        assert!(matches!(&tokens[2].kind, TokenKind::IntLit(1)));
        assert!(matches!(&tokens[3].kind, TokenKind::Eof));
    }

    #[test]
    fn unary_minus_after_eq_still_merges_digits() {
        let tokens = tokenize("=-42", "eq_unary.v3").expect("tokenize");
        assert!(tokens.len() >= 3, "expected Eq, signed literal, EOF");
        assert!(matches!(&tokens[0].kind, TokenKind::Eq));
        assert!(matches!(&tokens[1].kind, TokenKind::IntLit(-42)));
        assert!(matches!(&tokens[2].kind, TokenKind::Eof));
    }
}

/// PB-Runtime / flat-namespace: L1 behavior marker `ValueBehavior` frees bare
/// `Value` for runtime union carriers (`type Value = …` in user or evaluator DAG).
#[cfg(test)]
mod value_behavior_marker_tests {
    use crate::dag::Dag;
    use crate::{infer, lower, parse, tokenize};

    #[test]
    fn runtime_value_decl_coexists_with_value_behavior_marker() {
        let mut dag = Dag::new();
        let marker_id = dag.value_marker().expect("ValueBehavior marker cached");
        let marker_decl = dag
            .declaration_by_name("ValueBehavior")
            .expect("ValueBehavior declaration present");
        assert_eq!(marker_id, marker_decl.id);
        if let Some(runtime_value) = dag.declaration_by_name("Value") {
            assert_ne!(
                runtime_value.id, marker_id,
                "bootstrap bare `Value` must not alias ValueBehavior marker"
            );
            assert_eq!(
                runtime_value.span.file, "src/v3/std/runtime.dag",
                "bootstrap bare `Value`, when present, must be the runtime carrier"
            );
        }

        let source = "module test.pb_runtime_value_name_smoke\n\ntype UserValue {}\n";
        let file = "pb_runtime_value_name_smoke.v3";
        let tokens = tokenize::tokenize(source, file).expect("tokenize");
        let surface = parse::parse(&tokens, file).expect("parse");
        let user_start = dag.declarations().len();
        lower::lower_into(&mut dag, &surface);
        lower::finalize_strict_user_lower_range(&mut dag, user_start);
        infer::infer(&mut dag);
        assert!(
            dag.diagnostics().is_empty(),
            "user non-conflicting type should compile onto bootstrap; diagnostics: {:?}",
            dag.diagnostics()
        );
        assert_eq!(
            dag.value_marker(),
            Some(marker_id),
            "marker cache must still resolve ValueBehavior after user declarations land"
        );
    }
}
