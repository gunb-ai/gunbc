//! Resolved constructor and variant-value identity for `decl_facts` data-initializer projection.
//!
//! Identity is marshaled only from typechecked subjects in `InterpContext`. Resolution uses
//! inferred `Node` stamps and owning type-item nodes — never lossy name strings re-looked up
//! through first-pick authority helpers (`resolved_initializer_decl_ref`, etc.).

use std::rc::Rc;

use im::HashMap;

use crate::v1_compiler_infer_env::{lookup_binding_by_name_local, lookup_type};
use crate::v1_compiler_infer_items::{item_kind, ItemKind, TypedModule};
use crate::v1_compiler_infer_types::normalize_access_type_node;
use crate::v1_interpreter::{sorted_fields, str_value, InterpContext, InterpResult, Value};
use crate::v1_std_core::{
    authored_name_at, field_init_node_name_at, field_init_node_value, field_node_name_at,
    field_node_type_expr, find_child_named, inferred_to_node, Connective, ExprData, InferredNode,
    NewlineIndex, Node, VarBindingKind,
};

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

fn bare_symbol_tail(name: &str) -> &str {
    name.rsplit_once('.').map(|(_, tail)| tail).unwrap_or(name)
}

fn local_symbol_name(si: &SourceIndices, node: &Rc<Node>) -> String {
    bare_symbol_tail(&authored_name_at(si.clone(), node.clone())).to_string()
}

fn coproduct_record_lit_variant_name(si: &SourceIndices, body: &Rc<Node>) -> String {
    let from_authored = local_symbol_name(si, body);
    if !from_authored.is_empty() {
        return from_authored;
    }
    if !body.name.is_empty() {
        return body.name.clone();
    }
    String::new()
}

fn coproduct_variant_arm_by_name(
    coproduct: &Rc<Node>,
    variant_name: &str,
    si: &SourceIndices,
) -> Option<Rc<Node>> {
    if variant_name.is_empty() {
        return None;
    }
    if let Some(arm) = find_child_named(coproduct.clone(), variant_name.to_string(), si.clone()) {
        return Some(arm);
    }
    let bare = bare_symbol_tail(variant_name);
    if bare != variant_name {
        if let Some(arm) = find_child_named(coproduct.clone(), bare.to_string(), si.clone()) {
            return Some(arm);
        }
    }
    coproduct
        .children
        .iter()
        .find(|child| {
            child.name == variant_name
                || child.name == bare
                || local_symbol_name(si, child) == variant_name
                || local_symbol_name(si, child) == bare
        })
        .cloned()
}

fn coproduct_has_variant_named(
    coproduct: &Rc<Node>,
    variant_name: &str,
    si: &SourceIndices,
) -> bool {
    coproduct_variant_arm_by_name(coproduct, variant_name, si).is_some()
}

fn type_expr_bare_name(si: &SourceIndices, type_expr: &Rc<Node>) -> Option<String> {
    let ty = if type_expr.connective == Connective::Conj
        && type_expr.type_annotation.is_some()
        && type_expr.children.len() == 1
    {
        type_expr.children[0].clone()
    } else {
        type_expr.clone()
    };
    let ty = normalize_access_type_node(ty);
    if let Some(inferred) = ty
        .inferred
        .as_ref()
        .and_then(|inf| inferred_to_node(inf.clone()))
    {
        let inferred_name = authored_name_at(si.clone(), inferred.clone());
        if !inferred_name.is_empty() {
            return Some(bare_symbol_tail(&inferred_name).to_string());
        }
        if !inferred.name.is_empty() {
            return Some(bare_symbol_tail(&inferred.name).to_string());
        }
    }
    let name = authored_name_at(si.clone(), ty.clone());
    if name.is_empty() {
        if ty.name.is_empty() {
            None
        } else {
            Some(bare_symbol_tail(&ty.name).to_string())
        }
    } else {
        Some(bare_symbol_tail(&name).to_string())
    }
}

fn field_type_bare_on_variant_arm(
    arm: &Rc<Node>,
    field_name: &str,
    si: &SourceIndices,
) -> Option<String> {
    field_type_bare_on_record_fields(arm, field_name, si)
}

fn field_type_bare_on_record_fields(
    node: &Rc<Node>,
    field_name: &str,
    si: &SourceIndices,
) -> Option<String> {
    let bare_field = bare_symbol_tail(field_name);
    for child in node.children.iter() {
        let authored = authored_name_at(si.clone(), child.clone());
        let declared = field_node_name_at(child.clone(), si.clone());
        if authored != field_name
            && authored != bare_field
            && declared != field_name
            && declared != bare_field
            && child.name != field_name
            && child.name != bare_field
        {
            if child.connective == Connective::Conj {
                if let Some(bare) = field_type_bare_on_record_fields(child, field_name, si) {
                    return Some(bare);
                }
            }
            continue;
        }
        let ty = child
            .inferred
            .as_ref()
            .and_then(|inf| inferred_to_node(inf.clone()))
            .unwrap_or_else(|| field_node_type_expr(child.clone()));
        let ty = normalize_access_type_node(ty);
        if let Some(bare) = type_expr_bare_name(si, &ty) {
            return Some(bare);
        }
    }
    None
}

