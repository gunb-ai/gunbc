use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::cli_run::{collect_dag_files_tolerant, is_test_dag, repo_rel, workspace_root};
use crate::module_path_index::medium_structure_census::parse_dag_file;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_interpreter::{
    fields_get, sorted_fields, InterpContext, InterpError, InterpResult, Value,
};
use crate::v1_std_core::{
    authored_name_at, expr_var_name_at, field_node_type_expr, inferred_to_node, param_node_name_at,
    source_text_at, Connective, ExprData, NewlineIndex, Node, VarBindingKind,
};

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

fn expect_symbol<'a>(value: Option<&'a Value>, what: &str) -> InterpResult<&'a str> {
    match value {
        Some(Value::Str(s)) => Ok(s.as_str()),
        _ => Err(InterpError::TypeError {
            msg: format!("{what} requires a Symbol argument"),
        }),
    }
}

fn expect_string_lexeme(value: Option<&Value>, what: &str) -> InterpResult<String> {
    let val = value.ok_or_else(|| InterpError::TypeError {
        msg: format!("{what} requires a lexeme argument"),
    })?;
    match val {
        Value::Str(s) => Ok(s.clone()),
        _ => {
            let items = crate::v1_interpreter::free_monoid_to_vec(val).ok_or_else(|| {
                InterpError::TypeError {
                    msg: format!("{what} requires a String lexeme"),
                }
            })?;
            let mut spelling = String::new();
            for item in items {
                match item {
                    Value::Int(c) => {
                        if let Some(ch) = char::from_u32(c as u32) {
                            spelling.push(ch);
                        }
                    }
                    _ => {
                        return Err(InterpError::TypeError {
                            msg: format!("{what} lexeme contains non-Char element"),
                        });
                    }
                }
            }
            Ok(spelling)
        }
    }
}

pub fn eval_symbol_intern_lexeme(
    _ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let spelling = expect_string_lexeme(args.first().map(|(_, v)| v), "symbol_intern_lexeme")?;
    Ok(Value::Str(spelling))
}

pub(crate) fn type_item_by_name<'a>(
    ctx: &'a InterpContext,
    type_name: &str,
) -> InterpResult<(&'a Rc<Node>, String)> {
    let si = ctx.source_indices();
    for module in ctx.modules.iter() {
        for item in module.items.iter() {
            let name = authored_name_at(si.clone(), item.clone());
            if name != type_name {
                continue;
            }
            let is_type = ctx
                .item_registry
                .get(&name)
                .map(|info| info.kind == ItemKind::TypeItem)
                .unwrap_or(false)
                || ctx
                    .item_registry
                    .get(&item.name)
                    .map(|info| info.kind == ItemKind::TypeItem)
                    .unwrap_or(false);
            if is_type {
                return Ok((item, item.span.file.clone()));
            }
        }
    }
    Err(InterpError::TypeError {
        msg: format!("resolve_type_node: unknown closed type `{type_name}`"),
    })
}

fn nullary_connective_variant(ctx: &InterpContext, name: &str) -> Value {
    Value::Variant {
        type_name: ctx.sym("Connective"),
        variant_name: ctx.sym(name),
        fields: Rc::new(vec![]),
    }
}

fn atom_connective_variant(ctx: &InterpContext, identity: &str) -> Value {
    Value::Variant {
        type_name: ctx.sym("Connective"),
        variant_name: ctx.sym("Atom"),
        fields: Rc::new(vec![(
            ctx.sym("identity"),
            Value::Str(identity.to_string()),
        )]),
    }
}

fn node_kind_type_node(ctx: &InterpContext, connective: Value) -> Value {
    Value::Variant {
        type_name: ctx.sym("NodeKind"),
        variant_name: ctx.sym("TypeNode"),
        fields: Rc::new(vec![(ctx.sym("connective"), connective)]),
    }
}

fn synthetic_occurrence(ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("NodeOccurrenceId"),
        variant_name: ctx.sym("SyntheticOccurrence"),
        fields: Rc::new(vec![]),
    }
}

fn node_record(ctx: &InterpContext, kind: Value, children: Vec<Value>) -> Value {
    Value::Record {
        type_name: ctx.sym("Node"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("kind"), kind),
            (
                ctx.sym("children"),
                crate::v1_interpreter::list_value(children),
            ),
            (ctx.sym("occurrence_id"), synthetic_occurrence(ctx)),
        ])),
    }
}

fn edge_named(ctx: &InterpContext, name: &str, target: Value) -> Value {
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

fn unit_type_node(ctx: &InterpContext) -> Value {
    node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Conj")),
        vec![],
    )
}

