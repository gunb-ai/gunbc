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
//
// **M1 extension — inference mutates `Dag.declarations`:** operator / algebra
// resolution may `push_declaration` for anonymous `Instantiation` shapes (e.g.
// `Result<T, DivError>` for totalizing `div`); `find_equivalent_anonymous_instantiation`
// deduplicates so fixpoint growth stays bounded. `decide` takes `&mut Dag` and clones
// each `Behavior` before matching so borrows stay sound — inference is not read-only
// over the declaration table.

use std::collections::HashSet;

use crate::dag::{
    substrate_div_error_type_decl_suppressed_for_emit,
    substrate_result_type_decl_suppressed_for_emit, ArithmeticOp, ArrowBody, AtomPayload, Behavior,
    BindNode, BindNodeId, CardinalityBound, Dag, Declaration, DeclarationId, Field, LiteralBits,
    Lookup, NominalOpacity, PhantomParameter, PortId, PortState, TemplateArgument, TransformNode,
    TransformTarget, TypeConnective,
};
use crate::diagnostics::{
    declaration_display_name, example_source_for_decl, witness_correction_for_decl, Correction,
    Diagnostic, SourceSpan,
};
use crate::infer_helpers::{
    behavior_output_port, behavior_span, generated_template_arguments_match,
    normalize_instantiation_arguments, payload_binding_span as generated_payload_binding_span,
    push_template_argument_binding as generated_push_template_argument_binding,
    resolve_template_argument_value as generated_resolve_template_argument_value,
    template_argument_value as generated_template_argument_value, NormalizedInstantiationArgs,
    TemplateArgumentBinding, TemplateArgumentsMatch,
};
use std::str::FromStr;

use num_bigint::BigInt;

use crate::int_literal_ranges::{
    int_literal_fits_expected_type, integer_range_for_decl, literal_bigint_at,
    magnitude_out_of_range_for_interval, IntegerRangeLookup,
};
use crate::lower::{clone_predicate_body, outer_predicate_slots};
use crate::operators::{LogicalOp, OperatorKind};
use crate::types::TypeShape;

/// Outcome of [`try_reconcile_int_literal_decision_set`].
enum IntLiteralSetReconciliation {
    /// Reconciled without changing the port (narrow annotation already wins over default `Int`).
    Keep,
    /// Applied `set_port_type` to narrow a default-`Int` literal port to `ty`.
    SetNarrow,
    /// `mark_unresolved` with a magnitude or other diagnostic.
    Marked,
}

/// Reconcile `Decision::Set` for ports backed by an integer literal when the proposed
/// and existing [`TypeShape`] would otherwise `TypeMismatch`: (a) `let`/`data`
/// pre-seed vs default `Int`, (b) call-sites that propose a callee's narrow
/// type over a default-`Int` argument.
fn try_reconcile_int_literal_decision_set(
    dag: &mut Dag,
    port: PortId,
    existing: TypeShape,
    ty: TypeShape,
) -> Option<IntLiteralSetReconciliation> {
    let default_int = dag.int_shape()?;
    let literal = literal_bigint_at(dag, port)?;
    if type_shapes_equivalent(dag, &ty, &default_int) {
        match int_literal_fits_expected_type(dag, &literal, existing.declaration) {
            Ok(Some(true)) => Some(IntLiteralSetReconciliation::Keep),
            Ok(Some(false)) => {
                if let IntegerRangeLookup::Found(range) =
                    integer_range_for_decl(dag, existing.declaration)
                {
                    let span = node_span_for_port(dag, port).unwrap_or_else(synthetic_span);
                    dag.mark_unresolved(
                        port,
                        magnitude_out_of_range_for_interval(&literal, existing, range, span),
                    );
                    Some(IntLiteralSetReconciliation::Marked)
                } else {
                    None
                }
            }
            Ok(None) => None,
            Err(diag) => {
                if !matches!(dag.port(port).state(), PortState::Unresolved) {
                    dag.mark_unresolved(port, diag);
                }
                Some(IntLiteralSetReconciliation::Marked)
            }
        }
    } else if type_shapes_equivalent(dag, &existing, &default_int) {
        match int_literal_fits_expected_type(dag, &literal, ty.declaration) {
            Ok(Some(true)) => {
                dag.set_port_type(port, ty);
                Some(IntLiteralSetReconciliation::SetNarrow)
            }
            Ok(Some(false)) => {
                if let IntegerRangeLookup::Found(range) =
                    integer_range_for_decl(dag, ty.declaration)
                {
                    let span = node_span_for_port(dag, port).unwrap_or_else(synthetic_span);
                    dag.mark_unresolved(
                        port,
                        magnitude_out_of_range_for_interval(&literal, ty, range, span),
                    );
                    Some(IntLiteralSetReconciliation::Marked)
                } else {
                    None
                }
            }
            Ok(None) => None,
            Err(diag) => {
                if !matches!(dag.port(port).state(), PortState::Unresolved) {
                    dag.mark_unresolved(port, diag);
                }
                Some(IntLiteralSetReconciliation::Marked)
            }
        }
    } else {
        None
    }
}

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
        ensure_all_optional_match_disjs(dag);
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
                            match int_literal_magnitude_narrow_merge(dag, port, &existing, &ty) {
                                IntLiteralNarrowMerge::Merge => {
                                    dag.set_port_type(port, ty);
                                    changed = true;
                                    continue;
                                }
                                IntLiteralNarrowMerge::Reject(diag) => {
                                    dag.mark_unresolved(port, diag);
                                    changed = true;
                                    continue;
                                }
                                IntLiteralNarrowMerge::NotApplicable => {}
                            }
                            if is_retryable_generic_decl(dag, existing.declaration)
                                || is_retryable_generic_decl(dag, ty.declaration)
                            {
                                dag.set_port_type(port, ty);
                                changed = true;
                                continue;
                            }
                            if let Some(outcome) =
                                try_reconcile_int_literal_decision_set(dag, port, existing, ty)
                            {
                                match outcome {
                                    IntLiteralSetReconciliation::Keep => continue,
                                    IntLiteralSetReconciliation::SetNarrow
                                    | IntLiteralSetReconciliation::Marked => {
                                        changed = true;
                                        continue;
                                    }
                                }
                            }
                            let span = node_span_for_port(dag, port).unwrap_or_else(synthetic_span);
                            let fixes = witness_correction_for_decl(
                                dag,
                                existing.declaration,
                                span.clone(),
                                format!(
                                    "replace this expression with a `{}` value",
                                    declaration_display_name(dag, existing.declaration)
                                ),
                            )
                            .into_iter()
                            .collect();
                            let diag = Diagnostic::TypeMismatch {
                                expected: existing,
                                actual: ty,
                                span,
                                fixes,
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
                fixes: Vec::new(),
            },
        );
    }

    crate::enforced_lens_application::check_enforced_lens_applications(dag);
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
            TypeConnective::Cardinality(p)
                if p.bound() == crate::dag::CardinalityBound::AtMostOne =>
            {
                return existing_optional_match_disj_decl(dag, current);
            }
            TypeConnective::Instantiation { template, .. } => {
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
            TypeConnective::Cardinality(p)
                if p.bound() == crate::dag::CardinalityBound::AtMostOne =>
            {
                return Some(current);
            }
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
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
        TypeConnective::Cardinality(p) if p.bound() == crate::dag::CardinalityBound::AtMostOne => (
            p.element(),
            dag.declaration(cardinality_decl_id).span.clone(),
        ),
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
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        nominal_opacity: None,
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
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        nominal_opacity: None,
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
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        nominal_opacity: None,
        span,
    });
    dag.set_optional_match_disj(cardinality_decl_id, disj_id);

    Some(disj_id)
}