fn declared_field_type_bare_on_coproduct_variant(
    coproduct: &Rc<Node>,
    variant_name: &str,
    field_name: &str,
    si: &SourceIndices,
) -> Option<String> {
    let arm = coproduct_variant_arm_by_name(coproduct, variant_name, si)?;
    field_type_bare_on_variant_arm(&arm, field_name, si).or_else(|| {
        arm.inferred
            .as_ref()
            .and_then(|inf| inferred_to_node(inf.clone()))
            .and_then(|node| declared_field_type_bare_on_conj(&node, field_name, si))
    })
}

fn declared_field_type_bare_on_conj(
    type_item: &Rc<Node>,
    field_name: &str,
    si: &SourceIndices,
) -> Option<String> {
    if type_item.connective != Connective::Conj {
        return None;
    }
    field_type_bare_on_variant_arm(type_item, field_name, si)
}

fn coproduct_type_item_parse_tree(
    ctx: &InterpContext,
    bare_name: &str,
    si: &SourceIndices,
) -> Option<(Rc<Node>, String)> {
    for tm in ctx.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        for item in tm.items.iter() {
            if item_kind(item.clone()) != ItemKind::TypeItem || item.connective != Connective::Disj
            {
                continue;
            }
            if local_symbol_name(si, item) != bare_name {
                continue;
            }
            return Some((item.clone(), mod_name));
        }
    }
    None
}

fn declared_field_type_on_coproduct_variant_parse_tree(
    ctx: &InterpContext,
    coproduct_bare: &str,
    variant_name: &str,
    field_name: &str,
    si: &SourceIndices,
) -> Option<String> {
    let (coproduct, _) = coproduct_type_item_parse_tree(ctx, coproduct_bare, si)?;
    declared_field_type_bare_on_coproduct_variant(&coproduct, variant_name, field_name, si)
}

