// Surface → Dag lowering.
//
// Two-pass. Pass 1 walks all top-level items and allocates placeholder
// Declarations for each named type/fn, populating a symbol table
// (name → DeclarationId). Pass 2 fills in each declaration's connective
// and lowers function/let bodies to L1 behaviors, using the symbol table
// to resolve identifier references. See M1_DESIGN.md §8.1.
//
// Computation-side lowering follows M0 semantics unchanged:
//
//   IntLit/BoolLit/StringLit → Value(LiteralBits::*)
//   Var (local)              → scope lookup
//   Var (unresolved)         → placeholder port + ResolveError
//   Call                     → Transform { target: TransformTarget::Callable(DeclarationId), inputs }
//   Operator                 → Transform { target: TransformTarget::Operator(OperatorKind), inputs }
//   If                       → Branch with 2 Paths
//   Fn item                  → Bind with non-empty params + optional Loop wrapper
//   Let item                 → Bind with empty params
//
// `TransformTarget::Callable` points at a DeclarationId for user
// function calls and resolved named declarations. `TransformTarget::Operator`
// carries a structural `OperatorKind` directly — no anonymous stub
// declaration, no name-based inhabitance walk at infer time.

use std::collections::{HashMap, HashSet};

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BindNode, Bound, BranchNode, BranchPattern, CardinalityBound,
    Dag, Declaration, DeclarationId, Field, LiteralBits, LoopNode, NodeId, Path, PayloadBinding,
    PortId, TemplateArgument, TransformNode, TransformTarget, TypeConnective, ValueNode,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::operators::{ArithmeticOp, LogicalOp, OperatorKind};
use crate::parse::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceLiteral, SurfaceModule, SurfaceParam,
    SurfacePattern, SurfaceType, SurfaceVariant, VariantPayload,
};
use crate::types::TypeShape;

type CallableScope = HashMap<String, DeclarationId>;

fn declaration_name_preference_rank(file: &str) -> usize {
    if file.starts_with("src/v3/") {
        2
    } else if file.starts_with("dsl/") {
        0
    } else {
        1
    }
}

#[derive(Default)]
struct ScopeState {
    values: HashMap<String, PortId>,
    callables: CallableScope,
}

struct LambdaLoweringContext<'a> {
    dag: &'a mut Dag,
    scope: &'a HashMap<String, PortId>,
    callable_scope: &'a CallableScope,
    symbols: &'a HashMap<String, DeclarationId>,
}

pub fn lower(module: &SurfaceModule) -> Dag {
    let mut dag = Dag::new();
    let user_start = dag.declarations().len();
    lower_into(&mut dag, module);
    // User-module sweep: every Identifier stub allocated during
    // `lower_into` (id >= user_start) must resolve, or it becomes a
    // fail-closed ResolveError. Stubs in the bootstrap range are
    // resolved opportunistically but tolerated — the canonical std/
    // files (`dsl/std/algebra.dag`, etc.) have dangling references to
    // types that live in std/ modules outside the M1(2.6) load set
    // (e.g., `Tuple`), and those aren't user errors.
    resolve_pending_identifiers_strict(&mut dag, user_start);
    // User-mode scaffold rejection: `FnExternalBody` /
    // `ArrowBody::Unparsed` and `Data` / `ValueBody::Unparsed`
    // are load-bearing scaffolds for the std/bootstrap files
    // whose bodies the M1(2.8) parser cannot yet lower
    // (match / record literals / lambdas / etc.).
    // User-range declarations that rely on the scaffold are
    // fail-closed: ordinary user code has no business shipping
    // an opaque body the compiler cannot validate. Without this
    // sweep, `fn foo(x: Int) -> Int { junk }` would compile
    // cleanly and callers would get Resolved types from an
    // unvalidated body.
    reject_user_unparsed_scaffolds(&mut dag, user_start);
    dag
}

/// Lower a surface module into an existing Dag as a single-shot call.
/// Used by user-code compilation (`lower()` above) where there's only
/// one module in the pass. Bootstrap calls
/// `collect_symbols_phase` + `lower_bodies_phase` separately so it can
/// run phase 1 over ALL std/ files before phase 2 on any of them,
/// which is required for cross-file forward references (e.g.,
/// `bit.dag`'s `Word64 { bytes: List<Byte> }` references `List` from
/// `types.dag`, which loads later).
pub(crate) fn lower_into(dag: &mut Dag, module: &SurfaceModule) {
    let (symbols, is_first) = collect_symbols_phase(dag, &module.items);
    lower_bodies_phase(dag, module, &symbols, &is_first);
}

/// Phase 1 of two-phase lowering. Allocates placeholder declarations
/// and TypeParam children for every top-level named item in `items`,
/// updates the declaration table in place, and returns the symbols
/// map plus the duplicate-detection vector. Safe to call multiple
/// times for different modules — later calls seed their symbols from
/// earlier calls' declarations, so cross-module forward references
/// resolve via `resolve_pending_identifiers` at the end of phase 2.
pub(crate) fn collect_symbols_phase(
    dag: &mut Dag,
    items: &[SurfaceItem],
) -> (HashMap<String, DeclarationId>, Vec<bool>) {
    collect_symbols(dag, items)
}

/// Phase 2 of two-phase lowering. Given the already-allocated
/// placeholder declarations + type_params from phase 1, fills in each
/// item's connective and emits any behavior sub-DAGs. Intentionally
/// does NOT run `resolve_pending_identifiers` — bootstrap batches the
/// sweep across all files, and the user-code `lower()` entry point
/// runs the strict sweep itself.
pub(crate) fn lower_bodies_phase(
    dag: &mut Dag,
    module: &SurfaceModule,
    symbols: &HashMap<String, DeclarationId>,
    is_first: &[bool],
) {
    seed_function_signatures_phase(dag, &module.items, symbols, is_first);
    let mutually_recursive = compute_mutually_recursive(&module.items);
    let mut scope = ScopeState::default();
    // DB-10 (3a.2) ordering: dependency-ordered pre-passes so
    // references to top-level declarations resolve independent of
    // source order.
    //
    //   Pass 1: types (TypeRecord / TypeSum / TypeAlias / TypeAtom)
    //           — pure type connectives; don't reference values.
    //   Pass 2: data (SurfaceItem::Data) — depend on types for
    //           inhabitance checking; populate `value_body` so later
    //           fn-body `SurfaceExpr::Var` lookups find it.
    //   Pass 3: everything else in source order (fns, let, modules).
    //           Types and data are skipped here because they were
    //           lowered in the pre-passes.
    //
    // Without pass 2, a fn body referencing a later-declared `data`
    // item (e.g. `fn f() -> Int = answer` followed by
    // `data answer: Int = 42`) would resolve `answer` to a
    // declaration whose `value_body` is still `None` and fall
    // through to the unresolved diagnostic — order-dependent name
    // resolution.
    for (idx, item) in module.items.iter().enumerate() {
        if !is_first[idx] {
            continue;
        }
        if matches!(
            item,
            SurfaceItem::TypeRecord { .. }
                | SurfaceItem::TypeSum { .. }
                | SurfaceItem::TypeAlias { .. }
                | SurfaceItem::TypeAtom { .. }
        ) {
            scope = lower_item(item, dag, scope, symbols, &mutually_recursive);
        }
    }
    for (idx, item) in module.items.iter().enumerate() {
        if !is_first[idx] {
            continue;
        }
        if let SurfaceItem::Data {
            name,
            ty,
            body,
            body_span,
            ..
        } = item
        {
            lower_data_item(name, ty, body.as_ref(), body_span, dag, symbols);
        }
    }
    // DB-11 (3a.3) phase-ordered refinement lowering. Runs AFTER the
    // data pre-pass so predicates referencing top-level `data`
    // constants resolve against lowered declarations, not placeholders.
    // Sole caller of `lower_parameter_refinement` for parameter
    // `where` clauses (single construction authority).
    lower_parameter_refinements_phase(dag, module, symbols, is_first);
    for (idx, item) in module.items.iter().enumerate() {
        if !is_first[idx] {
            // Duplicate declaration — skipped at lower time so the
            // first-of-name's filled connective is not overwritten.
            // `collect_symbols` already emitted a fail-closed
            // diagnostic for the duplicate.
            continue;
        }
        // Types + Data lowered in the pre-passes above; skip here
        // so their lowering doesn't run twice.
        if matches!(
            item,
            SurfaceItem::Data { .. }
                | SurfaceItem::TypeRecord { .. }
                | SurfaceItem::TypeSum { .. }
                | SurfaceItem::TypeAlias { .. }
                | SurfaceItem::TypeAtom { .. }
        ) {
            continue;
        }
        scope = lower_item(item, dag, scope, symbols, &mutually_recursive);
    }
}

fn seed_function_signatures_phase(
    dag: &mut Dag,
    items: &[SurfaceItem],
    symbols: &HashMap<String, DeclarationId>,
    is_first: &[bool],
) {
    for (idx, item) in items.iter().enumerate() {
        if !is_first[idx] {
            continue;
        }
        match item {
            SurfaceItem::Fn {
                name,
                params,
                return_type,
                ..
            } => {
                seed_function_signature(name, params, return_type, ArrowBody::Pending, dag, symbols)
            }
            SurfaceItem::FnExternalBody {
                name,
                params,
                return_type,
                body_span,
                ..
            } => seed_function_signature(
                name,
                params,
                return_type,
                ArrowBody::Unparsed(body_span.clone()),
                dag,
                symbols,
            ),
            _ => {}
        }
    }
}

/// DB-11 (3a.3): lower a parameter's `where` refinement into a
/// predicate `Declaration` and a refined type `Declaration`.
///
/// - `base_decl_id`: the parameter's declared type declaration
///   (e.g. `Int` for `d: Int where d != 0`).
/// - `predicate`: the parsed surface expression after `where`. Lowered
///   in an isolated scope containing only the parameter (bound to a
///   fresh port typed as `base_decl_id`).
/// - Returns a refined type Declaration whose connective is
///   `Atom(ResolvedIdentifier(base_decl_id))` (structurally
///   equivalent to the base type) with `refinement: Some(pred_decl)`
///   pointing at the predicate declaration.
///
/// The predicate is itself a Declaration with an `Arrow` connective
/// taking the base type and returning `Bool`, with body
/// `UserDefined(bind_id)` pointing at a Bind that owns the predicate
/// sub-DAG. The Bind's `params[0]` is the predicate parameter port —
/// structural equality on predicates treats this slot as "the refined
/// parameter" and pairs it across both sides.
fn lower_parameter_refinement(
    base_decl_id: DeclarationId,
    predicate: &SurfaceExpr,
    param_name: &str,
    symbols: &HashMap<String, DeclarationId>,
    dag: &mut Dag,
    param_span: SourceSpan,
) -> DeclarationId {
    let pred_span = crate::parse::expr_span(predicate).clone();
    let pred_param_port = dag.alloc_port(None);
    match declaration_to_port_shape(base_decl_id, dag, &param_span) {
        Ok(shape) => dag.set_port_type(pred_param_port, shape),
        Err(diag) => dag.mark_unresolved(pred_param_port, diag),
    }

    let mut pred_scope: HashMap<String, PortId> = HashMap::new();
    pred_scope.insert(param_name.to_string(), pred_param_port);
    let pred_callable_scope: CallableScope = CallableScope::new();
    let bool_decl_id = symbols
        .get("Bool")
        .copied()
        .unwrap_or_else(|| alloc_identifier_stub(dag, "Bool", &pred_span));
    let pred_value_port = lower_expr(
        predicate,
        dag,
        &pred_scope,
        &pred_callable_scope,
        symbols,
        Some(bool_decl_id),
    );

    let bind_id = dag.alloc_node_id();
    dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: format!("<refinement:{param_name}>"),
        value: pred_value_port,
        params: vec![pred_param_port],
        span: pred_span.clone(),
    }));

    let pred_decl_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: pred_decl_id,
        name: None,
        connective: TypeConnective::Arrow {
            inputs: vec![base_decl_id],
            output: bool_decl_id,
            body: ArrowBody::UserDefined(bind_id),
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: pred_span,
    });

    let refined_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: refined_id,
        name: None,
        connective: TypeConnective::Atom(AtomPayload::ResolvedIdentifier(base_decl_id)),
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: Some(pred_decl_id),
        span: param_span,
    });
    refined_id
}

fn seed_function_signature(
    name: &str,
    params: &[SurfaceParam],
    return_type: &SurfaceType,
    body: ArrowBody,
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
) {
    let fn_decl_id = symbols[name];
    let local = local_scope_from_parent(dag, fn_decl_id);
    // DB-11 (3a.3) phase-ordering fix: seed the Arrow with BASE
    // declaration ids only. Parameter `where` clauses are lowered by
    // `lower_parameter_refinements_phase`, which runs AFTER the data
    // pre-pass populates top-level `data` declarations' connectives
    // and `value_body`s. Lowering a predicate here would evaluate any
    // references to top-level data constants against placeholder
    // declarations — the references would mark `Unresolved` even
    // though the constants are valid.
    let param_decl_inputs: Vec<DeclarationId> = params
        .iter()
        .map(|p| type_to_declaration_id(&p.ty, symbols, &local, dag))
        .collect();
    let return_decl_id = type_to_declaration_id(return_type, symbols, &local, dag);
    dag.declaration_mut(fn_decl_id).connective = TypeConnective::Arrow {
        inputs: param_decl_inputs,
        output: return_decl_id,
        body,
    };
}

/// DB-11 (3a.3): lower parameter `where` clauses for every Fn /
/// FnExternalBody item and update the fn's Arrow inputs with the
/// refined declaration ids. Runs between the data pre-pass and the
/// main fn-body pass so references inside predicates to top-level
/// `data` constants see fully-lowered declarations (not placeholders)
/// and resolve cleanly.
///
/// Sole caller of `lower_parameter_refinement` for parameter `where`
/// clauses — preserves single construction authority even though
/// the work is split off from seeding. The alternative (lowering
/// refinements at seed time) ordered-broke predicates that referenced
/// top-level data; the alternative (lowering at fn-body time) split
/// the authority across `Fn` (body path) and `FnExternalBody` (which
/// has no body to hook into). One dedicated phase handles both.
fn lower_parameter_refinements_phase(
    dag: &mut Dag,
    module: &SurfaceModule,
    symbols: &HashMap<String, DeclarationId>,
    is_first: &[bool],
) {
    for (idx, item) in module.items.iter().enumerate() {
        if !is_first[idx] {
            continue;
        }
        let (name, params) = match item {
            SurfaceItem::Fn { name, params, .. } => (name, params),
            SurfaceItem::FnExternalBody { name, params, .. } => (name, params),
            _ => continue,
        };
        let fn_decl_id = symbols[name];
        // Read the seeded Arrow's inputs and output (set by
        // `seed_function_signature` to the base declarations).
        let (existing_inputs, output, body) = match &dag.declaration(fn_decl_id).connective {
            TypeConnective::Arrow {
                inputs,
                output,
                body,
            } => (inputs.clone(), *output, body.clone()),
            _ => continue,
        };
        let mut refined_inputs = Vec::with_capacity(params.len());
        for (param, &base_decl) in params.iter().zip(existing_inputs.iter()) {
            let input_decl = match &param.refinement {
                Some(predicate) => match refinement_predicate_out_of_fragment(predicate) {
                    Some((shape_label, span)) => {
                        // DB-11 (3a.3) fail-closed boundary. The
                        // discharge walker (`predicate_discharges` +
                        // `refinement_ports_equal`) and the
                        // composite-narrowing clone
                        // (`clone_predicate_body`) both model only
                        // `Value` and `Transform` predicate bodies.
                        // Admitting a `where` predicate that lowers
                        // through `Branch` / `Loop` / `Bind` would
                        // produce a substrate state the consumers
                        // silently disagree about — discharge would
                        // reject matching shapes as "not equal," and
                        // narrowing would discard the composite.
                        // Reject at the lowering boundary so the
                        // user gets an honest "unsupported shape"
                        // diagnostic instead of a misleading
                        // downstream mismatch.
                        report_declaration_error(
                            dag,
                            Diagnostic::ResolveError {
                                name: format!(
                                    "`where` predicate shape not supported in \
                                     DB-11 3a.3: `{shape_label}` lowers through \
                                     a Branch/Loop/Bind node, but the discharge \
                                     walker and composite-narrowing path only \
                                     support Value and Transform expression \
                                     bodies. Use a direct comparison or a call \
                                     to a Bool-returning helper instead."
                                ),
                                span,
                            },
                        );
                        base_decl
                    }
                    None => lower_parameter_refinement(
                        base_decl,
                        predicate,
                        &param.name,
                        symbols,
                        dag,
                        param.ty.span().clone(),
                    ),
                },
                None => base_decl,
            };
            refined_inputs.push(input_decl);
        }
        dag.declaration_mut(fn_decl_id).connective = TypeConnective::Arrow {
            inputs: refined_inputs,
            output,
            body,
        };
    }
}

/// DB-11 (3a.3) fragment gate. Walks a `where`-predicate
/// `SurfaceExpr` and returns `Some((shape_label, span))` for the
/// first sub-expression whose lowering would produce a `Branch`,
/// `Loop`, or `Bind` behavior — i.e., an out-of-fragment shape that
/// the discharge walker and composite-narrowing clone path cannot
/// compare or reproduce.
///
/// Supported shapes return `None`:
/// - `Literal`, `Var`, `Path` — leaf Value / field-project
///   Transforms.
/// - `Call` / `Operator` — Transform nodes; arguments are recursed
///   into.
///
/// Out-of-fragment shapes:
/// - `If` / `Match` — lower to `Branch`.
/// - `Lambda` — lowers to `Bind`.
/// - `Record` / `List` — lower to fail-closed diagnostics at user
///   scope today; still rejected here so the user sees the refinement
///   boundary clearly rather than the generic lowering failure.
fn refinement_predicate_out_of_fragment(expr: &SurfaceExpr) -> Option<(&'static str, SourceSpan)> {
    match expr {
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } | SurfaceExpr::Path { .. } => None,
        SurfaceExpr::Call { args, .. } | SurfaceExpr::Operator { args, .. } => {
            args.iter().find_map(refinement_predicate_out_of_fragment)
        }
        SurfaceExpr::Lambda { span, .. } => Some(("lambda", span.clone())),
        SurfaceExpr::If { span, .. } => Some(("if", span.clone())),
        SurfaceExpr::Match { span, .. } => Some(("match", span.clone())),
        SurfaceExpr::Record { span, .. } => Some(("record literal", span.clone())),
        SurfaceExpr::List { span, .. } => Some(("list literal", span.clone())),
    }
}

