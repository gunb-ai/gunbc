// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 18 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::disallowed_macros,  // 14
    clippy::manual_unwrap_or,  // 1
    dead_code,  // 3
    unused_imports,  // 0 -- pre-existing
)]
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

/// Live compile-clean pipeline module paths for census exclusion silent-loss checks.
/// Shard entry paths plus their import closures — modules the compile-clean gate may touch.
pub fn compile_clean_live_pipeline_module_paths() -> Vec<String> {
    let pool_roots = default_source_roots();
    let facts = build_module_graph_facts_live(&pool_roots);
    let mut paths = BTreeSet::new();
    let roster = compile_clean_shard_entry_paths_fast().unwrap_or_else(|reason| {
        panic!("compile_clean_live_pipeline_module_paths: {reason}");
    });
    for entry in roster {
        for path in import_closure_live_paths_with_facts(&entry, &facts) {
            paths.insert(path);
        }
    }
    paths.into_iter().collect()
}

pub(crate) fn compile_clean_resolve_has_hard_errors(
    result: &v1_compiler_compile::ResolvedPipelineResult,
) -> bool {
    compile_clean_pipeline_has_hard_errors(result.diagnostics.as_ref())
}

pub fn compile_clean_unlisted_import_use_blocks_from_policy() -> Result<bool, String> {
    let roots = default_source_roots();
    let entry = resolve_entry_file_under_roots(&roots, COMPILE_CLEAN_DIAGNOSTIC_POLICY_ENTRY)
        .map_err(|e| format!("compile_clean_diagnostic_policy resolve: {e}"))?;
    let sources =
        policy_entry_closure_sources(&roots, &entry, "gunbc.compile_clean_diagnostic_policy")?;
    let (graph, indices) = resolved_graph_from_sources(sources, ResolveTypecheckGate::Strict)
        .map_err(|e| format!("compile_clean_diagnostic_policy resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
    match v1_interpreter::run_in_context_with_args(
        &ctx,
        "compile_clean_unlisted_import_use_blocks",
        &[],
        false,
    ) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "compile_clean_unlisted_import_use_blocks returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("compile_clean_unlisted_import_use_blocks: {e}")),
    }
}

pub(crate) fn compile_clean_unlisted_import_use_blocks_cached() -> Result<bool, String> {
    thread_local! {
        static CACHED: RefCell<Option<Result<bool, String>>> = const { RefCell::new(None) };
        static LOGGED_REFUSAL: Cell<bool> = const { Cell::new(false) };
    }
    CACHED.with(|c| {
        if let Some(v) = c.borrow().clone() {
            return v;
        }
        let v = compile_clean_unlisted_import_use_blocks_from_policy();
        if let Err(ref e) = v {
            LOGGED_REFUSAL.with(|logged| {
                if !logged.get() {
                    eprintln!(
                        "compile-clean policy: refused to read disposition row ({e}); failing gate"
                    );
                    logged.set(true);
                }
            });
        }
        *c.borrow_mut() = Some(v.clone());
        v
    })
}

pub(crate) fn compile_clean_policy_read_refuses_gate() -> bool {
    compile_clean_unlisted_import_use_blocks_cached().is_err()
}

/// Single authority (DESIGN.md §3/§7): whether a diagnostic blocks compile-clean.
/// `UnlistedImportUse` is governed by `gunbc.compile_clean_diagnostic_policy` (issue 11);
/// all other classes delegate to `00_core.dag` `is_interpreter_blocking_diagnostic`.
pub fn compile_clean_diagnostic_is_hard(d: &Rc<ErrorNode>) -> bool {
    use crate::v1_std_core::CompilerDiagnostic;
    match d.diagnostic.as_ref() {
        CompilerDiagnostic::UnlistedImportUse { .. } => {
            match compile_clean_unlisted_import_use_blocks_cached() {
                Ok(blocks) => blocks,
                Err(_) => true,
            }
        }
        _ => crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone()),
    }
}

pub fn compile_clean_pipeline_has_hard_errors(diagnostics: &im::Vector<Rc<ErrorNode>>) -> bool {
    if compile_clean_policy_read_refuses_gate() {
        return true;
    }
    diagnostics.iter().any(compile_clean_diagnostic_is_hard)
}

/// `ResolvedPipelineResult` / `im::Vector` adapter for compile-clean checks.
pub fn compile_clean_im_vector_has_hard_errors(diagnostics: &im::Vector<Rc<ErrorNode>>) -> bool {
    if compile_clean_policy_read_refuses_gate() {
        return true;
    }
    diagnostics.iter().any(compile_clean_diagnostic_is_hard)
}

pub(crate) fn compile_clean_all_touched_paths_docs_universe(
    touched_paths: &[String],
) -> Result<bool, String> {
    call_compile_clean_bool_list_fn(
        "compile_clean_all_touched_paths_docs_universe",
        "touched_paths",
        touched_paths,
    )
}

pub(crate) fn compile_clean_all_touched_paths_selectable(
    touched_paths: &[String],
) -> Result<bool, String> {
    call_compile_clean_bool_list_fn(
        "compile_clean_all_touched_paths_selectable",
        "touched_paths",
        touched_paths,
    )
}

pub(crate) fn compile_clean_departed_paths_outside_docs(
    departed_paths: &HashSet<String>,
) -> Result<bool, String> {
    let paths: Vec<String> = departed_paths.iter().cloned().collect();
    call_compile_clean_bool_list_fn(
        "compile_clean_departed_paths_outside_docs",
        "departed_paths",
        &paths,
    )
}

/// Module paths in the compile-clean closure (format-independent identity — see
/// `gunbc compile` census_only_sources wiring in main.rs).
pub(crate) fn compile_clean_closure_module_paths(
    compiled: &[Rc<v1_compiler_compile::SourceFile>],
) -> HashSet<String> {
    compiled
        .iter()
        .filter_map(|s| extract_module_path(&s.content))
        .collect()
}

/// Indexed pool modules outside the compile closure enter the name census only
/// (fill = whole tree; policy gates lookup, never fill).
pub(crate) fn compile_clean_census_only_sources_for_compiled(
    index: &MultiEntryIndex,
    compiled: &[Rc<v1_compiler_compile::SourceFile>],
) -> Vec<Rc<v1_compiler_compile::SourceFile>> {
    let closure_modules = compile_clean_closure_module_paths(compiled);
    let mut pool_rest: Vec<(String, Rc<v1_compiler_compile::SourceFile>)> = index
        .source_files
        .iter()
        .filter(|(module_path, _)| !closure_modules.contains(*module_path))
        .map(|(module_path, source)| (module_path.clone(), source.clone()))
        .collect();
    pool_rest.sort_by(|a, b| a.0.cmp(&b.0));
    pool_rest.into_iter().map(|(_, source)| source).collect()
}

/// Parse-grade census fill (annotation binding + parse errors). Used for
/// out-of-closure modules (#8204) and must run independently of semantic
/// resolve — a resolve refusal must not hide the annotation population.
pub(crate) fn compile_clean_census_fill_hard_diagnostics(
    census_only: &[Rc<v1_compiler_compile::SourceFile>],
) -> im::Vector<Rc<ErrorNode>> {
    if census_only.is_empty() {
        return im::Vector::new();
    }
    let fill = v1_compiler_compile::parse_census_fill_sources(Rc::new(census_only.to_vec().into()));
    fill.diagnostics
        .iter()
        .filter(|d| compile_clean_diagnostic_is_hard(d))
        .cloned()
        .collect()
}

pub(crate) fn compile_clean_pipeline_options_for_sources(
    index: Option<&MultiEntryIndex>,
    compiled: &[Rc<v1_compiler_compile::SourceFile>],
) -> Rc<v1_compiler_compile::CompilePipelineOptions> {
    let census_only = index
        .map(|idx| compile_clean_census_only_sources_for_compiled(idx, compiled))
        .unwrap_or_default();
    if census_only.is_empty() {
        return v1_compiler_compile::default_compile_pipeline_options();
    }
    eprintln!(
        "[census] {} indexed modules outside the compile-clean closure enter the name census only (not compiled)",
        census_only.len()
    );
    Rc::new(v1_compiler_compile::CompilePipelineOptions {
        analyze_complexity: false,
        census_only_sources: Rc::new(census_only.into()),
    })
}

