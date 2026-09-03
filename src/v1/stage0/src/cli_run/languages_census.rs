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

pub(crate) fn languages_census_collect_source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            languages_census_collect_source_files(&path, out);
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "dag" || ext == "rs" {
                out.push(path);
            }
        }
    }
}

pub(crate) fn languages_census_strip_content(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            while chars.next().is_some_and(|ch| ch != '\n') {}
            out.push('\n');
            continue;
        }
        if c == '"' {
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    chars.next();
                    continue;
                }
                if ch == '"' {
                    break;
                }
            }
            out.push(' ');
            continue;
        }
        if c == '`' {
            while chars.next().is_some_and(|ch| ch != '`') {}
            out.push(' ');
            continue;
        }
        out.push(c);
    }
    out
}

pub(crate) fn languages_census_extract_data_decl_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("data ")?;
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

pub(crate) fn languages_census_is_infrastructure_path(rel: &str) -> bool {
    rel.starts_with("src/v2/test/claim/languages_consumer_census/")
        || rel == "src/v2/lens/languages_consumer_census.dag"
}

pub(crate) fn languages_census_tokenize(content: &str) -> HashSet<String> {
    let stripped = languages_census_strip_content(content);
    stripped
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

pub(crate) fn languages_census_record_tokens(
    rel: &str,
    content: &str,
    decl_name_set: &HashSet<String>,
    by_decl: &mut HashMap<String, HashSet<String>>,
) {
    if rel == LANGUAGES_AUTHORITY_REL || languages_census_is_infrastructure_path(rel) {
        return;
    }
    let tokens = languages_census_tokenize(content);
    for decl_name in tokens.intersection(decl_name_set) {
        by_decl
            .get_mut(decl_name)
            .expect("decl map key")
            .insert(rel.to_string());
    }
}

pub(crate) fn languages_decl_records_from_inventory(
    inventory: &[PreparedSourceView],
) -> Vec<LanguagesDeclConsumerRecord> {
    let authority_content = inventory
        .iter()
        .find(|e| e.source.path.replace('\\', "/") == LANGUAGES_AUTHORITY_REL)
        .map(|e| e.source.content.as_str())
        .unwrap_or_else(|| {
            panic!(
                "languages_consumer_census: prepared inventory missing {LANGUAGES_AUTHORITY_REL}"
            )
        });
    let decl_names = languages_census_extract_data_decl_names(authority_content);
    let decl_name_set: HashSet<String> = decl_names.iter().cloned().collect();

    let mut by_decl: HashMap<String, HashSet<String>> = decl_names
        .iter()
        .map(|name| (name.clone(), HashSet::new()))
        .collect();

    let mut seen: HashSet<String> = HashSet::new();
    for entry in inventory {
        let rel = entry.source.path.replace('\\', "/");
        if !rel.starts_with("dag/") && !rel.starts_with("src/") {
            continue;
        }
        seen.insert(rel.clone());
        languages_census_record_tokens(&rel, &entry.source.content, &decl_name_set, &mut by_decl);
    }

    // Prepared inventory is dag + src/v2 `.dag` only. The disk census also tokenizes
    // `src/v1` and every `.rs` file; those are the external consumers of `rust_spec`.
    let ws = workspace_root();
    let mut extra = Vec::new();
    let src_root = ws.join("src");
    if src_root.is_dir() {
        languages_census_collect_source_files(&src_root, &mut extra);
    }
    for path in extra {
        let rel = path
            .strip_prefix(&ws)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if seen.contains(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        languages_census_record_tokens(&rel, &content, &decl_name_set, &mut by_decl);
    }

    let mut records = Vec::new();
    for decl_name in decl_names {
        let mut paths: Vec<String> = by_decl
            .remove(&decl_name)
            .expect("decl map key")
            .into_iter()
            .collect();
        paths.sort();
        records.push(LanguagesDeclConsumerRecord {
            decl_name,
            external_consumer_paths: paths,
        });
    }
    records
}

pub(crate) fn languages_decl_records_inner() -> Vec<LanguagesDeclConsumerRecord> {
    if floor_prepared_authority_active() {
        return FLOOR_LANGUAGES_RECORDS.with(|cell| {
            if cell.borrow().is_none() {
                let inventory = floor_prepared_inventory_snapshot()
                    .expect("floor languages census: authority active but inventory missing");
                *cell.borrow_mut() = Some(languages_decl_records_from_inventory(&inventory));
            }
            cell.borrow().clone().expect("floor languages records")
        });
    }
    let ws = workspace_root();
    let authority = ws.join(LANGUAGES_AUTHORITY_REL);
    let authority_content = std::fs::read_to_string(&authority).unwrap_or_else(|e| {
        panic!(
            "languages_consumer_census: failed to read {}: {e}",
            authority.display()
        )
    });
    let decl_names = languages_census_extract_data_decl_names(&authority_content);
    let decl_name_set: HashSet<String> = decl_names.iter().cloned().collect();

    let mut files = Vec::new();
    for tree in &["dag", "src"] {
        let root = ws.join(tree);
        if root.is_dir() {
            languages_census_collect_source_files(&root, &mut files);
        }
    }

    let mut by_decl: HashMap<String, HashSet<String>> = decl_names
        .iter()
        .map(|name| (name.clone(), HashSet::new()))
        .collect();

    for path in files {
        let rel = path
            .strip_prefix(&ws)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if rel == LANGUAGES_AUTHORITY_REL || languages_census_is_infrastructure_path(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        languages_census_record_tokens(&rel, &content, &decl_name_set, &mut by_decl);
    }

    let mut records = Vec::new();
    for decl_name in decl_names {
        let mut paths: Vec<String> = by_decl
            .remove(&decl_name)
            .expect("decl map key")
            .into_iter()
            .collect();
        paths.sort();
        records.push(LanguagesDeclConsumerRecord {
            decl_name,
            external_consumer_paths: paths,
        });
    }
    records
}

static LANGUAGES_DECL_RECORDS: OnceLock<Vec<LanguagesDeclConsumerRecord>> = OnceLock::new();

pub(crate) fn languages_decl_records_cached() -> &'static [LanguagesDeclConsumerRecord] {
    LANGUAGES_DECL_RECORDS.get_or_init(languages_decl_records_inner)
}

/// Whether the process-wide census has already been built — PROVENANCE for the floor's
/// preparation warm (`FloorPreparationPhase` `LanguagesConsumerCensusBuild`), so a warm that
/// finds the artifact already in hand reports `already-warm-on-entry` instead of a zero build.
pub(crate) fn languages_decl_records_already_built() -> bool {
    LANGUAGES_DECL_RECORDS.get().is_some()
}

/// Whether the floor's prepared subject carries the authority the census reads. The inventory
/// form panics on its absence by design (a census with no authority has no rows to answer for),
/// so the preparation warm asks first and skips, printed, rather than stopping a subject that
/// carries no consumer of the census at all.
pub(crate) fn languages_census_subject_carries_authority() -> bool {
    floor_prepared_inventory_snapshot().is_some_and(|inventory| {
        inventory
            .iter()
            .any(|e| e.source.path.replace('\\', "/") == LANGUAGES_AUTHORITY_REL)
    })
}

pub(crate) fn languages_decl_record_for(
    decl_name: &str,
) -> Option<&'static LanguagesDeclConsumerRecord> {
    languages_decl_records_cached()
        .iter()
        .find(|r| r.decl_name == decl_name)
}

pub fn languages_consumer_census_data_decl_count() -> i64 {
    languages_decl_records_cached().len() as i64
}

pub fn languages_consumer_census_per_language_row_count() -> i64 {
    languages_decl_records_cached()
        .iter()
        .filter(|r| !r.decl_name.ends_with("_format"))
        .count() as i64
}

pub fn languages_consumer_census_format_row_count() -> i64 {
    languages_decl_records_cached()
        .iter()
        .filter(|r| r.decl_name.ends_with("_format"))
        .count() as i64
}

pub fn languages_consumer_census_external_consumer_count(decl_name: String) -> i64 {
    languages_decl_record_for(&decl_name)
        .map(|r| r.external_consumer_paths.len() as i64)
        .unwrap_or(-1)
}

pub fn languages_consumer_census_is_composition_only(decl_name: String) -> bool {
    languages_decl_record_for(&decl_name)
        .map(|r| r.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

pub fn languages_consumer_census_has_external_consumer(decl_name: String) -> bool {
    languages_decl_record_for(&decl_name)
        .map(|r| !r.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

// --- Inert carrier census (folded from inert_carrier_project.rs) ---
//
// A type carrier is "inert" iff (a) declared in a non-test file, (b) its name appears in at least
// one *_test.dag file (self-tested), and (c) its name appears in NO non-test .dag file outside its
// own declaration block (zero real consumer). This is DESIGN §5 coverage-by-illusion.
// DISSOLUTION TRIGGER: when .dag gains compile-graph / reference-edge access (gunbc#5364), the
// token scan folds into a pure .dag reader over BindsTo edges and this Rust census deletes.

/// Wall time for the languages census over prepared inventory (floor path).
pub fn languages_decl_records_inventory_wall_ms(inventory: &[PreparedSourceView]) -> u128 {
    let started = std::time::Instant::now();
    languages_decl_records_from_inventory(inventory);
    started.elapsed().as_millis()
}

/// Wall time for the languages census filesystem scan (legacy path).
pub fn languages_decl_records_disk_scan_wall_ms() -> u128 {
    let started = std::time::Instant::now();
    languages_decl_records_inner();
    started.elapsed().as_millis()
}