fn type_expr_authored_name(
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    type_expr: &Rc<Node>,
) -> String {
    if type_expr.connective == Connective::Conj
        && type_expr.type_annotation.is_some()
        && type_expr.children.len() == 1
    {
        return type_expr_authored_name(si, &type_expr.children[0]);
    }
    let name = authored_name_at(si.clone(), type_expr.clone());
    if !name.is_empty() {
        return name;
    }
    if !type_expr.name.is_empty() {
        return type_expr.name.clone();
    }
    if let Some(inferred) = type_expr
        .inferred
        .as_ref()
        .and_then(|inf| inferred_to_node(inf.clone()))
    {
        let inferred_name = authored_name_at(si.clone(), inferred.clone());
        if !inferred_name.is_empty() {
            return inferred_name;
        }
        if !inferred.name.is_empty() {
            return inferred.name.clone();
        }
    }
    String::new()
}

fn marshal_type_expr_ref(
    ctx: &InterpContext,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    type_expr: &Rc<Node>,
) -> InterpResult<Value> {
    let name = type_expr_authored_name(si, type_expr);
    if name.is_empty() {
        return Err(InterpError::TypeError {
            msg: "marshal_type_expr_ref: empty authored type name".to_string(),
        });
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(ctx, atom_connective_variant(ctx, &name)),
        vec![],
    ))
}

fn marshal_variant_arm_target(
    ctx: &InterpContext,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    variant: &Rc<Node>,
) -> InterpResult<Value> {
    if variant.children.is_empty() {
        return Ok(unit_type_node(ctx));
    }
    let mut edges = Vec::with_capacity(variant.children.len());
    for field in variant.children.iter() {
        let field_name = authored_name_at(si.clone(), field.clone());
        let type_expr = field
            .inferred
            .as_ref()
            .and_then(|inf| inferred_to_node(inf.clone()))
            .unwrap_or_else(|| field_node_type_expr(field.clone()));
        let target = marshal_type_expr_ref(ctx, si, &type_expr)?;
        edges.push(edge_named(ctx, &field_name, target));
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Conj")),
        edges,
    ))
}

pub fn marshal_disj_type_item(
    ctx: &InterpContext,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    item: &Rc<Node>,
) -> InterpResult<Value> {
    if item.connective != Connective::Disj {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: type is not a closed coproduct (Disj)".to_string(),
        });
    }
    let mut edges = Vec::with_capacity(item.children.len());
    for child in item.children.iter() {
        let label = authored_name_at(si.clone(), child.clone());
        let target = marshal_variant_arm_target(ctx, si, child)?;
        edges.push(edge_named(ctx, &label, target));
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Disj")),
        edges,
    ))
}

pub fn eval_resolve_type_node(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "resolve_type_node")?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    marshal_disj_type_item(ctx, &ctx.source_indices(), item)
}

fn logical_qualified_name(module_name: &str, name: &str) -> String {
    if module_name.is_empty() {
        name.to_string()
    } else {
        format!("{module_name}.{name}")
    }
}

#[derive(Debug, Clone)]
struct ParsedTypeDecl {
    module_path: String,
    rel_path: String,
    name: String,
    item: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
}

/// Parse-only type-decl extraction over `build_module_path_index` — fail-closed on parse
/// errors (no silent skip). Shared substrate for `concept_decl_facts(pool_roots)`; distinct
/// from `decl_facts_corpus_walk` which skips test `.dag` files and tolerates parse failure.
fn type_decls_parse_only_fail_closed(
    roots: &[String],
) -> Result<(Vec<ParsedTypeDecl>, usize), String> {
    let ws = workspace_root();
    let index = crate::cli_run::build_module_path_index(roots);
    let mut modules: Vec<(String, String)> = index.into_iter().collect();
    modules.sort();
    let module_count = modules.len();
    let mut out = Vec::new();
    for (module_path, rel_path) in modules {
        let abs = ws.join(&rel_path);
        let parsed = parse_dag_file(&abs).ok_or_else(|| {
            format!(
                "concept_decl_facts: failed to parse `{rel_path}` (fail-closed; no silent skip)"
            )
        })?;
        let si = parsed.source_indices;
        for item in parsed.items.iter() {
            if item_kind(item.clone()) != ItemKind::TypeItem {
                continue;
            }
            let name = authored_name_at(si.clone(), item.clone());
            if name.is_empty() {
                continue;
            }
            out.push(ParsedTypeDecl {
                module_path: module_path.clone(),
                rel_path: rel_path.clone(),
                name,
                item: item.clone(),
                source_indices: si.clone(),
            });
        }
    }
    Ok((out, module_count))
}

fn concept_decl_node(
    ctx: &InterpContext,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    item: &Rc<Node>,
) -> InterpResult<Value> {
    match item.connective {
        Connective::Disj => marshal_disj_type_item(ctx, si, item),
        Connective::Conj => marshal_variant_arm_target(ctx, si, item),
        _ => Ok(unit_type_node(ctx)),
    }
}

pub fn eval_qualified_name_from_dotted_string(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let dotted = expect_string_lexeme(
        args.first().map(|(_, v)| v),
        "qualified_name_from_dotted_string",
    )?;
    Ok(crate::cli_run::qualified_name_value_from_dotted_string(
        ctx, &dotted,
    ))
}

