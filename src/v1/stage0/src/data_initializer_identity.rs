//! Resolved constructor and variant-value identity for `decl_facts` data-initializer projection.

use im::HashMap;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::cli_run::{build_module_path_index, workspace_root};
use crate::module_path_index::parsed_dag_file::parse_dag_file;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_compiler_infer_types::normalize_access_type_node;
use crate::v1_interpreter::{sorted_fields, InterpContext, InterpError, InterpResult, Value};
use crate::v1_std_core::{
    authored_name_at, expr_var_name_at, field_init_node_name_at, field_init_node_value,
    find_child_named, has_child_named, module_imports, param_node_type_expr,
    record_lit_type_name_at, Connective, ExprData, InferredNode, NewlineIndex, Node,
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

#[derive(Debug, Clone)]
struct TypeCatalogEntry {
    qualified_name: String,
    name: String,
    module_path: String,
    rel_path: String,
    item: Rc<Node>,
    source_indices: SourceIndices,
}

#[derive(Debug, Clone)]
pub struct CorpusTypeCatalog {
    types_by_qualified: HashMap<String, TypeCatalogEntry>,
    types_by_bare: BTreeMap<String, Vec<String>>,
    module_imports: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
enum TypeResolutionFailure {
    Missing,
    Ambiguous(Vec<String>),
}

impl CorpusTypeCatalog {
    pub fn build_for_pool(pool_roots: &[String]) -> Self {
        let ws = workspace_root();
        let mut catalog_roots: Vec<String> =
            vec![ws.join("dag/std").to_string_lossy().into_owned()];
        for root in pool_roots {
            catalog_roots.push(ws.join(root).to_string_lossy().into_owned());
        }
        let index = build_module_path_index(&catalog_roots);
        let mut types_by_qualified = HashMap::new();
        let mut types_by_bare: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut module_imports = HashMap::new();

        for (module_path, rel_path) in index {
            let abs = ws.join(&rel_path);
            let Some(parsed) = parse_dag_file(&abs) else {
                continue;
            };
            let si = parsed.source_indices.clone();
            let imports = import_map_for_module(parsed.module.clone(), &si);
            module_imports.insert(module_path.clone(), imports);

            for item in parsed.items.iter() {
                if item_kind(item.clone()) != ItemKind::TypeItem {
                    continue;
                }
                let name = authored_name_at(si.clone(), item.clone());
                if name.is_empty() {
                    continue;
                }
                let qualified_name = decl_logical_qualified_name(&module_path, &name);
                let entry = TypeCatalogEntry {
                    qualified_name: qualified_name.clone(),
                    name,
                    module_path: module_path.clone(),
                    rel_path: rel_path.clone(),
                    item: item.clone(),
                    source_indices: si.clone(),
                };
                types_by_qualified.insert(qualified_name.clone(), entry.clone());
                types_by_bare
                    .entry(entry.name.clone())
                    .or_default()
                    .push(qualified_name);
            }
        }

        CorpusTypeCatalog {
            types_by_qualified,
            types_by_bare,
            module_imports,
        }
    }

    fn declaration_identity_for_type(
        &self,
        qualified_name: &str,
    ) -> Option<ResolvedDeclarationIdentity> {
        self.types_by_qualified
            .get(qualified_name)
            .map(|e| ResolvedDeclarationIdentity {
                qualified_name: e.qualified_name.clone(),
                name: e.name.clone(),
                module_path: e.module_path.clone(),
                rel_path: e.rel_path.clone(),
            })
    }

    fn resolve_type_name(
        &self,
        bare_name: &str,
        importing_module: &str,
    ) -> Result<String, TypeResolutionFailure> {
        if let Some(imports) = self.module_imports.get(importing_module) {
            if let Some(qn) = imports.get(bare_name) {
                if self.types_by_qualified.contains_key(qn) {
                    return Ok(qn.clone());
                }
                return Err(TypeResolutionFailure::Missing);
            }
        }
        let local_qn = decl_logical_qualified_name(importing_module, bare_name);
        if self.types_by_qualified.contains_key(&local_qn) {
            return Ok(local_qn);
        }
        match self.types_by_bare.get(bare_name) {
            None => Err(TypeResolutionFailure::Missing),
            Some(matches) if matches.len() == 1 => Ok(matches[0].clone()),
            Some(matches) => Err(TypeResolutionFailure::Ambiguous(matches.clone())),
        }
    }

    fn coproduct_def_node(
        &self,
        entry: &TypeCatalogEntry,
    ) -> Result<Rc<Node>, TypeResolutionFailure> {
        if entry.item.connective == Connective::Disj {
            return Ok(entry.item.clone());
        }
        if let Some(inferred) = entry.item.inferred.as_ref() {
            if let InferredNode::Resolved { node, .. } = inferred.as_ref() {
                if node.connective == Connective::Disj {
                    return Ok(node.clone());
                }
            }
        }
        Err(TypeResolutionFailure::Missing)
    }

    fn resolve_variant_in_parent(
        &self,
        parent_qualified: &str,
        variant_name: &str,
    ) -> Result<ResolvedVariantIdentity, TypeResolutionFailure> {
        let entry = self
            .types_by_qualified
            .get(parent_qualified)
            .ok_or(TypeResolutionFailure::Missing)?;
        let coproduct = self.coproduct_def_node(entry)?;
        if !has_child_named(
            coproduct.clone(),
            variant_name.to_string(),
            entry.source_indices.clone(),
        ) {
            return Err(TypeResolutionFailure::Missing);
        }
        Ok(ResolvedVariantIdentity {
            parent_qualified_name: parent_qualified.to_string(),
            variant_name: variant_name.to_string(),
        })
    }

    fn field_type_name_in_variant(
        &self,
        parent_qualified: &str,
        variant_name: &str,
        field_name: &str,
    ) -> Result<String, TypeResolutionFailure> {
        let entry = self
            .types_by_qualified
            .get(parent_qualified)
            .ok_or(TypeResolutionFailure::Missing)?;
        let variant = find_child_named(
            entry.item.clone(),
            variant_name.to_string(),
            entry.source_indices.clone(),
        )
        .ok_or(TypeResolutionFailure::Missing)?;
        self.field_type_name_in_product_node(&entry, variant.clone(), field_name)
    }

    fn field_type_name_in_record(
        &self,
        parent_qualified: &str,
        field_name: &str,
    ) -> Result<String, TypeResolutionFailure> {
        let entry = self
            .types_by_qualified
            .get(parent_qualified)
            .ok_or(TypeResolutionFailure::Missing)?;
        self.field_type_name_in_product_node(&entry, entry.item.clone(), field_name)
    }

    fn field_type_name_in_product_node(
        &self,
        entry: &TypeCatalogEntry,
        product: Rc<Node>,
        field_name: &str,
    ) -> Result<String, TypeResolutionFailure> {
        for child in product.children.iter() {
            let fname = authored_name_at(entry.source_indices.clone(), child.clone());
            if fname != field_name {
                continue;
            }
            if let Some(inferred) = child.inferred.as_ref() {
                if let InferredNode::Resolved { node: ft, .. } = inferred.as_ref() {
                    let ty_name = authored_name_at(
                        entry.source_indices.clone(),
                        normalize_access_type_node(ft.clone()),
                    );
                    if !ty_name.is_empty() {
                        return Ok(ty_name);
                    }
                }
            }
            let ty = param_node_type_expr(child.clone());
            let ty_name = authored_name_at(entry.source_indices.clone(), ty);
            if ty_name.is_empty() {
                return Err(TypeResolutionFailure::Missing);
            }
            return Ok(ty_name);
        }
        Err(TypeResolutionFailure::Missing)
    }

    fn resolve_constructor_identity(
        &self,
        importing_module: &str,
        declared_type_bare: &str,
        variant_name: &str,
    ) -> Result<(ResolvedDeclarationIdentity, ResolvedVariantIdentity), TypeResolutionFailure> {
        let parent_qn = self.resolve_type_name(declared_type_bare, importing_module)?;
        let parent_id = self
            .declaration_identity_for_type(&parent_qn)
            .ok_or(TypeResolutionFailure::Missing)?;
        let variant_id = self.resolve_variant_in_parent(&parent_qn, variant_name)?;
        Ok((parent_id, variant_id))
    }

    pub fn resolve_variant_value_identity(
        &self,
        importing_module: &str,
        expected_type_bare: &str,
        value_name: &str,
    ) -> DataInitializerValueResolution {
        match self.resolve_type_name(expected_type_bare, importing_module) {
            Err(TypeResolutionFailure::Missing) => DataInitializerValueResolution::Missing,
            Err(TypeResolutionFailure::Ambiguous(cands)) => {
                let variants = cands
                    .into_iter()
                    .filter_map(|qn| self.resolve_variant_in_parent(&qn, value_name).ok())
                    .collect::<Vec<_>>();
                if variants.len() == 1 {
                    DataInitializerValueResolution::Resolved(variants[0].clone())
                } else if variants.is_empty() {
                    DataInitializerValueResolution::Missing
                } else {
                    DataInitializerValueResolution::Ambiguous(variants)
                }
            }
            Ok(parent_qn) => match self.resolve_variant_in_parent(&parent_qn, value_name) {
                Ok(v) => DataInitializerValueResolution::Resolved(v),
                Err(TypeResolutionFailure::Missing) => DataInitializerValueResolution::Missing,
                Err(TypeResolutionFailure::Ambiguous(cands)) => {
                    let variants = cands
                        .into_iter()
                        .filter_map(|qn| self.resolve_variant_in_parent(&qn, value_name).ok())
                        .collect::<Vec<_>>();
                    if variants.len() == 1 {
                        DataInitializerValueResolution::Resolved(variants[0].clone())
                    } else if variants.is_empty() {
                        DataInitializerValueResolution::Missing
                    } else {
                        DataInitializerValueResolution::Ambiguous(variants)
                    }
                }
            },
        }
    }
}

fn import_map_for_module(module: Rc<Node>, si: &SourceIndices) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for imp in module_imports(module).iter() {
        let module_path = imp.name.clone();
        if imp.body.is_some() {
            continue;
        }
        for child in imp.children.iter() {
            let local = authored_name_at(si.clone(), child.clone());
            if local.is_empty() {
                continue;
            }
            let qualified = if module_path.is_empty() {
                local.clone()
            } else {
                format!("{module_path}.{local}")
            };
            out.insert(local, qualified);
        }
    }
    out
}

fn carrier_module_from_qualified(qualified_name: &str, decl_name: &str) -> String {
    let suffix = format!(".{decl_name}");
    if qualified_name.ends_with(&suffix) {
        qualified_name[..qualified_name.len() - suffix.len()].to_string()
    } else {
        qualified_name.to_string()
    }
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

fn expr_var_bare_name(node: &Rc<Node>, si: &SourceIndices) -> Option<String> {
    if !matches!(node.expr_data.as_ref(), ExprData::ExprVar { .. }) {
        return None;
    }
    let name = expr_var_name_at(node.clone(), si.clone());
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
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

fn marshal_plain_record_projection(
    ctx: &InterpContext,
    body: &Rc<Node>,
    type_bare: &str,
    importing_module: &str,
    catalog: &CorpusTypeCatalog,
    si: &SourceIndices,
) -> InterpResult<Value> {
    let parent_qn = catalog
        .resolve_type_name(importing_module, type_bare)
        .map_err(|_| InterpError::TypeError {
            msg: format!("plain record type resolution refused for `{type_bare}`"),
        })?;
    let parent_id = catalog
        .declaration_identity_for_type(&parent_qn)
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("plain record declaration identity missing for `{parent_qn}`"),
        })?;

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
        let field_projection = match catalog.field_type_name_in_record(&parent_qn, &field_name) {
            Ok(expected_field_type) => marshal_field_initializer_projection(
                ctx,
                &field_value,
                &expected_field_type,
                importing_module,
                catalog,
                si,
            )?,
            Err(_) => {
                marshal_value_identity_node(ctx, &DataInitializerValueResolution::NotVariantValue)
            }
        };
        edges.push((field_name.clone(), field_projection));
    }

    Ok(projection_node_with_named_edges(
        ctx,
        "DataInitializerPlainRecordProjection",
        &edges,
    ))
}

