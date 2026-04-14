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
//   Call                     → Transform { target: DeclarationId, inputs }
//   If                       → Branch with 2 Paths
//   Fn item                  → Bind with non-empty params + optional Loop wrapper
//   Let item                 → Bind with empty params
//
// Transform.target is a DeclarationId. User function calls resolve at
// lower time via the symbol table (pass 1 allocated the fn's Declaration).
// Operator calls (target "+", "-", ...) resolve to an anonymous Identifier
// Atom declaration with resolved=None; inference later walks inhabitance
// to fill in the concrete algebra field (M1_DESIGN.md §8.9).

use std::collections::{HashMap, HashSet};

use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BindNode, Bound, BranchNode, CardinalityBound, Dag,
    Declaration, DeclarationId, Field, LiteralBits, LoopNode, NodeId, Path, PortId,
    TemplateArgument, TransformNode, TypeConnective, ValueNode,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::parse::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceModule, SurfaceParam, SurfaceType,
    SurfaceVariant,
};
use crate::types::{Prim, TypeShape};

pub fn lower(module: &SurfaceModule) -> Dag {
    let mut dag = Dag::new();
    lower_into(&mut dag, module);
    // User-module lowering may have emitted Identifier stubs for forward
    // references (`let x: Foo = ...` before `type Foo`). The sweep either
    // fills in each stub's `resolved` slot or surfaces a fail-closed
    // ResolveError via a phantom port.
    resolve_pending_identifiers(&mut dag);
    dag
}

/// Lower a surface module into an existing Dag. Used by bootstrap.rs to
/// layer the four fixture modules (logic, bit, algebra, types) onto the
/// same declaration table; each call's symbol collection seeds from the
/// declarations already present from prior calls. Intentionally does NOT
/// run `resolve_pending_identifiers` — callers are responsible for
/// batching the sweep so cross-file forward references (algebra → Bool
/// from types) resolve at the batch boundary.
pub(crate) fn lower_into(dag: &mut Dag, module: &SurfaceModule) {
    let symbols = collect_symbols(dag, &module.items);
    let mutually_recursive = compute_mutually_recursive(&module.items);
    let mut scope: HashMap<String, PortId> = HashMap::new();
    for item in &module.items {
        scope = lower_item(item, dag, scope, &symbols, &mutually_recursive);
    }
}

/// Pass 1: allocate a placeholder Declaration for every named top-level
/// item and record `name → DeclarationId` in the symbol table. Let items
/// are skipped — they produce behaviors, not declarations.
fn collect_symbols(
    dag: &mut Dag,
    items: &[SurfaceItem],
) -> HashMap<String, DeclarationId> {
    let mut symbols: HashMap<String, DeclarationId> = dag
        .declarations()
        .iter()
        .filter_map(|d| d.name.as_ref().map(|n| (n.clone(), d.id)))
        .collect();

    for item in items {
        let name = match item {
            SurfaceItem::Let { .. } => continue,
            SurfaceItem::Fn { name, .. } => name.clone(),
            SurfaceItem::TypeAtom { name, .. } => name.clone(),
            SurfaceItem::TypeRecord { name, .. } => name.clone(),
            SurfaceItem::TypeSum { name, .. } => name.clone(),
            SurfaceItem::TypeAlias { name, .. } => name.clone(),
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
            span,
        });
        symbols.insert(name, id);
    }
    symbols
}

fn placeholder_connective(name: &str) -> TypeConnective {
    TypeConnective::Atom(AtomPayload::Identifier {
        name: name.to_string(),
        resolved: None,
    })
}

