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
    ArrowBody, AtomPayload, Behavior, Dag, Declaration, DeclarationId, Field, LiteralBits, PortId,
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
                        PortState::Resolved(existing)
                            if type_shapes_equivalent(dag, &existing, &ty) => {}
                        PortState::Resolved(existing) => {
                            if is_retryable_generic_decl(dag, existing.declaration)
                                || is_retryable_generic_decl(dag, ty.declaration)
                            {
                                dag.set_port_type(port, ty);
                                changed = true;
                                continue;
                            }
                            let span = node_span_for_port(dag, port).unwrap_or_else(synthetic_span);
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
        if resolve_branch_payload_bindings(dag) {
            changed = true;
        }
        if resolve_field_project_targets(dag) {
            changed = true;
        }
        if resolve_callable_targets(dag) {
            changed = true;
        }
        if materialize_callable_signature_instantiations(dag) {
            changed = true;
        }
        if resolve_lambda_parameter_types(dag) {
            changed = true;
        }
        if validate_user_defined_function_signatures(dag) {
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
            TypeConnective::Cardinality {
                bound: crate::dag::CardinalityBound::AtMostOne,
                ..
            } => return existing_optional_match_disj_decl(dag, current),
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

fn existing_optional_match_disj_decl(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
) -> Option<DeclarationId> {
    dag.optional_match_disj(cardinality_decl_id)
}

fn walk_to_optional_cardinality_decl(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..WALK_DEPTH_LIMIT {
        match &dag.declaration(current).connective {
            TypeConnective::Cardinality {
                bound: crate::dag::CardinalityBound::AtMostOne,
                ..
            } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

fn ensure_optional_match_disj(
    dag: &mut Dag,
    cardinality_decl_id: DeclarationId,
) -> Option<DeclarationId> {
    if let Some(existing) = existing_optional_match_disj_decl(dag, cardinality_decl_id) {
        return Some(existing);
    }
    let (element, span) = match dag.declaration(cardinality_decl_id).connective.clone() {
        TypeConnective::Cardinality {
            element,
            bound: crate::dag::CardinalityBound::AtMostOne,
        } => (element, dag.declaration(cardinality_decl_id).span.clone()),
        _ => return None,
    };

    let some_payload_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: some_payload_id,
        name: None,
        connective: TypeConnective::Conj {
            children: vec![Field {
                label: "_0".to_string(),
                ty: element,
            }],
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: span.clone(),
    });

    let none_payload_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: none_payload_id,
        name: None,
        connective: TypeConnective::Conj {
            children: Vec::new(),
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: span.clone(),
    });

    let disj_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: disj_id,
        name: None,
        connective: TypeConnective::Disj {
            variants: vec![
                Field {
                    label: "Some".to_string(),
                    ty: some_payload_id,
                },
                Field {
                    label: "None".to_string(),
                    ty: none_payload_id,
                },
            ],
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span,
    });
    dag.set_optional_match_disj(cardinality_decl_id, disj_id);

    Some(disj_id)
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
    let optional_scrutinees: HashSet<DeclarationId> = dag
        .nodes()
        .iter()
        .filter_map(|node| {
            let Behavior::Branch(branch) = node else {
                return None;
            };
            let PortState::Resolved(ty) = dag.port(branch.input).state() else {
                return None;
            };
            walk_to_optional_cardinality_decl(dag, ty.declaration)
        })
        .collect();
    for decl_id in optional_scrutinees {
        if existing_optional_match_disj_decl(dag, decl_id).is_none()
            && ensure_optional_match_disj(dag, decl_id).is_some()
        {
            changed = true;
        }
    }
    // Collect the rewrites first (immutable borrow of nodes + ports +
    // declarations), then apply them (mutable borrow of nodes). This
    // two-phase pattern avoids borrow conflicts while still reading
    // declaration bodies.
    struct Rewrite {
        node_index: usize,
        path_index: usize,
        result: Result<DeclarationId, Diagnostic>,
        output_port: PortId,
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
        let Some(disj_decl_id) = walk_to_disj_decl(dag, scrutinee_ty.declaration).or_else(|| {
            walk_to_optional_cardinality_decl(dag, scrutinee_ty.declaration)
                .and_then(|cardinality| existing_optional_match_disj_decl(dag, cardinality))
        }) else {
            // Scrutinee doesn't resolve to a Disj — the main infer
            // pass caught this as a TypeMismatch already. Skip.
            continue;
        };
        let disj_variants: Vec<(String, DeclarationId)> =
            match &dag.declaration(disj_decl_id).connective {
                TypeConnective::Disj { variants } => {
                    variants.iter().map(|f| (f.label.clone(), f.ty)).collect()
                }
                _ => unreachable!("walk_to_disj_decl returned a non-Disj declaration"),
            };
        let mut resolved_arms: Vec<(DeclarationId, String)> = Vec::new();
        for (path_index, path) in b.paths.iter().enumerate() {
            let result = match &path.pattern {
                crate::dag::BranchPattern::ResolvedVariant(id) => {
                    // Already resolved (e.g., if/else paths on a
                    // second infer pass). Record for coverage check
                    // and skip the rewrite.
                    let arm_name = disj_variants
                        .iter()
                        .find(|(_, variant_id)| variant_id == id)
                        .map(|(label, _)| label.clone())
                        .unwrap_or_else(|| format!("declaration#{}", id.raw()));
                    resolved_arms.push((*id, arm_name));
                    continue;
                }
                crate::dag::BranchPattern::UnresolvedVariant { name, span } => {
                    match disj_variants.iter().find(|(label, _)| label == name) {
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
                    }
                }
            };
            rewrites.push(Rewrite {
                node_index,
                path_index,
                result,
                output_port: b.output,
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
                if let Behavior::Branch(b) = &mut dag.nodes_mut()[rewrite.node_index] {
                    b.paths[rewrite.path_index].pattern =
                        crate::dag::BranchPattern::ResolvedVariant(variant_id);
                    changed = true;
                }
            }
            Err(diag) => {
                if !matches!(dag.port(rewrite.output_port).state(), PortState::Unresolved) {
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

fn resolve_branch_payload_bindings(dag: &mut Dag) -> bool {
    struct BindingRewrite {
        port: PortId,
        span: SourceSpan,
        result: Result<PayloadBindingResolution, Diagnostic>,
    }

    let mut rewrites: Vec<BindingRewrite> = Vec::new();

    for node in dag.nodes() {
        let Behavior::Branch(b) = node else {
            continue;
        };

        match dag.port(b.input).state() {
            PortState::Uninferred => continue,
            PortState::Unresolved => {
                for path in &b.paths {
                    let Some(binding) = &path.binding else {
                        continue;
                    };
                    rewrites.push(BindingRewrite {
                        port: binding.payload_port,
                        span: payload_binding_span(path, &b.span),
                        result: Err(Diagnostic::ResolveError {
                            name: "(upstream failure in match payload binding)".to_string(),
                            span: payload_binding_span(path, &b.span),
                        }),
                    });
                }
            }
            PortState::Resolved(scrutinee_ty) => {
                let mut subst = SubstStack::new();
                let disj_decl_id =
                    walk_to_disj_decl_with_subst(dag, scrutinee_ty.declaration, &mut subst);
                if disj_decl_id.is_none()
                    && is_retryable_generic_decl(dag, scrutinee_ty.declaration)
                {
                    continue;
                }
                let variants = disj_decl_id.map(|id| match &dag.declaration(id).connective {
                    TypeConnective::Disj { variants } => variants
                        .iter()
                        .map(|field| (field.label.clone(), field.ty))
                        .collect::<Vec<_>>(),
                    _ => {
                        unreachable!("walk_to_disj_decl_with_subst returned a non-Disj declaration")
                    }
                });

                for path in &b.paths {
                    let Some(binding) = &path.binding else {
                        continue;
                    };
                    let span = payload_binding_span(path, &b.span);
                    let result = match &variants {
                        None => Err(Diagnostic::ResolveError {
                            name: format!(
                                "cannot bind payload `{}` because the match scrutinee does not walk to a Disj type",
                                binding.binding_name
                            ),
                            span: span.clone(),
                        }),
                        Some(variants) => {
                            let variant = match &path.pattern {
                                crate::dag::BranchPattern::ResolvedVariant(id) => {
                                    variants.iter().find(|(_, ty)| *ty == *id)
                                }
                                crate::dag::BranchPattern::UnresolvedVariant { name, .. } => {
                                    variants.iter().find(|(label, _)| label == name)
                                }
                            };
                            match variant {
                                Some((variant_name, variant_id)) => resolve_payload_binding_type(
                                    dag,
                                    *variant_id,
                                    &subst,
                                    variant_name,
                                    &binding.binding_name,
                                    &span,
                                ),
                                None => {
                                    let arm_name = match &path.pattern {
                                        crate::dag::BranchPattern::ResolvedVariant(id) => variants
                                            .iter()
                                            .find(|(_, ty)| *ty == *id)
                                            .map(|(label, _)| label.clone())
                                            .unwrap_or_else(|| format!("declaration#{}", id.raw())),
                                        crate::dag::BranchPattern::UnresolvedVariant { name, .. } => {
                                            name.clone()
                                        }
                                    };
                                    Err(Diagnostic::ResolveError {
                                        name: format!(
                                            "variant `{arm_name}` is not a constructor of this match's scrutinee type"
                                        ),
                                        span: span.clone(),
                                    })
                                }
                            }
                        }
                    };
                    rewrites.push(BindingRewrite {
                        port: binding.payload_port,
                        span,
                        result,
                    });
                }
            }
        }
    }

    let mut changed = false;
    for rewrite in rewrites {
        match rewrite.result {
            Ok(resolution) => {
                let ty = match resolution {
                    PayloadBindingResolution::Direct(ty) => ty,
                    PayloadBindingResolution::SpecializedRecord {
                        variant_decl_id,
                        subst,
                    } => materialize_specialized_payload_record(dag, variant_decl_id, &subst),
                };
                match dag.port(rewrite.port).state().clone() {
                    PortState::Unresolved => {}
                    PortState::Uninferred => {
                        dag.set_port_type(rewrite.port, ty);
                        changed = true;
                    }
                    PortState::Resolved(existing)
                        if type_shapes_equivalent(dag, &existing, &ty) => {}
                    PortState::Resolved(existing) => {
                        dag.mark_unresolved(
                            rewrite.port,
                            Diagnostic::TypeMismatch {
                                expected: existing,
                                actual: ty,
                                span: rewrite.span,
                            },
                        );
                        changed = true;
                    }
                }
            }
            Err(diag) => {
                if !matches!(dag.port(rewrite.port).state(), PortState::Unresolved) {
                    dag.mark_unresolved(rewrite.port, diag);
                    changed = true;
                }
            }
        }
    }

    changed
}

fn payload_binding_span(path: &crate::dag::Path, branch_span: &SourceSpan) -> SourceSpan {
    match &path.pattern {
        crate::dag::BranchPattern::UnresolvedVariant { span, .. } => span.clone(),
        crate::dag::BranchPattern::ResolvedVariant(_) => branch_span.clone(),
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
                    if walk_to_disj_decl(dag, ty.declaration).is_none()
                        && walk_to_optional_cardinality_decl(dag, ty.declaration).is_none()
                    {
                        if is_retryable_generic_decl(dag, ty.declaration) {
                            return Decision::Retry;
                        }
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
                    PortState::Resolved(other)
                        if type_shapes_equivalent(dag, other, &first_type) => {}
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
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            return decide_field_project(dag, t, field_label, *field_child);
        }
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
                                name: format!("(upstream failure in {})", op_kind.symbol()),
                                span: t.span.clone(),
                            },
                        );
                    }
                    PortState::Resolved(ty) => *ty,
                },
            };
            if is_retryable_generic_decl(dag, lhs_type.declaration) {
                return Decision::Retry;
            }
            match resolve_operator_arrow(dag, *op_kind, &lhs_type) {
                Some(sig) => sig,
                None => {
                    if is_retryable_generic_decl(dag, lhs_type.declaration) {
                        return Decision::Retry;
                    }
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
            match resolve_callable_target(dag, *target_id, &t.inputs, &t.span) {
                CallableTargetResolution::Retry => return Decision::Retry,
                CallableTargetResolution::Fail(diag) => return Decision::Fail(t.output, diag),
                CallableTargetResolution::Resolved { signature, .. } => signature,
            }
        }
    };

    // Arrow-body state check. The five variants partition into the
    // "check something" cases (UserDefined → Bind.value port state,
    // ExternalRealization → realization shape) and the "no body to
    // walk" cases (Pending/NoBody/Unparsed). Each failure surfaces as
    // a fail-closed diagnostic on the call site's output port.
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
        ArrowBody::Pending | ArrowBody::NoBody => {
            // Both shapes mean "no executable body to walk." `Pending`
            // is the bootstrap/anonymous "no body needed" scaffold (see
            // the `ArrowBody` ledger in `dag.rs`); `NoBody` is the
            // terminal "no body by construction" form used by named
            // type aliases (`type Callback = fn(Int) -> Int`).
            // `decide_transform` treats them identically: signature
            // inhabitance accepts; body walking is skipped. The variant
            // distinction exists so `lens_structural_resolution` can
            // single out the one shape (`Arrow { name: Some(_), body:
            // Pending }`) that indicates an R13-class regression in
            // body patching.
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
            PortState::Resolved(actual) if type_shapes_equivalent(dag, actual, expected_ty) => {}
            PortState::Resolved(actual) => {
                if is_retryable_generic_decl(dag, actual.declaration)
                    || is_retryable_generic_decl(dag, expected_ty.declaration)
                {
                    return Decision::Retry;
                }
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

struct ResolvedArrowDecl {
    inputs: Vec<DeclarationId>,
    output: DeclarationId,
}

#[derive(Clone)]
struct PortTypeContext {
    decl: DeclarationId,
    subst: SubstStack,
}

struct CallableSignatureContext {
    inputs: Vec<PortTypeContext>,
    output: PortTypeContext,
}

enum CallableTargetResolution {
    Retry,
    Fail(Diagnostic),
    Resolved {
        template: DeclarationId,
        arguments: Vec<TemplateArgument>,
        signature: ResolvedArrow,
    },
}

enum CallableBindingResolution {
    Resolved,
    Retry,
    Conflict,
}

/// Lazy substitution stack for `Instantiation` walks. When inference
/// descends into an `Instantiation { template, arguments }`, it pushes
/// `arguments` onto this stack; when a downstream `TypeParam` reference
/// is encountered, the stack is consulted top-down to find the bound
/// `DeclarationId`. Pop on Instantiation exit keeps the stack balanced.
/// See M1_DESIGN.md §4 Q4 / §5 for the walk semantics.
#[derive(Clone)]
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
                    if arg.value == param_id {
                        return None;
                    }
                    return Some(arg.value);
                }
            }
        }
        None
    }
}

enum PayloadBindingResolution {
    Direct(TypeShape),
    SpecializedRecord {
        variant_decl_id: DeclarationId,
        subst: SubstStack,
    },
}

fn declaration_is_callable(dag: &Dag, current: DeclarationId, depth: usize) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Arrow { .. } => true,
        TypeConnective::Instantiation { template, .. } => {
            declaration_is_callable(dag, *template, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            declaration_is_callable(dag, *next, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::TypeParam(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Cardinality { .. } => false,
    }
}

fn is_retryable_generic_decl(dag: &Dag, current: DeclarationId) -> bool {
    is_retryable_generic_decl_walk(dag, current, 0)
}

fn is_retryable_generic_decl_walk(dag: &Dag, current: DeclarationId, depth: usize) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => true,
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            is_retryable_generic_decl_walk(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation { arguments, .. } => arguments
            .iter()
            .any(|arg| is_retryable_generic_decl_walk(dag, arg.value, depth + 1)),
        TypeConnective::Cardinality { element, .. } => {
            is_retryable_generic_decl_walk(dag, *element, depth + 1)
        }
        TypeConnective::Conj { children } => children
            .iter()
            .any(|field| is_retryable_generic_decl_walk(dag, field.ty, depth + 1)),
        TypeConnective::Disj { variants } => variants
            .iter()
            .any(|field| is_retryable_generic_decl_walk(dag, field.ty, depth + 1)),
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Arrow { .. } => false,
    }
}

fn callable_template_arguments(
    dag: &Dag,
    target: DeclarationId,
) -> (DeclarationId, Vec<TemplateArgument>) {
    match &dag.declaration(target).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => (*template, arguments.clone()),
        _ => (target, Vec::new()),
    }
}

fn template_argument_value(
    arguments: &[TemplateArgument],
    parameter: DeclarationId,
) -> Option<DeclarationId> {
    arguments
        .iter()
        .find(|arg| arg.parameter == parameter)
        .map(|arg| arg.value)
}

fn resolve_template_argument_value(
    arguments: &[TemplateArgument],
    current: DeclarationId,
    depth: usize,
) -> DeclarationId {
    if depth >= WALK_DEPTH_LIMIT {
        return current;
    }
    let Some(next) = template_argument_value(arguments, current) else {
        return current;
    };
    if next == current {
        return current;
    }
    resolve_template_argument_value(arguments, next, depth + 1)
}

fn retained_template_arguments_for_target(
    dag: &Dag,
    template: DeclarationId,
    arguments: &[TemplateArgument],
) -> Vec<TemplateArgument> {
    let mut allowed: HashSet<DeclarationId> = dag
        .declaration(template)
        .type_params
        .iter()
        .copied()
        .collect();
    if let Some(raw_arrow) = resolve_arrow_decl_walk(dag, template, &mut SubstStack::new(), 0) {
        for input in raw_arrow.inputs {
            if declaration_is_callable(dag, input, 0) {
                allowed.insert(input);
            }
        }
    }

    let mut retained: Vec<TemplateArgument> = Vec::new();
    for argument in arguments {
        if !allowed.contains(&argument.parameter) {
            continue;
        }
        let resolved_value = resolve_template_argument_value(arguments, argument.value, 0);
        if let Some(existing) = retained
            .iter_mut()
            .find(|existing| existing.parameter == argument.parameter)
        {
            existing.value = resolved_value;
            continue;
        }
        retained.push(TemplateArgument {
            parameter: argument.parameter,
            value: resolved_value,
        });
    }
    retained
}

fn template_arguments_match(lhs: &[TemplateArgument], rhs: &[TemplateArgument]) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .zip(rhs.iter())
            .all(|(a, b)| a.parameter == b.parameter && a.value == b.value)
}

fn push_template_argument_binding(
    arguments: &mut Vec<TemplateArgument>,
    parameter: DeclarationId,
    value: DeclarationId,
) -> bool {
    for existing in arguments.iter_mut() {
        if existing.parameter == parameter {
            if existing.value == parameter {
                existing.value = value;
                return true;
            }
            return existing.value == value;
        }
    }
    arguments.push(TemplateArgument { parameter, value });
    true
}

fn resolve_arrow_decl_walk(
    dag: &Dag,
    current: DeclarationId,
    subst: &mut SubstStack,
    depth: usize,
) -> Option<ResolvedArrowDecl> {
    if depth >= WALK_DEPTH_LIMIT {
        return None;
    }
    let decl = dag.declaration(current);
    match &decl.connective {
        TypeConnective::Arrow { inputs, output, .. } => Some(ResolvedArrowDecl {
            inputs: inputs.clone(),
            output: *output,
        }),
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            subst.push(arguments.clone());
            let result = resolve_arrow_decl_walk(dag, *template, subst, depth + 1);
            subst.pop();
            result
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            resolve_arrow_decl_walk(dag, *next, subst, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
            let bound = subst.lookup(current)?;
            resolve_arrow_decl_walk(dag, bound, subst, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Cardinality { .. } => None,
    }
}

fn literal_decl_id(dag: &Dag, literal: &LiteralBits) -> Option<DeclarationId> {
    let name = match literal {
        LiteralBits::Int(_) => "Int",
        LiteralBits::Bool(_) => "Bool",
        LiteralBits::String(_) => "String",
    };
    dag.declaration_by_name(name).map(|decl| decl.id)
}

fn port_type_context(dag: &Dag, port: PortId) -> Option<PortTypeContext> {
    let resolved_decl = match dag.port(port).state() {
        PortState::Resolved(ty) => ty.declaration,
        PortState::Uninferred | PortState::Unresolved => return None,
    };
    let Some(produced_by) = dag.port(port).produced_by else {
        return Some(PortTypeContext {
            decl: resolved_decl,
            subst: SubstStack::new(),
        });
    };
    match dag.node(produced_by) {
        Behavior::Value(v) => Some(PortTypeContext {
            decl: literal_decl_id(dag, &v.data)?,
            subst: SubstStack::new(),
        }),
        Behavior::Transform(t) => match &t.target {
            TransformTarget::Callable(target) => {
                let CallableTargetResolution::Resolved {
                    template,
                    arguments,
                    ..
                } = resolve_callable_target(dag, *target, &t.inputs, &t.span)
                else {
                    return Some(PortTypeContext {
                        decl: resolved_decl,
                        subst: SubstStack::new(),
                    });
                };
                let mut subst = SubstStack::new();
                subst.push(arguments);
                let Some(arrow) = resolve_arrow_decl_walk(dag, template, &mut subst, 0) else {
                    return Some(PortTypeContext {
                        decl: resolved_decl,
                        subst: SubstStack::new(),
                    });
                };
                Some(PortTypeContext {
                    decl: arrow.output,
                    subst,
                })
            }
            TransformTarget::FieldProject {
                field_child: Some(field_child),
                ..
            } => {
                let decl = if is_retryable_generic_decl(dag, *field_child) {
                    resolved_decl
                } else {
                    *field_child
                };
                Some(PortTypeContext {
                    decl,
                    subst: SubstStack::new(),
                })
            }
            TransformTarget::FieldProject {
                field_child: None, ..
            } => None,
            TransformTarget::Operator(_) => Some(PortTypeContext {
                decl: resolved_decl,
                subst: SubstStack::new(),
            }),
        },
        Behavior::Branch(_) | Behavior::Loop(_) | Behavior::Bind(_) => Some(PortTypeContext {
            decl: resolved_decl,
            subst: SubstStack::new(),
        }),
    }
}

fn resolve_binding_decl(
    dag: &Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> Option<DeclarationId> {
    if depth >= WALK_DEPTH_LIMIT {
        return None;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => subst
            .lookup(current)
            .and_then(|bound| resolve_binding_decl(dag, bound, subst, depth + 1))
            .or_else(|| {
                walk_to_type_shape(dag, current, subst, depth + 1).map(|ty| ty.declaration)
            }),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            resolve_binding_decl(dag, *next, subst, depth + 1)
        }
        _ => walk_to_type_shape(dag, current, subst, depth + 1).map(|ty| ty.declaration),
    }
}

fn callable_signature_context(
    dag: &Dag,
    callable: DeclarationId,
) -> Option<CallableSignatureContext> {
    let decl = dag.declaration(callable);
    if let TypeConnective::Arrow {
        inputs,
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &decl.connective
    {
        let bind = dag.node(*bind_id).as_bind()?;
        if bind.params.len() < inputs.len() {
            return None;
        }
        let start = bind.params.len() - inputs.len();
        let mut input_contexts = Vec::with_capacity(inputs.len());
        for port in &bind.params[start..] {
            input_contexts.push(port_type_context(dag, *port)?);
        }
        return Some(CallableSignatureContext {
            inputs: input_contexts,
            output: port_type_context(dag, bind.value)?,
        });
    }

    let (template, arguments) = callable_template_arguments(dag, callable);
    let mut subst = SubstStack::new();
    subst.push(arguments);
    let raw_arrow = resolve_arrow_decl_walk(dag, template, &mut subst, 0)?;
    Some(CallableSignatureContext {
        inputs: raw_arrow
            .inputs
            .into_iter()
            .map(|decl| PortTypeContext {
                decl,
                subst: subst.clone(),
            })
            .collect(),
        output: PortTypeContext {
            decl: raw_arrow.output,
            subst,
        },
    })
}

fn bind_expected_callable_to_actual(
    dag: &Dag,
    expected_callable: DeclarationId,
    actual_callable: DeclarationId,
    arguments: &mut Vec<TemplateArgument>,
) -> CallableBindingResolution {
    let mut expected_subst = SubstStack::new();
    expected_subst.push(arguments.clone());
    let Some(expected_arrow) =
        resolve_arrow_decl_walk(dag, expected_callable, &mut expected_subst, 0)
    else {
        return CallableBindingResolution::Conflict;
    };
    let Some(actual_signature) = callable_signature_context(dag, actual_callable) else {
        return CallableBindingResolution::Retry;
    };
    if expected_arrow.inputs.len() != actual_signature.inputs.len() {
        return CallableBindingResolution::Conflict;
    }
    for (expected_input, actual_input) in expected_arrow
        .inputs
        .into_iter()
        .zip(actual_signature.inputs.iter())
    {
        if !bind_expected_decl_to_actual_context(dag, expected_input, actual_input, arguments, 0) {
            return CallableBindingResolution::Conflict;
        }
    }
    if !bind_expected_decl_to_actual_context(
        dag,
        expected_arrow.output,
        &actual_signature.output,
        arguments,
        0,
    ) {
        return CallableBindingResolution::Conflict;
    }
    CallableBindingResolution::Resolved
}

fn bind_expected_decl_to_actual_context(
    dag: &Dag,
    expected: DeclarationId,
    actual: &PortTypeContext,
    arguments: &mut Vec<TemplateArgument>,
    depth: usize,
) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }

    match &dag.declaration(expected).connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
            if let Some(bound) = template_argument_value(arguments, expected) {
                if bound != expected {
                    let expected_ctx = PortTypeContext {
                        decl: bound,
                        subst: SubstStack::new(),
                    };
                    return bind_expected_decl_to_actual_context(
                        dag,
                        expected_ctx.decl,
                        actual,
                        arguments,
                        depth + 1,
                    );
                }
            }
            let Some(value) = resolve_binding_decl(dag, actual.decl, &actual.subst, depth + 1)
            else {
                return false;
            };
            push_template_argument_binding(arguments, expected, value)
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            bind_expected_decl_to_actual_context(dag, *next, actual, arguments, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments: expected_args,
        } => {
            let actual_decl = match &dag.declaration(actual.decl).connective {
                TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                    let Some(bound) = actual.subst.lookup(actual.decl) else {
                        return false;
                    };
                    return bind_expected_decl_to_actual_context(
                        dag,
                        expected,
                        &PortTypeContext {
                            decl: bound,
                            subst: actual.subst.clone(),
                        },
                        arguments,
                        depth + 1,
                    );
                }
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                    return bind_expected_decl_to_actual_context(
                        dag,
                        expected,
                        &PortTypeContext {
                            decl: *next,
                            subst: actual.subst.clone(),
                        },
                        arguments,
                        depth + 1,
                    );
                }
                _ => actual.decl,
            };
            let TypeConnective::Instantiation {
                template: actual_template,
                arguments: actual_args,
            } = &dag.declaration(actual_decl).connective
            else {
                return false;
            };
            if *actual_template == expected && expected_args.len() == actual_args.len() {
                for (expected_arg, actual_arg) in expected_args.iter().zip(actual_args.iter()) {
                    if !bind_expected_decl_to_actual_context(
                        dag,
                        expected_arg.value,
                        &PortTypeContext {
                            decl: actual_arg.value,
                            subst: actual.subst.clone(),
                        },
                        arguments,
                        depth + 1,
                    ) {
                        return false;
                    }
                }
                return true;
            }
            if template != actual_template || expected_args.len() != actual_args.len() {
                return false;
            }
            for (expected_arg, actual_arg) in expected_args.iter().zip(actual_args.iter()) {
                if !bind_expected_decl_to_actual_context(
                    dag,
                    expected_arg.value,
                    &PortTypeContext {
                        decl: actual_arg.value,
                        subst: actual.subst.clone(),
                    },
                    arguments,
                    depth + 1,
                ) {
                    return false;
                }
            }
            true
        }
        TypeConnective::Cardinality { element, bound } => {
            let actual_decl = match &dag.declaration(actual.decl).connective {
                TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                    let Some(bound) = actual.subst.lookup(actual.decl) else {
                        return false;
                    };
                    return bind_expected_decl_to_actual_context(
                        dag,
                        expected,
                        &PortTypeContext {
                            decl: bound,
                            subst: actual.subst.clone(),
                        },
                        arguments,
                        depth + 1,
                    );
                }
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                    return bind_expected_decl_to_actual_context(
                        dag,
                        expected,
                        &PortTypeContext {
                            decl: *next,
                            subst: actual.subst.clone(),
                        },
                        arguments,
                        depth + 1,
                    );
                }
                _ => actual.decl,
            };
            let TypeConnective::Cardinality {
                element: actual_element,
                bound: actual_bound,
            } = &dag.declaration(actual_decl).connective
            else {
                return false;
            };
            if bound != actual_bound {
                return false;
            }
            bind_expected_decl_to_actual_context(
                dag,
                *element,
                &PortTypeContext {
                    decl: *actual_element,
                    subst: actual.subst.clone(),
                },
                arguments,
                depth + 1,
            )
        }
        _ => {
            let Some(expected_ty) =
                walk_to_type_shape(dag, expected, &SubstStack::new(), depth + 1)
            else {
                return false;
            };
            let Some(actual_ty) = walk_to_type_shape(dag, actual.decl, &actual.subst, depth + 1)
            else {
                return false;
            };
            expected_ty == actual_ty
        }
    }
}

