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

/// The frozen legacy path-deferral population, read from the one `.dag` authority by the same
/// interim source scan as the sibling rosters (`CLI_RUN_WITNESS_ADMISSION_SOURCE_SCAN_SCAFFOLD_MARKER`).
///
/// Fail-closed like its siblings: a `FrozenPathDeferral {` head that does not parse PANICS with
/// its location rather than yielding no key, because a silently-dropped freeze row would refuse a
/// legitimately frozen witness — and a silently-*added* one is impossible since keys only come
/// from parsed heads.
pub(crate) fn frozen_path_deferral_keys() -> &'static [String] {
    static KEYS: OnceLock<Vec<String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let content =
            std::fs::read_to_string(workspace_root().join(WITNESS_DEFERRAL_FREEZE_AUTHORITY_REL))
                .unwrap_or_else(|e| {
                    panic!(
                        "witness deferral freeze: failed to read {WITNESS_DEFERRAL_FREEZE_AUTHORITY_REL}: {e}"
                    )
                });
        frozen_path_deferral_keys_from_source(&content)
    })
}

pub(crate) fn frozen_path_deferral_rows_from_source(content: &str) -> Vec<(String, Vec<String>)> {
    const HEAD: &str = "FrozenPathDeferral {";
    let mut rows: Vec<(String, Vec<String>)> = Vec::new();
    let mut cursor = 0usize;
    while let Some(offset) = content[cursor..].find(HEAD) {
        let at = cursor + offset;
        cursor = at + HEAD.len();
        // `type FrozenPathDeferral {` is the declaration, not a row.
        if content[..at].trim_end().ends_with("type") {
            continue;
        }
        let rest = &content[cursor..];
        let entry = quoted_after_field(rest, "entry:").unwrap_or_else(|| {
            panic!(
                "witness deferral freeze: FrozenPathDeferral head at byte {at} of \
                 {WITNESS_DEFERRAL_FREEZE_AUTHORITY_REL} has no parseable `entry:` literal"
            )
        });
        let list = bracket_list_after_field(rest, "functions:").unwrap_or_else(|| {
            panic!(
                "witness deferral freeze: FrozenPathDeferral row `{entry}` in \
                 {WITNESS_DEFERRAL_FREEZE_AUTHORITY_REL} has no parseable `functions:` list"
            )
        });
        let functions = quoted_literals(&list);
        if functions.is_empty() {
            panic!(
                "witness deferral freeze: FrozenPathDeferral row `{entry}` declares an empty \
                 function list — a freeze row covering nothing is a row that should be deleted"
            );
        }
        rows.push((entry, functions));
    }
    rows
}

pub(crate) fn frozen_path_deferral_keys_from_source(content: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for (entry, functions) in frozen_path_deferral_rows_from_source(content) {
        for function in functions {
            let key = witness_admission_manifest_key(&entry, &function);
            if !keys.iter().any(|k| k == &key) {
                keys.push(key);
            }
        }
    }
    keys
}

/// The same freeze rows, joined against the ENROLLED-ROSTER identity space instead of the
/// admission-key space: `module.function` (as `floor_expected_red_roster` writes it) rather
/// than `entry::function` (as `witness_admission_manifest_key` writes it). Two different keys
/// over the same two facts, kept as two functions rather than one that returns both — a caller
/// asking "is this frozen?" wants the admission key; a caller asking "is this row also known-red?"
/// wants the qualified name, and conflating them would let a shape mismatch hide a real miss.
///
/// The module name is read from each entry file's own `module ...` line (`extract_module_path`),
/// the same authority `run_required_floor` reads for every other qualified name it reports — not
/// re-derived from the path string, which would silently diverge from the interpreter's own
/// binding the day a file's module line stops matching its directory.
pub(crate) fn frozen_path_deferral_qualified_identities_from_source(
    content: &str,
    root: &std::path::Path,
) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (entry, functions) in frozen_path_deferral_rows_from_source(content) {
        let file_content = match std::fs::read_to_string(root.join(&entry)) {
            Ok(c) => c,
            // A frozen row naming a file the tree no longer carries is stale-path debt, not a
            // roster-intersection question — `collect_stale_frozen_path_deferrals` already owns
            // that disposition, so this join skips it rather than panicking a second authority.
            Err(_) => continue,
        };
        let module = match extract_module_path(&file_content) {
            Some(m) => m,
            None => continue,
        };
        for function in functions {
            out.push((entry.clone(), format!("{module}.{function}")));
        }
    }
    out
}

/// The pure fold the gate is built from — the grain the fixture controls plant into.
pub(crate) fn frozen_path_deferral_additions_in(
    current: &[String],
    base: &[String],
) -> Vec<String> {
    current
        .iter()
        .filter(|k| !base.iter().any(|b| b == *k))
        .cloned()
        .collect()
}

pub fn complexity_linearity_audit_corpus_over_decl_facts(
    roots: &[String],
) -> ComplexityLinearityAuditSummary {
    let walk = decl_facts_corpus_walk(roots);
    let mut summary = ComplexityLinearityAuditSummary::default();
    summary.files_scanned = walk.files_scanned;
    summary.files_parsed = walk.files_parsed;

    for fact in &walk.facts {
        if !matches!(fact.kind, ItemKind::FnItem | ItemKind::FuncItem) {
            continue;
        }
        summary.fns_scanned += 1;
        summary
            .findings
            .extend(cla_audit_decl_fact(fact, &fact.source_indices));
    }
    summary.findings.sort();
    summary
}

pub fn complexity_linearity_audit_corpus_parse_only(
    roots: &[String],
) -> ComplexityLinearityAuditSummary {
    complexity_linearity_audit_corpus_over_decl_facts(roots)
}

pub fn complexity_linearity_audit_corpus_default_roots() -> ComplexityLinearityAuditSummary {
    complexity_linearity_audit_corpus_parse_only(&witness_layer_roots())
}

pub fn complexity_linearity_syntactic_finding_count() -> i64 {
    cla_cached_builtin_cache().finding_count
}

pub fn complexity_linearity_syntactic_site_fired(site: &str) -> bool {
    cla_cached_builtin_cache().sites.contains(site)
}

pub fn complexity_linearity_wildcard_facts() -> &'static [ComplexityLinearityWildcardFactRaw] {
    &cla_cached_wildcard_facts().facts
}

// --- Fallback-arm census (W2, fallback-arm-census (plan doc deleted 2026-08-28)) ---
// Per-arm structural wildcard walk over decl_facts. Classifies each MatchPattern::Wildcard
// arm into exactly one of: refuses | completes_closed_total | declared_interim |
// answers_on_open | unknown. Completeness: class counts sum to structural wildcard-arm
// count (authority). Text `_ =>` grep is a measured over-approx, not the reconciliation
// denominator. Zero production behavior change — register only.