/// Materialize `Some`/`None` match sums for every `Cardinality(_, AtMostOne)` row so
/// downstream structural consumers (substrate reflection, lenses) can use
/// `Dag::optional_match_disj` without requiring an optional scrutinee on a Branch.
pub(crate) fn ensure_all_optional_match_disjs(dag: &mut Dag) {
    let decl_ids: Vec<DeclarationId> = dag.declarations().iter().map(|d| d.id).collect();
    for decl_id in decl_ids {
        if matches!(
            &dag.declaration(decl_id).connective,
            TypeConnective::Cardinality(p) if p.bound() == CardinalityBound::AtMostOne
        ) && existing_optional_match_disj_decl(dag, decl_id).is_none()
        {
            ensure_optional_match_disj(dag, decl_id).expect(
                "AtMostOne cardinality without optional_match_disj should materialize a Some/None disj",
            );
        }
    }
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
                        fixes: Vec::new(),
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
                fixes: Vec::new(),
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
            let arm_prefix = if check.resolved_arms.is_empty() {
                ""
            } else {
                ", "
            };
            let fixes = match dag.port(check.output_port).state() {
                PortState::Resolved(output_ty) => missing
                    .iter()
                    .filter_map(|variant| {
                        let body = example_source_for_decl(dag, output_ty.declaration)?;
                        let insert_at = check.span.byte_end.saturating_sub(1);
                        Some(Correction {
                            description: format!("add a `{variant}` arm"),
                            span: SourceSpan::new(check.span.file.clone(), insert_at, insert_at),
                            new_source: format!("{arm_prefix}{variant} => {body}"),
                        })
                    })
                    .collect(),
                _ => Vec::new(),
            };
            dag.mark_unresolved(
                check.output_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "non-exhaustive match: missing arm(s) for variant(s) `{missing_list}` — every constructor of the scrutinee's sum type must be covered"
                    ),
                    span: check.span,
                    fixes,
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
                        span: generated_payload_binding_span(path, b.span.clone()),
                        result: Err(Diagnostic::ResolveError {
                            name: "(upstream failure in match payload binding)".to_string(),
                            span: generated_payload_binding_span(path, b.span.clone()),
                            fixes: Vec::new(),
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
                    let span = generated_payload_binding_span(path, b.span.clone());
                    let result = match &variants {
                        None => Err(Diagnostic::ResolveError {
                            name: format!(
                                "cannot bind payload `{}` because the match scrutinee does not walk to a Disj type",
                                binding.binding_name
                            ),
                            span: span.clone(),
                        fixes: Vec::new(),
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
                                    fixes: Vec::new(),
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
                                fixes: Vec::new(),
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

/// T-Modeling int-lit / PR #806 consumer: allow `i64` default-typed int
/// literal output ports to refine to a range-backed fixed-width `IntegerPrimitive`
/// when the magnitude fits, without treating it as a conflicting resolution.
#[derive(Debug)]
enum IntLiteralNarrowMerge {
    /// Narrow to the fixed-width type — caller should `set_port_type` to `to`.
    Merge,
    /// Not a default-`Int` literal → fixed-width call-site narrow case.
    NotApplicable,
    /// Preserve typed diagnostics (e.g. [`Diagnostic::MagnitudeOutOfRange`]) like the
    /// annotated-`let` value-node path, not a silent fallback to `TypeMismatch`.
    Reject(Diagnostic),
}

fn int_literal_magnitude_narrow_merge(
    dag: &Dag,
    port: PortId,
    from: &TypeShape,
    to: &TypeShape,
) -> IntLiteralNarrowMerge {
    let Some(lit) = literal_bigint_at(dag, port) else {
        return IntLiteralNarrowMerge::NotApplicable;
    };
    let Some(int_shape) = dag.int_shape() else {
        return IntLiteralNarrowMerge::NotApplicable;
    };
    if !type_shapes_equivalent(dag, from, &int_shape) {
        return IntLiteralNarrowMerge::NotApplicable;
    }
    let span = node_span_for_port(dag, port).unwrap_or_else(synthetic_span);
    match int_literal_fits_expected_type(dag, &lit, to.declaration) {
        Ok(Some(true)) => IntLiteralNarrowMerge::Merge,
        Ok(Some(false)) => match integer_range_for_decl(dag, to.declaration) {
            IntegerRangeLookup::Found(bound) => IntLiteralNarrowMerge::Reject(
                magnitude_out_of_range_for_interval(&lit, *to, bound, span),
            ),
            IntegerRangeLookup::Invalid(diag) => IntLiteralNarrowMerge::Reject(diag),
            IntegerRangeLookup::Missing => {
                IntLiteralNarrowMerge::Reject(Diagnostic::ResolveError {
                    name: "(internal: integer literal out of range but no range fact)".to_string(),
                    span,
                    fixes: Vec::new(),
                })
            }
        },
        Err(diag) => IntLiteralNarrowMerge::Reject(diag),
        Ok(None) => IntLiteralNarrowMerge::NotApplicable,
    }
}

enum Decision {
    Set(PortId, TypeShape),
    Fail(PortId, Diagnostic),
    Retry,
}

fn decide(dag: &mut Dag, index: usize) -> Decision {
    // Clone before matching so `decide_transform` can take `&mut Dag` without
    // holding an immutable borrow of `dag.nodes()` across the call.
    let behavior = dag.nodes()[index].clone();
    match &behavior {
        Behavior::Value(v) => {
            // `let x: UInt8 = 5` seeds the value port to `UInt8` after lowering;
            // the Int literal Value node would otherwise re-stamp `Int64` here and
            // clobber host narrowing. If the port is already a non-`Int` type, only
            // defer (Retry) when range facts prove the literal fits that type.
            // Otherwise fail closed with `MagnitudeOutOfRange` (same contract as
            // call-site narrowing) or fall through so normal mismatch handling
            // applies when there is no range-backed check.
            if let LiteralBits::Int(lit_str) = &v.data {
                let Some(literal) = BigInt::from_str(lit_str.as_str()).ok() else {
                    return Decision::Fail(
                        v.output,
                        Diagnostic::ResolveError {
                            name: "internal: malformed decimal integer literal".to_string(),
                            span: v.span.clone(),
                            fixes: Vec::new(),
                        },
                    );
                };
                if let PortState::Resolved(existing) = dag.port(v.output).state() {
                    if let Some(int_sh) = dag.int_shape() {
                        if !type_shapes_equivalent(dag, existing, &int_sh) {
                            match int_literal_fits_expected_type(
                                dag,
                                &literal,
                                existing.declaration,
                            ) {
                                Ok(Some(true)) => return Decision::Retry,
                                Ok(Some(false)) => {
                                    match integer_range_for_decl(dag, existing.declaration) {
                                        IntegerRangeLookup::Found(range) => {
                                            return Decision::Fail(
                                                v.output,
                                                magnitude_out_of_range_for_interval(
                                                    &literal,
                                                    *existing,
                                                    range,
                                                    v.span.clone(),
                                                ),
                                            );
                                        }
                                        IntegerRangeLookup::Invalid(diag) => {
                                            return Decision::Fail(v.output, diag);
                                        }
                                        IntegerRangeLookup::Missing => {
                                            return Decision::Fail(
                                                v.output,
                                                Diagnostic::ResolveError {
                                                    name: "(internal: integer literal out of range but no range fact)"
                                                        .to_string(),
                                                    span: v.span.clone(),
                                                    fixes: Vec::new(),
                                                },
                                            );
                                        }
                                    }
                                }
                                Err(diag) => return Decision::Fail(v.output, diag),
                                Ok(None) => {}
                            }
                        }
                    }
                }
            }
            let shape_and_name = match &v.data {
                // Unconstrained integer literals keep the existing explicit
                // default to Int64. Expected-type narrowing happens at
                // reconciliation sites below, against range facts.
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
                        fixes: Vec::new(),
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
                            fixes: Vec::new(),
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
                                fixes: Vec::new(),
                                },
                            );
                        };
                        return Decision::Fail(
                            b.output,
                            Diagnostic::TypeMismatch {
                                expected: bool_ty,
                                actual: *ty,
                                span: b.span.clone(),
                                fixes: Vec::new(),
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
                            fixes: Vec::new(),
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
                                fixes: Vec::new(),
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
                                fixes: Vec::new(),
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
                        fixes: Vec::new(),
                    },
                ),
                PortState::Resolved(body_ty) => Decision::Set(l.output, *body_ty),
            }
        }
        Behavior::Bind(_) => Decision::Retry,
    }
}

fn decide_transform(dag: &mut Dag, t: &TransformNode) -> Decision {
    // Structural dispatch on `TransformTarget`. `Operator` carries the
    // resolved `OperatorKind` directly and produces its signature from
    // the LHS operand type (arithmetic returns operand type, comparison
    // returns Bool). `Callable` points at a DeclarationId and walks
    // `resolve_arrow` through any Instantiation / ResolvedIdentifier
    // chain to recover the concrete Arrow signature.
    let signature = match &t.target {
        TransformTarget::UnresolvedFieldProject { field_label } => {
            return decide_field_project(dag, t, field_label);
        }
        TransformTarget::ResolvedFieldProject { field_label } => {
            return decide_field_project(dag, t, field_label);
        }
        TransformTarget::Operator(op_kind) => {
            let lhs_type = match t.inputs.first() {
                None => {
                    return Decision::Fail(
                        t.output,
                        Diagnostic::ArityMismatch {
                            function: crate::operators::symbol(*op_kind),
                            expected: 2,
                            actual: 0,
                            span: t.span.clone(),
                            fixes: Vec::new(),
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
                                    crate::operators::symbol(*op_kind)
                                ),
                                span: t.span.clone(),
                                fixes: Vec::new(),
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
                                crate::operators::symbol(*op_kind)
                            ),
                            span: t.span.clone(),
                            fixes: Vec::new(),
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
            match dag.port((*bind_id).bind(dag).value).state() {
                PortState::Uninferred => return Decision::Retry,
                PortState::Unresolved => {
                    let name = transform_target_display_name(dag, &t.target);
                    return Decision::Fail(
                        t.output,
                        Diagnostic::ResolveError {
                            name: format!("function `{name}` has an invalid body"),
                            span: t.span.clone(),
                            fixes: Vec::new(),
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
                    fixes: Vec::new(),
                    },
                );
            }
        }
        ArrowBody::Pending | ArrowBody::NoBody => {
            // Both shapes mean "no executable body to walk."
            // `Pending` is the remaining transient scaffold from
            // `seed_function_signature` during lowering; `NoBody`
            // is the terminal "no body by construction" form used
            // by arrow types and other non-executable carriers.
            // `decide_transform` treats them identically: signature
            // inhabitance accepts; body walking is skipped. The
            // variant distinction exists so
            // `lens_structural_resolution` can treat any surviving
            // `Arrow(Pending)` in the final Dag as an R13-class body-
            // patching regression.
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
                fixes: Vec::new(),
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
                        fixes: Vec::new(),
                    },
                );
            }
            PortState::Resolved(actual) => {
                let types_equivalent = type_shapes_equivalent(dag, actual, expected_ty);
                if !types_equivalent
                    && (is_retryable_generic_decl(dag, actual.declaration)
                        || is_retryable_generic_decl(dag, expected_ty.declaration))
                {
                    return Decision::Retry;
                }
                if let Some(diag) = phantom_unit_mismatch(
                    dag,
                    transform_target_display_name(dag, &t.target),
                    expected_ty,
                    actual,
                    &t.span,
                ) {
                    return Decision::Fail(t.output, diag);
                }
                if !types_equivalent {
                    if let Some(literal) = literal_bigint_at(dag, *input_port) {
                        match int_literal_fits_expected_type(dag, &literal, expected_ty.declaration)
                        {
                            Ok(Some(true)) => {
                                if let Some(diag) = check_refinement_discharge(
                                    dag,
                                    actual,
                                    expected_ty,
                                    &t.target,
                                    &t.span,
                                ) {
                                    return Decision::Fail(t.output, diag);
                                }
                                // Default literal shape is `Int`; align the argument port with
                                // the callee's narrow range-backed type (same as `let` / `data`
                                // pre-seed + `Decision::Set` reunion for annotated literals).
                                return Decision::Set(*input_port, *expected_ty);
                            }
                            Ok(Some(false)) | Ok(None) => {}
                            Err(diag) => return Decision::Fail(t.output, diag),
                        }
                        match integer_range_for_decl(dag, expected_ty.declaration) {
                            IntegerRangeLookup::Found(range) => {
                                return Decision::Fail(
                                    *input_port,
                                    magnitude_out_of_range_for_interval(
                                        &literal,
                                        *expected_ty,
                                        range,
                                        t.span.clone(),
                                    ),
                                );
                            }
                            IntegerRangeLookup::Invalid(diag) => {
                                return Decision::Fail(t.output, diag);
                            }
                            IntegerRangeLookup::Missing => {}
                        }
                    }
                    return Decision::Fail(
                        t.output,
                        Diagnostic::TypeMismatch {
                            expected: *expected_ty,
                            actual: *actual,
                            span: t.span.clone(),
                            fixes: Vec::new(),
                        },
                    );
                }
                // DB-11 (3a.3) refinement discharge. Structural type
                // equivalence just passed; now check that any refinement
                // declared on the callee's parameter type is also carried
                // by the argument's port type, with structurally-equal
                // predicate expression DAGs. No SMT, no implication — if
                // the predicates don't walk equal, the caller must
                // introduce a narrowing Branch that does satisfy the
                // expected refinement.
                if let Some(diag) =
                    check_refinement_discharge(dag, actual, expected_ty, &t.target, &t.span)
                {
                    return Decision::Fail(t.output, diag);
                }
            }
        }
    }
    Decision::Set(t.output, signature.output)
}

fn phantom_unit_mismatch(
    dag: &Dag,
    context: String,
    expected_ty: &TypeShape,
    actual_ty: &TypeShape,
    span: &SourceSpan,
) -> Option<Diagnostic> {
    let expected_inst = instantiation_parts(dag, expected_ty.declaration)?;
    let actual_inst = instantiation_parts(dag, actual_ty.declaration)?;
    let expected_template = dag.declaration(expected_inst.template);
    for phantom in &expected_template.phantom_params {
        if let Err(diag) =
            require_abelian_group_phantom_algebra(dag, phantom.algebra, phantom.parameter, span)
        {
            return Some(diag);
        }
        if !expected_template.type_params.contains(&phantom.parameter) {
            return Some(malformed_phantom_parameter_diagnostic(
                dag,
                expected_inst.template,
                phantom.parameter,
                span,
            ));
        }
        let Some(actual_phantom) =
            mapped_phantom_parameter(dag, expected_inst.template, phantom, actual_inst.template)
        else {
            continue;
        };
        if let Err(diag) = require_abelian_group_phantom_algebra(
            dag,
            actual_phantom.algebra,
            actual_phantom.parameter,
            span,
        ) {
            return Some(diag);
        }
        if !phantom_algebras_equivalent(dag, phantom.algebra, actual_phantom.algebra) {
            return Some(unsupported_phantom_algebra_diagnostic(
                dag,
                actual_phantom.algebra,
                actual_phantom.parameter,
                span,
            ));
        }
        let Some(expected_arg) = expected_inst
            .arguments
            .iter()
            .find(|arg| arg.parameter == phantom.parameter)
        else {
            return Some(missing_phantom_argument_diagnostic(
                dag,
                expected_inst.template,
                phantom.parameter,
                "expected",
                span,
            ));
        };
        let Some(actual_arg) = actual_inst
            .arguments
            .iter()
            .find(|arg| arg.parameter == actual_phantom.parameter)
        else {
            return Some(missing_phantom_argument_diagnostic(
                dag,
                actual_inst.template,
                actual_phantom.parameter,
                "actual",
                span,
            ));
        };
        if expected_arg.value != actual_arg.value {
            return Some(Diagnostic::UnitMismatch {
                operator: context,
                parameter: phantom_parameter_display_name(dag, phantom.parameter),
                expected: TypeShape::new(expected_arg.value),
                actual: TypeShape::new(actual_arg.value),
                span: span.clone(),
                fixes: Vec::new(),
            });
        }
    }
    None
}

fn malformed_phantom_parameter_diagnostic(
    dag: &Dag,
    template: DeclarationId,
    parameter: DeclarationId,
    span: &SourceSpan,
) -> Diagnostic {
    Diagnostic::ResolveError {
        name: format!(
            "malformed phantom parameter {} on {}: parameter is not in type_params",
            phantom_parameter_display_name(dag, parameter),
            declaration_display_name(dag, template),
        ),
        span: span.clone(),
        fixes: Vec::new(),
    }
}

fn missing_phantom_argument_diagnostic(
    dag: &Dag,
    template: DeclarationId,
    parameter: DeclarationId,
    side: &str,
    span: &SourceSpan,
) -> Diagnostic {
    Diagnostic::ResolveError {
        name: format!(
            "malformed phantom instantiation for {}: missing {side} argument for phantom parameter {}",
            declaration_display_name(dag, template),
            phantom_parameter_display_name(dag, parameter),
        ),
        span: span.clone(),
        fixes: Vec::new(),
    }
}

fn unsupported_phantom_algebra_diagnostic(
    dag: &Dag,
    algebra: DeclarationId,
    parameter: DeclarationId,
    span: &SourceSpan,
) -> Diagnostic {
    Diagnostic::ResolveError {
        name: format!(
            "unsupported phantom algebra {} for phantom parameter {}",
            declaration_display_name(dag, algebra),
            phantom_parameter_display_name(dag, parameter),
        ),
        span: span.clone(),
        fixes: Vec::new(),
    }
}

fn require_abelian_group_phantom_algebra(
    dag: &Dag,
    algebra: DeclarationId,
    parameter: DeclarationId,
    span: &SourceSpan,
) -> Result<(), Diagnostic> {
    let Some(canonical_abelian_group) = dag.abelian_group_decl() else {
        return Err(unsupported_phantom_algebra_diagnostic(
            dag, algebra, parameter, span,
        ));
    };
    let mut current = algebra;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        if current == canonical_abelian_group {
            return Ok(());
        }
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => {
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                current = *next;
            }
            _ => {
                return Err(unsupported_phantom_algebra_diagnostic(
                    dag, algebra, parameter, span,
                ));
            }
        }
    }
    Err(unsupported_phantom_algebra_diagnostic(
        dag, algebra, parameter, span,
    ))
}

fn phantom_algebras_equivalent(
    dag: &Dag,
    lhs_algebra: DeclarationId,
    rhs_algebra: DeclarationId,
) -> bool {
    lhs_algebra == rhs_algebra || declaration_shapes_equivalent(dag, lhs_algebra, rhs_algebra, 0)
}

fn mapped_phantom_parameter(
    dag: &Dag,
    source_template: DeclarationId,
    source_phantom: &PhantomParameter,
    target_template: DeclarationId,
) -> Option<PhantomParameter> {
    let source_decl = dag.declaration(source_template);
    let target_decl = dag.declaration(target_template);
    let source_index = source_decl
        .type_params
        .iter()
        .position(|parameter| *parameter == source_phantom.parameter)?;
    let target_parameter = target_decl.type_params.get(source_index).copied()?;
    target_decl
        .phantom_params
        .iter()
        .find(|target_phantom| target_phantom.parameter == target_parameter)
        .cloned()
}

struct InstantiationParts<'a> {
    template: DeclarationId,
    arguments: &'a [crate::dag::TemplateArgument],
}

fn instantiation_parts(dag: &Dag, declaration: DeclarationId) -> Option<InstantiationParts<'_>> {
    match &dag.declaration(declaration).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => Some(InstantiationParts {
            template: *template,
            arguments,
        }),
        _ => None,
    }
}

fn phantom_parameter_display_name(dag: &Dag, parameter: DeclarationId) -> String {
    match &dag.declaration(parameter).connective {
        TypeConnective::Atom(AtomPayload::TypeParam(name)) => name.clone(),
        _ => declaration_display_name(dag, parameter),
    }
}

/// DB-11 (3a.3) call-site refinement discharge. Returns `None` when
/// the expected parameter has no refinement (unconditionally OK), or
/// when the actual argument's top-level refinement discharges the
/// expected predicate. Returns a `Diagnostic` otherwise. Pure
/// structural walk — no SMT, no entailment beyond conjunction-member
/// matching against the canonical composite form.
///
/// Composite-canonical invariant (post-narrowing refactor): every
/// refinement in the DAG is a single predicate Declaration. Narrowing
/// produces a composite `outer_body && new_body` rather than an alias
/// chain of unary refinements, so discharge only has to look at the
/// top-level refinement on `actual.declaration` — no chain walk. The
/// composite-vs-single asymmetry is handled by
/// `predicate_discharges`, which accepts either a full structural
/// match against the expected predicate or a match against a
/// conjunct of `actual`'s composite body.
fn check_refinement_discharge(
    dag: &Dag,
    actual: &TypeShape,
    expected: &TypeShape,
    target: &TransformTarget,
    span: &SourceSpan,
) -> Option<Diagnostic> {
    let expected_pred = dag.declaration(expected.declaration).refinement?;
    let actual_pred = dag.declaration(actual.declaration).refinement;
    match actual_pred {
        Some(actual_pred) if predicate_discharges(dag, actual_pred, expected_pred, 0) => None,
        _ => {
            let callee = transform_target_display_name(dag, target);
            let name = if actual_pred.is_some() {
                format!(
                    "argument to `{callee}` does not satisfy the expected \
                     `where` refinement (caller's predicate does not walk \
                     structurally equal to the callee's, and no conjunct \
                     of the caller's composite matches)"
                )
            } else {
                format!(
                    "argument to `{callee}` does not satisfy the expected \
                     `where` refinement — no narrowing branch in scope"
                )
            };
            Some(Diagnostic::ResolveError {
                name,
                span: span.clone(),
                fixes: Vec::new(),
            })
        }
    }
}

/// Structural discharge check between an actual and expected
/// predicate declaration. Succeeds when:
///
/// - The two predicates are the same declaration id; OR
/// - The actual predicate's body walks structurally equal to the
///   expected predicate's body (param-paired); OR
/// - The actual predicate's body is a `Transform(Logical(And), [a, b])`
///   and either `a` or `b` (as a virtual sub-predicate sharing the
///   actual's `Bind.params[0]`) discharges the expected predicate.
///
/// The composite conjunct case is the load-bearing piece that lets
/// the post-narrowing composite `outer_body && new_body` match a
/// callee whose refinement is just `outer_body`. Pure structural,
/// depth-bounded.
fn predicate_discharges(
    dag: &Dag,
    actual_pred_decl: DeclarationId,
    expected_pred_decl: DeclarationId,
    depth: usize,
) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    if actual_pred_decl == expected_pred_decl {
        return true;
    }
    let Some((actual_param, actual_body)) = predicate_info(dag, actual_pred_decl) else {
        return false;
    };
    let Some((expected_param, expected_body)) = predicate_info(dag, expected_pred_decl) else {
        return false;
    };
    body_discharges(
        dag,
        actual_body,
        expected_body,
        actual_param,
        expected_param,
    )
}

