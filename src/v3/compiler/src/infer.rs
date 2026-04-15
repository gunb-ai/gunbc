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
    ArrowBody, AtomPayload, Behavior, Dag, Declaration, DeclarationId, LiteralBits,
    PortId, PortState, TemplateArgument, TransformNode, TypeConnective,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::operators::operator_field_name;
use crate::types::TypeShape;

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

/// Look up a primitive type's declaration id and wrap it in a TypeShape.
/// Returns None if the primitive isn't in the declaration table — that
/// only happens if bootstrap failed to load `dsl/std/integer.dag` /
/// `dsl/std/types.dag` / `dsl/std/string_type.dag`, in which case the
/// bootstrap diagnostic is already on the Dag and the compile fails
/// through the ordinary channel.
fn primitive_shape(dag: &Dag, name: &str) -> Option<TypeShape> {
    dag.declaration_by_name(name).map(|d| TypeShape::new(d.id))
}

fn decide(dag: &Dag, index: usize) -> Decision {
    match &dag.nodes()[index] {
        Behavior::Value(v) => {
            let name = match &v.data {
                LiteralBits::Int(_) => "Int",
                LiteralBits::Bool(_) => "Bool",
                LiteralBits::String(_) => "String",
            };
            let Some(ty) = primitive_shape(dag, name) else {
                return Decision::Fail(
                    v.output,
                    Diagnostic::ResolveError {
                        name: format!(
                            "primitive `{name}` missing from declaration table — bootstrap failed"
                        ),
                        span: v.span.clone(),
                    },
                );
            };
            Decision::Set(v.output, ty)
        }
        Behavior::Transform(t) => decide_transform(dag, t),
        Behavior::Branch(b) => {
            let Some(bool_ty) = primitive_shape(dag, "Bool") else {
                return Decision::Fail(
                    b.output,
                    Diagnostic::ResolveError {
                        name: "primitive `Bool` missing from declaration table — bootstrap failed"
                            .to_string(),
                        span: b.span.clone(),
                    },
                );
            };
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
                            actual: *ty,
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
                PortState::Resolved(t) => *t,
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
                                expected: first_type,
                                actual: *other,
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
                PortState::Resolved(body_ty) => Decision::Set(l.output, *body_ty),
            }
        }
        Behavior::Bind(_) => Decision::Retry,
    }
}