fn concept_decl_record(
    ctx: &InterpContext,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    module_name: &str,
    name: &str,
    item: &Rc<Node>,
) -> InterpResult<Value> {
    let qualified_name = logical_qualified_name(module_name, name);
    let node = concept_decl_node(ctx, si, item)?;
    Ok(Value::Record {
        type_name: ctx.sym("ConceptDecl"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("qualified_name"), Value::Str(qualified_name)),
            (ctx.sym("name"), Value::Str(name.to_string())),
            (ctx.sym("node"), node),
        ])),
    })
}

pub fn eval_concept_decl_facts(ctx: &InterpContext, pool_roots: &[String]) -> InterpResult<Value> {
    let ws = crate::cli_run::workspace_root();
    let abs_pool_roots: Vec<String> = pool_roots
        .iter()
        .map(|r| ws.join(r).to_string_lossy().into_owned())
        .collect();
    let (type_decls, module_count) = type_decls_parse_only_fail_closed(&abs_pool_roots)
        .map_err(|msg| InterpError::TypeError { msg })?;
    let files_parsed = module_count;
    let mut rows: Vec<Value> = Vec::new();
    for decl in type_decls {
        let qualified_name = logical_qualified_name(&decl.module_path, &decl.name);
        let row = concept_decl_record(
            ctx,
            &decl.source_indices,
            &decl.module_path,
            &decl.name,
            &decl.item,
        )
        .map_err(|e| InterpError::TypeError {
            msg: format!(
                "concept_decl_facts: failed to marshal `{qualified_name}` in `{}`: {e}",
                decl.rel_path
            ),
        })?;
        rows.push(row);
    }
    eprintln!(
        "concept_decl_facts: {} type concepts from {module_count} modules ({files_parsed} files parsed)",
        rows.len()
    );
    Ok(crate::v1_interpreter::list_value(rows))
}

/// Shared live-corpus item walk for reflection builtins (`*_decl_facts_live`).
/// Resolves authored names through each module's `item_registry` and filters by kind.
fn for_each_live_registry_item<F>(
    ctx: &InterpContext,
    kind_ok: impl Fn(ItemKind) -> bool,
    mut f: F,
) -> InterpResult<()>
where
    F: FnMut(&str, &str, &Rc<Node>) -> InterpResult<()>,
{
    let si = ctx.source_indices();
    for module in ctx.modules.iter() {
        for item in module.items.iter() {
            let name = authored_name_at(si.clone(), item.clone());
            if name.is_empty() {
                continue;
            }
            let info = module
                .item_registry
                .get(&name)
                .or_else(|| module.item_registry.get(&item.name));
            let Some(info) = info else { continue };
            if !kind_ok(info.kind) {
                continue;
            }
            f(&info.module_name, &name, item)?;
        }
    }
    Ok(())
}

pub fn eval_concept_decl_facts_live(
    ctx: &InterpContext,
    _args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let si = ctx.source_indices();
    let mut rows: Vec<Value> = Vec::new();
    for_each_live_registry_item(
        ctx,
        |k| k == ItemKind::TypeItem,
        |module_name, name, item| {
            rows.push(concept_decl_record(ctx, &si, module_name, name, item)?);
            Ok(())
        },
    )?;
    Ok(crate::v1_interpreter::list_value(rows))
}

fn edge_positional(ctx: &InterpContext, target: Value) -> Value {
    Value::Record {
        type_name: ctx.sym("Edge"),
        fields: Rc::new(sorted_fields(vec![
            (
                ctx.sym("label"),
                Value::Variant {
                    type_name: ctx.sym("EdgeLabel"),
                    variant_name: ctx.sym("Positional"),
                    fields: Rc::new(vec![]),
                },
            ),
            (ctx.sym("target"), target),
        ])),
    }
}

fn atom_identity_node(ctx: &InterpContext, identity: &str) -> Value {
    node_record(
        ctx,
        node_kind_type_node(ctx, atom_connective_variant(ctx, identity)),
        vec![],
    )
}

fn node_authored_name(node: &Rc<Node>, si: &Rc<HashMap<String, Rc<NewlineIndex>>>) -> String {
    if !node.name.is_empty() {
        node.name.clone()
    } else {
        expr_var_name_at(node.clone(), si.clone())
    }
}

// Does this node REFERENCE a declared parameter `name`? Two body forms reference a value
// parameter: an `ExprVar` value read (`x`) -- resolved to a `LocalValueBinding`, so a
// `FunctionValueBinding` global or `VariantValueBinding` constructor sharing the name is
// excluded; and an `ExprCall` whose callee IS the parameter (a fn-valued param applied:
// `predicate(x)`) -- the callee is the call node's own name, not a child, so it is invisible
// to a children-only walk. Both are genuine uses of a value parameter.
fn node_references_param(node: &Rc<Node>, name: &str, param_names: &[String]) -> bool {
    if name.is_empty() || !param_names.iter().any(|p| p.as_str() == name) {
        return false;
    }
    match node.expr_data.as_ref() {
        ExprData::ExprVar {
            binding_kind: Some(bk),
        } => {
            matches!(bk.as_ref(), VarBindingKind::LocalValueBinding)
        }
        ExprData::ExprCall { .. } => true,
        _ => false,
    }
}

