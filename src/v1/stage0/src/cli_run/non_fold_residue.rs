// Split from cli_run.rs (pure code motion; no semantic change).
#![allow(unused_imports)]
// cli_run.rs is this module's PARENT, and an `#![allow]` there reaches every module
// under it -- the same cascade this commit removed at the crate root, one level down.
// These are the names its roster carries that this module does not trip, restored to
// warn so `-D warnings` still judges them here. A name moves from this list to the
// allow list above only with a counted site, never silently.
#![warn(
    clippy::assertions_on_constants,
    clippy::clone_on_copy,
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_str_replace,
    clippy::disallowed_macros,
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::enum_variant_names,
    clippy::iter_kv_map,
    clippy::manual_is_multiple_of,
    clippy::manual_strip,
    clippy::map_identity,
    clippy::missing_const_for_thread_local,
    clippy::needless_borrow,
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_char_add_str,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_to_owned,
    clippy::unneeded_struct_pattern,
    clippy::useless_vec,
    dead_code,
    unused_mut
)]

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
    has_child_named, inferred_to_node, intern, is_error_diagnostic,
    is_interpreter_blocking_diagnostic, let_binding_name_at, let_value, make_error_node,
    match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver, module_items, no_span,
    param_node_name_at, param_node_type_expr, Cardinality, CompilerDiagnostic, Connective,
    ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable, MatchPattern, NewlineIndex,
    Node,
};
use serde::Serialize;

/// One co-located row as the identity-join prints it: path::fn, reason ident, dissolution ident.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NfrFrontierTriple {
    pub path: String,
    pub reason: String,
    pub dissolution: String,
}

pub(crate) fn nfr_identity_join_line(t: &NfrFrontierTriple) -> String {
    format!("{}\t{}\t{}", t.path, t.reason, t.dissolution)
}

const NFR_OBSERVING_LAYER_HOST: &str = "dag/gunbc/non_fold_residue.dag";

fn nfr_host_is_forbidden_upstream(host: &str) -> bool {
    host.starts_with("dag/extdeps/") || host.starts_with("src/v2/extdeps/")
}

fn nfr_observing_layer_home_exception(host: &str) -> bool {
    host == NFR_OBSERVING_LAYER_HOST
}

type NfrParsedModule = (
    std::rc::Rc<crate::v1_compiler_parse::ParseResult>,
    std::rc::Rc<HashMap<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>>,
);