pub(crate) fn compile_clean_scope_plan_from_touched_paths(
    touched_paths: &[String],
    departed_paths: &HashSet<String>,
) -> Result<CompileCleanScopePlan, String> {
    let roots = default_source_roots();
    let (graph, indices) = resolve_entry_graph_shared(&roots, COMPILE_CLEAN_SCOPE_ENTRY)
        .map_err(|e| format!("dag_compile_clean_scope resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let paths: Vec<v1_interpreter::Value> =
        touched_paths.iter().map(|s| str_value(s.clone())).collect();
    let mut departed_sorted: Vec<&String> = departed_paths.iter().collect();
    departed_sorted.sort();
    let departed: Vec<v1_interpreter::Value> = departed_sorted
        .into_iter()
        .map(|s| str_value(s.clone()))
        .collect();
    let args = [
        (
            Some("touched_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("departed_paths".to_string()),
            list_value_from_vec(departed),
        ),
    ];
    let result = v1_interpreter::run_in_context_with_args(
        &ctx,
        "compile_clean_scope_disposition_from_diff",
        &args,
        false,
    )
    .map_err(|e| format!("compile_clean_scope_disposition_from_diff: {e}"))?;
    match &result {
        v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "ScopedRun") => {
            let entry_paths = match ctx.field(fields, "entry_paths") {
                Some(v) => string_list_from_value(v, "entry_paths")?,
                None => return Err("ScopedRun missing `entry_paths`".to_string()),
            };
            Ok(CompileCleanScopePlan::Scoped { entry_paths })
        }
        v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "SkipNoAffectedEntries") => {
            let reason = match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => r.to_string(),
                _ => "no compile-clean entry affected".to_string(),
            };
            Ok(CompileCleanScopePlan::SkipNoAffected { reason })
        }
        v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RequireWholeTree") => {
            let reason = match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => r.to_string(),
                _ => "whole-tree baseline required".to_string(),
            };
            eprintln!("compile-clean scope: {reason}");
            Ok(CompileCleanScopePlan::WholeTree)
        }
        v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RefuseShardRosterDuplicate") => {
            let reason = match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => r.to_string(),
                _ => {
                    return Err(
                        "RefuseShardRosterDuplicate missing `reason` string field".to_string(),
                    )
                }
            };
            eprintln!("compile-clean scope: refused ({reason})");
            Ok(CompileCleanScopePlan::Refused { reason })
        }
        other => Err(format!(
            "compile_clean_scope_disposition_from_diff returned `{}`, expected ScopedRun | SkipNoAffectedEntries | RequireWholeTree | RefuseShardRosterDuplicate",
            ctx.format_value(other)
        )),
    }
}

/// `gunbc.ci_layer_roots.compile_clean_source_roots` — witness pool + `src/v1` for cross-tree
/// import resolution in compile-clean scope disposition (not the gate receipt pool).
pub(crate) fn compile_clean_source_roots() -> Vec<String> {
    let mut roots = witness_layer_roots();
    if !roots.iter().any(|r| r == "src/v1") {
        roots.push("src/v1".to_string());
    }
    roots
}

/// Host realization of `tools.dag_compile_clean_shard_roster.compile_clean_shard_entry_paths`
/// without resolving `dag_compile_clean_scope.dag` (the interpreter path cold-scans ~minutes).
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_COMPILE_CLEAN_SHARD_ENTRY_PATHS_FAST_SCAFFOLD_MARKER` / §7):
/// routes shard roster construction through `std.keyed_roster.keyed_roster_build` on the floor
/// CI hot path — same authority as the modeled `compile_clean_shard_entry_paths_from`, not a
/// parallel duplicate-key policy. Duplicate path keys refuse (terminal `Refused` disposition),
/// never sort/dedup absorption.
/// Entry roots are ALL of `witness_layer_roots` — the same tree the whole-tree gate
/// compiles — mirroring `tools.dag_compile_clean_partition.compile_clean_partition_boundary`
/// (see its note: a roster that is a strict subset of the compiled tree both widened
/// src/v2-only diffs to whole-tree and left affected src/v2 importers unselected on
/// scoped runs).
pub(crate) fn compile_clean_shard_entry_paths_from_decl_facts(
    decl_facts: &[ModuleDeclarationFactRaw],
) -> Result<Vec<String>, String> {
    let incomings: Rc<im::Vector<Rc<KeyedRow<String, ModuleDeclarationFactRaw>>>> = Rc::new(
        decl_facts
            .iter()
            .map(|decl| {
                let path = workspace_relative_repo_path(&decl.path);
                Rc::new(KeyedRow {
                    row_key: path,
                    value: decl.clone(),
                    _phantom: std::marker::PhantomData,
                })
            })
            .collect(),
    );
    match keyed_roster_build(incomings, |a: String, b: String| a == b).as_ref() {
        KeyedRosterBuild::KeyedRosterBuilt { rows } => {
            Ok(rows.iter().map(|row| row.row_key.clone()).collect())
        }
        KeyedRosterBuild::KeyedRosterBuildDuplicateKey { key, .. } => Err(format!(
            "shard roster construction refused duplicate path key at admission: {key}"
        )),
    }
}

pub(crate) fn compile_clean_shard_entry_paths_fast() -> Result<Vec<String>, String> {
    let entry_roots: Vec<String> = witness_layer_roots()
        .iter()
        .map(|root| anchor_source_root(root))
        .collect();
    compile_clean_shard_entry_paths_from_decl_facts(&module_declaration_facts(&entry_roots))
}

/// Floor CI hot path: mirrors `compile_clean_scope_disposition_from_diff`
/// (`tools.dag_compile_clean_scope`, module-graph import-closure grain — channel 2 of
/// operator fork (c) 2026-07-10) without the Wet interpreter fold over
/// `compile_clean_shard_entry_paths()`. Selection reuses the SAME certified realization
/// as the discovery-corpus channel (`entry_file_touched_via_import_closure`); every arm
/// that cannot answer falls back to the gate's whole-tree baseline, loudly.
///
/// Roster construction matches `compile_clean_scope_disposition_from_diff`: build and
/// validate the keyed shard roster before any disposition arm, so duplicate path keys
/// refuse even when the diff would otherwise skip or widen to whole-tree.
pub(crate) fn compile_clean_scope_plan_from_touched_paths_floor_fast(
    touched_paths: &[String],
    departed_paths: &HashSet<String>,
) -> CompileCleanScopePlan {
    compile_clean_scope_plan_from_touched_paths_floor_fast_impl(
        touched_paths,
        departed_paths,
        compile_clean_shard_entry_paths_fast(),
    )
}

pub(crate) fn compile_clean_scope_plan_from_touched_paths_floor_fast_impl(
    touched_paths: &[String],
    departed_paths: &HashSet<String>,
    roster: Result<Vec<String>, String>,
) -> CompileCleanScopePlan {
    let roster = match roster {
        Ok(paths) => paths,
        Err(reason) => {
            eprintln!("compile-clean scope: refused ({reason})");
            return CompileCleanScopePlan::Refused { reason };
        }
    };

    if touched_paths.is_empty() {
        // Mirrors `compile_clean_scope_disposition_probe`'s empty arm (#7412): an
        // observation that saw nothing is indistinguishable from one that could not
        // observe, so the whole tree is the only sound baseline. A main-push squash
        // merge lands here, which is what makes main-push a real cold control.
        eprintln!(
            "compile-clean scope: empty touched-path set — whole-tree baseline (diff observed nothing, or could not observe)"
        );
        return CompileCleanScopePlan::WholeTree;
    }

    match compile_clean_all_touched_paths_docs_universe(touched_paths) {
        Ok(true) => {
            let reason =
                "docs-only diff — no compile-clean entry selection required (Ruling 1 path grain)"
                    .to_string();
            eprintln!("compile-clean scope: skipped ({reason})");
            return CompileCleanScopePlan::SkipNoAffected { reason };
        }
        Ok(false) => {}
        Err(msg) => {
            return CompileCleanScopePlan::Refused { reason: msg };
        }
    }

    match compile_clean_all_touched_paths_selectable(touched_paths) {
        Ok(true) => {}
        Ok(false) => {
            eprintln!(
                "compile-clean scope: touched path outside the selectable universe — compiler/infra change, whole-tree baseline"
            );
            return CompileCleanScopePlan::WholeTree;
        }
        Err(msg) => {
            return CompileCleanScopePlan::Refused { reason: msg };
        }
    }

    match compile_clean_departed_paths_outside_docs(departed_paths) {
        Ok(true) => {
            eprintln!(
                "compile-clean scope: departed non-docs path in diff (deletion/rename) — whole-tree baseline"
            );
            return CompileCleanScopePlan::WholeTree;
        }
        Ok(false) => {}
        Err(msg) => {
            return CompileCleanScopePlan::Refused { reason: msg };
        }
    }

    let pool_roots = compile_clean_source_roots();
    let facts = build_module_graph_facts_live(&pool_roots);
    let declared_paths = facts.declared_repo_paths();
    let mut affected = Vec::new();
    for entry_path in roster {
        match entry_file_touched_via_import_closure(
            &entry_path,
            &facts,
            &declared_paths,
            touched_paths,
        ) {
            Ok(true) => affected.push(entry_path),
            Ok(false) => {}
            Err(msg) => {
                eprintln!("compile-clean scope: {msg} — whole-tree baseline");
                return CompileCleanScopePlan::WholeTree;
            }
        }
    }
    if !affected.is_empty() {
        eprintln!(
            "compile-clean scope: {} affected entr{} (floor fast path)",
            affected.len(),
            if affected.len() == 1 { "y" } else { "ies" }
        );
        return CompileCleanScopePlan::Scoped {
            entry_paths: affected,
        };
    }
    eprintln!(
        "compile-clean scope: non-empty diff with no shard intersection — whole-tree baseline"
    );
    CompileCleanScopePlan::WholeTree
}

pub(crate) fn compile_clean_scoping_active() -> bool {
    FLOOR_COMPILE_CLEAN_CI_SCOPING.load(Ordering::SeqCst)
        || std::env::var("GUNBC_CI_DIFF_BASE").is_ok()
        || std::env::var("GITHUB_ACTIONS")
            .map(|v| v == "true")
            .unwrap_or(false)
        || std::env::var("CI").map(|v| v == "true").unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Class B import-closure gate affected-set skip (#7835).
//
// `run_class_b_import_closure_gate` costs ~2.3 min wall per cold run; skip when the
// merge-base diff is provably disjoint from the gate's input closure (declared-import
// pool ∪ witness layer ∪ perturbation fixtures ∪ gate transport modules). Same shape as
// regen_floor_skip_label_for_ci: skip only on a
// non-empty diff proven disjoint; run on empty diff, departed non-docs paths, and any
// observation/closure failure (fail-closed — regen shape: still RUN the gate, but the two
// failure arms carry grep-countable labels distinct from structural run_class_b_gate).
// Gated to pull_request events — push-to-main runs the full gate as the cold control.
// ---------------------------------------------------------------------------

pub(crate) fn compile_clean_scope_plan_for_ci() -> CompileCleanScopePlan {
    // Falsifier cold-control arm: force the whole-tree compile before any diff observation.
    // Widen-to-more-checking only — this env can never skip or narrow the gate, so it is a
    // control, not an escape hatch (the deterministic whole-tree counterpart to the scoped
    // per-PR admission, on the falsifier cadence).
    if std::env::var("GUNBC_CI_COMPILE_CLEAN_COLD_CONTROL")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        eprintln!(
            "compile-clean scope: whole-tree cold control forced (GUNBC_CI_COMPILE_CLEAN_COLD_CONTROL=1)"
        );
        return CompileCleanScopePlan::WholeTree;
    }
    if !compile_clean_scoping_active() {
        eprintln!("compile-clean scope: whole-tree (ci diff scoping inactive)");
        return CompileCleanScopePlan::WholeTree;
    }
    match floor_git_diff_name_status_range() {
        Ok((changed_paths, departed_paths)) => {
            if FLOOR_COMPILE_CLEAN_CI_SCOPING.load(Ordering::SeqCst) {
                return compile_clean_scope_plan_from_touched_paths_floor_fast(
                    &changed_paths,
                    &departed_paths,
                );
            }
            match compile_clean_scope_plan_from_touched_paths(&changed_paths, &departed_paths) {
                Ok(plan) => plan,
                Err(msg) => CompileCleanScopePlan::Refused {
                    reason: format!("compile-clean scope disposition failed: {msg}"),
                },
            }
        }
        Err(msg) => CompileCleanScopePlan::Refused {
            reason: format!("diff observation failed: {msg}"),
        },
    }
}

/// Whole-tree `--target dag` compile-clean (witness_layer_roots closure).
/// Instrument path for diagnostic histogram — not for cargo tests.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_COMPILE_CLEAN_DIAGNOSTIC_HISTOGRAM_SCAFFOLD_MARKER` / §7):
/// dissolves when ROADMAP §1 namespace-only lane closes (import strip + global_bare wiring fixed)
/// or a floor-enrolled diagnostic-histogram lens subsumes this host transport.
/// Uses the same resolve kernel as `witness_layer_roots_compile_clean_check`
/// (`compile_to_resolved` on the whole-tree source closure).
pub fn compile_clean_whole_tree_hard_diagnostics() -> Result<im::Vector<Rc<ErrorNode>>, String> {
    let plan = CompileCleanScopePlan::WholeTree;
    let sources = match witness_layer_roots_compile_clean_sources_for_plan(&plan)? {
        None => return Err("compile-clean whole-tree: no sources (unexpected skip)".to_string()),
        Some(s) => s,
    };
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
    Ok(result
        .diagnostics
        .iter()
        .filter(|d| compile_clean_diagnostic_is_hard(d))
        .cloned()
        .collect())
}

