use std::collections::HashMap;
use std::rc::Rc;

use crate::v1_compiler_infer_items::ItemKind;
use crate::v1_interpreter::{
    fields_get, sorted_fields, InterpContext, InterpError, InterpResult, Value,
};
use crate::v1_std_core::{
    authored_name_at, expr_var_name_at, field_node_type_expr, inferred_to_node, param_node_name_at,
    source_text_at, Connective, ExprData, NewlineIndex, Node, VarBindingKind,
};

pub(crate) const NULLARY_PAYLOAD_TYPE_NAME: &str = "coproduct_nullary_payload";

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

fn type_expr_authored_name(ctx: &InterpContext, type_expr: &Rc<Node>) -> String {
    let si = ctx.source_indices();
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

fn marshal_type_expr_ref(ctx: &InterpContext, type_expr: &Rc<Node>) -> InterpResult<Value> {
    let name = type_expr_authored_name(ctx, type_expr);
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

fn marshal_variant_arm_target(ctx: &InterpContext, variant: &Rc<Node>) -> InterpResult<Value> {
    if variant.children.is_empty() {
        return Ok(unit_type_node(ctx));
    }
    let si = ctx.source_indices();
    let mut edges = Vec::with_capacity(variant.children.len());
    for field in variant.children.iter() {
        let field_name = authored_name_at(si.clone(), field.clone());
        let type_expr = field
            .inferred
            .as_ref()
            .and_then(|inf| inferred_to_node(inf.clone()))
            .unwrap_or_else(|| field_node_type_expr(field.clone()));
        let target = marshal_type_expr_ref(ctx, &type_expr)?;
        edges.push(edge_named(ctx, &field_name, target));
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Conj")),
        edges,
    ))
}

pub fn marshal_disj_type_item(ctx: &InterpContext, item: &Rc<Node>) -> InterpResult<Value> {
    if item.connective != Connective::Disj {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: type is not a closed coproduct (Disj)".to_string(),
        });
    }
    let si = ctx.source_indices();
    let mut edges = Vec::with_capacity(item.children.len());
    for child in item.children.iter() {
        let label = authored_name_at(si.clone(), child.clone());
        let target = marshal_variant_arm_target(ctx, child)?;
        edges.push(edge_named(ctx, &label, target));
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Disj")),
        edges,
    ))
}

fn source_text_from_ctx(ctx: &InterpContext, file: &str) -> InterpResult<String> {
    let indices = ctx.source_indices();
    let index = indices.get(file).ok_or_else(|| InterpError::TypeError {
        msg: format!("syntactic coproduct arm keys: no in-memory source for `{file}`"),
    })?;
    let len = index.char_codes.len() as i64;
    Ok(crate::v1_rt::chars_to_string(&index.char_codes, 0, len))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoproductArmSurface {
    pub label: String,
    pub payload_type_name: String,
}

pub fn syntactic_coproduct_arm_labels(
    ctx: &InterpContext,
    file: &str,
    type_name: &str,
) -> InterpResult<Vec<String>> {
    let content = source_text_from_ctx(ctx, file)?;
    extract_type_sum_arm_labels(&content, type_name).map_err(|msg| InterpError::TypeError { msg })
}

pub fn syntactic_coproduct_arm_pairs(
    ctx: &InterpContext,
    file: &str,
    type_name: &str,
) -> InterpResult<Vec<CoproductArmSurface>> {
    let content = source_text_from_ctx(ctx, file)?;
    extract_type_sum_arm_pairs(&content, type_name).map_err(|msg| InterpError::TypeError { msg })
}

fn same_line_leading_comment(s: &str) -> bool {
    match s.find('\n') {
        Some(0) => false,
        Some(nl) => s[..nl].trim().starts_with("//"),
        None => s.trim().starts_with("//"),
    }
}

fn read_identifier_prefix(s: &str) -> Option<(String, &str)> {
    let mut end = 0;
    for (i, ch) in s.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    Some((s[..end].to_string(), &s[end..]))
}

fn skip_braced(s: &str) -> Option<&str> {
    let mut depth = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&s[i + 1..]);
                }
            }
            _ => {}
        }
    }
    None
}