fn nfr_parse_module_from_source(
    module_rel_path: &str,
    content: &str,
) -> Result<NfrParsedModule, String> {
    let filename = module_rel_path.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(
        content.to_string(),
        filename.clone(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let source_indices = std::rc::Rc::new(source_indices);
    let result = crate::v1_compiler_parse::parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        return Err(format!(
            "nfr frontier reader: parse error in {module_rel_path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        ));
    }
    Ok((result, source_indices))
}

fn nfr_join_site_key(module_rel_path: &str, authored_path: &str) -> String {
    if authored_path.contains("::") {
        authored_path.to_string()
    } else {
        format!("{module_rel_path}::{authored_path}")
    }
}

fn nfr_expr_ident(
    value: std::rc::Rc<crate::v1_std_core::Node>,
    source_indices: std::rc::Rc<HashMap<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>>,
    field: &str,
    module_rel_path: &str,
) -> Result<String, String> {
    use crate::v1_std_core::{ExprData, LiteralValue};
    match value.expr_data.as_ref() {
        ExprData::ExprVar { .. } | ExprData::ExprCall { .. } => {
            Ok(crate::v1_std_core::authored_name_at(source_indices, value))
        }
        ExprData::ExprLiteral { value: lit } => match lit.as_ref() {
            LiteralValue::LitStr { value: s } => Ok(s.clone()),
            _ => Err(format!(
                "nfr frontier reader: `{field}` in a `{NON_FOLD_RESIDUE_FRONTIER_DATA_NAME}` row of \
                 {module_rel_path} is not an ident"
            )),
        },
        _ => Err(format!(
            "nfr frontier reader: `{field}` in a `{NON_FOLD_RESIDUE_FRONTIER_DATA_NAME}` row of \
             {module_rel_path} is not an ident"
        )),
    }
}

pub(crate) fn nfr_content_declares_frontier(content: &str) -> bool {
    content.contains(NON_FOLD_RESIDUE_FRONTIER_DATA_NAME)
}

/// Project the path site keys out of a `non_fold_residue_frontier` list of DIRECT record
/// literals via the real front-end. Per-row reasons and dissolution triggers are `.dag`-side
/// facts the host does not consume for the walk. Fail-closed: a parse error, a missing value
/// body, a non-record element, a missing/non-literal `subject.path` field, a duplicate path,
/// an empty list, or (when Required) a row whose path prefix is not this file is a typed
/// `Err` naming the file. A module with no decl is zero rows (`Ok([])`). An empty list decl
/// is dead data (DESIGN §4c): refuse it and say to delete the decl.
// 🟡 dissolve-on: hand-Rust reader over the `.dag` authority — dissolves with
// `witness_exclusion_rows_from_module_source` when the host consumes an emitted manifest of
// the rows (module-binding supply-carrier pattern), and with the scan below into a pure
// `.dag` Node-tree lens at gunbc#5364. v1 seed-reader change is admissible because it
// unblocks residue burn-down for the v2 program.
pub(crate) fn non_fold_residue_triples_from_module_source_with(
    module_rel_path: &str,
    content: &str,
) -> Result<Vec<NfrFrontierTriple>, String> {
    if !nfr_content_declares_frontier(content) {
        return Ok(Vec::new());
    }
    non_fold_residue_units_from_parse_source(module_rel_path, content)
}

#[cfg(test)]
pub(crate) fn non_fold_residue_units_from_module_source_with(
    module_rel_path: &str,
    content: &str,
) -> Result<Vec<String>, String> {
    Ok(
        non_fold_residue_triples_from_module_source_with(module_rel_path, content)?
            .into_iter()
            .map(|t| t.path)
            .collect(),
    )
}

fn non_fold_residue_units_from_parse_source(
    module_rel_path: &str,
    content: &str,
) -> Result<Vec<NfrFrontierTriple>, String> {
    use crate::v1_std_core::{ExprData, LiteralValue};

    let (result, source_indices) = nfr_parse_module_from_source(module_rel_path, content)?;
    let module = result
        .module
        .as_ref()
        .ok_or_else(|| format!("nfr frontier reader: {module_rel_path} parsed to no module"))?;
    let data_name = NON_FOLD_RESIDUE_FRONTIER_DATA_NAME;
    let mut found = None;
    for item in module.children.iter() {
        if item.module_item_kind != crate::v1_std_core::ParsedModuleItemKind::ModuleItemDataValue {
            continue;
        }
        if item.name.contains(data_name) && item.name != data_name {
            return Err(format!(
                "nfr frontier reader: near-miss data decl `{}` in {module_rel_path} \
                 (only `{data_name}` hosts rows; a similarly spelled data name is refused, \
                 not skipped)",
                item.name
            ));
        }
        if item.name == data_name {
            found = Some(item);
        }
    }
    let Some(item) = found else {
        return Ok(Vec::new());
    };
    let body = item.body.as_ref().ok_or_else(|| {
        format!("nfr frontier reader: `data {data_name}` in {module_rel_path} has no value body")
    })?;
    if !matches!(body.expr_data.as_ref(), ExprData::ExprListLit) {
        return Err(format!(
            "nfr frontier reader: `data {data_name}` in {module_rel_path} is not a list \
                 literal"
        ));
    }
    let mut units = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for el in body.children.iter() {
        if !matches!(el.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
            return Err(format!(
                "nfr frontier reader: an element of `{data_name}` in {module_rel_path} is \
                     not a record literal (refusing — rows must stay directly host-readable)"
            ));
        }
        let mut path: Option<String> = None;
        let mut reason: Option<String> = None;
        let mut dissolution: Option<String> = None;
        for field in el.children.iter() {
            let fname =
                crate::v1_std_core::field_init_node_name_at(field.clone(), source_indices.clone());
            let value = crate::v1_std_core::field_init_node_value(field.clone());
            if fname == "reason" {
                reason = Some(nfr_expr_ident(
                    value,
                    source_indices.clone(),
                    "reason",
                    module_rel_path,
                )?);
                continue;
            }
            if fname == "dissolution" {
                dissolution = Some(nfr_expr_ident(
                    value,
                    source_indices.clone(),
                    "dissolution",
                    module_rel_path,
                )?);
                continue;
            }
            if fname != "subject" {
                continue;
            }
            match value.expr_data.as_ref() {
                ExprData::ExprRecordLit { .. } => {
                    let variant_name =
                        crate::v1_std_core::authored_name_at(source_indices.clone(), value.clone());
                    if variant_name != "PathSubject" {
                        return Err(format!(
                            "nfr frontier reader: `subject` in a `{data_name}` row of \
                                 {module_rel_path} is not PathSubject {{ ... }}"
                        ));
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
                                _ => {
                                    return Err(format!(
                                        "nfr frontier reader: `path` in a `{data_name}` row \
                                         of {module_rel_path} is not a string literal"
                                    ))
                                }
                            },
                            _ => {
                                return Err(format!(
                                    "nfr frontier reader: `path` in a `{data_name}` row of \
                                     {module_rel_path} is not a literal"
                                ))
                            }
                        }
                    }
                    path = Some(row_path.ok_or_else(|| {
                        format!(
                            "nfr frontier reader: `subject` in a `{data_name}` row of \
                                 {module_rel_path} is `PathSubject` but carries no `path` field"
                        )
                    })?);
                }
                _ => {
                    return Err(format!(
                        "nfr frontier reader: `subject` in a `{data_name}` row of \
                         {module_rel_path} is not a record literal"
                    ))
                }
            }
        }
        let authored_path = path.ok_or_else(|| {
            format!(
                "nfr frontier reader: a `{data_name}` row in {module_rel_path} has no \
                     `subject` field"
            )
        })?;
        if nfr_host_is_forbidden_upstream(module_rel_path) {
            return Err(format!(
                "nfr frontier reader: `{data_name}` in {module_rel_path} stores repository \
                 coverage state in an upstream module; the observing layer is \
                 {NFR_OBSERVING_LAYER_HOST}"
            ));
        }
        if authored_path.contains("::") {
            let prefix = authored_path.split("::").next().unwrap_or("");
            if prefix != module_rel_path && !nfr_observing_layer_home_exception(module_rel_path) {
                return Err(format!(
                        "nfr frontier reader: row path {authored_path:?} in {module_rel_path} is not at \
                         home (subject file prefix {prefix:?} must equal the hosting module \
                         path {module_rel_path:?})"
                    ));
            }
        }
        let path = nfr_join_site_key(module_rel_path, &authored_path);
        if !seen.insert(path.clone()) {
            return Err(format!(
                "nfr frontier reader: duplicate path {path:?} in `{data_name}` of \
                     {module_rel_path} (the const this replaced tolerated duplicates; the \
                     typed roster refuses them)"
            ));
        }
        let reason = reason.ok_or_else(|| {
            format!(
                "nfr frontier reader: a `{data_name}` row in {module_rel_path} has no \
                     `reason` ident"
            )
        })?;
        let dissolution = dissolution.ok_or_else(|| {
            format!(
                "nfr frontier reader: a `{data_name}` row in {module_rel_path} has no \
                     `dissolution` ident"
            )
        })?;
        units.push(NfrFrontierTriple {
            path,
            reason,
            dissolution,
        });
    }
    if units.is_empty() {
        return Err(format!(
            "nfr frontier reader: `data {data_name}` in {module_rel_path} is empty; delete the decl \
             (a module with no decl already contributes zero rows)"
        ));
    }
    Ok(units)
}

fn nfr_workspace_carrier_files(root: &Path) -> Result<Vec<(String, String)>, String> {
    let mut acquired: Vec<crate::cli_run::derived_row_roster::AcquiredDag> = Vec::new();
    collect_dag_files(&root.join("dag"), &mut acquired);
    collect_dag_files(&root.join("src/v2"), &mut acquired);
    let mut files = Vec::new();
    for item in acquired {
        if item.is_derived() {
            continue;
        }
        let content = item.read().map_err(|e| {
            format!(
                "nfr frontier reader: failed to read {}: {e}",
                item.path().display()
            )
        })?;
        let rel = item
            .path()
            .strip_prefix(root)
            .unwrap_or(item.path())
            .to_string_lossy()
            .replace('\\', "/");
        if !nfr_content_declares_frontier(&content) {
            continue;
        }
        files.push((rel, content));
    }
    Ok(files)
}

pub(crate) fn nfr_identity_join_from_files(
    files: &[(String, String)],
) -> Result<Vec<NfrFrontierTriple>, String> {
    let mut all = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (rel, content) in files {
        if !nfr_content_declares_frontier(content) {
            continue;
        }
        for triple in non_fold_residue_triples_from_module_source_with(rel, content)? {
            if !seen.insert(triple.path.clone()) {
                return Err(format!(
                    "nfr frontier reader: duplicate path {:?} across co-located \
                     `{NON_FOLD_RESIDUE_FRONTIER_DATA_NAME}` lists (first collision involving \
                     {rel}; {})",
                    triple.path,
                    nfr_identity_join_line(&triple)
                ));
            }
            all.push(triple);
        }
    }
    if all.is_empty() {
        return Err(format!(
            "nfr frontier reader: no `data {NON_FOLD_RESIDUE_FRONTIER_DATA_NAME}` rows in the \
             corpus (fail-closed)"
        ));
    }
    Ok(all)
}

pub(crate) fn non_fold_residue_identity_join_triples(
) -> Result<&'static [NfrFrontierTriple], String> {
    static TRIPLES: std::sync::OnceLock<Result<Vec<NfrFrontierTriple>, String>> =
        std::sync::OnceLock::new();
    match TRIPLES.get_or_init(|| {
        nfr_workspace_carrier_files(&process_workspace_root())
            .and_then(|files| nfr_identity_join_from_files(&files))
    }) {
        Ok(v) => Ok(v.as_slice()),
        Err(e) => Err(e.clone()),
    }
}

pub(crate) fn non_fold_residue_roster_entries() -> Result<&'static [String], String> {
    static ENTRIES: std::sync::OnceLock<Result<Vec<String>, String>> = std::sync::OnceLock::new();
    match ENTRIES.get_or_init(|| {
        Ok(non_fold_residue_identity_join_triples()?
            .iter()
            .map(|t| t.path.clone())
            .collect())
    }) {
        Ok(v) => Ok(v.as_slice()),
        Err(e) => Err(e.clone()),
    }
}

pub(crate) fn non_fold_residue_roster_set() -> Result<&'static BTreeSet<String>, String> {
    static SET: std::sync::OnceLock<Result<BTreeSet<String>, String>> = std::sync::OnceLock::new();
    match SET.get_or_init(|| Ok(non_fold_residue_roster_entries()?.iter().cloned().collect())) {
        Ok(s) => Ok(s),
        Err(e) => Err(e.clone()),
    }
}

pub fn non_fold_residue_closed_coproduct_type_names() -> &'static std::collections::BTreeSet<String>
{
    nfr_closed_coproduct_name_set()
}

pub fn non_fold_residue_site_is_rostered(site: &str) -> Result<bool, String> {
    Ok(non_fold_residue_roster_set()?.contains(site))
}