/// Flatten-and-subset discharge check. A predicate body is treated
/// as a CONJUNCTION OF LEAVES — the maximal multiset of non-`And`
/// operand ports reachable by recursively unfolding every
/// `Transform(Logical(And), [lhs, rhs])` root. Actual discharges
/// expected iff every expected leaf is walk-equal (param-paired) to
/// some actual leaf.
///
/// This makes conjunction associativity / grouping irrelevant to
/// discharge: `a && (b && c)`, `(a && b) && c`, and `a && b && c`
/// all flatten to the same leaf set `{a, b, c}` and therefore
/// discharge each other symmetrically. Without the flattening,
/// `body_discharges` could only recurse into one root conjunct of
/// the actual side at a time, so an asymmetric grouping would fail
/// to discharge a syntax-equivalent callee contract (the
/// substrate-level blocker ChatGPT's R5 review called out on
/// `31a3709d`).
///
/// Still pure structural — no SMT, no absorption laws, no ordering
/// reasoning. The "canonical form" is the leaf multiset; two bodies
/// representing the same conjunction share one leaf multiset up to
/// reordering, and the subset check tolerates the multiset shape.
fn body_discharges(
    dag: &Dag,
    actual_body: PortId,
    expected_body: PortId,
    actual_param: PortId,
    expected_param: PortId,
) -> bool {
    let mut actual_leaves: Vec<PortId> = Vec::new();
    collect_conjunct_leaves(dag, actual_body, &mut actual_leaves, 0);
    let mut expected_leaves: Vec<PortId> = Vec::new();
    collect_conjunct_leaves(dag, expected_body, &mut expected_leaves, 0);
    expected_leaves.into_iter().all(|expected_leaf| {
        actual_leaves.iter().any(|&actual_leaf| {
            refinement_ports_equal(
                dag,
                actual_leaf,
                expected_leaf,
                actual_param,
                expected_param,
                0,
            )
        })
    })
}

/// Recursively unfold `Transform(Logical(And), [lhs, rhs])` roots
/// and collect the non-`And` leaves. Depth-bounded by
/// `WALK_DEPTH_LIMIT`; on overflow the current port is pushed as a
/// leaf so discharge degrades to the pre-flatten shape rather than
/// silently dropping conjuncts.
fn collect_conjunct_leaves(dag: &Dag, body_port: PortId, out: &mut Vec<PortId>, depth: usize) {
    if depth >= WALK_DEPTH_LIMIT {
        out.push(body_port);
        return;
    }
    if let Some(node_id) = dag.port(body_port).produced_by {
        if let Behavior::Transform(t) = dag.node(node_id) {
            if matches!(
                &t.target,
                TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And))
            ) && t.inputs.len() == 2
            {
                collect_conjunct_leaves(dag, t.inputs[0], out, depth + 1);
                collect_conjunct_leaves(dag, t.inputs[1], out, depth + 1);
                return;
            }
        }
    }
    out.push(body_port);
}

fn predicate_info(dag: &Dag, pred_decl: DeclarationId) -> Option<(PortId, PortId)> {
    if let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &dag.declaration(pred_decl).connective
    {
        let bind = (*bind_id).bind(dag);
        if let Some(param_port) = bind.params.first() {
            return Some((*param_port, bind.value));
        }
    }
    None
}

fn refinement_ports_equal(
    dag: &Dag,
    lhs_port: PortId,
    rhs_port: PortId,
    lhs_param: PortId,
    rhs_param: PortId,
    depth: usize,
) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    let lhs_is_param = lhs_port == lhs_param;
    let rhs_is_param = rhs_port == rhs_param;
    if lhs_is_param || rhs_is_param {
        return lhs_is_param && rhs_is_param;
    }
    let Some(lhs_nid) = dag.port(lhs_port).produced_by else {
        return lhs_port == rhs_port;
    };
    let Some(rhs_nid) = dag.port(rhs_port).produced_by else {
        return false;
    };
    match (dag.node(lhs_nid), dag.node(rhs_nid)) {
        (Behavior::Value(lv), Behavior::Value(rv)) => lv.data == rv.data,
        (Behavior::Transform(lt), Behavior::Transform(rt)) => {
            refinement_targets_equal(dag, &lt.target, &rt.target)
                && lt.inputs.len() == rt.inputs.len()
                && lt.inputs.iter().zip(rt.inputs.iter()).all(|(l, r)| {
                    refinement_ports_equal(dag, *l, *r, lhs_param, rhs_param, depth + 1)
                })
        }
        _ => false,
    }
}

fn refinement_targets_equal(dag: &Dag, lhs: &TransformTarget, rhs: &TransformTarget) -> bool {
    match (lhs, rhs) {
        // DB-11 (3a.3) structural callable identity. Call lowering
        // materializes a fresh `Instantiation` declaration per
        // call-site when the callee has retained template arguments
        // (see `retained_template_arguments_for_target` in lower.rs).
        // Two syntactically identical calls to a generic predicate
        // (e.g., `is_eq(d, 0)`) from two different refinement
        // contexts therefore carry different `Callable(DeclarationId)`
        // targets even though their template + argument values match.
        // Nominal id equality would reject those as distinct; the
        // authoritative identity is template + substituted arguments,
        // which `declaration_shapes_equivalent` already walks through
        // `Instantiation`/`ResolvedIdentifier` edges.
        (TransformTarget::Callable(a), TransformTarget::Callable(b)) => {
            declaration_shapes_equivalent(dag, *a, *b, 0)
        }
        (TransformTarget::Operator(a), TransformTarget::Operator(b)) => a == b,
        (
            TransformTarget::UnresolvedFieldProject {
                field_label: a_label,
            },
            TransformTarget::UnresolvedFieldProject {
                field_label: b_label,
            },
        ) => a_label == b_label,
        (
            TransformTarget::ResolvedFieldProject {
                field_label: a_label,
            },
            TransformTarget::ResolvedFieldProject {
                field_label: b_label,
            },
        ) => a_label == b_label,
        _ => false,
    }
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
pub(crate) struct SubstStack {
    frames: Vec<Vec<TemplateArgument>>,
}

impl SubstStack {
    pub(crate) fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub(crate) fn push(&mut self, args: Vec<TemplateArgument>) {
        self.frames.push(args);
    }

    pub(crate) fn pop(&mut self) {
        self.frames.pop();
    }

    pub(crate) fn lookup(&self, param_id: DeclarationId) -> Option<DeclarationId> {
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            declaration_is_callable(dag, *next, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::TypeParam(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Cardinality(_) => false,
    }
}

fn is_retryable_generic_decl(dag: &Dag, current: DeclarationId) -> bool {
    let mut visiting = Vec::new();
    is_retryable_generic_decl_walk(dag, current, 0, &mut visiting)
}

fn is_retryable_generic_decl_walk(
    dag: &Dag,
    current: DeclarationId,
    depth: usize,
    visiting: &mut Vec<DeclarationId>,
) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    if visiting.contains(&current) {
        return false;
    }
    visiting.push(current);
    let retryable = match &dag.declaration(current).connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => true,
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            is_retryable_generic_decl_walk(dag, *next, depth + 1, visiting)
        }
        TypeConnective::Instantiation { arguments, .. } => arguments
            .iter()
            .any(|arg| is_retryable_generic_decl_walk(dag, arg.value, depth + 1, visiting)),
        TypeConnective::Cardinality(p) => {
            is_retryable_generic_decl_walk(dag, p.element(), depth + 1, visiting)
        }
        TypeConnective::Conj { children } => children
            .iter()
            .any(|field| is_retryable_generic_decl_walk(dag, field.ty, depth + 1, visiting)),
        TypeConnective::Disj { variants } => variants
            .iter()
            .any(|field| is_retryable_generic_decl_walk(dag, field.ty, depth + 1, visiting)),
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Arrow { .. } => false,
    };
    visiting.pop();
    retryable
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
    match generated_template_argument_value(arguments, &parameter) {
        Lookup::Hit(value) => Some(value),
        Lookup::Miss => None,
    }
}

fn resolve_template_argument_value(
    arguments: &[TemplateArgument],
    current: DeclarationId,
    depth: usize,
) -> DeclarationId {
    generated_resolve_template_argument_value(
        &(WALK_DEPTH_LIMIT.saturating_sub(depth) as i64),
        arguments,
        current,
    )
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
    if let Some(enclosing_disj) = enclosing_disj_for_variant(dag, template) {
        allowed.extend(dag.declaration(enclosing_disj).type_params.iter().copied());
    }
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

fn push_template_argument_binding(
    arguments: &mut Vec<TemplateArgument>,
    parameter: DeclarationId,
    value: DeclarationId,
) -> bool {
    match generated_push_template_argument_binding(arguments, &parameter, value) {
        TemplateArgumentBinding::Conflict => false,
        TemplateArgumentBinding::NoOp => true,
        TemplateArgumentBinding::Append => {
            arguments.push(TemplateArgument { parameter, value });
            true
        }
        TemplateArgumentBinding::ReplaceAt {
            _0: index,
            _1: updated,
        } => {
            let Some(existing) = arguments.get_mut(index as usize) else {
                return false;
            };
            existing.value = updated;
            true
        }
    }
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
        | TypeConnective::Cardinality(_) => None,
    }
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
        // Use the port's resolved declaration, not the literal's default
        // substrate type (`Int` for all int literals). After range narrowing the
        // literal port may already be `UInt8` / etc.; `literal_decl_id`-style
        // `Int` here makes `resolve_callable_target` think the argument is still
        // the default `Int`, producing spurious implicit-template conflicts against
        // a `UInt8` parameter (e.g. `id_u8(7)`).
        Behavior::Value(_) => Some(PortTypeContext {
            decl: resolved_decl,
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
                    if let Some(enclosing_disj) = enclosing_disj_for_variant(dag, template) {
                        return Some(PortTypeContext {
                            decl: enclosing_disj,
                            subst,
                        });
                    }
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
            TransformTarget::UnresolvedFieldProject { field_label }
            | TransformTarget::ResolvedFieldProject { field_label } => {
                field_project_port_type_context(dag, t, field_label)
            }
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

fn field_project_port_type_context(
    dag: &Dag,
    t: &TransformNode,
    field_label: &str,
) -> Option<PortTypeContext> {
    if t.inputs.len() != 1 {
        return None;
    }
    let input_ty = match dag.port(t.inputs[0]).state() {
        PortState::Resolved(ty) => *ty,
        PortState::Uninferred | PortState::Unresolved => return None,
    };
    let mut subst = SubstStack::new();
    let conj_id = walk_to_conj_decl_with_subst(dag, input_ty.declaration, &mut subst)?;
    let TypeConnective::Conj { children } = &dag.declaration(conj_id).connective else {
        return None;
    };
    let field_decl = children
        .iter()
        .find(|field| field.label == field_label)
        .map(|field| field.ty)?;
    Some(PortTypeContext {
        decl: field_decl,
        subst,
    })
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
        let bind = (*bind_id).bind(dag);
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
                TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
                | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
                if actual_decl != *template {
                    return false;
                }
                let template_params = dag.declaration(*template).type_params.clone();
                if template_params.len() != expected_args.len() {
                    return false;
                }
                for (expected_arg, param_id) in expected_args.iter().zip(template_params) {
                    let Some(actual_value) = actual.subst.lookup(param_id) else {
                        return false;
                    };
                    if !bind_expected_decl_to_actual_context(
                        dag,
                        expected_arg.value,
                        &PortTypeContext {
                            decl: actual_value,
                            subst: actual.subst.clone(),
                        },
                        arguments,
                        depth + 1,
                    ) {
                        return false;
                    }
                }
                return true;
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
        TypeConnective::Cardinality(p) => {
            let element = p.element();
            let bound = p.bound();
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
                TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
                | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
            let TypeConnective::Cardinality(actual_p) = &dag.declaration(actual_decl).connective
            else {
                return false;
            };
            if bound != actual_p.bound() {
                return false;
            }
            bind_expected_decl_to_actual_context(
                dag,
                element,
                &PortTypeContext {
                    decl: actual_p.element(),
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
    fixes: Vec::new(),
    }
}

/// `port_type_context` reports the default `Int` type shape for integer literals, so
/// [`bind_expected_decl_to_actual_context`] can reject a callee that expects a range-backed
/// narrow type (`UInt8`, …). The main transform decision path already allows in-range
/// literal narrowing; mirror that here so [`resolve_callable_target`] can succeed. Out-of-range
/// literals surface as [`Diagnostic::MagnitudeOutOfRange`] to match
/// `decide(Transform)`-style call checks.
fn int_literal_implicit_bind_tolerated_for_expected(
    dag: &Dag,
    expected: DeclarationId,
    input_port: PortId,
    span: &SourceSpan,
) -> Result<bool, Diagnostic> {
    let Some(literal) = literal_bigint_at(dag, input_port) else {
        return Ok(false);
    };
    match int_literal_fits_expected_type(dag, &literal, expected) {
        Ok(Some(true)) => Ok(true),
        Ok(Some(false)) => match integer_range_for_decl(dag, expected) {
            IntegerRangeLookup::Found(range) => Err(magnitude_out_of_range_for_interval(
                &literal,
                TypeShape::new(expected),
                range,
                span.clone(),
            )),
            IntegerRangeLookup::Invalid(diag) => Err(diag),
            IntegerRangeLookup::Missing => Ok(false),
        },
        Ok(None) => Ok(false),
        Err(diag) => Err(diag),
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
            fixes: Vec::new(),
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
            fixes: Vec::new(),
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
                            fixes: Vec::new(),
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
                        fixes: Vec::new(),
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
                match int_literal_implicit_bind_tolerated_for_expected(
                    dag,
                    expected_input,
                    *input_port,
                    span,
                ) {
                    Ok(true) => {}
                    Ok(false) => {
                        return CallableTargetResolution::Fail(callable_instantiation_conflict(
                            dag,
                            target,
                            expected_input,
                            &actual_ctx,
                            span,
                        ));
                    }
                    Err(diag) => return CallableTargetResolution::Fail(diag),
                }
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
                        fixes: Vec::new(),
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
                            fixes: Vec::new(),
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
                    match int_literal_implicit_bind_tolerated_for_expected(
                        dag,
                        expected_input.declaration,
                        *input_port,
                        span,
                    ) {
                        Ok(true) => {}
                        Ok(false) => {
                            return CallableTargetResolution::Fail(
                                callable_instantiation_conflict(
                                    dag,
                                    target,
                                    expected_input.declaration,
                                    &actual_ctx,
                                    span,
                                ),
                            );
                        }
                        Err(diag) => return CallableTargetResolution::Fail(diag),
                    }
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
                        fixes: Vec::new(),
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
            fixes: Vec::new(),
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
        output: TypeShape::new(
            instantiated_enclosing_disj_for_variant(dag, template, arguments)
                .or_else(|| enclosing_disj_for_variant(dag, template))
                .unwrap_or(target),
        ),
        // Variant constructor synthesis: `Variant(payload)` is direct
        // construction, not a function call with an executable body.
        // `NoBody` rather than `Pending` so the synthesized signature
        // semantically matches the "no body by construction" cases in
        // declarations, even though this `ResolvedArrow` is transient
        // inference state (never stored in `Dag.declarations`).
        body: ArrowBody::NoBody,
    })
}

fn instantiated_enclosing_disj_for_variant(
    dag: &Dag,
    variant_decl_id: DeclarationId,
    arguments: &[TemplateArgument],
) -> Option<DeclarationId> {
    let enclosing_disj = enclosing_disj_for_variant(dag, variant_decl_id)?;
    let params = &dag.declaration(enclosing_disj).type_params;
    if params.is_empty() {
        return Some(enclosing_disj);
    }
    let disj_arguments = params
        .iter()
        .map(|parameter| {
            arguments
                .iter()
                .find(|argument| argument.parameter == *parameter)
                .cloned()
        })
        .collect::<Option<Vec<_>>>()?;
    find_equivalent_anonymous_instantiation(dag, enclosing_disj, &disj_arguments, None)
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
        if current_template == template
            && matches!(
                generated_template_arguments_match(&current_arguments, &arguments),
                TemplateArgumentsMatch::Match,
            )
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
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: None,
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
            let bind = (*bind_id).bind(dag);
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

    let user_defined_arrows: Vec<(DeclarationId, Vec<DeclarationId>, DeclarationId, BindNodeId)> =
        dag.declarations()
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
        let bind = bind_id.bind(dag);
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
                    fixes: witness_correction_for_decl(
                        dag,
                        output,
                        bind.span.clone(),
                        format!(
                            "replace the function body with a `{}` value",
                            declaration_display_name(dag, output)
                        ),
                    )
                    .into_iter()
                    .collect(),
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
                    fixes: witness_correction_for_decl(
                        dag,
                        expected_decl,
                        bind.span.clone(),
                        format!(
                            "replace the parameter use with a `{}` value",
                            declaration_display_name(dag, expected_decl)
                        ),
                    )
                    .into_iter()
                    .collect(),
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
                fixes: witness_correction_for_decl(
                    dag,
                    output,
                    bind.span.clone(),
                    format!(
                        "replace the function body with a `{}` value",
                        declaration_display_name(dag, output)
                    ),
                )
                .into_iter()
                .collect(),
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
    match walk_to_conj_decl_with_subst_or_opacity(dag, start, subst, false) {
        ConjWalkResult::Conj(id) => Some(id),
        ConjWalkResult::NoConj => None,
        ConjWalkResult::NominalOpaque(_) => {
            unreachable!("opacity check is disabled for ordinary Conj walks")
        }
    }
}

enum ConjWalkResult {
    Conj(DeclarationId),
    NominalOpaque(DeclarationId),
    NoConj,
}

fn walk_to_conj_decl_with_subst_or_opacity(
    dag: &Dag,
    start: DeclarationId,
    subst: &mut SubstStack,
    stop_at_nominal_opacity: bool,
) -> ConjWalkResult {
    let mut current = start;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        if stop_at_nominal_opacity && decl.nominal_opacity.is_some() {
            return ConjWalkResult::NominalOpaque(current);
        }
        match &decl.connective {
            TypeConnective::Conj { .. } => return ConjWalkResult::Conj(current),
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                subst.push(arguments.clone());
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                current = *next;
            }
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                let Some(next) = subst.lookup(current) else {
                    return ConjWalkResult::NoConj;
                };
                current = next;
            }
            _ => return ConjWalkResult::NoConj,
        }
    }
    ConjWalkResult::NoConj
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
            TypeConnective::Cardinality(p)
                if p.bound() == crate::dag::CardinalityBound::AtMostOne =>
            {
                return existing_optional_match_disj_decl(dag, current);
            }
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                subst.push(arguments.clone());
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
            fixes: Vec::new(),
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
                fixes: Vec::new(),
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
            fixes: Vec::new(),
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
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        nominal_opacity: None,
        span: variant_decl.span,
    });
    TypeShape::new(id)
}