fn marshal_field_initializer_projection(
    ctx: &InterpContext,
    field_value: &Rc<Node>,
    expected_field_type: &str,
    importing_module: &str,
    catalog: &CorpusTypeCatalog,
    si: &SourceIndices,
) -> InterpResult<Value> {
    match field_value.expr_data.as_ref() {
        ExprData::ExprRecordLit { .. } => {
            match catalog.resolve_type_name(importing_module, expected_field_type) {
                Ok(parent_qn) => {
                    let entry = catalog.types_by_qualified.get(&parent_qn).ok_or_else(|| {
                        InterpError::TypeError {
                            msg: format!("nested record catalog entry missing for `{parent_qn}`"),
                        }
                    })?;
                    if entry.item.connective == Connective::Conj {
                        marshal_plain_record_projection(
                            ctx,
                            field_value,
                            expected_field_type,
                            importing_module,
                            catalog,
                            si,
                        )
                    } else if catalog.coproduct_def_node(entry).is_ok() {
                        marshal_record_initializer_projection(
                            ctx,
                            field_value,
                            expected_field_type,
                            importing_module,
                            catalog,
                            si,
                        )
                    } else {
                        marshal_plain_record_projection(
                            ctx,
                            field_value,
                            expected_field_type,
                            importing_module,
                            catalog,
                            si,
                        )
                    }
                }
                Err(_) => Ok(marshal_value_identity_node(
                    ctx,
                    &DataInitializerValueResolution::NotVariantValue,
                )),
            }
        }
        _ => {
            let value_resolution = resolve_expr_value_identity(
                field_value,
                expected_field_type,
                importing_module,
                catalog,
                si,
            );
            Ok(marshal_value_identity_node(ctx, &value_resolution))
        }
    }
}