fn decl_logical_qualified_name(module_name: &str, name: &str) -> String {
    let logical = module_name.strip_prefix("v2.").unwrap_or(module_name);
    if logical.is_empty() {
        name.to_string()
    } else {
        format!("{logical}.{name}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDeclarationLocator {
    pub qualified_name: String,
    pub name: String,
    pub module_path: String,
    pub rel_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVariantLocator {
    pub parent: ResolvedDeclarationLocator,
    pub arm: ResolvedDeclarationLocator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataInitializerValueResolution {
    Resolved(ResolvedVariantLocator),
    NotVariantValue,
    Missing,
    Ambiguous(Vec<ResolvedVariantLocator>),
}

fn rel_path_from_node(node: &Rc<Node>) -> String {
    node.span.file.clone()
}

/// The declared type's AUTHORED name, qualification intact.
///
/// `declared_type_bare_name` deliberately keeps returning the tail, because variant matching
/// and field lookup are bare-name operations. Only the type-item LOOKUP needs the prefix, and
/// discarding it there is what made a cross-module declared type unresolvable.
fn declared_type_authored_name(item: &Rc<Node>, si: &SourceIndices) -> Option<String> {
    let ann = item.type_annotation.as_ref()?;
    let name = authored_name_at(si.clone(), ann.clone());
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn declared_type_bare_name(item: &Rc<Node>, si: &SourceIndices) -> Option<String> {
    let ann = item.type_annotation.as_ref()?;
    let name = authored_name_at(si.clone(), ann.clone());
    if name.is_empty() {
        None
    } else {
        Some(bare_symbol_tail(&name).to_string())
    }
}

fn typed_module_for_path(ctx: &InterpContext, module_path: &str) -> Option<Rc<TypedModule>> {
    let si = ctx.source_indices.clone();
    ctx.modules
        .iter()
        .find(|tm| authored_name_at(si.clone(), tm.module.clone()) == module_path)
        .cloned()
}

fn lookup_type_binding_in_importing_module(
    tm: &TypedModule,
    bare_name: &str,
) -> Option<Rc<crate::v1_compiler_infer_env::TypeBinding>> {
    lookup_binding_by_name_local(tm.type_env.clone(), bare_name.to_string())
}

fn type_item_from_importing_module_type_env(
    tm: &TypedModule,
    bare_name: &str,
    si: &SourceIndices,
) -> Option<Rc<Node>> {
    let binding = lookup_type_binding_in_importing_module(tm, bare_name)?;
    let node = binding.resolved.clone();
    if item_kind(node.clone()) != ItemKind::TypeItem {
        return None;
    }
    let name = authored_name_at(si.clone(), node.clone());
    if bare_symbol_tail(&name) != bare_name {
        return None;
    }
    Some(node)
}

fn coproduct_type_item_with_variant_children(
    _ctx: &InterpContext,
    tm: &TypedModule,
    bare_name: &str,
    si: &SourceIndices,
) -> Option<(Rc<Node>, String)> {
    let node = type_item_from_importing_module_type_env(tm, bare_name, si)?;
    if node.connective != Connective::Disj {
        return None;
    }
    let importing_module = authored_name_at(si.clone(), tm.module.clone());
    Some((node, importing_module))
}

fn module_path_for_type_decl_node(
    ctx: &InterpContext,
    type_item: &Rc<Node>,
    si: &SourceIndices,
) -> Option<String> {
    let file = type_item.span.file.as_str();
    let type_name = authored_name_at(si.clone(), type_item.clone());
    for tm in ctx.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        for item in tm.items.iter() {
            if item_kind(Rc::clone(item)) != ItemKind::TypeItem {
                continue;
            }
            if item.span.file.as_str() != file {
                continue;
            }
            if authored_name_at(si.clone(), Rc::clone(item)) == type_name {
                return Some(mod_name);
            }
        }
    }
    None
}

fn declaration_identity_for_type_item(
    ctx: &InterpContext,
    type_item: &Rc<Node>,
    si: &SourceIndices,
    fallback_module_path: &str,
) -> ResolvedDeclarationLocator {
    let module_path = module_path_for_type_decl_node(ctx, type_item, si)
        .unwrap_or_else(|| fallback_module_path.to_string());
    let name = authored_name_at(si.clone(), type_item.clone());
    ResolvedDeclarationLocator {
        qualified_name: decl_logical_qualified_name(&module_path, &name),
        name,
        module_path,
        rel_path: rel_path_from_node(type_item),
    }
}

fn declaration_identity_for_variant_arm(
    ctx: &InterpContext,
    coproduct: &Rc<Node>,
    variant_name: &str,
    si: &SourceIndices,
    fallback_module_path: &str,
) -> Option<ResolvedVariantLocator> {
    let arm = coproduct_variant_arm_by_name(coproduct, variant_name, si)?;
    let parent = declaration_identity_for_type_item(ctx, coproduct, si, fallback_module_path);
    let arm_name = {
        let authored = authored_name_at(si.clone(), arm.clone());
        if authored.is_empty() {
            variant_name.to_string()
        } else {
            bare_symbol_tail(&authored).to_string()
        }
    };
    let arm_id = ResolvedDeclarationLocator {
        qualified_name: format!("{}.{}", parent.qualified_name, arm_name),
        name: arm_name,
        module_path: parent.module_path.clone(),
        rel_path: parent.rel_path.clone(),
    };
    Some(ResolvedVariantLocator {
        parent,
        arm: arm_id,
    })
}

fn expr_var_binding_kind(expr: &Rc<Node>) -> Option<Rc<VarBindingKind>> {
    match expr.expr_data.as_ref() {
        ExprData::ExprVar { binding_kind, .. } => binding_kind.clone(),
        _ => None,
    }
}

fn is_variant_value_binding(expr: &Rc<Node>) -> bool {
    matches!(
        expr_var_binding_kind(expr).as_deref(),
        Some(VarBindingKind::VariantValueBinding { .. })
    )
}

fn variant_value_binding_parent_enum(expr: &Rc<Node>) -> Option<String> {
    match expr_var_binding_kind(expr).as_deref() {
        Some(VarBindingKind::VariantValueBinding { parent_enum }) => Some(parent_enum.clone()),
        _ => None,
    }
}

/// Resolve a QUALIFIED parent-type name through the module its prefix names.
///
/// A qualified reference names its declaring module directly, so this is the constructor
/// path's replacement for what an import list used to do: the referenced type never enters
/// the referencing module's type env, and a bare lookup there cannot find it.
///
/// The defining module returned is the DECLARING one, not the referencing one -- that is the
/// whole point, since the constructor's parent identity is a fact about where the type is
/// declared.
fn coproduct_through_qualified_prefix(
    ctx: &InterpContext,
    parent_enum: &str,
    si: &SourceIndices,
) -> Option<(Rc<Node>, String)> {
    let (prefix, bare) = parent_enum.rsplit_once('.')?;
    let declaring = typed_module_for_path(ctx, prefix)?;
    let node = type_item_from_importing_module_type_env(&declaring, bare, si)?;
    if node.connective != Connective::Disj {
        return None;
    }
    Some((node, prefix.to_string()))
}

fn coproduct_from_variant_value_binding(
    ctx: &InterpContext,
    tm: &TypedModule,
    parent_enum: &str,
    si: &SourceIndices,
) -> Option<(Rc<Node>, String)> {
    let bare = bare_symbol_tail(parent_enum);
    // The local type env first: it still carries the module's OWN declarations, which is the
    // local-constructor case and stays exactly as it was.
    if let Some(node) = type_item_from_importing_module_type_env(tm, bare, si) {
        if node.connective == Connective::Disj {
            let importing_module = authored_name_at(si.clone(), tm.module.clone());
            return Some((node, importing_module));
        }
    }
    // Then the qualified prefix. Without this arm a cross-module constructor written as
    // `other.module.Variant { .. }` compiles clean and produces a graph, but its parent type
    // silently fails to resolve -- so the projection reports ConstructorResolutionRefused
    // while nothing in the compile refuses. That is the fail-open the namespace cut would
    // otherwise introduce corpus-wide, since every cross-module constructor is now written
    // in exactly this form.
    coproduct_through_qualified_prefix(ctx, parent_enum, si)
}

fn variant_value_from_typechecked_expr(
    ctx: &InterpContext,
    importing_module: &str,
    expr: &Rc<Node>,
    si: &SourceIndices,
    declared_type_bare: Option<&str>,
) -> DataInitializerValueResolution {
    if matches!(
        expr.inferred.as_deref(),
        Some(InferredNode::CompilerError { .. })
    ) {
        return DataInitializerValueResolution::Missing;
    }

    if !matches!(expr.expr_data.as_ref(), ExprData::ExprVar { .. }) {
        return DataInitializerValueResolution::NotVariantValue;
    }

    if !is_variant_value_binding(expr) {
        return DataInitializerValueResolution::NotVariantValue;
    }

    let inferred_node = match expr.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => node.clone(),
        _ => return DataInitializerValueResolution::NotVariantValue,
    };

    let variant_name = {
        let from_inferred = local_symbol_name(si, &inferred_node);
        if !from_inferred.is_empty() {
            from_inferred
        } else {
            authored_name_at(si.clone(), inferred_node.clone())
        }
    };
    if variant_name.is_empty() {
        return DataInitializerValueResolution::NotVariantValue;
    }

    let expr_variant_name = {
        let bare = crate::v1_std_core::expr_var_name_at(expr.clone(), si.clone());
        if bare.is_empty() {
            variant_name.clone()
        } else {
            bare
        }
    };

    let tm = match typed_module_for_path(ctx, importing_module) {
        Some(tm) => tm,
        None => return DataInitializerValueResolution::Missing,
    };

    let parent_enum = match variant_value_binding_parent_enum(expr) {
        Some(parent_enum) => parent_enum,
        None => return DataInitializerValueResolution::NotVariantValue,
    };

    let (coproduct, defining_module) =
        match coproduct_from_variant_value_binding(ctx, &tm, &parent_enum, si) {
            Some(found) => found,
            None => {
                if let Some(bare) = declared_type_bare.filter(|bare| !bare.is_empty()) {
                    match coproduct_type_item_with_variant_children(ctx, &tm, bare, si) {
                        Some(found) => found,
                        None => return DataInitializerValueResolution::Missing,
                    }
                } else {
                    return DataInitializerValueResolution::Missing;
                }
            }
        };

    if !coproduct_has_variant_named(&coproduct, &expr_variant_name, si)
        && !coproduct_has_variant_named(&coproduct, &variant_name, si)
    {
        return DataInitializerValueResolution::Missing;
    }

    let resolved_variant_name = if coproduct_has_variant_named(&coproduct, &expr_variant_name, si) {
        expr_variant_name
    } else {
        variant_name
    };

    match declaration_identity_for_variant_arm(
        ctx,
        &coproduct,
        &resolved_variant_name,
        si,
        &defining_module,
    ) {
        Some(variant_id) => DataInitializerValueResolution::Resolved(variant_id),
        None => DataInitializerValueResolution::Missing,
    }
}

fn constructor_resolution_refused_projection(ctx: &InterpContext) -> Value {
    projection_node_with_named_edges(
        ctx,
        "DataInitializerConstructorResolutionRefusedProjection",
        &[],
    )
}

fn projection_atom_identity_node(ctx: &InterpContext, identity: &str) -> Value {
    Value::Record {
        type_name: ctx.sym("Node"),
        fields: Rc::new(sorted_fields(vec![
            (
                ctx.sym("kind"),
                Value::Variant {
                    type_name: ctx.sym("NodeKind"),
                    variant_name: ctx.sym("TypeNode"),
                    fields: Rc::new(vec![(
                        ctx.sym("connective"),
                        Value::Variant {
                            type_name: ctx.sym("Connective"),
                            variant_name: ctx.sym("Atom"),
                            fields: Rc::new(vec![(
                                ctx.sym("identity"),
                                str_value(identity.to_string()),
                            )]),
                        },
                    )]),
                },
            ),
            (
                ctx.sym("children"),
                crate::v1_interpreter::list_value(vec![]),
            ),
            (
                ctx.sym("occurrence_id"),
                Value::Variant {
                    type_name: ctx.sym("NodeOccurrenceId"),
                    variant_name: ctx.sym("SyntheticOccurrence"),
                    fields: Rc::new(vec![]),
                },
            ),
        ])),
    }
}

fn projection_edge_named(ctx: &InterpContext, name: &str, target: Value) -> Value {
    Value::Record {
        type_name: ctx.sym("Edge"),
        fields: Rc::new(sorted_fields(vec![
            (
                ctx.sym("label"),
                Value::Variant {
                    type_name: ctx.sym("EdgeLabel"),
                    variant_name: ctx.sym("Named"),
                    fields: Rc::new(vec![(ctx.sym("name"), str_value(name.to_string()))]),
                },
            ),
            (ctx.sym("target"), target),
        ])),
    }
}

fn projection_node_record(ctx: &InterpContext, projection_kind: &str, edges: Vec<Value>) -> Value {
    Value::Record {
        type_name: ctx.sym("Node"),
        fields: Rc::new(sorted_fields(vec![
            (
                ctx.sym("kind"),
                Value::Variant {
                    type_name: ctx.sym("NodeKind"),
                    variant_name: ctx.sym("TypeNode"),
                    fields: Rc::new(vec![(
                        ctx.sym("connective"),
                        Value::Variant {
                            type_name: ctx.sym("Connective"),
                            variant_name: ctx.sym("Atom"),
                            fields: Rc::new(vec![(
                                ctx.sym("identity"),
                                str_value(projection_kind.to_string()),
                            )]),
                        },
                    )]),
                },
            ),
            (
                ctx.sym("children"),
                crate::v1_interpreter::list_value(edges),
            ),
            (
                ctx.sym("occurrence_id"),
                Value::Variant {
                    type_name: ctx.sym("NodeOccurrenceId"),
                    variant_name: ctx.sym("SyntheticOccurrence"),
                    fields: Rc::new(vec![]),
                },
            ),
        ])),
    }
}

fn projection_node_with_named_edges(
    ctx: &InterpContext,
    projection_kind: &str,
    edges: &[(String, Value)],
) -> Value {
    let mut children = Vec::with_capacity(edges.len());
    for (name, target) in edges {
        children.push(projection_edge_named(ctx, name, target.clone()));
    }
    projection_node_record(ctx, projection_kind, children)
}

fn marshal_declaration_identity_node(
    ctx: &InterpContext,
    id: &ResolvedDeclarationLocator,
) -> Value {
    projection_node_with_named_edges(
        ctx,
        "DeclarationIdentityProjection",
        &[
            (
                "qualified_name".to_string(),
                projection_atom_identity_node(ctx, &id.qualified_name),
            ),
            (
                "name".to_string(),
                projection_atom_identity_node(ctx, &id.name),
            ),
            (
                "module_path".to_string(),
                projection_atom_identity_node(ctx, &id.module_path),
            ),
            (
                "rel_path".to_string(),
                projection_atom_identity_node(ctx, &id.rel_path),
            ),
        ],
    )
}

fn marshal_variant_identity_node(ctx: &InterpContext, id: &ResolvedVariantLocator) -> Value {
    projection_node_with_named_edges(
        ctx,
        "VariantDeclarationIdentityProjection",
        &[
            (
                "parent_qualified_name".to_string(),
                projection_atom_identity_node(ctx, &id.parent.qualified_name),
            ),
            (
                "variant_name".to_string(),
                projection_atom_identity_node(ctx, &id.arm.name),
            ),
            (
                "parent_type".to_string(),
                marshal_declaration_identity_node(ctx, &id.parent),
            ),
            (
                "arm".to_string(),
                marshal_declaration_identity_node(ctx, &id.arm),
            ),
        ],
    )
}

fn marshal_constructor_identity_node(
    ctx: &InterpContext,
    parent: &ResolvedDeclarationLocator,
    variant: &ResolvedVariantLocator,
) -> Value {
    projection_node_with_named_edges(
        ctx,
        "DataInitializerConstructorIdentityProjection",
        &[
            (
                "parent_type".to_string(),
                marshal_declaration_identity_node(ctx, parent),
            ),
            (
                "constructor".to_string(),
                marshal_variant_identity_node(ctx, variant),
            ),
        ],
    )
}

fn marshal_value_identity_node(
    ctx: &InterpContext,
    resolution: &DataInitializerValueResolution,
) -> Value {
    match resolution {
        DataInitializerValueResolution::Resolved(v) => projection_node_with_named_edges(
            ctx,
            "ResolvedVariantValueProjection",
            &[
                (
                    "parent_qualified_name".to_string(),
                    projection_atom_identity_node(ctx, &v.parent.qualified_name),
                ),
                (
                    "variant_name".to_string(),
                    projection_atom_identity_node(ctx, &v.arm.name),
                ),
                (
                    "parent_type".to_string(),
                    marshal_declaration_identity_node(ctx, &v.parent),
                ),
                (
                    "arm".to_string(),
                    marshal_declaration_identity_node(ctx, &v.arm),
                ),
            ],
        ),
        DataInitializerValueResolution::NotVariantValue => {
            projection_node_with_named_edges(ctx, "NotVariantValueProjection", &[])
        }
        DataInitializerValueResolution::Missing => {
            projection_node_with_named_edges(ctx, "VariantValueResolutionMissingProjection", &[])
        }
        DataInitializerValueResolution::Ambiguous(cands) => {
            let mut edges: Vec<(String, Value)> = Vec::new();
            for (i, c) in cands.iter().enumerate() {
                edges.push((
                    format!("candidate_{i}"),
                    marshal_variant_identity_node(ctx, c),
                ));
            }
            projection_node_with_named_edges(
                ctx,
                "VariantValueResolutionAmbiguousProjection",
                &edges,
            )
        }
    }
}

fn inferred_field_type_bare(si: &SourceIndices, field_init: &Rc<Node>) -> Option<String> {
    match field_init.inferred.as_deref() {
        Some(InferredNode::Resolved { node: ty, .. }) => {
            let name = authored_name_at(si.clone(), normalize_access_type_node(ty.clone()));
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        }
        _ => None,
    }
}

/// Resolve a declared type to its item and its DECLARING module.
///
/// The referencing module's type env is tried first, which is where a module's own
/// declarations live and where an imported name used to be bound. When that fails, the
/// authored name's qualified prefix names the declaring module directly -- which is what
/// replaces the import, and without it every cross-module declared type is unresolvable.
///
/// The module returned is the one that DECLARES the type, because it is what the parent
/// identity is built from: attributing a cross-module type to the referencing module would
/// produce a confidently wrong qualified name rather than a refusal.
fn type_item_and_home(
    ctx: &InterpContext,
    tm: &TypedModule,
    declared_type_bare: &str,
    declared_type_authored: &str,
    si: &SourceIndices,
) -> Option<(Rc<Node>, String)> {
    if let Some(node) = type_item_from_importing_module_type_env(tm, declared_type_bare, si) {
        return Some((node, authored_name_at(si.clone(), tm.module.clone())));
    }

    // The referencing module does not carry the type, so the reference is cross-module.
    //
    // The prefix is NOT recoverable from the annotation's authored name: measured, a
    // `test.decl_facts_order.carrier.Disposition` annotation reports its authored name as
    // plain `Disposition`, because qualification lives in the annotation node's structure
    // rather than its name. Two earlier repairs failed on exactly that -- they reconstructed
    // an owner from a string that never carried one.
    //
    // So resolve by DECLARATION IDENTITY instead: find the module that actually declares this
    // type. A unique declarer is the answer; more than one is genuinely ambiguous at this
    // grain and REFUSES, because picking the first would resolve a cross-module parent to
    // whichever module happened to be walked first -- a confidently wrong qualified name in
    // place of a refusal, which is the failure mode this whole path exists to avoid.
    let mut found: Option<(Rc<Node>, String)> = None;
    for candidate in ctx.modules.iter() {
        let Some(node) =
            type_item_from_importing_module_type_env(candidate, declared_type_bare, si)
        else {
            continue;
        };
        let module_path = authored_name_at(si.clone(), candidate.module.clone());
        match &found {
            // Same declaration reachable through more than one module's env is not a second
            // declarer; only a DIFFERENT owning module makes it ambiguous.
            Some((seen, seen_module)) if Rc::ptr_eq(seen, &node) || *seen_module == module_path => {
            }
            Some(_) => return None,
            None => found = Some((node, module_path)),
        }
    }
    let _ = declared_type_authored;
    found
}

fn marshal_record_literal_projection(
    ctx: &InterpContext,
    importing_module: &str,
    body: &Rc<Node>,
    declared_type_bare: &str,
    declared_type_authored: &str,
    si: &SourceIndices,
    tm: &TypedModule,
) -> Value {
    let (type_item, type_home_module) =
        match type_item_and_home(ctx, tm, declared_type_bare, declared_type_authored, si) {
            Some(found) => found,
            None => return constructor_resolution_refused_projection(ctx),
        };
    // Classification uses the importing module type env + resolved connective only.
    // A failed lookup must refuse — never fall through to the coproduct arm.
    if type_item.connective == Connective::Disj {
        marshal_coproduct_record_projection(
            ctx,
            importing_module,
            &type_home_module,
            body,
            declared_type_bare,
            si,
            type_item,
        )
    } else {
        marshal_plain_record_projection(
            ctx,
            importing_module,
            &type_home_module,
            body,
            declared_type_bare,
            si,
            type_item,
        )
    }
}

fn marshal_field_initializer_projection(
    ctx: &InterpContext,
    importing_module: &str,
    field_value: &Rc<Node>,
    si: &SourceIndices,
    declared_type_override: Option<&str>,
) -> Value {
    if matches!(
        field_value.inferred.as_deref(),
        Some(InferredNode::CompilerError { .. })
    ) {
        return marshal_value_identity_node(ctx, &DataInitializerValueResolution::Missing);
    }

    match field_value.expr_data.as_ref() {
        ExprData::ExprRecordLit { .. } => {
            if let Some(override_bare) = declared_type_override.filter(|bare| !bare.is_empty()) {
                let tm = typed_module_for_path(ctx, importing_module);
                if let Some(tm) = tm {
                    let coproduct =
                        coproduct_type_item_with_variant_children(ctx, &tm, override_bare, si)
                            .map(|(node, _)| node);
                    if let Some(coproduct) =
                        coproduct.filter(|node| node.connective == Connective::Disj)
                    {
                        let variant_name = coproduct_record_lit_variant_name(si, field_value);
                        if !variant_name.is_empty()
                            && !coproduct_has_variant_named(&coproduct, &variant_name, si)
                        {
                            return marshal_value_identity_node(
                                ctx,
                                &DataInitializerValueResolution::Missing,
                            );
                        }
                    }
                }
            }
            let type_bare = declared_type_override
                .filter(|bare| !bare.is_empty())
                .map(|bare| bare.to_string())
                .or(inferred_field_type_bare(si, field_value))
                .unwrap_or_default();
            if type_bare.is_empty() {
                return marshal_value_identity_node(
                    ctx,
                    &DataInitializerValueResolution::NotVariantValue,
                );
            }
            let tm = match typed_module_for_path(ctx, importing_module) {
                Some(tm) => tm,
                None => return constructor_resolution_refused_projection(ctx),
            };
            marshal_record_literal_projection(
                ctx,
                importing_module,
                field_value,
                &type_bare,
                &type_bare,
                si,
                &tm,
            )
        }
        _ => {
            let type_bare = inferred_field_type_bare(si, field_value);
            let resolution = variant_value_from_typechecked_expr(
                ctx,
                importing_module,
                field_value,
                si,
                type_bare.as_deref(),
            );
            marshal_value_identity_node(ctx, &resolution)
        }
    }
}

fn marshal_plain_record_projection(
    ctx: &InterpContext,
    importing_module: &str,
    type_home_module: &str,
    body: &Rc<Node>,
    type_bare: &str,
    si: &SourceIndices,
    type_item: Rc<Node>,
) -> Value {
    if type_item.connective == Connective::Disj {
        return constructor_resolution_refused_projection(ctx);
    }
    let parent_id = declaration_identity_for_type_item(ctx, &type_item, si, type_home_module);

    let mut edges: Vec<(String, Value)> = vec![(
        "parent_type".to_string(),
        marshal_declaration_identity_node(ctx, &parent_id),
    )];

    for child in body.children.iter() {
        let field_name = field_init_node_name_at(child.clone(), si.clone());
        if field_name.is_empty() {
            continue;
        }
        let field_value = field_init_node_value(child.clone());
        let declared_field = declared_field_type_bare_on_conj(&type_item, &field_name, si);
        let field_projection = marshal_field_initializer_projection(
            ctx,
            importing_module,
            &field_value,
            si,
            declared_field.as_deref(),
        );
        edges.push((field_name.clone(), field_projection));
    }

    projection_node_with_named_edges(ctx, "DataInitializerPlainRecordProjection", &edges)
}

fn marshal_coproduct_record_projection(
    ctx: &InterpContext,
    importing_module: &str,
    type_home_module: &str,
    body: &Rc<Node>,
    declared_type_bare: &str,
    si: &SourceIndices,
    coproduct_item: Rc<Node>,
) -> Value {
    if coproduct_item.connective != Connective::Disj {
        return constructor_resolution_refused_projection(ctx);
    }
    let variant_name = coproduct_record_lit_variant_name(si, body);
    if variant_name.is_empty() || !coproduct_has_variant_named(&coproduct_item, &variant_name, si) {
        return marshal_value_identity_node(ctx, &DataInitializerValueResolution::Missing);
    }

    let parent_id = declaration_identity_for_type_item(ctx, &coproduct_item, si, type_home_module);
    let variant_id = match declaration_identity_for_variant_arm(
        ctx,
        &coproduct_item,
        &variant_name,
        si,
        importing_module,
    ) {
        Some(variant_id) => variant_id,
        None => ResolvedVariantLocator {
            parent: parent_id.clone(),
            arm: ResolvedDeclarationLocator {
                qualified_name: format!("{}.{}", parent_id.qualified_name, variant_name),
                name: variant_name.clone(),
                module_path: parent_id.module_path.clone(),
                rel_path: parent_id.rel_path.clone(),
            },
        },
    };

    let mut edges: Vec<(String, Value)> = vec![(
        "constructor_identity".to_string(),
        marshal_constructor_identity_node(ctx, &parent_id, &variant_id),
    )];

    for child in body.children.iter() {
        let field_name = field_init_node_name_at(child.clone(), si.clone());
        if field_name.is_empty() {
            continue;
        }
        let field_value = field_init_node_value(child.clone());
        let declared_field = declared_field_type_on_coproduct_variant_parse_tree(
            ctx,
            declared_type_bare,
            &variant_name,
            &field_name,
            si,
        )
        .or_else(|| {
            declared_field_type_bare_on_coproduct_variant(
                &coproduct_item,
                &variant_name,
                &field_name,
                si,
            )
        })
        .or_else(|| {
            body.inferred
                .as_ref()
                .and_then(|inf| inferred_to_node(inf.clone()))
                .and_then(|node| declared_field_type_bare_on_conj(&node, &field_name, si))
        });
        let field_projection = marshal_field_initializer_projection(
            ctx,
            importing_module,
            &field_value,
            si,
            declared_field.as_deref(),
        );
        edges.push((field_name.clone(), field_projection));
    }

    projection_node_with_named_edges(ctx, "DataInitializerRecordProjection", &edges)
}

pub fn marshal_data_initializer_projection_for_item(
    ctx: &InterpContext,
    item: Rc<Node>,
    qualified_name: &str,
) -> InterpResult<Value> {
    let si = ctx.source_indices.clone();
    let importing_module = {
        let decl_name = authored_name_at(si.clone(), item.clone());
        let suffix = format!(".{decl_name}");
        if qualified_name.ends_with(&suffix) {
            qualified_name[..qualified_name.len() - suffix.len()].to_string()
        } else {
            qualified_name.to_string()
        }
    };

    let body = match item.body.as_ref() {
        Some(body) => body,
        None => return Ok(constructor_resolution_refused_projection(ctx)),
    };

    let declared_type_bare = match declared_type_bare_name(&item, &si) {
        Some(name) => name,
        None => return Ok(constructor_resolution_refused_projection(ctx)),
    };
    let declared_type_authored =
        declared_type_authored_name(&item, &si).unwrap_or_else(|| declared_type_bare.clone());

    let tm = match typed_module_for_path(ctx, &importing_module) {
        Some(tm) => tm,
        None => return Ok(constructor_resolution_refused_projection(ctx)),
    };

    match body.expr_data.as_ref() {
        ExprData::ExprRecordLit { .. } => Ok(marshal_record_literal_projection(
            ctx,
            &importing_module,
            body,
            &declared_type_bare,
            &declared_type_authored,
            &si,
            &tm,
        )),
        ExprData::ExprVar { .. } => {
            if lookup_type_binding_in_importing_module(&tm, &declared_type_bare).is_none() {
                return Ok(constructor_resolution_refused_projection(ctx));
            }
            let resolution = variant_value_from_typechecked_expr(
                ctx,
                &importing_module,
                body,
                &si,
                Some(&declared_type_bare),
            );
            let mut edges: Vec<(String, Value)> = vec![(
                "value_identity".to_string(),
                marshal_value_identity_node(ctx, &resolution),
            )];
            if let DataInitializerValueResolution::Resolved(v) = &resolution {
                if let Some(type_item) =
                    type_item_from_importing_module_type_env(&tm, &declared_type_bare, &si)
                {
                    let parent_id =
                        declaration_identity_for_type_item(ctx, &type_item, &si, &importing_module);
                    edges.push((
                        "constructor_identity".to_string(),
                        marshal_constructor_identity_node(ctx, &parent_id, v),
                    ));
                }
            }
            Ok(projection_node_with_named_edges(
                ctx,
                "DataInitializerNullaryProjection",
                &edges,
            ))
        }
        _ => Ok(marshal_value_identity_node(
            ctx,
            &DataInitializerValueResolution::NotVariantValue,
        )),
    }
}

pub fn marshal_data_initializer_projection(
    ctx: &InterpContext,
    qualified_name: &str,
) -> InterpResult<Value> {
    match ctx.lookup_fn_node(qualified_name) {
        Some(item) => marshal_data_initializer_projection_for_item(ctx, item, qualified_name),
        None => {
            let bare = bare_symbol_tail(qualified_name);
            if ctx
                .item_registry
                .get(bare)
                .is_some_and(|info| info.kind == ItemKind::DataItem)
            {
                Ok(constructor_resolution_refused_projection(ctx))
            } else {
                Ok(typechecked_subject_absent_projection(ctx))
            }
        }
    }
}

pub fn typechecked_subject_absent_projection(ctx: &InterpContext) -> Value {
    projection_node_with_named_edges(
        ctx,
        "DataInitializerTypecheckedSubjectAbsentProjection",
        &[],
    )
}

#[cfg(test)]
mod projection_marshal_tests {
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_interpreter::{str_value, ExecutionMode, InterpContext, Value};

    use super::{marshal_data_initializer_projection, typechecked_subject_absent_projection};

    fn empty_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
    }

    fn projection_kind_lexeme(ctx: &InterpContext, projection: &Value) -> Option<String> {
        match projection {
            Value::Record { fields, .. } => {
                let kind_key = ctx.sym("kind");
                let connective_key = ctx.sym("connective");
                let identity_key = ctx.sym("identity");
                let kind = fields
                    .iter()
                    .find(|(k, _)| *k == kind_key)
                    .map(|(_, v)| v)?;
                match kind {
                    Value::Variant {
                        fields: kind_fields,
                        ..
                    } => {
                        let connective = kind_fields
                            .iter()
                            .find(|(k, _)| *k == connective_key)
                            .map(|(_, v)| v)?;
                        match connective {
                            Value::Variant {
                                fields: conn_fields,
                                ..
                            } => conn_fields
                                .iter()
                                .find(|(k, _)| *k == identity_key)
                                .and_then(|(_, v)| match v {
                                    Value::Str(s) => Some(s.to_string()),
                                    _ => None,
                                }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    #[test]
    fn missing_typechecked_subject_marshals_absent_projection_not_error() {
        let ctx = empty_ctx();
        let projection = marshal_data_initializer_projection(
            &ctx,
            "extdeps.units.dimensionless.extdeps_external_authority_anchor",
        )
        .expect("unresolvable marshal subject must refuse with projection, never throw");
        let absent = typechecked_subject_absent_projection(&ctx);
        assert_eq!(
            projection_kind_lexeme(&ctx, &projection),
            projection_kind_lexeme(&ctx, &absent),
            "missing typechecked subject must marshal TypecheckedSubjectAbsentProjection"
        );
    }
}