/// DB-11 (3a.3) arm-local refinement narrowing. Given an `if` / match
/// guard expression `cond` and the outer scope, if `cond` is a
/// recognizable predicate on a scope-bound name (e.g. `d != 0` where
/// `d` is in scope), returns a new scope where that name is rebound to
/// a freshly-allocated port whose type carries a refinement matching
/// the predicate. The predicate is lowered identically to a parameter
/// refinement, so call-site structural discharge in
/// `infer::decide_transform` treats the two predicates as equal.
///
/// Returns `None` when the cond doesn't match a narrowing shape, when
/// no scope-bound Var appears in the cond's operand list, or when the
/// outer port's type cannot be recovered. The caller keeps using the
/// unmodified outer scope in that case.
///
/// Scope of recognition (3a.3 M-sized subset): two-argument
/// `SurfaceExpr::Operator` / `SurfaceExpr::Call` forms where exactly
/// one operand is a `Var` bound in the outer scope. More complex
/// predicate shapes (conjunctions, nested calls on the narrowed
/// parameter, negation-of-predicate on else-arm) fall through with
/// `None` — users either add an explicit guard or live without
/// narrowing. Expanding this recognition is 3a.3-follow-up work.
fn narrow_scope_for_predicate(
    cond: &SurfaceExpr,
    scope: &HashMap<String, PortId>,
    symbols: &HashMap<String, DeclarationId>,
    dag: &mut Dag,
) -> Option<HashMap<String, PortId>> {
    let narrow_name = narrowable_var_name(cond, scope)?;
    let &outer_port = scope.get(&narrow_name)?;
    let outer_shape = match dag.port(outer_port).state() {
        crate::dag::PortState::Resolved(ty) => *ty,
        _ => return None,
    };
    // DB-11 (3a.3) composite-canonical narrowing. Walk past any
    // existing refinement to the TRUE BASE; the new refined
    // declaration aliases that base directly, not the outer refined
    // declaration. No alias chain survives. If the outer port already
    // carried a refinement, its predicate body is cloned into the new
    // composite Bind with the refined-parameter slot re-pointed at the
    // composite's fresh param, then `&&`-joined with the freshly
    // lowered cond — the result is one composite predicate Declaration,
    // structurally identical to what a user-written
    // `where outer_pred && new_cond` would produce.
    let (true_base_decl, outer_pred_decl) = walk_to_refinement_base(dag, outer_shape.declaration);
    let refined_decl = build_narrowed_refinement(
        dag,
        symbols,
        true_base_decl,
        outer_pred_decl,
        cond,
        &narrow_name,
    )?;
    let narrow_port = dag.alloc_port(None);
    match declaration_to_port_shape(refined_decl, dag, expr_span(cond)) {
        Ok(shape) => dag.set_port_type(narrow_port, shape),
        Err(_) => return None,
    }
    let mut narrowed = scope.clone();
    narrowed.insert(narrow_name, narrow_port);
    Some(narrowed)
}

/// Walk the `Atom(ResolvedIdentifier(...))` chain from a parameter's
/// declaration, skipping past any refinement-carrier declarations to
/// reach the true base declaration. Returns `(base_decl, deepest_refinement)`
/// where `deepest_refinement` is the top-level refinement that was on
/// the input declaration (if any) — the outer predicate that the
/// caller wants to compose with a new narrowing predicate.
///
/// For a freshly seeded `Int where d != 0` parameter, returns
/// `(Int_decl, Some(d_ne_0_pred_decl))`. For an unrefined `Int`
/// parameter, returns `(Int_decl, None)`. The loop is depth-bounded
/// so pathological cyclic aliases can't hang lowering.
fn walk_to_refinement_base(
    dag: &Dag,
    decl_id: DeclarationId,
) -> (DeclarationId, Option<DeclarationId>) {
    const DEPTH_LIMIT: usize = 32;
    let top_refinement = dag.declaration(decl_id).refinement;
    let mut current = decl_id;
    for _ in 0..DEPTH_LIMIT {
        let decl = dag.declaration(current);
        if decl.refinement.is_none() {
            return (current, top_refinement);
        }
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                current = *next;
            }
            _ => return (current, top_refinement),
        }
    }
    (current, top_refinement)
}

/// Build a refined parameter declaration for a narrowed scope. The
/// returned declaration aliases `true_base_decl` directly (no alias
/// chain) and carries a single composite predicate:
///
/// - If `outer_pred_decl` is `None`, the predicate is the freshly
///   lowered `new_cond` — identical shape to what a bare
///   `where new_cond` parameter clause would produce.
/// - If `outer_pred_decl` is `Some`, the predicate body is
///   `outer_body && new_body`, where `outer_body` is cloned from
///   the outer predicate's `Bind` with its param slot re-pointed at
///   the fresh composite param. Structurally identical to
///   `where outer_pred_source && new_cond_source` if the user had
///   written it that way.
fn build_narrowed_refinement(
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
    true_base_decl: DeclarationId,
    outer_pred_decl: Option<DeclarationId>,
    new_cond: &SurfaceExpr,
    narrow_name: &str,
) -> Option<DeclarationId> {
    let Some(outer_pred_decl) = outer_pred_decl else {
        // No existing refinement — the fresh predicate IS the
        // refinement. Same shape as a seeded `where` clause.
        return Some(lower_parameter_refinement(
            true_base_decl,
            new_cond,
            narrow_name,
            symbols,
            dag,
            expr_span(new_cond).clone(),
        ));
    };

    let pred_span = expr_span(new_cond).clone();
    // Allocate a fresh composite parameter port typed as the true
    // base — the composite predicate's sole parameter slot.
    let composite_param_port = dag.alloc_port(None);
    match declaration_to_port_shape(true_base_decl, dag, &pred_span) {
        Ok(shape) => dag.set_port_type(composite_param_port, shape),
        Err(diag) => dag.mark_unresolved(composite_param_port, diag),
    }

    // Clone the outer predicate's body into the composite's scope,
    // substituting outer's Bind.params[0] → composite_param_port.
    let (outer_param_port, outer_value_port) = outer_predicate_slots(dag, outer_pred_decl)?;
    let outer_cloned_value = clone_predicate_body(
        dag,
        outer_value_port,
        outer_param_port,
        composite_param_port,
        0,
    )?;

    // Lower the new cond in a scope where narrow_name resolves to
    // composite_param_port.
    let mut pred_scope: HashMap<String, PortId> = HashMap::new();
    pred_scope.insert(narrow_name.to_string(), composite_param_port);
    let pred_callable_scope: CallableScope = CallableScope::new();
    let bool_decl_id = symbols
        .get("Bool")
        .copied()
        .unwrap_or_else(|| alloc_identifier_stub(dag, "Bool", &pred_span));
    let new_value_port = lower_expr(
        new_cond,
        dag,
        &pred_scope,
        &pred_callable_scope,
        symbols,
        Some(bool_decl_id),
    );

    // `outer_cloned && new` — single Transform(Logical(And)). The
    // output port inherits Bool shape via `resolve_operator_arrow`
    // during inference.
    let and_node_id = dag.alloc_node_id();
    let and_output = dag.alloc_port(Some(and_node_id));
    dag.push_node(Behavior::Transform(TransformNode {
        id: and_node_id,
        target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
        inputs: vec![outer_cloned_value, new_value_port],
        output: and_output,
        span: pred_span.clone(),
    }));

    // Wrap in the composite Bind; single params slot, composite body.
    let bind_id = dag.alloc_node_id();
    dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: format!("<refinement:{narrow_name}>"),
        value: and_output,
        params: vec![composite_param_port],
        span: pred_span.clone(),
    }));

    // Predicate declaration with Arrow body, same shape as
    // `lower_parameter_refinement` produces — so discharge logic
    // does not need a special case.
    let pred_decl_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: pred_decl_id,
        name: None,
        connective: TypeConnective::Arrow {
            inputs: vec![true_base_decl],
            output: bool_decl_id,
            body: ArrowBody::UserDefined(bind_id),
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: pred_span.clone(),
    });

    // Refined type declaration aliasing the TRUE BASE (not the
    // outer refined decl) — no alias chain.
    let refined_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: refined_id,
        name: None,
        connective: TypeConnective::Atom(AtomPayload::ResolvedIdentifier(true_base_decl)),
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: Some(pred_decl_id),
        span: pred_span,
    });
    Some(refined_id)
}

/// Read a predicate declaration's Bind slots: the sole parameter
/// port (`Bind.params[0]`) and the body value port (`Bind.value`).
/// Returns `None` if the predicate declaration isn't the expected
/// `Arrow { body: UserDefined(bind) }` shape with a single param
/// slot — in which case composite narrowing falls back to treating
/// the outer refinement as absent (safe degradation).
fn outer_predicate_slots(dag: &Dag, pred_decl: DeclarationId) -> Option<(PortId, PortId)> {
    let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &dag.declaration(pred_decl).connective
    else {
        return None;
    };
    let Behavior::Bind(bind) = dag.node(*bind_id) else {
        return None;
    };
    let param = *bind.params.first()?;
    Some((param, bind.value))
}

/// Clone a predicate body sub-DAG, rewriting every reference to
/// `substitute_from` into `substitute_to`. Fresh node ids + port ids
/// are allocated for each cloned node so the result is structurally
/// independent from the source — two composite Binds in the same
/// DAG do not share intermediate ports.
///
/// Walks only `Value` and `Transform` nodes, matching the predicate
/// fragment that `predicates_structurally_equal` already accepts.
/// `Branch` / `Loop` / `Bind` in a predicate body are out of scope
/// for DB-11 3a.3 (tracked ROADMAP debt) — clone returns `None` and
/// narrowing falls back cleanly.
fn clone_predicate_body(
    dag: &mut Dag,
    source_port: PortId,
    substitute_from: PortId,
    substitute_to: PortId,
    depth: usize,
) -> Option<PortId> {
    const DEPTH_LIMIT: usize = 64;
    if depth >= DEPTH_LIMIT {
        return None;
    }
    if source_port == substitute_from {
        return Some(substitute_to);
    }
    let Some(node_id) = dag.port(source_port).produced_by else {
        // No producer — external port (e.g. another function's
        // param). Reuse directly; no rewrite needed.
        return Some(source_port);
    };
    let cloned = match dag.node(node_id).clone() {
        Behavior::Value(v) => {
            let new_node_id = dag.alloc_node_id();
            let new_output = dag.alloc_port(Some(new_node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: new_node_id,
                data: v.data,
                output: new_output,
                span: v.span,
            }));
            new_output
        }
        Behavior::Transform(t) => {
            let mut new_inputs = Vec::with_capacity(t.inputs.len());
            for input_port in &t.inputs {
                new_inputs.push(clone_predicate_body(
                    dag,
                    *input_port,
                    substitute_from,
                    substitute_to,
                    depth + 1,
                )?);
            }
            let new_node_id = dag.alloc_node_id();
            let new_output = dag.alloc_port(Some(new_node_id));
            dag.push_node(Behavior::Transform(TransformNode {
                id: new_node_id,
                target: t.target,
                inputs: new_inputs,
                output: new_output,
                span: t.span,
            }));
            new_output
        }
        Behavior::Branch(_) | Behavior::Loop(_) | Behavior::Bind(_) => {
            // Out-of-fragment predicate shapes — tracked as DB-11
            // 3a.3 ROADMAP debt (admitted surface > supported
            // fragment). Fail-closed at narrowing time rather than
            // producing a malformed composite.
            return None;
        }
    };
    Some(cloned)
}

/// Return the name of the single scope-bound `Var` referenced in a
/// predicate-shaped cond expression, or `None` if the cond doesn't
/// match the narrowing-eligible shape.
///
/// A cond is narrowing-eligible iff:
/// - it is a two-argument `Operator` or `Call` expression, AND
/// - it references **exactly one** scope-bound name (the candidate
///   refined parameter). Top-level symbols and literals are fine.
///
/// The single-scope-bound-var requirement is load-bearing: a predicate
/// that references two scope-locals (e.g. `x != y`) would need a
/// multi-arity refinement, which DB-11's structural-equality proof
/// theory does not model. Inside a match-arm body where a pattern
/// binding (e.g. `Cons(payload)`) is in scope, predicates like
/// `behavior_id(payload.head) == node_id` have two scope-bound vars
/// (`payload` and `node_id`) and skip narrowing — otherwise the
/// predicate-lowering side effects would attempt to resolve the
/// non-candidate vars in an isolated scope and fail, polluting the
/// main DAG with unresolvable sub-expressions.
fn narrowable_var_name(cond: &SurfaceExpr, scope: &HashMap<String, PortId>) -> Option<String> {
    match cond {
        SurfaceExpr::Operator { args, .. } if args.len() == 2 => {}
        SurfaceExpr::Call { args, .. } if args.len() == 2 => {}
        _ => return None,
    }
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    collect_scope_bound_free_vars(cond, scope, &mut seen);
    if seen.len() == 1 {
        seen.into_iter().next()
    } else {
        None
    }
}

/// Walk a surface expression and collect the names of all free
/// variables that resolve to entries in `scope`. Lambda parameter
/// shadowing and match-arm binding shadowing are respected. Top-level
/// declarations (not in `scope`) are filtered out by construction
/// since `scope` holds only local-variable bindings, not symbols.
fn collect_scope_bound_free_vars(
    expr: &SurfaceExpr,
    scope: &HashMap<String, PortId>,
    out: &mut std::collections::HashSet<String>,
) {
    match expr {
        SurfaceExpr::Var { name, .. } => {
            if scope.contains_key(name) {
                out.insert(name.clone());
            }
        }
        SurfaceExpr::Literal { .. } => {}
        SurfaceExpr::Path { segments, .. } => {
            // The head of a dotted path is a local-variable reference
            // when it resolves to scope (field access). A top-level
            // typed path (e.g. `OrderedRing.add`) wouldn't be in
            // `scope` so it's filtered out.
            if let Some(head) = segments.first() {
                if scope.contains_key(head) {
                    out.insert(head.clone());
                }
            }
        }
        SurfaceExpr::Call { args, .. } | SurfaceExpr::Operator { args, .. } => {
            for arg in args {
                collect_scope_bound_free_vars(arg, scope, out);
            }
        }
        SurfaceExpr::Lambda { params, body, .. } => {
            let mut shadowed = scope.clone();
            for p in params {
                shadowed.remove(p);
            }
            // Use a placeholder port so `contains_key` would return
            // true only for non-shadowed names; we don't use it.
            collect_scope_bound_free_vars(body, &shadowed, out);
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_scope_bound_free_vars(cond, scope, out);
            collect_scope_bound_free_vars(then_branch, scope, out);
            collect_scope_bound_free_vars(else_branch, scope, out);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_scope_bound_free_vars(scrutinee, scope, out);
            for arm in arms {
                // Arm payload bindings shadow the scope within the arm
                // body. Use a synthetic shadowed scope by removing the
                // binding name from a clone. We only remove — not
                // insert a new port — because this routine only cares
                // about scope membership for filtering, not port
                // identity.
                let mut arm_scope = scope.clone();
                if let SurfacePattern::VariantWith { binding, .. } = &arm.pattern {
                    arm_scope.remove(binding);
                }
                collect_scope_bound_free_vars(&arm.body, &arm_scope, out);
            }
        }
        SurfaceExpr::Record { fields, .. } => {
            for field in fields {
                collect_scope_bound_free_vars(&field.value, scope, out);
            }
        }
        SurfaceExpr::List { elements, .. } => {
            for el in elements {
                collect_scope_bound_free_vars(el, scope, out);
            }
        }
    }
}

/// Pass 1: allocate a placeholder Declaration for every named top-level
/// item and record `name → DeclarationId` in the symbol table. Let /
/// Module / Import items produce no declaration.
///
/// Returns `(symbols, is_first)`. `symbols` maps each name to its
/// **first** declaration id (consistent with `Dag::declaration_by_name`'s
/// linear first-match semantics). `is_first[idx]` is false for items
/// whose name already appears in the symbols table at the time they're
/// processed — i.e., duplicates. `lower_into` skips duplicates so the
/// first-of-name's filled connective is not overwritten later.
///
/// Duplicate declarations emit a fail-closed `ResolveError` via a
/// phantom port, so the compile surfaces through
/// `Err(CompileError::Semantic)`. The duplicate's own declaration slot
/// is still allocated (for structural consistency in the declaration
/// table), but it stays as a placeholder and is unreachable by name.
fn collect_symbols(
    dag: &mut Dag,
    items: &[SurfaceItem],
) -> (HashMap<String, DeclarationId>, Vec<bool>) {
    // Seed from already-present declarations with the same preference
    // policy as `Dag::declaration_by_name`: v3 declarations shadow
    // legacy `dsl/` duplicates, otherwise earlier declarations win.
    let mut symbols: HashMap<String, DeclarationId> = HashMap::new();
    for d in dag.declarations() {
        if let Some(name) = &d.name {
            match symbols.get(name).copied() {
                None => {
                    symbols.insert(name.clone(), d.id);
                }
                Some(existing_id) => {
                    let existing = dag.declaration(existing_id);
                    let new_rank = declaration_name_preference_rank(&d.span.file);
                    let existing_rank = declaration_name_preference_rank(&existing.span.file);
                    if new_rank > existing_rank {
                        symbols.insert(name.clone(), d.id);
                    }
                }
            }
        }
    }

    let mut is_first = vec![true; items.len()];
    for (idx, item) in items.iter().enumerate() {
        // Extract the surface-level name and type parameter list.
        // Items that don't allocate declarations:
        // - Let bindings (they produce a BindNode at lower time, not
        //   a Declaration).
        // - Module / Import items (parsed facts preserved but have no
        //   lowered declaration at M1(2.7)).
        // Every other surface form allocates one top-level declaration
        // with the declared name, including `FnExternalBody` and `Data`
        // scaffold forms — their facts flow forward.
        let (name, surface_type_params): (String, &[String]) = match item {
            SurfaceItem::Let { .. } => continue,
            SurfaceItem::Module { .. } => continue,
            SurfaceItem::Import { .. } => continue,
            SurfaceItem::Fn {
                name, type_params, ..
            } => (name.clone(), type_params.as_slice()),
            SurfaceItem::FnExternalBody {
                name, type_params, ..
            } => (name.clone(), type_params.as_slice()),
            SurfaceItem::Data { name, .. } => (name.clone(), &[]),
            SurfaceItem::TypeAtom {
                name, type_params, ..
            }
            | SurfaceItem::TypeRecord {
                name, type_params, ..
            }
            | SurfaceItem::TypeSum {
                name, type_params, ..
            }
            | SurfaceItem::TypeAlias {
                name, type_params, ..
            } => (name.clone(), type_params.as_slice()),
        };
        let span = item_span(item);
        let id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id,
            name: Some(name.clone()),
            connective: placeholder_connective(&name),
            type_params: Vec::new(),
            meta_tag: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            span: span.clone(),
        });

        // Allocate TypeParam declarations for this item's generic
        // parameters and link them via `Declaration.type_params`.
        // Doing this in collect_symbols (pass 1) instead of
        // lower_type_* (pass 2) means that by the time ANY other
        // module's body references `List<Byte>`, List's
        // `type_params` slot is already populated — so
        // `build_template_arguments` can always look up the real
        // template parameter id at construction time. No half-valid
        // state, no post-sweep fixup pass.
        if !surface_type_params.is_empty() {
            let mut param_ids = Vec::with_capacity(surface_type_params.len());
            for param in surface_type_params {
                let param_id = dag.alloc_declaration_id();
                dag.push_declaration(Declaration {
                    id: param_id,
                    name: None,
                    connective: TypeConnective::Atom(AtomPayload::TypeParam(param.clone())),
                    type_params: Vec::new(),
                    meta_tag: None,
                    inhabits: None,

                    value_body: None,
                    refinement: None,
                    span: span.clone(),
                });
                param_ids.push(param_id);
            }
            dag.declaration_mut(id).type_params = param_ids;
        }

        if let Some(&existing_id) = symbols.get(&name) {
            let existing = dag.declaration(existing_id);
            let new_rank = declaration_name_preference_rank(&span.file);
            let existing_rank = declaration_name_preference_rank(&existing.span.file);
            if new_rank > existing_rank {
                symbols.insert(name, id);
            } else {
                is_first[idx] = false;
                let existing_span = existing.span.clone();
                report_declaration_error(
                    dag,
                    Diagnostic::ResolveError {
                        name: format!(
                            "duplicate declaration `{name}` (first declared in `{}` at bytes {}..{})",
                            existing_span.file,
                            existing_span.byte_start,
                            existing_span.byte_end,
                        ),
                        span,
                    },
                );
            }
        } else {
            symbols.insert(name, id);
        }
    }
    (symbols, is_first)
}