fn marshal_record_initializer_projection(
    ctx: &InterpContext,
    body: &Rc<Node>,
    declared_type_bare: &str,
    importing_module: &str,
    catalog: &CorpusTypeCatalog,
    si: &SourceIndices,
) -> InterpResult<Value> {
    let variant_name = record_lit_type_name_at(body.clone(), si.clone()).unwrap_or_default();
    if variant_name.is_empty() {
        return Ok(projection_node_with_named_edges(
            ctx,
            "DataInitializerConstructorResolutionRefusedProjection",
            &[],
        ));
    }
    let (parent_id, variant_id) = match catalog.resolve_constructor_identity(
        importing_module,
        declared_type_bare,
        &variant_name,
    ) {
        Ok(ids) => ids,
        Err(_) => {
            return Ok(projection_node_with_named_edges(
                ctx,
                "DataInitializerConstructorResolutionRefusedProjection",
                &[],
            ));
        }
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
        let field_projection = match catalog.field_type_name_in_variant(
            &parent_id.qualified_name,
            &variant_id.variant_name,
            &field_name,
        ) {
            Ok(expected_field_type) => marshal_field_initializer_projection(
                ctx,
                &field_value,
                &expected_field_type,
                importing_module,
                catalog,
                si,
            )?,
            Err(_) => {
                marshal_value_identity_node(ctx, &DataInitializerValueResolution::NotVariantValue)
            }
        };
        edges.push((field_name.clone(), field_projection));
    }

    Ok(projection_node_with_named_edges(
        ctx,
        "DataInitializerRecordProjection",
        &edges,
    ))
}

