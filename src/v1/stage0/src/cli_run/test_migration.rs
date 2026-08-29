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

pub(crate) fn test_migration_debt_v1_test_dir() -> PathBuf {
    workspace_root().join("src/v1/tests/src")
}

pub(crate) fn test_migration_named_fn_after_attribute<'a, I>(lines: &mut I) -> Option<String>
where
    I: Iterator<Item = &'a str>,
{
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#[") {
            continue;
        }
        let after_visibility = trimmed.strip_prefix("pub ").unwrap_or(trimmed);
        let after_async = after_visibility
            .strip_prefix("async ")
            .unwrap_or(after_visibility);
        return after_async
            .strip_prefix("fn ")
            .and_then(|rest| rest.split('(').next())
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string);
    }
    None
}

pub(crate) fn test_migration_rust_test_fn_names(content: &str) -> Result<Vec<String>, String> {
    let mut lines = content.lines();
    let mut names = Vec::new();
    while let Some(line) = lines.next() {
        if line.trim() != "#[test]" {
            continue;
        }
        match test_migration_named_fn_after_attribute(&mut lines) {
            Some(name) => names.push(name),
            None => return Err("#[test] is not followed by a named Rust function".to_string()),
        }
    }
    Ok(names)
}

pub(crate) fn test_migration_dag_test_fn_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("test fn ")
                .and_then(|rest| rest.split('(').next())
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string)
        })
        .collect()
}

pub(crate) fn test_migration_behavior_discovery_report(
) -> &'static TestMigrationBehaviorDiscoveryReport {
    static REPORT: OnceLock<TestMigrationBehaviorDiscoveryReport> = OnceLock::new();
    shared_fill::once(
        &REPORT,
        "test_migration_behavior_discovery",
        "corpus",
        build_test_migration_behavior_discovery_report,
    )
}

pub fn test_migration_legacy_behavior_ids() -> Vec<String> {
    test_migration_behavior_discovery_report()
        .legacy_behavior_ids
        .clone()
}

pub fn test_migration_witness_behavior_ids() -> Vec<String> {
    test_migration_behavior_discovery_report()
        .witness_behavior_ids
        .clone()
}

pub fn test_migration_behavior_discovery_holds() -> bool {
    test_migration_behavior_discovery_report().errors.is_empty()
}

pub(crate) fn test_migration_debt_stem(name: &str) -> String {
    let stem = name
        .strip_suffix(".rs")
        .or_else(|| name.strip_suffix(".dag"))
        .unwrap_or(name);
    stem.strip_suffix("_test").unwrap_or(stem).to_string()
}

pub(crate) fn test_migration_debt_floor_stems() -> Vec<String> {
    let mut stems: Vec<String> = corpus_dag_files()
        .into_iter()
        .map(|(path, _)| path)
        .filter(|p| is_test_dag(p))
        .map(|p| {
            let file_name = std::path::Path::new(&p)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&p)
                .to_string();
            test_migration_debt_stem(&file_name)
        })
        .collect();
    stems.sort();
    stems.dedup();
    stems
}

// Exact-stem equality only. A substring match (either direction) was tried and reviewed
// unsound: e.g. v1 stem "pipeline" (the single largest debt module, 418 `#[test]` fns) is a
// substring of the floor stem "typescript_import_pipeline", so a fuzzy match falsely marked
// the whole module covered — hiding debt rather than counting it. Exact equality is decidable
// and cannot understate debt; it may list a module the operator judges topically covered by a
// differently-named floor witness, which is a correct false-debt (never a false-coverage) bias.
pub(crate) fn test_migration_debt_stem_covered(v1_stem: &str, floor_stems: &[String]) -> bool {
    floor_stems.iter().any(|floor_stem| floor_stem == v1_stem)
}

// The DEBT-EXCLUDE set (consumed by `build_test_migration_debt_report`): floor-witness stems
// (migrate path) ∪ retired stems (delete path) ∪ retained-non-migratable stems (retain path). A
// module in any of the three is accounted for and drops out of the debt roster.
// Retired/retained stem sources removed 2026-08-12 (V1-CONSUMER-DRAIN-1): the
// `_retired.dag` / `_retained.dag` carriers they scanned for were deleted with the
// test-suite cutover (gunbc#8146), so both returned empty in-corpus. Behaviour is
// unchanged by construction — extending with two empty vectors was a no-op.
pub(crate) fn test_migration_debt_exclude_stems() -> Vec<String> {
    let mut stems = test_migration_debt_floor_stems();
    stems.sort();
    stems.dedup();
    stems
}

pub(crate) fn test_migration_debt_report() -> &'static TestMigrationDebtReport {
    static REPORT: OnceLock<TestMigrationDebtReport> = OnceLock::new();
    shared_fill::once(
        &REPORT,
        "test_migration_debt",
        "corpus",
        build_test_migration_debt_report,
    )
}

pub fn test_migration_debt_module_names() -> Vec<String> {
    test_migration_debt_report()
        .entries
        .iter()
        .map(|e| e.module.clone())
        .collect()
}