pub(crate) fn concretize_decl_with_subst(
    dag: &mut Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> DeclarationId {
    if depth >= WALK_DEPTH_LIMIT {
        return current;
    }
    let decl = dag.declaration(current).clone();
    // DB-16 (3a.3 closure): refinement-bearing declarations whose base
    // requires substitution materialize a fresh substituted-refined
    // carrier — structurally identical to what `lower_parameter_refinement`
    // would produce if the user had authored the refinement on the
    // concrete type directly. Runs before the connective match so the
    // refinement edge is preserved across the walk.
    if decl.refinement.is_some() && refinement_base_requires_substitution(dag, current, subst) {
        return materialize_substituted_refined_decl(dag, current, subst);
    }
    match decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => subst.lookup(current).unwrap_or(current),
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
            if let Some(existing) = find_equivalent_anonymous_instantiation(
                dag,
                template,
                &specialized_arguments,
                decl.nominal_opacity.as_ref(),
            ) {
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
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: decl.nominal_opacity.clone(),
                span: decl.span,
            });
            id
        }
        TypeConnective::Cardinality(p) => {
            let element = p.element();
            let bound = p.bound();
            let specialized_element = concretize_decl_with_subst(dag, element, subst, depth + 1);
            if let Some(existing) =
                find_equivalent_anonymous_cardinality(dag, specialized_element, &bound)
            {
                return existing;
            }
            dag.alloc_cardinality_decl(specialized_element, bound, decl.span.clone())
        }
        _ => current,
    }
}

/// DB-16 (3a.3 closure): gate used by both `concretize_decl_with_subst`
/// (construction side) and `signature_type_shape` (consumer side) to
/// decide whether a refinement-bearing declaration needs substitution
/// before its refinement edge can be meaningfully consumed.
///
/// Returns `true` iff walking the refined carrier's base through
/// `Atom(ResolvedIdentifier(_))` hops lands on either:
/// - a `TypeParam` declaration whose id is bound in `subst`, or
/// - an `Instantiation` whose arguments reference substitution-bound
///   TypeParams (i.e., substitution would materially change the
///   resolved base).
///
/// For concrete refined carriers (`Int where pred(x)`) the walk
/// short-circuits to `false`; DB-11's identity-terminator fires
/// unchanged. For refined TypeParams with no active binding (earlier
/// iteration retry case) the walk also returns `false`.
fn refinement_base_requires_substitution(
    dag: &Dag,
    current: DeclarationId,
    subst: &SubstStack,
) -> bool {
    let decl = dag.declaration(current);
    let TypeConnective::Atom(
        AtomPayload::ResolvedByStructure(base) | AtomPayload::ResolvedByName(base),
    ) = &decl.connective
    else {
        return false;
    };
    refinement_base_walk(dag, *base, subst, 0)
}

fn refinement_base_walk(
    dag: &Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    let decl = dag.declaration(current);
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => subst.lookup(current).is_some(),
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            refinement_base_walk(dag, *next, subst, depth + 1)
        }
        TypeConnective::Instantiation { arguments, .. } => arguments
            .iter()
            .any(|arg| refinement_base_walk(dag, arg.value, subst, depth + 1)),
        TypeConnective::Cardinality(p) => refinement_base_walk(dag, p.element(), subst, depth + 1),
        _ => false,
    }
}

/// DB-16 (3a.3 closure): the sole construction site for substituted
/// refined carriers. Called from `concretize_decl_with_subst`'s
/// refinement branch during the `materialize_callable_signature_instantiations`
/// phase. Produces a fresh anonymous `Declaration` structurally
/// identical to what `lower_parameter_refinement` would produce if
/// the user had authored the refinement on the concrete substituted
/// base directly.
///
/// Dedups against existing substituted-refined carriers in the Dag
/// before allocating — at fixpoint iteration N+1, if iteration N
/// already produced a carrier for this (template, subst), the scan
/// finds it and no new allocation happens.
///
/// Fail-closed per C-8: substrate-integrity violations register
/// diagnostics rather than silently degrading to the template carrier.
fn materialize_substituted_refined_decl(
    dag: &mut Dag,
    template_refined: DeclarationId,
    subst: &SubstStack,
) -> DeclarationId {
    // Dedup: if a matching substituted-refined carrier already exists
    // (e.g., from an earlier fixpoint iteration), reuse it.
    if let Some(existing) = find_equivalent_substituted_refined_decl(dag, template_refined, subst) {
        return existing;
    }

    let template_decl = dag.declaration(template_refined).clone();
    let Some(template_pred_decl_id) = template_decl.refinement else {
        // Caller (concretize_decl_with_subst's refinement branch) checks
        // `decl.refinement.is_some()` before entering. Reaching this
        // point means the caller contract was violated — surface that
        // rather than masking it with a silent fallthrough.
        unreachable!(
            "materialize_substituted_refined_decl entered on declaration {:?} without refinement edge",
            template_refined
        );
    };
    let TypeConnective::Atom(
        AtomPayload::ResolvedByStructure(template_base)
        | AtomPayload::ResolvedByName(template_base),
    ) = template_decl.connective
    else {
        // Caller also checks `refinement_base_requires_substitution`,
        // whose first step returns false for any connective other than
        // `Atom(ResolvedIdentifier(_))`. Reaching this point means the
        // caller contract was violated.
        unreachable!(
            "materialize_substituted_refined_decl entered on declaration {:?} whose connective is not Atom(ResolvedIdentifier(_))",
            template_refined
        );
    };
    let template_span = template_decl.span.clone();

    // Step 1: resolve the substituted base.
    let Some(substituted_base) = resolve_decl_with_subst(dag, template_base, subst, 0) else {
        dag.attach_diagnostic(Diagnostic::ResolveError {
            name: "refined-generic substitution: substituted base did not resolve".to_string(),
            span: template_span.clone(),
            fixes: Vec::new(),
        });
        return template_refined;
    };

    // Step 2: extract original predicate slots.
    let Some((original_param_port, original_body_port)) =
        outer_predicate_slots(dag, template_pred_decl_id)
    else {
        let pred_span = dag.declaration(template_pred_decl_id).span.clone();
        dag.attach_diagnostic(Diagnostic::ResolveError {
            name: "refined-generic substitution: malformed predicate shape".to_string(),
            span: pred_span,
            fixes: Vec::new(),
        });
        return template_refined;
    };

    // Step 3: allocate fresh composite param port typed as substituted base.
    let fresh_param_port = dag.alloc_port(None);
    dag.set_port_type(fresh_param_port, TypeShape::new(substituted_base));

    // Step 4: clone the predicate body, routing Transform targets
    // through the active substitution stack.
    let Some(cloned_body_port) = clone_predicate_body(
        dag,
        original_body_port,
        original_param_port,
        fresh_param_port,
        subst,
        0,
    ) else {
        dag.attach_diagnostic(Diagnostic::ResolveError {
            name: "refined-generic substitution: out-of-fragment predicate body reached materialization"
                .to_string(),
            span: template_span.clone(),
            fixes: Vec::new(),
        });
        return template_refined;
    };

    // Step 5: wrap cloned body in a fresh Bind.
    let bind_id = dag.alloc_node_id();
    dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: "<refinement:substituted>".to_string(),
        value: cloned_body_port,
        params: vec![fresh_param_port],
        span: template_span.clone(),
        lane2_workflow: None,
        emit_participation: None,
    }));

    // Step 5 (cont.): build the fresh predicate-Arrow Declaration.
    // Inherit the original predicate's output type (Bool) — reading it
    // from the original Arrow keeps this module independent of the
    // Bool-decl lookup pattern `lower_parameter_refinement` uses.
    let TypeConnective::Arrow {
        output: bool_decl_id,
        ..
    } = &dag.declaration(template_pred_decl_id).connective
    else {
        // `outer_predicate_slots` rejected non-Arrow predicates at
        // step 2 above; reaching this point means the predicate
        // declaration's connective mutated between step 2 and step 5,
        // which is impossible under `&mut Dag` exclusive access.
        unreachable!(
            "predicate declaration {:?} connective is not Arrow despite outer_predicate_slots succeeding",
            template_pred_decl_id
        );
    };
    let bool_decl_id = *bool_decl_id;

    let fresh_pred_decl_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: fresh_pred_decl_id,
        name: None,
        connective: TypeConnective::Arrow {
            inputs: vec![substituted_base],
            output: bool_decl_id,
            body: ArrowBody::UserDefined(
                BindNodeId::from_bind_node(dag, bind_id)
                    .expect("UserDefined Arrow body bind id must point at a Bind"),
            ),
        },
        type_params: Vec::new(),
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        nominal_opacity: None,
        span: template_span.clone(),
    });

    // Step 6: allocate the substituted-refined carrier Declaration.
    let fresh_refined_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: fresh_refined_id,
        name: None,
        connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(substituted_base)),
        type_params: Vec::new(),
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: Some(fresh_pred_decl_id),
        nominal_opacity: None,
        span: template_span,
    });

    fresh_refined_id
}

/// DB-16 (3a.3 closure): read-only lookup used by both
/// `concretize_decl_with_subst` (pre-allocation dedup) and
/// `signature_type_shape` (consumer-side lookup gate).
///
/// Given a template refined carrier and an active substitution, scans
/// `dag.declarations()` for an already-materialized substituted-refined
/// carrier whose base matches the substituted base AND whose predicate
/// body walks structurally equal to what cloning the template's body
/// with the given subst WOULD produce. Mirrors
/// `find_equivalent_anonymous_instantiation`'s dedup pattern.
///
/// Pure read; no allocation. `&Dag`.
fn find_equivalent_substituted_refined_decl(
    dag: &Dag,
    template_refined: DeclarationId,
    subst: &SubstStack,
) -> Option<DeclarationId> {
    let template_decl = dag.declaration(template_refined);
    let TypeConnective::Atom(
        AtomPayload::ResolvedByStructure(template_base)
        | AtomPayload::ResolvedByName(template_base),
    ) = &template_decl.connective
    else {
        return None;
    };
    let template_pred_id = template_decl.refinement?;

    let substituted_base = resolve_decl_with_subst(dag, *template_base, subst, 0)?;
    if substituted_base == *template_base {
        return None;
    }

    let (template_param, template_body) = outer_predicate_slots(dag, template_pred_id)?;

    for decl in dag.declarations() {
        if decl.name.is_some() {
            continue;
        }
        if decl.id == template_refined {
            continue;
        }
        let TypeConnective::Atom(
            AtomPayload::ResolvedByStructure(cand_base) | AtomPayload::ResolvedByName(cand_base),
        ) = &decl.connective
        else {
            continue;
        };
        if *cand_base != substituted_base {
            continue;
        }
        let Some(cand_pred_id) = decl.refinement else {
            continue;
        };
        let Some((cand_param, cand_body)) = outer_predicate_slots(dag, cand_pred_id) else {
            continue;
        };
        if predicate_bodies_equal_under_subst(
            dag,
            cand_body,
            cand_param,
            template_body,
            template_param,
            subst,
            0,
        ) {
            return Some(decl.id);
        }
    }
    None
}

/// DB-16 (3a.3 closure): lockstep structural equality on two predicate
/// bodies, threading the active substitution stack through the
/// template side's Transform target decl-level references. Used by
/// `find_equivalent_substituted_refined_decl` to compare a candidate
/// materialized body against what cloning the template would produce.
///
/// Pure read; walks Value/Transform nodes, treats the parameter-slot
/// on each side as equivalent. Mirrors `clone_predicate_body`'s shape
/// exactly so the comparison is faithful to what cloning emits.
fn predicate_bodies_equal_under_subst(
    dag: &Dag,
    cand_port: PortId,
    cand_param: PortId,
    template_port: PortId,
    template_param: PortId,
    subst: &SubstStack,
    depth: usize,
) -> bool {
    if depth >= WALK_DEPTH_LIMIT {
        return false;
    }
    let cand_is_param = cand_port == cand_param;
    let template_is_param = template_port == template_param;
    if cand_is_param != template_is_param {
        return false;
    }
    if cand_is_param {
        return true;
    }

    let cand_node_id = dag.port(cand_port).produced_by;
    let template_node_id = dag.port(template_port).produced_by;
    match (cand_node_id, template_node_id) {
        (None, None) => cand_port == template_port,
        (Some(cand_id), Some(template_id)) => match (dag.node(cand_id), dag.node(template_id)) {
            (Behavior::Value(cand_v), Behavior::Value(template_v)) => {
                cand_v.data == template_v.data
            }
            (Behavior::Transform(cand_t), Behavior::Transform(template_t)) => {
                if !transform_targets_equal_under_subst(
                    dag,
                    &cand_t.target,
                    &template_t.target,
                    subst,
                ) {
                    return false;
                }
                if cand_t.inputs.len() != template_t.inputs.len() {
                    return false;
                }
                cand_t
                    .inputs
                    .iter()
                    .zip(template_t.inputs.iter())
                    .all(|(c, t)| {
                        predicate_bodies_equal_under_subst(
                            dag,
                            *c,
                            cand_param,
                            *t,
                            template_param,
                            subst,
                            depth + 1,
                        )
                    })
            }
            _ => false,
        },
        _ => false,
    }
}