fn callable_instantiation_conflict(
    dag: &Dag,
    target: DeclarationId,
    expected: DeclarationId,
    actual: &PortTypeContext,
    span: &SourceSpan,
) -> Diagnostic {
    let expected_name = target_display_name(dag, expected);
    let actual_name = resolve_binding_decl(dag, actual.decl, &actual.subst, 0)
        .map(|id| target_display_name(dag, id))
        .unwrap_or_else(|| format!("declaration#{}", actual.decl.raw()));
    Diagnostic::ResolveError {
        name: format!(
            "implicit template binding for `{}` conflicts while matching `{expected_name}` against `{actual_name}`",
            target_display_name(dag, target)
        ),
        span: span.clone(),
    }
}

fn resolve_callable_target(
    dag: &Dag,
    target: DeclarationId,
    runtime_inputs: &[PortId],
    span: &SourceSpan,
) -> CallableTargetResolution {
    let (template, mut arguments) = callable_template_arguments(dag, target);
    arguments = retained_template_arguments_for_target(dag, template, &arguments);
    let Some(signature) = resolve_direct_target_signature(dag, target, &arguments) else {
        let name = target_display_name(dag, target);
        return CallableTargetResolution::Fail(Diagnostic::ResolveError {
            name,
            span: span.clone(),
        });
    };
    let mut raw_subst = SubstStack::new();
    raw_subst.push(arguments.clone());
    let raw_arrow_inputs =
        resolve_arrow_decl_walk(dag, template, &mut raw_subst, 0).map(|arrow| arrow.inputs);

    let expected_runtime_arity = raw_arrow_inputs
        .as_ref()
        .map(|inputs| {
            inputs
                .iter()
                .filter(|input| !declaration_is_callable(dag, **input, 0))
                .count()
        })
        .unwrap_or_else(|| signature.inputs.len());
    if expected_runtime_arity != runtime_inputs.len() {
        return CallableTargetResolution::Fail(Diagnostic::ArityMismatch {
            function: target_display_name(dag, target),
            expected: expected_runtime_arity,
            actual: runtime_inputs.len(),
            span: span.clone(),
        });
    }

    let mut runtime_iter = runtime_inputs.iter();
    if let Some(raw_inputs) = raw_arrow_inputs {
        for expected_input in raw_inputs {
            if declaration_is_callable(dag, expected_input, 0) {
                let Some(actual_callable) = template_argument_value(&arguments, expected_input)
                else {
                    return CallableTargetResolution::Retry;
                };
                match bind_expected_callable_to_actual(
                    dag,
                    expected_input,
                    actual_callable,
                    &mut arguments,
                ) {
                    CallableBindingResolution::Resolved => {}
                    CallableBindingResolution::Retry => {
                        return CallableTargetResolution::Retry;
                    }
                    CallableBindingResolution::Conflict => {
                        return CallableTargetResolution::Fail(
                            Diagnostic::ResolveError {
                                name: format!(
                                    "callable argument to `{}` does not match the expected function type",
                                    target_display_name(dag, target)
                                ),
                                span: span.clone(),
                            },
                        );
                    }
                }
                continue;
            }
            let Some(input_port) = runtime_iter.next() else {
                break;
            };
            match dag.port(*input_port).state() {
                PortState::Uninferred => return CallableTargetResolution::Retry,
                PortState::Unresolved => {
                    return CallableTargetResolution::Fail(Diagnostic::ResolveError {
                        name: format!("(upstream failure in {})", target_display_name(dag, target)),
                        span: span.clone(),
                    });
                }
                PortState::Resolved(_) => {}
            }
            let Some(actual_ctx) = port_type_context(dag, *input_port) else {
                return CallableTargetResolution::Retry;
            };
            if !bind_expected_decl_to_actual_context(
                dag,
                expected_input,
                &actual_ctx,
                &mut arguments,
                0,
            ) {
                return CallableTargetResolution::Fail(callable_instantiation_conflict(
                    dag,
                    target,
                    expected_input,
                    &actual_ctx,
                    span,
                ));
            }
        }
    } else {
        for expected_input in &signature.inputs {
            if declaration_is_callable(dag, expected_input.declaration, 0) {
                let Some(actual_callable) =
                    template_argument_value(&arguments, expected_input.declaration)
                else {
                    return CallableTargetResolution::Retry;
                };
                match bind_expected_callable_to_actual(
                    dag,
                    expected_input.declaration,
                    actual_callable,
                    &mut arguments,
                ) {
                    CallableBindingResolution::Resolved => {}
                    CallableBindingResolution::Retry => {
                        return CallableTargetResolution::Retry;
                    }
                    CallableBindingResolution::Conflict => {
                        return CallableTargetResolution::Fail(Diagnostic::ResolveError {
                            name: format!(
                                "callable argument to `{}` does not match the expected function type",
                                target_display_name(dag, target)
                            ),
                            span: span.clone(),
                        });
                    }
                }
                continue;
            }
            let Some(actual_callable) =
                template_argument_value(&arguments, expected_input.declaration)
            else {
                let Some(input_port) = runtime_iter.next() else {
                    break;
                };
                match dag.port(*input_port).state() {
                    PortState::Uninferred => return CallableTargetResolution::Retry,
                    PortState::Unresolved => {
                        return CallableTargetResolution::Fail(Diagnostic::ResolveError {
                            name: format!(
                                "(upstream failure in {})",
                                target_display_name(dag, target)
                            ),
                            span: span.clone(),
                        });
                    }
                    PortState::Resolved(_) => {}
                }
                let Some(actual_ctx) = port_type_context(dag, *input_port) else {
                    return CallableTargetResolution::Retry;
                };
                if !bind_expected_decl_to_actual_context(
                    dag,
                    expected_input.declaration,
                    &actual_ctx,
                    &mut arguments,
                    0,
                ) {
                    return CallableTargetResolution::Fail(callable_instantiation_conflict(
                        dag,
                        target,
                        expected_input.declaration,
                        &actual_ctx,
                        span,
                    ));
                }
                continue;
            };
            match bind_expected_callable_to_actual(
                dag,
                expected_input.declaration,
                actual_callable,
                &mut arguments,
            ) {
                CallableBindingResolution::Resolved => {}
                CallableBindingResolution::Retry => {
                    return CallableTargetResolution::Retry;
                }
                CallableBindingResolution::Conflict => {
                    return CallableTargetResolution::Fail(Diagnostic::ResolveError {
                        name: format!(
                            "callable argument to `{}` does not match the expected function type",
                            target_display_name(dag, target)
                        ),
                        span: span.clone(),
                    });
                }
            }
        }
    }

    let Some(signature) = resolve_direct_target_signature(dag, target, &arguments) else {
        let name = target_display_name(dag, target);
        return CallableTargetResolution::Fail(Diagnostic::ResolveError {
            name,
            span: span.clone(),
        });
    };
    CallableTargetResolution::Resolved {
        template,
        arguments,
        signature,
    }
}

