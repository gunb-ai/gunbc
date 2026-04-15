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

use std::collections::HashSet;

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, Dag, DeclarationId, LiteralBits, PortId,
    PortState, TemplateArgument, TransformNode, TransformTarget, TypeConnective,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::operators::OperatorKind;
use crate::types::TypeShape;

pub fn infer(dag: &mut Dag) {
    // Fixpoint loop. Runs decide for every node, then pattern
    // resolution + exhaustiveness + uniqueness for every Branch.
    // Pattern resolution is folded into the loop (not run after it)
    // so a Branch that gets flipped to Unresolved by
    // non-exhaustive/duplicate-arm checks propagates to downstream
    // consumers: the next iteration's decide pass sees the upstream
    // Unresolved and cascades Decision::Fail through Transform /
    // Branch / Loop / Bind consumers. Running pattern resolution
    // after the main loop would leave downstream types stale and
    // violate FAIL-CLOSED.
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
        if resolve_branch_patterns(dag) {
            changed = true;
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

/// Walk a declaration through `Instantiation` / `ResolvedIdentifier`
/// edges until it reaches a declaration whose connective is
/// `Disj`. Returns the `DeclarationId` of that Disj declaration —
/// or `None` if no Disj is reachable within `WALK_DEPTH_LIMIT`
/// steps, or the chain terminates at some other connective.
///
/// Used by both the `decide_transform` Branch gate and the
/// post-infer `resolve_branch_patterns` pass so that aliased or
/// instantiated sum types (`type Hue = Color` where
/// `Color = Red | Green | Blue`) resolve to the same underlying
/// `Disj` fact whether their immediate connective is the Disj
/// itself, an `Instantiation` pointing at it, or an
/// `Atom(ResolvedIdentifier(...))` chain of aliases. The
/// alternative — reading `decl.connective` directly — would
/// reject aliases even though the underlying sum fact survives
/// in the declaration graph, violating Facts Flow Forward.
fn walk_to_disj_decl(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Disj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => {
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                current = *next;
            }
            _ => return None,
        }
    }
    None
}