pub(crate) fn compile_clean_whole_tree_resolved(
) -> Result<Rc<v1_compiler_compile::ResolvedPipelineResult>, String> {
    let plan = CompileCleanScopePlan::WholeTree;
    let sources = match witness_layer_roots_compile_clean_sources_for_plan(&plan)? {
        None => return Err("compile-clean whole-tree: no sources (unexpected skip)".to_string()),
        Some(s) => s,
    };
    Ok(v1_compiler_compile::compile_to_resolved(Rc::new(
        sources.into(),
    )))
}

fn unlisted_import_rows_from_resolved(
    result: &v1_compiler_compile::ResolvedPipelineResult,
) -> Result<Vec<UnlistedImportCensusRow>, String> {
    use crate::v1_std_core::CompilerDiagnostic;
    let graph = result
        .graph
        .clone()
        .ok_or_else(|| "compile-clean census: compilation produced no graph".to_string())?;
    let mut rows = Vec::new();
    for d in result.diagnostics.iter() {
        let CompilerDiagnostic::UnlistedImportUse { name, .. } = d.diagnostic.as_ref() else {
            continue;
        };
        let (binding_source, definer_module) =
            classify_unlisted_import_binding_source(&graph, &d.module_name, name);
        rows.push(UnlistedImportCensusRow {
            file: diagnostic_decl_file_for_census(d),
            position: diagnostic_to_span(d.diagnostic.clone()).start,
            referenced_name: name.clone(),
            referencing_module: d.module_name.clone(),
            definer_module,
            binding_source,
        });
    }
    rows.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.position.cmp(&b.position))
            .then_with(|| a.referenced_name.cmp(&b.referenced_name))
            .then_with(|| a.referencing_module.cmp(&b.referencing_module))
    });
    Ok(rows)
}

/// Disposition of one observed diagnostic class, carried as an ATTRIBUTE of the census row —
/// never a filter on whether the row exists. This is the raw-census rework's central move
/// (review 2026-09-09, finding 1): the census this replaced partitioned by severity before
/// counting, so a class whose disposition changed vanished from the instrument exactly when the
/// change happened, and the zero assertion went vacuous over the rows that mattered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticCensusDisposition {
    Advisory,
    Blocking,
}

/// One diagnostic class counted over the whole-tree compile-clean closure, BEFORE any severity
/// decision. Mirrors `gunbc.target_binding` `RawDiagnosticClassObservation`.
///
/// THE ROW KEY IS (CLASS, DISPOSITION), NOT CLASS ALONE. Disposition is a per-INSTANCE fact:
/// `diagnostic_disposition` decides `WhereRefinementUnenforced` by the instance's reason (six
/// advisory reason strings; any other reason blocks), so one class can carry instances on both
/// sides of the partition. A mixed class produces one row per disposition rather than a single
/// row carrying a fabricated class-level answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDiagnosticClassObservation {
    pub class_name: String,
    pub disposition: DiagnosticCensusDisposition,
    pub diagnostics: usize,
    pub distinct_modules: usize,
    pub distinct_positions: usize,
}

/// The counting state for one (class, disposition) row: the running diagnostic count plus the
/// distinct module and position sets the row's `distinct_*` counts are derived from.
type ClassAccumulation = (usize, BTreeSet<String>, BTreeSet<String>);

/// The observation's provenance: five digests that each close one way the reading could be about
/// something else. Mirrors `gunbc.target_binding` `CompileCleanDiagnosticCensusIdentity`, whose
/// comment block is the authority on what each digest binds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCleanDiagnosticCensusIdentity {
    pub source_vector_digest: String,
    pub compiler_executable_digest: String,
    pub resolver_policy_digest: String,
    pub diagnostic_class_schema_digest: String,
    pub closure_digest: String,
}