fn resolve_direct_target_signature(
    dag: &Dag,
    target: DeclarationId,
    arguments: &[TemplateArgument],
) -> Option<ResolvedArrow> {
    let (template, _) = callable_template_arguments(dag, target);
    let mut subst = SubstStack::new();
    subst.push(arguments.to_vec());
    if let Some(arrow) = resolve_arrow_walk(dag, template, &mut subst, 0) {
        return Some(arrow);
    }

    let TypeConnective::Conj { children } = &dag.declaration(template).connective else {
        return None;
    };
    let inputs = children
        .iter()
        .map(|child| signature_type_shape(dag, child.ty, &subst, 0))
        .collect::<Option<Vec<_>>>()?;
    Some(ResolvedArrow {
        inputs,
        output: TypeShape::new(enclosing_disj_for_variant(dag, template).unwrap_or(target)),
        // Variant constructor synthesis: `Variant(payload)` is direct
        // construction, not a function call with an executable body.
        // `NoBody` rather than `Pending` so the synthesized signature
        // semantically matches the "no body by construction" cases in
        // declarations, even though this `ResolvedArrow` is transient
        // inference state (never stored in `Dag.declarations`).
        body: ArrowBody::NoBody,
    })
}

fn resolve_callable_targets(dag: &mut Dag) -> bool {
    struct Rewrite {
        node_index: usize,
        template: DeclarationId,
        arguments: Vec<TemplateArgument>,
    }

    let mut rewrites: Vec<Rewrite> = Vec::new();
    for (node_index, node) in dag.nodes().iter().enumerate() {
        let Behavior::Transform(t) = node else {
            continue;
        };
        let TransformTarget::Callable(target) = t.target else {
            continue;
        };
        let CallableTargetResolution::Resolved {
            template,
            arguments,
            ..
        } = resolve_callable_target(dag, target, &t.inputs, &t.span)
        else {
            continue;
        };
        let (current_template, current_arguments) = callable_template_arguments(dag, target);
        if current_template == template && template_arguments_match(&current_arguments, &arguments)
        {
            continue;
        }
        if arguments.is_empty() && current_template == template {
            continue;
        }
        rewrites.push(Rewrite {
            node_index,
            template,
            arguments,
        });
    }

    let mut changed = false;
    for rewrite in rewrites {
        let new_target = if rewrite.arguments.is_empty() {
            rewrite.template
        } else {
            let instantiation_id = dag.alloc_declaration_id();
            dag.push_declaration(crate::dag::Declaration {
                id: instantiation_id,
                name: None,
                connective: TypeConnective::Instantiation {
                    template: rewrite.template,
                    arguments: rewrite.arguments,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                span: synthetic_span(),
            });
            instantiation_id
        };
        let Behavior::Transform(t) = &mut dag.nodes_mut()[rewrite.node_index] else {
            continue;
        };
        let TransformTarget::Callable(target) = &mut t.target else {
            continue;
        };
        if *target != new_target {
            *target = new_target;
            changed = true;
        }
    }
    changed
}

fn materialize_callable_signature_instantiations(dag: &mut Dag) -> bool {
    let requests: Vec<(DeclarationId, Vec<TemplateArgument>)> = dag
        .nodes()
        .iter()
        .filter_map(|node| {
            let Behavior::Transform(t) = node else {
                return None;
            };
            let TransformTarget::Callable(target) = t.target else {
                return None;
            };
            let (template, arguments) = callable_template_arguments(dag, target);
            (!arguments.is_empty()).then_some((template, arguments))
        })
        .collect();

    let initial_count = dag.declarations().len();
    for (template, arguments) in requests {
        let mut subst = SubstStack::new();
        subst.push(arguments);
        let Some(raw_arrow) = resolve_arrow_decl_walk(dag, template, &mut subst, 0) else {
            continue;
        };
        for input in raw_arrow.inputs {
            let _ = concretize_decl_with_subst(dag, input, &subst, 0);
        }
        let _ = concretize_decl_with_subst(dag, raw_arrow.output, &subst, 0);
    }
    dag.declarations().len() != initial_count
}

fn resolve_lambda_parameter_types(dag: &mut Dag) -> bool {
    struct Rewrite {
        port: PortId,
        ty: TypeShape,
    }

    let mut rewrites: Vec<Rewrite> = Vec::new();

    for node in dag.nodes() {
        let Behavior::Transform(t) = node else {
            continue;
        };
        let TransformTarget::Callable(target) = t.target else {
            continue;
        };
        let (template, arguments) = match resolve_callable_target(dag, target, &t.inputs, &t.span) {
            CallableTargetResolution::Resolved {
                template,
                arguments,
                ..
            } => (template, arguments),
            CallableTargetResolution::Retry => {
                bind_non_callable_target_arguments(dag, target, &t.inputs)
                    .unwrap_or_else(|| callable_template_arguments(dag, target))
            }
            CallableTargetResolution::Fail(_) => continue,
        };
        let mut outer_subst = SubstStack::new();
        outer_subst.push(arguments.clone());
        let Some(raw_arrow) = resolve_arrow_decl_walk(dag, template, &mut SubstStack::new(), 0)
        else {
            continue;
        };
        for expected_input in raw_arrow.inputs {
            if !declaration_is_callable(dag, expected_input, 0) {
                continue;
            }
            let Some(actual_callable) = template_argument_value(&arguments, expected_input) else {
                continue;
            };
            let Some(expected_signature) =
                resolve_arrow_walk(dag, expected_input, &mut outer_subst.clone(), 0)
            else {
                continue;
            };
            let TypeConnective::Arrow { body, .. } = &dag.declaration(actual_callable).connective
            else {
                continue;
            };
            let ArrowBody::UserDefined(bind_id) = body else {
                continue;
            };
            let Some(bind) = dag.node(*bind_id).as_bind() else {
                continue;
            };
            if bind.params.len() < expected_signature.inputs.len() {
                continue;
            }
            let start = bind.params.len() - expected_signature.inputs.len();
            for (port, ty) in bind.params[start..]
                .iter()
                .copied()
                .zip(expected_signature.inputs.iter().copied())
            {
                rewrites.push(Rewrite { port, ty });
            }
        }
    }

    let mut changed = false;
    for rewrite in rewrites {
        let current = dag.port(rewrite.port).state().clone();
        if matches!(current, PortState::Resolved(existing) if existing == rewrite.ty) {
            continue;
        }
        if matches!(current, PortState::Unresolved) {
            continue;
        }
        dag.set_port_type(rewrite.port, rewrite.ty);
        changed = true;
    }
    changed
}

fn validate_user_defined_function_signatures(dag: &mut Dag) -> bool {
    struct Failure {
        port: PortId,
        diagnostic: Diagnostic,
    }

    let user_defined_arrows: Vec<(
        DeclarationId,
        Vec<DeclarationId>,
        DeclarationId,
        crate::dag::NodeId,
    )> = dag
        .declarations()
        .iter()
        .filter_map(|decl| match &decl.connective {
            TypeConnective::Arrow {
                inputs,
                output,
                body: ArrowBody::UserDefined(bind_id),
            } => Some((decl.id, inputs.clone(), *output, *bind_id)),
            _ => None,
        })
        .collect();

    let mut failures = Vec::new();

    'declarations: for (decl_id, inputs, output, bind_id) in user_defined_arrows {
        let Some(bind) = dag.node(bind_id).as_bind() else {
            continue;
        };
        if matches!(dag.port(bind.value).state(), PortState::Unresolved) {
            continue;
        }
        if bind.params.len() < inputs.len() {
            failures.push(Failure {
                port: bind.value,
                diagnostic: Diagnostic::ResolveError {
                    name: format!(
                        "function `{}` body does not satisfy its declared signature",
                        dag.declaration(decl_id)
                            .name
                            .as_deref()
                            .unwrap_or("<anonymous>")
                    ),
                    span: bind.span.clone(),
                },
            });
            continue;
        }

        let start = bind.params.len() - inputs.len();
        let mut arguments = Vec::new();
        for (index, (port, expected_decl)) in bind.params[start..]
            .iter()
            .copied()
            .zip(inputs.iter().copied())
            .enumerate()
        {
            if declaration_is_callable(dag, expected_decl, 0) {
                continue;
            }
            match dag.port(port).state() {
                PortState::Uninferred | PortState::Unresolved => continue 'declarations,
                PortState::Resolved(_) => {}
            }
            let Some(actual_ctx) = port_type_context(dag, port) else {
                continue 'declarations;
            };
            if bind_expected_decl_to_actual_context(
                dag,
                expected_decl,
                &actual_ctx,
                &mut arguments,
                0,
            ) {
                continue;
            }

            failures.push(Failure {
                port: bind.value,
                diagnostic: Diagnostic::ResolveError {
                    name: format!(
                        "function `{}` parameter {} does not satisfy its declared signature",
                        dag.declaration(decl_id)
                            .name
                            .as_deref()
                            .unwrap_or("<anonymous>"),
                        index + 1
                    ),
                    span: bind.span.clone(),
                },
            });
            continue 'declarations;
        }

        match dag.port(bind.value).state() {
            PortState::Uninferred | PortState::Unresolved => continue,
            PortState::Resolved(_) => {}
        }
        if declaration_is_callable(dag, output, 0) {
            continue;
        }
        let Some(actual_ctx) = port_type_context(dag, bind.value) else {
            continue;
        };
        if bind_expected_decl_to_actual_context(dag, output, &actual_ctx, &mut arguments, 0) {
            continue;
        }

        failures.push(Failure {
            port: bind.value,
            diagnostic: Diagnostic::ResolveError {
                name: format!(
                    "function `{}` body does not satisfy its declared return type",
                    dag.declaration(decl_id)
                        .name
                        .as_deref()
                        .unwrap_or("<anonymous>")
                ),
                span: bind.span.clone(),
            },
        });
    }

    let mut changed = false;
    for failure in failures {
        if matches!(dag.port(failure.port).state(), PortState::Unresolved) {
            continue;
        }
        dag.mark_unresolved(failure.port, failure.diagnostic);
        changed = true;
    }
    changed
}