// Does this node REFERENCE some local binding (param OR let/lambda-local), by name? This is
// the PERMISSIVE companion of `node_references_param`: it captures every value read
// (`ExprVar`, any binding kind) and call-callee name, used ONLY to thread the data-flow
// reference set that decides let-liveness (below). It is deliberately over-inclusive --
// counting a name that is actually a global as "referenced" can only keep a `let` LIVE
// (graft its RHS), so it can never manufacture a false dead-wire RED; it merely forgoes a
// dead-wire it cannot prove. Atom EMISSION stays on the strict `node_references_param`
// (LocalValueBinding-only), so the reachability query itself is unchanged.
fn node_local_reference_name(node: &Rc<Node>, name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    match node.expr_data.as_ref() {
        ExprData::ExprVar { .. } | ExprData::ExprCall { .. } => Some(name.to_string()),
        _ => None,
    }
}

// The lambda's own parameters (children[1..]; child 0 is the body) are bound WITHIN it, so a
// reference to one of them is not a free reference of the enclosing scope -- subtract them
// from a lambda subtree's reference set so a `let` named like a lambda param is not kept
// spuriously live by the lambda's shadowing use.
fn lambda_param_names_of(
    node: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<String> {
    node.children
        .iter()
        .skip(1)
        .map(|c| authored_name_at(si.clone(), c.clone()))
        .filter(|s| !s.is_empty())
        .collect()
}

// Project a fn body's internal expression tree onto a substrate Node skeleton AND return the
// set of local names the projected skeleton actually references. Each node becomes a neutral
// `Conj` container whose positional children are the marshaled sub-expressions, and a node
// that references a declared parameter additionally carries an identity-bearing `Atom` leaf
// -- byte-identical to the declared-input atom `eval_fn_arrow_decl_facts_live` emits, so
// `v2.lens.wiring_liveness` matches it under Node equality. Identity lives ONLY on genuine
// parameter-reference sites, so a declared parameter is structurally reachable from the body
// output iff it is genuinely used.
//
// DATA-FLOW DIRECTED AT THE RETURN (closes the wiring_liveness construction_justification
// HONEST BOUNDARY (2)): statement sequences (blocks, and the let-continuation chain) are
// projected so a `let b = rhs` grafts `rhs`'s skeleton ONLY when `b` is referenced
// downstream of the binding in code that itself reaches the return -- the reverse-fold over
// statements in `marshal_stmt_sequence` carries the live reference set toward the binding.
// A param referenced ONLY inside a DEAD let RHS (its bound name never reaches the return,
// possibly transitively through a chain of dead lets) is therefore absent from the grafted
// skeleton and correctly flagged as a dead wire. (Residue: a `let`/lambda local, or a global
// fn called as `name(..)`, that shadows a parameter name; see the lens
// construction_justification boundary (3).)
fn marshal_fn_body_skeleton(
    ctx: &InterpContext,
    node: &Rc<Node>,
    param_names: &[String],
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Value {
    marshal_skeleton(ctx, node, param_names, si).0
}

fn conj_record(ctx: &InterpContext, edges: Vec<Value>) -> Value {
    node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Conj")),
        edges,
    )
}

fn marshal_skeleton(
    ctx: &InterpContext,
    node: &Rc<Node>,
    param_names: &[String],
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> (Value, std::collections::BTreeSet<String>) {
    match node.expr_data.as_ref() {
        ExprData::ExprBlock => marshal_stmt_sequence(ctx, &node.children, param_names, si),
        ExprData::ExprLet => {
            marshal_stmt_sequence(ctx, std::slice::from_ref(node), param_names, si)
        }
        _ => marshal_generic(ctx, node, param_names, si),
    }
}

fn marshal_generic(
    ctx: &InterpContext,
    node: &Rc<Node>,
    param_names: &[String],
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> (Value, std::collections::BTreeSet<String>) {
    let name = node_authored_name(node, si);
    let mut edges: Vec<Value> = Vec::with_capacity(node.children.len() + 1);
    let mut refs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if node_references_param(node, &name, param_names) {
        edges.push(edge_positional(ctx, atom_identity_node(ctx, &name)));
    }
    if let Some(ref_name) = node_local_reference_name(node, &name) {
        refs.insert(ref_name);
    }
    for child in node.children.iter() {
        let (child_skel, child_refs) = marshal_skeleton(ctx, child, param_names, si);
        edges.push(edge_positional(ctx, child_skel));
        refs.extend(child_refs);
    }
    if let Some(inner) = node.body.as_ref() {
        let (inner_skel, inner_refs) = marshal_skeleton(ctx, inner, param_names, si);
        edges.push(edge_positional(ctx, inner_skel));
        refs.extend(inner_refs);
    }
    if matches!(node.expr_data.as_ref(), ExprData::ExprLambda) {
        for pname in lambda_param_names_of(node, si) {
            refs.remove(&pname);
        }
    }
    (conj_record(ctx, edges), refs)
}

// A statement sequence -- a block's children, or a single standalone `let` (whose optional
// children[1] is its continuation) -- folded RIGHT-TO-LEFT so the live reference set flows
// from the return (the last statement) back toward each binding. A `let b = rhs` grafts
// `rhs` iff `b` is in the live set accumulated from the statements that follow it (the
// downstream that reaches the return); a dead `let` drops its `rhs`, and because its bound
// name's references are dropped with it, deadness propagates transitively to earlier lets
// that fed only the dead one. A terminal `let` with no continuation is the result position,
// so its value is always grafted (never a false RED on a degenerate body).
fn marshal_stmt_sequence(
    ctx: &InterpContext,
    stmts: &[Rc<Node>],
    param_names: &[String],
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> (Value, std::collections::BTreeSet<String>) {
    let mut edges: Vec<Value> = Vec::new();
    let mut live_refs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (rev_idx, stmt) in stmts.iter().rev().enumerate() {
        let is_terminal = rev_idx == 0;
        match stmt.expr_data.as_ref() {
            ExprData::ExprLet => {
                let bound = node_authored_name(stmt, si);
                let cont = stmt.children.get(1);
                if let Some(c) = cont {
                    let (cont_skel, cont_refs) = marshal_skeleton(ctx, c, param_names, si);
                    edges.push(edge_positional(ctx, cont_skel));
                    live_refs.extend(cont_refs);
                }
                let bound_is_live = !bound.is_empty() && live_refs.contains(&bound);
                // A `_`-prefixed binding (`let _width = width`) is the established declared-inert
                // convention (boundary (4)) applied to a local: the author DELIBERATELY consumes
                // and discards the RHS, so it is a sink, not an accidental dead wire. Graft it so
                // a param flowing only into a `_`-sink stays GREEN, while a normally-named dead
                // `let` (accidental) still drops its RHS and flags its sole-feeding param. A
                // terminal `let` with no continuation is the result position, always grafted.
                let force_live = (is_terminal && cont.is_none()) || bound.starts_with('_');
                if bound_is_live || force_live {
                    if let Some(value) = stmt.children.first() {
                        let (value_skel, value_refs) =
                            marshal_skeleton(ctx, value, param_names, si);
                        edges.push(edge_positional(ctx, value_skel));
                        live_refs.extend(value_refs);
                    }
                }
                if !bound.is_empty() {
                    live_refs.remove(&bound);
                }
            }
            _ => {
                let (stmt_skel, stmt_refs) = marshal_skeleton(ctx, stmt, param_names, si);
                edges.push(edge_positional(ctx, stmt_skel));
                live_refs.extend(stmt_refs);
            }
        }
    }
    (conj_record(ctx, edges), live_refs)
}

// A generic type parameter (`<T>`) and a value parameter (`(xs: List<T>)`) both land in the
// runtime item's `params` (parser: `all_params = concat(type_params, value_params)`). They
// are NOT value inputs and never appear as body value-expressions, so they must be excluded
// from the wiring check. A type parameter is built as `make_param_node(name,
// leaf_type_node(name), ..)` -- its sole type-expr child is a leaf named after the parameter
// itself (`T : T`) -- whereas a value parameter's type-expr names a different type
// (`xs : List`). So: a parameter is a type parameter iff its first child's authored name
// equals its own.
fn param_is_type_param(p: &Rc<Node>, si: &Rc<HashMap<String, Rc<NewlineIndex>>>) -> bool {
    let pname = authored_name_at(si.clone(), p.clone());
    if pname.is_empty() {
        return false;
    }
    match p.children.first() {
        Some(child0) => authored_name_at(si.clone(), child0.clone()) == pname,
        None => false,
    }
}

fn fn_arrow_param_record(ctx: &InterpContext, param_name: &str) -> Value {
    Value::Record {
        type_name: ctx.sym("FnArrowParam"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("name"), Value::Str(param_name.to_string())),
            (ctx.sym("node"), atom_identity_node(ctx, param_name)),
        ])),
    }
}

fn fn_item_param_names(item: &Rc<Node>, si: &SourceIndices) -> Vec<String> {
    let mut param_names = Vec::new();
    for p in item.params.iter() {
        if param_is_type_param(p, si) {
            continue;
        }
        let pn = param_node_name_at(p.clone(), si.clone());
        // A `_`-prefixed name is the established declared-inert convention (e.g.
        // node.dag `step: fn(acc, _edge, sub)`): the author has declared the input
        // genuinely irrelevant, so it is not a dead wire (plan section 4). Skip it.
        if pn.is_empty() || pn.starts_with('_') || param_names.iter().any(|q| q == &pn) {
            continue;
        }
        param_names.push(pn);
    }
    param_names
}

fn fn_arrow_decl_record(
    ctx: &InterpContext,
    si: &SourceIndices,
    module_name: &str,
    name: &str,
    item: &Rc<Node>,
) -> Option<Value> {
    let body = item.body.as_ref()?;
    let param_names = fn_item_param_names(item, si);
    let output = marshal_fn_body_skeleton(ctx, body, &param_names, si);
    let params: Vec<Value> = param_names
        .iter()
        .map(|pn| fn_arrow_param_record(ctx, pn))
        .collect();
    let qualified_name = logical_qualified_name(module_name, name);
    Some(Value::Record {
        type_name: ctx.sym("FnArrowDecl"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("qualified_name"), Value::Str(qualified_name)),
            (ctx.sym("name"), Value::Str(name.to_string())),
            (ctx.sym("output"), output),
            (ctx.sym("params"), crate::v1_interpreter::list_value(params)),
        ])),
    })
}