fn placeholder_connective(name: &str) -> TypeConnective {
    TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name.to_string()))
}

fn item_span(item: &SurfaceItem) -> SourceSpan {
    match item {
        SurfaceItem::Let { expr, .. } => expr_span(expr).clone(),
        SurfaceItem::Fn { span, .. }
        | SurfaceItem::FnExternalBody { span, .. }
        | SurfaceItem::Data { span, .. }
        | SurfaceItem::Module { span, .. }
        | SurfaceItem::Import { span, .. }
        | SurfaceItem::TypeAtom { span, .. }
        | SurfaceItem::TypeRecord { span, .. }
        | SurfaceItem::TypeSum { span, .. }
        | SurfaceItem::TypeAlias { span, .. } => span.clone(),
    }
}

fn lower_item(
    item: &SurfaceItem,
    dag: &mut Dag,
    scope: ScopeState,
    symbols: &HashMap<String, DeclarationId>,
    mutually_recursive: &HashSet<String>,
) -> ScopeState {
    let mut scope = scope;
    match item {
        SurfaceItem::Let {
            name,
            type_ann,
            expr,
        } => {
            let mut lambda_callable: Option<DeclarationId> = None;
            let value_port = if let SurfaceExpr::Lambda { params, body, span } = expr {
                let Some(ty) = type_ann else {
                    let port = dag.alloc_port(None);
                    dag.mark_unresolved(
                        port,
                        Diagnostic::ResolveError {
                            name: "lambda expressions currently require an expected function type (for example a `let` annotation or a function-typed parameter position)".to_string(),
                            span: span.clone(),
                        },
                    );
                    scope.values.insert(name.clone(), port);
                    return scope;
                };
                let decl_id = type_to_declaration_id(ty, symbols, &HashMap::new(), dag);
                let mut lambda_ctx = LambdaLoweringContext {
                    dag,
                    scope: &scope.values,
                    callable_scope: &scope.callables,
                    symbols,
                };
                match lower_lambda_expr(params, body, span, decl_id, &mut lambda_ctx) {
                    Ok(lambda_decl_id) => {
                        lambda_callable = Some(lambda_decl_id);
                        let port = lambda_ctx.dag.alloc_port(None);
                        match declaration_to_port_shape(decl_id, lambda_ctx.dag, ty.span()) {
                            Ok(expected) => lambda_ctx.dag.set_port_type(port, expected),
                            Err(diag) => lambda_ctx.dag.mark_unresolved(port, diag),
                        }
                        port
                    }
                    Err(diag) => {
                        let port = lambda_ctx.dag.alloc_port(None);
                        lambda_ctx.dag.mark_unresolved(port, diag);
                        port
                    }
                }
            } else {
                let expected_decl = type_ann
                    .as_ref()
                    .map(|ty| type_to_declaration_id(ty, symbols, &HashMap::new(), dag));
                lower_expr(
                    expr,
                    dag,
                    &scope.values,
                    &scope.callables,
                    symbols,
                    expected_decl,
                )
            };
            if let Some(ty) = type_ann {
                // Compute the annotated type's declaration id ONCE.
                // Port TypeShape wraps the same id — no second
                // allocation, no lower↔infer identity split.
                let local: HashMap<String, DeclarationId> = HashMap::new();
                let decl_id = type_to_declaration_id(ty, symbols, &local, dag);
                match declaration_to_port_shape(decl_id, dag, ty.span()) {
                    Ok(declared) => dag.set_port_type(value_port, declared),
                    Err(diag) => dag.mark_unresolved(value_port, diag),
                }
            }
            let bind_id = dag.alloc_node_id();
            let bind_span = match type_ann {
                Some(ty) => ty.span().clone(),
                None => expr_span(expr).clone(),
            };
            dag.push_node(Behavior::Bind(BindNode {
                id: bind_id,
                name: name.clone(),
                value: value_port,
                params: Vec::new(),
                span: bind_span,
            }));
            scope.values.insert(name.clone(), value_port);
            if let Some(lambda_decl_id) = lambda_callable {
                scope.callables.insert(name.clone(), lambda_decl_id);
            }
            scope
        }
        SurfaceItem::Fn {
            name,
            params,
            return_type,
            body,
            span: _,
            ..
        } => {
            scope.values = lower_fn_item_expr_body(
                name,
                params,
                return_type,
                body,
                dag,
                scope.values,
                symbols,
                mutually_recursive,
            );
            scope
        }
        SurfaceItem::FnExternalBody { .. } => {
            // DB-11 (3a.3): signature was fully built by
            // `seed_function_signatures_phase` with
            // `ArrowBody::Unparsed(body_span)`. Nothing remains to do
            // at body-lowering time — the declaration is already in
            // its final form.
            scope
        }
        SurfaceItem::Data {
            name,
            ty,
            body,
            body_span,
            span: _,
        } => {
            lower_data_item(name, ty, body.as_ref(), body_span, dag, symbols);
            scope
        }
        SurfaceItem::Module { .. } | SurfaceItem::Import { .. } => {
            // No-op at M1(2.7). The parsed facts are preserved in the
            // SurfaceItem for M2 module scoping to consume; lowering
            // doesn't currently need them because name resolution
            // spans the flat declaration table.
            scope
        }
        SurfaceItem::TypeAtom { name, .. } => {
            let decl_id = symbols[name];
            // Empty product — equivalent to a unit type.
            dag.declaration_mut(decl_id).connective = TypeConnective::Conj {
                children: Vec::new(),
            };
            scope
        }
        SurfaceItem::TypeRecord {
            name,
            type_params,
            fields,
            ..
        } => {
            lower_type_record(dag, symbols, name, type_params, fields);
            scope
        }
        SurfaceItem::TypeSum {
            name,
            type_params,
            variants,
            ..
        } => {
            lower_type_sum(dag, symbols, name, type_params, variants);
            scope
        }
        SurfaceItem::TypeAlias {
            name,
            type_params,
            target,
            ..
        } => {
            lower_type_alias(dag, symbols, name, type_params, target);
            scope
        }
    }
}

/// Read a parent declaration's pre-populated `type_params` slot and
/// build a `name → DeclarationId` local scope map for field-type and
/// variant-payload lookups. The actual TypeParam declarations were
/// allocated up front in `collect_symbols` so every declaration
/// reaches `lower_type_*` with its `type_params` slot already filled.
/// This is the shape that lets `build_template_arguments` look up real
/// parameter ids at construction time — no half-valid state, no
/// post-sweep fixup pass.
fn local_scope_from_parent(dag: &Dag, parent_id: DeclarationId) -> HashMap<String, DeclarationId> {
    let parent = dag.declaration(parent_id);
    let mut scope: HashMap<String, DeclarationId> = parent
        .type_params
        .iter()
        .filter_map(|pid| {
            if let TypeConnective::Atom(AtomPayload::TypeParam(name)) =
                &dag.declaration(*pid).connective
            {
                Some((name.clone(), *pid))
            } else {
                None
            }
        })
        .collect();
    // Recursive declarations must be able to refer to themselves
    // while their connective is still being lowered. Without this,
    // `tail: List<T>` inside `type List<T> = ...` allocates an
    // UnresolvedIdentifier stub that later resolves back to the
    // declaration and trips the "resolves to itself" fail-closed
    // guard intended for bad aliases, not real recursive sums.
    if let Some(name) = &parent.name {
        scope.insert(name.clone(), parent_id);
    }
    scope
}

fn lower_type_record(
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
    name: &str,
    _type_params: &[String],
    fields: &[SurfaceField],
) {
    let decl_id = symbols[name];
    let local = local_scope_from_parent(dag, decl_id);
    let mut children: Vec<Field> = Vec::with_capacity(fields.len());
    for field in fields {
        let ty_id = type_to_declaration_id(&field.ty, symbols, &local, dag);
        children.push(Field {
            label: field.name.clone(),
            ty: ty_id,
        });
    }
    dag.declaration_mut(decl_id).connective = TypeConnective::Conj { children };
}

fn lower_type_sum(
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
    name: &str,
    _type_params: &[String],
    variants: &[SurfaceVariant],
) {
    let decl_id = symbols[name];
    let local = local_scope_from_parent(dag, decl_id);
    let mut variant_fields: Vec<Field> = Vec::with_capacity(variants.len());
    for variant in variants {
        // Build payload children FIRST, then allocate the variant declaration.
        // Allocating the variant id before its payload children would wedge
        // the dense-sequential invariant on `Dag.declarations` because the
        // child declarations push into slots between the variant's reserved
        // id and its eventual push.
        let connective = match &variant.payload {
            // Unit variants surface as `Positional(vec![])` — the
            // empty-positional case handles them with zero Field
            // children, which is structurally indistinguishable
            // from a Unit variant and avoids a dedicated enum arm.
            VariantPayload::Positional(payload_types) => {
                let children: Vec<Field> = payload_types
                    .iter()
                    .enumerate()
                    .map(|(idx, ty)| Field {
                        label: format!("_{idx}"),
                        ty: type_to_declaration_id(ty, symbols, &local, dag),
                    })
                    .collect();
                TypeConnective::Conj { children }
            }
            VariantPayload::Record(fields) => {
                let children: Vec<Field> = fields
                    .iter()
                    .map(|f| Field {
                        label: f.name.clone(),
                        ty: type_to_declaration_id(&f.ty, symbols, &local, dag),
                    })
                    .collect();
                TypeConnective::Conj { children }
            }
        };
        // Sum variant declarations are anonymous. The variant name
        // lives structurally in the parent `Disj.variants` Field label,
        // not in the child declaration's name slot. Keeps variant names
        // (`True`, `False`, `Less`, `Equal`, `Greater`) out of
        // `Dag::declaration_by_name`'s flat scan.
        let variant_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: variant_id,
            name: None,
            connective,
            type_params: Vec::new(),
            meta_tag: None,
            inhabits: None,

            value_body: None,
            refinement: None,
            span: variant.span.clone(),
        });
        variant_fields.push(Field {
            label: variant.name.clone(),
            ty: variant_id,
        });
    }
    dag.declaration_mut(decl_id).connective = TypeConnective::Disj {
        variants: variant_fields,
    };
}

fn lower_type_alias(
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
    name: &str,
    _type_params: &[String],
    target: &SurfaceType,
) {
    let decl_id = symbols[name];
    let local = local_scope_from_parent(dag, decl_id);
    let connective = type_to_connective(target, symbols, &local, dag);
    dag.declaration_mut(decl_id).connective = connective;
}

/// Lower a `SurfaceType` to a fresh DeclarationId. Used for field types,
/// Arrow parameters, and template arguments. Allocates anonymous (unnamed)
/// declarations for composite shapes; looks up named references against
/// `local` (type params in scope) first, then `symbols` (top-level names).
/// Unknown names get an Identifier Atom stub with `resolved: None` — pass
/// 2 and inference are responsible for filling it in or reporting.
fn type_to_declaration_id(
    ty: &SurfaceType,
    symbols: &HashMap<String, DeclarationId>,
    local: &HashMap<String, DeclarationId>,
    dag: &mut Dag,
) -> DeclarationId {
    match ty {
        SurfaceType::Named { name, .. } => {
            if let Some(id) = local.get(name) {
                return *id;
            }
            if let Some(id) = symbols.get(name) {
                return *id;
            }
            alloc_identifier_stub(dag, name, ty.span())
        }
        SurfaceType::Parameterized { name, args, span } => {
            let template_id = local
                .get(name)
                .or_else(|| symbols.get(name))
                .copied()
                .unwrap_or_else(|| alloc_identifier_stub(dag, name, span));
            let arguments =
                build_template_arguments(dag, symbols, local, template_id, name, args, span);
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Instantiation {
                    template: template_id,
                    arguments,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,

                value_body: None,
                refinement: None,
                span: span.clone(),
            });
            id
        }
        SurfaceType::Optional { inner, span } => {
            let element = type_to_declaration_id(inner, symbols, local, dag);
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Cardinality {
                    element,
                    bound: CardinalityBound::AtMostOne,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,

                value_body: None,
                refinement: None,
                span: span.clone(),
            });
            id
        }
        SurfaceType::Arrow {
            inputs,
            output,
            span,
        } => {
            let input_ids: Vec<DeclarationId> = inputs
                .iter()
                .map(|i| type_to_declaration_id(i, symbols, local, dag))
                .collect();
            let output_id = type_to_declaration_id(output, symbols, local, dag);
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Arrow {
                    inputs: input_ids,
                    output: output_id,
                    body: ArrowBody::Pending,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,

                value_body: None,
                refinement: None,
                span: span.clone(),
            });
            id
        }
    }
}

/// Lower a `SurfaceType` directly to a connective (not a new declaration).
/// Used for `TypeAlias` targets where we want the alias declaration itself
/// to carry the aliased shape, not a one-level-indirect wrapper.
fn type_to_connective(
    ty: &SurfaceType,
    symbols: &HashMap<String, DeclarationId>,
    local: &HashMap<String, DeclarationId>,
    dag: &mut Dag,
) -> TypeConnective {
    match ty {
        SurfaceType::Named { name, .. } => {
            let template = local
                .get(name)
                .or_else(|| symbols.get(name))
                .copied()
                .unwrap_or_else(|| alloc_identifier_stub(dag, name, ty.span()));
            TypeConnective::Instantiation {
                template,
                arguments: Vec::new(),
            }
        }
        SurfaceType::Parameterized { name, args, span } => {
            let template = local
                .get(name)
                .or_else(|| symbols.get(name))
                .copied()
                .unwrap_or_else(|| alloc_identifier_stub(dag, name, span));
            let arguments =
                build_template_arguments(dag, symbols, local, template, name, args, span);
            TypeConnective::Instantiation {
                template,
                arguments,
            }
        }
        SurfaceType::Optional { inner, .. } => TypeConnective::Cardinality {
            element: type_to_declaration_id(inner, symbols, local, dag),
            bound: CardinalityBound::AtMostOne,
        },
        SurfaceType::Arrow { inputs, output, .. } => TypeConnective::Arrow {
            inputs: inputs
                .iter()
                .map(|i| type_to_declaration_id(i, symbols, local, dag))
                .collect(),
            output: type_to_declaration_id(output, symbols, local, dag),
            body: ArrowBody::Pending,
        },
    }
}

/// Build the `TemplateArgument` list for an `Instantiation`.
///
/// Two-phase bootstrap means every real declaration's `type_params`
/// slot is populated by the time lowering runs, so real templates
/// always produce well-formed `TemplateArgument`s whose `parameter`
/// edge points at a genuine TypeParam atom — honoring the field's
/// contract in `dag::TemplateArgument`.
///
/// **Stub templates** — anonymous `UnresolvedIdentifier` stubs
/// created when a name can't be resolved at lower time — produce an
/// **empty argument list**. The stub itself already has a diagnostic
/// waiting for it (bootstrap mode tolerates bootstrap-range dangling
/// refs like `Tuple` in `algebra.dag`; user-code mode fails closed
/// via the strict sweep), so there is no additional consumer that
/// needs a per-argument `TemplateArgument` when the template is
/// unresolved. The old "self-reference tolerated scaffold" path is
/// deleted per QW4 — `TemplateArgument.parameter` is no longer
/// representable as a non-TypeParam reference at construction
/// time.
fn build_template_arguments(
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
    local: &HashMap<String, DeclarationId>,
    template: DeclarationId,
    template_name: &str,
    args: &[SurfaceType],
    span: &SourceSpan,
) -> Vec<TemplateArgument> {
    let template_decl = dag.declaration(template);
    let template_is_stub = template_decl.name.is_none()
        && matches!(
            &template_decl.connective,
            TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        );
    if template_is_stub {
        // Consume the argument surface types for their side effects
        // (nested stub allocation, diagnostic attachment), but do not
        // construct any TemplateArgument values — the template's
        // pending error is the authoritative failure for this
        // instantiation, and a TemplateArgument whose parameter
        // wasn't a TypeParam would violate the field contract.
        for arg in args {
            let _ = type_to_declaration_id(arg, symbols, local, dag);
        }
        return Vec::new();
    }
    let template_param_count = dag.declaration(template).type_params.len();
    if template_param_count != args.len() {
        report_declaration_error(
            dag,
            Diagnostic::ArityMismatch {
                function: format!("type `{template_name}`"),
                expected: template_param_count,
                actual: args.len(),
                span: span.clone(),
            },
        );
        // Arity mismatch is an authoritative failure. Consume the
        // argument surface types for their side effects and return
        // nothing — building partial TemplateArguments would require
        // inventing parameter references that don't exist, which
        // would violate the field contract.
        for arg in args {
            let _ = type_to_declaration_id(arg, symbols, local, dag);
        }
        return Vec::new();
    }
    args.iter()
        .enumerate()
        .map(|(idx, arg)| {
            let value = type_to_declaration_id(arg, symbols, local, dag);
            let parameter = template_param_id(dag, template, idx).expect(
                "template_param_count equality was checked immediately above — \
                 param lookup at idx < count must succeed",
            );
            TemplateArgument { parameter, value }
        })
        .collect()
}

/// Allocate a stub Identifier-atom declaration. Callers reach this path when
/// name resolution fails against the local type-parameter scope and the
/// current top-level symbol table snapshot. The stub deliberately carries
/// `resolved: None` and stays in the declaration graph so that a later
/// `resolve_pending_identifiers` sweep can either fill it in (forward
/// references across bootstrap fixtures) or emit a fail-closed diagnostic.
fn alloc_identifier_stub(dag: &mut Dag, name: &str, span: &SourceSpan) -> DeclarationId {
    let id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id,
        name: None,
        connective: TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name.to_string())),
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,

        value_body: None,
        refinement: None,
        span: span.clone(),
    });
    id
}

/// Emit a declaration-level diagnostic via `Dag::attach_diagnostic`.
/// Thin wrapper kept for naming symmetry with the per-item fail-closed
/// call sites in lowering; bootstrap and tests use
/// `Dag::attach_diagnostic` directly.
fn report_declaration_error(dag: &mut Dag, diag: Diagnostic) {
    dag.attach_diagnostic(diag);
}

/// Final-pass resolution over anonymous Identifier-atom declarations in
/// **bootstrap** mode. Resolves every stub whose name is in the
/// declaration table; **tolerates** stubs whose name is dangling.
/// Tolerance is bootstrap-specific: the canonical `dsl/std/*.dag`
/// files carry forward references to types that live in std/ modules
/// outside the M1(2.6) load set (e.g., `Tuple` referenced from
/// `algebra.dag` with no defining `.dag` file among the seven loaded),
/// and those are not bootstrap errors. User-facing code uses the
/// strict variant below.
pub(crate) fn resolve_pending_identifiers(dag: &mut Dag) {
    run_identifier_sweep(dag, /*strict_from=*/ usize::MAX);
}