fn bind_non_callable_target_arguments(
    dag: &Dag,
    target: DeclarationId,
    runtime_inputs: &[PortId],
) -> Option<(DeclarationId, Vec<TemplateArgument>)> {
    let (template, mut arguments) = callable_template_arguments(dag, target);
    let signature = resolve_direct_target_signature(dag, target, &arguments)?;
    let mut raw_subst = SubstStack::new();
    raw_subst.push(arguments.clone());
    let raw_arrow_inputs =
        resolve_arrow_decl_walk(dag, template, &mut raw_subst, 0).map(|arrow| arrow.inputs);

    let expected_runtime_arity = raw_arrow_inputs
        .as_ref()
        .map(|inputs| {
            inputs
                .iter()
                .filter(|input| !declaration_is_callable(dag, **input, 0))
                .count()
        })
        .unwrap_or_else(|| signature.inputs.len());
    if expected_runtime_arity != runtime_inputs.len() {
        return None;
    }

    let mut runtime_iter = runtime_inputs.iter();
    if let Some(raw_inputs) = raw_arrow_inputs {
        for expected_input in raw_inputs {
            if declaration_is_callable(dag, expected_input, 0) {
                continue;
            }
            let input_port = *runtime_iter.next()?;
            let actual_ctx = port_type_context(dag, input_port)?;
            if !bind_expected_decl_to_actual_context(
                dag,
                expected_input,
                &actual_ctx,
                &mut arguments,
                0,
            ) {
                return None;
            }
        }
    } else {
        for expected_input in &signature.inputs {
            if declaration_is_callable(dag, expected_input.declaration, 0) {
                continue;
            }
            let input_port = *runtime_iter.next()?;
            let actual_ctx = port_type_context(dag, input_port)?;
            if !bind_expected_decl_to_actual_context(
                dag,
                expected_input.declaration,
                &actual_ctx,
                &mut arguments,
                0,
            ) {
                return None;
            }
        }
    }

    Some((template, arguments))
}