fn transform_targets_equal_under_subst(
    dag: &Dag,
    cand: &TransformTarget,
    template: &TransformTarget,
    subst: &SubstStack,
) -> bool {
    match (cand, template) {
        (TransformTarget::Operator(a), TransformTarget::Operator(b)) => a == b,
        (TransformTarget::Callable(cand_id), TransformTarget::Callable(template_id)) => {
            callable_decls_equal_under_subst(dag, *cand_id, *template_id, subst)
        }
        (
            TransformTarget::UnresolvedFieldProject {
                field_label: cand_label,
            },
            TransformTarget::UnresolvedFieldProject {
                field_label: template_label,
            },
        ) => cand_label == template_label,
        (
            TransformTarget::ResolvedFieldProject {
                field_label: cand_label,
            },
            TransformTarget::ResolvedFieldProject {
                field_label: template_label,
            },
        ) => cand_label == template_label,
        _ => false,
    }
}

/// DB-16 (3a.3 closure): structural equivalence between two declaration
/// references under an active substitution stack. Used by
/// `transform_targets_equal_under_subst` to compare Callable /
/// FieldProject target decl-level references when one side is a
/// template-context Instantiation (possibly carrying extra bindings
/// for outer TypeParams) and the other is a concrete site
/// Instantiation (only bindings for the callee's own TypeParams).
///
/// Strategy: normalize both sides to their template-own-type-param
/// bindings (filtering out substitution-stack artifacts from the
/// template side), resolve through the subst stack on the template
/// side, then compare. Falls back to
/// `declaration_shapes_equivalent` when either side isn't an
/// Instantiation (direct decl id match).
fn callable_decls_equal_under_subst(
    dag: &Dag,
    cand_id: DeclarationId,
    template_id: DeclarationId,
    subst: &SubstStack,
) -> bool {
    if cand_id == template_id {
        return true;
    }
    // Fast path: resolve template through subst and compare directly.
    let template_resolved =
        resolve_decl_with_subst(dag, template_id, subst, 0).unwrap_or(template_id);
    if cand_id == template_resolved
        || declaration_shapes_equivalent(dag, cand_id, template_resolved, 0)
    {
        return true;
    }
    // Fallback: both are Instantiations, compare by template own-type-params.
    let (Some(cand_norm), Some(template_norm)) = (
        normalized_instantiation_args(dag, cand_id),
        normalized_instantiation_args(dag, template_id),
    ) else {
        return false;
    };
    if cand_norm.template != template_norm.template {
        return false;
    }
    if cand_norm.args.len() != template_norm.args.len() {
        return false;
    }
    cand_norm
        .args
        .iter()
        .zip(template_norm.args.iter())
        .all(|(c_arg, t_arg)| {
            c_arg.parameter == t_arg.parameter && {
                let t_val =
                    resolve_decl_with_subst(dag, t_arg.value, subst, 0).unwrap_or(t_arg.value);
                c_arg.value == t_val || declaration_shapes_equivalent(dag, c_arg.value, t_val, 0)
            }
        })
}

struct NormalizedInstantiation {
    template: DeclarationId,
    args: Vec<TemplateArgument>,
}

/// DB-16 (3a.3 closure): normalize an `Instantiation` declaration by
/// stripping **only** self-bindings (`arg.parameter == arg.value`).
///
/// Self-bindings are reattachment artifacts produced by
/// `resolve_callable_target`'s unification when a generic call site
/// is encountered under an outer generic scope: the outer TypeParam
/// binds to itself in the callee's argument list because no concrete
/// value has been inferred yet. Those self-bindings are no-op under
/// substitution (`SubstStack::lookup` short-circuits to `None` on
/// them) but inflate argument-length comparisons in strict
/// structural checks.
///
/// Non-self bindings are NEVER stripped: retained callable-argument
/// identities (per `retained_template_arguments_for_target`) carry
/// semantic meaning that must survive DB-16's equivalence walk. Two
/// Instantiations that differ only by a non-self retained binding
/// are structurally distinct and must compare unequal.
fn normalized_instantiation_args(
    dag: &Dag,
    decl: DeclarationId,
) -> Option<NormalizedInstantiation> {
    match normalize_instantiation_arguments(&dag.declaration(decl).connective) {
        NormalizedInstantiationArgs::NotInstantiation => None,
        NormalizedInstantiationArgs::Normalized { template, args } => {
            Some(NormalizedInstantiation { template, args })
        }
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
    nominal_opacity: Option<&NominalOpacity>,
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
            && decl.nominal_opacity.as_ref() == nominal_opacity
            && matches!(
                generated_template_arguments_match(existing_arguments, arguments),
                TemplateArgumentsMatch::Match,
            ))
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
        let TypeConnective::Cardinality(existing) = &decl.connective else {
            return None;
        };
        (element == existing.element() && existing.bound() == *bound).then_some(decl.id)
    })
}

enum FieldProjectResolution {
    Retry,
    Fail(Diagnostic),
    Resolved { output_ty: TypeShape },
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
            fixes: Vec::new(),
        });
    }

    let input_ty = match dag.port(t.inputs[0]).state() {
        PortState::Uninferred => return FieldProjectResolution::Retry,
        PortState::Unresolved => {
            return FieldProjectResolution::Fail(Diagnostic::ResolveError {
                name: format!("(upstream failure in field `{field_label}`)"),
                span: t.span.clone(),
                fixes: Vec::new(),
            })
        }
        PortState::Resolved(ty) => *ty,
    };

    let mut subst = SubstStack::new();
    if let Some(opaque_decl) = first_nominal_opaque_on_conj_walk(dag, input_ty.declaration, &subst)
    {
        return FieldProjectResolution::Fail(Diagnostic::NominalOpacityViolation {
            declaration: opaque_decl,
            accessor: None,
            span: t.span.clone(),
            fixes: Vec::new(),
        });
    }
    let Some(actual_conj_id) = walk_to_conj_decl_with_subst(dag, input_ty.declaration, &mut subst)
    else {
        return FieldProjectResolution::Fail(Diagnostic::ResolveError {
            name: format!(
                "field `{field_label}` cannot be projected from `{}` because it does not walk to a Conj type",
                target_display_name(dag, input_ty.declaration),
            ),
            span: t.span.clone(),
        fixes: Vec::new(),
        });
    };

    let children = match &dag.declaration(actual_conj_id).connective {
        TypeConnective::Conj { children } => children,
        _ => unreachable!("walk_to_conj_decl returned a non-Conj declaration"),
    };
    let Some(field_decl_id) = children
        .iter()
        .find(|field| field.label == field_label)
        .map(|field| field.ty)
    else {
        let field_start = t.span.byte_end.saturating_sub(field_label.len() as u32);
        let fixes = children
            .iter()
            .take(5)
            .map(|field| Correction {
                description: format!("did you mean field `{}`?", field.label),
                span: SourceSpan::new(t.span.file.clone(), field_start, t.span.byte_end),
                new_source: field.label.clone(),
            })
            .collect();
        return FieldProjectResolution::Fail(Diagnostic::ResolveError {
            name: format!(
                "field `{field_label}` does not exist on `{}`",
                target_display_name(dag, input_ty.declaration),
            ),
            span: t.span.clone(),
            fixes,
        });
    };
    let Some(output_ty) = walk_to_type_shape(dag, field_decl_id, &subst, 0) else {
        return FieldProjectResolution::Fail(Diagnostic::ResolveError {
            name: format!(
                "field `{field_label}` on `{}` does not resolve to a port type",
                target_display_name(dag, input_ty.declaration),
            ),
            span: t.span.clone(),
            fixes: Vec::new(),
        });
    };

    FieldProjectResolution::Resolved { output_ty }
}

fn first_nominal_opaque_on_conj_walk(
    dag: &Dag,
    start: DeclarationId,
    subst: &SubstStack,
) -> Option<DeclarationId> {
    let mut subst = subst.clone();
    match walk_to_conj_decl_with_subst_or_opacity(dag, start, &mut subst, true) {
        ConjWalkResult::NominalOpaque(id) => Some(id),
        ConjWalkResult::Conj(_) | ConjWalkResult::NoConj => None,
    }
}

fn decide_field_project(dag: &Dag, t: &TransformNode, field_label: &str) -> Decision {
    match resolve_field_project(dag, t, field_label) {
        FieldProjectResolution::Retry => Decision::Retry,
        FieldProjectResolution::Fail(diag) => Decision::Fail(t.output, diag),
        FieldProjectResolution::Resolved { output_ty, .. } => Decision::Set(t.output, output_ty),
    }
}

fn resolve_field_project_targets(dag: &mut Dag) -> bool {
    let mut rewrites: Vec<(usize, String)> = Vec::new();
    for (node_index, node) in dag.nodes().iter().enumerate() {
        let Behavior::Transform(t) = node else {
            continue;
        };
        let TransformTarget::UnresolvedFieldProject { field_label } = &t.target else {
            continue;
        };
        let FieldProjectResolution::Resolved { .. } = resolve_field_project(dag, t, field_label)
        else {
            continue;
        };
        rewrites.push((node_index, field_label.clone()));
    }

    let mut changed = false;
    for (node_index, field_label) in rewrites {
        let Behavior::Transform(t) = &mut dag.nodes_mut()[node_index] else {
            continue;
        };
        let target = &mut t.target;
        let TransformTarget::UnresolvedFieldProject { .. } = target else {
            continue;
        };
        *target = TransformTarget::ResolvedFieldProject { field_label };
        changed = true;
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
/// - `Bool`: ~~no structural link~~ now uses `Declaration.inhabits` on
///   the kernel `Bool` sum in `dsl/std/types.dag`, populated at bootstrap
///   (`bootstrap::patch_kernel_bool_boolean_algebra_inhabits`) so v2
///   keeps a plain `type Bool = True | False` surface. The walk follows
///   `Declaration.inhabits` when the connective chain hits a `Disj`
///   without an inline algebra.
/// - `String` / collection-level algebras whose receiver is
///   `FreeMonoid<T>` / `Set<T>` / `Map<K, V>` (the whole
///   instantiation, not just `T`): needs a refinement to the
///   substitution rule.
///
/// For those cases the resolver falls back to a Rust-side bridge
/// with a `(T, T) -> T` / `(T, T) -> Bool` signature. The fallback
/// is explicit about being a scaffold (see OperatorKind's
/// dissolution receipt).
///
/// **Dedup invariant:** anonymous `Result<…, DivError>` instantiations
/// must reuse `find_equivalent_anonymous_instantiation` on every pass;
/// a missed structural match would mint fresh `DeclarationId`s each
/// iteration and the fixpoint loop would not terminate.
fn div_total_result_output_shape(dag: &mut Dag, base_lhs: TypeShape) -> Option<TypeShape> {
    let result_template = canonical_result_decl(dag)?.id;
    let div_error = canonical_div_error_decl(dag)?.id;
    let (t_ok, t_err, span) = {
        let rdecl = dag.declaration(result_template);
        (
            rdecl.type_params.first().copied()?,
            rdecl.type_params.get(1).copied()?,
            rdecl.span.clone(),
        )
    };
    let args = vec![
        TemplateArgument {
            parameter: t_ok,
            value: base_lhs.declaration,
        },
        TemplateArgument {
            parameter: t_err,
            value: div_error,
        },
    ];
    if let Some(existing) =
        find_equivalent_anonymous_instantiation(dag, result_template, &args, None)
    {
        return Some(TypeShape::new(existing));
    }
    let id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id,
        name: None,
        connective: TypeConnective::Instantiation {
            template: result_template,
            arguments: args,
        },
        type_params: Vec::new(),
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        nominal_opacity: None,
        span,
    });
    Some(TypeShape::new(id))
}

fn resolve_operator_arrow(
    dag: &mut Dag,
    op_kind: OperatorKind,
    lhs_type: &TypeShape,
) -> Option<ResolvedArrow> {
    // DB-11 (3a.3) operand normalization. Primitive operators (`+`,
    // `>`, `!=`, etc.) are structurally over BASE types — refinements
    // are surface-level facts about values, not part of the operator's
    // arrow contract. Mirroring a refined lhs like `Int where d != 0`
    // onto both operand positions (as the old fallback did) made the
    // call-site refinement-discharge pass treat the refinement as a
    // real requirement on every operand — so a literal `10` in
    // `d > 10` failed discharge because literals carry no refinement.
    // Strip refinements once up front; algebra-Conj walks and the
    // primitive fallback both operate on the base.
    let source_id = strip_refinement_to_base(dag, lhs_type.declaration);
    let base_lhs = TypeShape::new(source_id);
    // Walk the source type's declaration chain to find the algebra
    // Conj. We follow Instantiation/ResolvedIdentifier edges; at
    // each step the template argument bindings are intentionally
    // discarded — see the receiver substitution rule in the doc
    // comment.
    let mut current = source_id;
    let mut saw_algebra_conj = false;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current).clone();
        match &decl.connective {
            TypeConnective::Conj { children } => {
                saw_algebra_conj = true;
                // Found the algebra. Look up the operator's field by
                // name.
                let field_name = crate::operators::algebra_field_name(op_kind);
                if let Some(field) = children.iter().find(|f| f.label == field_name) {
                    return read_algebra_field(dag, &decl, field.ty, source_id, op_kind, &base_lhs);
                }
                // Algebra doesn't declare this operator's field —
                // fall back to the Rust-side scaffold bridge below.
                break;
            }
            TypeConnective::Instantiation { template, .. } => {
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                current = *next;
            }
            // Terminal connectives: try `Declaration.inhabits` (e.g. Bool
            // → BooleanAlgebra<Bool>) before giving up.
            TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
            | TypeConnective::Atom(AtomPayload::TypeParam(_))
            | TypeConnective::Atom(AtomPayload::Literal(_))
            | TypeConnective::Disj { .. }
            | TypeConnective::Arrow { .. }
            | TypeConnective::Cardinality(_) => {
                if let Some(inh) = decl.inhabits {
                    current = inh;
                } else {
                    break;
                }
            }
        }
    }
    // Fallback: Rust-side scaffold bridge for primitives whose
    // structural walk doesn't terminate at an algebra Conj (Bool,
    // Classical; collection-level algebras). Documented class-5 gap.
    //
    // Logical operators are Bool-monomorphic: both operands and the
    // output are always Bool, independent of `lhs_type`. An Int lhs
    // on `&&` / `||` must surface a type mismatch, not propagate
    // through the operand slots the way Arithmetic / Comparison do.
    //
    // Fail-closed for LHS that the walk never reached any Conj for:
    // every loop exit saw a terminal connective with no `inhabits`,
    // an Arrow / Disj / Cardinality with no `inhabits`, or depth
    // exhaustion. The pre-fix fallback fabricated `(T,T)->T` /
    // `(T,T)->Bool` / `(Bool,Bool)->Bool` / `(T,T)->Result<T,DivError>`
    // for any such LHS — inventing an arrow contract for a type that
    // declares none. Refuse synthesis here so the caller surfaces a
    // typed unsupported-operator diagnostic. The Conj-reached-but-
    // missing-field path (`saw_algebra_conj == true`) keeps its
    // existing scaffold; tightening that further (distinguishing
    // algebra carriers from arbitrary product structs) requires a
    // structural signal beyond "is a Conj" — see PR #1329 STOP+PING.
    if !saw_algebra_conj {
        return None;
    }
    let (inputs, output) = match op_kind {
        // Division leaves the class-5 scaffold: total shape matches algebra `div`.
        OperatorKind::Arithmetic(ArithmeticOp::Div) => (
            vec![base_lhs, base_lhs],
            div_total_result_output_shape(dag, base_lhs)?,
        ),
        OperatorKind::Arithmetic(_) => (vec![base_lhs, base_lhs], base_lhs),
        OperatorKind::Comparison(_) => (vec![base_lhs, base_lhs], dag.bool_shape()?),
        OperatorKind::Logical(_) => {
            let bool_shape = dag.bool_shape()?;
            (vec![bool_shape, bool_shape], bool_shape)
        }
    };
    Some(ResolvedArrow {
        inputs,
        output,
        body: ArrowBody::Pending,
    })
}