/// Final-pass resolution in **strict** mode: every Identifier stub at
/// a declaration id `>= strict_from` that cannot resolve emits a
/// fail-closed `ResolveError`. Used by `lower` for user modules, where
/// `strict_from` is the declaration count before user lowering began.
/// Stubs in the bootstrap range (id < strict_from) are still resolved
/// opportunistically but tolerated.
pub(crate) fn resolve_pending_identifiers_strict(dag: &mut Dag, strict_from: usize) {
    run_identifier_sweep(dag, strict_from);
}

/// User-mode scaffold rejection. `FnExternalBody` lowers to
/// `ArrowBody::Unparsed(SourceSpan)`; `Data` lowers to a declaration
/// with `value_body = Some(ValueBody::Unparsed(SourceSpan))`. Both
/// are load-bearing scaffolds for std/bootstrap files whose bodies
/// the M1(2.8) parser cannot yet lower. For ordinary user code they
/// are a fail-closed concern: an opaque body that the compiler has
/// not validated shipping through `compile_to_dag` violates the
/// fail-closed static-grounding invariant.
///
/// This sweep walks declarations at id `>= strict_from` (i.e. the
/// user-lowered range) and attaches a fail-closed diagnostic for
/// each `Unparsed` scaffold found. Bootstrap-range declarations
/// (id < strict_from) are tolerated — those scaffolds exist by
/// design until the parser grows to cover the remaining std/
/// grammar (match / record literals / lambdas / pipes / data body
/// literals — see `DOWNSTREAM_REQUIREMENTS.md` class-5 gap list).
///
/// Diagnostics are attached via `report_declaration_error`
/// (`Dag::attach_diagnostic`), which routes through the phantom-port
/// channel so they surface via `compile_to_dag`'s standard
/// `CompileError::Semantic` path.
fn reject_user_unparsed_scaffolds(dag: &mut Dag, strict_from: usize) {
    // Snapshot the offending (id, span) pairs first so we can emit
    // diagnostics without holding a long immutable borrow of the
    // declaration slice.
    let mut unparsed_fn_scaffolds: Vec<(DeclarationId, String, SourceSpan)> = Vec::new();
    let mut unparsed_data_scaffolds: Vec<(DeclarationId, String, SourceSpan)> = Vec::new();
    for decl in dag.declarations() {
        if (decl.id.raw() as usize) < strict_from {
            continue;
        }
        let name = decl
            .name
            .clone()
            .unwrap_or_else(|| format!("declaration#{}", decl.id.raw()));
        if let TypeConnective::Arrow {
            body: ArrowBody::Unparsed(span),
            ..
        } = &decl.connective
        {
            unparsed_fn_scaffolds.push((decl.id, name.clone(), span.clone()));
        }
        if let Some(crate::dag::ValueBody::Unparsed(span)) = &decl.value_body {
            unparsed_data_scaffolds.push((decl.id, name, span.clone()));
        }
    }
    for (_, name, span) in unparsed_fn_scaffolds {
        report_declaration_error(
            dag,
            Diagnostic::ResolveError {
                name: format!(
                    "function `{name}` has an opaque block body — M1(2.8) user code cannot yet use match / record literals / lambdas inside block-bodied fn definitions (see DOWNSTREAM_REQUIREMENTS.md class-5 gaps)"
                ),
                span,
            },
        );
    }
    for (_, name, span) in unparsed_data_scaffolds {
        report_declaration_error(
            dag,
            Diagnostic::ResolveError {
                name: format!(
                    "data `{name}` has an opaque body — M1(2.8) user code cannot yet use record / list / map literals inside data bodies (see DOWNSTREAM_REQUIREMENTS.md class-5 gap #3)"
                ),
                span,
            },
        );
    }
}

fn run_identifier_sweep(dag: &mut Dag, strict_from: usize) {
    let snapshot: Vec<(DeclarationId, String, SourceSpan)> = dag
        .declarations()
        .iter()
        .filter_map(|d| match &d.connective {
            TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) => {
                Some((d.id, name.clone(), d.span.clone()))
            }
            _ => None,
        })
        .collect();

    for (decl_id, name, span) in snapshot {
        // Operator identifiers (`+`, `-`, ...) no longer flow through
        // this sweep — they are committed to
        // `TransformTarget::Operator(OperatorKind)` at parse time and
        // never allocate a stub declaration. Any `UnresolvedIdentifier`
        // atom that reaches this point is a real name-reference stub
        // waiting for lower-time or sweep-time resolution.
        if let Some(target) = dag.declaration_by_name(&name).map(|d| d.id) {
            if target == decl_id {
                report_declaration_error(
                    dag,
                    Diagnostic::ResolveError {
                        name: format!("type `{name}` resolves to itself"),
                        span,
                    },
                );
                continue;
            }
            // Structural phase transition: rewrite the connective from
            // `UnresolvedIdentifier(name)` to `ResolvedIdentifier(target)`.
            // The phase is now visible in the variant, not hidden in
            // an Option field.
            dag.declaration_mut(decl_id).connective =
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(target));
        } else if (decl_id.raw() as usize) >= strict_from {
            // Strict mode for user-lowering stubs: any
            // `UnresolvedIdentifier` that survives the sweep is a
            // fail-closed ResolveError. After this loop every
            // user-range Identifier atom is either a
            // ResolvedIdentifier or has attached a diagnostic.
            report_declaration_error(
                dag,
                Diagnostic::ResolveError {
                    name: format!("unresolved type identifier `{name}`"),
                    span,
                },
            );
        }
        // else: bootstrap-range dangling ref, tolerated. The canonical
        // std/ files may forward-reference types that live in std/
        // modules outside the M1(2.6) load set; those are not errors.
    }
}

/// Return the `idx`-th type parameter declaration of a template. Reads the
/// canonical `Declaration.type_params` slot rather than filtering
/// `Conj.children` — type params are first-class on `Declaration` per
/// M1(2.5)'s refactor. Returns None when the template has fewer params than
/// `idx` requires; callers must fail-closed (not substitute a fallback).
fn template_param_id(dag: &Dag, template: DeclarationId, idx: usize) -> Option<DeclarationId> {
    dag.declaration(template).type_params.get(idx).copied()
}

// DB-11 (3a.3) single-construction-authority cleanup:
// `lower_fn_item_unparsed` used to run a full parameter-decl walk
// and an Arrow overwrite here — duplicating the work that
// `seed_function_signatures_phase` already did for every
// `SurfaceItem::FnExternalBody`. When a parameter carried a `where`
// refinement, the duplication pushed a second predicate Bind and a
// second refined Declaration into the DAG and then overwrote the
// Arrow's `inputs` to point at the duplicates, leaving the seeded
// predicate Bind and refined Declaration orphaned. Seeding already
// produces the final Arrow shape (`ArrowBody::Unparsed(body_span)`),
// so the function was redundant and is now deleted; the caller in
// the `SurfaceItem::FnExternalBody` arm has been simplified to a
// no-op since the signature flows directly from seeding.

/// Lower a `SurfaceItem::Data` — a typed constant whose body is not
/// yet parseable as a `SurfaceExpr`. The declaration carries two
/// facts:
///
/// - `connective` = the resolved type annotation (via
///   `type_to_connective`). The declaration's type is accessible
///   through the same path as type aliases.
/// - `value_body = Some(ValueBody::Unparsed(body_span))`. This
///   makes the declaration structurally distinguishable from a
///   type alias (which has `value_body = None`) so consumers can
///   tell "data value with this type" from "type alias of this
///   type" by reading the substrate directly.
///
/// **QW2 + structural scaffold honesty** (M1(2.7) review round 9):
/// before this fix, lowering dropped the body entirely and set
/// the connective to the type. The declaration was structurally
/// identical to a type alias — the substrate admitted a state
/// where "data vs type" had no distinction. Now the fact survives
/// as an Unparsed scaffold with an explicit dissolution trigger
/// (M2+ record/map/list literal parser extension), and the
/// declaration carries both the declared type and the body span
/// for future parser passes to consume.
fn lower_data_item(
    name: &str,
    ty: &SurfaceType,
    body: Option<&SurfaceExpr>,
    body_span: &SourceSpan,
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
) {
    let decl_id = symbols[name];
    let local: HashMap<String, DeclarationId> = HashMap::new();
    // Compute the declaration id of the type annotation (e.g.
    // Realization's DeclarationId for `data rust_int: Realization`)
    // — the walk starts here.
    let ty_decl_id = type_to_declaration_id(ty, symbols, &local, dag);
    let connective = type_to_connective(ty, symbols, &local, dag);
    // meta_tag edge from the data item to its type annotation. This
    // is what makes a data declaration structurally distinct from a
    // type alias (R9/R10 + PR-B): the type is readable from the
    // same field shape as Realization instances in rust.dag, and
    // downstream consumers (emit_rust) can filter by meta_tag to
    // find all declarations of a given meta-type.
    dag.declaration_mut(decl_id).meta_tag = Some(ty_decl_id);
    dag.declaration_mut(decl_id).connective = connective;
    // Attempt structural inhabitance checking if the body parsed
    // as a record literal. Falls back to Unparsed on any failure
    // (walk doesn't terminate at a Conj, missing field, extra
    // field, value isn't a literal, type mismatch). Fail-closed
    // paths attach a diagnostic via `report_declaration_error` so
    // user-facing code sees the error.
    // DB-10 (3a.2): `data x: T = v` bodies lower by shape:
    //   - Record literal → ValueBody::Structural (existing path).
    //   - Scalar literal  → ValueBody::Scalar (new path, DB-10).
    //   - Anything else   → ValueBody::Unparsed fallback.
    let value_body = match body {
        Some(SurfaceExpr::Record { fields, .. }) => {
            lower_record_to_structural(name, fields, ty_decl_id, body_span, symbols, dag)
        }
        Some(lit_expr @ SurfaceExpr::Literal { .. }) => {
            lower_scalar_literal_for_type(lit_expr, ty_decl_id, dag)
                .map(crate::dag::ValueBody::Scalar)
                .or_else(|| {
                    report_declaration_error(
                        dag,
                        Diagnostic::ResolveError {
                            name: format!(
                                "data `{name}`'s scalar body does not match declared type",
                            ),
                            span: body_span.clone(),
                        },
                    );
                    None
                })
        }
        _ => None,
    };
    let final_body =
        value_body.unwrap_or_else(|| crate::dag::ValueBody::Unparsed(body_span.clone()));
    dag.declaration_mut(decl_id).value_body = Some(final_body);
}

/// Walk a declaration through `Instantiation` / `ResolvedIdentifier`
/// edges until it reaches a declaration whose connective is
/// `Conj`. Returns the `DeclarationId` of that Conj declaration — or
/// `None` if no Conj is reachable within the walk depth limit, or
/// the chain terminates at some other connective.
///
/// Used by `lower_data_item` for data body inhabitance checking.
/// Mirrors the `walk_to_disj_decl` helper in `infer.rs`: the two
/// walks look for complementary terminal shapes (Conj for data
/// item inhabitance vs Disj for Branch scrutinees), but follow the
/// same alias/instantiation chains otherwise.
fn walk_to_conj_decl(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Conj { .. } => return Some(current),
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

fn walk_to_disj_decl(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
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

#[derive(Clone, Default)]
struct LowerSubstStack {
    frames: Vec<Vec<TemplateArgument>>,
}

impl LowerSubstStack {
    fn push(&mut self, frame: Vec<TemplateArgument>) {
        self.frames.push(frame);
    }

    fn lookup(&self, parameter: DeclarationId) -> Option<DeclarationId> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.iter().find(|arg| arg.parameter == parameter))
            .map(|arg| arg.value)
    }
}

fn find_equivalent_decl_instantiation_lower(
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

fn resolve_decl_with_subst_lower(
    dag: &Dag,
    current: DeclarationId,
    subst: &LowerSubstStack,
    depth: usize,
) -> Option<DeclarationId> {
    if depth >= 32 {
        return None;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => subst
            .lookup(current)
            .and_then(|bound| resolve_decl_with_subst_lower(dag, bound, subst, depth + 1))
            .or(Some(current)),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            resolve_decl_with_subst_lower(dag, *next, subst, depth + 1)
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
                        value: resolve_decl_with_subst_lower(dag, arg.value, subst, depth + 1)?,
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
            find_equivalent_decl_instantiation_lower(dag, *template, &specialized_arguments)
                .or(Some(current))
        }
        TypeConnective::Cardinality { .. }
        | TypeConnective::Arrow { .. }
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::Literal(_)) => Some(current),
    }
}

fn walk_to_conj_decl_with_subst_lower(
    dag: &Dag,
    start: DeclarationId,
    subst: &mut LowerSubstStack,
) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                subst.push(arguments.clone());
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => current = subst.lookup(current)?,
            _ => return None,
        }
    }
    None
}

fn walk_to_type_shape_lower(
    dag: &Dag,
    current: DeclarationId,
    subst: &LowerSubstStack,
    depth: usize,
) -> Option<TypeShape> {
    if depth >= 32 {
        return None;
    }
    let decl = dag.declaration(current);
    if decl.name.is_some() {
        return Some(TypeShape::new(current));
    }
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
            if let Some(bound) = subst.lookup(current) {
                walk_to_type_shape_lower(dag, bound, subst, depth + 1)
            } else {
                Some(TypeShape::new(current))
            }
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            walk_to_type_shape_lower(dag, *next, subst, depth + 1)
        }
        TypeConnective::Instantiation { .. } => {
            resolve_decl_with_subst_lower(dag, current, subst, depth + 1)
                .map(TypeShape::new)
                .or_else(|| Some(TypeShape::new(current)))
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Arrow { .. }
        | TypeConnective::Cardinality { .. } => None,
    }
}

fn resolve_static_field_project(
    dag: &Dag,
    input_port: PortId,
    field_label: &str,
) -> Option<(DeclarationId, TypeShape)> {
    let input_ty = match dag.port(input_port).state() {
        crate::dag::PortState::Resolved(ty) => *ty,
        crate::dag::PortState::Uninferred | crate::dag::PortState::Unresolved => return None,
    };
    let mut subst = LowerSubstStack::default();
    let conj_id = walk_to_conj_decl_with_subst_lower(dag, input_ty.declaration, &mut subst)?;
    let children = match &dag.declaration(conj_id).connective {
        TypeConnective::Conj { children } => children,
        _ => return None,
    };
    let field_decl = children
        .iter()
        .find(|field| field.label == field_label)
        .map(|field| field.ty)?;
    let output_ty = walk_to_type_shape_lower(dag, field_decl, &subst, 0)?;
    Some((field_decl, output_ty))
}

