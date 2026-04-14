// Type inference: fills Port.state by propagating types forward through
// the DAG. Dispatches on TypeConnective (M1_DESIGN.md §4) instead of the
// old name-based function signature lookup.
//
// Transform.target is a DeclarationId. Inference reads the target's
// Declaration and:
//   - Arrow { inputs, output, body }: direct signature from the input and
//     output declarations, converted to port-level TypeShape via the
//     `declaration_to_type_shape` bridge.
//   - Atom(Identifier { name, resolved }): follow the resolved link if set,
//     otherwise look up the name in the Dag's declaration table
//     (§8.9 inhabitance walk, reduced at M1(2.5) to direct primitive lookup
//     pending the full algebra bootstrap in Phase 5).
//   - Anything else: not callable.
//
// Fail-closed (invariant C-8): every detectable problem goes through
// Dag::mark_unresolved. After inference, a post-sweep drives any remaining
// Uninferred ports to Unresolved with a generic diagnostic so the
// biconditional
//   state == Unresolved  iff  diagnostics.contains(port_id)
// and the post-invariant
//   state != Uninferred after infer completes
// both hold.

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, Dag, DeclarationId, LiteralBits, PortId, PortState,
    TransformNode, TypeConnective,
};
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
                        PortState::Resolved(existing) if existing == ty => {}
                        PortState::Resolved(existing) => {
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

    // Post-sweep: any port still Uninferred means decide() missed a case.
    let uninferred_ports: Vec<PortId> = dag
        .all_ports()
        .filter(|p| matches!(p.state(), PortState::Uninferred))
        .map(|p| p.id())
        .collect();
    for port in uninferred_ports {
        let span = node_span_for_port(dag, port).unwrap_or_else(synthetic_span);
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
    Set(PortId, TypeShape),
    Fail(PortId, Diagnostic),
    Retry,
}

fn decide(dag: &Dag, index: usize) -> Decision {
    match &dag.nodes()[index] {
        Behavior::Value(v) => {
            let ty = match &v.data {
                LiteralBits::Int(_) => TypeShape::Primitive(Prim::Int),
                LiteralBits::Bool(_) => TypeShape::Primitive(Prim::Bool),
                LiteralBits::String(_) => TypeShape::Primitive(Prim::String),
            };
            Decision::Set(v.output, ty)
        }
        Behavior::Transform(t) => decide_transform(dag, t),
        Behavior::Branch(b) => {
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
            if l.body == l.id {
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
                PortState::Resolved(body_ty) => Decision::Set(l.output, body_ty.clone()),
            }
        }
        Behavior::Bind(_) => Decision::Retry,
    }
}

fn decide_transform(dag: &Dag, t: &TransformNode) -> Decision {
    // Follow Transform.target to its Arrow declaration. For unresolved
    // identifier atoms, look up the name in the declaration table; this is
    // the §8.9 operator dispatch reduced to direct lookup against the
    // bootstrap primitives.
    let Some(signature) = resolve_arrow(dag, t.target) else {
        let name = target_display_name(dag, t.target);
        return Decision::Fail(
            t.output,
            Diagnostic::ResolveError {
                name,
                span: t.span.clone(),
            },
        );
    };

    // User-function Bind-state check: if the Arrow body is UserDefined,
    // verify the Bind.value port is Resolved before trusting the signature.
    // An Unresolved function body means the body conflicts with the
    // declared signature (caught at the function's own return port), so
    // call sites should not propagate the declared return type downstream.
    if let ArrowBody::UserDefined(bind_id) = signature.body {
        match dag.port(dag.node(bind_id).as_bind().map(|b| b.value).unwrap_or(t.output)).state() {
            PortState::Uninferred => return Decision::Retry,
            PortState::Unresolved => {
                let name = target_display_name(dag, t.target);
                return Decision::Fail(
                    t.output,
                    Diagnostic::ResolveError {
                        name: format!("function `{name}` has an invalid body"),
                        span: t.span.clone(),
                    },
                );
            }
            PortState::Resolved(_) => {}
        }
    }

    if signature.inputs.len() != t.inputs.len() {
        return Decision::Fail(
            t.output,
            Diagnostic::ArityMismatch {
                function: target_display_name(dag, t.target),
                expected: signature.inputs.len(),
                actual: t.inputs.len(),
                span: t.span.clone(),
            },
        );
    }
    for (input_port, expected_ty) in t.inputs.iter().zip(signature.inputs.iter()) {
        match dag.port(*input_port).state() {
            PortState::Uninferred => return Decision::Retry,
            PortState::Unresolved => {
                return Decision::Fail(
                    t.output,
                    Diagnostic::ResolveError {
                        name: format!(
                            "(upstream failure in {})",
                            target_display_name(dag, t.target)
                        ),
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
    Decision::Set(t.output, signature.output)
}

struct ResolvedArrow {
    inputs: Vec<TypeShape>,
    output: TypeShape,
    body: ArrowBody,
}

/// Walk Transform.target to its terminal Arrow declaration and bridge
/// the input/output DeclarationIds to port-level TypeShape. Handles three
/// target shapes: direct Arrow, resolved Identifier (follow the link),
/// unresolved Identifier (look up the name in the declaration table).
fn resolve_arrow(dag: &Dag, target: DeclarationId) -> Option<ResolvedArrow> {
    let mut current = target;
    // Bounded walk — a three-hop chain covers the M1(2.5) cases
    // (identifier stub → named primitive → arrow). The bound is defensive
    // against accidental cycles in the declaration table.
    for _ in 0..8 {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Arrow { inputs, output, body } => {
                let input_shapes: Option<Vec<TypeShape>> = inputs
                    .iter()
                    .map(|id| declaration_to_type_shape(dag, *id))
                    .collect();
                let output_shape = declaration_to_type_shape(dag, *output)?;
                return Some(ResolvedArrow {
                    inputs: input_shapes?,
                    output: output_shape,
                    body: body.clone(),
                });
            }
            TypeConnective::Atom(AtomPayload::Identifier { name, resolved }) => {
                if let Some(next) = resolved {
                    current = *next;
                    continue;
                }
                // Unresolved identifier: look up the name in the
                // declaration table. This covers both operator atoms
                // (`+`, `-`, ...) and primitive type references (`Int`).
                let next = dag.declaration_by_name(name)?.id;
                if next == current {
                    return None;
                }
                current = next;
            }
            _ => return None,
        }
    }
    None
}

/// Map a DeclarationId to a port-level TypeShape. For M1(2.5) this is a
/// name-based lookup against the bootstrap primitives; once Phase 5 loads
/// the algebra fixture, this becomes the inhabitance-walk bridge from
/// e.g. `Word64` back to `Int`.
fn declaration_to_type_shape(dag: &Dag, id: DeclarationId) -> Option<TypeShape> {
    let decl = dag.declaration(id);
    let shape_from_name = |name: &str| match name {
        "Int" => Some(TypeShape::Primitive(Prim::Int)),
        "Bool" => Some(TypeShape::Primitive(Prim::Bool)),
        "String" => Some(TypeShape::Primitive(Prim::String)),
        _ => None,
    };
    if let Some(name) = &decl.name {
        if let Some(shape) = shape_from_name(name) {
            return Some(shape);
        }
    }
    if let TypeConnective::Atom(AtomPayload::Identifier { name, .. }) = &decl.connective {
        return shape_from_name(name);
    }
    None
}

/// Best-effort human-readable name for a Transform.target DeclarationId,
/// used in diagnostics. Walks through resolved Identifier atoms to find
/// something nameable.
fn target_display_name(dag: &Dag, target: DeclarationId) -> String {
    let decl = dag.declaration(target);
    if let Some(name) = &decl.name {
        return name.clone();
    }
    if let TypeConnective::Atom(AtomPayload::Identifier { name, .. }) = &decl.connective {
        return name.clone();
    }
    format!("declaration#{}", target.raw())
}

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
