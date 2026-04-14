// Type inference: fills Port.state by propagating types forward
// through the DAG.
//
// M0 scope:
//   - ValueNode(literal)  -> output port = literal type
//   - TransformNode       -> output type from target function's
//                            signature (user signature registry OR
//                            primitive table). Arity-mismatch and
//                            unknown-target are fail-closed.
//   - BranchNode          -> output type = unified path outputs.
//                            Path-disagreement is fail-closed.
//   - LoopNode            -> output type pre-set during lowering
//                            from declared return type; infer
//                            leaves it alone.
//   - BindNode            -> no output port to fill; the bound name
//                            reuses its value port via the scope
//                            map set during lowering.
//
// Fail-closed (invariant C-8): every detectable problem goes
// through Dag::mark_unresolved. There are no silent skips, no
// panics, no ambiguous states. After inference completes, a
// post-sweep drives any remaining Uninferred ports to Unresolved
// with a generic diagnostic so the biconditional
//   state == Unresolved  iff  diagnostics.contains(port_id)
// and the stronger post-invariant
//   state != Uninferred after infer completes
// both hold.
//
// Fixpoint loop: the DAG is topologically ordered, but inference
// inside a recursive function body depends on the enclosing
// function's declared return type. We iterate until no port types
// change to make sure transitively-dependent ports settle.
//
// G5: never return Result<_, TypeError>, never throw.

use crate::dag::{Behavior, Dag, FunctionRef, LiteralValue, PortId, PortState};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::types::{Prim, TypeShape};

pub fn infer(dag: &mut Dag) {
    loop {
        let mut changed = false;
        let node_count = dag.nodes().len();
        for i in 0..node_count {
            match decide(dag, i) {
                Decision::Set(port, ty) => {
                    if matches!(dag.port(port).state(), PortState::Unresolved) {
                        continue;
                    }
                    match dag.port(port).state().clone() {
                        PortState::Uninferred => {
                            dag.set_port_type(port, ty);
                            changed = true;
                        }
                        PortState::Resolved(existing) if existing == ty => {
                            // Declared and inferred types agree.
                        }
                        PortState::Resolved(existing) => {
                            // Declared annotation conflicts with
                            // inferred type. Point the diagnostic at
                            // the node that produced the conflict.
                            let span = node_span_for_port(dag, port)
                                .unwrap_or_else(synthetic_span);
                            let diag = Diagnostic::TypeMismatch {
                                expected: existing,
                                actual: ty,
                                span,
                            };
                            dag.mark_unresolved(port, diag);
                            changed = true;
                        }
                        PortState::Unresolved => unreachable!("guarded above"),
                    }
                }
                Decision::Fail(port, diag) => {
                    if !matches!(dag.port(port).state(), PortState::Unresolved) {
                        dag.mark_unresolved(port, diag);
                        changed = true;
                    }
                }
                Decision::Retry => {}
            }
        }
        if !changed {
            break;
        }
    }

    // Post-sweep: if the fixpoint stalled with any port still
    // Uninferred, mark it Unresolved with a catch-all diagnostic.
    // This preserves the post-invariant that inference terminates
    // with every port in Resolved or Unresolved state. Reaching
    // this path is a bug (means decide() missed a case), but the
    // invariant audit will catch it loudly instead of silently
    // leaking a half-typed DAG to downstream lenses.
    let uninferred_ports: Vec<PortId> = dag
        .all_ports()
        .filter(|p| matches!(p.state(), PortState::Uninferred))
        .map(|p| p.id())
        .collect();
    for port in uninferred_ports {
        let span =
            node_span_for_port(dag, port).unwrap_or_else(synthetic_span);
        dag.mark_unresolved(
            port,
            Diagnostic::ResolveError {
                name: "(inference did not resolve this port)".to_string(),
                span,
            },
        );
    }
}

enum Decision {
    /// Inputs ready and compatible; set the output port's type.
    Set(PortId, TypeShape),
    /// Definitive failure; mark the output port unresolved.
    Fail(PortId, Diagnostic),
    /// Inputs not yet typed; try again next fixpoint iteration.
    Retry,
}

