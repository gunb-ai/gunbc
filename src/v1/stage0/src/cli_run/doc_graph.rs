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

// Memo keyed on the declared input (the extra-roots list) — cache-impurity discipline: a
// roots-blind OnceLock would serve the first caller's report to every later root set.
pub(crate) fn doc_graph_report(extra_roots: &[String]) -> std::sync::Arc<DocGraphReport> {
    static CACHE: OnceLock<std::sync::Mutex<HashMap<Vec<String>, std::sync::Arc<DocGraphReport>>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    let mut guard = cache.lock().expect("doc_graph_report cache poisoned");
    let key = extra_roots.join("\u{1f}");
    if let Some(r) = guard.get(extra_roots) {
        shared_fill::record_hit("doc_graph_report", &key);
        return r.clone();
    }
    shared_fill::begin_fill();
    let start = std::time::Instant::now();
    let report = std::sync::Arc::new(build_doc_graph_report(extra_roots));
    shared_fill::record_fill("doc_graph_report", &key, start.elapsed().as_nanos() as u64);
    guard.insert(extra_roots.to_vec(), report.clone());
    report
}

pub fn doc_graph_orphan_count(extra_roots: Vec<String>) -> i64 {
    doc_graph_report(&extra_roots).orphans.len() as i64
}

pub fn doc_graph_admitted_root_count(extra_roots: Vec<String>) -> i64 {
    doc_graph_report(&extra_roots).admitted_extra_roots as i64
}

pub fn doc_graph_dangling_link_count() -> i64 {
    doc_graph_report(&[]).dangling.len() as i64
}

pub fn doc_graph_doc_count() -> i64 {
    doc_graph_report(&[]).doc_count as i64
}