// Corpus-wide fn/arrow reflection: the gunbc#5364 widen trigger named in
// `v2.lens.wiring_liveness`'s construction_justification. Sibling of
// `eval_concept_decl_facts_live` (which filters to `ItemKind::TypeItem`); this yields
// one `FnArrowDecl` per declared function across every loaded module -- the body
// projected to a reachability skeleton (`output`) plus its declared parameter atoms
// (`params`) -- so the wiring lens folds over REAL fn params corpus-wide, not synthetic
// arrows. Host SOURCE half; dissolves with `concept_decl_facts_live` on the same #5364
// corpus-as-node accessor.
pub fn eval_fn_arrow_decl_facts_live(
    ctx: &InterpContext,
    _args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let si = ctx.source_indices();
    let mut rows: Vec<Value> = Vec::new();
    for_each_live_registry_item(
        ctx,
        |k| k == ItemKind::FnItem || k == ItemKind::FuncItem,
        |module_name, name, item| {
            if let Some(row) = fn_arrow_decl_record(ctx, &si, module_name, name, item) {
                rows.push(row);
            }
            Ok(())
        },
    )?;
    Ok(crate::v1_interpreter::list_value(rows))
}

fn literal_source_lexeme(
    node: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    let index = si.get(&node.span.file)?;
    let text = source_text_at(index.clone(), node.span.clone());
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn data_init_literal_fingerprint(
    body: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    // Encoding must match `v2.std.data_index` (`literal_fingerprint_*_lexeme`).
    let lexeme = literal_source_lexeme(body, si)?;
    match body.expr_data.as_ref() {
        ExprData::ExprLiteral { value, .. } => match value.as_ref() {
            crate::std_syntax::LiteralValue::LitInt { .. }
            | crate::std_syntax::LiteralValue::LitFloat { .. } => Some(format!("num:{lexeme}")),
            crate::std_syntax::LiteralValue::LitBool { .. } => Some(format!("bool:{lexeme}")),
            // Str uses decoded content, not source lexeme: witness RHS is a skeleton atom
            // name (bare content), so this matches `literal_fingerprint_str_content` when the
            // initializer is a plain quoted literal without escapes.
            crate::std_syntax::LiteralValue::LitStr { value: s, .. } => {
                Some(format!("str:\"{s}\""))
            }
            _ => None,
        },
        _ => None,
    }
}

fn data_init_decl_record(
    ctx: &InterpContext,
    si: &SourceIndices,
    module_name: &str,
    name: &str,
    item: &Rc<Node>,
) -> Option<Value> {
    let body = item.body.as_ref()?;
    let literal_fp = data_init_literal_fingerprint(body, si)?;
    let qualified_name = logical_qualified_name(module_name, name);
    Some(Value::Record {
        type_name: ctx.sym("DataInitDecl"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("qualified_name"), Value::Str(qualified_name)),
            (ctx.sym("module"), Value::Str(module_name.to_string())),
            (ctx.sym("name"), Value::Str(name.to_string())),
            (ctx.sym("literal_fp"), Value::Str(literal_fp)),
        ])),
    })
}