fn decide(dag: &Dag, index: usize) -> Decision {
    match &dag.nodes()[index] {
        Behavior::Value(v) => {
            let ty = match &v.data {
                LiteralValue::Int(_) => TypeShape::Primitive(Prim::Int),
                LiteralValue::Bool(_) => TypeShape::Primitive(Prim::Bool),
                LiteralValue::String(_) => TypeShape::Primitive(Prim::String),
            };
            Decision::Set(v.output, ty)
        }
        Behavior::Transform(t) => {
            // User-function Bind-state check: if the target is a
            // user function, verify that its Bind.value port is in
            // Resolved state. An Unresolved function means the body
            // conflicted with the declared signature (caught by the
            // apply-level conflict check on the function's return
            // port), and the declared signature is no longer
            // trustworthy. An Uninferred function means the body
            // hasn't been processed yet this iteration; Retry.
            //
            // This is the producer-fact-reaches-consumer path: the
            // body-validity fact lives on the function's Bind port,
            // and call sites consult it before trusting the
            // registered signature. Without this check, a function
            // with a body/signature mismatch silently propagates
            // its declared return type to all call sites.
            if dag.signature(&t.target.name).is_some() {
                if let Some(bind) = dag.lookup_function(&t.target.name) {
                    match dag.port(bind.value).state() {
                        PortState::Uninferred => return Decision::Retry,
                        PortState::Unresolved => {
                            return Decision::Fail(
                                t.output,
                                Diagnostic::ResolveError {
                                    name: format!(
                                        "function `{}` has an invalid body",
                                        t.target.name
                                    ),
                                    span: t.span.clone(),
                                },
                            );
                        }
                        PortState::Resolved(_) => {}
                    }
                }
            }

            let Some(sig) = lookup_signature(dag, &t.target) else {
                return Decision::Fail(
                    t.output,
                    Diagnostic::ResolveError {
                        name: t.target.name.clone(),
                        span: t.span.clone(),
                    },
                );
            };
            if sig.params.len() != t.inputs.len() {
                return Decision::Fail(
                    t.output,
                    Diagnostic::ArityMismatch {
                        function: t.target.name.clone(),
                        expected: sig.params.len(),
                        actual: t.inputs.len(),
                        span: t.span.clone(),
                    },
                );
            }
            for (input_port, expected_ty) in t.inputs.iter().zip(sig.params.iter()) {
                match dag.port(*input_port).state() {
                    PortState::Uninferred => return Decision::Retry,
                    PortState::Unresolved => {
                        return Decision::Fail(
                            t.output,
                            Diagnostic::ResolveError {
                                name: format!("(upstream failure in {})", t.target.name),
                                span: t.span.clone(),
                            },
                        );
                    }
                    PortState::Resolved(actual) if actual == expected_ty => {}
                    PortState::Resolved(actual) => {
                        return Decision::Fail(
                            t.output,
                            Diagnostic::TypeMismatch {
                                expected: expected_ty.clone(),
                                actual: actual.clone(),
                                span: t.span.clone(),
                            },
                        );
                    }
                }
            }
            Decision::Set(t.output, sig.return_type)
        }
        Behavior::Branch(b) => {
            // Branch input must be Bool — the condition selects
            // which path fires. Not checking this was the embarrassing
            // miss from the M0.1-M0.6 review: `if 1 then 2 else 3`
            // typed as Int with no diagnostic.
            let bool_ty = TypeShape::Primitive(Prim::Bool);
            match dag.port(b.input).state() {
                PortState::Uninferred => return Decision::Retry,
                PortState::Unresolved => {
                    return Decision::Fail(
                        b.output,
                        Diagnostic::ResolveError {
                            name: "(upstream failure in branch condition)".to_string(),
                            span: b.span.clone(),
                        },
                    );
                }
                PortState::Resolved(ty) if *ty == bool_ty => {}
                PortState::Resolved(ty) => {
                    return Decision::Fail(
                        b.output,
                        Diagnostic::TypeMismatch {
                            expected: bool_ty,
                            actual: ty.clone(),
                            span: b.span.clone(),
                        },
                    );
                }
            }

            let mut iter = b.paths.iter();
            let Some(first_path) = iter.next() else {
                return Decision::Retry;
            };
            let first_type = match dag.port(first_path.output).state() {
                PortState::Uninferred => return Decision::Retry,
                PortState::Unresolved => {
                    return Decision::Fail(
                        b.output,
                        Diagnostic::ResolveError {
                            name: "(upstream failure in branch path)".to_string(),
                            span: b.span.clone(),
                        },
                    );
                }
                PortState::Resolved(t) => t.clone(),
            };
            for path in iter {
                match dag.port(path.output).state() {
                    PortState::Uninferred => return Decision::Retry,
                    PortState::Unresolved => {
                        return Decision::Fail(
                            b.output,
                            Diagnostic::ResolveError {
                                name: "(upstream failure in branch path)".to_string(),
                                span: b.span.clone(),
                            },
                        );
                    }
                    PortState::Resolved(other) if *other == first_type => {}
                    PortState::Resolved(other) => {
                        return Decision::Fail(
                            b.output,
                            Diagnostic::TypeMismatch {
                                expected: first_type.clone(),
                                actual: other.clone(),
                                span: b.span.clone(),
                            },
                        );
                    }
                }
            }
            Decision::Set(b.output, first_type)
        }
        Behavior::Loop(l) => {
            // M0.10: reconcile the body's actual return type with
            // the loop's output (which was pre-seeded during
            // lowering from the declared return type). The pre-seed
            // is load-bearing for the fixpoint — recursive calls
            // inside the body look up the function's Bind.value
            // port (which is loop.output) to determine their own
            // return type, so loop.output must be Resolved before
            // the body can settle. But without a reconciliation
            // step, loop.output stays at the pre-seeded declared
            // type even if the body computes something different.
            //
            // Fix: after the body settles, Set loop.output to the
            // body's actual return type. If that type matches the
            // pre-seeded declared type, the apply loop is a no-op.
            // If it doesn't match, the apply loop's conflict
            // detection marks loop.output Unresolved with a
            // TypeMismatch diagnostic pointing at the Loop's span.
            //
            // Known deferred: the self-referential Transform nodes
            // inside the Loop body (the `bad(n - 1)` calls in the
            // body) are still literal function-call nodes, not a
            // Recur primitive. The Loop wrapper is currently a
            // structural annotation that the substrate doesn't
            // interpret specially beyond this reconciliation. A
            // full "recursion rewrites during lowering" fix would
            // replace the self-calls with a Recur node, making the
            // Loop genuinely terminal for iteration. That's M1+
            // work — the current fix addresses the producer-fact-
            // reaches-consumer hole without restructuring the
            // lowering.
            if l.body == l.id {
                // Degenerate case: body_root was None during
                // lowering (placeholder body), so loop_body_node
                // defaulted to loop_id. No meaningful reconciliation
                // — body is already broken upstream.
                return Decision::Retry;
            }
            let body_node = dag.node(l.body);
            let body_output = behavior_output_port(body_node);
            match dag.port(body_output).state() {
                PortState::Uninferred => Decision::Retry,
                PortState::Unresolved => Decision::Fail(
                    l.output,
                    Diagnostic::ResolveError {
                        name: "function body is unresolved".to_string(),
                        span: l.span.clone(),
                    },
                ),
                PortState::Resolved(body_ty) => {
                    Decision::Set(l.output, body_ty.clone())
                }
            }
        }
        Behavior::Bind(_) => Decision::Retry,
    }
}

