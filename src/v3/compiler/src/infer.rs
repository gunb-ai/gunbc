// Type inference: fills Port.value_type by propagating types
// forward through the DAG.
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
// through Dag::mark_unresolved. There are no silent None returns,
// no panics, no ambiguous states. Either the port settles to a
// Some(type) OR the diagnostic table has an entry for it. The
// structural invariant test in tests/m0_acceptance.rs audits the
// biconditional across every DAG the M0 suite builds.
//
// Fixpoint loop: the DAG is topologically ordered, but inference
// inside a recursive function body depends on the enclosing
// function's declared return type. We iterate until no port types
// change to make sure transitively-dependent ports settle.
//
// G5: never return Result<_, TypeError>, never throw.

use crate::dag::{Behavior, Dag, FunctionRef, LiteralValue, NodeId, PortId};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::types::{Prim, TypeShape};

pub fn infer(dag: &mut Dag) {
    loop {
        let mut changed = false;
        let node_count = dag.nodes().len();
        for i in 0..node_count {
            match decide(dag, i) {
                Decision::Set(port, ty) => {
                    if dag.diagnostics().contains(port) {
                        continue;
                    }
                    let current = dag.port(port).value_type().cloned();
                    match current {
                        None => {
                            dag.set_port_type(port, ty);
                            changed = true;
                        }
                        Some(existing) if existing == ty => {
                            // Declared and inferred types agree.
                        }
                        Some(existing) => {
                            let span = dag
                                .annotation_span(port)
                                .cloned()
                                .unwrap_or_else(|| {
                                    node_span_for_port(dag, port)
                                        .unwrap_or_else(synthetic_span)
                                });
                            let diag = Diagnostic::TypeMismatch {
                                expected: existing,
                                actual: ty,
                                span,
                            };
                            dag.mark_unresolved(port, diag);
                            changed = true;
                        }
                    }
                }
                Decision::Fail(port, diag) => {
                    if !dag.diagnostics().contains(port) {
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
            let Some(sig) = lookup_signature(dag, &t.target) else {
                return Decision::Fail(
                    t.output,
                    Diagnostic::ResolveError {
                        name: t.target.name.clone(),
                        span: node_span(dag, t.id).unwrap_or_else(synthetic_span),
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
                        span: node_span(dag, t.id).unwrap_or_else(synthetic_span),
                    },
                );
            }
            for (input_port, expected_ty) in t.inputs.iter().zip(sig.params.iter()) {
                match dag.port(*input_port).value_type() {
                    None => {
                        if dag.diagnostics().contains(*input_port) {
                            // An upstream failure already marked this
                            // input. Propagate the unresolved state
                            // to our output with a fresh diagnostic
                            // so the invariant holds here too.
                            return Decision::Fail(
                                t.output,
                                Diagnostic::ResolveError {
                                    name: format!("(upstream failure in {})", t.target.name),
                                    span: node_span(dag, t.id)
                                        .unwrap_or_else(synthetic_span),
                                },
                            );
                        }
                        return Decision::Retry;
                    }
                    Some(actual) if actual == expected_ty => {}
                    Some(actual) => {
                        return Decision::Fail(
                            t.output,
                            Diagnostic::TypeMismatch {
                                expected: expected_ty.clone(),
                                actual: actual.clone(),
                                span: node_span(dag, t.id).unwrap_or_else(synthetic_span),
                            },
                        );
                    }
                }
            }
            Decision::Set(t.output, sig.return_type)
        }
        Behavior::Branch(b) => {
            let mut iter = b.paths.iter();
            let Some(first_path) = iter.next() else {
                return Decision::Retry;
            };
            let first_type = match dag.port(first_path.output).value_type() {
                None if dag.diagnostics().contains(first_path.output) => {
                    return Decision::Fail(
                        b.output,
                        Diagnostic::ResolveError {
                            name: "(upstream failure in branch path)".to_string(),
                            span: node_span(dag, b.id).unwrap_or_else(synthetic_span),
                        },
                    );
                }
                None => return Decision::Retry,
                Some(t) => t.clone(),
            };
            for path in iter {
                match dag.port(path.output).value_type() {
                    None if dag.diagnostics().contains(path.output) => {
                        return Decision::Fail(
                            b.output,
                            Diagnostic::ResolveError {
                                name: "(upstream failure in branch path)".to_string(),
                                span: node_span(dag, b.id).unwrap_or_else(synthetic_span),
                            },
                        );
                    }
                    None => return Decision::Retry,
                    Some(other) if *other == first_type => {}
                    Some(other) => {
                        return Decision::Fail(
                            b.output,
                            Diagnostic::TypeMismatch {
                                expected: first_type.clone(),
                                actual: other.clone(),
                                span: node_span(dag, b.id).unwrap_or_else(synthetic_span),
                            },
                        );
                    }
                }
            }
            Decision::Set(b.output, first_type)
        }
        Behavior::Loop(_) | Behavior::Bind(_) => Decision::Retry,
    }
}

fn node_span(dag: &Dag, node: NodeId) -> Option<SourceSpan> {
    dag.node_span(node).cloned()
}

fn node_span_for_port(dag: &Dag, port: PortId) -> Option<SourceSpan> {
    dag.port(port)
        .produced_by
        .and_then(|node_id| dag.node_span(node_id).cloned())
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