fn walk_to_conj_decl_with_subst(
    dag: &Dag,
    start: DeclarationId,
    subst: &mut SubstStack,
) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                subst.push(arguments.clone());
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                current = *next;
            }
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                current = subst.lookup(current)?;
            }
            _ => return None,
        }
    }
    None
}

fn walk_to_disj_decl_with_subst(
    dag: &Dag,
    start: DeclarationId,
    subst: &mut SubstStack,
) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Disj { .. } => return Some(current),
            TypeConnective::Cardinality {
                bound: crate::dag::CardinalityBound::AtMostOne,
                ..
            } => return existing_optional_match_disj_decl(dag, current),
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                subst.push(arguments.clone());
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                current = *next;
            }
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                current = subst.lookup(current)?;
            }
            _ => return None,
        }
    }
    None
}

fn enclosing_disj_for_variant(dag: &Dag, variant_decl_id: DeclarationId) -> Option<DeclarationId> {
    dag.declarations().iter().find_map(|decl| {
        let TypeConnective::Disj { variants } = &decl.connective else {
            return None;
        };
        variants
            .iter()
            .find(|variant| variant.ty == variant_decl_id)
            .map(|_| decl.id)
    })
}

fn resolve_payload_binding_type(
    dag: &Dag,
    variant_decl_id: DeclarationId,
    subst: &SubstStack,
    variant_name: &str,
    binding_name: &str,
    span: &SourceSpan,
) -> Result<PayloadBindingResolution, Diagnostic> {
    match &dag.declaration(variant_decl_id).connective {
        TypeConnective::Conj { children } if children.is_empty() => {
            Err(Diagnostic::ResolveError {
                name: format!(
                    "variant `{variant_name}` does not carry a payload and cannot bind `{binding_name}`"
                ),
                span: span.clone(),
            })
        }
        TypeConnective::Conj { children }
            if children.len() == 1 && children[0].label.as_str() == "_0" =>
        {
            let payload_type_decl = children[0].ty;
            walk_to_type_shape(dag, payload_type_decl, subst, 0)
                .map(PayloadBindingResolution::Direct)
                .ok_or_else(|| Diagnostic::ResolveError {
                    name: format!(
                        "variant `{variant_name}` payload does not resolve to a port type for binding `{binding_name}`"
                    ),
                    span: span.clone(),
                })
        }
        TypeConnective::Conj { .. } => Ok(PayloadBindingResolution::SpecializedRecord {
            variant_decl_id,
            subst: subst.clone(),
        }),
        _ => {
            Err(Diagnostic::ResolveError {
                name: format!(
                    "variant `{variant_name}` does not lower to a payload Conj and cannot bind `{binding_name}`"
                ),
                span: span.clone(),
            })
        }
    }
}