fn nullary_variant_spelling(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

fn resolve_expr_value_identity(
    node: &Rc<Node>,
    expected_type_bare: &str,
    importing_module: &str,
    catalog: &CorpusTypeCatalog,
    si: &SourceIndices,
) -> DataInitializerValueResolution {
    if let Some(var_name) = expr_var_bare_name(node, si) {
        return catalog.resolve_variant_value_identity(
            importing_module,
            expected_type_bare,
            &var_name,
        );
    }
    if matches!(node.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        let variant_name = record_lit_type_name_at(node.clone(), si.clone()).unwrap_or_default();
        if !variant_name.is_empty() && node.children.is_empty() {
            return catalog.resolve_variant_value_identity(
                importing_module,
                expected_type_bare,
                &variant_name,
            );
        }
        return DataInitializerValueResolution::NotVariantValue;
    }
    let authored = authored_name_at(si.clone(), node.clone());
    if nullary_variant_spelling(&authored) && node.children.is_empty() {
        let resolution =
            catalog.resolve_variant_value_identity(importing_module, expected_type_bare, &authored);
        if !matches!(resolution, DataInitializerValueResolution::NotVariantValue) {
            return resolution;
        }
    }
    if matches!(node.expr_data.as_ref(), ExprData::ExprLiteral { .. }) {
        return DataInitializerValueResolution::NotVariantValue;
    }
    DataInitializerValueResolution::NotVariantValue
}

pub fn marshal_data_initializer_projection(
    ctx: &InterpContext,
    item: &Rc<Node>,
    qualified_name: &str,
    decl_name: &str,
    catalog: &CorpusTypeCatalog,
    si: &SourceIndices,
) -> InterpResult<Value> {
    let importing_module = carrier_module_from_qualified(qualified_name, decl_name);
    let body = item.body.as_ref().ok_or_else(|| InterpError::TypeError {
        msg: "data item missing initializer body".to_string(),
    })?;
    let declared_type_bare = match declared_type_bare_name(item, si) {
        Some(name) => name,
        None => {
            return Ok(projection_node_with_named_edges(
                ctx,
                "DataInitializerConstructorResolutionRefusedProjection",
                &[],
            ));
        }
    };

    match body.expr_data.as_ref() {
        ExprData::ExprRecordLit { .. } => marshal_record_initializer_projection(
            ctx,
            body,
            &declared_type_bare,
            &importing_module,
            catalog,
            si,
        ),
        ExprData::ExprVar { .. } => {
            let var_name = expr_var_bare_name(body, si).unwrap_or_default();
            let resolution = catalog.resolve_variant_value_identity(
                &importing_module,
                &declared_type_bare,
                &var_name,
            );
            let mut edges: Vec<(String, Value)> = vec![(
                "value_identity".to_string(),
                marshal_value_identity_node(ctx, &resolution),
            )];
            if let Ok((parent_id, variant_id)) = catalog.resolve_constructor_identity(
                &importing_module,
                &declared_type_bare,
                &var_name,
            ) {
                edges.push((
                    "constructor_identity".to_string(),
                    marshal_constructor_identity_node(ctx, &parent_id, &variant_id),
                ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_dissolves_to_field_type_resolves() {
        let catalog = CorpusTypeCatalog::build_for_pool(&["dag".to_string()]);
        let got = catalog
            .field_type_name_in_variant("std.disposition.Disposition", "Scaffold", "dissolves_to")
            .expect("dissolves_to field type");
        assert_eq!(got, "ConstructionMechanism");
    }

    #[test]
    fn scaffold_bind_field_type_resolves() {
        let catalog = CorpusTypeCatalog::build_for_pool(&["dag".to_string()]);
        let got = catalog
            .field_type_name_in_variant("std.disposition.Disposition", "Scaffold", "bind")
            .expect("bind field type");
        assert_eq!(got, "DeclarationRef");
    }

    #[test]
    fn declaration_ref_field_type_resolves() {
        let catalog = CorpusTypeCatalog::build_for_pool(&["dag".to_string()]);
        let got = catalog
            .field_type_name_in_record("std.decl_ref.DeclarationRef", "field")
            .expect("field type");
        assert_eq!(got, "DeclField");
    }

    #[test]
    fn decl_field_whole_declaration_variant_resolves() {
        let catalog = CorpusTypeCatalog::build_for_pool(&["dag".to_string()]);
        let resolution = catalog.resolve_variant_value_identity(
            "test.fixture.decl_facts_reflection.specimens",
            "DeclField",
            "WholeDeclaration",
        );
        match resolution {
            DataInitializerValueResolution::Resolved(v) => {
                assert_eq!(v.parent_qualified_name, "std.decl_ref.DeclField");
                assert_eq!(v.variant_name, "WholeDeclaration");
            }
            other => panic!("expected Resolved WholeDeclaration, got {other:?}"),
        }
    }

    #[test]
    fn disposition_scaffold_bind_field_value_shape() {
        use crate::cli_run::workspace_root;
        use crate::module_path_index::parsed_dag_file::parse_dag_file;
        use crate::v1_compiler_compile::compile_to_resolved;
        use crate::v1_interpreter::{ExecutionMode, InterpContext};
        use crate::v1_std_core::{
            authored_name_at, field_init_node_name_at, field_init_node_value, SourceFile,
        };
        use im::vector;
        use std::rc::Rc;

        let path = workspace_root()
            .join("dag/test/fixture/decl_facts_reflection/specimens.dag");
        let parsed = parse_dag_file(&path).expect("parse specimens");
        let si = parsed.source_indices.clone();
        let item = parsed
            .items
            .iter()
            .find(|i| authored_name_at(si.clone(), i.clone()) == "disposition_scaffold")
            .expect("disposition_scaffold item");
        let body = item.body.as_ref().expect("body");
        for child in body.children.iter() {
            let fname = field_init_node_name_at(child.clone(), si.clone());
            if fname != "bind" {
                continue;
            }
            let val = field_init_node_value(child.clone());
            assert!(
                val.children.is_empty(),
                "bind field value should be nullary WholeDeclaration, children={}",
                val.children.len()
            );
            assert_eq!(
                authored_name_at(si.clone(), val.clone()),
                "WholeDeclaration"
            );
        }

        let content = std::fs::read_to_string(&path).unwrap();
        let result = compile_to_resolved(Rc::new(vector![Rc::new(SourceFile {
            path: path.to_string_lossy().into_owned(),
            content,
        })]));
        let graph = result.graph.as_ref().expect("graph");
        let ctx = InterpContext::new(graph, result.source_indices.clone(), ExecutionMode::Hermetic);
        let catalog = CorpusTypeCatalog::build_for_pool(&["dag".to_string(), "src/v2".to_string()]);
        let projection = marshal_data_initializer_projection(
            &ctx,
            item,
            "test.fixture.decl_facts_reflection.specimens.disposition_scaffold",
            "disposition_scaffold",
            &catalog,
            &si,
        )
        .expect("marshal projection");
        let bind_projection = projection_named_child(&projection, "bind").expect("bind projection");
        let field_projection =
            projection_named_child(&bind_projection, "field").expect("field projection");
        let field_kind = projection_kind_lexeme(&field_projection).expect("field kind");
        assert!(
            field_kind == "ResolvedVariantValueProjection"
                || field_kind == "DataInitializerRecordProjection",
            "unexpected field projection kind: {field_kind}"
        );
        let variant = projection_variant_label(&field_projection).expect("field variant label");
        assert_eq!(variant, "WholeDeclaration");
    }

    fn projection_children(projection: &Value) -> Option<Vec<Value>> {
        match projection {
            Value::Record { fields, .. } => {
                fields
                    .iter()
                    .find(|(k, _)| k.as_str() == "children")
                    .and_then(|(_, v)| match v {
                        Value::List(items) => Some(items.clone()),
                        _ => None,
                    })
            }
            _ => None,
        }
    }

    fn projection_named_child(projection: &Value, label: &str) -> Option<Value> {
        for edge in projection_children(projection)? {
            if let Value::Record { fields, .. } = edge {
                let edge_label = fields.iter().find(|(k, _)| k.as_str() == "label");
                let target = fields.iter().find(|(k, _)| k.as_str() == "target");
                if let (
                    Some((_, Value::Variant { fields: label_fields, .. })),
                    Some((_, target)),
                ) = (edge_label, target)
                {
                    let name = label_fields
                        .iter()
                        .find(|(k, _)| k.as_str() == "name")
                        .and_then(|(_, v)| match v {
                            Value::Str(s) => Some(s.as_str()),
                            _ => None,
                        });
                    if name == Some(label) {
                        return Some(target.clone());
                    }
                }
            }
        }
        None
    }

    fn projection_kind_lexeme(projection: &Value) -> Option<String> {
        let kind = projection_named_child(projection, "kind")?;
        projection_atom_lexeme(&kind)
    }

    fn projection_atom_lexeme(node: &Value) -> Option<String> {
        match node {
            Value::Record { fields, .. } => {
                let connective = projection_named_child(node, "connective")?;
                match connective {
                    Value::Variant { fields, .. } => fields
                        .iter()
                        .find(|(k, _)| k.as_str() == "identity")
                        .and_then(|(_, v)| match v {
                            Value::Str(s) => Some(s.clone()),
                            _ => None,
                        }),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn projection_variant_label(projection: &Value) -> Option<String> {
        let kind = projection_kind_lexeme(projection)?;
        if kind == "ResolvedVariantValueProjection" {
            projection_named_child(projection, "variant_name")
                .and_then(|n| projection_atom_lexeme(&n))
        } else if kind == "DataInitializerRecordProjection" {
            let ctor = projection_named_child(projection, "constructor_identity")?;
            let variant = projection_named_child(&ctor, "constructor")?;
            projection_named_child(&variant, "variant_name")
                .and_then(|n| projection_atom_lexeme(&n))
        } else {
            None
        }
    }
}