/// Attempt to lower a parsed record literal body into a
/// `ValueBody::Structural { fields }` by matching each record field
/// against the declared type's Conj children. Returns `None` (and
/// attaches one or more diagnostics) if any check fails; the caller
/// falls back to `ValueBody::Unparsed`.
///
/// Validation rules:
/// 1. The type annotation's declaration must walk to a `Conj`
///    declaration via `walk_to_conj_decl`. Failure → diagnostic
///    "data item's type has no record shape" → return None.
/// 2. The record literal's field labels must exactly match the
///    Conj's children — no extras, no missing.
/// 3. Each field's value must be a `SurfaceExpr::Literal`
///    (scalar literal). Nested records, variable references, and
///    computed expressions are class-5 gap #3 follow-ups.
/// 4. Each literal's type must match the corresponding Conj
///    field's declared type (validated via the primitive cache).
fn lower_record_to_structural(
    data_name: &str,
    record_fields: &[crate::parse::SurfaceRecordField],
    ty_decl_id: DeclarationId,
    body_span: &SourceSpan,
    symbols: &HashMap<String, DeclarationId>,
    dag: &mut Dag,
) -> Option<crate::dag::ValueBody> {
    let Some(conj_id) = walk_to_conj_decl(dag, ty_decl_id) else {
        report_declaration_error(
            dag,
            Diagnostic::ResolveError {
                name: format!(
                    "data `{data_name}`'s type annotation does not resolve to a record type (Conj); cannot apply inhabitance checking to the body"
                ),
                span: body_span.clone(),
            },
        );
        return None;
    };
    // Snapshot the Conj's children to avoid borrowing conflicts
    // while we walk the record literal.
    let type_fields: Vec<(String, DeclarationId)> = match &dag.declaration(conj_id).connective {
        TypeConnective::Conj { children } => {
            children.iter().map(|f| (f.label.clone(), f.ty)).collect()
        }
        _ => unreachable!("walk_to_conj_decl returned a non-Conj declaration"),
    };
    // Check no extra fields in the data body.
    for record_field in record_fields {
        if !type_fields
            .iter()
            .any(|(label, _)| *label == record_field.name)
        {
            report_declaration_error(
                dag,
                Diagnostic::ResolveError {
                    name: format!(
                        "data `{data_name}` has field `{}` but the type has no such field",
                        record_field.name
                    ),
                    span: record_field.span.clone(),
                },
            );
            return None;
        }
    }
    // Check no missing fields.
    for (type_label, _) in &type_fields {
        if !record_fields.iter().any(|f| f.name == *type_label) {
            report_declaration_error(
                dag,
                Diagnostic::ResolveError {
                    name: format!("data `{data_name}` is missing required field `{type_label}`"),
                    span: body_span.clone(),
                },
            );
            return None;
        }
    }
    // Determine which realization category (if any) this data
    // item belongs to. Used below to narrow the acceptable shape
    // of `target` / `op` field values per category. The category
    // is identified by name lookup (Phase 2 doesn't have the
    // realization meta cache populated yet — that runs at
    // bootstrap end). Same lookup as for primitives above.
    let category = realization_category_for_meta(dag, ty_decl_id);

    let mut structural_fields: Vec<(String, crate::dag::FieldValue)> =
        Vec::with_capacity(type_fields.len());
    for (type_label, type_field_id) in &type_fields {
        let record_field = record_fields
            .iter()
            .find(|f| f.name == *type_label)
            .expect("checked above");
        let field_value = lower_structural_field_value(
            data_name,
            type_label,
            &record_field.value,
            *type_field_id,
            symbols,
            dag,
            category,
            &record_field.span,
        )?;
        structural_fields.push((type_label.clone(), field_value));
    }
    Some(crate::dag::ValueBody::Structural {
        fields: structural_fields,
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_structural_field_value(
    data_name: &str,
    field_label: &str,
    expr: &SurfaceExpr,
    expected_type: DeclarationId,
    symbols: &HashMap<String, DeclarationId>,
    dag: &mut Dag,
    category: Option<RealizationCategoryTag>,
    span: &SourceSpan,
) -> Option<crate::dag::FieldValue> {
    if let Some(marker_id) = dag.declaration_by_name("DeclarationRef").map(|d| d.id) {
        if walks_to(dag, expected_type, marker_id) {
            let decl_id = resolve_field_value_as_declaration_ref(expr, symbols, dag)?;
            if let Some(cat) = category {
                if let Err(reason) =
                    validate_realization_field_target(dag, cat, field_label, decl_id)
                {
                    report_declaration_error(
                        dag,
                        Diagnostic::ResolveError {
                            name: format!(
                                "data `{data_name}` field `{field_label}` does not satisfy the {cat:?} realization constraint: {reason}"
                            ),
                            span: span.clone(),
                        },
                    );
                    return None;
                }
            }
            return Some(crate::dag::FieldValue::Reference(decl_id));
        }
    }

    if let Some(literal_bits) = lower_scalar_literal_for_type(expr, expected_type, dag) {
        return Some(crate::dag::FieldValue::Literal(literal_bits));
    }

    if let Some(element_type) = list_element_type(dag, expected_type) {
        let SurfaceExpr::List { elements, .. } = expr else {
            report_declaration_error(
                dag,
                Diagnostic::ResolveError {
                    name: format!(
                        "data `{data_name}` field `{field_label}` must be a list literal matching the declared List<_> type"
                    ),
                    span: span.clone(),
                },
            );
            return None;
        };
        let mut lowered = Vec::with_capacity(elements.len());
        for element in elements {
            lowered.push(lower_structural_field_value(
                data_name,
                field_label,
                element,
                element_type,
                symbols,
                dag,
                None,
                expr_span(element),
            )?);
        }
        return Some(crate::dag::FieldValue::List(lowered));
    }

    if let Some(conj_id) = walk_to_conj_decl(dag, expected_type) {
        let SurfaceExpr::Record { fields, .. } = expr else {
            report_declaration_error(
                dag,
                Diagnostic::ResolveError {
                    name: format!(
                        "data `{data_name}` field `{field_label}` must be a record literal matching the declared record type"
                    ),
                    span: span.clone(),
                },
            );
            return None;
        };
        let expected_fields: Vec<(String, DeclarationId)> =
            match &dag.declaration(conj_id).connective {
                TypeConnective::Conj { children } => children
                    .iter()
                    .map(|child| (child.label.clone(), child.ty))
                    .collect(),
                _ => unreachable!("walk_to_conj_decl returned non-Conj"),
            };
        for field in fields {
            if !expected_fields
                .iter()
                .any(|(label, _)| *label == field.name)
            {
                report_declaration_error(
                    dag,
                    Diagnostic::ResolveError {
                        name: format!(
                            "data `{data_name}` field `{field_label}` has nested field `{}` but the declared type has no such field",
                            field.name
                        ),
                        span: field.span.clone(),
                    },
                );
                return None;
            }
        }
        for (label, _) in &expected_fields {
            if !fields.iter().any(|field| field.name == *label) {
                report_declaration_error(
                    dag,
                    Diagnostic::ResolveError {
                        name: format!(
                            "data `{data_name}` field `{field_label}` is missing nested field `{label}`"
                        ),
                        span: span.clone(),
                    },
                );
                return None;
            }
        }
        let mut lowered = Vec::with_capacity(expected_fields.len());
        for (label, ty) in expected_fields {
            let nested = fields
                .iter()
                .find(|field| field.name == label)
                .expect("checked above");
            lowered.push((
                label.clone(),
                lower_structural_field_value(
                    data_name,
                    &label,
                    &nested.value,
                    ty,
                    symbols,
                    dag,
                    None,
                    &nested.span,
                )?,
            ));
        }
        return Some(crate::dag::FieldValue::Record(lowered));
    }

    if let Some(disj_id) = walk_to_disj_decl(dag, expected_type) {
        let (variant_name, args, variant_span) = match expr {
            SurfaceExpr::Call { target, args, span } => (target.as_str(), args.as_slice(), span),
            SurfaceExpr::Var { name, span } => (name.as_str(), &[][..], span),
            _ => {
                report_declaration_error(
                    dag,
                    Diagnostic::ResolveError {
                        name: format!(
                            "data `{data_name}` field `{field_label}` must be a constructor call matching the declared sum type"
                        ),
                        span: span.clone(),
                    },
                );
                return None;
            }
        };
        let variants: Vec<(String, DeclarationId)> = match &dag.declaration(disj_id).connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .map(|field| (field.label.clone(), field.ty))
                .collect(),
            _ => unreachable!("walk_to_disj_decl returned non-Disj"),
        };
        let Some((_, variant_decl_id)) = variants.iter().find(|(label, _)| label == variant_name)
        else {
            report_declaration_error(
                dag,
                Diagnostic::ResolveError {
                    name: format!(
                        "data `{data_name}` field `{field_label}` uses constructor `{variant_name}` which is not a variant of the declared sum type"
                    ),
                    span: variant_span.clone(),
                },
            );
            return None;
        };
        let payload_fields: Vec<DeclarationId> = match &dag.declaration(*variant_decl_id).connective
        {
            TypeConnective::Conj { children } => children.iter().map(|child| child.ty).collect(),
            other => {
                report_declaration_error(
                    dag,
                    Diagnostic::ResolveError {
                        name: format!(
                            "data `{data_name}` field `{field_label}` constructor `{variant_name}` does not lower to a record payload shape: got {other:?}"
                        ),
                        span: variant_span.clone(),
                    },
                );
                return None;
            }
        };
        if payload_fields.len() != args.len() {
            report_declaration_error(
                dag,
                Diagnostic::ArityMismatch {
                    function: variant_name.to_string(),
                    expected: payload_fields.len(),
                    actual: args.len(),
                    span: variant_span.clone(),
                },
            );
            return None;
        }
        let mut payload = Vec::with_capacity(args.len());
        for (arg, payload_field_ty) in args.iter().zip(payload_fields.iter()) {
            payload.push(lower_structural_field_value(
                data_name,
                field_label,
                arg,
                *payload_field_ty,
                symbols,
                dag,
                None,
                expr_span(arg),
            )?);
        }
        return Some(crate::dag::FieldValue::Variant {
            constructor: *variant_decl_id,
            payload,
        });
    }

    report_declaration_error(
        dag,
        Diagnostic::ResolveError {
            name: format!(
                "data `{data_name}` field `{field_label}` does not match the declared structural type"
            ),
            span: span.clone(),
        },
    );
    None
}

fn lower_scalar_literal_for_type(
    expr: &SurfaceExpr,
    expected_type: DeclarationId,
    dag: &Dag,
) -> Option<LiteralBits> {
    let SurfaceExpr::Literal { value, .. } = expr else {
        return None;
    };
    let literal_bits = match value {
        SurfaceLiteral::Int(v) => LiteralBits::Int(*v),
        SurfaceLiteral::Bool(v) => LiteralBits::Bool(*v),
        SurfaceLiteral::String(v) => LiteralBits::String(v.clone()),
    };
    let int_decl_id = dag.declaration_by_name("Int").map(|d| d.id);
    let bool_decl_id = dag.declaration_by_name("Bool").map(|d| d.id);
    let string_decl_id = dag.declaration_by_name("String").map(|d| d.id);
    let type_ok = match &literal_bits {
        LiteralBits::Int(_) => int_decl_id
            .map(|id| walks_to(dag, expected_type, id))
            .unwrap_or(false),
        LiteralBits::Bool(_) => bool_decl_id
            .map(|id| walks_to(dag, expected_type, id))
            .unwrap_or(false),
        LiteralBits::String(_) => string_decl_id
            .map(|id| walks_to(dag, expected_type, id))
            .unwrap_or(false),
    };
    type_ok.then_some(literal_bits)
}

fn list_element_type(dag: &Dag, expected_type: DeclarationId) -> Option<DeclarationId> {
    let list_id = dag.declaration_by_name("List")?.id;
    match &dag.declaration(expected_type).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if *template == list_id && arguments.len() == 1 => Some(arguments[0].value),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            list_element_type(dag, *next)
        }
        _ => None,
    }
}

/// Resolve a record-literal field value as a typed declaration
/// reference. Accepts:
///
///   - `SurfaceExpr::Var { name }` — top-level identifier looked up
///     in the symbols map.
///   - `SurfaceExpr::Path { segments }` — dotted-path. The first
///     segment is a top-level identifier; subsequent segments walk
///     the parent's `Conj` children by label, returning the child
///     declaration's `ty` (the field's type declaration).
///
/// Returns `None` if the expression is not a recognized reference
/// shape or any segment fails to resolve.
fn resolve_field_value_as_declaration_ref(
    expr: &SurfaceExpr,
    symbols: &HashMap<String, DeclarationId>,
    dag: &Dag,
) -> Option<DeclarationId> {
    let segments: Vec<&str> = match expr {
        SurfaceExpr::Var { name, .. } => vec![name.as_str()],
        SurfaceExpr::Path { segments, .. } => segments.iter().map(|s| s.as_str()).collect(),
        _ => return None,
    };
    let (first, rest) = segments.split_first()?;
    let mut current = *symbols.get(*first)?;
    for segment in rest {
        // Walk `current` through aliases / instantiation to a Conj,
        // then find the child field whose label matches `segment`.
        // The child's `ty` becomes the new `current` for the next
        // segment (or the final answer if this was the last one).
        let conj_id = walk_to_conj_decl(dag, current)?;
        let children = match &dag.declaration(conj_id).connective {
            TypeConnective::Conj { children } => children,
            _ => return None,
        };
        let next = children.iter().find(|f| f.label == *segment)?;
        current = next.ty;
    }
    Some(current)
}

/// Realization category tag for the lower-time narrowing check.
/// Mirrors `emit_rust::RealizationCategory` but lives in lower's
/// own namespace because the lowerer can't depend on emit_rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RealizationCategoryTag {
    Type,
    Operator,
    Behavior,
    Callable,
    TypeInstantiation,
    Pattern,
}

/// Identify which realization category (if any) a data item's
/// meta-type belongs to. Returns `None` for non-realization data
/// items (which skip the narrowing check entirely).
///
/// Looks up the meta-type by name. The realization meta cache on
/// `Dag` isn't populated until `populate_primitive_cache` runs at
/// bootstrap end, but `collect_symbols_phase` has registered every
/// top-level declaration by the time `lower_record_to_structural`
/// runs in Phase 2, so `declaration_by_name` works here. Same
/// pattern as the primitive lookups above.
fn realization_category_for_meta(
    dag: &Dag,
    meta_decl_id: DeclarationId,
) -> Option<RealizationCategoryTag> {
    let type_meta = dag.declaration_by_name("TypeRealization").map(|d| d.id);
    let op_meta = dag.declaration_by_name("OperatorRealization").map(|d| d.id);
    let behavior_meta = dag.declaration_by_name("BehaviorRealization").map(|d| d.id);
    let callable_meta = dag.declaration_by_name("CallableRealization").map(|d| d.id);
    let type_instantiation_meta = dag
        .declaration_by_name("TypeInstantiationRealization")
        .map(|d| d.id);
    let pattern_meta = dag.declaration_by_name("PatternRealization").map(|d| d.id);
    if Some(meta_decl_id) == type_meta {
        Some(RealizationCategoryTag::Type)
    } else if Some(meta_decl_id) == op_meta {
        Some(RealizationCategoryTag::Operator)
    } else if Some(meta_decl_id) == behavior_meta {
        Some(RealizationCategoryTag::Behavior)
    } else if Some(meta_decl_id) == callable_meta {
        Some(RealizationCategoryTag::Callable)
    } else if Some(meta_decl_id) == type_instantiation_meta {
        Some(RealizationCategoryTag::TypeInstantiation)
    } else if Some(meta_decl_id) == pattern_meta {
        Some(RealizationCategoryTag::Pattern)
    } else {
        None
    }
}

/// Validate that the resolved target of a realization-category
/// field satisfies the per-(category, field_label) structural
/// constraint. Returns `Ok(())` on success or `Err(reason)` with
/// a human-readable explanation of the violation.
///
/// **The constraints encode the "DeclarationRef sentinel is too
/// permissive" narrowing.** PR #445's R1 review on the unwind
/// flagged that `BehaviorRealization { target: Int }` would
/// type-check at the substrate level because the field type is
/// `DeclarationRef` (the universal sentinel). The fully structural
/// fix (typed marker hierarchy via `inhabits` syntax or `where`
/// clauses) requires parser/lower extensions that are out of
/// scope for the PR-B-unwind round; this function is the
/// fail-closed lower-time alternative. Bad wirings now surface
/// as fail-closed diagnostics at lower time, not as silent skips
/// or downstream "missing realization" errors at emit time.
///
/// Constraints, by category × field label:
///
///   - `TypeRealization.target` — must be a primitive type
///     declaration (Int / Bool / String). Anything else is a
///     spec error.
///   - `OperatorRealization.target` — must be either a primitive
///     type or an atomic identity handle (the operand type for a
///     realized operator, e.g. Int for `1 + 2` or PortId for
///     reflected handle equality).
///   - `OperatorRealization.op` — must walk to an Arrow
///     declaration that is a child of an algebra Conj (e.g.
///     OrderedRing.add). The constraint is "the resolved
///     declaration's connective is Arrow," which is the structural
///     shape every algebra field has.
///   - `BehaviorRealization.target` — must be one of the v3_l1
///     substrate behavior markers (Bind / Branch / Loop /
///     Transform / Value / Main).
///   - `CallableRealization.target` — must walk to an Arrow
///     declaration (a callable target).
///   - `TypeInstantiationRealization.target` — must be a named
///     generic declaration with at least one type parameter.
///   - `PatternRealization.target` — must walk to a Disj
///     declaration (a matchable sum carrier).
fn validate_realization_field_target(
    dag: &Dag,
    category: RealizationCategoryTag,
    field_label: &str,
    target: DeclarationId,
) -> Result<(), String> {
    let int_id = dag.declaration_by_name("Int").map(|d| d.id);
    let bool_id = dag.declaration_by_name("Bool").map(|d| d.id);
    let string_id = dag.declaration_by_name("String").map(|d| d.id);
    let node_id = dag.declaration_by_name("NodeId").map(|d| d.id);
    let port_id = dag.declaration_by_name("PortId").map(|d| d.id);
    let declaration_id = dag.declaration_by_name("DeclarationId").map(|d| d.id);

    let is_primitive =
        |id: DeclarationId| Some(id) == int_id || Some(id) == bool_id || Some(id) == string_id;
    let is_atomic_handle = |id: DeclarationId| {
        Some(id) == node_id || Some(id) == port_id || Some(id) == declaration_id
    };
    let is_behavior_marker = |id: DeclarationId| {
        let markers = [
            dag.declaration_by_name("Value"),
            dag.declaration_by_name("Transform"),
            dag.declaration_by_name("Branch"),
            dag.declaration_by_name("Loop"),
            dag.declaration_by_name("Bind"),
            dag.declaration_by_name("Main"),
        ];
        markers.iter().any(|m| m.map(|d| d.id) == Some(id))
    };
    let is_algebra_field =
        |id: DeclarationId| matches!(dag.declaration(id).connective, TypeConnective::Arrow { .. });
    let walks_to_arrow = |start: DeclarationId| {
        let mut current = start;
        for _ in 0..32 {
            match &dag.declaration(current).connective {
                TypeConnective::Arrow { .. } => return true,
                TypeConnective::Instantiation { template, .. } => current = *template,
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
                _ => return false,
            }
        }
        false
    };
    let walks_to_disj = |start: DeclarationId| {
        let mut current = start;
        for _ in 0..32 {
            match &dag.declaration(current).connective {
                TypeConnective::Disj { .. } => return true,
                TypeConnective::Instantiation { template, .. } => current = *template,
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
                _ => return false,
            }
        }
        false
    };

    match (category, field_label) {
        (RealizationCategoryTag::Type, "target") => {
            if is_behavior_marker(target) {
                Err(format!(
                    "TypeRealization.target must reference a realizable structural type, not a behavior marker; got declaration {target:?}"
                ))
            } else {
                Ok(())
            }
        }
        (RealizationCategoryTag::Operator, "target") => {
            if is_primitive(target) || is_atomic_handle(target) {
                Ok(())
            } else {
                Err(format!(
                    "OperatorRealization.target must reference a primitive operand type or atomic handle (Int/Bool/String/NodeId/PortId/DeclarationId); got declaration {target:?}"
                ))
            }
        }
        (RealizationCategoryTag::Operator, "op") => {
            if is_algebra_field(target) {
                Ok(())
            } else {
                Err(format!(
                    "OperatorRealization.op must reference an algebra field declaration whose connective is Arrow (e.g. OrderedRing.add); got declaration {target:?}"
                ))
            }
        }
        (RealizationCategoryTag::Behavior, "target") => {
            if is_behavior_marker(target) {
                Ok(())
            } else {
                Err(format!(
                    "BehaviorRealization.target must reference one of the v3_l1 behavior markers (Bind/Branch/Loop/Transform/Value/Main); got declaration {target:?}"
                ))
            }
        }
        (RealizationCategoryTag::Callable, "target") => {
            if walks_to_arrow(target) {
                Ok(())
            } else {
                Err(format!(
                    "CallableRealization.target must reference a callable declaration whose connective walks to Arrow; got declaration {target:?}"
                ))
            }
        }
        (RealizationCategoryTag::TypeInstantiation, "target") => {
            if !dag.declaration(target).type_params.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "TypeInstantiationRealization.target must reference a generic declaration with at least one type parameter; got declaration {target:?}"
                ))
            }
        }
        (RealizationCategoryTag::Pattern, "target") => {
            if walks_to_disj(target) {
                Ok(())
            } else {
                Err(format!(
                    "PatternRealization.target must reference a matchable sum declaration whose connective walks to Disj; got declaration {target:?}"
                ))
            }
        }
        // Fields without a category-specific narrowing constraint
        // (e.g. carrier and cost are scalar literals, not
        // Declaration references). The category check is a
        // partial constraint applied only to the fields where it
        // makes sense; other fields skip it.
        _ => Ok(()),
    }
}

