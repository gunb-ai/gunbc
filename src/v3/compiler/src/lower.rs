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
    ArrowBody, AtomPayload, Behavior, BindNode, Bound, BranchNode, CardinalityBound, Dag,
    Declaration, DeclarationId, Field, LiteralBits, LoopNode, NodeId, Path, PortId,
    TemplateArgument, TransformNode, TransformTarget, TypeConnective, ValueNode,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::operators::{ArithmeticOp, OperatorKind};
use crate::parse::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceLiteral, SurfaceModule, SurfaceParam,
    SurfaceType, SurfaceVariant, VariantPayload,
};
use crate::types::TypeShape;

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
    let mutually_recursive = compute_mutually_recursive(&module.items);
    let mut scope: HashMap<String, PortId> = HashMap::new();
    for (idx, item) in module.items.iter().enumerate() {
        if !is_first[idx] {
            // Duplicate declaration — skipped at lower time so the
            // first-of-name's filled connective is not overwritten.
            // `collect_symbols` already emitted a fail-closed
            // diagnostic for the duplicate.
            continue;
        }
        scope = lower_item(item, dag, scope, symbols, &mutually_recursive);
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
    // Seed from already-present declarations with first-match semantics
    // (matches `Dag::declaration_by_name`). If a name appears multiple
    // times in the prior bootstrap batch — e.g., a cross-file duplicate
    // — the seed is idempotent: the first id wins.
    let mut symbols: HashMap<String, DeclarationId> = HashMap::new();
    for d in dag.declarations() {
        if let Some(name) = &d.name {
            symbols.entry(name.clone()).or_insert(d.id);
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
            SurfaceItem::Fn { name, .. } => (name.clone(), &[]),
            SurfaceItem::FnExternalBody { name, .. } => (name.clone(), &[]),
            SurfaceItem::Data { name, .. } => (name.clone(), &[]),
            SurfaceItem::TypeAtom {
                name,
                type_params,
                ..
            }
            | SurfaceItem::TypeRecord {
                name,
                type_params,
                ..
            }
            | SurfaceItem::TypeSum {
                name,
                type_params,
                ..
            }
            | SurfaceItem::TypeAlias {
                name,
                type_params,
                ..
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
                    connective: TypeConnective::Atom(AtomPayload::TypeParam(
                        param.clone(),
                    )),
                    type_params: Vec::new(),
                    meta_tag: None,
                    inhabits: None,
                    span: span.clone(),
                });
                param_ids.push(param_id);
            }
            dag.declaration_mut(id).type_params = param_ids;
        }

        if let Some(&existing_id) = symbols.get(&name) {
            is_first[idx] = false;
            let existing_span = dag.declaration(existing_id).span.clone();
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
    scope: HashMap<String, PortId>,
    symbols: &HashMap<String, DeclarationId>,
    mutually_recursive: &HashSet<String>,
) -> HashMap<String, PortId> {
    let mut scope = scope;
    match item {
        SurfaceItem::Let {
            name,
            type_ann,
            expr,
        } => {
            let value_port = lower_expr(expr, dag, &scope, symbols);
            if let Some(ty) = type_ann {
                match lower_type_for_port(ty, dag, symbols) {
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
            scope.insert(name.clone(), value_port);
            scope
        }
        SurfaceItem::Fn {
            name,
            params,
            return_type,
            body,
            span: _,
        } => lower_fn_item_expr_body(
            name,
            params,
            return_type,
            body,
            dag,
            scope,
            symbols,
            mutually_recursive,
        ),
        SurfaceItem::FnExternalBody {
            name,
            params,
            return_type,
            body_span,
            span: _,
        } => {
            lower_fn_item_unparsed(name, params, return_type, body_span, dag, symbols);
            scope
        }
        SurfaceItem::Data {
            name,
            ty,
            body_span: _,
            span: _,
        } => {
            lower_data_item(name, ty, dag, symbols);
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
    dag.declaration(parent_id)
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
        .collect()
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
            let arguments = build_template_arguments(
                dag, symbols, local, template_id, name, args, span,
            );
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
            let arguments = build_template_arguments(
                dag, symbols, local, template, name, args, span,
            );
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
    let template_is_stub = matches!(
        &dag.declaration(template).connective,
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
fn alloc_identifier_stub(
    dag: &mut Dag,
    name: &str,
    span: &SourceSpan,
) -> DeclarationId {
    let id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id,
        name: None,
        connective: TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name.to_string())),
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
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
fn template_param_id(
    dag: &Dag,
    template: DeclarationId,
    idx: usize,
) -> Option<DeclarationId> {
    dag.declaration(template).type_params.get(idx).copied()
}

/// Lower a `SurfaceItem::FnExternalBody` — a block-bodied fn whose
/// body is not expressible in the M1(2.7) surface grammar. The
/// declaration's connective becomes an `Arrow` carrying
/// `ArrowBody::Unparsed(body_span)`. Inputs and output flow through
/// `type_to_declaration_id` so the signature matches what a fully
/// lowered `Fn` would produce.
///
/// **Scaffold honesty** (QW1): the fact "this name declares a fn
/// with this signature" flows forward through the declaration
/// table. Callers that reference the name resolve it correctly
/// against the Arrow signature at M1(2.7). The body is preserved
/// by `SourceSpan` for M2+ parser extensions to complete the
/// lowering.
fn lower_fn_item_unparsed(
    name: &str,
    params: &[SurfaceParam],
    return_type: &SurfaceType,
    body_span: &SourceSpan,
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
) {
    let fn_decl_id = symbols[name];
    let local: HashMap<String, DeclarationId> = HashMap::new();
    let param_decl_inputs: Vec<DeclarationId> = params
        .iter()
        .map(|p| type_to_declaration_id(&p.ty, symbols, &local, dag))
        .collect();
    let return_decl_id = type_to_declaration_id(return_type, symbols, &local, dag);
    dag.declaration_mut(fn_decl_id).connective = TypeConnective::Arrow {
        inputs: param_decl_inputs,
        output: return_decl_id,
        body: ArrowBody::Unparsed(body_span.clone()),
    };
}

/// Lower a `SurfaceItem::Data` — a typed constant whose body is not
/// yet parseable. The declaration's connective becomes the resolved
/// type annotation (walked through `type_to_connective` so aliases
/// land as `Instantiation`s, not one-level-indirect wrappers). The
/// body span is preserved on the SurfaceItem for M2+ but is not
/// re-stored on the declaration itself — lowering just commits the
/// type fact.
///
/// **Scaffold honesty** (QW2): `dsl/std/*.dag` data declarations like
/// `data kernel_algebra_profile: KernelAlgebraProfile = { ... }` now
/// survive into the declaration table as named declarations typed by
/// `KernelAlgebraProfile`. The body payload is still dropped, but
/// the fact that the name exists with that type flows forward.
fn lower_data_item(
    name: &str,
    ty: &SurfaceType,
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
) {
    let decl_id = symbols[name];
    let local: HashMap<String, DeclarationId> = HashMap::new();
    let connective = type_to_connective(ty, symbols, &local, dag);
    dag.declaration_mut(decl_id).connective = connective;
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

    // 1. Allocate parameter ports and set declared port types. Unknown
    //    names fail-closed via mark_unresolved; the sentinel TypeShape
    //    returned from the Err arm is never observed by inference
    //    (the port is already Unresolved).
    let mut param_ports: Vec<PortId> = Vec::with_capacity(params.len());
    let mut param_types: Vec<TypeShape> = Vec::with_capacity(params.len());
    let mut body_scope: HashMap<String, PortId> = outer_scope.clone();
    let mut param_decl_inputs: Vec<DeclarationId> = Vec::with_capacity(params.len());
    let local: HashMap<String, DeclarationId> = HashMap::new();
    for param in params {
        let port = dag.alloc_port(None);
        // Port TypeShape and Arrow-declaration input id come from the
        // same type_to_declaration_id walk, wrapped through
        // `lower_type_for_port` so port-side fail-closed
        // diagnostics anchor to the annotation span.
        let input_decl = type_to_declaration_id(&param.ty, symbols, &local, dag);
        param_decl_inputs.push(input_decl);
        let ty = match lower_type_for_port(&param.ty, dag, symbols) {
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
        param_ports.push(port);
        param_types.push(ty);
    }

    // 2. Compute return type (both port-side and declaration-side). Same
    //    two-query structure as params: declaration-side for the Arrow,
    //    port-side for the bind's return port.
    let return_decl_id = type_to_declaration_id(return_type, symbols, &local, dag);
    let return_ty = match lower_type_for_port(return_type, dag, symbols) {
        Ok(ty) => ty,
        Err(diag) => {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(err_port, diag);
            sentinel_type_shape(dag)
        }
    };

    // 3. Mutual recursion check — same as M0. Reject with an Unresolved
    //    placeholder and keep the fn's Declaration as a placeholder Arrow
    //    with body=Pending so call sites produce a cascade.
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
            body: ArrowBody::Pending,
        };
        let mut outer_scope = outer_scope;
        outer_scope.insert(name.to_string(), err_port);
        return outer_scope;
    }

    // 4. Lower the body.
    let body_return_port = lower_expr(body, dag, &body_scope, symbols);
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
        } else if !descent_provable(body, name, &params[0].name) {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(
                err_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "cannot prove recursion in `{name}` terminates; expected each recursive call's first argument to be `{param} - <positive int>`",
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
/// Lower a user-code type annotation to a port `TypeShape`.
///
/// **Dissolution receipt — QW5 SINGLE AUTHORITY.** Before M1(2.7) this
/// function hardcoded `"Int" | "Bool" | "String"` as a whitelist
/// parallel to `type_to_declaration_id`'s structural walk. The
/// whitelist is gone: the port-side authority is the same declaration
/// table that the type-record / type-sum / type-alias lowering paths
/// use. A user-code `let x: Foo = ...` annotation where `Foo` is any
/// known declaration resolves through the same path as the field
/// types inside `type R { x: Foo }`.
///
/// The one remaining structural check is a fail-closed guard: if
/// `type_to_declaration_id` allocated a fresh anonymous
/// `UnresolvedIdentifier` stub (because the name was not in the
/// symbol table), return `Err` so the caller marks the port
/// `Unresolved`. The resolve sweep ALSO fires against the stub at
/// the end of lowering, but this path anchors the failure to the
/// annotation span rather than a phantom port.
fn lower_type_for_port(
    ty: &SurfaceType,
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
) -> Result<TypeShape, Diagnostic> {
    let local: HashMap<String, DeclarationId> = HashMap::new();
    let decl_id = type_to_declaration_id(ty, symbols, &local, dag);
    let decl = dag.declaration(decl_id);
    if decl.name.is_none() {
        if let TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) = &decl.connective {
            return Err(Diagnostic::ResolveError {
                name: format!("unknown type `{name}`"),
                span: ty.span().clone(),
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

fn lower_expr(
    expr: &SurfaceExpr,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
    symbols: &HashMap<String, DeclarationId>,
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
            let input_ports: Vec<PortId> = args
                .iter()
                .map(|a| lower_expr(a, dag, scope, symbols))
                .collect();
            let target_decl = symbols.get(target).copied().unwrap_or_else(|| {
                alloc_identifier_stub(dag, target, span)
            });
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
                .map(|a| lower_expr(a, dag, scope, symbols))
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
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            span,
        } => {
            let cond_port = lower_expr(cond, dag, scope, symbols);
            let then_port = lower_expr(then_branch, dag, scope, symbols);
            let else_port = lower_expr(else_branch, dag, scope, symbols);
            let branch_id = dag.alloc_node_id();
            let branch_output = dag.alloc_port(Some(branch_id));
            let then_body = producer_of(dag, then_port).unwrap_or(branch_id);
            let else_body = producer_of(dag, else_port).unwrap_or(branch_id);
            dag.push_node(Behavior::Branch(BranchNode {
                id: branch_id,
                input: cond_port,
                paths: vec![
                    Path {
                        body: then_body,
                        output: then_port,
                    },
                    Path {
                        body: else_body,
                        output: else_port,
                    },
                ],
                output: branch_output,
                span: span.clone(),
            }));
            branch_output
        }
    }
}

fn producer_of(dag: &Dag, port: PortId) -> Option<NodeId> {
    dag.port(port).produced_by
}

fn is_recursive(expr: &SurfaceExpr, self_name: &str) -> bool {
    match expr {
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } => false,
        SurfaceExpr::Call { target, args, .. } => {
            if target == self_name {
                return true;
            }
            args.iter().any(|a| is_recursive(a, self_name))
        }
        SurfaceExpr::Operator { args, .. } => {
            args.iter().any(|a| is_recursive(a, self_name))
        }
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
fn descent_provable(expr: &SurfaceExpr, self_name: &str, first_param: &str) -> bool {
    match expr {
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } => true,
        SurfaceExpr::Call { target, args, .. } => {
            if target == self_name {
                match args.first() {
                    None => false,
                    Some(first_arg) => {
                        if !is_strictly_smaller(first_arg, first_param) {
                            return false;
                        }
                        args.iter()
                            .skip(1)
                            .all(|a| descent_provable(a, self_name, first_param))
                    }
                }
            } else {
                args.iter().all(|a| descent_provable(a, self_name, first_param))
            }
        }
        SurfaceExpr::Operator { args, .. } => args
            .iter()
            .all(|a| descent_provable(a, self_name, first_param)),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            descent_provable(cond, self_name, first_param)
                && descent_provable(then_branch, self_name, first_param)
                && descent_provable(else_branch, self_name, first_param)
        }
    }
}

fn is_strictly_smaller(expr: &SurfaceExpr, first_param: &str) -> bool {
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

fn expr_span(expr: &SurfaceExpr) -> &SourceSpan {
    match expr {
        SurfaceExpr::Literal { span, .. }
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::Call { span, .. }
        | SurfaceExpr::Operator { span, .. }
        | SurfaceExpr::If { span, .. } => span,
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

fn transitive_reach(
    start: &str,
    calls: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
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

fn collect_calls(
    expr: &SurfaceExpr,
    fn_names: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    match expr {
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } => {}
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
    }
}
