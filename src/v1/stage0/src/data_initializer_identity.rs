//! Resolved constructor and variant-value identity for `decl_facts` data-initializer projection.
//!
//! Identity is marshaled only from typechecked subjects in `InterpContext`. Resolution uses
//! inferred `Node` stamps and owning type-item nodes — never lossy name strings re-looked up
//! through first-pick authority helpers (`resolved_initializer_decl_ref`, etc.).

use std::rc::Rc;

use im::HashMap;

use crate::v1_compiler_infer_emit_info::EmitGraphInfo;
use crate::v1_compiler_infer_env::{lookup_binding_by_name, lookup_type};
use crate::v1_compiler_infer_items::{item_kind, ItemKind, TypedModule};
use crate::v1_compiler_infer_types::normalize_access_type_node;
use crate::v1_interpreter::{sorted_fields, InterpContext, InterpError, InterpResult, Value};
use crate::v1_std_core::{
    authored_name_at, field_init_node_name_at, field_init_node_value, Connective, ExprData,
    InferredNode, NewlineIndex, Node,
};

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

fn decl_logical_qualified_name(module_name: &str, name: &str) -> String {
    let logical = module_name.strip_prefix("v2.").unwrap_or(module_name);
    if logical.is_empty() {
        name.to_string()
    } else {
        format!("{logical}.{name}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDeclarationIdentity {
    pub qualified_name: String,
    pub name: String,
    pub module_path: String,
    pub rel_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVariantIdentity {
    pub parent_qualified_name: String,
    pub variant_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataInitializerValueResolution {
    Resolved(ResolvedVariantIdentity),
    NotVariantValue,
    Missing,
    Ambiguous(Vec<ResolvedVariantIdentity>),
}

fn rel_path_from_node(node: &Rc<Node>) -> String {
    node.span.file.clone()
}

fn declared_type_bare_name(item: &Rc<Node>, si: &SourceIndices) -> Option<String> {
    let ann = item.type_annotation.as_ref()?;
    let name = authored_name_at(si.clone(), ann.clone());
    if name.is_empty() {
        None
    } else {
        Some(name)
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
    lookup_binding_by_name(tm.type_env.clone(), bare_name.to_string())
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
    if name != bare_name {
        return None;
    }
    Some(node)
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
) -> ResolvedDeclarationIdentity {
    let module_path = module_path_for_type_decl_node(ctx, type_item, si)
        .unwrap_or_else(|| fallback_module_path.to_string());
    let name = authored_name_at(si.clone(), type_item.clone());
    ResolvedDeclarationIdentity {
        qualified_name: decl_logical_qualified_name(&module_path, &name),
        name,
        module_path,
        rel_path: rel_path_from_node(type_item),
    }
}

fn cross_module_coproduct_variant_candidates(
    ctx: &InterpContext,
    variant_name: &str,
    si: &SourceIndices,
) -> Vec<ResolvedVariantIdentity> {
    let mut cands = Vec::new();
    for tm in ctx.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        for item in tm.items.iter() {
            if item.connective != Connective::Disj {
                continue;
            }
            if !crate::v1_std_core::has_child_named(
                item.clone(),
                variant_name.to_string(),
                si.clone(),
            ) {
                continue;
            }
            let parent_name = authored_name_at(si.clone(), item.clone());
            cands.push(ResolvedVariantIdentity {
                parent_qualified_name: decl_logical_qualified_name(&mod_name, &parent_name),
                variant_name: variant_name.to_string(),
            });
        }
    }
    cands
}

fn duplicate_bare_type_name_variant_candidates(
    ctx: &InterpContext,
    variant_name: &str,
    si: &SourceIndices,
) -> Vec<ResolvedVariantIdentity> {
    let mut bare_to_entries: HashMap<String, Vec<(String, Rc<Node>)>> = HashMap::new();
    for tm in ctx.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        for item in tm.items.iter() {
            if item_kind(Rc::clone(item)) != ItemKind::TypeItem
                || item.connective != Connective::Disj
            {
                continue;
            }
            let bare = authored_name_at(si.clone(), item.clone());
            bare_to_entries
                .entry(bare)
                .or_default()
                .push((mod_name.clone(), Rc::clone(item)));
        }
    }

    let mut cands = Vec::new();
    for (bare, entries) in bare_to_entries {
        if entries.len() < 2 {
            continue;
        }
        for (mod_name, item) in entries {
            if !crate::v1_std_core::has_child_named(
                item.clone(),
                variant_name.to_string(),
                si.clone(),
            ) {
                continue;
            }
            cands.push(ResolvedVariantIdentity {
                parent_qualified_name: decl_logical_qualified_name(&mod_name, &bare),
                variant_name: variant_name.to_string(),
            });
        }
    }
    cands
}

fn ambiguous_variant_candidates(
    ctx: &InterpContext,
    variant_name: &str,
    si: &SourceIndices,
) -> Vec<ResolvedVariantIdentity> {
    let mut cands =
        if variant_to_enum_is_ambiguous_sentinel(ctx.emit_graph_info.as_ref(), variant_name) {
            cross_module_coproduct_variant_candidates(ctx, variant_name, si)
        } else {
            Vec::new()
        };
    if cands.len() < 2 {
        let dup = duplicate_bare_type_name_variant_candidates(ctx, variant_name, si);
        if dup.len() >= 2 {
            cands = dup;
        }
    }
    cands
}

fn variant_to_enum_is_ambiguous_sentinel(emit_info: &EmitGraphInfo, variant_name: &str) -> bool {
    emit_info
        .variant_to_enum
        .get(variant_name)
        .is_some_and(|parent| parent.is_empty())
}

fn variant_value_from_typechecked_expr(
    ctx: &InterpContext,
    importing_module: &str,
    expr: &Rc<Node>,
    si: &SourceIndices,
) -> DataInitializerValueResolution {
    if matches!(
        expr.inferred.as_deref(),
        Some(InferredNode::CompilerError { .. })
    ) {
        return DataInitializerValueResolution::Missing;
    }

    let inferred_node = match expr.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => node.clone(),
        _ => return DataInitializerValueResolution::NotVariantValue,
    };

    if !matches!(expr.expr_data.as_ref(), ExprData::ExprVar { .. }) {
        return DataInitializerValueResolution::NotVariantValue;
    }

    let variant_name = authored_name_at(si.clone(), inferred_node.clone());
    if variant_name.is_empty() {
        return DataInitializerValueResolution::NotVariantValue;
    }

    let ambiguous_cands = ambiguous_variant_candidates(ctx, &variant_name, si);
    if ambiguous_cands.len() >= 2 {
        return DataInitializerValueResolution::Ambiguous(ambiguous_cands);
    }

    let tm = match typed_module_for_path(ctx, importing_module) {
        Some(tm) => tm,
        None => return DataInitializerValueResolution::Missing,
    };

    let coproduct = match inferred_node.ident.as_ref() {
        Some(id) => lookup_type(tm.type_env.clone(), id.clone()),
        None => None,
    };
    let coproduct = match coproduct {
        Some(node) => node,
        None => return DataInitializerValueResolution::NotVariantValue,
    };

    if coproduct.connective != Connective::Disj {
        return DataInitializerValueResolution::NotVariantValue;
    }

    let parent_id =
        declaration_identity_for_type_item(ctx, &coproduct, si, importing_module);
    DataInitializerValueResolution::Resolved(ResolvedVariantIdentity {
        parent_qualified_name: parent_id.qualified_name,
        variant_name,
    })
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
                                Value::Str(identity.to_string()),
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
                    fields: Rc::new(vec![(ctx.sym("name"), Value::Str(name.to_string()))]),
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
                                Value::Str(projection_kind.to_string()),
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
    id: &ResolvedDeclarationIdentity,
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

fn marshal_variant_identity_node(ctx: &InterpContext, id: &ResolvedVariantIdentity) -> Value {
    projection_node_with_named_edges(
        ctx,
        "VariantDeclarationIdentityProjection",
        &[
            (
                "parent_qualified_name".to_string(),
                projection_atom_identity_node(ctx, &id.parent_qualified_name),
            ),
            (
                "variant_name".to_string(),
                projection_atom_identity_node(ctx, &id.variant_name),
            ),
        ],
    )
}

fn marshal_constructor_identity_node(
    ctx: &InterpContext,
    parent: &ResolvedDeclarationIdentity,
    variant: &ResolvedVariantIdentity,
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
                    projection_atom_identity_node(ctx, &v.parent_qualified_name),
                ),
                (
                    "variant_name".to_string(),
                    projection_atom_identity_node(ctx, &v.variant_name),
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

fn marshal_field_initializer_projection(
    ctx: &InterpContext,
    importing_module: &str,
    field_value: &Rc<Node>,
    si: &SourceIndices,
) -> Value {
    match field_value.expr_data.as_ref() {
        ExprData::ExprRecordLit { .. } => {
            let type_bare = inferred_field_type_bare(si, field_value).unwrap_or_default();
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
            let type_item = match type_item_from_importing_module_type_env(&tm, &type_bare, si) {
                Some(node) => node,
                None => return constructor_resolution_refused_projection(ctx),
            };
            if type_item.connective == Connective::Disj {
                marshal_coproduct_record_projection(
                    ctx,
                    importing_module,
                    field_value,
                    &type_bare,
                    si,
                    type_item,
                )
            } else {
                marshal_plain_record_projection(
                    ctx,
                    importing_module,
                    field_value,
                    &type_bare,
                    si,
                    type_item,
                )
            }
        }
        _ => {
            let resolution =
                variant_value_from_typechecked_expr(ctx, importing_module, field_value, si);
            marshal_value_identity_node(ctx, &resolution)
        }
    }
}

fn marshal_plain_record_projection(
    ctx: &InterpContext,
    importing_module: &str,
    body: &Rc<Node>,
    type_bare: &str,
    si: &SourceIndices,
    type_item: Rc<Node>,
) -> Value {
    let parent_id =
        declaration_identity_for_type_item(ctx, &type_item, si, importing_module);

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
        let field_projection =
            marshal_field_initializer_projection(ctx, importing_module, &field_value, si);
        edges.push((field_name.clone(), field_projection));
    }

    projection_node_with_named_edges(ctx, "DataInitializerPlainRecordProjection", &edges)
}

fn marshal_coproduct_record_projection(
    ctx: &InterpContext,
    importing_module: &str,
    body: &Rc<Node>,
    declared_type_bare: &str,
    si: &SourceIndices,
    coproduct_item: Rc<Node>,
) -> Value {
    let variant_name = authored_name_at(si.clone(), body.clone());
    if variant_name.is_empty()
        || !crate::v1_std_core::has_child_named(
            coproduct_item.clone(),
            variant_name.clone(),
            si.clone(),
        )
    {
        return constructor_resolution_refused_projection(ctx);
    }

    let parent_id =
        declaration_identity_for_type_item(ctx, &coproduct_item, si, importing_module);
    let variant_id = ResolvedVariantIdentity {
        parent_qualified_name: parent_id.qualified_name.clone(),
        variant_name,
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
        let field_projection =
            marshal_field_initializer_projection(ctx, importing_module, &field_value, si);
        edges.push((field_name.clone(), field_projection));
    }

    projection_node_with_named_edges(ctx, "DataInitializerRecordProjection", &edges)
}

pub fn marshal_data_initializer_projection(
    ctx: &InterpContext,
    qualified_name: &str,
) -> InterpResult<Value> {
    let item = match ctx.lookup_typed_item(qualified_name) {
        Some(item) => item,
        None => return Ok(typechecked_subject_absent_projection(ctx)),
    };

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

    let tm = match typed_module_for_path(ctx, &importing_module) {
        Some(tm) => tm,
        None => return Ok(constructor_resolution_refused_projection(ctx)),
    };

    match body.expr_data.as_ref() {
        ExprData::ExprRecordLit { .. } => {
            let type_item = match type_item_from_importing_module_type_env(&tm, &declared_type_bare, &si)
            {
                Some(node) => node,
                None => return Ok(constructor_resolution_refused_projection(ctx)),
            };
            if type_item.connective == Connective::Disj {
                Ok(marshal_coproduct_record_projection(
                    ctx,
                    &importing_module,
                    body,
                    &declared_type_bare,
                    &si,
                    type_item,
                ))
            } else {
                Ok(marshal_plain_record_projection(
                    ctx,
                    &importing_module,
                    body,
                    &declared_type_bare,
                    &si,
                    type_item,
                ))
            }
        }
        ExprData::ExprVar { .. } => {
            let resolution = variant_value_from_typechecked_expr(ctx, &importing_module, body, &si);
            let mut edges: Vec<(String, Value)> = vec![(
                "value_identity".to_string(),
                marshal_value_identity_node(ctx, &resolution),
            )];
            if let DataInitializerValueResolution::Resolved(v) = &resolution {
                if let Some(type_item) =
                    type_item_from_importing_module_type_env(&tm, &declared_type_bare, &si)
                {
                    let parent_id = declaration_identity_for_type_item(
                        ctx,
                        &type_item,
                        &si,
                        &importing_module,
                    );
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

pub fn typechecked_subject_absent_projection(ctx: &InterpContext) -> Value {
    projection_node_with_named_edges(
        ctx,
        "DataInitializerTypecheckedSubjectAbsentProjection",
        &[],
    )
}