fn payload_type_name_from_rest(rest: &str) -> Result<(String, &str), String> {
    let rest = rest.trim_start();
    if rest.starts_with('{') {
        let after = skip_braced(rest).ok_or_else(|| {
            "syntactic coproduct arm pairs: unclosed `{` in arm payload".to_string()
        })?;
        let payload_len = rest.len() - after.len();
        let payload = rest[..payload_len].trim().to_string();
        Ok((payload, after))
    } else {
        Ok((NULLARY_PAYLOAD_TYPE_NAME.to_string(), rest))
    }
}

fn find_type_decl_start(source: &str, type_name: &str) -> Option<usize> {
    let needle = format!("type {type_name}");
    let mut search_from = 0usize;
    while let Some(rel) = source[search_from..].find(&needle) {
        let start = search_from + rel;
        let after = start + needle.len();
        let boundary_ok = source[after..]
            .chars()
            .next()
            .is_none_or(|c| c.is_whitespace() || c == '=' || c == '{');
        let prefix_ok = start == 0
            || source
                .as_bytes()
                .get(start.saturating_sub(1))
                .is_some_and(|b| b.is_ascii_whitespace());
        if boundary_ok && prefix_ok {
            return Some(start);
        }
        search_from = start + 1;
    }
    None
}

fn extract_type_sum_arm_pairs(
    source: &str,
    type_name: &str,
) -> Result<Vec<CoproductArmSurface>, String> {
    let start = find_type_decl_start(source, type_name).ok_or_else(|| {
        format!("syntactic coproduct arm pairs: `{type_name}` not found in source")
    })?;
    let needle = format!("type {type_name}");
    let after_type = &source[start + needle.len()..];
    let eq_rel = after_type
        .find('=')
        .ok_or_else(|| format!("syntactic coproduct arm pairs: `{type_name}` missing `=`"))?;
    let mut rest = after_type[eq_rel + 1..].trim_start();
    let mut arms = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with('|') {
            rest = rest[1..].trim_start();
        }
        let arm_name = read_identifier_prefix(rest).ok_or_else(|| {
            format!("syntactic coproduct arm pairs: expected arm identifier for `{type_name}`")
        })?;
        rest = arm_name.1;
        if same_line_leading_comment(rest) {
            return Err(format!(
                "syntactic coproduct arm pairs: unexpected `//` comment mid-declaration for `{type_name}`"
            ));
        }
        rest = rest.trim_start();
        let (payload_type_name, after_payload) = payload_type_name_from_rest(rest)?;
        arms.push(CoproductArmSurface {
            label: arm_name.0,
            payload_type_name,
        });
        rest = after_payload.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("//") {
            break;
        }
        if !rest.starts_with('|') {
            let suffix = rest.trim_start();
            if suffix.starts_with("//")
                || suffix.starts_with("type ")
                || suffix.starts_with("fn ")
                || suffix.starts_with("module ")
                || suffix.starts_with("import ")
            {
                break;
            }
            let peek = rest.chars().next().unwrap_or('\0');
            return Err(format!(
                "syntactic coproduct arm pairs: unexpected token `{peek}` mid-declaration for `{type_name}`"
            ));
        }
    }
    if arms.is_empty() {
        return Err(format!(
            "syntactic coproduct arm pairs: `{type_name}` has no arms"
        ));
    }
    Ok(arms)
}

fn extract_type_sum_arm_labels(source: &str, type_name: &str) -> Result<Vec<String>, String> {
    extract_type_sum_arm_pairs(source, type_name)
        .map(|pairs| pairs.into_iter().map(|p| p.label).collect())
}

pub fn eval_resolve_type_node(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "resolve_type_node")?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    marshal_disj_type_item(ctx, item)
}