/// Return the "output" port of a behavior node — the port that
/// carries the value this node produces. For Bind, this is the
/// value port the Bind aliases, not a separately-owned port.
fn behavior_output_port(node: &Behavior) -> PortId {
    match node {
        Behavior::Value(v) => v.output,
        Behavior::Transform(t) => t.output,
        Behavior::Branch(b) => b.output,
        Behavior::Loop(l) => l.output,
        Behavior::Bind(b) => b.value,
    }
}

fn node_span_for_port(dag: &Dag, port: PortId) -> Option<SourceSpan> {
    dag.port(port)
        .produced_by
        .map(|node_id| dag.node(node_id).span().clone())
}

fn synthetic_span() -> SourceSpan {
    SourceSpan::new("<inferred>", 0, 0)
}

/// Resolve a FunctionRef to a signature:
///   1. User function registry (populated during lowering)
///   2. Hardcoded primitive table (M0 placeholder for std/ algebra
///      declarations — migrates in M1)
fn lookup_signature(dag: &Dag, target: &FunctionRef) -> Option<ResolvedSignature> {
    if let Some(sig) = dag.signature(&target.name) {
        return Some(ResolvedSignature {
            params: sig.params.clone(),
            return_type: sig.return_type.clone(),
        });
    }
    primitive_signature(&target.name)
}

struct ResolvedSignature {
    params: Vec<TypeShape>,
    return_type: TypeShape,
}

fn primitive_signature(name: &str) -> Option<ResolvedSignature> {
    let int = || TypeShape::Primitive(Prim::Int);
    let bool_ty = || TypeShape::Primitive(Prim::Bool);
    match name {
        "std::int::add" | "std::int::sub" | "std::int::mul" | "std::int::div" => {
            Some(ResolvedSignature {
                params: vec![int(), int()],
                return_type: int(),
            })
        }
        "std::int::eq" | "std::int::ne" | "std::int::lt" | "std::int::le"
        | "std::int::gt" | "std::int::ge" => Some(ResolvedSignature {
            params: vec![int(), int()],
            return_type: bool_ty(),
        }),
        _ => None,
    }
}