fn decide_transform(dag: &Dag, t: &TransformNode) -> Decision {
    // Operator identifiers (`+`, `-`, ...) dispatch via §8.9 inhabitance
    // walks: the LHS input type's declaration is walked down through its
    // Instantiation chain until an algebra field matching the operator
    // symbol is found, at which point the field's Arrow signature — with
    // template arguments substituted — becomes the call's signature.
    //
    // Non-operator targets (user functions, named primitives) resolve via
    // the plain declaration walk in `resolve_arrow`.
    let target_decl = dag.declaration(t.target);
    let signature = if let Some(op_name) = unresolved_operator_name(target_decl) {
        let lhs_type = match t.inputs.first() {
            None => {
                return Decision::Fail(
                    t.output,
                    Diagnostic::ArityMismatch {
                        function: op_name.to_string(),
                        expected: 2,
                        actual: 0,
                        span: t.span.clone(),
                    },
                );
            }
            Some(port) => match dag.port(*port).state() {
                PortState::Uninferred => return Decision::Retry,
                PortState::Unresolved => {
                    return Decision::Fail(
                        t.output,
                        Diagnostic::ResolveError {
                            name: format!("(upstream failure in {op_name})"),
                            span: t.span.clone(),
                        },
                    );
                }
                PortState::Resolved(ty) => *ty,
            },
        };
        match resolve_operator_arrow(dag, op_name, &lhs_type) {
            Some(sig) => sig,
            None => {
                return Decision::Fail(
                    t.output,
                    Diagnostic::ResolveError {
                        name: format!(
                            "cannot dispatch operator `{op_name}` on {lhs_type:?}"
                        ),
                        span: t.span.clone(),
                    },
                );
            }
        }
    } else {
        let Some(sig) = resolve_arrow(dag, t.target) else {
            let name = target_display_name(dag, t.target);
            return Decision::Fail(
                t.output,
                Diagnostic::ResolveError {
                    name,
                    span: t.span.clone(),
                },
            );
        };
        sig
    };

    // Arrow-body state check. The three variants demand three different
    // dispatch-time invariants; each failure surfaces as a fail-closed
    // diagnostic on the call site's output port.
    match &signature.body {
        ArrowBody::UserDefined(bind_id) => {
            // User function: the call site's signature is only trustworthy
            // once the function's Bind.value port has reached Resolved.
            // Uninferred → Retry; Unresolved → fail the call site.
            let bind_id = *bind_id;
            match dag
                .port(
                    dag.node(bind_id)
                        .as_bind()
                        .map(|b| b.value)
                        .unwrap_or(t.output),
                )
                .state()
            {
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
        ArrowBody::ExternalRealization(realization_id) => {
            // Primitive realization: the target declaration must be a
            // Conj whose `meta_tag` edge points at the `Realization`
            // meta-type. The typed-edge check was already asserted at
            // bootstrap construction (`assert_realization_shape`), but
            // re-validating here catches drift from any future mutation
            // path that stores a non-realization declaration in the
            // body and bypasses the construction-time invariant.
            if !is_realization_shape(dag, *realization_id) {
                let name = target_display_name(dag, t.target);
                return Decision::Fail(
                    t.output,
                    Diagnostic::ResolveError {
                        name: format!(
                            "arrow `{name}` carries an ExternalRealization body whose target is not a realization declaration"
                        ),
                        span: t.span.clone(),
                    },
                );
            }
        }
        ArrowBody::Pending => {
            // Scaffold state: signature type-checks via inhabitance,
            // body-walking is skipped. `Pending` dissolves by M3 per
            // the §8.11 ratchet; at M1(2.6) it's valid at dispatch time.
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
                        expected: *expected_ty,
                        actual: *actual,
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

/// Lazy substitution stack for `Instantiation` walks. When inference
/// descends into an `Instantiation { template, arguments }`, it pushes
/// `arguments` onto this stack; when a downstream `TypeParam` reference
/// is encountered, the stack is consulted top-down to find the bound
/// `DeclarationId`. Pop on Instantiation exit keeps the stack balanced.
/// See M1_DESIGN.md §4 Q4 / §5 for the walk semantics.
struct SubstStack {
    frames: Vec<Vec<TemplateArgument>>,
}

impl SubstStack {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }

    fn push(&mut self, args: Vec<TemplateArgument>) {
        self.frames.push(args);
    }

    fn pop(&mut self) {
        self.frames.pop();
    }

    fn lookup(&self, param_id: DeclarationId) -> Option<DeclarationId> {
        for frame in self.frames.iter().rev() {
            for arg in frame {
                if arg.parameter == param_id {
                    return Some(arg.value);
                }
            }
        }
        None
    }
}

const WALK_DEPTH_LIMIT: usize = 32;

/// Detect whether a target declaration is an unresolved operator
/// identifier (e.g., an `Atom(Identifier { name: "+", resolved: None })`
/// whose name matches the `OPERATOR_FIELD_MAP` in `crate::operators`).
/// Operators stay unresolved through lowering and are dispatched at
/// inference time via `resolve_operator_arrow`.
fn unresolved_operator_name(decl: &Declaration) -> Option<&str> {
    if let TypeConnective::Atom(AtomPayload::Identifier {
        name,
        resolved: None,
    }) = &decl.connective
    {
        if operator_field_name(name).is_some() {
            return Some(name.as_str());
        }
    }
    None
}

/// §8.9 operator dispatch. At M1(2.6), operator signature checking takes
/// a **fast path** for both arithmetic and comparison operators:
///
/// - Arithmetic (`+`, `-`, `*`, `/`): signature is `(T, T) -> T`.
/// - Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`): signature is
///   `(T, T) -> Bool`.
///
/// The real `dsl/std/algebra.dag` expresses these through algebra
/// fields that are either primitive (`add`, `mul`) or derived
/// (`sub = add(a, negate(b))`, `div = mul(a, reciprocal(b))`, `lt = compare(a, b) == Less`).
/// Walking a derivation chain at compile time would require evaluating
/// `.dag` expressions inside arrow bodies — a surface-grammar feature
/// deferred to M2+. Until then, signature checking is the fast path
/// above; the derivation lives in runtime/emission layers.
///
/// The fast path is the one localized name-based bridge that remains
/// after SINGLE AUTHORITY cleanup. It dissolves in M2+ once the
/// surface grammar exposes algebra field access directly (e.g., writing
/// `Int.add(a, b)` instead of `a + b`). The `OPERATOR_FIELD_MAP`
/// constant is retained so the dissolution trigger is discoverable.
///
/// The `walk_for_algebra_field` / `SubstStack` machinery is still used
/// by `resolve_arrow` for non-operator targets (user function calls,
/// resolved type aliases) where the walk is direct — no algebra-field
/// indirection required.
fn resolve_operator_arrow(
    dag: &Dag,
    op_symbol: &str,
    lhs_type: &TypeShape,
) -> Option<ResolvedArrow> {
    // Touch OPERATOR_FIELD_MAP so the docs-enforced bridge is a real
    // precondition: if the operator isn't in the table, dispatch fails.
    operator_field_name(op_symbol)?;
    let output = if is_comparison_operator(op_symbol) {
        primitive_shape(dag, "Bool")?
    } else {
        *lhs_type
    };
    Some(ResolvedArrow {
        inputs: vec![*lhs_type, *lhs_type],
        output,
        body: ArrowBody::Pending,
    })
}

fn is_comparison_operator(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

/// Dispatch-time invariant check on an `ExternalRealization` target:
/// the linked declaration must be a `Conj` whose `meta_tag` edge
/// points at the `Realization` meta-type. Mirrors the assertion in
/// `bootstrap::assert_realization_shape` as a runtime safety net.
fn is_realization_shape(dag: &Dag, realization_id: DeclarationId) -> bool {
    let decl = dag.declaration(realization_id);
    if !matches!(decl.connective, TypeConnective::Conj { .. }) {
        return false;
    }
    let Some(meta_tag) = decl.meta_tag else {
        return false;
    };
    dag.declaration(meta_tag).name.as_deref() == Some("Realization")
}

/// Walk `Transform.target` to its terminal `Arrow` declaration without
/// expecting an intermediate algebra-field lookup. Used for named Arrow
/// targets (user functions, resolved type aliases) — not operators.
fn resolve_arrow(dag: &Dag, target: DeclarationId) -> Option<ResolvedArrow> {
    let mut subst = SubstStack::new();
    resolve_arrow_walk(dag, target, &mut subst, 0)
}

/// Substrate walk: given a DeclarationId, descend through
/// `Instantiation` (pushing subst frames), `Atom(Identifier { resolved
/// })` (following the link), and `Atom(TypeParam)` (looking up the
/// subst stack) until an `Arrow` is reached. At the Arrow, substitute
/// each input and output DeclarationId through the subst stack and
/// bridge to `TypeShape` via `walk_to_type_shape`.
fn resolve_arrow_walk(
    dag: &Dag,
    current: DeclarationId,
    subst: &mut SubstStack,
    depth: usize,
) -> Option<ResolvedArrow> {
    if depth >= WALK_DEPTH_LIMIT {
        return None;
    }
    let decl = dag.declaration(current);
    match &decl.connective {
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => {
            let input_shapes: Vec<TypeShape> = inputs
                .iter()
                .map(|id| walk_to_type_shape(dag, *id, subst, depth + 1))
                .collect::<Option<_>>()?;
            let output_shape = walk_to_type_shape(dag, *output, subst, depth + 1)?;
            Some(ResolvedArrow {
                inputs: input_shapes,
                output: output_shape,
                body: body.clone(),
            })
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            subst.push(arguments.clone());
            let result = resolve_arrow_walk(dag, *template, subst, depth + 1);
            subst.pop();
            result
        }
        TypeConnective::Atom(AtomPayload::Identifier {
            resolved: Some(next),
            ..
        }) => resolve_arrow_walk(dag, *next, subst, depth + 1),
        _ => None,
    }
}

/// Walk a type declaration down to a port-level `TypeShape`.
///
/// `TypeShape` is a newtype around `DeclarationId` — port types ARE
/// declaration identities. The walk descends through anonymous
/// declarations (TypeParam via subst stack, resolved Identifier link,
/// anonymous Instantiation template) until it hits the first named
/// top-level declaration, and wraps that declaration's id as the
/// port's type. There is no name-keyed bridge back to a coarse
/// primitive tag — the declaration graph IS the type identity.
fn walk_to_type_shape(
    dag: &Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> Option<TypeShape> {
    if depth >= WALK_DEPTH_LIMIT {
        return None;
    }
    let decl = dag.declaration(current);
    // Named top-level declaration: this is the port's type identity.
    if decl.name.is_some() {
        return Some(TypeShape::new(current));
    }
    // Anonymous declaration: follow the chain through the substrate.
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
            let bound = subst.lookup(current)?;
            walk_to_type_shape(dag, bound, subst, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::Identifier {
            resolved: Some(next),
            ..
        }) => walk_to_type_shape(dag, *next, subst, depth + 1),
        TypeConnective::Instantiation { template, .. } => {
            walk_to_type_shape(dag, *template, subst, depth + 1)
        }
        _ => None,
    }
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
