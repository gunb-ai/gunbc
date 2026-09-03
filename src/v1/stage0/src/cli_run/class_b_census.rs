// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 7 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::disallowed_macros,  // 7
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

/// True when this diagnostic is an exact named row above — same file AND same unresolved
/// type. Any other diagnostic, including a different unresolved type in the same file,
/// is not exempt.
pub(crate) fn class_b_diagnostic_is_named_exception(d: &Rc<ErrorNode>) -> bool {
    use crate::v1_std_core::CompilerDiagnostic;
    let CompilerDiagnostic::UnresolvedType { name, span } = d.diagnostic.as_ref() else {
        return false;
    };
    CLASS_B_ACCIDENTAL_COVERAGE_EXCEPTIONS
        .iter()
        .any(|(file, ty)| span.file == *file && name == ty)
}

pub(crate) fn class_b_overlay_authority_content() -> &'static str {
    static CONTENT: OnceLock<String> = OnceLock::new();
    CONTENT
        .get_or_init(|| {
            let path = process_workspace_root().join(CLASS_B_OVERLAY_REL);
            std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "class_b overlay authority: failed to read {}: {e}",
                    path.display()
                )
            })
        })
        .as_str()
}

/// Project `class_b_declared_import_pool_roots` out of the overlay authority source text.
pub(crate) fn class_b_declared_import_pool_roots_from_source(content: &str) -> Vec<String> {
    string_list_data_from_module_source(
        CLASS_B_OVERLAY_REL,
        content,
        CLASS_B_DECLARED_POOL_ROOTS_DATA_NAME,
        false,
    )
}

/// The Class B rows 1–2 declared-import pool roots, read live from the single `.dag` authority.
pub(crate) fn class_b_declared_import_pool_roots() -> Vec<String> {
    static ROOTS: OnceLock<Vec<String>> = OnceLock::new();
    ROOTS
        .get_or_init(|| {
            class_b_declared_import_pool_roots_from_source(class_b_overlay_authority_content())
        })
        .clone()
}

pub(crate) fn class_b_pool_source_roots(workspace: &Path, pool_roots: &[String]) -> Vec<PathBuf> {
    pool_roots.iter().map(|rel| workspace.join(rel)).collect()
}

/// Every workspace-relative path whose content can change `run_class_b_import_closure_gate`'s
/// verdict: witness-layer import closure of the gate transport modules (rows 3–4 wide pool),
/// declared-import-pool closure of the subject entry (rows 1–2 minimal pool), perturbation
/// fixtures, sorted.
///
/// 🟡 dissolve-on (two triggers, near then terminal):
///
/// NEAR — the import walk here duplicates the shape the deleted selection-control suite used, and
/// `regen_input_sources` (whole-root seed under `src/v1` vs. declared entry list). They differ in
/// entry selection and duplicate policy (refuse vs. superset). DISSOLVES WHEN lifted to one
/// parameterized helper (duplicate policy + entry source as arguments).
///
/// TERMINAL — owning lane: `affected-set-precompute-pruning (plan doc deleted 2026-08-28)`, whose **Step 5
/// "delete Rust parallel"** (NOT STARTED, gated on Step 4) is what retires host-side selection
/// Rust in favour of the `.dag` authority. This fn and
/// `class_b_import_closure_gate_skip_label_for_ci` are new members of exactly that Rust-parallel
/// set — a path/import-closure skip decision living in the seed rather than in `.dag` — so they
/// inherit Step 5's terminal condition. They are ENUMERATED on that roster as an explicit
/// deferral (the "Step 5 roster — CI skip-decision surfaces" row, extended by PR #7835), which
/// is what makes this a declared, countable seed-retained surface rather than a silent escape
/// hatch (DESIGN §7). Why deferred rather than modeled now: the decision must run BEFORE the
/// floor resolves anything — that is its entire purpose — so a `.dag` consumer would pay the
/// ~100s cold whole-pool resolve the skip exists to avoid; it therefore dissolves with the
/// persistent content-keyed node store, not on its own schedule. Declared pool roots are NOT
/// forked here: they are projected live from
/// `gunbc.class_b_import_closure_overlay.class_b_declared_import_pool_roots` (same authority the
/// transport and witnesses read).
///
/// Receipt bar, per DESIGN §5: this is a scaffold because the decision is *checkable* by
/// execution — skip/run label arms (structural + 2 refusal), discriminating in both directions,
/// plus bin unit tests and a live authority identity join for the declared pool roots.
pub fn class_b_import_closure_input_sources(workspace: &Path) -> Result<Vec<String>, String> {
    let witness_roots = class_b_pool_source_roots(workspace, &witness_layer_roots());
    let pool_roots = class_b_pool_source_roots(workspace, &class_b_declared_import_pool_roots());
    let mut seen = import_closure_dag_files(workspace, &witness_roots, CLASS_B_GATE_INPUT_ENTRIES)?;
    seen.extend(import_closure_dag_files(
        workspace,
        &pool_roots,
        &[CLASS_B_ENTRY_REL],
    )?);
    collect_repo_files_under_prefix(workspace, CLASS_B_FIXTURES_PREFIX, &mut seen)?;
    let mut result: Vec<String> = seen.into_iter().collect();
    result.sort();
    Ok(result)
}