/// The per-class RAW diagnostic census: one entry per diagnostic class the compiler emitted over
/// the same whole-tree closure the floor gate compiles, counted before severity partitioning,
/// plus the binding-source-attributed UnlistedImportUse rows (the burndown worklist for the
/// class whose promotion is staged in `gunbc.compile_clean_diagnostic_policy`).
#[derive(Debug, Clone)]
pub struct CompileCleanDiagnosticCensus {
    pub identity: CompileCleanDiagnosticCensusIdentity,
    pub closure_modules: usize,
    pub raw_diagnostics: usize,
    pub classes: Vec<RawDiagnosticClassObservation>,
    pub unlisted_import_rows: Vec<UnlistedImportCensusRow>,
}

/// The census's typed refusal causes, one per way it can fail to be the authority it claims.
/// Mirrors `gunbc.target_binding` `CompileCleanDiagnosticCensusRefusalCause`; the payloads are
/// the located detail DESIGN section 5 requires of every refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileCleanDiagnosticCensusRefusal {
    ClosureUnresolvable(String),
    GraphAbsent,
    PolicyUnreadable(String),
    ConservationViolated {
        raw: usize,
        summed: usize,
    },
    DetailCountMismatch {
        class_count: usize,
        detail_rows: usize,
    },
    CeilingRosterDuplicate {
        class_name: String,
    },
    CeilingRosterStale {
        class_name: String,
    },
    CeilingRosterUncovered {
        class_name: String,
    },
    IdentityUnavailable(String),
}

/// Renders each cause in the modeled vocabulary (`gunbc.instrument_targets`
/// `diagnostic_census_refusal_cause_rendered`) with the located payload appended. A free
/// function, not an impl method, so the seed-growth roster can cite it — the same discipline
/// `target_invocation_host.rs` states.
pub fn diagnostic_census_refusal_rendered(cause: &CompileCleanDiagnosticCensusRefusal) -> String {
    match cause {
        CompileCleanDiagnosticCensusRefusal::ClosureUnresolvable(detail) => {
            format!("whole-tree closure unresolvable: {detail}")
        }
        CompileCleanDiagnosticCensusRefusal::GraphAbsent => {
            "compilation produced no graph".to_string()
        }
        CompileCleanDiagnosticCensusRefusal::PolicyUnreadable(detail) => {
            format!("disposition policy or ceiling roster unreadable: {detail}")
        }
        CompileCleanDiagnosticCensusRefusal::ConservationViolated { raw, summed } => format!(
            "per-class counts do not sum to the raw diagnostics vector: raw={raw} summed={summed}"
        ),
        CompileCleanDiagnosticCensusRefusal::DetailCountMismatch {
            class_count,
            detail_rows,
        } => format!(
            "class count disagrees with the detailed UnlistedImportUse worklist: class_count={class_count} detail_rows={detail_rows}"
        ),
        CompileCleanDiagnosticCensusRefusal::CeilingRosterDuplicate { class_name } => format!(
            "two ceiling rows name one diagnostic class: {class_name}"
        ),
        CompileCleanDiagnosticCensusRefusal::CeilingRosterStale { class_name } => format!(
            "a ceiling row names a class outside the schema or no longer advisory: {class_name}"
        ),
        CompileCleanDiagnosticCensusRefusal::CeilingRosterUncovered { class_name } => format!(
            "an observed advisory class carries no ceiling row: {class_name}"
        ),
        CompileCleanDiagnosticCensusRefusal::IdentityUnavailable(which) => format!(
            "an identity digest could not be computed; the observation would be unbound: {which}"
        ),
    }
}

/// Content identity for the census's five digests: the seed's fnv1a64 authority
/// (`v1_rt::bytes_identity_hash` / `hash_combine` — the same key every other identity digest in
/// the seed consumes, per `multi_module_fixture_source_digest`'s note), folded over per-part
/// digests so a boundary shift between parts is a different whole.
fn census_identity_digest(parts: &[&[u8]]) -> String {
    let mut h = v1_rt::atom_identity_hash("compile-clean-diagnostic-census-v1".to_string());
    for part in parts {
        h = v1_rt::hash_combine(h, v1_rt::bytes_identity_hash(part));
    }
    h
}