/// Walk a declaration chain looking for a specific target
/// declaration id. Returns true if `current` eventually resolves
/// to `target` via Instantiation/ResolvedIdentifier edges or if
/// `current == target` directly. Used by
/// `lower_record_to_structural` to check "is this field's type
/// annotation ultimately Int/Bool/String?"
fn walks_to(dag: &Dag, start: DeclarationId, target: DeclarationId) -> bool {
    let mut current = start;
    for _ in 0..32 {
        if current == target {
            return true;
        }
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => {
                current = *template;
            }
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                current = *next;
            }
            _ => return false,
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn lower_fn_item_expr_body(
    name: &str,
    params: &[SurfaceParam],
    return_type: &SurfaceType,
    body: &SurfaceExpr,
    dag: &mut Dag,
    outer_scope: HashMap<String, PortId>,
    symbols: &HashMap<String, DeclarationId>,
    mutually_recursive: &HashSet<String>,
) -> HashMap<String, PortId> {
    let fn_decl_id = symbols[name];

    // 1. Read the seeded Arrow — single-construction-authority for
    //    parameter declarations (DB-11 3a.3). `seed_function_signatures_phase`
    //    already ran `type_to_declaration_id` for each parameter and,
    //    when a `where` refinement is present, built the predicate
    //    Declaration + refined Declaration via `lower_parameter_refinement`.
    //    Re-lowering them here would push a parallel Bind + refined
    //    Declaration into the DAG for every `where` clause and then
    //    overwrite the fn's Arrow inputs to point at the duplicates
    //    — the seeded ones would stay in the DAG, orphaned. Read
    //    the seeded inputs directly instead.
    let (param_decl_inputs, return_decl_id) = match &dag.declaration(fn_decl_id).connective {
        TypeConnective::Arrow { inputs, output, .. } => (inputs.clone(), *output),
        _ => {
            // Defensive fallback — should not occur because
            // `seed_function_signatures_phase` runs unconditionally
            // on every `Fn` item before bodies are lowered. If the
            // connective is not an Arrow here, the seed phase was
            // skipped or the declaration was clobbered; re-derive
            // to keep the body-lowering pass going and surface the
            // real issue upstream.
            let local_fallback = local_scope_from_parent(dag, fn_decl_id);
            let inputs: Vec<DeclarationId> = params
                .iter()
                .map(|p| {
                    let base = type_to_declaration_id(&p.ty, symbols, &local_fallback, dag);
                    match &p.refinement {
                        Some(predicate) => lower_parameter_refinement(
                            base,
                            predicate,
                            &p.name,
                            symbols,
                            dag,
                            p.ty.span().clone(),
                        ),
                        None => base,
                    }
                })
                .collect();
            let output = type_to_declaration_id(return_type, symbols, &local_fallback, dag);
            (inputs, output)
        }
    };
    let mut param_ports: Vec<PortId> = Vec::with_capacity(params.len());
    let mut param_types: Vec<TypeShape> = Vec::with_capacity(params.len());
    let mut body_scope: HashMap<String, PortId> = outer_scope.clone();
    let mut callable_scope: CallableScope = CallableScope::new();
    for (param, &input_decl) in params.iter().zip(param_decl_inputs.iter()) {
        let port = dag.alloc_port(None);
        let ty = match declaration_to_port_shape(input_decl, dag, param.ty.span()) {
            Ok(ty) => {
                dag.set_port_type(port, ty);
                ty
            }
            Err(diag) => {
                dag.mark_unresolved(port, diag);
                sentinel_type_shape(dag)
            }
        };
        body_scope.insert(param.name.clone(), port);
        if declaration_is_callable(dag, input_decl, 0) {
            callable_scope.insert(param.name.clone(), input_decl);
        }
        param_ports.push(port);
        param_types.push(ty);
    }

    // 2. Compute return `TypeShape` from the seeded Arrow's output
    //    declaration (read above). Same single-construction-authority
    //    invariant as the parameter loop.
    let return_ty = match declaration_to_port_shape(return_decl_id, dag, return_type.span()) {
        Ok(ty) => ty,
        Err(diag) => {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(err_port, diag);
            sentinel_type_shape(dag)
        }
    };

    // 3. Mutual recursion check — same as M0. Reject with an
    //    Unresolved Bind value port AND set the Arrow body to
    //    `UserDefined(bind_id)` pointing at that Bind. `decide_transform`'s
    //    `UserDefined` arm reads the Bind's value port state: if it's
    //    Unresolved, every caller fails with "function has an invalid
    //    body," cascading the rejection through the normal
    //    upstream-failure mechanism.
    //
    //    **R13 fix.** Earlier revisions set `body = ArrowBody::Pending`
    //    here, which `decide_transform` accepts as a realization-lag
    //    scaffold (signature type-checks, body walking skipped). That
    //    path doesn't read the Bind, so callers got Resolved types
    //    from an invalid fn — a FAIL-CLOSED violation. Using
    //    `UserDefined(bind_id)` routes callers through the
    //    Unresolved-body guard that poisons them correctly. The
    //    non-mutually-recursive rejection paths below (zero-param
    //    recursion, non-descent-provable recursion) already use this
    //    shape via the common bottom-of-function code.
    if mutually_recursive.contains(name) {
        let err_port = dag.alloc_port(None);
        let body_span = expr_span(body).clone();
        dag.mark_unresolved(
            err_port,
            Diagnostic::ResolveError {
                name: format!(
                    "function `{name}` is part of a mutual recursion cycle; mutual recursion is not yet supported in v3"
                ),
                span: body_span.clone(),
            },
        );
        let bind_id = dag.alloc_node_id();
        dag.push_node(Behavior::Bind(BindNode {
            id: bind_id,
            name: name.to_string(),
            value: err_port,
            params: param_ports,
            span: body_span,
        }));
        dag.declaration_mut(fn_decl_id).connective = TypeConnective::Arrow {
            inputs: param_decl_inputs,
            output: return_decl_id,
            body: ArrowBody::UserDefined(bind_id),
        };
        let mut outer_scope = outer_scope;
        outer_scope.insert(name.to_string(), err_port);
        return outer_scope;
    }

    // 4. Lower the body.
    let body_return_port = lower_expr(
        body,
        dag,
        &body_scope,
        &callable_scope,
        symbols,
        Some(return_decl_id),
    );
    let body_root = dag.port(body_return_port).produced_by;
    let body_span = expr_span(body).clone();

    // 5. Handle recursion: bounded Loop wrapping (descent-provable) or
    //    fail-closed rejection (unprovable or zero-arg). Same M0 logic; the
    //    descent check operates on the unresolved operator-identifier shape
    //    (SurfaceExpr::Call with target "-"), per §8.9 Option A.
    let value_port = if is_recursive(body, name) {
        if param_ports.is_empty() {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(
                err_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "function `{name}` is recursive but has no parameters; cannot terminate"
                    ),
                    span: body_span.clone(),
                },
            );
            err_port
        } else if !descent_provable(
            body,
            dag,
            param_decl_inputs[0],
            name,
            &params[0].name,
            &HashMap::new(),
        ) {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(
                err_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "cannot prove recursion in `{name}` terminates; expected each recursive call's first argument to be `{param} - <positive int>` or a structurally smaller recursive child",
                        param = &params[0].name,
                    ),
                    span: body_span.clone(),
                },
            );
            err_port
        } else {
            let loop_id = dag.alloc_node_id();
            let loop_output = dag.alloc_port(Some(loop_id));
            dag.set_port_type(loop_output, return_ty);
            let loop_body_node = body_root.unwrap_or(loop_id);
            dag.push_node(Behavior::Loop(LoopNode {
                id: loop_id,
                source: param_ports[0],
                init: param_ports[0],
                body: loop_body_node,
                bound: Bound {
                    count: param_ports[0],
                },
                output: loop_output,
                span: body_span.clone(),
            }));
            loop_output
        }
    } else {
        body_return_port
    };

    dag.set_port_type(value_port, return_ty);
    let bind_id = dag.alloc_node_id();
    dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: name.to_string(),
        value: value_port,
        params: param_ports,
        span: body_span,
    }));

    // 6. Fill in the function's Declaration with the Arrow connective.
    dag.declaration_mut(fn_decl_id).connective = TypeConnective::Arrow {
        inputs: param_decl_inputs,
        output: return_decl_id,
        body: ArrowBody::UserDefined(bind_id),
    };

    let mut outer_scope = outer_scope;
    outer_scope.insert(name.to_string(), value_port);
    outer_scope
}

/// Convert a surface type annotation into a port-level `TypeShape`.
/// User code at M1(2.6) only accepts primitive type annotations
/// Wrap a pre-computed `DeclarationId` as a port `TypeShape`, with a
/// fail-closed check for fresh unresolved stubs.
///
/// **Dissolution receipt — QW5 SINGLE AUTHORITY + R9 double-lower fix.**
/// Before M1(2.7) port typing hardcoded `"Int"|"Bool"|"String"` as a
/// whitelist parallel to `type_to_declaration_id`'s structural walk.
/// The whitelist dissolved into `type_to_declaration_id` (Class 1
/// resolution). Before R9, the port path *re-called*
/// `type_to_declaration_id` in a separate helper, so compound types
/// like `List<Int>` allocated two anonymous declarations — one for
/// the Arrow input, one for the port TypeShape — with different
/// `DeclarationId`s. R9 collapses that: callers run
/// `type_to_declaration_id` **once** and pass the resulting id to
/// this helper, so the Arrow's declaration slot and the port's
/// `TypeShape` share identity.
///
/// The fail-closed guard detects a fresh anonymous
/// `UnresolvedIdentifier` stub (name wasn't in the symbol table)
/// and returns `Err` so the caller marks the port `Unresolved`. The
/// resolve sweep ALSO fires against the stub at end of lowering,
/// but this path anchors the failure to the annotation span.
fn declaration_to_port_shape(
    decl_id: DeclarationId,
    dag: &Dag,
    annotation_span: &SourceSpan,
) -> Result<TypeShape, Diagnostic> {
    let decl = dag.declaration(decl_id);
    if decl.name.is_none() {
        if let TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) = &decl.connective {
            return Err(Diagnostic::ResolveError {
                name: format!("unknown type `{name}`"),
                span: annotation_span.clone(),
            });
        }
    }
    Ok(TypeShape::new(decl_id))
}

/// Sentinel TypeShape returned when a type annotation failed to
/// resolve. The port it's assigned to has already been `mark_unresolved`ed
/// with the underlying diagnostic, so the sentinel value itself is
/// never observed by inference — it exists only to satisfy Rust's
/// "must initialize" requirement. Uses the cached `Int` primitive
/// shape, falling back to `DeclarationId(0)` if even the cache is
/// empty (unreachable post-bootstrap).
fn sentinel_type_shape(dag: &Dag) -> TypeShape {
    dag.int_shape()
        .unwrap_or_else(|| TypeShape::new(dag.declarations()[0].id))
}