fn materialize_specialized_payload_record(
    dag: &mut Dag,
    variant_decl_id: DeclarationId,
    subst: &SubstStack,
) -> TypeShape {
    let variant_decl = dag.declaration(variant_decl_id).clone();
    let TypeConnective::Conj { children } = variant_decl.connective else {
        return TypeShape::new(variant_decl_id);
    };
    let specialized_children: Vec<Field> = children
        .into_iter()
        .map(|field| Field {
            label: field.label,
            ty: concretize_decl_with_subst(dag, field.ty, subst, 0),
        })
        .collect();
    if let Some(existing) = find_equivalent_anonymous_conj(dag, &specialized_children) {
        return TypeShape::new(existing);
    }
    let id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id,
        name: None,
        connective: TypeConnective::Conj {
            children: specialized_children,
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: variant_decl.span,
    });
    TypeShape::new(id)
}

fn concretize_decl_with_subst(
    dag: &mut Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> DeclarationId {
    if depth >= WALK_DEPTH_LIMIT {
        return current;
    }
    let decl = dag.declaration(current).clone();
    match decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => subst.lookup(current).unwrap_or(current),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            concretize_decl_with_subst(dag, next, subst, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let specialized_arguments: Vec<TemplateArgument> = arguments
                .into_iter()
                .map(|arg| TemplateArgument {
                    parameter: arg.parameter,
                    value: concretize_decl_with_subst(dag, arg.value, subst, depth + 1),
                })
                .collect();
            if let Some(existing) =
                find_equivalent_anonymous_instantiation(dag, template, &specialized_arguments)
            {
                return existing;
            }
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Instantiation {
                    template,
                    arguments: specialized_arguments,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                span: decl.span,
            });
            id
        }
        TypeConnective::Cardinality { element, bound } => {
            let specialized_element = concretize_decl_with_subst(dag, element, subst, depth + 1);
            if let Some(existing) =
                find_equivalent_anonymous_cardinality(dag, specialized_element, &bound)
            {
                return existing;
            }
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Cardinality {
                    element: specialized_element,
                    bound,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                span: decl.span,
            });
            id
        }
        _ => current,
    }
}