fn item_span(item: &SurfaceItem) -> SourceSpan {
    match item {
        SurfaceItem::Let { expr, .. } => expr_span(expr).clone(),
        SurfaceItem::Fn { body, .. } => expr_span(body).clone(),
        SurfaceItem::TypeAtom { span, .. }
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
                match lower_type_for_port(ty) {
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
        } => lower_fn_item(
            name,
            params,
            return_type,
            body,
            dag,
            scope,
            symbols,
            mutually_recursive,
        ),
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

/// Allocate TypeParam Atom declarations for each surface type parameter and
/// populate the parent declaration's canonical `type_params` slot. Returns
/// the local scope map from parameter name to DeclarationId for field-type
/// lookups. All three generic-carrying lowering paths share this helper
/// so `type_params` is the single authority for generics.
fn allocate_type_params(
    dag: &mut Dag,
    parent_id: DeclarationId,
    type_params: &[String],
) -> HashMap<String, DeclarationId> {
    let mut local = HashMap::with_capacity(type_params.len());
    let mut ids = Vec::with_capacity(type_params.len());
    for param in type_params {
        let param_id = dag.alloc_declaration_id();
        let span = dag.declaration(parent_id).span.clone();
        dag.push_declaration(Declaration {
            id: param_id,
            name: Some(param.clone()),
            connective: TypeConnective::Atom(AtomPayload::TypeParam(param.clone())),
            type_params: Vec::new(),
            meta_tag: None,
            inhabits: None,
            span,
        });
        local.insert(param.clone(), param_id);
        ids.push(param_id);
    }
    dag.declaration_mut(parent_id).type_params = ids;
    local
}

fn lower_type_record(
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
    name: &str,
    type_params: &[String],
    fields: &[SurfaceField],
) {
    let decl_id = symbols[name];
    let local = allocate_type_params(dag, decl_id, type_params);
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
    type_params: &[String],
    variants: &[SurfaceVariant],
) {
    let decl_id = symbols[name];
    let local = allocate_type_params(dag, decl_id, type_params);
    let mut variant_fields: Vec<Field> = Vec::with_capacity(variants.len());
    for variant in variants {
        // Build payload children FIRST, then allocate the variant declaration.
        // Allocating the variant id before its payload children would wedge
        // the dense-sequential invariant on `Dag.declarations` because the
        // child declarations push into slots between the variant's reserved
        // id and its eventual push.
        let connective = if variant.payload.is_empty() {
            TypeConnective::Conj {
                children: Vec::new(),
            }
        } else {
            let children: Vec<Field> = variant
                .payload
                .iter()
                .enumerate()
                .map(|(idx, ty)| Field {
                    label: format!("_{idx}"),
                    ty: type_to_declaration_id(ty, symbols, &local, dag),
                })
                .collect();
            TypeConnective::Conj { children }
        };
        let variant_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: variant_id,
            name: Some(variant.name.clone()),
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
    type_params: &[String],
    target: &SurfaceType,
) {
    let decl_id = symbols[name];
    let local = allocate_type_params(dag, decl_id, type_params);
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

/// Build the `TemplateArgument` list for an Instantiation, fail-closed on
/// template/argument arity mismatch. When the template exposes no parameter
/// at position `idx`, we emit a declaration-level diagnostic (via a phantom
/// port) and substitute the arg's own DeclarationId as the parameter slot so
/// the resulting Declaration is structurally well-formed. The diagnostic
/// means the compile will fail at the fail-closed boundary regardless of
/// the fallback choice.
fn build_template_arguments(
    dag: &mut Dag,
    symbols: &HashMap<String, DeclarationId>,
    local: &HashMap<String, DeclarationId>,
    template: DeclarationId,
    template_name: &str,
    args: &[SurfaceType],
    span: &SourceSpan,
) -> Vec<TemplateArgument> {
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
    }
    args.iter()
        .enumerate()
        .map(|(idx, arg)| {
            let value = type_to_declaration_id(arg, symbols, local, dag);
            let parameter = match template_param_id(dag, template, idx) {
                Some(param_id) => param_id,
                None => {
                    report_declaration_error(
                        dag,
                        Diagnostic::ResolveError {
                            name: format!(
                                "template `{template_name}` has no parameter at position {idx}"
                            ),
                            span: span.clone(),
                        },
                    );
                    value
                }
            };
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
        connective: TypeConnective::Atom(AtomPayload::Identifier {
            name: name.to_string(),
            resolved: None,
        }),
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        span: span.clone(),
    });
    id
}

/// Emit a declaration-level diagnostic via a phantom port. Declaration-level
/// failures (unresolved identifiers, template metadata mismatch) do not have
/// a natural PortId anchor; we allocate a detached port purely as a
/// diagnostic carrier so the Dag's existing fail-closed biconditional
/// (port state ↔ diagnostic entry) surfaces the error through
/// `compile_to_dag` without a parallel diagnostic channel.
fn report_declaration_error(dag: &mut Dag, diag: Diagnostic) {
    let phantom = dag.alloc_port(None);
    dag.mark_unresolved(phantom, diag);
}

/// Final-pass resolution over anonymous Identifier-atom declarations. Any
/// declaration with `Atom(Identifier { name, resolved: None })` whose name
/// appears in the declaration table at sweep time gets its `resolved` slot
/// filled in-place. Anything still unresolved after the sweep emits a
/// fail-closed ResolveError diagnostic via a phantom port.
///
/// This is the post-lowering pass that closes the fail-closed gap: stubs
/// created during lowering (forward references, unknown names) either
/// resolve or surface as diagnostics by the time `lower` / `bootstrap`
/// returns. Called once at the end of `lower_into` for user modules and
/// once at the end of `bootstrap::bootstrap` for the primitive fixtures.
pub(crate) fn resolve_pending_identifiers(dag: &mut Dag) {
    let snapshot: Vec<(DeclarationId, String, SourceSpan)> = dag
        .declarations()
        .iter()
        .filter_map(|d| match &d.connective {
            TypeConnective::Atom(AtomPayload::Identifier {
                name,
                resolved: None,
            }) => Some((d.id, name.clone(), d.span.clone())),
            _ => None,
        })
        .collect();

    for (decl_id, name, span) in snapshot {
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
            if let TypeConnective::Atom(AtomPayload::Identifier { resolved, .. }) =
                &mut dag.declaration_mut(decl_id).connective
            {
                *resolved = Some(target);
            }
        } else {
            report_declaration_error(
                dag,
                Diagnostic::ResolveError {
                    name: format!("unresolved type identifier `{name}`"),
                    span,
                },
            );
        }
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

#[allow(clippy::too_many_arguments)]
fn lower_fn_item(
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
    //    names land as TypeShape::Primitive(Prim::Int) sentinel with a
    //    ResolveError, preserving M0 fail-closed behavior.
    let mut param_ports: Vec<PortId> = Vec::with_capacity(params.len());
    let mut param_types: Vec<TypeShape> = Vec::with_capacity(params.len());
    let mut body_scope: HashMap<String, PortId> = outer_scope.clone();
    let mut param_decl_inputs: Vec<DeclarationId> = Vec::with_capacity(params.len());
    let local: HashMap<String, DeclarationId> = HashMap::new();
    for param in params {
        let port = dag.alloc_port(None);
        let ty = match lower_type_for_port(&param.ty) {
            Ok(ty) => {
                dag.set_port_type(port, ty.clone());
                ty
            }
            Err(diag) => {
                dag.mark_unresolved(port, diag);
                TypeShape::Primitive(Prim::Int)
            }
        };
        body_scope.insert(param.name.clone(), port);
        param_ports.push(port);
        param_types.push(ty);
        let input_decl = type_to_declaration_id(&param.ty, symbols, &local, dag);
        param_decl_inputs.push(input_decl);
    }

    // 2. Compute return type (both port-side and declaration-side).
    let return_ty = match lower_type_for_port(return_type) {
        Ok(ty) => ty,
        Err(diag) => {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(err_port, diag);
            TypeShape::Primitive(Prim::Int)
        }
    };
    let return_decl_id = type_to_declaration_id(return_type, symbols, &local, dag);

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
            dag.set_port_type(loop_output, return_ty.clone());
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

    dag.set_port_type(value_port, return_ty.clone());
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

fn lower_type_for_port(ty: &SurfaceType) -> Result<TypeShape, Diagnostic> {
    match ty {
        SurfaceType::Named { name, span } => match name.as_str() {
            "Int" => Ok(TypeShape::Primitive(Prim::Int)),
            "Bool" => Ok(TypeShape::Primitive(Prim::Bool)),
            "String" => Ok(TypeShape::Primitive(Prim::String)),
            _ => Err(Diagnostic::ResolveError {
                name: format!("unknown type `{name}`"),
                span: span.clone(),
            }),
        },
        SurfaceType::Parameterized { span, .. }
        | SurfaceType::Optional { span, .. }
        | SurfaceType::Arrow { span, .. } => Err(Diagnostic::ResolveError {
            name: "compound type annotations are not yet supported in user code"
                .to_string(),
            span: span.clone(),
        }),
    }
}

fn lower_expr(
    expr: &SurfaceExpr,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
    symbols: &HashMap<String, DeclarationId>,
) -> PortId {
    match expr {
        SurfaceExpr::IntLit { value, span } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data: LiteralBits::Int(*value),
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::BoolLit { value, span } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data: LiteralBits::Bool(*value),
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::StringLit { value, span } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data: LiteralBits::String(value.clone()),
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
                target: target_decl,
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
        SurfaceExpr::IntLit { .. }
        | SurfaceExpr::BoolLit { .. }
        | SurfaceExpr::StringLit { .. }
        | SurfaceExpr::Var { .. } => false,
        SurfaceExpr::Call { target, args, .. } => {
            if target == self_name {
                return true;
            }
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
/// must be `first_param - <positive int>`. The surface shape is
/// `SurfaceExpr::Call { target: "-", args: [Var(first_param), IntLit(k)] }`
/// per §8.9 Option A — operators emit raw identifiers, not pre-resolved
/// path names.
fn descent_provable(expr: &SurfaceExpr, self_name: &str, first_param: &str) -> bool {
    match expr {
        SurfaceExpr::IntLit { .. }
        | SurfaceExpr::BoolLit { .. }
        | SurfaceExpr::StringLit { .. }
        | SurfaceExpr::Var { .. } => true,
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
    let SurfaceExpr::Call { target, args, .. } = expr else {
        return false;
    };
    if target != "-" || args.len() != 2 {
        return false;
    }
    let lhs_is_param = matches!(
        &args[0],
        SurfaceExpr::Var { name, .. } if name == first_param
    );
    let rhs_is_positive = matches!(
        &args[1],
        SurfaceExpr::IntLit { value, .. } if *value > 0
    );
    lhs_is_param && rhs_is_positive
}

fn expr_span(expr: &SurfaceExpr) -> &SourceSpan {
    match expr {
        SurfaceExpr::IntLit { span, .. }
        | SurfaceExpr::BoolLit { span, .. }
        | SurfaceExpr::StringLit { span, .. }
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::Call { span, .. }
        | SurfaceExpr::If { span, .. } => span,
    }
}

fn compute_mutually_recursive(items: &[SurfaceItem]) -> HashSet<String> {
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
        SurfaceExpr::IntLit { .. }
        | SurfaceExpr::BoolLit { .. }
        | SurfaceExpr::StringLit { .. }
        | SurfaceExpr::Var { .. } => {}
        SurfaceExpr::Call { target, args, .. } => {
            if fn_names.contains(target) {
                out.insert(target.clone());
            }
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