fn declaration_callable_inputs(
    dag: &Dag,
    current: DeclarationId,
    depth: usize,
) -> Option<Vec<DeclarationId>> {
    if depth >= 32 {
        return None;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Arrow { inputs, .. } => Some(inputs.clone()),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            declaration_callable_inputs(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation { template, .. } => {
            declaration_callable_inputs(dag, *template, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::TypeParam(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Cardinality { .. } => None,
    }
}

fn declaration_is_callable(dag: &Dag, current: DeclarationId, depth: usize) -> bool {
    declaration_callable_inputs(dag, current, depth).is_some()
}

fn declaration_callable_output(
    dag: &Dag,
    current: DeclarationId,
    depth: usize,
) -> Option<DeclarationId> {
    if depth >= 32 {
        return None;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Arrow { output, .. } => Some(*output),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            declaration_callable_output(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation { template, .. } => {
            declaration_callable_output(dag, *template, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::TypeParam(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Conj { .. }
        | TypeConnective::Disj { .. }
        | TypeConnective::Cardinality { .. } => None,
    }
}

fn declaration_callable_signature(
    dag: &Dag,
    current: DeclarationId,
    depth: usize,
) -> Option<(Vec<DeclarationId>, DeclarationId)> {
    Some((
        declaration_callable_inputs(dag, current, depth)?,
        declaration_callable_output(dag, current, depth)?,
    ))
}

fn lambda_synthetic_name(span: &SourceSpan) -> String {
    format!("__anon_lambda_{}_{}", span.byte_start, span.byte_end)
}

fn collect_lambda_free_names(
    expr: &SurfaceExpr,
    bound: &HashSet<String>,
    free: &mut HashSet<String>,
) {
    match expr {
        SurfaceExpr::Literal { .. } => {}
        SurfaceExpr::Var { name, .. } => {
            if !bound.contains(name) {
                free.insert(name.clone());
            }
        }
        SurfaceExpr::Path { segments, .. } => {
            if let Some(head) = segments.first() {
                if !bound.contains(head) {
                    free.insert(head.clone());
                }
            }
        }
        SurfaceExpr::Call { target, args, .. } => {
            if !bound.contains(target) {
                free.insert(target.clone());
            }
            for arg in args {
                collect_lambda_free_names(arg, bound, free);
            }
        }
        SurfaceExpr::Operator { args, .. } => {
            for arg in args {
                collect_lambda_free_names(arg, bound, free);
            }
        }
        SurfaceExpr::Lambda { params, body, .. } => {
            let mut inner_bound = bound.clone();
            inner_bound.extend(params.iter().cloned());
            collect_lambda_free_names(body, &inner_bound, free);
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_lambda_free_names(cond, bound, free);
            collect_lambda_free_names(then_branch, bound, free);
            collect_lambda_free_names(else_branch, bound, free);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_lambda_free_names(scrutinee, bound, free);
            for arm in arms {
                let mut arm_bound = bound.clone();
                if let SurfacePattern::VariantWith { binding, .. } = &arm.pattern {
                    arm_bound.insert(binding.clone());
                }
                collect_lambda_free_names(&arm.body, &arm_bound, free);
            }
        }
        SurfaceExpr::Record { fields, .. } => {
            for field in fields {
                collect_lambda_free_names(&field.value, bound, free);
            }
        }
        SurfaceExpr::List { elements, .. } => {
            for element in elements {
                collect_lambda_free_names(element, bound, free);
            }
        }
    }
}

fn lower_lambda_expr(
    params: &[String],
    body: &SurfaceExpr,
    span: &SourceSpan,
    expected_decl: DeclarationId,
    ctx: &mut LambdaLoweringContext<'_>,
) -> Result<DeclarationId, Diagnostic> {
    let Some((expected_inputs, expected_output)) =
        declaration_callable_signature(ctx.dag, expected_decl, 0)
    else {
        return Err(Diagnostic::ResolveError {
            name: "lambda expression requires an expected function type".to_string(),
            span: span.clone(),
        });
    };
    if expected_inputs.len() != params.len() {
        return Err(Diagnostic::ResolveError {
            name: format!(
                "lambda parameter count mismatch: expected {} parameter(s) from the surrounding function type, found {}",
                expected_inputs.len(),
                params.len(),
            ),
            span: span.clone(),
        });
    }

    let mut free = HashSet::new();
    let bound: HashSet<String> = params.iter().cloned().collect();
    collect_lambda_free_names(body, &bound, &mut free);
    let mut free_names: Vec<String> = free.into_iter().collect();
    free_names.sort();

    let mut capture_ports: Vec<PortId> = Vec::new();
    let mut body_scope: HashMap<String, PortId> = HashMap::new();
    let mut body_callable_scope: CallableScope = CallableScope::new();
    for name in free_names {
        let mut found_binding = false;
        if let Some(&port) = ctx.scope.get(&name) {
            capture_ports.push(port);
            body_scope.insert(name.clone(), port);
            found_binding = true;
        }
        if let Some(&decl_id) = ctx.callable_scope.get(&name) {
            body_callable_scope.insert(name.clone(), decl_id);
            found_binding = true;
        }
        if !found_binding && ctx.symbols.contains_key(&name) {
            found_binding = true;
        }
        if !found_binding {
            return Err(Diagnostic::ResolveError {
                name,
                span: span.clone(),
            });
        }
    }

    let mut param_ports: Vec<PortId> = Vec::with_capacity(params.len());
    for (name, input_decl) in params.iter().zip(expected_inputs.iter()) {
        let port = ctx.dag.alloc_port(None);
        let ty = declaration_to_port_shape(*input_decl, ctx.dag, span)?;
        ctx.dag.set_port_type(port, ty);
        body_scope.insert(name.clone(), port);
        if declaration_is_callable(ctx.dag, *input_decl, 0) {
            body_callable_scope.insert(name.clone(), *input_decl);
        }
        param_ports.push(port);
    }

    let body_return_port = lower_expr(
        body,
        ctx.dag,
        &body_scope,
        &body_callable_scope,
        ctx.symbols,
        Some(expected_output),
    );
    let bind_id = ctx.dag.alloc_node_id();
    let mut bind_params = capture_ports;
    bind_params.extend(param_ports);
    ctx.dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: lambda_synthetic_name(span),
        value: body_return_port,
        params: bind_params,
        span: span.clone(),
    }));

    let lambda_decl_id = ctx.dag.alloc_declaration_id();
    ctx.dag.push_declaration(Declaration {
        id: lambda_decl_id,
        name: None,
        connective: TypeConnective::Arrow {
            inputs: expected_inputs,
            output: expected_output,
            body: ArrowBody::UserDefined(bind_id),
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: span.clone(),
    });
    Ok(lambda_decl_id)
}

fn resolve_callable_reference(
    expr: &SurfaceExpr,
    dag: &mut Dag,
    callable_scope: &CallableScope,
    symbols: &HashMap<String, DeclarationId>,
) -> DeclarationId {
    match expr {
        SurfaceExpr::Var { name, span } => callable_scope
            .get(name)
            .copied()
            .or_else(|| symbols.get(name).copied())
            .unwrap_or_else(|| alloc_identifier_stub(dag, name, span)),
        SurfaceExpr::Path { segments, span } => {
            alloc_identifier_stub(dag, &segments.join("."), span)
        }
        other => alloc_identifier_stub(dag, "__callable_argument__", expr_span(other)),
    }
}

fn push_template_argument_binding(
    arguments: &mut Vec<TemplateArgument>,
    parameter: DeclarationId,
    value: DeclarationId,
) -> bool {
    for existing in arguments.iter() {
        if existing.parameter == parameter {
            return existing.value == value;
        }
    }
    arguments.push(TemplateArgument { parameter, value });
    true
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
    if depth >= 32 {
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
    base_target_decl: DeclarationId,
    arguments: &[TemplateArgument],
) -> Vec<TemplateArgument> {
    let template = match &dag.declaration(base_target_decl).connective {
        TypeConnective::Instantiation { template, .. } => *template,
        _ => base_target_decl,
    };
    let mut allowed: HashSet<DeclarationId> = dag
        .declaration(template)
        .type_params
        .iter()
        .copied()
        .collect();
    if let Some(inputs) = direct_invocation_input_decls(dag, template, 0) {
        for input in inputs {
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

fn callable_binding_conflict_diagnostic(
    target: &str,
    arg_index: usize,
    span: &SourceSpan,
) -> Diagnostic {
    Diagnostic::ResolveError {
        name: format!(
            "callable argument {} to `{target}` does not match the expected function type or conflicts with earlier template bindings",
            arg_index + 1
        ),
        span: span.clone(),
    }
}

fn bind_expected_type_to_actual(
    dag: &Dag,
    expected_id: DeclarationId,
    actual_id: DeclarationId,
    arguments: &mut Vec<TemplateArgument>,
    depth: usize,
) -> bool {
    if depth >= 32 {
        return false;
    }
    let expected_decl = dag.declaration(expected_id);
    match &expected_decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
            if let Some(bound) = template_argument_value(arguments, expected_id) {
                if bound != expected_id {
                    return bind_expected_type_to_actual(
                        dag,
                        bound,
                        actual_id,
                        arguments,
                        depth + 1,
                    );
                }
            }
            push_template_argument_binding(arguments, expected_id, actual_id)
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            bind_expected_type_to_actual(dag, *next, actual_id, arguments, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments: expected_arguments,
        } => {
            let actual_decl = dag.declaration(actual_id);
            match &actual_decl.connective {
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                    bind_expected_type_to_actual(dag, expected_id, *next, arguments, depth + 1)
                }
                TypeConnective::Instantiation {
                    template: actual_template,
                    arguments: actual_arguments,
                } => {
                    if *actual_template == expected_id
                        && expected_arguments.len() == actual_arguments.len()
                    {
                        return expected_arguments.iter().zip(actual_arguments.iter()).all(
                            |(expected, actual)| {
                                bind_expected_type_to_actual(
                                    dag,
                                    expected.value,
                                    actual.value,
                                    arguments,
                                    depth + 1,
                                )
                            },
                        );
                    }
                    if template != actual_template
                        || expected_arguments.len() != actual_arguments.len()
                    {
                        return false;
                    }
                    expected_arguments.iter().zip(actual_arguments.iter()).all(
                        |(expected, actual)| {
                            bind_expected_type_to_actual(
                                dag,
                                expected.value,
                                actual.value,
                                arguments,
                                depth + 1,
                            )
                        },
                    )
                }
                _ => false,
            }
        }
        TypeConnective::Arrow { inputs, output, .. } => {
            let actual_decl = dag.declaration(actual_id);
            match &actual_decl.connective {
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                    bind_expected_type_to_actual(dag, expected_id, *next, arguments, depth + 1)
                }
                TypeConnective::Arrow {
                    inputs: actual_inputs,
                    output: actual_output,
                    ..
                } => {
                    if inputs.len() != actual_inputs.len() {
                        return false;
                    }
                    inputs
                        .iter()
                        .zip(actual_inputs.iter())
                        .all(|(expected, actual)| {
                            bind_expected_type_to_actual(
                                dag,
                                *expected,
                                *actual,
                                arguments,
                                depth + 1,
                            )
                        })
                        && bind_expected_type_to_actual(
                            dag,
                            *output,
                            *actual_output,
                            arguments,
                            depth + 1,
                        )
                }
                _ => false,
            }
        }
        TypeConnective::Cardinality { element, bound } => {
            let actual_decl = dag.declaration(actual_id);
            match &actual_decl.connective {
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                    bind_expected_type_to_actual(dag, expected_id, *next, arguments, depth + 1)
                }
                TypeConnective::Cardinality {
                    element: actual_element,
                    bound: actual_bound,
                } if bound == actual_bound => bind_expected_type_to_actual(
                    dag,
                    *element,
                    *actual_element,
                    arguments,
                    depth + 1,
                ),
                _ => false,
            }
        }
        _ => {
            let actual_decl = dag.declaration(actual_id);
            match &actual_decl.connective {
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                    bind_expected_type_to_actual(dag, expected_id, *next, arguments, depth + 1)
                }
                _ => expected_id == actual_id,
            }
        }
    }
}

fn specialize_decl_for_lowering(
    dag: &mut Dag,
    current: DeclarationId,
    arguments: &[TemplateArgument],
    depth: usize,
) -> DeclarationId {
    if depth >= 32 {
        return current;
    }
    let decl = dag.declaration(current).clone();
    match decl.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => arguments
            .iter()
            .find(|arg| arg.parameter == current)
            .map(|arg| arg.value)
            .unwrap_or(current),
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            specialize_decl_for_lowering(dag, next, arguments, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments: inner_arguments,
        } => {
            let specialized_arguments: Vec<TemplateArgument> = inner_arguments
                .iter()
                .map(|arg| TemplateArgument {
                    parameter: arg.parameter,
                    value: specialize_decl_for_lowering(dag, arg.value, arguments, depth + 1),
                })
                .collect();
            if specialized_arguments.len() == inner_arguments.len()
                && specialized_arguments
                    .iter()
                    .zip(inner_arguments.iter())
                    .all(|(lhs, rhs)| lhs.parameter == rhs.parameter && lhs.value == rhs.value)
            {
                return current;
            }
            if let Some(existing) =
                find_equivalent_decl_instantiation_lower(dag, template, &specialized_arguments)
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
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => {
            let specialized_inputs: Vec<DeclarationId> = inputs
                .iter()
                .map(|input| specialize_decl_for_lowering(dag, *input, arguments, depth + 1))
                .collect();
            let specialized_output =
                specialize_decl_for_lowering(dag, output, arguments, depth + 1);
            if specialized_inputs == inputs && specialized_output == output {
                return current;
            }
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Arrow {
                    inputs: specialized_inputs,
                    output: specialized_output,
                    body,
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
        TypeConnective::Conj { children } => {
            let specialized_children: Vec<Field> = children
                .iter()
                .map(|child| Field {
                    label: child.label.clone(),
                    ty: specialize_decl_for_lowering(dag, child.ty, arguments, depth + 1),
                })
                .collect();
            if specialized_children.len() == children.len()
                && specialized_children
                    .iter()
                    .zip(children.iter())
                    .all(|(lhs, rhs)| lhs.label == rhs.label && lhs.ty == rhs.ty)
            {
                return current;
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
                span: decl.span,
            });
            id
        }
        TypeConnective::Disj { variants } => {
            let specialized_variants: Vec<Field> = variants
                .iter()
                .map(|variant| Field {
                    label: variant.label.clone(),
                    ty: specialize_decl_for_lowering(dag, variant.ty, arguments, depth + 1),
                })
                .collect();
            if specialized_variants.len() == variants.len()
                && specialized_variants
                    .iter()
                    .zip(variants.iter())
                    .all(|(lhs, rhs)| lhs.label == rhs.label && lhs.ty == rhs.ty)
            {
                return current;
            }
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Disj {
                    variants: specialized_variants,
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
        TypeConnective::Cardinality { .. }
        | TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::Literal(_)) => current,
    }
}

fn unresolved_port(dag: &mut Dag, diagnostic: Diagnostic) -> PortId {
    let port = dag.alloc_port(None);
    dag.mark_unresolved(port, diagnostic);
    port
}

fn lower_field_path_expr(
    segments: &[String],
    span: &SourceSpan,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
    symbols: &HashMap<String, DeclarationId>,
) -> PortId {
    let Some((head, rest)) = segments.split_first() else {
        return unresolved_port(
            dag,
            Diagnostic::ResolveError {
                name: "empty dotted path expression".to_string(),
                span: span.clone(),
            },
        );
    };
    // Path A: head is in local variable scope (fn params, let bindings,
    // match-arm pattern bindings). Existing type-driven walk.
    if let Some(&port) = scope.get(head) {
        let mut current_port = port;
        for field_label in rest {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            let static_resolution = resolve_static_field_project(dag, current_port, field_label);
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: TransformTarget::FieldProject {
                    field_label: field_label.clone(),
                    field_child: static_resolution.map(|(field_child, _)| field_child),
                },
                inputs: vec![current_port],
                output,
                span: span.clone(),
            }));
            if let Some((_, ty)) = static_resolution {
                dag.set_port_type(output, ty);
            }
            current_port = output;
        }
        return current_port;
    }
    // Path B: DB-10 (3a.2) — head resolves to a `data` declaration
    // with a structural value_body. Walk the compile-time value
    // literal rather than performing a runtime FieldProject walk.
    // `cfg.host` where cfg: Config = { host: 1, port: 8080 } becomes
    // a Value(Int(1)) node directly.
    if let Some(&decl_id) = symbols.get(head) {
        let value_body_opt = dag.declaration(decl_id).value_body.clone();
        if let Some(value_body) = value_body_opt {
            if let Some(port) = resolve_data_path(dag, &value_body, rest, span) {
                return port;
            }
        }
    }
    unresolved_port(
        dag,
        Diagnostic::ResolveError {
            name: format!(
                "dotted path `{}` is not a local field access; expression-position dotted paths currently require a local-variable head or a `data` declaration with a compile-time value",
                segments.join("."),
            ),
            span: span.clone(),
        },
    )
}

/// DB-10 (3a.2): walk a sequence of field segments through a data
/// declaration's `ValueBody`, emitting a `Value` node at the terminal
/// scalar literal. Returns `None` if the path cannot be fully
/// resolved — caller falls back to an unresolved diagnostic.
///
/// Only scalar literals terminate the walk; nested `Record` payloads
/// admit further descent but `List`, `Map`, and `Variant` terminal
/// reads are out of scope for 3a.2 and fall through to `None`.
fn resolve_data_path(
    dag: &mut Dag,
    value_body: &crate::dag::ValueBody,
    segments: &[String],
    span: &SourceSpan,
) -> Option<PortId> {
    match value_body {
        crate::dag::ValueBody::Unparsed(_) => None,
        crate::dag::ValueBody::Scalar(bits) => {
            if !segments.is_empty() {
                return None;
            }
            Some(emit_literal_as_value_port(dag, bits.clone(), span))
        }
        crate::dag::ValueBody::Structural { fields } => {
            resolve_structural_field_path(dag, fields, segments, span)
        }
    }
}

fn resolve_structural_field_path(
    dag: &mut Dag,
    fields: &[(String, crate::dag::FieldValue)],
    segments: &[String],
    span: &SourceSpan,
) -> Option<PortId> {
    let (head, rest) = segments.split_first()?;
    let field_value = fields
        .iter()
        .find(|(label, _)| label == head)
        .map(|(_, v)| v.clone())?;
    if rest.is_empty() {
        return emit_field_value_as_port(dag, &field_value, span);
    }
    match &field_value {
        crate::dag::FieldValue::Record(inner) => {
            resolve_structural_field_path(dag, inner, rest, span)
        }
        _ => None,
    }
}

fn emit_field_value_as_port(
    dag: &mut Dag,
    fv: &crate::dag::FieldValue,
    span: &SourceSpan,
) -> Option<PortId> {
    match fv {
        crate::dag::FieldValue::Literal(bits) => {
            Some(emit_literal_as_value_port(dag, bits.clone(), span))
        }
        // Reference / Record / List / Variant terminal reads require
        // more emission machinery than DB-10 covers (acceptance is
        // scalar-at-leaf). Leave unresolved; call site falls back to
        // an unresolved diagnostic.
        _ => None,
    }
}

/// DB-10 (3a.2): emit a scalar literal from its `LiteralBits`
/// directly as a `Behavior::Value` node. Shared by
/// `ValueBody::Scalar` inlining and `FieldValue::Literal` terminal
/// reads.
fn emit_literal_as_value_port(dag: &mut Dag, bits: LiteralBits, span: &SourceSpan) -> PortId {
    let node_id = dag.alloc_node_id();
    let output = dag.alloc_port(Some(node_id));
    dag.push_node(Behavior::Value(ValueNode {
        id: node_id,
        data: bits,
        output,
        span: span.clone(),
    }));
    output
}

fn lower_payload_binding(dag: &mut Dag, binding_name: &str) -> PayloadBinding {
    let payload_port = dag.alloc_port(None);
    PayloadBinding {
        binding_name: binding_name.to_string(),
        payload_port,
    }
}

struct LoweredMatchArm {
    output: PortId,
    body_node: Option<NodeId>,
    pattern_name: String,
    pattern_span: SourceSpan,
    binding: Option<PayloadBinding>,
}

fn lower_expr(
    expr: &SurfaceExpr,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
    callable_scope: &CallableScope,
    symbols: &HashMap<String, DeclarationId>,
    expected_decl: Option<DeclarationId>,
) -> PortId {
    match expr {
        SurfaceExpr::Literal { value, span } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            let data = match value {
                SurfaceLiteral::Int(v) => LiteralBits::Int(*v),
                SurfaceLiteral::Bool(v) => LiteralBits::Bool(*v),
                SurfaceLiteral::String(v) => LiteralBits::String(v.clone()),
            };
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data,
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::Var { name, span } => match scope.get(name) {
            Some(port) => *port,
            None => {
                if let Some(target) = resolve_expected_variant_constructor(dag, expected_decl, name)
                {
                    return lower_constructor_invocation(dag, target, Vec::new(), span.clone());
                }
                // DB-10 (3a.2): if `name` resolves to a `data` decl
                // with a scalar value_body, inline the value at this
                // use site as a Value(LiteralBits) node. Emission then
                // renders the target-native literal without extra
                // wiring. Record-valued data references are currently
                // unsupported — those sites fall through to the
                // existing unresolved diagnostic (acceptance test
                // `test_3a2_data_referenced_in_fn_body_compiles`
                // uses scalar only).
                if let Some(decl_id) = symbols.get(name) {
                    if let Some(crate::dag::ValueBody::Scalar(bits)) =
                        &dag.declaration(*decl_id).value_body
                    {
                        let bits = bits.clone();
                        return emit_literal_as_value_port(dag, bits, span);
                    }
                }
                let port = dag.alloc_port(None);
                dag.mark_unresolved(
                    port,
                    Diagnostic::ResolveError {
                        name: name.clone(),
                        span: span.clone(),
                    },
                );
                port
            }
        },
        SurfaceExpr::Call { target, args, span } => {
            let base_target_decl = resolve_expected_variant_constructor(dag, expected_decl, target)
                .or_else(|| callable_scope.get(target).copied())
                .or_else(|| symbols.get(target).copied())
                .unwrap_or_else(|| alloc_identifier_stub(dag, target, span));
            let target_inputs = direct_invocation_input_decls(dag, base_target_decl, 0);
            let mut input_ports: Vec<PortId> = Vec::new();
            let mut template_arguments: Vec<TemplateArgument> = Vec::new();
            if let (Some(expected_result), Some(target_output)) = (
                expected_decl,
                declaration_callable_output(dag, base_target_decl, 0),
            ) {
                let _ = bind_expected_type_to_actual(
                    dag,
                    target_output,
                    expected_result,
                    &mut template_arguments,
                    0,
                );
            }
            for (idx, arg) in args.iter().enumerate() {
                let raw_expected_input = target_inputs
                    .as_ref()
                    .and_then(|inputs| inputs.get(idx))
                    .copied();
                let specialized_expected_input = raw_expected_input.map(|expected_decl| {
                    specialize_decl_for_lowering(dag, expected_decl, &template_arguments, 0)
                });
                if let Some(expected_decl) = raw_expected_input {
                    if declaration_is_callable(dag, expected_decl, 0) {
                        let specialized_expected =
                            specialized_expected_input.unwrap_or(expected_decl);
                        let actual_callable = match arg {
                            SurfaceExpr::Lambda { params, body, span } => {
                                let mut lambda_ctx = LambdaLoweringContext {
                                    dag,
                                    scope,
                                    callable_scope,
                                    symbols,
                                };
                                match lower_lambda_expr(
                                    params,
                                    body,
                                    span,
                                    specialized_expected,
                                    &mut lambda_ctx,
                                ) {
                                    Ok(lambda_decl_id) => lambda_decl_id,
                                    Err(diag) => {
                                        report_declaration_error(lambda_ctx.dag, diag);
                                        alloc_identifier_stub(lambda_ctx.dag, "__lambda__", span)
                                    }
                                }
                            }
                            _ => resolve_callable_reference(arg, dag, callable_scope, symbols),
                        };
                        let matches_expected = push_template_argument_binding(
                            &mut template_arguments,
                            expected_decl,
                            actual_callable,
                        ) && bind_expected_type_to_actual(
                            dag,
                            specialized_expected,
                            actual_callable,
                            &mut template_arguments,
                            0,
                        );
                        if !matches_expected {
                            let port = dag.alloc_port(None);
                            dag.mark_unresolved(
                                port,
                                callable_binding_conflict_diagnostic(target, idx, expr_span(arg)),
                            );
                            return port;
                        }
                        continue;
                    }
                }
                let lowered = lower_expr(
                    arg,
                    dag,
                    scope,
                    callable_scope,
                    symbols,
                    specialized_expected_input,
                );
                if let Some(expected_decl) = raw_expected_input {
                    if let crate::dag::PortState::Resolved(actual_ty) = dag.port(lowered).state() {
                        let _ = bind_expected_type_to_actual(
                            dag,
                            expected_decl,
                            actual_ty.declaration,
                            &mut template_arguments,
                            0,
                        );
                    }
                }
                input_ports.push(lowered);
            }
            let retained_arguments =
                retained_template_arguments_for_target(dag, base_target_decl, &template_arguments);
            let target_decl = if retained_arguments.is_empty() {
                base_target_decl
            } else {
                let instantiation_id = dag.alloc_declaration_id();
                dag.push_declaration(Declaration {
                    id: instantiation_id,
                    name: None,
                    connective: TypeConnective::Instantiation {
                        template: base_target_decl,
                        arguments: retained_arguments,
                    },
                    type_params: Vec::new(),
                    meta_tag: None,
                    inhabits: None,
                    value_body: None,
                    refinement: None,
                    span: span.clone(),
                });
                instantiation_id
            };
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: TransformTarget::Callable(target_decl),
                inputs: input_ports,
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::Operator { op, args, span } => {
            let input_ports: Vec<PortId> = args
                .iter()
                .map(|a| lower_expr(a, dag, scope, callable_scope, symbols, None))
                .collect();
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: TransformTarget::Operator(*op),
                inputs: input_ports,
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::Lambda { span, .. } => {
            let port = dag.alloc_port(None);
            dag.mark_unresolved(
                port,
                Diagnostic::ResolveError {
                    name: "lambda expression requires an expected function type at this position"
                        .to_string(),
                    span: span.clone(),
                },
            );
            port
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            span,
        } => {
            let cond_port = lower_expr(cond, dag, scope, callable_scope, symbols, None);
            // DB-11 (3a.3) arm-local refinement narrowing. If `cond` is
            // a predicate applied to a scope-bound Var (e.g. `d != 0`
            // where `d` is a parameter), allocate a narrowed port in
            // the then-arm whose type carries a refinement matching
            // the predicate. Call-site refinement discharge in
            // `decide_transform` then sees the refinement on the
            // narrowed port and accepts calls that would otherwise
            // fail. No narrowing on the else-arm — by convention the
            // else-arm's refinement would be the negation, and DB-11
            // does not model negated predicates (structural equality
            // only).
            let then_scope_owned = narrow_scope_for_predicate(cond, scope, symbols, dag);
            let then_scope_ref = then_scope_owned.as_ref().unwrap_or(scope);
            let then_port = lower_expr(
                then_branch,
                dag,
                then_scope_ref,
                callable_scope,
                symbols,
                expected_decl,
            );
            let else_port = lower_expr(
                else_branch,
                dag,
                scope,
                callable_scope,
                symbols,
                expected_decl,
            );
            let branch_id = dag.alloc_node_id();
            let branch_output = dag.alloc_port(Some(branch_id));
            let then_body = producer_of(dag, then_port).unwrap_or(branch_id);
            let else_body = producer_of(dag, else_port).unwrap_or(branch_id);
            // if-then-else is a match on Bool with two arms. Emit
            // explicit pattern labels so the discriminator lives
            // structurally on Path, not by positional convention.
            // Infer resolves each "True"/"False" against Bool's
            // Disj children.
            dag.push_node(Behavior::Branch(BranchNode {
                id: branch_id,
                input: cond_port,
                paths: vec![
                    Path {
                        body: then_body,
                        output: then_port,
                        pattern: BranchPattern::UnresolvedVariant {
                            name: "True".to_string(),
                            span: expr_span(then_branch).clone(),
                        },
                        binding: None,
                    },
                    Path {
                        body: else_body,
                        output: else_port,
                        pattern: BranchPattern::UnresolvedVariant {
                            name: "False".to_string(),
                            span: expr_span(else_branch).clone(),
                        },
                        binding: None,
                    },
                ],
                output: branch_output,
                span: span.clone(),
            }));
            branch_output
        }
        SurfaceExpr::Match {
            scrutinee,
            arms,
            span,
        } => {
            // Lower scrutinee and all arm bodies FIRST. `alloc_node_id`
            // comes after so the Branch's id matches `nodes.len()`
            // at push time (arm lowering pushes its own nodes). Same
            // ordering pattern as the `If` arm above.
            let scrutinee_port = lower_expr(scrutinee, dag, scope, callable_scope, symbols, None);
            let mut lowered_arms: Vec<LoweredMatchArm> = Vec::with_capacity(arms.len());
            for arm in arms {
                let mut arm_scope = scope.clone();
                let (pattern_name, pattern_span, binding) = match &arm.pattern {
                    SurfacePattern::BareVariant { name, span } => {
                        (name.clone(), span.clone(), None)
                    }
                    SurfacePattern::VariantWith {
                        name,
                        binding,
                        span,
                    } => {
                        let payload_binding = lower_payload_binding(dag, binding);
                        arm_scope.insert(
                            payload_binding.binding_name.clone(),
                            payload_binding.payload_port,
                        );
                        (name.clone(), span.clone(), Some(payload_binding))
                    }
                };
                let arm_output_port = lower_expr(
                    &arm.body,
                    dag,
                    &arm_scope,
                    callable_scope,
                    symbols,
                    expected_decl,
                );
                let body_node = producer_of(dag, arm_output_port);
                lowered_arms.push(LoweredMatchArm {
                    output: arm_output_port,
                    body_node,
                    pattern_name,
                    pattern_span,
                    binding,
                });
            }
            let branch_id = dag.alloc_node_id();
            let branch_output = dag.alloc_port(Some(branch_id));
            let paths: Vec<Path> = lowered_arms
                .into_iter()
                .map(|arm| Path {
                    body: arm.body_node.unwrap_or(branch_id),
                    output: arm.output,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: arm.pattern_name,
                        span: arm.pattern_span,
                    },
                    binding: arm.binding,
                })
                .collect();
            dag.push_node(Behavior::Branch(BranchNode {
                id: branch_id,
                input: scrutinee_port,
                paths,
                output: branch_output,
                span: span.clone(),
            }));
            branch_output
        }
        SurfaceExpr::Record { fields, span } => lower_record_literal_expr(
            fields,
            span,
            dag,
            scope,
            callable_scope,
            symbols,
            expected_decl,
        ),
        SurfaceExpr::List { elements, span } => lower_list_literal_expr(
            elements,
            span,
            dag,
            scope,
            callable_scope,
            symbols,
            expected_decl,
        ),
        SurfaceExpr::Path { segments, span } => {
            lower_field_path_expr(segments, span, dag, scope, symbols)
        }
    }
}

fn lower_constructor_invocation(
    dag: &mut Dag,
    target: DeclarationId,
    inputs: Vec<PortId>,
    span: SourceSpan,
) -> PortId {
    let node_id = dag.alloc_node_id();
    let output = dag.alloc_port(Some(node_id));
    dag.push_node(Behavior::Transform(TransformNode {
        id: node_id,
        target: TransformTarget::Callable(target),
        inputs,
        output,
        span,
    }));
    output
}

fn direct_invocation_input_decls(
    dag: &Dag,
    current: DeclarationId,
    depth: usize,
) -> Option<Vec<DeclarationId>> {
    if depth >= 32 {
        return None;
    }
    match &dag.declaration(current).connective {
        TypeConnective::Arrow { inputs, .. } => Some(inputs.clone()),
        TypeConnective::Conj { children } => Some(children.iter().map(|child| child.ty).collect()),
        TypeConnective::Instantiation { template, .. } => {
            direct_invocation_input_decls(dag, *template, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            direct_invocation_input_decls(dag, *next, depth + 1)
        }
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
        | TypeConnective::Atom(AtomPayload::TypeParam(_))
        | TypeConnective::Atom(AtomPayload::Literal(_))
        | TypeConnective::Disj { .. }
        | TypeConnective::Cardinality { .. } => None,
    }
}

fn lower_record_literal_expr(
    fields: &[crate::parse::SurfaceRecordField],
    span: &SourceSpan,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
    callable_scope: &CallableScope,
    symbols: &HashMap<String, DeclarationId>,
    expected_decl: Option<DeclarationId>,
) -> PortId {
    let Some(target_decl) = expected_decl else {
        return unresolved_port(
            dag,
            Diagnostic::ResolveError {
                name: "record literals require an expected record type at this position"
                    .to_string(),
                span: span.clone(),
            },
        );
    };
    let Some(conj_id) = walk_to_conj_decl(dag, target_decl) else {
        return unresolved_port(
            dag,
            Diagnostic::ResolveError {
                name: "record literal does not have an expected record type at this position"
                    .to_string(),
                span: span.clone(),
            },
        );
    };
    let expected_fields: Vec<(String, DeclarationId)> = match &dag.declaration(conj_id).connective {
        TypeConnective::Conj { children } => children
            .iter()
            .map(|child| (child.label.clone(), child.ty))
            .collect(),
        _ => unreachable!("walk_to_conj_decl returned non-Conj"),
    };
    for field in fields {
        if !expected_fields
            .iter()
            .any(|(label, _)| *label == field.name)
        {
            return unresolved_port(
                dag,
                Diagnostic::ResolveError {
                    name: format!(
                        "record literal has field `{}` but the expected type has no such field",
                        field.name
                    ),
                    span: field.span.clone(),
                },
            );
        }
    }
    let mut inputs = Vec::with_capacity(expected_fields.len());
    for (label, field_ty) in expected_fields {
        let Some(field) = fields.iter().find(|field| field.name == label) else {
            return unresolved_port(
                dag,
                Diagnostic::ResolveError {
                    name: format!(
                        "record literal is missing required field `{label}` for the expected type"
                    ),
                    span: span.clone(),
                },
            );
        };
        inputs.push(lower_expr(
            &field.value,
            dag,
            scope,
            callable_scope,
            symbols,
            Some(field_ty),
        ));
    }
    lower_constructor_invocation(dag, target_decl, inputs, span.clone())
}

fn lower_list_literal_expr(
    elements: &[SurfaceExpr],
    span: &SourceSpan,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
    callable_scope: &CallableScope,
    symbols: &HashMap<String, DeclarationId>,
    expected_decl: Option<DeclarationId>,
) -> PortId {
    let empty_decl = match symbols.get("empty").copied() {
        Some(id) => id,
        None => {
            return unresolved_port(
                dag,
                Diagnostic::ResolveError {
                    name: "std `empty` constructor is unavailable while lowering a list literal"
                        .to_string(),
                    span: span.clone(),
                },
            );
        }
    };
    let singleton_decl = match symbols.get("singleton").copied() {
        Some(id) => id,
        None => {
            return unresolved_port(
                dag,
                Diagnostic::ResolveError {
                    name:
                        "std `singleton` constructor is unavailable while lowering a list literal"
                            .to_string(),
                    span: span.clone(),
                },
            );
        }
    };
    let cons_decl = match symbols.get("cons").copied() {
        Some(id) => id,
        None => {
            return unresolved_port(
                dag,
                Diagnostic::ResolveError {
                    name: "std `cons` constructor is unavailable while lowering a list literal"
                        .to_string(),
                    span: span.clone(),
                },
            );
        }
    };

    if elements.is_empty() {
        return lower_constructor_invocation(dag, empty_decl, Vec::new(), span.clone());
    }

    let element_expected = expected_decl.and_then(|decl| list_element_type(dag, decl));
    let mut element_ports: Vec<PortId> = elements
        .iter()
        .map(|element| {
            lower_expr(
                element,
                dag,
                scope,
                callable_scope,
                symbols,
                element_expected,
            )
        })
        .collect();
    let last = element_ports
        .pop()
        .expect("non-empty list literal has a last element");
    let mut current = lower_constructor_invocation(
        dag,
        singleton_decl,
        vec![last],
        expr_span(elements.last().expect("non-empty")).clone(),
    );
    while let Some(head) = element_ports.pop() {
        current = lower_constructor_invocation(dag, cons_decl, vec![head, current], span.clone());
    }
    current
}

fn resolve_expected_variant_constructor(
    dag: &mut Dag,
    expected_decl: Option<DeclarationId>,
    variant_name: &str,
) -> Option<DeclarationId> {
    let expected_decl = expected_decl?;
    let expected_span = dag.declaration(expected_decl).span.clone();
    let disj_id = walk_to_disj_decl(dag, expected_decl)?;
    let variant_decl = match &dag.declaration(disj_id).connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .find(|variant| variant.label == variant_name)
            .map(|variant| variant.ty)?,
        _ => unreachable!("walk_to_disj_decl returned non-Disj"),
    };
    let instantiation_arguments = match &dag.declaration(expected_decl).connective {
        TypeConnective::Instantiation { arguments, .. } if !arguments.is_empty() => {
            Some(arguments.clone())
        }
        _ => None,
    };
    match instantiation_arguments {
        Some(arguments) => {
            let instantiation_id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id: instantiation_id,
                name: None,
                connective: TypeConnective::Instantiation {
                    template: variant_decl,
                    arguments,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                span: expected_span,
            });
            Some(instantiation_id)
        }
        _ => Some(variant_decl),
    }
}

fn producer_of(dag: &Dag, port: PortId) -> Option<NodeId> {
    dag.port(port).produced_by
}

fn is_recursive(expr: &SurfaceExpr, self_name: &str) -> bool {
    match expr {
        SurfaceExpr::Literal { .. }
        | SurfaceExpr::Var { .. }
        | SurfaceExpr::Path { .. }
        | SurfaceExpr::List { .. } => false,
        SurfaceExpr::Call { target, args, .. } => {
            if target == self_name {
                return true;
            }
            args.iter().any(|a| is_recursive(a, self_name))
        }
        SurfaceExpr::Operator { args, .. } => args.iter().any(|a| is_recursive(a, self_name)),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            is_recursive(cond, self_name)
                || is_recursive(then_branch, self_name)
                || is_recursive(else_branch, self_name)
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            is_recursive(scrutinee, self_name)
                || arms.iter().any(|arm| is_recursive(&arm.body, self_name))
        }
        SurfaceExpr::Lambda { body, .. } => is_recursive(body, self_name),
        SurfaceExpr::Record { fields, .. } => {
            fields.iter().any(|f| is_recursive(&f.value, self_name))
        }
    }
}

/// Partial termination analysis: every recursive self-call's first argument
/// must be `first_param - <positive int>`. The surface shape is now
/// `SurfaceExpr::Operator { op: ArithmeticOp::Sub, args: [Var(first_param),
/// Literal(Int(k))] }` — the operator dispatch is committed at parse
/// time via `OperatorKind::Arithmetic(ArithmeticOp::Sub)`, so this check
/// is structural rather than a string match against `target == "-"`.
///
/// **On the remaining string comparisons** (`target == self_name`,
/// `name == first_param`): these are *not* name-based dispatch. Both
/// sides are parser-stage identifier strings from a single parse tree
/// reaching `lower_fn_item_expr_body` together — no declaration table
/// lookup, no cross-module symbol resolution. `self_name` is the fn's
/// declared name and `first_param` is its parameter name, both
/// captured from `SurfaceItem::Fn` immediately above on the call
/// stack. The strings on the other side (`SurfaceExpr::Call.target`,
/// `SurfaceExpr::Var.name`) are raw parser tokens from the body of
/// that same fn.
///
/// The alternative shape — comparing typed `NodeId`/`PortId` edges —
/// isn't available at M1(2.7) because the `BindNode` that owns this
/// function and its param ports doesn't exist yet: this analysis runs
/// *inside* `lower_fn_item_expr_body`, before the Bind is pushed onto
/// the node graph. A future post-lowering `DescentEvidence` lens
/// would walk the emitted Loop/Transform graph by typed id, but
/// that's a substrate extension, not a bridge to dissolve here.
#[derive(Debug, Clone, Default)]
struct StructuralBindingInfo {
    whole_payload_recursive: bool,
    recursive_fields: HashSet<String>,
}

fn descent_provable(
    expr: &SurfaceExpr,
    dag: &Dag,
    first_param_decl: DeclarationId,
    self_name: &str,
    first_param: &str,
    bindings: &HashMap<String, StructuralBindingInfo>,
) -> bool {
    match expr {
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } | SurfaceExpr::Path { .. } => true,
        SurfaceExpr::Call { target, args, .. } => {
            if target == self_name {
                match args.first() {
                    None => false,
                    Some(first_arg) => {
                        if !is_strictly_smaller(first_arg, first_param, bindings) {
                            return false;
                        }
                        args.iter().skip(1).all(|a| {
                            descent_provable(
                                a,
                                dag,
                                first_param_decl,
                                self_name,
                                first_param,
                                bindings,
                            )
                        })
                    }
                }
            } else {
                args.iter().all(|a| {
                    descent_provable(a, dag, first_param_decl, self_name, first_param, bindings)
                })
            }
        }
        SurfaceExpr::Operator { args, .. } => args
            .iter()
            .all(|a| descent_provable(a, dag, first_param_decl, self_name, first_param, bindings)),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            descent_provable(
                cond,
                dag,
                first_param_decl,
                self_name,
                first_param,
                bindings,
            ) && descent_provable(
                then_branch,
                dag,
                first_param_decl,
                self_name,
                first_param,
                bindings,
            ) && descent_provable(
                else_branch,
                dag,
                first_param_decl,
                self_name,
                first_param,
                bindings,
            )
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            if !descent_provable(
                scrutinee,
                dag,
                first_param_decl,
                self_name,
                first_param,
                bindings,
            ) {
                return false;
            }

            let scrutinee_is_first_param = matches!(
                scrutinee.as_ref(),
                SurfaceExpr::Var { name, .. } if name == first_param
            );
            arms.iter().all(|arm| {
                let mut arm_bindings = bindings.clone();
                if scrutinee_is_first_param {
                    if let SurfacePattern::VariantWith { name, binding, .. } = &arm.pattern {
                        if let Some(info) =
                            structural_binding_info_for_variant(dag, first_param_decl, name)
                        {
                            arm_bindings.insert(binding.clone(), info);
                        }
                    }
                }
                descent_provable(
                    &arm.body,
                    dag,
                    first_param_decl,
                    self_name,
                    first_param,
                    &arm_bindings,
                )
            })
        }
        SurfaceExpr::Lambda { body, .. } => descent_provable(
            body,
            dag,
            first_param_decl,
            self_name,
            first_param,
            bindings,
        ),
        SurfaceExpr::Record { fields, .. } => fields.iter().all(|f| {
            descent_provable(
                &f.value,
                dag,
                first_param_decl,
                self_name,
                first_param,
                bindings,
            )
        }),
        SurfaceExpr::List { elements, .. } => elements.iter().all(|element| {
            descent_provable(
                element,
                dag,
                first_param_decl,
                self_name,
                first_param,
                bindings,
            )
        }),
    }
}

fn is_strictly_smaller(
    expr: &SurfaceExpr,
    first_param: &str,
    bindings: &HashMap<String, StructuralBindingInfo>,
) -> bool {
    if is_structurally_smaller(expr, bindings) {
        return true;
    }
    let SurfaceExpr::Operator { op, args, .. } = expr else {
        return false;
    };
    // Structural check — no string match. Subtract is one of the four
    // arithmetic variants.
    if !matches!(op, OperatorKind::Arithmetic(ArithmeticOp::Sub)) || args.len() != 2 {
        return false;
    }
    let lhs_is_param = matches!(
        &args[0],
        SurfaceExpr::Var { name, .. } if name == first_param
    );
    let rhs_is_positive = matches!(
        &args[1],
        SurfaceExpr::Literal {
            value: SurfaceLiteral::Int(v), ..
        } if *v > 0
    );
    lhs_is_param && rhs_is_positive
}

fn is_structurally_smaller(
    expr: &SurfaceExpr,
    bindings: &HashMap<String, StructuralBindingInfo>,
) -> bool {
    match expr {
        SurfaceExpr::Var { name, .. } => bindings
            .get(name)
            .is_some_and(|info| info.whole_payload_recursive),
        SurfaceExpr::Path { segments, .. } if segments.len() == 2 => bindings
            .get(&segments[0])
            .is_some_and(|info| info.recursive_fields.contains(&segments[1])),
        _ => false,
    }
}

fn structural_binding_info_for_variant(
    dag: &Dag,
    first_param_decl: DeclarationId,
    variant_name: &str,
) -> Option<StructuralBindingInfo> {
    let disj_id = walk_to_disj_decl(dag, first_param_decl)?;
    let variant_decl_id = match &dag.declaration(disj_id).connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .find(|variant| variant.label == variant_name)
            .map(|variant| variant.ty)?,
        _ => unreachable!("walk_to_disj_decl returned non-Disj"),
    };
    let TypeConnective::Conj { children } = &dag.declaration(variant_decl_id).connective else {
        return None;
    };

    let mut info = StructuralBindingInfo::default();
    if children.len() == 1
        && children[0].label.as_str() == "_0"
        && recursive_decl_equivalent(dag, children[0].ty, first_param_decl)
    {
        info.whole_payload_recursive = true;
    }
    for child in children {
        if recursive_decl_equivalent(dag, child.ty, first_param_decl) {
            info.recursive_fields.insert(child.label.clone());
        }
    }
    Some(info)
}