fn logical_qualified_name(module_name: &str, name: &str) -> String {
    let logical = module_name.strip_prefix("v2.").unwrap_or(module_name);
    if logical.is_empty() {
        name.to_string()
    } else {
        format!("{logical}.{name}")
    }
}

fn concept_decl_node(ctx: &InterpContext, item: &Rc<Node>) -> InterpResult<Value> {
    match item.connective {
        Connective::Disj => marshal_disj_type_item(ctx, item),
        Connective::Conj => marshal_variant_arm_target(ctx, item),
        _ => Ok(unit_type_node(ctx)),
    }
}

pub fn eval_concept_decl_facts_live(
    ctx: &InterpContext,
    _args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let si = ctx.source_indices();
    let mut rows: Vec<Value> = Vec::new();
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
            if info.kind != ItemKind::TypeItem {
                continue;
            }
            let qualified_name = logical_qualified_name(&info.module_name, &name);
            let node = concept_decl_node(ctx, item)?;
            rows.push(Value::Record {
                type_name: ctx.sym("ConceptDecl"),
                fields: Rc::new(sorted_fields(vec![
                    (ctx.sym("qualified_name"), Value::Str(qualified_name)),
                    (ctx.sym("name"), Value::Str(name.clone())),
                    (ctx.sym("node"), node),
                ])),
            });
        }
    }
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
            if info.kind != ItemKind::FnItem && info.kind != ItemKind::FuncItem {
                continue;
            }
            let Some(body) = item.body.as_ref() else {
                continue;
            };
            let mut param_names: Vec<String> = Vec::new();
            for p in item.params.iter() {
                if param_is_type_param(p, &si) {
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
            let output = marshal_fn_body_skeleton(ctx, body, &param_names, &si);
            let params: Vec<Value> = param_names
                .iter()
                .map(|pn| fn_arrow_param_record(ctx, pn))
                .collect();
            let qualified_name = logical_qualified_name(&info.module_name, &name);
            rows.push(Value::Record {
                type_name: ctx.sym("FnArrowDecl"),
                fields: Rc::new(sorted_fields(vec![
                    (ctx.sym("qualified_name"), Value::Str(qualified_name)),
                    (ctx.sym("name"), Value::Str(name.clone())),
                    (ctx.sym("output"), output),
                    (ctx.sym("params"), crate::v1_interpreter::list_value(params)),
                ])),
            });
        }
    }
    Ok(crate::v1_interpreter::list_value(rows))
}

/// Source-text fallback: true iff `param_name` appears as a complete identifier token
/// anywhere in the source text of `body`'s span. Used when the strict skeleton check
/// misses a genuine use because `binding_kind = None` (unresolved inference in whole-tree
/// context). Reading from source is immune to inference state and catches every syntactic
/// use of the parameter name.
fn body_source_mentions_name(
    body: &Rc<Node>,
    param_name: &str,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    let span = body.span.clone();
    let Some(index) = si.get(&span.file) else {
        return false;
    };
    let text = source_text_at(index.clone(), span);
    // Split on any char that cannot appear in a .dag identifier (non-alphanumeric, non-_).
    // If any token equals param_name exactly, the param is used in the body.
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|word| word == param_name)
}