/// For every Branch node, resolve each Path's `BranchPattern` by
/// matching the arm's variant name against the scrutinee's Disj
/// children and verify exhaustiveness + uniqueness. Mutates the
/// Path in place for successful rewrites, marks the Branch's
/// output port Unresolved (via `Dag::mark_unresolved`) for
/// failures. Skips Branches whose input port is not yet Resolved —
/// those will be revisited on the next fixpoint iteration.
///
/// Aliases / instantiations of sum types normalize through
/// `walk_to_disj_decl` before variant resolution, so `type Hue =
/// Color` and `Color = Red | Green | Blue` both resolve arms
/// against `Color`'s variants.
///
/// Returns `true` if any state (Path.pattern rewrite or Branch
/// output port transition) changed, so the calling fixpoint loop
/// knows to keep iterating. Pattern-resolution changes and
/// coverage-check failures both propagate downstream through the
/// normal `decide_transform` cascade in subsequent iterations.
fn resolve_branch_patterns(dag: &mut Dag) -> bool {
    let mut changed = false;
    // Collect the rewrites first (immutable borrow of nodes + ports +
    // declarations), then apply them (mutable borrow of nodes). This
    // two-phase pattern avoids borrow conflicts while still reading
    // declaration bodies.
    /// Per-path rewrite the fixpoint loop applies after the
    /// read phase. Carries enough to reconstruct the right
    /// BranchPattern variant (bare vs with-payload) under
    /// mutable borrow.
    enum RewriteShape {
        Bare,
        With {
            binding_name: String,
            payload_port: PortId,
        },
    }
    struct Rewrite {
        node_index: usize,
        path_index: usize,
        result: Result<DeclarationId, Diagnostic>,
        output_port: PortId,
        shape: RewriteShape,
    }
    let mut rewrites: Vec<Rewrite> = Vec::new();
    // Coverage check: for each Branch, after resolving all paths,
    // verify the resolved set equals the scrutinee's Disj variant set
    // exactly (every variant covered by exactly one arm). Missing or
    // duplicated arms are fail-closed diagnostics on the Branch's
    // output port.
    struct CoverageCheck {
        output_port: PortId,
        span: SourceSpan,
        expected: Vec<(String, DeclarationId)>,
        // Parallel vec of (resolved_decl_id, arm_name_for_diagnostic)
        // collected in path order. Paths whose resolution failed are
        // not included — their error already fires, so coverage is
        // reported only on the subset that resolved.
        resolved_arms: Vec<(DeclarationId, String)>,
    }
    let mut coverage_checks: Vec<CoverageCheck> = Vec::new();
    for (node_index, node) in dag.nodes().iter().enumerate() {
        let Behavior::Branch(b) = node else {
            continue;
        };
        let scrutinee_ty = match dag.port(b.input).state() {
            PortState::Resolved(ty) => *ty,
            PortState::Unresolved | PortState::Uninferred => continue,
        };
        // Walk the scrutinee through alias / instantiation edges to
        // the underlying Disj. `type Hue = Color` and
        // `Color = Red | Green | Blue` both resolve arms against
        // `Color`'s variants via this walk.
        let Some(disj_decl_id) =
            walk_to_disj_decl(dag, scrutinee_ty.declaration)
        else {
            // Scrutinee doesn't resolve to a Disj — the main infer
            // pass caught this as a TypeMismatch already. Skip.
            continue;
        };
        let disj_variants: Vec<(String, DeclarationId)> =
            match &dag.declaration(disj_decl_id).connective {
                TypeConnective::Disj { variants } => variants
                    .iter()
                    .map(|f| (f.label.clone(), f.ty))
                    .collect(),
                _ => unreachable!("walk_to_disj_decl returned a non-Disj declaration"),
            };
        let mut resolved_arms: Vec<(DeclarationId, String)> = Vec::new();
        for (path_index, path) in b.paths.iter().enumerate() {
            let (result, shape) = match &path.pattern {
                crate::dag::BranchPattern::ResolvedVariant(id) => {
                    // Already resolved (e.g., if/else paths on a
                    // second infer pass). Record for coverage check
                    // and skip the rewrite.
                    resolved_arms.push((
                        *id,
                        dag.declaration(*id)
                            .name
                            .clone()
                            .unwrap_or_else(|| format!("declaration#{}", id.raw())),
                    ));
                    continue;
                }
                crate::dag::BranchPattern::ResolvedVariantWith { decl, binding_name, .. } => {
                    // Already resolved on a prior infer pass.
                    resolved_arms.push((
                        *decl,
                        dag.declaration(*decl)
                            .name
                            .clone()
                            .unwrap_or_else(|| binding_name.clone()),
                    ));
                    continue;
                }
                crate::dag::BranchPattern::UnresolvedVariant { name, span } => {
                    let res = match disj_variants.iter().find(|(label, _)| label == name) {
                        Some((_, ty)) => {
                            resolved_arms.push((*ty, name.clone()));
                            Ok(*ty)
                        }
                        None => Err(Diagnostic::ResolveError {
                            name: format!(
                                "variant `{name}` is not a constructor of this match's scrutinee type"
                            ),
                            span: span.clone(),
                        }),
                    };
                    (res, RewriteShape::Bare)
                }
                crate::dag::BranchPattern::UnresolvedVariantWith {
                    name,
                    binding_name,
                    payload_port,
                    span,
                } => {
                    let res = match disj_variants.iter().find(|(label, _)| label == name) {
                        Some((_, ty)) => {
                            resolved_arms.push((*ty, name.clone()));
                            Ok(*ty)
                        }
                        None => Err(Diagnostic::ResolveError {
                            name: format!(
                                "variant `{name}` is not a constructor of this match's scrutinee type"
                            ),
                            span: span.clone(),
                        }),
                    };
                    (
                        res,
                        RewriteShape::With {
                            binding_name: binding_name.clone(),
                            payload_port: *payload_port,
                        },
                    )
                }
            };
            rewrites.push(Rewrite {
                node_index,
                path_index,
                result,
                output_port: b.output,
                shape,
            });
        }
        coverage_checks.push(CoverageCheck {
            output_port: b.output,
            span: b.span.clone(),
            expected: disj_variants,
            resolved_arms,
        });
    }
    for rewrite in rewrites {
        match rewrite.result {
            Ok(variant_id) => {
                // Reconstruct the right resolved variant from the
                // captured shape. Bare patterns rewrite to
                // ResolvedVariant; with-payload patterns rewrite
                // to ResolvedVariantWith and preserve the binding
                // name + payload port edge.
                let resolved = match rewrite.shape {
                    RewriteShape::Bare => {
                        crate::dag::BranchPattern::ResolvedVariant(variant_id)
                    }
                    RewriteShape::With {
                        binding_name,
                        payload_port,
                    } => crate::dag::BranchPattern::ResolvedVariantWith {
                        decl: variant_id,
                        binding_name,
                        payload_port,
                    },
                };
                if let Behavior::Branch(b) = &mut dag.nodes_mut()[rewrite.node_index] {
                    b.paths[rewrite.path_index].pattern = resolved;
                    changed = true;
                }
            }
            Err(diag) => {
                if !matches!(
                    dag.port(rewrite.output_port).state(),
                    PortState::Unresolved
                ) {
                    dag.mark_unresolved(rewrite.output_port, diag);
                    changed = true;
                }
            }
        }
    }
    // Coverage pass: for each Branch whose paths resolved (fully or
    // partially), verify exhaustiveness (every variant covered) and
    // uniqueness (no variant duplicated). Skip if the Branch's output
    // already has a diagnostic from a pattern resolution failure —
    // coverage is meaningless on a partially-resolved arm set.
    for check in coverage_checks {
        if matches!(dag.port(check.output_port).state(), PortState::Unresolved) {
            continue;
        }
        // Uniqueness: each resolved DeclarationId appears at most once.
        let mut seen: HashSet<DeclarationId> = HashSet::new();
        let mut duplicate: Option<String> = None;
        for (id, name) in &check.resolved_arms {
            if !seen.insert(*id) {
                duplicate = Some(name.clone());
                break;
            }
        }
        if let Some(name) = duplicate {
            dag.mark_unresolved(
                check.output_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "duplicate match arm for variant `{name}` — each variant of a sum type must match exactly once"
                    ),
                    span: check.span.clone(),
                },
            );
            changed = true;
            continue;
        }
        // Exhaustiveness: every expected variant must be in the
        // resolved set. Surface the first missing variant's name as
        // the diagnostic label.
        let missing: Vec<&str> = check
            .expected
            .iter()
            .filter(|(_, id)| !seen.contains(id))
            .map(|(label, _)| label.as_str())
            .collect();
        if !missing.is_empty() {
            let missing_list = missing.join(", ");
            dag.mark_unresolved(
                check.output_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "non-exhaustive match: missing arm(s) for variant(s) `{missing_list}` — every constructor of the scrutinee's sum type must be covered"
                    ),
                    span: check.span,
                },
            );
            changed = true;
        }
    }
    changed
}