fn find_equivalent_anonymous_conj(dag: &Dag, children: &[Field]) -> Option<DeclarationId> {
    dag.declarations().iter().find_map(|decl| {
        if decl.name.is_some() {
            return None;
        }
        let TypeConnective::Conj { children: existing } = &decl.connective else {
            return None;
        };
        (existing.len() == children.len()
            && existing
                .iter()
                .zip(children.iter())
                .all(|(lhs, rhs)| lhs.label == rhs.label && lhs.ty == rhs.ty))
        .then_some(decl.id)
    })
}

fn find_equivalent_anonymous_instantiation(
    dag: &Dag,
    template: DeclarationId,
    arguments: &[TemplateArgument],
) -> Option<DeclarationId> {
    dag.declarations().iter().find_map(|decl| {
        if decl.name.is_some() {
            return None;
        }
        let TypeConnective::Instantiation {
            template: existing_template,
            arguments: existing_arguments,
        } = &decl.connective
        else {
            return None;
        };
        (template == *existing_template
            && existing_arguments.len() == arguments.len()
            && existing_arguments
                .iter()
                .zip(arguments.iter())
                .all(|(lhs, rhs)| lhs.parameter == rhs.parameter && lhs.value == rhs.value))
        .then_some(decl.id)
    })
}

fn find_equivalent_anonymous_cardinality(
    dag: &Dag,
    element: DeclarationId,
    bound: &crate::dag::CardinalityBound,
) -> Option<DeclarationId> {
    dag.declarations().iter().find_map(|decl| {
        if decl.name.is_some() {
            return None;
        }
        let TypeConnective::Cardinality {
            element: existing_element,
            bound: existing_bound,
        } = &decl.connective
        else {
            return None;
        };
        (element == *existing_element && existing_bound == bound).then_some(decl.id)
    })
}

enum FieldProjectResolution {
    Retry,
    Fail(Diagnostic),
    Resolved {
        field_child: DeclarationId,
        output_ty: TypeShape,
    },
}

fn resolve_field_project(
    dag: &Dag,
    t: &TransformNode,
    field_label: &str,
) -> FieldProjectResolution {
    if t.inputs.len() != 1 {
        return FieldProjectResolution::Fail(Diagnostic::ArityMismatch {
            function: format!(".{field_label}"),
            expected: 1,
            actual: t.inputs.len(),
            span: t.span.clone(),
        });
    }

    let input_ty = match dag.port(t.inputs[0]).state() {
        PortState::Uninferred => return FieldProjectResolution::Retry,
        PortState::Unresolved => {
            return FieldProjectResolution::Fail(Diagnostic::ResolveError {
                name: format!("(upstream failure in field `{field_label}`)"),
                span: t.span.clone(),
            })
        }
        PortState::Resolved(ty) => *ty,
    };

    let mut subst = SubstStack::new();
    let Some(actual_conj_id) = walk_to_conj_decl_with_subst(dag, input_ty.declaration, &mut subst)
    else {
        return FieldProjectResolution::Fail(Diagnostic::ResolveError {
            name: format!(
                "field `{field_label}` cannot be projected from `{}` because it does not walk to a Conj type",
                target_display_name(dag, input_ty.declaration),
            ),
            span: t.span.clone(),
        });
    };

    let children = match &dag.declaration(actual_conj_id).connective {
        TypeConnective::Conj { children } => children,
        _ => unreachable!("walk_to_conj_decl returned a non-Conj declaration"),
    };
    let field_decl_id = if let TransformTarget::FieldProject {
        field_child: Some(field_child),
        ..
    } = &t.target
    {
        *field_child
    } else {
        let Some(field_decl_id) = children
            .iter()
            .find(|field| field.label == field_label)
            .map(|field| field.ty)
        else {
            return FieldProjectResolution::Fail(Diagnostic::ResolveError {
                name: format!(
                    "field `{field_label}` does not exist on `{}`",
                    target_display_name(dag, input_ty.declaration),
                ),
                span: t.span.clone(),
            });
        };
        field_decl_id
    };
    let Some(output_ty) = walk_to_type_shape(dag, field_decl_id, &subst, 0) else {
        return FieldProjectResolution::Fail(Diagnostic::ResolveError {
            name: format!(
                "field `{field_label}` on `{}` does not resolve to a port type",
                target_display_name(dag, input_ty.declaration),
            ),
            span: t.span.clone(),
        });
    };

    FieldProjectResolution::Resolved {
        field_child: field_decl_id,
        output_ty,
    }
}

fn decide_field_project(
    dag: &Dag,
    t: &TransformNode,
    field_label: &str,
    _field_child: Option<DeclarationId>,
) -> Decision {
    match resolve_field_project(dag, t, field_label) {
        FieldProjectResolution::Retry => Decision::Retry,
        FieldProjectResolution::Fail(diag) => Decision::Fail(t.output, diag),
        FieldProjectResolution::Resolved { output_ty, .. } => Decision::Set(t.output, output_ty),
    }
}

fn resolve_field_project_targets(dag: &mut Dag) -> bool {
    let mut rewrites: Vec<(usize, DeclarationId)> = Vec::new();
    for (node_index, node) in dag.nodes().iter().enumerate() {
        let Behavior::Transform(t) = node else {
            continue;
        };
        let TransformTarget::FieldProject {
            field_label,
            field_child: None,
        } = &t.target
        else {
            continue;
        };
        let FieldProjectResolution::Resolved { field_child, .. } =
            resolve_field_project(dag, t, field_label)
        else {
            continue;
        };
        rewrites.push((node_index, field_child));
    }

    let mut changed = false;
    for (node_index, field_child) in rewrites {
        let Behavior::Transform(t) = &mut dag.nodes_mut()[node_index] else {
            continue;
        };
        let TransformTarget::FieldProject {
            field_child: slot, ..
        } = &mut t.target
        else {
            continue;
        };
        if slot.is_none() {
            *slot = Some(field_child);
            changed = true;
        }
    }
    changed
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
                    return read_algebra_field(dag, decl, field.ty, source_id, op_kind, lhs_type);
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
            let mut input_shapes: Vec<TypeShape> = Vec::new();
            for id in inputs {
                if subst.lookup(*id).is_some() && declaration_is_callable(dag, *id, depth + 1) {
                    continue;
                }
                input_shapes.push(signature_type_shape(dag, *id, subst, depth + 1)?);
            }
            let output_shape = signature_type_shape(dag, *output, subst, depth + 1)?;
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
/// declarations until it finds a stable type identity:
///
/// - named top-level declarations keep their own `DeclarationId`
/// - anonymous instantiations keep THEIR instantiation id so
///   `List<Int>` and bare `List` do not collapse to the same shape
/// - type params resolve through the substitution stack when bound
///
/// There is no name-keyed bridge back to a coarse primitive tag — the
/// declaration graph IS the type identity.
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
            if let Some(bound) = subst.lookup(current) {
                walk_to_type_shape(dag, bound, subst, depth + 1)
            } else {
                Some(TypeShape::new(current))
            }
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            walk_to_type_shape(dag, *next, subst, depth + 1)
        }
        TypeConnective::Instantiation { .. } => {
            resolve_decl_with_subst(dag, current, subst, depth + 1)
                .map(TypeShape::new)
                .or_else(|| Some(TypeShape::new(current)))
        }
        // Terminal non-follow cases. An anonymous `UnresolvedIdentifier`
        // means the sweep did not resolve the reference — the phantom
        // diagnostic is already attached, and this walk fails so the
        // caller can surface it. Anonymous optionals still need port
        // identities today because reflected substrate fields such as
        // `DagPort.produced_by: NodeId?` are legal field-project outputs.
        // The broader anonymous structural cases stay fail-closed until
        // port-type extension work admits them deliberately.
        // Enumerated explicitly (rather than `_ => None`) so any
        // future variant forces consideration here.
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_)) => None,
        TypeConnective::Atom(AtomPayload::Literal(_)) => None,
        TypeConnective::Conj { .. } => None,
        TypeConnective::Disj { .. } => None,
        TypeConnective::Arrow { .. } => None,
        TypeConnective::Cardinality { .. } => Some(TypeShape::new(current)),
    }
}

