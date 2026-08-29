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

pub(crate) fn inert_carrier_identifier_tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in line.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub(crate) fn inert_carrier_count_token(text: &str, name: &str) -> i64 {
    let mut n = 0i64;
    for raw in text.lines() {
        for tok in inert_carrier_identifier_tokens(&strip_line_comment(raw)) {
            if tok == name {
                n += 1;
            }
        }
    }
    n
}

pub(crate) fn inert_carrier_type_carrier_blocks(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let Some(rest) = trimmed.strip_prefix("type ") else {
            i += 1;
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut block = String::new();
        block.push_str(lines[i]);
        block.push('\n');
        let mut depth = brace_delta(lines[i]);
        i += 1;
        while i < lines.len() {
            let nt = lines[i].trim_start();
            if depth <= 0 {
                if !(nt.starts_with('|') || nt.starts_with('=')) {
                    break;
                }
            }
            block.push_str(lines[i]);
            block.push('\n');
            depth += brace_delta(lines[i]);
            i += 1;
        }
        out.push((name, block));
    }
    out
}

// A coproduct is consumed through its variant names (constructors, match arms) at least as
// often as through the type name itself, which may appear only at the declaration. Credit
// variant occurrences to the parent type or a live state machine reads as inert. Variant
// names shared across coproducts merge their tallies — an approximation that errs toward
// not flagging; the roster stays the per-name override.
//
// Nominal records (`type Foo = Foo { field: T }`) must NOT be read as a one-variant
// coproduct: the repeated type name after `=` is the record constructor spelling, not a
// variant. Treating it as one double-counts `self_block_refs` and can drive consumption
// to ≤0 for a carrier that is actively constructed in the same file (HostToolchainReach
// falsifier red 2026-07-25 — live consumer in gunbc.host_toolchain_ensure, falsely inert).
pub(crate) fn inert_carrier_variant_names(type_name: &str, block: &str) -> Vec<String> {
    if inert_carrier_block_is_nominal_record(type_name, block) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (idx, raw) in block.lines().enumerate() {
        let t = raw.trim_start();
        let payload = if idx == 0 {
            match t.find('=') {
                Some(p) => &t[p + 1..],
                None => continue,
            }
        } else if let Some(rest) = t.strip_prefix('=') {
            rest
        } else if let Some(rest) = t.strip_prefix('|') {
            rest
        } else {
            continue;
        };
        for seg in payload.split('|') {
            let name: String = seg
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                out.push(name);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(crate) fn inert_carrier_block_is_nominal_record(type_name: &str, block: &str) -> bool {
    let Some(first) = block.lines().next() else {
        return false;
    };
    let trimmed = first.trim_start();
    let Some(eq) = trimmed.find('=') else {
        // `type Foo { ... }` brace-record form — no coproduct variants.
        return trimmed.contains('{');
    };
    if trimmed[eq + 1..].contains('|') {
        return false;
    }
    let after = trimmed[eq + 1..].trim_start();
    after.starts_with(type_name)
        && after
            .as_bytes()
            .get(type_name.len())
            .is_some_and(|b| *b == b'{' || b.is_ascii_whitespace())
        && after.contains('{')
}

pub fn inert_carrier_names_live() -> Vec<String> {
    build_inert_carrier_data().inert_names.clone()
}

pub fn inert_carrier_declared_count_live() -> i64 {
    build_inert_carrier_data().declared_count as i64
}
