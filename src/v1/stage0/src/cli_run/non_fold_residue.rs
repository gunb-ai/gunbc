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

pub fn non_fold_residue_wildcard_red_fixture_holds() -> bool {
    let fixture = vec![(
        "m.dag".to_string(),
        "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n"
            .to_string(),
    )];
    nfr_residue_sites(&fixture).contains(&"m.dag::f".to_string())
}

pub fn non_fold_residue_total_fold_green_fixture_holds() -> bool {
    let fixture = vec![(
        "m.dag".to_string(),
        "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    B => false\n    C => false\n  }\n}\n"
            .to_string(),
    )];
    !nfr_residue_sites(&fixture).contains(&"m.dag::f".to_string())
}

pub fn non_fold_residue_roster_red_fixture_holds() -> bool {
    !non_fold_residue_site_is_rostered("synthetic/unrostered_site.dag::would_fail")
}

pub fn non_fold_residue_synthetic_unrostered_red_holds() -> bool {
    let fixture = vec![(
        "synthetic_red_fixture.dag".to_string(),
        "module synthetic_red_fixture\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n"
            .to_string(),
    )];
    let sites = nfr_residue_sites(&fixture);
    let site = "synthetic_red_fixture.dag::f";
    sites.contains(&site.to_string()) && !non_fold_residue_site_is_rostered(site)
}

/// Project the path site keys out of the typed `non_fold_residue_frontier` rows of the
/// `gunbc.non_fold_residue` authority SOURCE TEXT via the real front-end — the roster's
/// re-home off this file's former `NON_FOLD_RESIDUE_ROSTER` const (group-of-units ruling,
/// enrolled in `gunbc.roster_registry`). Per-row reasons and dissolution triggers are
/// `.dag`-side facts the host does not consume. Fail-closed: a parse error, a missing data
/// def, a non-record element, a missing/non-literal `subject.path` field, a duplicate path,
/// or an empty roster is a loud panic, never a silent fallback.
// 🟡 dissolve-on: hand-Rust reader over the `.dag` authority — dissolves with
// `witness_exclusion_rows_from_module_source` when the host consumes an emitted manifest of
// the rows (module-binding supply-carrier pattern), and with the scan below into a pure
// `.dag` Node-tree lens at gunbc#5364.
pub(crate) fn non_fold_residue_units_from_module_source(
    module_rel_path: &str,
    content: &str,
) -> Vec<String> {
    use crate::v1_std_core::{ExprData, LiteralValue};

    let filename = module_rel_path.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), filename.clone());
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let source_indices = std::rc::Rc::new(source_indices);
    let result = crate::v1_compiler_parse::parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "nfr frontier reader: parse error in {module_rel_path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .unwrap_or_else(|| panic!("nfr frontier reader: {module_rel_path} parsed to no module"));
    let data_name = NON_FOLD_RESIDUE_FRONTIER_DATA_NAME;
    for item in module.children.iter() {
        if item.name != data_name
            || !crate::v1_compiler_emit_core_support::is_data_def_item(item.clone())
        {
            continue;
        }
        let body = item.body.as_ref().unwrap_or_else(|| {
            panic!("nfr frontier reader: `data {data_name}` in {module_rel_path} has no value body")
        });
        if !matches!(body.expr_data.as_ref(), ExprData::ExprListLit) {
            panic!(
                "nfr frontier reader: `data {data_name}` in {module_rel_path} is not a list \
                 literal"
            );
        }
        let mut units = Vec::new();
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for el in body.children.iter() {
            if !matches!(el.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
                panic!(
                    "nfr frontier reader: an element of `{data_name}` in {module_rel_path} is \
                     not a record literal (refusing — rows must stay directly host-readable)"
                );
            }
            let mut path: Option<String> = None;
            for field in el.children.iter() {
                let fname = crate::v1_std_core::field_init_node_name_at(
                    field.clone(),
                    source_indices.clone(),
                );
                if fname != "subject" {
                    continue;
                }
                let value = crate::v1_std_core::field_init_node_value(field.clone());
                match value.expr_data.as_ref() {
                    ExprData::ExprRecordLit { .. } => {
                        let variant_name = crate::v1_std_core::authored_name_at(
                            source_indices.clone(),
                            value.clone(),
                        );
                        if variant_name != "PathSubject" {
                            panic!(
                                "nfr frontier reader: `subject` in a `{data_name}` row of \
                                 {module_rel_path} is not PathSubject {{ ... }}"
                            );
                        }
                        let mut row_path: Option<String> = None;
                        for subfield in value.children.iter() {
                            let subname = crate::v1_std_core::field_init_node_name_at(
                                subfield.clone(),
                                source_indices.clone(),
                            );
                            if subname != "path" {
                                continue;
                            }
                            let path_value =
                                crate::v1_std_core::field_init_node_value(subfield.clone());
                            match path_value.expr_data.as_ref() {
                                ExprData::ExprLiteral { value: lit } => match lit.as_ref() {
                                    LiteralValue::LitStr { value: s } => row_path = Some(s.clone()),
                                    _ => panic!(
                                        "nfr frontier reader: `path` in a `{data_name}` row \
                                         of {module_rel_path} is not a string literal"
                                    ),
                                },
                                _ => panic!(
                                    "nfr frontier reader: `path` in a `{data_name}` row of \
                                     {module_rel_path} is not a literal"
                                ),
                            }
                        }
                        path = Some(row_path.unwrap_or_else(|| {
                            panic!(
                                "nfr frontier reader: `subject` in a `{data_name}` row of \
                                 {module_rel_path} is `PathSubject` but carries no `path` field"
                            )
                        }));
                    }
                    _ => panic!(
                        "nfr frontier reader: `subject` in a `{data_name}` row of \
                         {module_rel_path} is not a record literal"
                    ),
                }
            }
            let path = path.unwrap_or_else(|| {
                panic!(
                    "nfr frontier reader: a `{data_name}` row in {module_rel_path} has no \
                     `subject` field"
                )
            });
            if !seen.insert(path.clone()) {
                panic!(
                    "nfr frontier reader: duplicate path {path:?} in `{data_name}` of \
                     {module_rel_path} (the const this replaced tolerated duplicates; the \
                     typed roster refuses them)"
                );
            }
            units.push(path);
        }
        if units.is_empty() {
            panic!(
                "nfr frontier reader: `{data_name}` in {module_rel_path} is empty (fail-closed)"
            );
        }
        return units;
    }
    panic!("nfr frontier reader: no `data {data_name}` def in {module_rel_path}")
}