/// Streaming wiring-liveness check over all fn/func items in the context.
///
/// Equivalent to iterating `eval_fn_arrow_decl_facts_live` and running the
/// body-skeleton DFS per param, but processes one fn at a time so peak memory is
/// O(max_skeleton_per_fn) rather than O(sum_of_all_skeletons). The callback
/// `report_dead` is called with `(qualified_name, param_name)` for every dead wire.
/// Returns `(fn_count, dead_wire_count)`.
pub fn check_wiring_liveness_streaming(
    ctx: &InterpContext,
    mut report_dead: impl FnMut(&str, &str),
) -> (usize, usize) {
    let si = ctx.source_indices();
    let mut fn_count = 0usize;
    let mut dead_count = 0usize;

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
            if info.kind != ItemKind::FnItem && info.kind != ItemKind::FuncItem {
                continue;
            }
            let Some(body) = item.body.as_ref() else {
                continue;
            };
            let mut param_names: Vec<String> = Vec::new();
            for p in item.params.iter() {
                if param_is_type_param(p, &si) {
                    continue;
                }
                let pn = param_node_name_at(p.clone(), si.clone());
                if pn.is_empty() || pn.starts_with('_') || param_names.iter().any(|q| q == &pn) {
                    continue;
                }
                param_names.push(pn);
            }
            if param_names.is_empty() {
                continue;
            }
            fn_count += 1;
            let qualified_name = logical_qualified_name(&info.module_name, &name);
            // Build skeleton and check; skeleton is dropped at end of this block.
            let output = marshal_fn_body_skeleton(ctx, body, &param_names, &si);
            for param_name in &param_names {
                if !value_skeleton_contains_atom(ctx, &output, param_name) {
                    // Fallback: in a whole-tree context some ExprVar nodes may have
                    // binding_kind=None (type inference did not fully resolve them).
                    // node_references_param requires Some(LocalValueBinding), so they
                    // don't emit Atom leaves. Do a raw-AST walk; if the param name
                    // appears as an authored ExprVar or ExprCall callee name anywhere
                    // in the body tree, treat the wire as ambiguously live (don't report).
                    if body_source_mentions_name(body, param_name, &si) {
                        continue;
                    }
                    dead_count += 1;
                    report_dead(&qualified_name, param_name);
                }
            }
            // `output` drops here — peak memory is bounded by the largest single skeleton.
        }
    }
    (fn_count, dead_count)
}

/// True iff the body skeleton rooted at `node` contains an Atom with the given
/// identity anywhere in its subtree. Mirrors the `wiring_reach_saturate` reachability
/// check the DAG lens performs, but in Rust without the interpreter fold.
fn value_skeleton_contains_atom(ctx: &InterpContext, node: &Value, param_name: &str) -> bool {
    match node {
        Value::Record { type_name, fields } => {
            if ctx.sym_eq(*type_name, "Node") {
                if let Some(kind) = fields_get(fields, ctx.sym("kind")) {
                    if is_atom_kind_with_identity(ctx, kind, param_name) {
                        return true;
                    }
                }
                if let Some(children) = fields_get(fields, ctx.sym("children")) {
                    return value_skeleton_contains_atom(ctx, children, param_name);
                }
                return false;
            }
            if ctx.sym_eq(*type_name, "Edge") {
                if let Some(target) = fields_get(fields, ctx.sym("target")) {
                    return value_skeleton_contains_atom(ctx, target, param_name);
                }
                return false;
            }
            false
        }
        Value::List(items) => items
            .iter()
            .any(|v| value_skeleton_contains_atom(ctx, v, param_name)),
        _ => false,
    }
}

fn is_atom_kind_with_identity(ctx: &InterpContext, kind: &Value, target_identity: &str) -> bool {
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = kind
    else {
        return false;
    };
    if !ctx.sym_eq(*variant_name, "TypeNode") {
        return false;
    }
    let Some(connective) = fields_get(fields, ctx.sym("connective")) else {
        return false;
    };
    let Value::Variant {
        variant_name: cn_name,
        fields: cn_fields,
        ..
    } = connective
    else {
        return false;
    };
    if !ctx.sym_eq(*cn_name, "Atom") {
        return false;
    }
    let Some(identity) = fields_get(cn_fields, ctx.sym("identity")) else {
        return false;
    };
    match identity {
        Value::Str(s) => s == target_identity,
        _ => false,
    }
}

pub fn eval_syntactic_coproduct_arm_keys(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "syntactic_coproduct_arm_keys")?;
    let (_, file) = type_item_by_name(ctx, type_name)?;
    let labels = syntactic_coproduct_arm_labels(ctx, &file, type_name)?;
    let items: Vec<Value> = labels
        .iter()
        .map(|label| Value::Str(label.clone()))
        .collect();
    Ok(crate::v1_interpreter::list_value(items))
}

