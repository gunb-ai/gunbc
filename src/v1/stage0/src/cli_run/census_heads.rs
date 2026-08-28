// Split from cli_run.rs (pure code motion; no semantic change).
#![allow(unused_imports)]
use super::*;
use im::HashMap;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::coproduct_reflection::{decl_facts_corpus_walk, DeclFactRaw};
use crate::module_path_index::{
    parse_module_binding, ModuleBindingOutcome, ModuleBindingRefusal, ParsedModuleBinding,
};
use crate::shared_typecheck_store::{self, SharedTypecheckCaches};
use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::{kernel_type_set, SourceSpan};
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::{
    lookup_binding_by_name, lookup_type_by_name, qualified_all_but_last, symbol_index_insert,
    symbol_index_lookup, GlobalBareLookupState, SymbolIndex, TypeEnv,
};
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_infer_lookup::global_bare_callable_node;
use crate::v1_compiler_infer_method::infer_builtin_call_type;
use crate::v1_compiler_infer_sigs::{lookup_resolved_sig, ResolvedFuncEnv, ResolvedFuncSig};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_interpreter::str_value;
use crate::v1_interpreter::Value;
use crate::v1_rt;
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_body, arm_pattern, authored_name_at, block_stmts,
    build_newline_index, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    empty_intern_table, empty_node_list, expr_call_func_at, expr_method_name_at, expr_var_name_at,
    field_access_base, field_access_field_at, field_init_node_name_at, field_init_node_value,
    has_child_named, inferred_to_node, intern, is_discovery_corpus_blocking_diagnostic,
    is_error_diagnostic, is_interpreter_blocking_diagnostic, let_binding_name_at, let_value,
    make_error_node, match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver,
    module_items, no_span, param_node_name_at, param_node_type_expr, Cardinality,
    CompilerDiagnostic, Connective, ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable,
    MatchPattern, NewlineIndex, Node,
};
use serde::Serialize;

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn census_heads_fn_stand_in_for_test() -> Rc<Node> {
    stripped_fn_body_marker()
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn census_heads_module_node_for_test(module: Rc<Node>) -> Rc<Node> {
    census_heads_module_node(module)
}

pub(crate) fn census_heads_children(
    children: &Rc<im::Vector<Rc<Node>>>,
) -> Rc<im::Vector<Rc<Node>>> {
    Rc::new(
        children
            .iter()
            .cloned()
            .map(census_heads_module_item)
            .collect(),
    )
}

/// Fn-decl discriminator for heads-only shrink — must match `local_binding_for_item`'s
/// fn arm (`04_infer.dag`: `NoConnective && body.is_some() && transport.is_none()`).
pub(crate) fn census_heads_item_is_fn_decl(item: &Rc<Node>) -> bool {
    item.connective == Connective::NoConnective && item.body.is_some() && item.transport.is_none()
}

pub(crate) fn census_heads_module_item(item: Rc<Node>) -> Rc<Node> {
    let body = if census_heads_item_is_fn_decl(&item) {
        Some(stripped_fn_body_marker())
    } else {
        None
    };
    let children = if item.children.is_empty() {
        item.children.clone()
    } else {
        census_heads_children(&item.children)
    };
    Rc::new(Node {
        name: item.name.clone(),
        span: item.span.clone(),
        ident_span: item.ident_span.clone(),
        children,
        connective: item.connective.clone(),
        params: item.params.clone(),
        inferred: item.inferred.clone(),
        return_cardinality: item.return_cardinality.clone(),
        uses: empty_node_list(),
        body,
        transport: item.transport.clone(),
        properties: item.properties.clone(),
        type_annotation: item.type_annotation.clone(),
        is_self_recursive: item.is_self_recursive,
        has_non_tail_self_call: item.has_non_tail_self_call,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: item.ident.clone(),
    })
}

pub(crate) fn census_heads_module_node(module: Rc<Node>) -> Rc<Node> {
    Rc::new(Node {
        name: module.name.clone(),
        span: module.span.clone(),
        ident_span: module.ident_span.clone(),
        children: Rc::new(
            module_items(module.clone())
                .iter()
                .cloned()
                .map(census_heads_module_item)
                .collect(),
        ),
        connective: module.connective.clone(),
        params: module.params.clone(),
        inferred: module.inferred.clone(),
        return_cardinality: module.return_cardinality.clone(),
        uses: empty_node_list(),
        body: None,
        transport: module.transport.clone(),
        properties: module.properties.clone(),
        type_annotation: module.type_annotation.clone(),
        is_self_recursive: module.is_self_recursive,
        has_non_tail_self_call: module.has_non_tail_self_call,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: module.ident.clone(),
    })
}

/// One module read BOTH ways and normalized by the census, so the two readings can be
/// compared as an identity rather than described as similar.
///
/// `census_heads_module_node` is applied to both sides. That is what makes the comparison
/// meaningful rather than trivially false: the body slot is the one slot the heads reading
/// deliberately fills differently, the normalizer overwrites it on both sides, and what
/// remains is exactly the declaration heads the pool census consumes. A skip that swallowed
/// a declaration, mis-counted a brace depth, or left the token stream one token off changes
/// the head list and diverges here.
///
/// Both readings start from the SAME intern-table snapshot and neither writes back, so the
/// sides are symmetric — a difference is the reading, never the order they ran in.
pub(crate) fn census_heads_both_readings(
    index: &MultiEntryIndex,
    source: &Rc<v1_compiler_compile::SourceFile>,
) -> (
    (Result<Rc<Node>, String>, u128),
    (Result<Rc<Node>, String>, u128),
) {
    let table = index.intern_table.borrow().clone();
    let read = |heads_only: bool| -> (Result<Rc<Node>, String>, u128) {
        let tokens = v1_compiler_tokenize::tokenize(source.content.clone(), source.path.clone());
        let nl_index = build_newline_index(source.path.clone(), source.content.clone());
        let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
            let mut m = HashMap::new();
            m.insert(source.path.clone(), nl_index);
            m
        });
        // Only the parse is inside the timer: tokenize, newline index and setup are
        // identical work in both readings and sit outside it on purpose.
        let started = std::time::Instant::now();
        let parsed = if heads_only {
            v1_compiler_parse::parse_heads_with_table(tokens, single_si, table.clone())
        } else {
            v1_compiler_parse::parse_with_table(tokens, single_si, table.clone())
        };
        let nanos = started.elapsed().as_nanos();
        if let Some(err) = &parsed.result.error {
            return (Err(diagnostic_to_message(err.diagnostic.clone())), nanos);
        }
        match &parsed.result.module {
            Some(module) => (Ok(census_heads_module_node(module.clone())), nanos),
            None => (Err("no module in parse result".to_string()), nanos),
        }
    };
    (read(false), read(true))
}