pub(crate) fn non_fold_residue_roster_entries() -> &'static [String] {
    static ENTRIES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    ENTRIES.get_or_init(|| {
        let path = process_workspace_root().join(NON_FOLD_RESIDUE_AUTHORITY_REL);
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "nfr frontier reader: failed to read {}: {e}",
                path.display()
            )
        });
        non_fold_residue_units_from_module_source(NON_FOLD_RESIDUE_AUTHORITY_REL, &content)
    })
}

pub(crate) fn non_fold_residue_roster_set() -> &'static std::collections::BTreeSet<&'static str> {
    static SET: std::sync::OnceLock<std::collections::BTreeSet<&'static str>> =
        std::sync::OnceLock::new();
    SET.get_or_init(|| {
        non_fold_residue_roster_entries()
            .iter()
            .map(|s| s.as_str())
            .collect()
    })
}

pub fn non_fold_residue_closed_coproduct_type_names() -> &'static std::collections::BTreeSet<String>
{
    &nfr_build_report().closed_coproduct_names
}

pub fn non_fold_residue_count() -> i64 {
    nfr_build_report().sites.len() as i64
}

pub fn non_fold_residue_unrostered_count() -> i64 {
    let roster = non_fold_residue_roster_set();
    nfr_build_report()
        .sites
        .iter()
        .filter(|s| !roster.contains(s.as_str()))
        .count() as i64
}

pub fn non_fold_residue_site_is_rostered(site: &str) -> bool {
    non_fold_residue_roster_set().contains(site)
}

pub fn non_fold_residue_stale_roster_count() -> i64 {
    let live: std::collections::BTreeSet<&str> = nfr_build_report()
        .sites
        .iter()
        .map(|s| s.as_str())
        .collect();
    non_fold_residue_roster_entries()
        .iter()
        .filter(|s| !live.contains(s.as_str()))
        .count() as i64
}

pub fn non_fold_residue_coproduct_universe_count() -> i64 {
    nfr_build_report().coproduct_universe as i64
}