pub fn eval_syntactic_coproduct_arm_pairs(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(
        args.first().map(|(_, v)| v),
        "syntactic_coproduct_arm_pairs",
    )?;
    let (_, file) = type_item_by_name(ctx, type_name)?;
    let pairs = syntactic_coproduct_arm_pairs(ctx, &file, type_name)?;
    let items: Vec<Value> = pairs
        .iter()
        .map(|pair| Value::Record {
            type_name: ctx.sym("CoproductArmPayloadPair"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("label"), Value::Str(pair.label.clone())),
                (
                    ctx.sym("payload_type_name"),
                    Value::Str(pair.payload_type_name.clone()),
                ),
            ])),
        })
        .collect();
    Ok(crate::v1_interpreter::list_value(items))
}

pub fn arm_payload_pairs_from_marshaled_node(
    ctx: &InterpContext,
    node: &Value,
) -> InterpResult<Vec<CoproductArmSurface>> {
    let Value::Record { fields, .. } = node else {
        return Err(InterpError::TypeError {
            msg: "expected Node record".to_string(),
        });
    };
    let children =
        fields_get(fields, ctx.sym("children")).ok_or_else(|| InterpError::TypeError {
            msg: "Node missing children".to_string(),
        })?;
    let edges = crate::v1_interpreter::free_monoid_to_vec(children).ok_or_else(|| {
        InterpError::TypeError {
            msg: "children not a list".to_string(),
        }
    })?;
    let mut pairs = Vec::with_capacity(edges.len());
    for edge in edges {
        let Value::Record { fields: ef, .. } = edge else {
            return Err(InterpError::TypeError {
                msg: "expected Edge record".to_string(),
            });
        };
        let label_v = fields_get(&ef, ctx.sym("label")).ok_or_else(|| InterpError::TypeError {
            msg: "Edge missing label".to_string(),
        })?;
        let target = fields_get(&ef, ctx.sym("target")).ok_or_else(|| InterpError::TypeError {
            msg: "Edge missing target".to_string(),
        })?;
        let Value::Variant {
            variant_name,
            fields: lf,
            ..
        } = label_v
        else {
            continue;
        };
        if ctx.resolve(*variant_name) != "Named" {
            continue;
        }
        let Some(Value::Str(label)) = fields_get(&lf, ctx.sym("name")) else {
            continue;
        };
        let payload_type_name = payload_type_name_from_target_node(ctx, target)?;
        pairs.push(CoproductArmSurface {
            label: label.clone(),
            payload_type_name,
        });
    }
    Ok(pairs)
}