/// DB-11 (3a.3) refinement-strip helper. Walks the
/// `Atom(ResolvedIdentifier(...))` chain, skipping past any
/// declaration that carries a `refinement` edge, until it reaches a
/// declaration with no refinement. Used by `resolve_operator_arrow`
/// to normalize primitive-operator inputs to their base types —
/// refinements are surface-level facts about values, not part of an
/// operator's arrow contract.
///
/// Also used by [`crate::emit::operator_carrier_realization`] so emit
/// and infer share one refinement-stripping rule (no drift).
///
/// Terminates at the first un-refined declaration OR when the chain
/// can no longer be followed (any non-ResolvedIdentifier connective).
/// Depth-bounded by `WALK_DEPTH_LIMIT`.
pub(crate) fn strip_refinement_to_base(dag: &Dag, decl_id: DeclarationId) -> DeclarationId {
    let mut current = decl_id;
    for _ in 0..WALK_DEPTH_LIMIT {
        let decl = dag.declaration(current);
        if decl.refinement.is_none() {
            return current;
        }
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                current = *next;
            }
            _ => return current,
        }
    }
    current
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
    dag: &mut Dag,
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
        .map(|id| substitute_receiver(dag, *id, receiver_param, source_id).map(TypeShape::new))
        .collect::<Option<_>>()?;
    let output_shape =
        substitute_receiver(dag, arrow_output, receiver_param, source_id).map(TypeShape::new)?;
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
    dag: &mut Dag,
    current: DeclarationId,
    receiver_param: Option<DeclarationId>,
    source_id: DeclarationId,
) -> Option<DeclarationId> {
    if Some(current) == receiver_param {
        return Some(source_id);
    }
    let decl = dag.declaration(current).clone();
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            substitute_receiver(dag, *next, receiver_param, source_id)
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let mut new_args: Vec<TemplateArgument> = Vec::with_capacity(arguments.len());
            let mut any_change = false;
            for arg in arguments {
                let new_val = substitute_receiver(dag, arg.value, receiver_param, source_id)?;
                if new_val != arg.value {
                    any_change = true;
                }
                new_args.push(TemplateArgument {
                    parameter: arg.parameter,
                    value: new_val,
                });
            }
            if !any_change {
                return Some(current);
            }
            // Same dedup contract as `div_total_result_output_shape`: false negatives
            // here allocate unbounded anonymous instantiations across fixpoint iterations.
            if let Some(existing) = find_equivalent_anonymous_instantiation(
                dag,
                *template,
                &new_args,
                decl.nominal_opacity.as_ref(),
            ) {
                return Some(existing);
            }
            let id = dag.alloc_declaration_id();
            let span = decl.span.clone();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Instantiation {
                    template: *template,
                    arguments: new_args,
                },
                type_params: Vec::new(),
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: decl.nominal_opacity.clone(),
                span,
            });
            Some(id)
        }
        TypeConnective::Atom(AtomPayload::TypeParam(_))
        | TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_)) => None,
        _ => {
            if decl.name.is_some() {
                Some(current)
            } else {
                None
            }
        }
    }
}

fn canonical_result_decl(dag: &Dag) -> Option<&Declaration> {
    dag.declarations()
        .iter()
        .find(|decl| substrate_result_type_decl_suppressed_for_emit(dag, decl))
}