/// One resolve of the policy closure, reading the ceiling-roster projection the bijection
/// witness joins against the observed population, and returning the closure sources the
/// resolver-policy digest binds. The UnlistedImportUse disposition bool is deliberately NOT read
/// here: the partition's single authority is `compile_clean_diagnostic_is_hard`, whose cached
/// read the census preflights (and thereby primes) before counting.
fn compile_clean_diagnostic_policy_census_read(
) -> Result<(Vec<String>, Vec<Rc<v1_compiler_compile::SourceFile>>), String> {
    let roots = default_source_roots();
    let entry = resolve_entry_file_under_roots(&roots, COMPILE_CLEAN_DIAGNOSTIC_POLICY_ENTRY)
        .map_err(|e| format!("compile_clean_diagnostic_policy resolve: {e}"))?;
    let sources =
        policy_entry_closure_sources(&roots, &entry, "gunbc.compile_clean_diagnostic_policy")?;
    let (graph, indices) =
        resolved_graph_from_sources(sources.clone(), ResolveTypecheckGate::Strict)
            .map_err(|e| format!("compile_clean_diagnostic_policy resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
    let roster = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "advisory_ceiling_roster_class_names",
        &[],
        false,
    ) {
        Ok(v) => string_list_from_value(&v, "advisory_ceiling_roster_class_names")?,
        Err(e) => return Err(format!("advisory_ceiling_roster_class_names: {e}")),
    };
    Ok((roster, sources))
}

/// ONE FABRICATED INSTANCE OF EVERY `CompilerDiagnostic` VARIANT — the diagnostic class schema,
/// as data the census can hash and evaluate dispositions over.
///
/// COMPLETENESS IS GUARDED IN BOTH DIRECTIONS, and the guards are stated here because neither is
/// optional. REMOVING or RENAMING a variant breaks the constructor below — compile-time. ADDING a
/// variant breaks the total match in `compile_clean_diagnostic_histogram_key` (same file, no
/// wildcard arm) — compile-time — and the repair of that error must add the specimen row IN THE
/// SAME EDIT; the runtime backstop is the census's bijection witness, which refuses
/// CeilingRosterStale when a roster row names a class this specimen does not know. The field
/// values are placeholders with ONE load-bearing exception: `WhereRefinementUnenforced`'s
/// disposition is decided per-instance from its reason string (`v1.std.core`
/// `is_where_refinement_unenforced_advisory_reason`), so the specimen carries a reason from the
/// advisory set — the Stale check asks whether the class is still advisory-CAPABLE, and a
/// placeholder reason would answer that question with the blocking arm and refuse a live row.
pub fn compile_clean_diagnostic_class_specimen() -> Vec<CompilerDiagnostic> {
    use crate::v1_std_core::CompilerDiagnostic::*;
    let s = || -> String { "<specimen>".to_string() };
    let istrings = || -> Rc<im::Vector<String>> { Rc::new(im::Vector::new()) };
    vec![
        UnresolvedImport { module_path: s(), importing_module: s(), span: no_span() },
        MissingExport { name: s(), module_path: s(), importing_module: s(), span: no_span() },
        ImportShadowedByLocalDefinition { name: s(), module_path: s(), importing_module: s(), span: no_span() },
        UnresolvedType { name: s(), span: no_span() },
        UnitVariantPhantomIdentityEvidenceUnavailable { name: s(), span: no_span() },
        TypeMismatch { expected: s(), got: s(), span: no_span() },
        ArityMismatch { name: s(), expected: 0, got: 0, span: no_span() },
        VariantNotFound { variant: s(), type_name: s(), span: no_span() },
        FieldNotFound { field: s(), type_name: s(), span: no_span() },
        MethodNotFound { method: s(), receiver_type: s(), span: no_span() },
        MethodExistenceUndecided { method: s(), receiver_type: s(), span: no_span() },
        MethodExistenceFrontierAdmitted { method: s(), receiver_type: s(), trigger: s(), span: no_span() },
        ReceiverTypeUnestablished { method: s(), span: no_span() },
        AlgebraApplicationEvidenceUnavailable { receiver_type: s(), argument_index: 0, span: no_span() },
        FrontierOccurrenceBudgetExceeded { method: s(), receiver_type: s(), declared: 0, observed: 0, span: no_span() },
        MissingField { field: s(), type_name: s(), span: no_span() },
        NonExhaustiveMatch { missing: istrings(), span: no_span() },
        CircularDependency { modules: istrings(), span: no_span() },
        DuplicateModule { name: s(), span: no_span() },
        DuplicateDeclaration { module_name: s(), name: s(), span: no_span() },
        MissingAnnotation { fn_name: s(), what: s(), span: no_span() },
        ParseError { message: s(), span: no_span() },
        InternalError { message: s(), span: no_span() },
        ComplexityUnknown { func_name: s(), reason: s(), span: no_span() },
        WhereRefinementUnenforced {
            predicate: s(),
            formal_type: s(),
            reason: "predicate not enforced at compile time".to_string(),
            span: no_span(),
        },
        OwnershipViolation { binding: s(), fn_name: s(), consumers: 0, span: no_span() },
        VariantCollision { variant: s(), enum1: s(), enum2: s(), span: no_span() },
        SoleConstructorViolation { type_name: s(), span: no_span() },
        OptionalCastNotEliminated { source_type: s(), target_type: s(), span: no_span() },
        BareNoneNotAdmittedByFieldType { field: s(), type_name: s(), declared_type: s(), span: no_span() },
        SourceAnnotationRefused {
            refusal: Rc::new(crate::std_source_annotation::AnnotationAttachmentRefusal::UnattachedAtScopeEnd { origin: no_span() }),
        },
        ConstructorCallAdmissionRefused {
            constructor_module_path: s(),
            constructor_decl_name: s(),
            caller_module_path: s(),
            caller_decl_name: s(),
            permitted_callers: istrings(),
            span: no_span(),
        },
        AdmitCallersEntryNotDeclRef { constructor_decl_name: s(), span: no_span() },
        DeclaredTypeNotInhabited { position: s(), expected: s(), got: s(), span: no_span() },
        DeclaredTypeInhabitanceUndecided { position: s(), reason: s(), span: no_span() },
        UnlistedImportUse { name: s(), span: no_span() },
        ReferenceDerivedImportProviderUnknown { name: s(), referencing_module: s(), span: no_span() },
        ReferenceDerivedImportExportUnproven { name: s(), referencing_module: s(), provider_module: s(), span: no_span() },
        UnlistedVariantValueUse { name: s(), span: no_span() },
        AmbiguousReference { name: s(), candidates: istrings(), span: no_span() },
        DataReferenceVisibilityBudgetExceeded { name: s(), span: no_span() },
        ParameterDefaultFormNotAdmitted { parameter: s(), admitted: istrings(), span: no_span() },
        AmbiguousAnonymousRecordLiteral { candidates: istrings(), span: no_span() },
        ModuleFilenameCollision { filename: s(), modules: istrings(), span: no_span() },
        EffectSummaryIncompleteAtFunctionValue { caller: s(), span: no_span() },
        EffectSummaryIncompleteAtLocalBinding { caller: s(), name: s(), span: no_span() },
        CallArgumentNameUnknown { callee: s(), argument: s(), declared: istrings(), span: no_span() },
        CallPositionalSurplus { callee: s(), supplied: 0, capacity: 0, span: no_span() },
        CallArgumentDuplicate { callee: s(), argument: s(), span: no_span() },
        CallPositionalDeficit { callee: s(), parameter: s(), supplied: 0, required: 0, span: no_span() },
        CallNamedArgOnFunctionValue { callee: s(), argument: s(), span: no_span() },
        EqualityOnFunctionMember { type_name: s(), member: s(), span: no_span() },
        EqualityMemberUnjudgeable { type_name: s(), member: s(), span: no_span() },
        TypeArgumentArityMismatch { type_name: s(), supplied: 0, declared: 0, span: no_span() },
        OccurrenceTransportViolation {
            refusal: Rc::new(crate::std_occurrence_identity::OccurrenceTransportRefusal::MissingAuthoredOccurrenceIdentity { diagnostic_span: no_span() }),
        },
        ContainerSpellingUnrecognized { name: s(), container_leaf: s(), span: no_span() },
        ServiceConfigReferenceJudgmentDeferred { field: s(), referenced_name: s(), trigger: s(), span: no_span() },
        TransportEmissionNotModeled {
            transport_kind: s(),
            service: s(),
            operation: s(),
            declaring_module: s(),
            target: s(),
            missing_realization_fact: s(),
            span: no_span(),
        },
        EmissionConstructUnprojectable {
            construct: crate::v1_std_core::UnprojectableConstruct::FilterInBranchCondition,
            span: no_span(),
        },
    ]
}

/// Per-class RAW diagnostic census over the whole-tree compile-clean closure.
///
/// THE THREE MOVES THE RAW REWORK MAKES, each answering a named finding of the 2026-09-09
/// review. FIRST (finding 1): every emitted diagnostic is counted; the disposition is computed
/// per INSTANCE through `compile_clean_diagnostic_is_hard` — the partition's single authority,
/// never a restated predicate — and recorded as a row attribute, so a severity change changes an
/// attribute and cannot change the population. Per-instance rather than per-class because the
/// authority itself decides per instance: `WhereRefinementUnenforced` reads its reason string,
/// so one class can carry rows on both sides of the partition. SECOND (finding 2): the whole
/// source vector is materialized into memory, hashed, and the SAME in-memory bytes are handed
/// to the compiler, so the source-vector digest is bound to the observation by construction;
/// the compiler-executable, resolver-policy, class-schema and closure digests bind the rest of
/// the provenance. THIRD (finding 5): the ceiling roster is joined against the class schema and
/// the observed population by execution — duplicate, stale and uncovered rows are typed
/// refusals, not review tells.
///
/// The class key is `compile_clean_diagnostic_histogram_key`'s first component — the total
/// match over `CompilerDiagnostic` — so a class name here can never drift from the compiler's
/// own variant identity.
pub fn compile_clean_diagnostic_census(
) -> Result<CompileCleanDiagnosticCensus, CompileCleanDiagnosticCensusRefusal> {
    // THE CHEAP REFUSALS FIRE BEFORE THE EXPENSIVE WALK. A policy that cannot be read must
    // refuse the census, not report a population it could not attribute (finding 1's
    // policy-unreadable-is-not-empty rule); preflighting the cached read also primes it, so
    // every `compile_clean_diagnostic_is_hard` call below reads one value for the whole run.
    if let Err(e) = compile_clean_unlisted_import_use_blocks_cached() {
        return Err(CompileCleanDiagnosticCensusRefusal::PolicyUnreadable(e));
    }
    let (roster_names, policy_sources) = compile_clean_diagnostic_policy_census_read()
        .map_err(CompileCleanDiagnosticCensusRefusal::PolicyUnreadable)?;

    // THE CLASS SCHEMA AND ITS DISPOSITIONS, from the specimen through the two total
    // authorities (histogram key for the name, compile_clean_diagnostic_is_hard for the
    // partition).
    let specimen = compile_clean_diagnostic_class_specimen();
    let mut schema_names: Vec<String> = Vec::new();
    let mut disposition_by_class: BTreeMap<String, DiagnosticCensusDisposition> = BTreeMap::new();
    for d in &specimen {
        let node = Rc::new(ErrorNode {
            diagnostic: Rc::new(d.clone()),
            module_name: "<specimen>".to_string(),
        });
        let (class, _) = compile_clean_diagnostic_histogram_key(&node);
        let disposition = if compile_clean_diagnostic_is_hard(&node) {
            DiagnosticCensusDisposition::Blocking
        } else {
            DiagnosticCensusDisposition::Advisory
        };
        schema_names.push(class.clone());
        disposition_by_class.insert(class, disposition);
    }
    let diagnostic_class_schema_digest = census_identity_digest(
        &schema_names
            .iter()
            .map(|n| n.as_bytes())
            .collect::<Vec<_>>(),
    );

    // THE BIJECTION WITNESS, ROSTER SIDE: no duplicate rows, and no row naming a class the
    // schema does not know or whose disposition is no longer advisory (a promoted class's row
    // must delete with the promotion).
    {
        let mut seen: BTreeSet<&String> = BTreeSet::new();
        for name in &roster_names {
            if !seen.insert(name) {
                return Err(
                    CompileCleanDiagnosticCensusRefusal::CeilingRosterDuplicate {
                        class_name: name.clone(),
                    },
                );
            }
            match disposition_by_class.get(name) {
                Some(DiagnosticCensusDisposition::Advisory) => {}
                _ => {
                    return Err(CompileCleanDiagnosticCensusRefusal::CeilingRosterStale {
                        class_name: name.clone(),
                    })
                }
            }
        }
    }

    // THE SUBJECT, MATERIALIZED BEFORE IT IS READ: the whole-tree source vector is loaded into
    // memory ONCE, hashed from those bytes, and the same bytes are compiled — the tree going
    // A -> B -> A mid-walk cannot produce a mixed population that hashes clean, because nothing
    // is re-read after the hash.
    let sources =
        match witness_layer_roots_compile_clean_sources_for_plan(&CompileCleanScopePlan::WholeTree)
        {
            Ok(Some(s)) => s,
            Ok(None) => {
                return Err(CompileCleanDiagnosticCensusRefusal::ClosureUnresolvable(
                    "whole-tree plan returned no sources".to_string(),
                ))
            }
            Err(e) => return Err(CompileCleanDiagnosticCensusRefusal::ClosureUnresolvable(e)),
        };
    let source_vector_digest = {
        let mut sorted: Vec<&Rc<v1_compiler_compile::SourceFile>> = sources.iter().collect();
        sorted.sort_by(|a, b| a.path.cmp(&b.path));
        let mut parts: Vec<&[u8]> = Vec::new();
        for s in &sorted {
            parts.push(s.path.as_bytes());
            parts.push(s.content.as_bytes());
        }
        census_identity_digest(&parts)
    };
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
    let graph = result
        .graph
        .clone()
        .ok_or(CompileCleanDiagnosticCensusRefusal::GraphAbsent)?;
    let closure_modules = graph.modules.len();
    let closure_digest = {
        let mut module_paths: Vec<&str> = graph
            .modules
            .iter()
            .map(|m| m.type_env.module_path.as_str())
            .collect();
        module_paths.sort_unstable();
        census_identity_digest(
            &module_paths
                .iter()
                .map(|p| p.as_bytes())
                .collect::<Vec<_>>(),
        )
    };

    // THE RAW COUNT: every emitted diagnostic, no severity filter anywhere on this path. The
    // row key is (CLASS, DISPOSITION) and the disposition is computed PER INSTANCE through the
    // same authority the gate reads — `WhereRefinementUnenforced` decides by the instance's
    // reason string, so a mixed class produces one row per disposition rather than one row
    // carrying a fabricated class-level answer.
    let mut by_class: BTreeMap<(String, DiagnosticCensusDisposition), ClassAccumulation> =
        BTreeMap::new();
    for d in result.diagnostics.iter() {
        let (class, _) = compile_clean_diagnostic_histogram_key(d);
        let disposition = if compile_clean_diagnostic_is_hard(d) {
            DiagnosticCensusDisposition::Blocking
        } else {
            DiagnosticCensusDisposition::Advisory
        };
        let span = diagnostic_to_span(d.diagnostic.clone());
        let position = if span.file.is_empty() {
            format!("module:{}", d.module_name)
        } else {
            format!(
                "{}:{}",
                normalize_repo_relative_path_for_census(&span.file),
                span.start
            )
        };
        let entry = by_class.entry((class, disposition)).or_default();
        entry.0 += 1;
        entry.1.insert(d.module_name.clone());
        entry.2.insert(position);
    }

    // CONSERVATION: the per-class counts must sum to the raw vector length — a diagnostic
    // dropped or invented between emission and counting is a refusal, never a rounding note.
    let raw_diagnostics = result.diagnostics.len();
    let summed: usize = by_class.values().map(|(n, _, _)| *n).sum();
    if summed != raw_diagnostics {
        return Err(CompileCleanDiagnosticCensusRefusal::ConservationViolated {
            raw: raw_diagnostics,
            summed,
        });
    }

    // THE DETAIL-COUNT LAW: the UnlistedImportUse class count and the per-row worklist are two
    // reads of the same vector, and their agreement is checked rather than assumed. The class
    // count sums across dispositions — the policy row can move the whole class across the
    // partition, and the law must hold on either side of it.
    let unlisted_import_rows = unlisted_import_rows_from_resolved(&result)
        .map_err(CompileCleanDiagnosticCensusRefusal::ClosureUnresolvable)?;
    let unlisted_class_count: usize = by_class
        .iter()
        .filter(|((class, _), _)| class == "UnlistedImportUse")
        .map(|(_, (n, _, _))| *n)
        .sum();
    if unlisted_class_count != unlisted_import_rows.len() {
        return Err(CompileCleanDiagnosticCensusRefusal::DetailCountMismatch {
            class_count: unlisted_class_count,
            detail_rows: unlisted_import_rows.len(),
        });
    }

    // THE BIJECTION WITNESS, OBSERVED SIDE: every observed ADVISORY class carries a ceiling row.
    // This is the reintroduction wall — a class that returns after burning to zero refuses here
    // until its ceiling is re-declared. Blocking rows need no roster row: the gate already
    // refuses them, and a roster row for a class that is no longer advisory-capable is the
    // Stale arm above. Each row reads its own observed disposition, so a mixed class answers
    // for its advisory arm only.
    let roster_set: BTreeSet<&String> = roster_names.iter().collect();
    for (class, disposition) in by_class.keys() {
        if *disposition == DiagnosticCensusDisposition::Advisory && !roster_set.contains(class) {
            return Err(
                CompileCleanDiagnosticCensusRefusal::CeilingRosterUncovered {
                    class_name: class.clone(),
                },
            );
        }
    }

    // THE REMAINING IDENTITY DIGESTS. The executable digest reads the running binary's own
    // bytes; a read failure is IdentityUnavailable, because an unbound observation is not a
    // smaller one.
    let compiler_executable_digest = std::env::current_exe()
        .and_then(std::fs::read)
        .map(|bytes| v1_rt::bytes_identity_hash(&bytes))
        .map_err(|e| {
            CompileCleanDiagnosticCensusRefusal::IdentityUnavailable(format!(
                "compiler executable: {e}"
            ))
        })?;
    let resolver_policy_digest = {
        let mut sorted: Vec<&Rc<v1_compiler_compile::SourceFile>> = policy_sources.iter().collect();
        sorted.sort_by(|a, b| a.path.cmp(&b.path));
        let mut parts: Vec<&[u8]> = Vec::new();
        for s in &sorted {
            parts.push(s.path.as_bytes());
            parts.push(s.content.as_bytes());
        }
        census_identity_digest(&parts)
    };

    let classes = by_class
        .into_iter()
        .map(
            |((class_name, disposition), (diagnostics, modules, positions))| {
                RawDiagnosticClassObservation {
                    class_name,
                    disposition,
                    diagnostics,
                    distinct_modules: modules.len(),
                    distinct_positions: positions.len(),
                }
            },
        )
        .collect();
    Ok(CompileCleanDiagnosticCensus {
        identity: CompileCleanDiagnosticCensusIdentity {
            source_vector_digest,
            compiler_executable_digest,
            resolver_policy_digest,
            diagnostic_class_schema_digest,
            closure_digest,
        },
        closure_modules,
        raw_diagnostics,
        classes,
        unlisted_import_rows,
    })
}

/// Floor compile-clean verdict over the whole-tree closure (shared-index receipt semantics).
pub fn compile_clean_floor_verdict_whole_tree() -> Result<bool, String> {
    let roots = default_source_roots();
    let sources = match witness_layer_roots_compile_clean_sources_for_plan(
        &CompileCleanScopePlan::WholeTree,
    )? {
        None => return Ok(true),
        Some(s) => s,
    };
    Ok(floor_compile_clean_emit_ok_via_index(sources, &roots).0)
}

/// CLI compile-clean verdict: same source closure and diagnostic policy as the floor,
/// without the shared-index receipt shortcut (the standalone `gunbc compile` transport).
pub fn compile_clean_cli_verdict_whole_tree() -> Result<bool, String> {
    let result = compile_clean_whole_tree_resolved()?;
    let graph_ok = result.graph.is_some();
    let hard = compile_clean_im_vector_has_hard_errors(result.diagnostics.as_ref());
    Ok(graph_ok && !hard)
}

/// Both realizations must agree modulo the single policy row.
pub fn compile_clean_cli_floor_verdicts_agree() -> Result<bool, String> {
    let floor = compile_clean_floor_verdict_whole_tree()?;
    let cli = compile_clean_cli_verdict_whole_tree()?;
    Ok(floor == cli)
}

/// `(class, name)` key for histogram aggregation over hard diagnostics.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_COMPILE_CLEAN_DIAGNOSTIC_HISTOGRAM_SCAFFOLD_MARKER` / §7):
/// total match over `CompilerDiagnostic` variants — no silent widening.
pub fn compile_clean_diagnostic_histogram_key(d: &Rc<ErrorNode>) -> (String, String) {
    use crate::v1_std_core::CompilerDiagnostic;
    let class = match d.diagnostic.as_ref() {
        CompilerDiagnostic::UnresolvedImport { .. } => "UnresolvedImport",
        CompilerDiagnostic::MissingExport { .. } => "MissingExport",
        CompilerDiagnostic::ImportShadowedByLocalDefinition { .. } => {
            "ImportShadowedByLocalDefinition"
        }
        CompilerDiagnostic::UnresolvedType { .. } => "UnresolvedType",
        CompilerDiagnostic::UnitVariantPhantomIdentityEvidenceUnavailable { .. } => {
            "UnitVariantPhantomIdentityEvidenceUnavailable"
        }
        CompilerDiagnostic::TypeMismatch { .. } => "TypeMismatch",
        CompilerDiagnostic::ArityMismatch { .. } => "ArityMismatch",
        CompilerDiagnostic::VariantNotFound { .. } => "VariantNotFound",
        CompilerDiagnostic::FieldNotFound { .. } => "FieldNotFound",
        CompilerDiagnostic::MethodNotFound { .. } => "MethodNotFound",
        CompilerDiagnostic::MethodExistenceUndecided { .. } => "MethodExistenceUndecided",
        CompilerDiagnostic::ReceiverTypeUnestablished { .. } => "ReceiverTypeUnestablished",
        CompilerDiagnostic::FrontierOccurrenceBudgetExceeded { .. } => {
            "FrontierOccurrenceBudgetExceeded"
        }
        CompilerDiagnostic::MethodExistenceFrontierAdmitted { .. } => {
            "MethodExistenceFrontierAdmitted"
        }
        CompilerDiagnostic::MissingField { .. } => "MissingField",
        CompilerDiagnostic::NonExhaustiveMatch { .. } => "NonExhaustiveMatch",
        CompilerDiagnostic::CircularDependency { .. } => "CircularDependency",
        CompilerDiagnostic::DuplicateModule { .. } => "DuplicateModule",
        CompilerDiagnostic::DuplicateDeclaration { .. } => "DuplicateDeclaration",
        CompilerDiagnostic::MissingAnnotation { .. } => "MissingAnnotation",
        CompilerDiagnostic::ParseError { .. } => "ParseError",
        CompilerDiagnostic::InternalError { .. } => "InternalError",
        CompilerDiagnostic::ComplexityUnknown { .. } => "ComplexityUnknown",
        CompilerDiagnostic::WhereRefinementUnenforced { .. } => "WhereRefinementUnenforced",
        CompilerDiagnostic::OwnershipViolation { .. } => "OwnershipViolation",
        CompilerDiagnostic::VariantCollision { .. } => "VariantCollision",
        CompilerDiagnostic::SoleConstructorViolation { .. } => "SoleConstructorViolation",
        CompilerDiagnostic::OptionalCastNotEliminated { .. } => "OptionalCastNotEliminated",
        CompilerDiagnostic::BareNoneNotAdmittedByFieldType { .. } => {
            "BareNoneNotAdmittedByFieldType"
        }
        CompilerDiagnostic::ConstructorCallAdmissionRefused { .. } => {
            "ConstructorCallAdmissionRefused"
        }
        CompilerDiagnostic::AdmitCallersEntryNotDeclRef { .. } => "AdmitCallersEntryNotDeclRef",
        CompilerDiagnostic::DeclaredTypeNotInhabited { .. } => "DeclaredTypeNotInhabited",
        CompilerDiagnostic::DeclaredTypeInhabitanceUndecided { .. } => {
            "DeclaredTypeInhabitanceUndecided"
        }
        CompilerDiagnostic::UnlistedImportUse { .. } => "UnlistedImportUse",
        CompilerDiagnostic::AmbiguousReference { .. } => "AmbiguousReference",
        CompilerDiagnostic::DataReferenceVisibilityBudgetExceeded { .. } => {
            "DataReferenceVisibilityBudgetExceeded"
        }
        CompilerDiagnostic::ParameterDefaultFormNotAdmitted { .. } => {
            "ParameterDefaultFormNotAdmitted"
        }
        CompilerDiagnostic::AmbiguousAnonymousRecordLiteral { .. } => {
            "AmbiguousAnonymousRecordLiteral"
        }
        CompilerDiagnostic::ModuleFilenameCollision { .. } => "ModuleFilenameCollision",
        CompilerDiagnostic::EffectSummaryIncompleteAtFunctionValue { .. } => {
            "EffectSummaryIncompleteAtFunctionValue"
        }
        CompilerDiagnostic::EffectSummaryIncompleteAtLocalBinding { .. } => {
            "EffectSummaryIncompleteAtLocalBinding"
        }
        CompilerDiagnostic::CallArgumentNameUnknown { .. } => "CallArgumentNameUnknown",
        CompilerDiagnostic::CallPositionalSurplus { .. } => "CallPositionalSurplus",
        CompilerDiagnostic::CallPositionalDeficit { .. } => "CallPositionalDeficit",
        CompilerDiagnostic::CallArgumentDuplicate { .. } => "CallArgumentDuplicate",
        CompilerDiagnostic::CallNamedArgOnFunctionValue { .. } => "CallNamedArgOnFunctionValue",
        CompilerDiagnostic::TypeArgumentArityMismatch { .. } => "TypeArgumentArityMismatch",
        CompilerDiagnostic::EqualityOnFunctionMember { .. } => "EqualityOnFunctionMember",
        CompilerDiagnostic::EqualityMemberUnjudgeable { .. } => "EqualityMemberUnjudgeable",
        CompilerDiagnostic::OccurrenceTransportViolation { .. } => "OccurrenceTransportViolation",
        CompilerDiagnostic::SourceAnnotationRefused { .. } => "SourceAnnotationRefused",
        CompilerDiagnostic::ContainerSpellingUnrecognized { .. } => "ContainerSpellingUnrecognized",
        CompilerDiagnostic::TransportEmissionNotModeled { .. } => "TransportEmissionNotModeled",
        CompilerDiagnostic::EmissionConstructUnprojectable { .. } => {
            "EmissionConstructUnprojectable"
        }
        CompilerDiagnostic::ServiceConfigReferenceJudgmentDeferred { .. } => {
            "ServiceConfigReferenceJudgmentDeferred"
        }
        CompilerDiagnostic::UnlistedVariantValueUse { .. } => "UnlistedVariantValueUse",
        CompilerDiagnostic::ReferenceDerivedImportProviderUnknown { .. } => {
            "ReferenceDerivedImportProviderUnknown"
        }
        CompilerDiagnostic::ReferenceDerivedImportExportUnproven { .. } => {
            "ReferenceDerivedImportExportUnproven"
        }
        CompilerDiagnostic::AlgebraApplicationEvidenceUnavailable { .. } => {
            "AlgebraApplicationEvidenceUnavailable"
        }
    };
    let name = match d.diagnostic.as_ref() {
        CompilerDiagnostic::UnresolvedImport { module_path, .. } => module_path.clone(),
        CompilerDiagnostic::MissingExport { name, .. } => name.clone(),
        CompilerDiagnostic::ImportShadowedByLocalDefinition { name, .. } => name.clone(),
        CompilerDiagnostic::UnresolvedType { name, .. } => name.clone(),
        CompilerDiagnostic::UnitVariantPhantomIdentityEvidenceUnavailable { name, .. } => {
            name.clone()
        }
        CompilerDiagnostic::TypeMismatch { got, .. } => got.clone(),
        CompilerDiagnostic::ArityMismatch { name, .. } => name.clone(),
        CompilerDiagnostic::VariantNotFound { variant, .. } => variant.clone(),
        CompilerDiagnostic::FieldNotFound { field, .. } => field.clone(),
        CompilerDiagnostic::MethodNotFound { method, .. } => method.clone(),
        CompilerDiagnostic::MethodExistenceUndecided { method, .. } => method.clone(),
        CompilerDiagnostic::MethodExistenceFrontierAdmitted { method, .. } => method.clone(),
        CompilerDiagnostic::ReceiverTypeUnestablished { method, .. } => method.clone(),
        CompilerDiagnostic::FrontierOccurrenceBudgetExceeded { method, .. } => method.clone(),
        CompilerDiagnostic::MissingField { field, .. } => field.clone(),
        CompilerDiagnostic::NonExhaustiveMatch { .. } => "(non-exhaustive)".to_string(),
        CompilerDiagnostic::CircularDependency { .. } => "(cycle)".to_string(),
        CompilerDiagnostic::DuplicateModule { name, .. } => name.clone(),
        CompilerDiagnostic::DuplicateDeclaration { name, .. } => name.clone(),
        CompilerDiagnostic::MissingAnnotation { fn_name, .. } => fn_name.clone(),
        CompilerDiagnostic::ParseError { message, .. } => truncate_histogram_label(message, 80),
        CompilerDiagnostic::InternalError { message, .. } => {
            compile_clean_internal_error_histogram_name(message)
        }
        CompilerDiagnostic::ComplexityUnknown { func_name, .. } => func_name.clone(),
        // The NAME joins the predicate AND the deferral reason, because the burn-down this
        // histogram feeds is a list of deferral CLASSES to close and the reasons do not close
        // together. "int predicate not implemented" fires both where a range's bounds are
        // literals and evaluation still declined, and where no evaluator exists at all; keying
        // on the predicate alone puts an evaluator that exists in the same row as one that does
        // not, so closing either is invisible here. Keying on the reason alone would instead
        // spread one deferral class across a row per predicate that happens to hit it.
        CompilerDiagnostic::WhereRefinementUnenforced {
            predicate, reason, ..
        } => format!("{predicate}: {reason}"),
        CompilerDiagnostic::OwnershipViolation { binding, .. } => binding.clone(),
        CompilerDiagnostic::VariantCollision { variant, .. } => variant.clone(),
        CompilerDiagnostic::SoleConstructorViolation { type_name, .. } => type_name.clone(),
        CompilerDiagnostic::OptionalCastNotEliminated { source_type, .. } => source_type.clone(),
        CompilerDiagnostic::BareNoneNotAdmittedByFieldType { field, .. } => field.clone(),
        CompilerDiagnostic::ConstructorCallAdmissionRefused {
            constructor_decl_name,
            ..
        } => constructor_decl_name.clone(),
        CompilerDiagnostic::AdmitCallersEntryNotDeclRef {
            constructor_decl_name,
            ..
        } => constructor_decl_name.clone(),
        CompilerDiagnostic::DeclaredTypeNotInhabited { position, .. } => position.clone(),
        CompilerDiagnostic::DeclaredTypeInhabitanceUndecided { position, .. } => position.clone(),
        CompilerDiagnostic::UnlistedImportUse { name, .. } => name.clone(),
        CompilerDiagnostic::UnlistedVariantValueUse { name, .. } => name.clone(),
        CompilerDiagnostic::ReferenceDerivedImportProviderUnknown { name, .. } => name.clone(),
        CompilerDiagnostic::ReferenceDerivedImportExportUnproven { name, .. } => name.clone(),
        CompilerDiagnostic::AmbiguousReference { name, .. } => name.clone(),
        CompilerDiagnostic::DataReferenceVisibilityBudgetExceeded { name, .. } => name.clone(),
        CompilerDiagnostic::ParameterDefaultFormNotAdmitted { parameter, .. } => parameter.clone(),
        CompilerDiagnostic::AmbiguousAnonymousRecordLiteral { candidates, .. } => {
            candidates.iter().cloned().collect::<Vec<_>>().join("|")
        }
        CompilerDiagnostic::ModuleFilenameCollision { filename, .. } => filename.clone(),
        CompilerDiagnostic::EffectSummaryIncompleteAtFunctionValue { caller, .. } => caller.clone(),
        CompilerDiagnostic::EffectSummaryIncompleteAtLocalBinding { caller, .. } => caller.clone(),
        CompilerDiagnostic::CallArgumentNameUnknown { argument, .. } => argument.clone(),
        CompilerDiagnostic::CallPositionalSurplus { callee, .. } => callee.clone(),
        CompilerDiagnostic::CallPositionalDeficit { parameter, .. } => parameter.clone(),
        CompilerDiagnostic::CallArgumentDuplicate { argument, .. } => argument.clone(),
        CompilerDiagnostic::CallNamedArgOnFunctionValue { argument, .. } => argument.clone(),
        CompilerDiagnostic::TypeArgumentArityMismatch { type_name, .. } => type_name.clone(),
        CompilerDiagnostic::EqualityOnFunctionMember { type_name, .. } => type_name.clone(),
        CompilerDiagnostic::EqualityMemberUnjudgeable { type_name, .. } => type_name.clone(),
        CompilerDiagnostic::OccurrenceTransportViolation { .. } => {
            "(occurrence-transport-refusal)".to_string()
        }
        // The three refusal kinds are separate failure classes — a reader fixing
        // "prose in the wrong place" needs to know WHICH wrong place, so they stay
        // distinct in the histogram rather than collapsing to one annotation bucket.
        CompilerDiagnostic::SourceAnnotationRefused { refusal, .. } => {
            use crate::std_source_annotation::AnnotationAttachmentRefusal;
            match refusal.as_ref() {
                AnnotationAttachmentRefusal::UnattachedAtScopeEnd { .. } => {
                    "(annotation-unattached)".to_string()
                }
                AnnotationAttachmentRefusal::TrailingNotModeled { .. } => {
                    "(annotation-trailing)".to_string()
                }
                AnnotationAttachmentRefusal::BodyGrainNotModeled { .. } => {
                    "(annotation-body-grain)".to_string()
                }
            }
        }
        // The NAME is the full spelling, not its container leaf: the burn-down this
        // histogram feeds is a list of spellings to declare a row for, and every
        // refusal of one leaf would otherwise aggregate into a single row.
        CompilerDiagnostic::ContainerSpellingUnrecognized { name, .. } => name.clone(),
        // The NAME is the qualified operation, not the transport kind: the burn-down this
        // histogram feeds is the list of operations awaiting a realization handler, and keying
        // on "file" would aggregate every one of them into a single row.
        CompilerDiagnostic::TransportEmissionNotModeled {
            service, operation, ..
        } => format!("{service}.{operation}"),
        CompilerDiagnostic::EmissionConstructUnprojectable { construct, .. } => {
            crate::v1_std_core::unprojectable_construct_identity(*construct)
        }
        // The NAME is the config FIELD, not the referenced spelling: the burn-down this
        // histogram feeds is the list of service-config fields still awaiting the reference
        // judgment, and keying on the referenced name would spread one unjudged field across
        // a row per service that happens to use a different word in it.
        CompilerDiagnostic::ServiceConfigReferenceJudgmentDeferred { field, .. } => field.clone(),
        // The NAME is the RECEIVER TYPE, not the argument index: the histogram this feeds
        // groups by the subject whose evidence is missing, and keying on the index would
        // scatter one unresolvable receiver across a row per argument position it appears in.
        CompilerDiagnostic::AlgebraApplicationEvidenceUnavailable { receiver_type, .. } => {
            receiver_type.clone()
        }
    };
    (class.to_string(), name)
}

pub(crate) fn compile_clean_internal_error_histogram_name(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("function '") {
        if let Some(name) = rest.split_once('\'').map(|(n, _)| n) {
            return format!("function:{name}");
        }
    }
    if let Some(rest) = message.strip_prefix("undefined variable '") {
        if let Some(name) = rest.split_once('\'').map(|(n, _)| n) {
            return format!("variable:{name}");
        }
    }
    truncate_histogram_label(message, 80)
}

pub(crate) fn compile_clean_diags_from_resolved_stages(
    resolve_diags: &Rc<im::Vector<Rc<ErrorNode>>>,
    norm_diags: &Rc<im::Vector<Rc<ErrorNode>>>,
    typed: &Rc<v1_compiler_compile::ResolvedGraph>,
    ownership_diags: &Rc<im::Vector<Rc<ErrorNode>>>,
) -> Rc<im::Vector<Rc<ErrorNode>>> {
    let mut acc = im::Vector::new();
    for d in resolve_diags.iter() {
        acc.push_back(d.clone());
    }
    acc.extend(norm_diags.iter().cloned());
    for d in typed.diagnostics.iter() {
        acc.push_back(d.clone());
    }
    acc.extend(ownership_diags.iter().cloned());
    Rc::new(acc)
}

pub(crate) fn compile_clean_touched_path_norm(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

pub(crate) fn compile_clean_touched_path_is_docs_only(path: &str) -> bool {
    compile_clean_touched_path_norm(path).starts_with("docs/")
}

pub(crate) fn compile_clean_touched_path_is_dag_source(path: &str) -> bool {
    compile_clean_touched_path_norm(path).ends_with(".dag")
}

pub(crate) fn compile_clean_verdict_affecting_touch(touched_paths: &[String]) -> bool {
    !touched_paths.is_empty()
        && !touched_paths
            .iter()
            .all(|p| compile_clean_touched_path_is_docs_only(p))
        && touched_paths
            .iter()
            .any(|p| compile_clean_touched_path_is_dag_source(p))
}

pub(crate) fn compile_clean_broad_stop_line_blocks_skip(
    entry_path: &str,
    touched_paths: &[String],
) -> bool {
    if !compile_clean_verdict_affecting_touch(touched_paths) {
        return false;
    }
    let entry_rel = workspace_relative_repo_path(entry_path);
    [
        COMPILE_CLEAN_SHARD_A_VALIDATING_ENTRY,
        COMPILE_CLEAN_SCOPE_VALIDATING_ENTRY,
    ]
    .iter()
    .any(|check| workspace_relative_repo_path(check) == entry_rel)
}