fn payload_type_name_from_target_node(ctx: &InterpContext, target: &Value) -> InterpResult<String> {
    let Value::Record { fields, .. } = target else {
        return Err(InterpError::TypeError {
            msg: "expected Node target record".to_string(),
        });
    };
    let kind = fields_get(fields, ctx.sym("kind")).ok_or_else(|| InterpError::TypeError {
        msg: "target missing kind".to_string(),
    })?;
    let children =
        fields_get(fields, ctx.sym("children")).ok_or_else(|| InterpError::TypeError {
            msg: "target missing children".to_string(),
        })?;
    let Value::Variant {
        variant_name: kind_variant,
        fields: kf,
        ..
    } = kind
    else {
        return Err(InterpError::TypeError {
            msg: "expected TypeNode kind".to_string(),
        });
    };
    if ctx.resolve(*kind_variant) != "TypeNode" {
        return Err(InterpError::TypeError {
            msg: "expected TypeNode".to_string(),
        });
    }
    let connective =
        fields_get(&kf, ctx.sym("connective")).ok_or_else(|| InterpError::TypeError {
            msg: "TypeNode missing connective".to_string(),
        })?;
    match connective {
        Value::Variant {
            variant_name: conn, ..
        } if ctx.resolve(*conn) == "Conj" => {
            let edge_items =
                crate::v1_interpreter::free_monoid_to_vec(children).ok_or_else(|| {
                    InterpError::TypeError {
                        msg: "children not list".to_string(),
                    }
                })?;
            if edge_items.is_empty() {
                return Ok(NULLARY_PAYLOAD_TYPE_NAME.to_string());
            }
            let mut parts = Vec::new();
            for edge in edge_items {
                let Value::Record { fields: ef, .. } = edge else {
                    continue;
                };
                let field_name = fields_get(&ef, ctx.sym("label"))
                    .and_then(|label| {
                        let Value::Variant { fields: lf, .. } = label else {
                            return None;
                        };
                        fields_get(&lf, ctx.sym("name")).and_then(|v| match v {
                            Value::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                    })
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "named edge missing label".to_string(),
                    })?;
                let field_target =
                    fields_get(&ef, ctx.sym("target")).ok_or_else(|| InterpError::TypeError {
                        msg: "edge missing target".to_string(),
                    })?;
                let type_name = payload_type_name_from_target_node(ctx, field_target)?;
                parts.push(format!("{field_name}: {type_name}"));
            }
            Ok(format!("{{ {} }}", parts.join(", ")))
        }
        Value::Variant {
            variant_name: conn,
            fields: cf,
            ..
        } if ctx.resolve(*conn) == "Atom" => {
            let Some(Value::Str(name)) = fields_get(&cf, ctx.sym("identity")) else {
                return Err(InterpError::TypeError {
                    msg: "Atom missing identity".to_string(),
                });
            };
            Ok(name.clone())
        }
        _ => Err(InterpError::TypeError {
            msg: "unsupported payload target shape".to_string(),
        }),
    }
}

pub fn arm_labels_from_marshaled_node(
    ctx: &InterpContext,
    node: &Value,
) -> InterpResult<Vec<String>> {
    let Value::Record { fields, .. } = node else {
        return Err(InterpError::TypeError {
            msg: "expected Node record".to_string(),
        });
    };
    let children =
        fields_get(fields, ctx.sym("children")).ok_or_else(|| InterpError::TypeError {
            msg: "Node missing children".to_string(),
        })?;
    let edges = crate::v1_interpreter::free_monoid_to_vec(children).ok_or_else(|| {
        InterpError::TypeError {
            msg: "children not a list".to_string(),
        }
    })?;
    let mut labels = Vec::with_capacity(edges.len());
    for edge in edges {
        let Value::Record { fields: ef, .. } = edge else {
            return Err(InterpError::TypeError {
                msg: "expected Edge record".to_string(),
            });
        };
        let label = fields_get(&ef, ctx.sym("label")).ok_or_else(|| InterpError::TypeError {
            msg: "Edge missing label".to_string(),
        })?;
        let Value::Variant {
            variant_name,
            fields: lf,
            ..
        } = label
        else {
            continue;
        };
        if ctx.resolve(*variant_name) != "Named" {
            continue;
        }
        let Some(Value::Str(name)) = fields_get(&lf, ctx.sym("name")) else {
            continue;
        };
        labels.push(name.clone());
    }
    Ok(labels)
}

