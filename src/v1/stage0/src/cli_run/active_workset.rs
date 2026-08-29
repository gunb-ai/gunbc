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

pub fn active_workset_admit(entry: &str, function: &str) {
    let attempt_id = witness_attempt_id(entry, function);
    append_active_workset_phase_journal(
        "admitted",
        &format!("attempt_id={attempt_id} entry={entry} function={function}"),
    );
    let mut g = ACTIVE_WORKSET.lock().unwrap_or_else(|p| p.into_inner());
    g.entries.retain(|e| e.attempt_id != attempt_id);
    g.entries.push(ActiveWorksetEntry {
        attempt_id,
        entry: entry.to_string(),
        function: function.to_string(),
    });
}

pub fn active_workset_complete(entry: &str, function: &str) {
    let attempt_id = witness_attempt_id(entry, function);
    append_active_workset_phase_journal(
        "completed",
        &format!("attempt_id={attempt_id} entry={entry} function={function}"),
    );
    let mut g = ACTIVE_WORKSET.lock().unwrap_or_else(|p| p.into_inner());
    g.entries.retain(|e| e.attempt_id != attempt_id);
}

pub fn active_workset_snapshot() -> Vec<ActiveWorksetEntry> {
    ACTIVE_WORKSET
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .entries
        .clone()
}

/// Test helper: clears the process-wide active-workset registry between discriminating tests.
#[doc(hidden)]
pub fn active_workset_reset_for_test() {
    ACTIVE_WORKSET
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .entries
        .clear();
}