// Corpus-wide data-init reflection: sibling of `eval_fn_arrow_decl_facts_live` and
// `eval_concept_decl_facts_live`. Yields one `DataInitDecl` per `ItemKind::DataItem` whose
// initializer is a literal, carrying `literal_fp` for literal-mirror detection in
// `v2.lens.no_dual_representation_test`. Host SOURCE half; dissolves with
// `fn_arrow_decl_facts_live` / `concept_decl_facts_live` on the same gunbc#5364
// corpus-as-node accessor widen trigger named in `v2.lens.wiring_liveness`'s
// construction_justification. Fingerprint encoding spec: `v2.std.data_index`
// (`literal_fingerprint_*_lexeme`); this host block is a transient SOURCE projection on
// the gunbc#5364 corpus-accessor widen — not an independent authority.
pub fn eval_data_init_decl_facts_live(
    ctx: &InterpContext,
    _args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let si = ctx.source_indices();
    let mut rows: Vec<Value> = Vec::new();
    for_each_live_registry_item(
        ctx,
        |k| k == ItemKind::DataItem,
        |module_name, name, item| {
            if let Some(row) = data_init_decl_record(ctx, &si, module_name, name, item) {
                rows.push(row);
            }
            Ok(())
        },
    )?;
    Ok(crate::v1_interpreter::list_value(rows))
}