pub fn eval_resolve_type_node_with_dropped_last_arm(
    ctx: &InterpContext,
    type_name: &str,
) -> InterpResult<Value> {
    let (item, _) = type_item_by_name(ctx, type_name)?;
    let node = marshal_disj_type_item(ctx, item)?;
    let Value::Record { fields, .. } = node else {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: expected Node record".to_string(),
        });
    };
    let children =
        fields_get(&fields, ctx.sym("children")).ok_or_else(|| InterpError::TypeError {
            msg: "resolve_type_node: Node missing children".to_string(),
        })?;
    let Some(items) = crate::v1_interpreter::free_monoid_to_vec(children) else {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: children not a list".to_string(),
        });
    };
    if items.is_empty() {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: Disj has no arms".to_string(),
        });
    }
    let mut trimmed = items[..items.len() - 1].to_vec();
    if trimmed.is_empty() {
        trimmed = vec![];
    }
    Ok(Value::Record {
        type_name: ctx.sym("Node"),
        fields: Rc::new(sorted_fields(vec![
            (
                ctx.sym("kind"),
                fields_get(&fields, ctx.sym("kind"))
                    .cloned()
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "resolve_type_node: Node missing kind".to_string(),
                    })?,
            ),
            (
                ctx.sym("children"),
                crate::v1_interpreter::list_value(trimmed),
            ),
            (
                ctx.sym("occurrence_id"),
                fields_get(&fields, ctx.sym("occurrence_id"))
                    .cloned()
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "resolve_type_node: Node missing occurrence_id".to_string(),
                    })?,
            ),
        ])),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marshaled_connective_payload_pairs_match_syntactic() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/v2/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let syntactic = extract_type_sum_arm_pairs(&source, "Connective").expect("syntactic pairs");
        assert_eq!(syntactic[0].payload_type_name, "{ identity: Symbol }");
        assert_eq!(syntactic[1].payload_type_name, NULLARY_PAYLOAD_TYPE_NAME);
    }

    #[test]
    fn syntactic_extractor_finds_connective_arms() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/v2/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let arms = extract_type_sum_arm_labels(&source, "Connective").expect("Connective arms");
        assert_eq!(
            arms,
            vec![
                "Atom",
                "Conj",
                "Disj",
                "Arrow",
                "Cardinality",
                "Instantiation"
            ]
        );
    }

    #[test]
    fn syntactic_extractor_finds_behavior_arms() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/v2/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let arms = extract_type_sum_arm_labels(&source, "Behavior").expect("Behavior arms");
        assert_eq!(
            arms,
            vec!["Value", "Transform", "Branch", "Loop", "Bind", "Match"]
        );
    }

    #[test]
    fn syntactic_pair_extractor_nullary_and_typed_connective() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/v2/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let pairs = extract_type_sum_arm_pairs(&source, "Connective").expect("Connective pairs");
        assert_eq!(pairs[0].label, "Atom");
        assert_eq!(pairs[0].payload_type_name, "{ identity: Symbol }");
        assert_eq!(pairs[1].label, "Conj");
        assert_eq!(pairs[1].payload_type_name, NULLARY_PAYLOAD_TYPE_NAME);
        assert_eq!(pairs[2].label, "Disj");
        assert_eq!(pairs[2].payload_type_name, NULLARY_PAYLOAD_TYPE_NAME);
    }

    #[test]
    fn syntactic_pair_extractor_all_nullary_behavior() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/v2/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let pairs = extract_type_sum_arm_pairs(&source, "Behavior").expect("Behavior pairs");
        assert!(pairs
            .iter()
            .all(|p| p.payload_type_name == NULLARY_PAYLOAD_TYPE_NAME));
    }

    #[test]
    fn syntactic_extractor_rejects_connective_prefix_of_longer_type_name() {
        let source = "type ConnectiveCoproductVariant = Foo | Bar\ntype Connective = Atom | Conj\n";
        assert_eq!(
            extract_type_sum_arm_labels(source, "ConnectiveCoproductVariant").expect("variant"),
            vec!["Foo", "Bar"]
        );
        assert_eq!(
            extract_type_sum_arm_labels(source, "Connective").expect("Connective"),
            vec!["Atom", "Conj"]
        );
    }

    #[test]
    fn syntactic_extractor_fails_loud_on_mid_decl_comment() {
        let source = "type Connective = Atom | Conj // trailing\n";
        let err = extract_type_sum_arm_labels(source, "Connective").unwrap_err();
        assert!(
            err.contains("//"),
            "expected mid-decl comment diagnostic, got: {err}"
        );
    }

    #[test]
    fn syntactic_extractor_fails_loud_on_unexpected_token_mid_decl() {
        let source = "type Connective = Atom , Conj\n";
        let err = extract_type_sum_arm_labels(source, "Connective").unwrap_err();
        assert!(
            err.contains("unexpected token"),
            "expected unexpected-token diagnostic, got: {err}"
        );
    }
}