enum Decision {
    Set(PortId, TypeShape),
    Fail(PortId, Diagnostic),
    Retry,
}

fn decide(dag: &Dag, index: usize) -> Decision {
    match &dag.nodes()[index] {
        Behavior::Value(v) => {
            let shape_and_name = match &v.data {
                LiteralBits::Int(_) => (dag.int_shape(), "Int"),
                LiteralBits::Bool(_) => (dag.bool_shape(), "Bool"),
                LiteralBits::String(_) => (dag.string_shape(), "String"),
            };
            let Some(ty) = shape_and_name.0 else {
                return Decision::Fail(
                    v.output,
                    Diagnostic::ResolveError {
                        name: format!(
                            "primitive `{}` missing from declaration table — bootstrap failed",
                            shape_and_name.1
                        ),
                        span: v.span.clone(),
                    },
                );
            };
            Decision::Set(v.output, ty)
        }
        Behavior::Transform(t) => decide_transform(dag, t),
        Behavior::Branch(b) => {
            // Branch dispatches on a sum type. Its input must
            // walk through alias/instantiation edges to a
            // declaration whose connective is
            // `TypeConnective::Disj { .. }`. Bool and Classical are
            // Disj declarations (True | False), so `if` lowers here
            // naturally. User-defined sums like
            // `type Sign = Plus | Minus` also pass. Aliased sums
            // (`type Hue = Color` where Color is a sum) pass
            // via `walk_to_disj_decl`. String, Int, Float, etc.
            // fail this check — the previous M0 "must be Bool"
            // rule was the degenerate case of this broader rule.
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
                PortState::Resolved(ty) => {
                    if walk_to_disj_decl(dag, ty.declaration).is_none() {
                        let Some(bool_ty) = dag.bool_shape() else {
                            return Decision::Fail(
                                b.output,
                                Diagnostic::ResolveError {
                                    name: "primitive `Bool` missing from declaration table — bootstrap failed"
                                        .to_string(),
                                    span: b.span.clone(),
                                },
                            );
                        };
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
    // Structural dispatch on `TransformTarget`. `Operator` carries the
    // resolved `OperatorKind` directly and produces its signature from
    // the LHS operand type (arithmetic returns operand type, comparison
    // returns Bool). `Callable` points at a DeclarationId and walks
    // `resolve_arrow` through any Instantiation / ResolvedIdentifier
    // chain to recover the concrete Arrow signature.
    let signature = match &t.target {
        TransformTarget::Operator(op_kind) => {
            let lhs_type = match t.inputs.first() {
                None => {
                    return Decision::Fail(
                        t.output,
                        Diagnostic::ArityMismatch {
                            function: op_kind.symbol().to_string(),
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
                                name: format!(
                                    "(upstream failure in {})",
                                    op_kind.symbol()
                                ),
                                span: t.span.clone(),
                            },
                        );
                    }
                    PortState::Resolved(ty) => *ty,
                },
            };
            match resolve_operator_arrow(dag, *op_kind, &lhs_type) {
                Some(sig) => sig,
                None => {
                    return Decision::Fail(
                        t.output,
                        Diagnostic::ResolveError {
                            name: format!(
                                "cannot dispatch operator `{}` on {lhs_type:?}",
                                op_kind.symbol()
                            ),
                            span: t.span.clone(),
                        },
                    );
                }
            }
        }
        TransformTarget::Callable(target_id) => {
            let Some(sig) = resolve_arrow(dag, *target_id) else {
                let name = target_display_name(dag, *target_id);
                return Decision::Fail(
                    t.output,
                    Diagnostic::ResolveError {
                        name,
                        span: t.span.clone(),
                    },
                );
            };
            sig
        }
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
                    let name = transform_target_display_name(dag, &t.target);
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
            // Conj whose `meta_tag` edge points at the cached
            // `Realization` meta-type (populated at bootstrap). The
            // typed-edge check was asserted at bootstrap construction
            // (`assert_realization_shape`); re-validating here catches
            // drift from any future mutation path that bypasses the
            // construction-time invariant.
            if !is_realization_shape(dag, *realization_id) {
                let name = transform_target_display_name(dag, &t.target);
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
            // Realization-lag scaffold: signature type-checks via
            // inhabitance, body-walking is skipped. Dissolves by M3
            // per the §8.11 ratchet; at M1(2.7) it's valid at dispatch
            // time.
        }
        ArrowBody::Unparsed(_) => {
            // Surface-grammar scaffold: signature type-checks,
            // body is not yet parseable under the M1(2.7) grammar.
            // Callers can dispatch against the signature; the body
            // source span is preserved for M2+ parser extensions.
            // Dissolves when block bodies become fully parseable.
        }
    }

    if signature.inputs.len() != t.inputs.len() {
        return Decision::Fail(
            t.output,
            Diagnostic::ArityMismatch {
                function: transform_target_display_name(dag, &t.target),
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
                            transform_target_display_name(dag, &t.target)
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

/// §8.9 operator dispatch via structural algebra walk.
///
/// Walks the LHS type's declaration chain (following `Instantiation`
/// and `ResolvedIdentifier` edges) until it reaches an algebra
/// `Conj` declaration, then looks up the operator's field by name
/// (via `OperatorKind::algebra_field_name`). The field's Arrow
/// signature is read from the declaration graph — the compiler
/// consumes `std/algebra.dag` as authority rather than fabricating
/// signatures in Rust.
///
/// **Receiver substitution rule.** The algebra's first type
/// parameter is the "receiver" — it represents "the inhabitant of
/// this algebra." When the algebra field's Arrow has `(T, T) -> T`,
/// the resolved signature substitutes `T → source_id` (the user's
/// `lhs_type` declaration, e.g., `Int`), not `T → Word64` (the
/// algebra instantiation's template argument). This keeps port
/// types consistent with user-facing primitives. Non-receiver
/// positions (`Bool` in the comparison arrows, `Ordering` in
/// `compare`) stay as-is.
///
/// **Current coverage.** Works for scalar algebras whose receiver
/// is the first type parameter used directly: `OrderedRing<T>`,
/// `Ring<T>`, `Semiring<T>`, `Field<T>`, `Lattice<T>`,
/// `BooleanAlgebra<T>`. This is sufficient for `Int`, `Float`, and
/// any user type that aliases an `OrderedRing`/`Ring`/`Field`
/// instantiation.
///
/// **Not yet covered** (tracked in DOWNSTREAM_REQUIREMENTS.md as
/// class-5 gaps):
/// - `Bool`: no structural link from `Classical` to
///   `BooleanAlgebra`. Requires either `inhabits` surface syntax
///   and a `logic.dag` edit, or consumption of the
///   `kernel_algebra_profile` data table.
/// - `String` / collection-level algebras whose receiver is
///   `FreeMonoid<T>` / `Set<T>` / `Map<K, V>` (the whole
///   instantiation, not just `T`): needs a refinement to the
///   substitution rule.
///
/// For those cases the resolver falls back to a Rust-side bridge
/// with a `(T, T) -> T` / `(T, T) -> Bool` signature. The fallback
/// is explicit about being a scaffold (see OperatorKind's
/// dissolution receipt).
fn resolve_operator_arrow(
    dag: &Dag,
    op_kind: OperatorKind,
    lhs_type: &TypeShape,
) -> Option<ResolvedArrow> {
    let source_id = lhs_type.declaration;
    // Walk the source type's declaration chain to find the algebra
    // Conj. We follow Instantiation/ResolvedIdentifier edges; at
    // each step the template argument bindings are intentionally
    // discarded — see the receiver substitution rule in the doc
    // comment.
    let mut current = source_id;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Conj { children } => {
                // Found the algebra. Look up the operator's field by
                // name.
                let field_name = op_kind.algebra_field_name();
                if let Some(field) = children.iter().find(|f| f.label == field_name) {
                    return read_algebra_field(
                        dag, decl, field.ty, source_id, op_kind, lhs_type,
                    );
                }
                // Algebra doesn't declare this operator's field —
                // fall back to the Rust-side scaffold bridge below.
                break;
            }
            TypeConnective::Instantiation { template, .. } => {
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                current = *next;
            }
            // Terminal non-follow cases — no algebra in this chain.
            TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
            | TypeConnective::Atom(AtomPayload::TypeParam(_))
            | TypeConnective::Atom(AtomPayload::Literal(_))
            | TypeConnective::Disj { .. }
            | TypeConnective::Arrow { .. }
            | TypeConnective::Cardinality { .. } => break,
        }
    }
    // Fallback: Rust-side scaffold bridge for primitives whose
    // structural walk doesn't terminate at an algebra Conj (Bool,
    // Classical; collection-level algebras). Documented class-5 gap.
    let output = match op_kind {
        OperatorKind::Arithmetic(_) => *lhs_type,
        OperatorKind::Comparison(_) => dag.bool_shape()?,
    };
    Some(ResolvedArrow {
        inputs: vec![*lhs_type, *lhs_type],
        output,
        body: ArrowBody::Pending,
    })
}

/// Read an algebra field's Arrow signature and substitute the
/// algebra's receiver type parameter to the source declaration.
///
/// The algebra field `field_ty` points at an Arrow declaration
/// emitted by `type_to_declaration_id` when lowering the algebra's
/// record type (e.g., `OrderedRing<T>`'s `add: fn(T, T) -> T`).
/// The Arrow's input/output DeclarationIds reference the algebra's
/// receiver type parameter directly. Substitution replaces those
/// references with `source_id`; non-receiver positions walk to
/// their named anchor via the same walk `walk_to_type_shape` does.
fn read_algebra_field(
    dag: &Dag,
    algebra_decl: &crate::dag::Declaration,
    field_ty: DeclarationId,
    source_id: DeclarationId,
    op_kind: OperatorKind,
    lhs_type: &TypeShape,
) -> Option<ResolvedArrow> {
    // The algebra's first type parameter is the receiver. For
    // `OrderedRing<T>` this is T's DeclarationId.
    let receiver_param = algebra_decl.type_params.first().copied();
    // Unwrap the field's connective to the underlying Arrow. The
    // field declaration was emitted by `type_to_declaration_id`'s
    // Arrow arm as `TypeConnective::Arrow { inputs, output, body }`.
    let field_decl = dag.declaration(field_ty);
    let (arrow_inputs, arrow_output, arrow_body) = match &field_decl.connective {
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => (inputs.clone(), *output, body.clone()),
        // Algebra field that isn't an Arrow (e.g., the `zero: T`
        // constant). Not a callable operator — fall back.
        _ => return None,
    };
    // Translate each input / output DeclarationId through the
    // receiver-substitution rule.
    let input_shapes: Vec<TypeShape> = arrow_inputs
        .iter()
        .map(|id| substitute_receiver(dag, *id, receiver_param, source_id))
        .collect::<Option<_>>()?;
    let output_shape = substitute_receiver(dag, arrow_output, receiver_param, source_id)?;
    // Sanity check: the arity is always 2 for binary operators.
    // If algebra.dag ever declares a field under one of our
    // operator names with a different arity, the check downstream
    // in `decide_transform` catches the mismatch as a
    // `Diagnostic::ArityMismatch`, but we also want to preserve
    // `lhs_type` in the signature as a debug fallback if the walk
    // succeeds but the shape is unexpectedly wrong.
    let _ = lhs_type;
    let _ = op_kind;
    Some(ResolvedArrow {
        inputs: input_shapes,
        output: output_shape,
        body: arrow_body,
    })
}

/// Substitute a declaration id through the receiver-substitution
/// rule: if the id matches the algebra's receiver type parameter,
/// return the source type; otherwise walk to the nearest named
/// declaration (same model as `walk_to_type_shape` but with the
/// receiver case short-circuited).
fn substitute_receiver(
    dag: &Dag,
    current: DeclarationId,
    receiver_param: Option<DeclarationId>,
    source_id: DeclarationId,
) -> Option<TypeShape> {
    if Some(current) == receiver_param {
        return Some(TypeShape::new(source_id));
    }
    let decl = dag.declaration(current);
    // Named top-level declaration (Bool, Ordering, etc.) is the
    // anchor.
    if decl.name.is_some() {
        return Some(TypeShape::new(current));
    }
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            substitute_receiver(dag, *next, receiver_param, source_id)
        }
        TypeConnective::Instantiation { template, .. } => {
            substitute_receiver(dag, *template, receiver_param, source_id)
        }
        // Non-receiver TypeParam in an algebra field (e.g., a
        // second generic parameter) isn't resolvable at M1(2.7).
        // Multi-parameter algebra operator dispatch is a class-5
        // gap; M2 will refine `substitute_receiver` to cover it.
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => None,
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_)) => None,
        TypeConnective::Atom(AtomPayload::Literal(_)) => None,
        TypeConnective::Conj { .. } => None,
        TypeConnective::Disj { .. } => None,
        TypeConnective::Arrow { .. } => None,
        TypeConnective::Cardinality { .. } => None,
    }
}

/// Dispatch-time invariant check on an `ExternalRealization` target:
/// the linked declaration must be a `Conj` with a non-`None`
/// `meta_tag` edge. The meta_tag IS the realization marker — no
/// further name/id comparison is required.
///
/// **Round-10 correction.** Earlier revisions compared `meta_tag`
/// against a cached `Dag::realization_meta_id()` pointing at a
/// `Realization` declaration. Production bootstrap doesn't load a
/// `Realization` declaration (realization facts live in
/// `dsl/extdeps/languages/*` per the thesis, not in the std/ set
/// the M1(2.7) bootstrap consumes), so the cache was always `None`
/// and the check always failed. The shape check is now purely
/// structural: "Conj with a meta_tag" is the realization marker.
/// The structural shape is what `bootstrap::assert_realization_shape`
/// and the `#[cfg(test)]` realization smoke test already validate
/// at construction time. Any drift is caught at both construction
/// and dispatch.
fn is_realization_shape(dag: &Dag, realization_id: DeclarationId) -> bool {
    let decl = dag.declaration(realization_id);
    if !matches!(decl.connective, TypeConnective::Conj { .. }) {
        return false;
    }
    decl.meta_tag.is_some()
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
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            resolve_arrow_walk(dag, *next, subst, depth + 1)
        }
        // Terminal non-Arrow cases. Each is unreachable in well-formed
        // user code; enumerated explicitly (rather than `_ => None`) so
        // that any future `TypeConnective` or `AtomPayload` variant
        // forces consideration here instead of silently falling through.
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_)) => None,
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => None,
        TypeConnective::Atom(AtomPayload::Literal(_)) => None,
        TypeConnective::Conj { .. } => None,
        TypeConnective::Disj { .. } => None,
        TypeConnective::Cardinality { .. } => None,
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
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            walk_to_type_shape(dag, *next, subst, depth + 1)
        }
        TypeConnective::Instantiation { template, .. } => {
            walk_to_type_shape(dag, *template, subst, depth + 1)
        }
        // Terminal non-follow cases. An anonymous `UnresolvedIdentifier`
        // means the sweep did not resolve the reference — the phantom
        // diagnostic is already attached, and this walk fails so the
        // caller can surface it. The structural Conj/Disj/Arrow/
        // Cardinality cases represent anonymous inline types that
        // have no `TypeShape` identity at M1(2.7) — M2 port-type
        // extension will either admit them or keep returning None.
        // Enumerated explicitly (rather than `_ => None`) so any
        // future variant forces consideration here.
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_)) => None,
        TypeConnective::Atom(AtomPayload::Literal(_)) => None,
        TypeConnective::Conj { .. } => None,
        TypeConnective::Disj { .. } => None,
        TypeConnective::Arrow { .. } => None,
        TypeConnective::Cardinality { .. } => None,
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
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) => name.clone(),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            target_display_name(dag, *next)
        }
        _ => format!("declaration#{}", target.raw()),
    }
}

/// Best-effort human-readable name for a `TransformTarget`. Dispatches
/// on the variant: `Callable` goes through `target_display_name`,
/// `Operator` renders the source symbol directly.
fn transform_target_display_name(dag: &Dag, target: &TransformTarget) -> String {
    match target {
        TransformTarget::Callable(id) => target_display_name(dag, *id),
        TransformTarget::Operator(op_kind) => op_kind.symbol().to_string(),
    }
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
