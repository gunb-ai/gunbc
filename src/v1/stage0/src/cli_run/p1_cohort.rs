// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 3 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
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

/// P1 retention-vs-drain cohort receipt (floor-prep-tax-program (plan doc deleted 2026-08-28) §P1):
/// per-entry-group instrumentation distinct from `emit_floor_drain_group_line`'s
/// cumulative cache-size line — this line prices the per-group wall/resolve/eval
/// tax the program's diagnosing, plus the typecheck-cache-hit / resolved-graph-hit
/// / eviction facts needed to tell "shared prep reused" from "shared prep re-paid":
/// `typecheck_cache_hit` is whether `typecheck_compute_count()` moved during this
/// group (a typecheck-memo hit/miss signal, NOT schedule-retention cache
/// occupancy — schedule-retention has no per-group hit/miss concept to read,
/// only cumulative eviction counters, which is why `modules_evicted`/
/// `graphs_evicted` carry that side of the story instead). Gated on its own env var
/// (never folded into `GUNBC_FLOOR_DRAIN_RETENTION`) so enabling one measurement
/// mode does not silently change the other's log shape (§3 — two distinct facts,
/// two distinct switches).
///
/// Scaffold, not a second production floor driver: this instrumentation and its
/// sole consumer, `p1_cohort_probe`, are diagnostic-only (opt-in, zero effect on
/// default eviction behavior — see `p1-retention-vs-drain-cohort-receipt (plan doc deleted 2026-08-28)`).
/// Dissolve-on: once P1 is banked and no other open lane needs cohort-scoped A/B
/// retention receipts, delete `emit_p1_cohort_entry_line`/`p1_cohort_receipt_enabled`/
/// `p1_cohort_cgroup_memory`, `resolved_graph_evictions` on `IndexRetentionSnapshot`,
/// and `src/v1/stage0/src/bin/p1_cohort_probe.rs` together (§6 — no parallel-ledger
/// mechanism kept around past the measurement it was built for).
pub(crate) fn p1_cohort_receipt_enabled() -> bool {
    std::env::var("GUNBC_P1_COHORT_RECEIPT")
        .ok()
        .as_deref()
        .map(|v| matches!(v, "1" | "true" | "TRUE"))
        .unwrap_or(false)
}

/// P1 cohort / 2×2 matrix experiment is active — gates optional shared-store arms on Serial
/// and private-store arms on ControlledWidthTwo without affecting production defaults.
pub(crate) fn p1_cohort_experiment_active() -> bool {
    p1_cohort_receipt_enabled()
        || std::env::var("GUNBC_P1_MATRIX_CELL").is_ok()
        || std::env::var("GUNBC_P1_SHARED_TYPED_STORE").is_ok()
}

/// Best-effort cgroup `memory.current` / `memory.peak` readback for the P1 cohort
/// receipt (§3 reuse of `memory_governor`'s cgroup-walk authority — no second
/// cgroup-path derivation here). `memory.peak` is absent on some kernels; that
/// arm reads `None` rather than fabricating a value (§5).
pub fn p1_cohort_cgroup_memory() -> (Option<u64>, Option<u64>) {
    match crate::memory_governor::leaf_cgroup_dir() {
        Some(dir) => (
            crate::memory_governor::read_cgroup_u64(&dir, "memory.current"),
            crate::memory_governor::read_cgroup_u64(&dir, "memory.peak"),
        ),
        None => (None, None),
    }
}