/// Locked whole-tree declaration-fact carrier (neat-fox-279 / #5966 follow-up).
#[derive(Debug, Clone)]
pub struct DeclFactRaw {
    pub qualified_name: String,
    pub name: String,
    pub kind: ItemKind,
    pub node: Rc<Node>,
    pub rel_path: String,
    pub source_indices: SourceIndices,
}

fn decl_logical_qualified_name(module_name: &str, name: &str) -> String {
    let logical = module_name.strip_prefix("v2.").unwrap_or(module_name);
    if logical.is_empty() {
        name.to_string()
    } else {
        format!("{logical}.{name}")
    }
}

fn extract_module_path_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return Some(trimmed["module ".len()..].trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

fn corpus_dag_files_for_roots(roots: &[String]) -> Vec<PathBuf> {
    let ws = workspace_root();
    let mut files = Vec::new();
    for root in roots {
        let root_path = ws.join(root);
        if root_path.is_dir() {
            collect_dag_files_tolerant(&root_path, &mut files);
        }
    }
    files.sort();
    files
}

/// Whole-tree declaration-fact walk with corpus file counters (parse-only, non-test boundary).
#[derive(Debug, Clone)]
pub struct DeclFactsCorpusWalk {
    pub facts: Vec<DeclFactRaw>,
    /// Non-test `.dag` files visited (includes unparseable and zero-decl files).
    pub files_scanned: usize,
    /// Subset of `files_scanned` that parsed successfully.
    pub files_parsed: usize,
}

/// Parse-only whole-tree `decl_facts(roots)` substrate — shared by host builtin and emit audits.
///
/// Preserves the non-test corpus boundary for emit-only audits: `is_test_dag(rel)` skips
/// before any `DeclFactRaw` row is materialized (same exclusion as the pre-#6158 walk).
pub fn decl_facts_corpus_walk(pool_roots: &[String]) -> DeclFactsCorpusWalk {
    let mut out = Vec::new();
    let mut files_scanned = 0usize;
    let mut files_parsed = 0usize;
    for file in corpus_dag_files_for_roots(pool_roots) {
        let rel = repo_rel(&file);
        if is_test_dag(&rel) {
            continue;
        }
        files_scanned += 1;
        let content = std::fs::read_to_string(&file).ok();
        let module_path = content
            .as_ref()
            .and_then(|c| extract_module_path_from_content(c))
            .unwrap_or_default();
        let Some(parsed) = parse_dag_file(&file) else {
            continue;
        };
        files_parsed += 1;
        let si = parsed.source_indices;
        for item in parsed.items.iter() {
            let name = authored_name_at(si.clone(), item.clone());
            if name.is_empty() {
                continue;
            }
            let kind = item_kind(item.clone());
            out.push(DeclFactRaw {
                qualified_name: decl_logical_qualified_name(&module_path, &name),
                name,
                kind,
                node: item.clone(),
                rel_path: rel.clone(),
                source_indices: si.clone(),
            });
        }
    }
    out.sort_by(|a, b| {
        let kind_ord = |k: ItemKind| match k {
            ItemKind::FnItem => 0,
            ItemKind::FuncItem => 1,
            ItemKind::TypeItem => 2,
            ItemKind::DataItem => 3,
            ItemKind::ServiceItem => 4,
            ItemKind::OtherItem => 5,
        };
        (a.rel_path.as_str(), a.name.as_str(), kind_ord(a.kind)).cmp(&(
            b.rel_path.as_str(),
            b.name.as_str(),
            kind_ord(b.kind),
        ))
    });
    DeclFactsCorpusWalk {
        facts: out,
        files_scanned,
        files_parsed,
    }
}

/// Declaration facts for `roots`; preserves the non-test corpus boundary (delegates to `decl_facts_corpus_walk`).
pub fn decl_facts_for_roots(pool_roots: &[String]) -> Vec<DeclFactRaw> {
    decl_facts_corpus_walk(pool_roots).facts
}

fn marshal_decl_item_kind(ctx: &InterpContext, kind: ItemKind) -> Value {
    let variant = match kind {
        ItemKind::FnItem => "FnItem",
        ItemKind::FuncItem => "FuncItem",
        ItemKind::TypeItem => "TypeItem",
        ItemKind::DataItem => "DataItem",
        ItemKind::ServiceItem => "ServiceItem",
        ItemKind::OtherItem => "OtherItem",
    };
    Value::Variant {
        type_name: ctx.sym("ItemKind"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(vec![]),
    }
}

fn marshal_decl_fact_node(
    ctx: &InterpContext,
    item: &Rc<Node>,
    kind: ItemKind,
    si: &SourceIndices,
) -> InterpResult<Value> {
    match kind {
        ItemKind::TypeItem => concept_decl_node(ctx, si, item),
        ItemKind::FnItem | ItemKind::FuncItem => {
            if let Some(body) = item.body.as_ref() {
                let param_names = fn_item_param_names(item, si);
                Ok(marshal_fn_body_skeleton(ctx, body, &param_names, si))
            } else {
                Ok(unit_type_node(ctx))
            }
        }
        ItemKind::DataItem => {
            if let Some(body) = item.body.as_ref() {
                Ok(marshal_fn_body_skeleton(ctx, body, &[], si))
            } else {
                Ok(unit_type_node(ctx))
            }
        }
        _ => Ok(unit_type_node(ctx)),
    }
}

pub fn eval_decl_facts(ctx: &InterpContext, pool_roots: &[String]) -> InterpResult<Value> {
    let facts = decl_facts_for_roots(pool_roots);
    let mut rows = Vec::with_capacity(facts.len());
    for fact in facts {
        let node = marshal_decl_fact_node(ctx, &fact.node, fact.kind, &fact.source_indices)
            .map_err(|e| InterpError::TypeError {
                msg: format!(
                    "decl_facts: failed to marshal `{}` ({:?}) in `{}`: {e}",
                    fact.qualified_name, fact.kind, fact.rel_path
                ),
            })?;
        rows.push(Value::Record {
            type_name: ctx.sym("DeclFact"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("qualified_name"), Value::Str(fact.qualified_name)),
                (ctx.sym("name"), Value::Str(fact.name)),
                (ctx.sym("kind"), marshal_decl_item_kind(ctx, fact.kind)),
                (ctx.sym("node"), node),
                (ctx.sym("rel_path"), Value::Str(fact.rel_path)),
            ])),
        });
    }
    Ok(crate::v1_interpreter::list_value(rows))
}