pub(crate) fn class_b_path_affects_gate(changed: &str, dag_closure: &HashSet<String>) -> bool {
    let p = normalize_repo_path(changed);
    if p.starts_with("src/v1/") {
        return true;
    }
    if p.starts_with("fixtures/class_b_import_closure/") || p == "fixtures/class_b_import_closure" {
        return true;
    }
    if p == "Cargo.lock"
        || p == "Cargo.toml"
        || p.ends_with("/Cargo.toml")
        || p == "rust-toolchain.toml"
        || p == "rust-toolchain"
        || p == ".cargo/config.toml"
        || p == ".cargo/config"
    {
        return true;
    }
    dag_closure.contains(&p)
}

/// CI skip label for the Class B gate inside `source_root_ingest_gate_passes`.
pub fn class_b_import_closure_gate_skip_label_for_ci() -> String {
    if std::env::var("GITHUB_EVENT_NAME").ok().as_deref() != Some("pull_request") {
        eprintln!("class B gate skip: not pull_request — run gate (cold control)");
        return RUN_CLASS_B_GATE_LABEL.to_string();
    }
    let (changed_paths, departed_paths) = match floor_git_diff_name_status_range() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!(
                "[{RUN_CLASS_B_GATE_DIFF_OBSERVATION_FAILED_LABEL}] class B gate skip: diff observation failed ({msg}) — run gate"
            );
            return RUN_CLASS_B_GATE_DIFF_OBSERVATION_FAILED_LABEL.to_string();
        }
    };
    if changed_paths.is_empty() {
        eprintln!("class B gate skip: empty diff — run gate (fail-closed cold control)");
        return RUN_CLASS_B_GATE_LABEL.to_string();
    }
    if let Some(gone) = departed_paths.iter().find(|p| {
        let n = normalize_repo_path(p);
        !n.starts_with("docs/")
    }) {
        eprintln!(
            "class B gate skip: departed non-docs path in diff ({}) — run gate (current-tree closure cannot see deletions)",
            normalize_repo_path(gone)
        );
        return RUN_CLASS_B_GATE_LABEL.to_string();
    }
    let workspace = workspace_root();
    let dag_closure: HashSet<String> = match class_b_import_closure_input_sources(&workspace) {
        Ok(sources) => sources.into_iter().collect(),
        Err(msg) => {
            eprintln!(
                "[{RUN_CLASS_B_GATE_INPUT_CLOSURE_FAILED_LABEL}] class B gate skip: input-closure computation failed ({msg}) — run gate"
            );
            return RUN_CLASS_B_GATE_INPUT_CLOSURE_FAILED_LABEL.to_string();
        }
    };
    match changed_paths
        .iter()
        .find(|p| class_b_path_affects_gate(p, &dag_closure))
    {
        Some(example) => {
            eprintln!(
                "class B gate skip: diff intersects Class B gate inputs (e.g. {}) — run gate",
                normalize_repo_path(example)
            );
            RUN_CLASS_B_GATE_LABEL.to_string()
        }
        None => {
            eprintln!(
                "class B gate skip: {} changed path(s), none intersect the Class B gate input closure (declared-import pool ∪ witness layer ∪ fixtures ∪ src/v1/** ∪ Cargo/toolchain) — gate verdict provably unchanged (push-to-main runs gate unconditionally as cold control)",
                changed_paths.len()
            );
            CLASS_B_GATE_NOT_AFFECTED_SKIP_LABEL.to_string()
        }
    }
}

/// Builtin backing `class_b_import_closure_gate_not_affected_skip` in the transport gate.
pub fn class_b_import_closure_gate_not_affected_skip_for_ci() -> bool {
    class_b_import_closure_gate_skip_label_for_ci() == CLASS_B_GATE_NOT_AFFECTED_SKIP_LABEL
}