fn canonical_div_error_decl(dag: &Dag) -> Option<&Declaration> {
    dag.declarations()
        .iter()
        .find(|decl| substrate_div_error_type_decl_suppressed_for_emit(dag, decl))
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
        TypeConnective::Cardinality(_) => None,
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
        TypeConnective::Cardinality(_) => Some(TypeShape::new(current)),
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
    // DB-16 (3a.3 closure): read-only lookup gate. When the refinement
    // base requires substitution, check whether
    // `materialize_callable_signature_instantiations` has already
    // produced a substituted-refined carrier for this
    // (template_refined, subst) combination. If so, return that
    // carrier. Construction lives at the phase; this gate reads.
    if decl.refinement.is_some() && refinement_base_requires_substitution(dag, current, subst) {
        if let Some(materialized) = find_equivalent_substituted_refined_decl(dag, current, subst) {
            return Some(TypeShape::new(materialized));
        }
        // Lookup miss: the phase didn't materialize for this
        // combination (TypeParam unbound at phase time, or call-site
        // outside the phase walk). Fall through to the DB-11
        // identity-terminator; downstream `is_retryable_generic_decl`
        // classifies as retry.
    }
    // DB-11 (3a.3): refinement-bearing declarations are identity
    // terminators for signature walks. Without this, a refined param
    // type `Int where d != 0` (connective: ResolvedIdentifier(Int),
    // refinement: Some(pred)) would be walked to its base `Int`, and
    // the call-site refinement-discharge check in `decide_transform`
    // would never see the predicate edge on the callee side.
    if decl.refinement.is_some() {
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
        TypeConnective::Cardinality(_) => Some(TypeShape::new(current)),
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
        TypeConnective::Cardinality(p) => {
            let element = p.element();
            let bound = p.bound();
            let specialized_element = resolve_decl_with_subst(dag, element, subst, depth + 1)?;
            if bound == crate::dag::CardinalityBound::AtMostOne {
                if let Some(idem) =
                    crate::dag::cardinality_idempotent_target(dag, specialized_element, bound)
                {
                    return Some(idem);
                }
            }
            if specialized_element == element {
                return Some(current);
            }
            find_equivalent_decl_cardinality(dag, specialized_element, &bound).or(Some(current))
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
        let TypeConnective::Cardinality(existing) = &decl.connective else {
            return None;
        };
        (element == existing.element() && existing.bound() == *bound).then_some(decl.id)
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
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
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
        TransformTarget::UnresolvedFieldProject { field_label } => format!(".{field_label}"),
        TransformTarget::ResolvedFieldProject { field_label } => format!(".{field_label}"),
        TransformTarget::Operator(op_kind) => crate::operators::symbol(*op_kind),
    }
}

fn node_span_for_port(dag: &Dag, port: PortId) -> Option<SourceSpan> {
    dag.port(port)
        .produced_by
        .map(|node_id| behavior_span(dag.node(node_id)))
}

fn synthetic_span() -> SourceSpan {
    SourceSpan::new("<inferred>", 0, 0)
}

pub(crate) fn type_shapes_equivalent(dag: &Dag, lhs: &TypeShape, rhs: &TypeShape) -> bool {
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
        (
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)),
            _,
        ) => declaration_shapes_equivalent(dag, *next, rhs, depth + 1),
        (
            _,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)),
        ) => declaration_shapes_equivalent(dag, lhs, *next, depth + 1),
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
                && phantom_arguments_equivalent_for_shape(
                    dag,
                    *lhs_template,
                    lhs_arguments,
                    *rhs_template,
                    rhs_arguments,
                )
                && lhs_arguments.len() == rhs_arguments.len()
                && lhs_arguments
                    .iter()
                    .zip(rhs_arguments.iter())
                    .all(|(lhs_arg, rhs_arg)| {
                        declaration_shapes_equivalent(dag, lhs_arg.value, rhs_arg.value, depth + 1)
                    })
        }
        (TypeConnective::Cardinality(lhs_p), TypeConnective::Cardinality(rhs_p)) => {
            lhs_p.bound() == rhs_p.bound()
                && declaration_shapes_equivalent(dag, lhs_p.element(), rhs_p.element(), depth + 1)
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

fn phantom_arguments_equivalent_for_shape(
    dag: &Dag,
    lhs_template: DeclarationId,
    lhs_arguments: &[TemplateArgument],
    rhs_template: DeclarationId,
    rhs_arguments: &[TemplateArgument],
) -> bool {
    phantom_arguments_preserved_from_template(
        dag,
        lhs_template,
        lhs_arguments,
        rhs_template,
        rhs_arguments,
    ) && phantom_arguments_preserved_from_template(
        dag,
        rhs_template,
        rhs_arguments,
        lhs_template,
        lhs_arguments,
    )
}

fn phantom_arguments_preserved_from_template(
    dag: &Dag,
    source_template: DeclarationId,
    source_arguments: &[TemplateArgument],
    target_template: DeclarationId,
    target_arguments: &[TemplateArgument],
) -> bool {
    let source_decl = dag.declaration(source_template);
    for phantom in &source_decl.phantom_params {
        if require_abelian_group_phantom_algebra(
            dag,
            phantom.algebra,
            phantom.parameter,
            &synthetic_span(),
        )
        .is_err()
        {
            return false;
        }
        let Some(target_phantom) =
            mapped_phantom_parameter(dag, source_template, phantom, target_template)
        else {
            return false;
        };
        if require_abelian_group_phantom_algebra(
            dag,
            target_phantom.algebra,
            target_phantom.parameter,
            &synthetic_span(),
        )
        .is_err()
        {
            return false;
        }
        if !phantom_algebras_equivalent(dag, phantom.algebra, target_phantom.algebra) {
            return false;
        }
        let Some(source_arg) = source_arguments
            .iter()
            .find(|arg| arg.parameter == phantom.parameter)
        else {
            return false;
        };
        let Some(target_arg) = target_arguments
            .iter()
            .find(|arg| arg.parameter == target_phantom.parameter)
        else {
            return false;
        };
        if source_arg.value != target_arg.value {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod bool_logical_operator_arrow_tests {
    use super::*;
    use crate::dag::PhantomParameter;
    use crate::infer_helpers::filter_non_self_template_arguments;
    use crate::operators::{ArithmeticOp, ComparisonOp};

    #[test]
    fn nominal_opaque_field_project_fails_closed_before_structural_descent() {
        let mut dag = Dag::new();
        let span = SourceSpan::new("<nominal-opacity-test>", 0, 1);
        let int = dag.int_shape().expect("bootstrap Int").declaration;

        let payload = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: payload,
            name: Some("OpaquePayload".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "value".to_string(),
                    ty: int,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let opaque = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: opaque,
            name: Some("OpaqueWrapper".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(payload)),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: Some(NominalOpacity {
                permitted_accessors: Vec::new(),
            }),
            span: span.clone(),
        });

        let input = dag.alloc_port_with_shape(TypeShape::new(opaque));
        let output = dag.push_transform(
            TransformTarget::UnresolvedFieldProject {
                field_label: "value".to_string(),
            },
            vec![input],
            span,
        );

        infer(&mut dag);

        assert!(matches!(
            dag.diagnostics().get(output),
            Some(Diagnostic::NominalOpacityViolation {
                declaration,
                accessor: None,
                ..
            }) if *declaration == opaque
        ));
        assert!(
            matches!(dag.port(output).state(), PortState::Unresolved),
            "opacity violation must leave the projected output unresolved"
        );
    }

    #[test]
    fn nominal_opaque_field_project_preserves_instantiation_substitution() {
        let mut dag = Dag::new();
        let span = SourceSpan::new("<nominal-opacity-subst-test>", 0, 1);
        let int = dag.int_shape().expect("bootstrap Int").declaration;

        let payload = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: payload,
            name: Some("SecretPayloadForSubst".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "value".to_string(),
                    ty: int,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let secret = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: secret,
            name: Some("SecretForSubst".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(payload)),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: Some(NominalOpacity {
                permitted_accessors: Vec::new(),
            }),
            span: span.clone(),
        });
        let box_param = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: box_param,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let box_alias = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: box_alias,
            name: Some("BoxAlias".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(box_param)),
            type_params: vec![box_param],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let boxed_secret = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: boxed_secret,
            name: Some("BoxedSecret".to_string()),
            connective: TypeConnective::Instantiation {
                template: box_alias,
                arguments: vec![TemplateArgument {
                    parameter: box_param,
                    value: secret,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let input = dag.alloc_port_with_shape(TypeShape::new(boxed_secret));
        let output = dag.push_transform(
            TransformTarget::UnresolvedFieldProject {
                field_label: "value".to_string(),
            },
            vec![input],
            span,
        );

        infer(&mut dag);

        assert!(matches!(
            dag.diagnostics().get(output),
            Some(Diagnostic::NominalOpacityViolation {
                declaration,
                accessor: None,
                ..
            }) if *declaration == secret
        ));
    }

    #[test]
    fn field_project_without_nominal_opacity_remains_structural() {
        let mut dag = Dag::new();
        let span = SourceSpan::new("<nominal-opacity-absent-test>", 0, 1);
        let int = dag.int_shape().expect("bootstrap Int").declaration;

        let record = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: record,
            name: Some("PlainPayload".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "value".to_string(),
                    ty: int,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let input = dag.alloc_port_with_shape(TypeShape::new(record));
        let output = dag.push_transform(
            TransformTarget::UnresolvedFieldProject {
                field_label: "value".to_string(),
            },
            vec![input],
            span,
        );

        infer(&mut dag);

        assert!(
            dag.diagnostics().get(output).is_none(),
            "plain structural projection should not emit opacity diagnostics"
        );
        assert!(matches!(
            dag.port(output).state(),
            PortState::Resolved(shape) if shape.declaration == int
        ));
    }

    #[test]
    fn bootstrapped_secret_is_marked_nominal_opaque() {
        let dag = Dag::new();
        let secret = dag
            .declaration_by_name("Secret")
            .expect("bootstrap Secret declaration");
        assert!(
            secret.nominal_opacity.is_some(),
            "Secret must consume the nominal-opacity carrier before Modeling dispatch"
        );
    }

    #[test]
    fn arithmetic_on_non_algebra_lhs_fails_closed_instead_of_synthesizing_arrow() {
        // Director audit (post-merge finding): pre-fix, when the
        // structural walk in `resolve_operator_arrow` did not encounter
        // any algebra Conj on the LHS (e.g. an Arrow / Disj / Atom with
        // no `inhabits` link), the fallback fabricated `(T,T)->T` for
        // arithmetic and `(T,T)->Bool` for comparison. That invented
        // an operator-rule arrow contract for a type that declares
        // none. With the fail-closed check in place, the resolver
        // returns None so the call site emits a typed
        // unsupported-operator diagnostic.
        let mut dag = Dag::new();
        let span = SourceSpan::new("<non-algebra-lhs-test>", 0, 1);
        let int = dag.int_shape().expect("bootstrap Int").declaration;

        // A non-algebra LHS: an anonymous Arrow declaration. No
        // `inhabits`, no Conj on the walk path.
        let arrow_decl = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: arrow_decl,
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![int],
                output: int,
                body: ArrowBody::NoBody,
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let lhs = TypeShape::new(arrow_decl);

        assert!(
            resolve_operator_arrow(&mut dag, OperatorKind::Arithmetic(ArithmeticOp::Add), &lhs,)
                .is_none(),
            "`+` on a non-algebra LHS must fail-closed: the pre-fix \
             fallback fabricated `(Arrow,Arrow)->Arrow` instead of \
             surfacing an unsupported-operator diagnostic"
        );
        assert!(
            resolve_operator_arrow(&mut dag, OperatorKind::Comparison(ComparisonOp::Lt), &lhs,)
                .is_none(),
            "`<` on a non-algebra LHS must fail-closed: the pre-fix \
             fallback fabricated `(Arrow,Arrow)->Bool` instead of \
             surfacing an unsupported-operator diagnostic"
        );
        assert!(
            resolve_operator_arrow(&mut dag, OperatorKind::Arithmetic(ArithmeticOp::Div), &lhs,)
                .is_none(),
            "`/` on a non-algebra LHS must fail-closed: the pre-fix \
             Div fallback synthesized `(Arrow,Arrow)->Result<Arrow,DivError>`"
        );
        assert!(
            resolve_operator_arrow(&mut dag, OperatorKind::Logical(LogicalOp::And), &lhs,)
                .is_none(),
            "`&&` on a non-algebra LHS must fail-closed: the pre-fix \
             logical fallback produced `(Bool,Bool)->Bool` regardless \
             of LHS, inventing a carrier the type does not declare"
        );
    }

    #[test]
    fn bool_logical_and_resolves_via_boolean_algebra_meet_not_pending_fallback() {
        let mut dag = Dag::new();
        let bool_shape = dag.bool_shape().expect("bootstrap Bool");
        let sig =
            resolve_operator_arrow(&mut dag, OperatorKind::Logical(LogicalOp::And), &bool_shape)
                .expect("&& should resolve on Bool");
        assert!(
            !matches!(sig.body, ArrowBody::Pending),
            "expected Bool && via inhabits → BooleanAlgebra.meet, not Pending scaffold; got {:?}",
            sig.body
        );
        assert_eq!(sig.inputs.len(), 2);
        assert_eq!(sig.inputs[0].declaration, bool_shape.declaration);
        assert_eq!(sig.inputs[1].declaration, bool_shape.declaration);
        assert_eq!(sig.output.declaration, bool_shape.declaration);
    }

    #[test]
    fn bool_logical_or_resolves_via_boolean_algebra_join_not_pending_fallback() {
        let mut dag = Dag::new();
        let bool_shape = dag.bool_shape().expect("bootstrap Bool");
        let sig =
            resolve_operator_arrow(&mut dag, OperatorKind::Logical(LogicalOp::Or), &bool_shape)
                .expect("|| should resolve on Bool");
        assert!(
            !matches!(sig.body, ArrowBody::Pending),
            "expected Bool || via inhabits → BooleanAlgebra.join, not Pending scaffold; got {:?}",
            sig.body
        );
        assert_eq!(sig.inputs.len(), 2);
        assert_eq!(sig.inputs[0].declaration, bool_shape.declaration);
        assert_eq!(sig.inputs[1].declaration, bool_shape.declaration);
        assert_eq!(sig.output.declaration, bool_shape.declaration);
    }

    #[test]
    fn arithmetic_operator_checks_abelian_phantom_unit_closure_before_type_mismatch() {
        let mut dag = Dag::new();
        let span = SourceSpan::new("<money-unit-test>", 0, 1);
        let int = dag.int_shape().expect("bootstrap Int").declaration;
        let abelian_group = dag
            .declaration_by_name("AbelianGroup")
            .expect("bootstrap AbelianGroup")
            .id;

        let currency = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: currency,
            name: Some("Currency".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: usd,
            name: Some("USD".to_string()),
            connective: TypeConnective::Instantiation {
                template: currency,
                arguments: vec![],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let eur = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: eur,
            name: Some("EUR".to_string()),
            connective: TypeConnective::Instantiation {
                template: currency,
                arguments: vec![],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let currency_abelian_group = dag.alloc_declaration_id();
        let abelian_receiver = dag
            .declaration(abelian_group)
            .type_params
            .first()
            .copied()
            .expect("AbelianGroup receiver type parameter");
        dag.push_declaration(Declaration {
            id: currency_abelian_group,
            name: Some("CurrencyAbelianGroup".to_string()),
            connective: TypeConnective::Instantiation {
                template: abelian_group,
                arguments: vec![TemplateArgument {
                    parameter: abelian_receiver,
                    value: currency,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let int_abelian_group = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: int_abelian_group,
            name: Some("IntAbelianGroup".to_string()),
            connective: TypeConnective::Instantiation {
                template: abelian_group,
                arguments: vec![TemplateArgument {
                    parameter: abelian_receiver,
                    value: int,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let money = dag.alloc_declaration_id();
        let c_param = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: money,
            name: Some("Money".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "amount".to_string(),
                    ty: int,
                }],
            },
            type_params: vec![c_param],
            phantom_params: vec![PhantomParameter {
                parameter: c_param,
                algebra: currency_abelian_group,
            }],
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        dag.push_declaration(Declaration {
            id: c_param,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("C".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let money_usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: money_usd,
            name: Some("MoneyUSD".to_string()),
            connective: TypeConnective::Instantiation {
                template: money,
                arguments: vec![TemplateArgument {
                    parameter: c_param,
                    value: usd,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let money_eur = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: money_eur,
            name: Some("MoneyEUR".to_string()),
            connective: TypeConnective::Instantiation {
                template: money,
                arguments: vec![TemplateArgument {
                    parameter: c_param,
                    value: eur,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let money_c = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: money_c,
            name: Some("MoneyC".to_string()),
            connective: TypeConnective::Instantiation {
                template: money,
                arguments: vec![TemplateArgument {
                    parameter: c_param,
                    value: c_param,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let alt_c_param = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: alt_c_param,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("AltC".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let alt_money = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: alt_money,
            name: Some("AltMoney".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "amount".to_string(),
                    ty: int,
                }],
            },
            type_params: vec![alt_c_param],
            phantom_params: vec![PhantomParameter {
                parameter: alt_c_param,
                algebra: currency_abelian_group,
            }],
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let alt_money_usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: alt_money_usd,
            name: Some("AltMoneyUSD".to_string()),
            connective: TypeConnective::Instantiation {
                template: alt_money,
                arguments: vec![TemplateArgument {
                    parameter: alt_c_param,
                    value: usd,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let alt_money_eur = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: alt_money_eur,
            name: Some("AltMoneyEUR".to_string()),
            connective: TypeConnective::Instantiation {
                template: alt_money,
                arguments: vec![TemplateArgument {
                    parameter: alt_c_param,
                    value: eur,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let other_algebra_money = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: other_algebra_money,
            name: Some("OtherAlgebraMoney".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "amount".to_string(),
                    ty: int,
                }],
            },
            type_params: vec![alt_c_param],
            phantom_params: vec![PhantomParameter {
                parameter: alt_c_param,
                algebra: int_abelian_group,
            }],
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let other_algebra_money_usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: other_algebra_money_usd,
            name: Some("OtherAlgebraMoneyUSD".to_string()),
            connective: TypeConnective::Instantiation {
                template: other_algebra_money,
                arguments: vec![TemplateArgument {
                    parameter: alt_c_param,
                    value: usd,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let lhs = dag.alloc_port_with_shape(TypeShape::new(money_usd));
        let rhs_same = dag.alloc_port_with_shape(TypeShape::new(money_usd));
        let rhs_other = dag.alloc_port_with_shape(TypeShape::new(money_eur));
        let cross_template_rhs_other = dag.alloc_port_with_shape(TypeShape::new(alt_money_eur));
        let ok_output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, rhs_same],
            span.clone(),
        );
        let bad_output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, rhs_other],
            span.clone(),
        );
        let cross_template_bad_output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, cross_template_rhs_other],
            span.clone(),
        );
        assert!(
            type_shapes_equivalent(
                &dag,
                &TypeShape::new(money_usd),
                &TypeShape::new(alt_money_usd)
            ),
            "structural equivalence should map phantom axes across equivalent templates"
        );
        assert!(
            !type_shapes_equivalent(
                &dag,
                &TypeShape::new(money_usd),
                &TypeShape::new(other_algebra_money_usd),
            ),
            "structural equivalence must compare mapped phantom algebra authority"
        );
        let money_missing_arg = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: money_missing_arg,
            name: Some("MoneyMissingArg".to_string()),
            connective: TypeConnective::Instantiation {
                template: money,
                arguments: Vec::new(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let malformed_lhs = dag.alloc_port_with_shape(TypeShape::new(money_missing_arg));
        let malformed_rhs = dag.alloc_port_with_shape(TypeShape::new(money_missing_arg));
        let malformed_output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![malformed_lhs, malformed_rhs],
            span.clone(),
        );
        let rogue_param = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: rogue_param,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("Rogue".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let bad_money = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: bad_money,
            name: Some("BadMoney".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "amount".to_string(),
                    ty: int,
                }],
            },
            type_params: vec![c_param],
            phantom_params: vec![PhantomParameter {
                parameter: rogue_param,
                algebra: currency_abelian_group,
            }],
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let bad_money_usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: bad_money_usd,
            name: Some("BadMoneyUSD".to_string()),
            connective: TypeConnective::Instantiation {
                template: bad_money,
                arguments: vec![TemplateArgument {
                    parameter: rogue_param,
                    value: usd,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span,
        });
        let invalid_lhs = dag.alloc_port_with_shape(TypeShape::new(bad_money_usd));
        let invalid_rhs = dag.alloc_port_with_shape(TypeShape::new(bad_money_usd));
        let invalid_output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![invalid_lhs, invalid_rhs],
            SourceSpan::new("<money-unit-test>", 1, 2),
        );
        let unsupported_algebra = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: unsupported_algebra,
            name: Some("CurrencyMonoid".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 2, 3),
        });
        let unsupported_money = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: unsupported_money,
            name: Some("UnsupportedMoney".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "amount".to_string(),
                    ty: int,
                }],
            },
            type_params: vec![c_param],
            phantom_params: vec![PhantomParameter {
                parameter: c_param,
                algebra: unsupported_algebra,
            }],
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 2, 3),
        });
        let unsupported_money_usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: unsupported_money_usd,
            name: Some("UnsupportedMoneyUSD".to_string()),
            connective: TypeConnective::Instantiation {
                template: unsupported_money,
                arguments: vec![TemplateArgument {
                    parameter: c_param,
                    value: usd,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 2, 3),
        });
        let unsupported_lhs = dag.alloc_port_with_shape(TypeShape::new(unsupported_money_usd));
        let unsupported_rhs = dag.alloc_port_with_shape(TypeShape::new(unsupported_money_usd));
        let unsupported_output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![unsupported_lhs, unsupported_rhs],
            SourceSpan::new("<money-unit-test>", 2, 3),
        );
        let fake_abelian_group = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: fake_abelian_group,
            name: Some("AbelianGroup".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 3, 4),
        });
        let fake_algebra_money = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: fake_algebra_money,
            name: Some("FakeAlgebraMoney".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "amount".to_string(),
                    ty: int,
                }],
            },
            type_params: vec![c_param],
            phantom_params: vec![PhantomParameter {
                parameter: c_param,
                algebra: fake_abelian_group,
            }],
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 3, 4),
        });
        let fake_algebra_money_usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: fake_algebra_money_usd,
            name: Some("FakeAlgebraMoneyUSD".to_string()),
            connective: TypeConnective::Instantiation {
                template: fake_algebra_money,
                arguments: vec![TemplateArgument {
                    parameter: c_param,
                    value: usd,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 3, 4),
        });
        let fake_lhs = dag.alloc_port_with_shape(TypeShape::new(fake_algebra_money_usd));
        let fake_rhs = dag.alloc_port_with_shape(TypeShape::new(fake_algebra_money_usd));
        let fake_output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![fake_lhs, fake_rhs],
            SourceSpan::new("<money-unit-test>", 3, 4),
        );
        let accept_eur = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: accept_eur,
            name: Some("accept_eur".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![money_eur],
                output: money_eur,
                body: ArrowBody::NoBody,
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 3, 4),
        });
        let callable_lhs = dag.alloc_port_with_shape(TypeShape::new(money_usd));
        let callable_output = dag.push_transform(
            TransformTarget::Callable(accept_eur),
            vec![callable_lhs],
            SourceSpan::new("<money-unit-test>", 3, 4),
        );
        let generic_accept_money = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: generic_accept_money,
            name: Some("accept_money".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![money_c],
                output: money_c,
                body: ArrowBody::NoBody,
            },
            type_params: vec![c_param],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 4, 5),
        });
        let generic_callable_lhs = dag.alloc_port_with_shape(TypeShape::new(money_usd));
        let generic_callable_output = dag.push_transform(
            TransformTarget::Callable(generic_accept_money),
            vec![generic_callable_lhs],
            SourceSpan::new("<money-unit-test>", 4, 5),
        );
        let untracked_param = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: untracked_param,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("U".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 4, 5),
        });
        let untracked_money = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: untracked_money,
            name: Some("UntrackedMoney".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "amount".to_string(),
                    ty: int,
                }],
            },
            type_params: vec![untracked_param],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 4, 5),
        });
        let untracked_money_usd = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: untracked_money_usd,
            name: Some("UntrackedMoneyUSD".to_string()),
            connective: TypeConnective::Instantiation {
                template: untracked_money,
                arguments: vec![TemplateArgument {
                    parameter: untracked_param,
                    value: usd,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("<money-unit-test>", 4, 5),
        });
        let untracked_callable_lhs = dag.alloc_port_with_shape(TypeShape::new(untracked_money_usd));
        let untracked_callable_output = dag.push_transform(
            TransformTarget::Callable(accept_eur),
            vec![untracked_callable_lhs],
            SourceSpan::new("<money-unit-test>", 4, 5),
        );
        assert!(
            !type_shapes_equivalent(
                &dag,
                &TypeShape::new(untracked_money_usd),
                &TypeShape::new(money_eur),
            ),
            "structural equivalence must preserve phantom axes from both templates"
        );

        infer(&mut dag);

        assert_eq!(
            dag.port(ok_output).state(),
            &PortState::Resolved(TypeShape::new(money_usd))
        );
        let bad_diag = dag
            .diagnostics()
            .get(bad_output)
            .expect("mismatched phantom unit should fail on operator output");
        assert!(
            matches!(
                bad_diag,
                Diagnostic::UnitMismatch {
                    parameter,
                    expected,
                    actual,
                    ..
                } if parameter == "C"
                    && expected.declaration == usd
                    && actual.declaration == eur
            ),
            "unexpected diagnostic: {bad_diag:?}"
        );
        let cross_template_bad_diag = dag
            .diagnostics()
            .get(cross_template_bad_output)
            .expect("mapped phantom units should fail across structurally equivalent templates");
        assert!(
            matches!(
                cross_template_bad_diag,
                Diagnostic::UnitMismatch {
                    parameter,
                    expected,
                    actual,
                    ..
                } if parameter == "C"
                    && expected.declaration == usd
                    && actual.declaration == eur
            ),
            "unexpected diagnostic: {cross_template_bad_diag:?}"
        );
        let malformed_diag = dag
            .diagnostics()
            .get(malformed_output)
            .expect("missing phantom instantiation argument should fail closed");
        assert!(
            matches!(
                malformed_diag,
                Diagnostic::ResolveError { name, .. }
                    if name.contains("missing expected argument for phantom parameter C")
            ),
            "unexpected diagnostic: {malformed_diag:?}"
        );
        let invalid_diag = dag
            .diagnostics()
            .get(invalid_output)
            .expect("phantom parameter outside type_params should fail closed");
        assert!(
            matches!(
                invalid_diag,
                Diagnostic::ResolveError { name, .. }
                    if name.contains("parameter is not in type_params")
            ),
            "unexpected diagnostic: {invalid_diag:?}"
        );
        let unsupported_diag = dag
            .diagnostics()
            .get(unsupported_output)
            .expect("unsupported phantom algebra should fail closed");
        assert!(
            matches!(
                unsupported_diag,
                Diagnostic::ResolveError { name, .. }
                    if name.contains("unsupported phantom algebra CurrencyMonoid")
            ),
            "unexpected diagnostic: {unsupported_diag:?}"
        );
        let fake_diag = dag
            .diagnostics()
            .get(fake_output)
            .expect("non-canonical declaration named AbelianGroup should fail closed");
        assert!(
            matches!(
                fake_diag,
                Diagnostic::ResolveError { name, .. }
                    if name.contains("unsupported phantom algebra AbelianGroup")
            ),
            "unexpected diagnostic: {fake_diag:?}"
        );
        let callable_diag = dag
            .diagnostics()
            .get(callable_output)
            .expect("callable boundary should enforce phantom units");
        assert!(
            matches!(
                callable_diag,
                Diagnostic::UnitMismatch {
                    operator,
                    parameter,
                    expected,
                    actual,
                    ..
                } if operator == "accept_eur"
                    && parameter == "C"
                    && expected.declaration == eur
                    && actual.declaration == usd
            ),
            "unexpected diagnostic: {callable_diag:?}"
        );
        assert!(
            matches!(
                dag.port(generic_callable_output).state(),
                PortState::Resolved(_)
            ),
            "generic callable should not fail phantom unit checking before C can bind"
        );
        assert!(
            !matches!(
                dag.diagnostics().get(generic_callable_output),
                Some(Diagnostic::UnitMismatch { .. })
            ),
            "generic callable must not emit UnitMismatch before C can bind"
        );
        let untracked_callable_diag = dag
            .diagnostics()
            .get(untracked_callable_output)
            .expect("structural wrapper without phantom axis should not satisfy phantom wrapper");
        assert!(
            matches!(
                untracked_callable_diag,
                Diagnostic::TypeMismatch { .. } | Diagnostic::ResolveError { .. }
            ),
            "unexpected diagnostic: {untracked_callable_diag:?}"
        );
    }

    #[test]
    fn template_argument_reconciliation_helpers_match_and_update_consistently() {
        let dag = Dag::new();
        let p0 = dag.bool_shape().expect("bootstrap Bool").declaration;
        let p1 = dag.int_shape().expect("bootstrap Int").declaration;
        let v0 = dag.string_shape().expect("bootstrap String").declaration;
        let v1 = dag.bool_shape().expect("bootstrap Bool").declaration;
        let mut arguments = vec![
            TemplateArgument {
                parameter: p0,
                value: p0,
            },
            TemplateArgument {
                parameter: p1,
                value: v1,
            },
        ];

        assert!(push_template_argument_binding(&mut arguments, p0, v0));
        assert!(matches!(
            generated_template_arguments_match(
                &arguments,
                &[
                    TemplateArgument {
                        parameter: p0,
                        value: v0,
                    },
                    TemplateArgument {
                        parameter: p1,
                        value: v1,
                    },
                ]
            ),
            TemplateArgumentsMatch::Match,
        ));
        assert!(!push_template_argument_binding(&mut arguments, p1, v0));
    }

    #[test]
    fn template_arguments_match_generated_projection_agrees_across_length_and_value_cases() {
        let dag = Dag::new();
        let p0 = dag.bool_shape().expect("bootstrap Bool").declaration;
        let p1 = dag.int_shape().expect("bootstrap Int").declaration;
        let v0 = dag.string_shape().expect("bootstrap String").declaration;
        let v1 = dag.bool_shape().expect("bootstrap Bool").declaration;
        let pair = |parameter, value| TemplateArgument { parameter, value };
        let is_match = |lhs: &[TemplateArgument], rhs: &[TemplateArgument]| -> bool {
            matches!(
                generated_template_arguments_match(lhs, rhs),
                TemplateArgumentsMatch::Match,
            )
        };

        let empty: Vec<TemplateArgument> = vec![];
        assert!(is_match(&empty, &empty));
        assert!(!is_match(&empty, &[pair(p0, v0)]));
        assert!(!is_match(&[pair(p0, v0)], &empty));

        let two = vec![pair(p0, v0), pair(p1, v1)];
        assert!(is_match(&two, &two.clone()));
        assert!(!is_match(&two, &[pair(p0, v0), pair(p1, v0)]));
        assert!(!is_match(&two, &[pair(p0, v0), pair(p0, v1)]));
    }

    #[test]
    fn filter_non_self_template_arguments_matches_handwritten_filter() {
        let dag = Dag::new();
        let p0 = dag.bool_shape().expect("bootstrap Bool").declaration;
        let p1 = dag.int_shape().expect("bootstrap Int").declaration;
        let args = vec![
            TemplateArgument {
                parameter: p0,
                value: p0,
            },
            TemplateArgument {
                parameter: p1,
                value: p0,
            },
            TemplateArgument {
                parameter: p1,
                value: p1,
            },
        ];
        let expected: Vec<TemplateArgument> = args
            .iter()
            .filter(|a| a.parameter != a.value)
            .cloned()
            .collect();
        let actual = filter_non_self_template_arguments(&args);
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected.iter()) {
            assert_eq!(a.parameter, e.parameter);
            assert_eq!(a.value, e.value);
        }
    }

    /// SG-4b (callable-instantiation normalization cluster): exercise
    /// `find_equivalent_anonymous_instantiation` through the real
    /// declaration-table path so the generated template-argument
    /// comparison is the one doing the dedup.
    #[test]
    fn find_equivalent_anonymous_instantiation_dedups_via_generated_comparison() {
        let mut dag = Dag::new();
        let template = dag.bool_shape().expect("bootstrap Bool").declaration;
        let p0 = dag.int_shape().expect("bootstrap Int").declaration;
        let p1 = dag.string_shape().expect("bootstrap String").declaration;
        let v0 = dag.int_shape().expect("bootstrap Int").declaration;
        let v1 = dag.bool_shape().expect("bootstrap Bool").declaration;

        let args = vec![
            TemplateArgument {
                parameter: p0,
                value: v0,
            },
            TemplateArgument {
                parameter: p1,
                value: v1,
            },
        ];

        let anon_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: anon_id,
            name: None,
            connective: TypeConnective::Instantiation {
                template,
                arguments: args.clone(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: synthetic_span(),
        });

        // Exact arguments match → dedup hit.
        assert_eq!(
            find_equivalent_anonymous_instantiation(&dag, template, &args, None),
            Some(anon_id),
        );

        // One value differs → no hit.
        let mut value_diff = args.clone();
        value_diff[1].value = v0;
        assert_eq!(
            find_equivalent_anonymous_instantiation(&dag, template, &value_diff, None),
            None,
        );

        // Length differs → no hit.
        assert_eq!(
            find_equivalent_anonymous_instantiation(&dag, template, &args[..1], None),
            None,
        );

        let opaque = NominalOpacity {
            permitted_accessors: Vec::new(),
        };
        assert_eq!(
            find_equivalent_anonymous_instantiation(&dag, template, &args, Some(&opaque)),
            None,
            "anonymous instantiation interning must not collapse distinct nominal-opacity carriers"
        );

        let opaque_anon_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: opaque_anon_id,
            name: None,
            connective: TypeConnective::Instantiation {
                template,
                arguments: args.clone(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: Some(opaque.clone()),
            span: synthetic_span(),
        });
        assert_eq!(
            find_equivalent_anonymous_instantiation(&dag, template, &args, Some(&opaque)),
            Some(opaque_anon_id),
        );
        let opaque_with_accessor = NominalOpacity {
            permitted_accessors: vec![p0],
        };
        assert_eq!(
            find_equivalent_anonymous_instantiation(
                &dag,
                template,
                &args,
                Some(&opaque_with_accessor),
            ),
            None,
            "permitted-accessor changes are part of the nominal-opacity interning key"
        );

        // Self-binding normalization: a `parameter == value` entry is a
        // real structural distinction at this call site —
        // `find_equivalent_anonymous_instantiation` does not strip self
        // bindings itself; `normalized_instantiation_args` owns that.
        // An instantiation with a self-binding added does not dedup
        // against one without it, confirming the comparison is strict
        // pairwise and the generated helper's length check fires.
        let mut with_self = args.clone();
        with_self.push(TemplateArgument {
            parameter: p0,
            value: p0,
        });
        assert_eq!(
            find_equivalent_anonymous_instantiation(&dag, template, &with_self, None),
            None,
        );
    }

    #[test]
    fn arithmetic_division_uses_totalized_result_shape() {
        let mut dag = Dag::new();
        let int = dag.int_shape().expect("bootstrap Int").declaration;
        let lhs = TypeShape::new(int);
        let first =
            resolve_operator_arrow(&mut dag, OperatorKind::Arithmetic(ArithmeticOp::Div), &lhs)
                .expect("division must resolve");
        let after_first = dag.declarations().len();
        let second =
            resolve_operator_arrow(&mut dag, OperatorKind::Arithmetic(ArithmeticOp::Div), &lhs)
                .expect("division must resolve");

        assert_eq!(first.output.declaration, second.output.declaration);
        assert_eq!(
            after_first,
            dag.declarations().len(),
            "div resolution should dedup anonymous instantiation",
        );

        let result_shape_decl = dag.declaration(first.output.declaration);
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &result_shape_decl.connective
        else {
            panic!("division output must remain an instantiation");
        };
        let std_result = canonical_result_decl(&dag).expect("std Result").id;
        let std_div_error = canonical_div_error_decl(&dag).expect("std DivError").id;
        assert_eq!(*template, std_result);
        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].value, int);
        assert_eq!(arguments[1].value, std_div_error);
    }

    #[test]
    fn arithmetic_division_canonical_result_and_diverror_are_structural_not_path_keyed() {
        let mut dag = Dag::new();
        let int = dag.int_shape().expect("bootstrap Int").declaration;
        let result = canonical_result_decl(&dag).expect("bootstrap Result").id;
        let div_error = canonical_div_error_decl(&dag)
            .expect("bootstrap DivError")
            .id;

        dag.declaration_mut(result).span.file = "renamed/std/errors.dag".to_string();
        dag.declaration_mut(div_error).span.file = "renamed/std/errors.dag".to_string();

        let resolved = resolve_operator_arrow(
            &mut dag,
            OperatorKind::Arithmetic(ArithmeticOp::Div),
            &TypeShape::new(int),
        )
        .expect("division must resolve through structural error primitives");
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &dag.declaration(resolved.output.declaration).connective
        else {
            panic!("division output must remain an instantiation");
        };

        assert_eq!(*template, result);
        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].value, int);
        assert_eq!(arguments[1].value, div_error);
        assert_eq!(
            canonical_result_decl(&dag).expect("structural Result").id,
            result
        );
        assert_eq!(
            canonical_div_error_decl(&dag)
                .expect("structural DivError")
                .id,
            div_error
        );
    }

    #[test]
    fn arithmetic_division_ignores_user_shadowed_result_carriers() {
        let mut dag = Dag::new();
        let int = dag.int_shape().expect("bootstrap Int").declaration;
        let span = synthetic_span();

        let shadow_ok = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: shadow_ok,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("Ok".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let shadow_err = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: shadow_err,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("Err".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let shadow_result = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: shadow_result,
            name: Some("Result".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Ok".to_string(),
                        ty: shadow_ok,
                    },
                    Field {
                        label: "Err".to_string(),
                        ty: shadow_err,
                    },
                ],
            },
            type_params: vec![shadow_ok, shadow_err],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let shadow_div_error = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: shadow_div_error,
            name: Some("DivError".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "DivideByZero".to_string(),
                        ty: int,
                    },
                    Field {
                        label: "Overflow".to_string(),
                        ty: int,
                    },
                ],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span,
        });
        assert!(
            !substrate_div_error_type_decl_suppressed_for_emit(
                &dag,
                dag.declaration(shadow_div_error)
            ),
            "payload-bearing DivError twin must not match the canonical unit-variant fingerprint",
        );

        let resolved = resolve_operator_arrow(
            &mut dag,
            OperatorKind::Arithmetic(ArithmeticOp::Div),
            &TypeShape::new(int),
        )
        .expect("division must resolve through std carrier");
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &dag.declaration(resolved.output.declaration).connective
        else {
            panic!("division output must remain an instantiation");
        };

        assert_ne!(*template, shadow_result);
        assert_ne!(arguments[1].value, shadow_div_error);
        assert_eq!(
            *template,
            canonical_result_decl(&dag).expect("std Result").id
        );
        assert_eq!(
            arguments[1].value,
            canonical_div_error_decl(&dag).expect("std DivError").id
        );
    }

    #[test]
    fn arithmetic_division_prefers_bootstrap_error_primitives_over_exact_twins() {
        let mut dag = Dag::new();
        let int = dag.int_shape().expect("bootstrap Int").declaration;
        let std_result = canonical_result_decl(&dag).expect("std Result").id;
        let std_div_error = canonical_div_error_decl(&dag).expect("std DivError").id;
        let span = synthetic_span();

        let twin_ok = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_ok,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("ok".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let twin_err = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_err,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("err".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let twin_ok_payload = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_ok_payload,
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "value".to_string(),
                    ty: twin_ok,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let twin_err_payload = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_err_payload,
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "value".to_string(),
                    ty: twin_err,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let twin_result = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_result,
            name: Some("Result".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Ok".to_string(),
                        ty: twin_ok_payload,
                    },
                    Field {
                        label: "Err".to_string(),
                        ty: twin_err_payload,
                    },
                ],
            },
            type_params: vec![twin_ok, twin_err],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let twin_divide_by_zero = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_divide_by_zero,
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let twin_overflow = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_overflow,
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });
        let twin_div_error = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: twin_div_error,
            name: Some("DivError".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "DivideByZero".to_string(),
                        ty: twin_divide_by_zero,
                    },
                    Field {
                        label: "Overflow".to_string(),
                        ty: twin_overflow,
                    },
                ],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span,
        });

        assert!(substrate_result_type_decl_suppressed_for_emit(
            &dag,
            dag.declaration(twin_result)
        ));
        assert!(substrate_div_error_type_decl_suppressed_for_emit(
            &dag,
            dag.declaration(twin_div_error)
        ));
        assert_eq!(canonical_result_decl(&dag).expect("Result").id, std_result);
        assert_eq!(
            canonical_div_error_decl(&dag).expect("DivError").id,
            std_div_error
        );

        let resolved = resolve_operator_arrow(
            &mut dag,
            OperatorKind::Arithmetic(ArithmeticOp::Div),
            &TypeShape::new(int),
        )
        .expect("division must resolve through bootstrap error primitives");
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &dag.declaration(resolved.output.declaration).connective
        else {
            panic!("division output must remain an instantiation");
        };
        assert_eq!(*template, std_result);
        assert_eq!(arguments[1].value, std_div_error);
        assert_ne!(*template, twin_result);
        assert_ne!(arguments[1].value, twin_div_error);
    }

    #[test]
    fn read_algebra_field_fails_closed_on_unresolved_instantiation_argument() {
        let mut dag = Dag::new();
        let span = synthetic_span();
        let int = dag.int_shape().expect("bootstrap Int").declaration;

        let receiver_param = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: receiver_param,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("Self".to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let unresolved = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: unresolved,
            name: None,
            connective: TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(
                "Missing".to_string(),
            )),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let arithmetic_field = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: arithmetic_field,
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![receiver_param, unresolved],
                output: int,
                body: ArrowBody::NoBody,
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let algebra = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: algebra,
            name: Some("DivProbeAlgebra".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "div".to_string(),
                    ty: arithmetic_field,
                }],
            },
            type_params: vec![receiver_param],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span.clone(),
        });

        let before = dag.declarations().len();
        let algebra_decl = dag.declaration(algebra).clone();
        assert!(read_algebra_field(
            &mut dag,
            &algebra_decl,
            arithmetic_field,
            int,
            OperatorKind::Arithmetic(ArithmeticOp::Div),
            &TypeShape::new(int),
        )
        .is_none());
        assert_eq!(dag.declarations().len(), before);
    }
}