fn signature_type_shape(
    dag: &Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> Option<TypeShape> {
    if depth >= WALK_DEPTH_LIMIT {
        return None;
    }
    let decl = dag.declaration(current);
    if decl.name.is_some() {
        return Some(TypeShape::new(current));
    }
    match &decl.connective {
        TypeConnective::Instantiation { .. } => {
            resolve_decl_with_subst(dag, current, subst, depth + 1)
                .map(TypeShape::new)
                .or_else(|| Some(TypeShape::new(current)))
        }
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
            if let Some(bound) = subst.lookup(current) {
                signature_type_shape(dag, bound, subst, depth + 1)
            } else {
                Some(TypeShape::new(current))
            }
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            signature_type_shape(dag, *next, subst, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_)) => None,
        TypeConnective::Atom(AtomPayload::Literal(_)) => None,
        TypeConnective::Conj { .. } => None,
        TypeConnective::Disj { .. } => None,
        TypeConnective::Arrow { .. } => None,
        // Cardinality-typed signatures (e.g., `fn port(d, id) -> DagPort?`)
        // keep the anonymous Cardinality declaration id as the port's type
        // identity. Mirrors `walk_to_type_shape`'s Cardinality case —
        // optional returns are legal throughout the substrate.
        TypeConnective::Cardinality { .. } => Some(TypeShape::new(current)),
    }
}

fn resolve_decl_with_subst(
    dag: &Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> Option<DeclarationId> {
    if depth >= WALK_DEPTH_LIMIT {
        return None;
    }
    let decl = dag.declaration(current);
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => subst
            .lookup(current)
            .and_then(|bound| resolve_decl_with_subst(dag, bound, subst, depth + 1))
            .or(Some(current)),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            resolve_decl_with_subst(dag, *next, subst, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let specialized_arguments: Vec<TemplateArgument> = arguments
                .iter()
                .map(|arg| {
                    Some(TemplateArgument {
                        parameter: arg.parameter,
                        value: resolve_decl_with_subst(dag, arg.value, subst, depth + 1)?,
                    })
                })
                .collect::<Option<_>>()?;
            if specialized_arguments
                .iter()
                .zip(arguments.iter())
                .all(|(lhs, rhs)| lhs.parameter == rhs.parameter && lhs.value == rhs.value)
            {
                return Some(current);
            }
            find_equivalent_decl_instantiation(dag, *template, &specialized_arguments)
                .or(Some(current))
        }
        TypeConnective::Cardinality { element, bound } => {
            let specialized_element = resolve_decl_with_subst(dag, *element, subst, depth + 1)?;
            if specialized_element == *element {
                return Some(current);
            }
            find_equivalent_decl_cardinality(dag, specialized_element, bound).or(Some(current))
        }
        _ => Some(current),
    }
}

fn find_equivalent_decl_instantiation(
    dag: &Dag,
    template: DeclarationId,
    arguments: &[TemplateArgument],
) -> Option<DeclarationId> {
    dag.declarations().iter().find_map(|decl| {
        let TypeConnective::Instantiation {
            template: existing_template,
            arguments: existing_arguments,
        } = &decl.connective
        else {
            return None;
        };
        (template == *existing_template
            && existing_arguments.len() == arguments.len()
            && existing_arguments
                .iter()
                .zip(arguments.iter())
                .all(|(lhs, rhs)| lhs.parameter == rhs.parameter && lhs.value == rhs.value))
        .then_some(decl.id)
    })
}

fn find_equivalent_decl_cardinality(
    dag: &Dag,
    element: DeclarationId,
    bound: &crate::dag::CardinalityBound,
) -> Option<DeclarationId> {
    dag.declarations().iter().find_map(|decl| {
        let TypeConnective::Cardinality {
            element: existing_element,
            bound: existing_bound,
        } = &decl.connective
        else {
            return None;
        };
        (element == *existing_element && existing_bound == bound).then_some(decl.id)
    })
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
        TransformTarget::FieldProject { field_label, .. } => format!(".{field_label}"),
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

fn type_shapes_equivalent(dag: &Dag, lhs: &TypeShape, rhs: &TypeShape) -> bool {
    if lhs == rhs {
        return true;
    }
    declaration_shapes_equivalent(dag, lhs.declaration, rhs.declaration, 0)
}

fn declaration_shapes_equivalent(
    dag: &Dag,
    lhs: DeclarationId,
    rhs: DeclarationId,
    depth: usize,
) -> bool {
    if lhs == rhs {
        return true;
    }
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    let lhs_decl = dag.declaration(lhs);
    let rhs_decl = dag.declaration(rhs);
    match (&lhs_decl.connective, &rhs_decl.connective) {
        (TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)), _) => {
            declaration_shapes_equivalent(dag, *next, rhs, depth + 1)
        }
        (_, TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next))) => {
            declaration_shapes_equivalent(dag, lhs, *next, depth + 1)
        }
        (
            TypeConnective::Instantiation {
                template: lhs_template,
                arguments: lhs_arguments,
            },
            TypeConnective::Instantiation {
                template: rhs_template,
                arguments: rhs_arguments,
            },
        ) => {
            declaration_shapes_equivalent(dag, *lhs_template, *rhs_template, depth + 1)
                && lhs_arguments.len() == rhs_arguments.len()
                && lhs_arguments
                    .iter()
                    .zip(rhs_arguments.iter())
                    .all(|(lhs_arg, rhs_arg)| {
                        declaration_shapes_equivalent(dag, lhs_arg.value, rhs_arg.value, depth + 1)
                    })
        }
        (
            TypeConnective::Cardinality {
                element: lhs_element,
                bound: lhs_bound,
            },
            TypeConnective::Cardinality {
                element: rhs_element,
                bound: rhs_bound,
            },
        ) => {
            lhs_bound == rhs_bound
                && declaration_shapes_equivalent(dag, *lhs_element, *rhs_element, depth + 1)
        }
        (
            TypeConnective::Conj {
                children: lhs_children,
            },
            TypeConnective::Conj {
                children: rhs_children,
            },
        ) => {
            lhs_children.len() == rhs_children.len()
                && lhs_children
                    .iter()
                    .zip(rhs_children.iter())
                    .all(|(lhs_child, rhs_child)| {
                        lhs_child.label == rhs_child.label
                            && declaration_shapes_equivalent(
                                dag,
                                lhs_child.ty,
                                rhs_child.ty,
                                depth + 1,
                            )
                    })
        }
        (
            TypeConnective::Disj {
                variants: lhs_variants,
            },
            TypeConnective::Disj {
                variants: rhs_variants,
            },
        ) => {
            lhs_variants.len() == rhs_variants.len()
                && lhs_variants
                    .iter()
                    .zip(rhs_variants.iter())
                    .all(|(lhs_variant, rhs_variant)| {
                        lhs_variant.label == rhs_variant.label
                            && declaration_shapes_equivalent(
                                dag,
                                lhs_variant.ty,
                                rhs_variant.ty,
                                depth + 1,
                            )
                    })
        }
        _ => false,
    }
}