fn variant_is_nullary(variant: &Rc<Node>) -> bool {
    variant.children.is_empty()
}

fn nullary_coproduct_variant_value(
    ctx: &InterpContext,
    type_name: &str,
    variant_label: &str,
) -> Value {
    Value::Variant {
        type_name: ctx.sym(type_name),
        variant_name: ctx.sym(variant_label),
        fields: Rc::new(vec![]),
    }
}

pub fn eval_coproduct_nullary_inhabitants(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(
        args.first().map(|(_, v)| v),
        "coproduct_nullary_inhabitants",
    )?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    if item.connective != Connective::Disj {
        return Err(InterpError::TypeError {
            msg: "coproduct_nullary_inhabitants: not a closed coproduct".to_string(),
        });
    }
    let si = ctx.source_indices();
    let mut inhabitants = Vec::with_capacity(item.children.len());
    for variant in item.children.iter() {
        if !variant_is_nullary(variant) {
            return Ok(outcome_rejected_value(
                ctx,
                "coproduct_nullary_inhabitants: payload arm is not nullary",
            ));
        }
        let label = authored_name_at(si.clone(), variant.clone());
        let value = nullary_coproduct_variant_value(ctx, type_name, &label);
        if discriminant_symbol(ctx, &value)? != label {
            return Ok(outcome_rejected_value(
                ctx,
                "coproduct_nullary_inhabitants: discriminant witness failed",
            ));
        }
        inhabitants.push(value);
    }
    Ok(outcome_accepted_list(ctx, inhabitants))
}

fn discriminant_symbol(ctx: &InterpContext, value: &Value) -> InterpResult<String> {
    match value {
        Value::Variant { variant_name, .. } => Ok(ctx.resolve(*variant_name).to_string()),
        Value::Record { type_name, .. } => Ok(ctx.resolve(*type_name).to_string()),
        _ => Err(InterpError::TypeError {
            msg: "discriminant: expected variant or record".to_string(),
        }),
    }
}

fn outcome_accepted_list(ctx: &InterpContext, values: Vec<Value>) -> Value {
    Value::Variant {
        type_name: ctx.sym("Outcome"),
        variant_name: ctx.sym("Accepted"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("value"), crate::v1_interpreter::list_value(values)),
            (
                ctx.sym("diagnostics"),
                crate::v1_interpreter::list_value(vec![]),
            ),
        ])),
    }
}

fn outcome_rejected_value(ctx: &InterpContext, reason: &str) -> Value {
    Value::Variant {
        type_name: ctx.sym("Outcome"),
        variant_name: ctx.sym("Rejected"),
        fields: Rc::new(vec![(
            ctx.sym("diagnostics"),
            crate::v1_interpreter::list_value(vec![Value::Record {
                type_name: ctx.sym("Diagnostic"),
                fields: Rc::new(vec![(ctx.sym("reason"), Value::Str(reason.to_string()))]),
            }]),
        )]),
    }
}