fn recursive_decl_equivalent(dag: &Dag, lhs: DeclarationId, rhs: DeclarationId) -> bool {
    canonical_decl_for_descent(dag, lhs) == canonical_decl_for_descent(dag, rhs)
}

fn canonical_decl_for_descent(dag: &Dag, start: DeclarationId) -> DeclarationId {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => current = *next,
            _ => return current,
        }
    }
    current
}

fn expr_span(expr: &SurfaceExpr) -> &SourceSpan {
    match expr {
        SurfaceExpr::Literal { span, .. }
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::Path { span, .. }
        | SurfaceExpr::Call { span, .. }
        | SurfaceExpr::Operator { span, .. }
        | SurfaceExpr::Lambda { span, .. }
        | SurfaceExpr::If { span, .. }
        | SurfaceExpr::Match { span, .. }
        | SurfaceExpr::Record { span, .. }
        | SurfaceExpr::List { span, .. } => span,
    }
}

fn compute_mutually_recursive(items: &[SurfaceItem]) -> HashSet<String> {
    // Only expression-body fns participate in the mutual recursion
    // analysis — block-bodied `FnExternalBody` items have their bodies
    // deferred as `ArrowBody::Unparsed`, so there's no reachable call
    // graph to walk at M1(2.7).
    let fn_names: HashSet<String> = items
        .iter()
        .filter_map(|i| match i {
            SurfaceItem::Fn { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    let mut calls: HashMap<String, HashSet<String>> = HashMap::new();
    for item in items {
        if let SurfaceItem::Fn { name, body, .. } = item {
            let mut callees = HashSet::new();
            collect_calls(body, &fn_names, &mut callees);
            calls.insert(name.clone(), callees);
        }
    }

    let mut reach_cache: HashMap<String, HashSet<String>> = HashMap::new();
    for f in &fn_names {
        reach_cache.insert(f.clone(), transitive_reach(f, &calls));
    }

    let mut mutually = HashSet::new();
    for f in &fn_names {
        let reach_f = &reach_cache[f];
        for g in reach_f {
            if g == f {
                continue;
            }
            let reach_g = &reach_cache[g];
            if reach_g.contains(f) {
                mutually.insert(f.clone());
                mutually.insert(g.clone());
            }
        }
    }
    mutually
}

fn transitive_reach(start: &str, calls: &HashMap<String, HashSet<String>>) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = vec![start.to_string()];
    while let Some(f) = queue.pop() {
        if !visited.insert(f.clone()) {
            continue;
        }
        if let Some(callees) = calls.get(&f) {
            for c in callees {
                queue.push(c.clone());
            }
        }
    }
    visited.remove(start);
    visited
}

fn collect_calls(expr: &SurfaceExpr, fn_names: &HashSet<String>, out: &mut HashSet<String>) {
    match expr {
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } | SurfaceExpr::Path { .. } => {}
        SurfaceExpr::Call { target, args, .. } => {
            if fn_names.contains(target) {
                out.insert(target.clone());
            }
            for a in args {
                collect_calls(a, fn_names, out);
            }
        }
        SurfaceExpr::Operator { args, .. } => {
            // Operators never name user functions; just recurse into
            // the operand expressions.
            for a in args {
                collect_calls(a, fn_names, out);
            }
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_calls(cond, fn_names, out);
            collect_calls(then_branch, fn_names, out);
            collect_calls(else_branch, fn_names, out);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_calls(scrutinee, fn_names, out);
            for arm in arms {
                collect_calls(&arm.body, fn_names, out);
            }
        }
        SurfaceExpr::Lambda { body, .. } => collect_calls(body, fn_names, out),
        SurfaceExpr::Record { fields, .. } => {
            for f in fields {
                collect_calls(&f.value, fn_names, out);
            }
        }
        SurfaceExpr::List { elements, .. } => {
            for element in elements {
                collect_calls(element, fn_names, out);
            }
        }
    }
}
