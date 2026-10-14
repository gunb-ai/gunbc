// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 97 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::disallowed_macros,  // 75
    clippy::needless_borrow,  // 7
    clippy::needless_return,  // 1
    clippy::nonminimal_bool,  // 1
    clippy::type_complexity,  // 1
    dead_code,  // 10
    unused_imports,  // 0 -- pre-existing
    unused_variables,  // 2
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
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_char_add_str,
    clippy::too_many_arguments,
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

/// The two grounds on which the floor removes an enrolled identity from a roster BEFORE the
/// fold. The authority is `v1.compiler.expected_red_roster_join`; this alias only shortens the
/// path at the suppression site, which is the one place both grounds are decided.
use crate::v1_compiler_expected_red_roster_join::ExpectedRedSuppressionGround as SuppressionGround;
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

pub(crate) fn floor_inventory_content_digest(inventory: &[PreparedSourceView]) -> String {
    use crate::v1_rt::{atom_identity_hash, hash_combine};
    let mut entries: Vec<(&str, &str)> = inventory
        .iter()
        .map(|e| (e.source.path.as_str(), e.source.content.as_str()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut h = atom_identity_hash("floor-prepared-inventory".to_string());
    for (path, content) in entries {
        h = hash_combine(h, atom_identity_hash(path.to_string()));
        h = hash_combine(h, atom_identity_hash(content.to_string()));
    }
    h
}

/// Raw-pipeline `--target dag` compile-clean over a source set. NOT the floor's
/// receipt path (that is `floor_compile_clean_emit_ok_via_index`, which shares the
/// process's typed-module universe) — this is the index-independent oracle behind
/// `witness_layer_roots_compile_clean_emit_check` (cargo tests, enrolled witnesses),
/// and precisely BECAUSE it shares no caches with the via-index path it is the
/// standing second opinion for verdict equivalence
/// (`compile_clean_via_index_verdict_equivalence` tests).
pub(crate) fn floor_compile_clean_emit_ok(
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index: Option<&MultiEntryIndex>,
) -> bool {
    use crate::v1_compiler_artifact::RenderTarget;
    let options = compile_clean_pipeline_options_for_sources(index, &sources);
    let result = v1_compiler_compile::compile_sources_with_options(
        Rc::new(sources.into()),
        RenderTarget::Dag,
        options,
    );
    let has_hard_errors = compile_clean_pipeline_has_hard_errors(result.diagnostics.as_ref());
    if has_hard_errors {
        eprint_compile_clean_hard_diagnostics(result.diagnostics.as_ref());
    } else if result.files.is_empty() {
        eprintln!("floor compile-clean: refused — compile produced zero files (empty emit set)");
    }
    !has_hard_errors && !result.files.is_empty()
}

/// The floor receipt's compile: the same source closure as the raw oracle, routed
/// through the shared `MultiEntryIndex` cached path (`resolved_graph_from_sources_with_index`)
/// so every module's content-keyed typecheck is computed once per process and reused
/// by batch-2's witness resolves (PR #6766 receipts: verdict-equivalent green AND on
/// the planted `GUNBC_TEST_FLOOR_COMPILE_CLEAN_INJECT_UNRESOLVED` red; warm heavy
/// witnesses 1.0–1.4s vs 10–90s cold; red verdict at the failing stage in seconds).
/// The emit leg (`--target dag` render) runs over the already-typed graph.
pub(crate) fn floor_compile_clean_emit_ok_via_index(
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index_roots: &[String],
) -> (bool, String) {
    use crate::v1_compiler_artifact::RenderTarget;
    use crate::v1_compiler_complexity::empty_complexity_report;
    let index = process_shared_index(index_roots);
    let census_only = compile_clean_census_only_sources_for_compiled(&index, &sources);
    // #8204 out-of-closure fill runs independently of resolve: an earlier
    // resolution refusal must not mask a parse refusal in the rest of the indexed pool.
    let census_fill_diags = compile_clean_census_fill_hard_diagnostics(&census_only);
    let (graph, si, compile_clean_diags) = match resolved_graph_from_sources_with_index(
        &index,
        sources,
        "floor-compile-clean-gate",
        // Ephemeral: the whole-tree aggregate graph must NOT join the process share tier
        // (D0.1) — it would pin every TypedModule in the tree for the process lifetime.
        ResolvedGraphMemoShare::Ephemeral,
    ) {
        Ok(resolved) => resolved,
        Err(msg) => {
            if !census_fill_diags.is_empty() {
                eprint_compile_clean_hard_diagnostics(&census_fill_diags);
                let census_msg = format_first_compile_clean_hard_diagnostic(&census_fill_diags);
                eprintln!("compile-clean: hard diagnostics:\n{msg}");
                return (false, format!("{census_msg}\ncompile-clean: {msg}"));
            }
            eprintln!("compile-clean: hard diagnostics:\n{msg}");
            return (false, format!("compile-clean: {msg}"));
        }
    };
    let all_compile_clean_diags = if census_fill_diags.is_empty() {
        compile_clean_diags.clone()
    } else {
        let mut merged = compile_clean_diags
            .iter()
            .cloned()
            .collect::<im::Vector<_>>();
        merged.extend(census_fill_diags.iter().cloned());
        Rc::new(merged)
    };
    if compile_clean_pipeline_has_hard_errors(all_compile_clean_diags.as_ref()) {
        eprint_compile_clean_hard_diagnostics(all_compile_clean_diags.as_ref());
        return (
            false,
            format_first_compile_clean_hard_diagnostic(all_compile_clean_diags.as_ref()),
        );
    }
    let newline_indices: Rc<im::Vector<Rc<NewlineIndex>>> =
        Rc::new(si.values().cloned().collect::<im::Vector<_>>());
    let resolved = Rc::new(v1_compiler_compile::ResolvedPipelineResult {
        graph: Some(graph),
        diagnostics: Rc::new(im::Vector::new()),
        source_indices: si,
        complexity: empty_complexity_report(),
        ownership: Rc::new(im::Vector::new()),
        newline_indices,
    });
    let result = v1_compiler_compile::emit_resolved_for_target(resolved, RenderTarget::Dag);
    if result.files.is_empty() {
        let detail = "floor compile-clean: refused — compile produced zero files (empty emit set)"
            .to_string();
        eprintln!("{detail}");
        return (false, detail);
    }
    (true, String::new())
}

pub fn floor_compile_clean_receipt_installed() -> bool {
    FLOOR_COMPILE_CLEAN_RECEIPT
        .lock()
        .ok()
        .map(|g| g.is_some())
        .unwrap_or(false)
}

pub fn floor_walk_attempt_id_from_env() -> String {
    std::env::var("GUNBC_FLOOR_WALK_ATTEMPT_ID")
        .or_else(|_| std::env::var("GUNBC_WALK_ATTEMPT_ID"))
        .unwrap_or_else(|_| "local".to_string())
}

/// Why the discovery authority's per-file fold produced no roster. Three causes, three repairs: a
/// source's test-marked declarations could not be READ (it does not parse, so which of its
/// declarations are tests is undeterminable), the authority could not be EVALUATED (a frame or
/// interpreter defect, located at the source when one was being folded), or it evaluated and
/// REFUSED the corpus (a placement or live_tree_disposition violation, carried as the producer's
/// own reason).
pub(crate) enum FloorDiscoveryFoldRefusal {
    Unparseable {
        source: String,
        cause: String,
    },
    Unevaluable {
        source: Option<String>,
        function: &'static str,
        error: String,
    },
    Refused {
        reason: String,
    },
}

impl FloorDiscoveryFoldRefusal {
    pub(crate) fn rendered(&self) -> String {
        match self {
            FloorDiscoveryFoldRefusal::Unparseable { source, cause } => {
                format!("cause=FloorDiscoveryTestMarkersUnreadable source={source} — {cause}")
            }
            FloorDiscoveryFoldRefusal::Unevaluable {
                source: Some(source),
                function,
                error,
            } => format!(
                "cause=FloorDiscoveryAuthorityUnevaluable source={source} — {function}: {error}"
            ),
            FloorDiscoveryFoldRefusal::Unevaluable {
                source: None,
                function,
                error,
            } => format!("cause=FloorDiscoveryAuthorityUnevaluable — {function}: {error}"),
            FloorDiscoveryFoldRefusal::Refused { reason } => {
                format!("cause=FloorDiscoveryRefused — {reason}")
            }
        }
    }
}

/// THE FLOOR'S DISCOVERY, AS ONE FOLD OVER WHATEVER SOURCES THE CALLER HOLDS.
///
/// `v2.workflow.floor_discovery_source_authority` `discover_floor_rows_for_source` per source, then
/// `floor_discovery_finalize_source_outcomes` over the outcomes -- the one authority for which
/// `test` declarations a source enrolls. The required floor folds it over its full inventory;
/// `gunbc test`'s claim route folds it over the modules its operand names. Same authority, same
/// calls, narrower subject: the subject narrows, the decision does not move (review 44641's rule).
/// `frame` must have that module in scope by its qualified name.
///
/// THE MARKER FACT IS SUPPLIED, NOT SCANNED FOR. The authority neither parses nor resolves the
/// files it enumerates, so each source's `marked` argument is the names of the declarations
/// carrying the authored `test` marker, read once by `test_marker::test_marked_decl_names` (the
/// seed's single consumer of the grammar's mark). A source that does not parse refuses here
/// (`Unparseable`); no text scan stands in for the missing fact.
pub(crate) fn floor_discovery_rows_over_sources<'a>(
    frame: &v1_interpreter::InterpContext,
    sources: impl IntoIterator<Item = &'a v1_compiler_compile::SourceFile>,
) -> Result<Vec<DiscoveryRow>, FloorDiscoveryFoldRefusal> {
    let mut outcomes: Vec<v1_interpreter::Value> = Vec::new();
    for source in sources {
        let repo_path = source.path.replace('\\', "/");
        let marked =
            test_marker::test_marked_decl_names(&source.content, &repo_path).map_err(|cause| {
                FloorDiscoveryFoldRefusal::Unparseable {
                    source: repo_path.clone(),
                    cause,
                }
            })?;
        let args = [
            (Some("repo_path".to_string()), str_value(repo_path.clone())),
            (
                Some("marked".to_string()),
                list_value_from_vec(marked.into_iter().map(str_value).collect()),
            ),
            (
                Some("content".to_string()),
                str_value(source.content.clone()),
            ),
        ];
        let outcome = v1_interpreter::run_in_context_with_args(
            frame,
            "v2.workflow.floor_discovery_source_authority.discover_floor_rows_for_source",
            &args,
            false,
        )
        .map_err(|e| FloorDiscoveryFoldRefusal::Unevaluable {
            source: Some(repo_path),
            function: "discover_floor_rows_for_source",
            error: e.to_string(),
        })?;
        outcomes.push(outcome);
    }
    let finalized = v1_interpreter::run_in_context_with_args(
        frame,
        "v2.workflow.floor_discovery_source_authority.floor_discovery_finalize_source_outcomes",
        &[(Some("outcomes".to_string()), list_value_from_vec(outcomes))],
        false,
    )
    .map_err(|e| FloorDiscoveryFoldRefusal::Unevaluable {
        source: None,
        function: "floor_discovery_finalize_source_outcomes",
        error: e.to_string(),
    })?;
    parse_floor_discovery_producer_result(frame, &finalized)
        .map_err(|reason| FloorDiscoveryFoldRefusal::Refused { reason })
}

/// Read the grounded opaque-host-call surface, or refuse.
///
/// RETURNS THE OPERATIONS, NEVER AN EMPTY VEC ON FAILURE. `opaque_host_call_surface()` answers a
/// coproduct whose ungrounded arm carries WHY the join broke (`unresolved` / `not_free_call`),
/// and both of those are reported here rather than collapsed into one sentence, because they
/// have different repairs: an unresolved identity is a roster row pointing at nothing, while a
/// not-free-call arm is an identity whose dispatch shape the criterion does not admit.
pub(crate) fn floor_required_opaque_host_call_surface(
    ctx: &v1_interpreter::InterpContext,
) -> Result<Vec<String>, String> {
    let qualified = "gunbc.v1.v1_interpreter_opaque_host_call.opaque_host_call_surface";
    let value = v1_interpreter::run_in_context(ctx, qualified, false)
        .map_err(|e| format!("{qualified}: {e}"))?;
    let v1_interpreter::Value::Variant {
        variant_name,
        fields,
        ..
    } = &value
    else {
        return Err(format!(
            "{qualified}: expected an OpaqueHostCallSurface variant, got {}",
            floor_value_shape(Some(&value))
        ));
    };
    match ctx.resolve(*variant_name).as_str() {
        "OpaqueHostCallSurfaceGrounded" => {
            let operations = floor_decode_list(ctx, ctx.field(fields, "operations"))?;
            operations
                .into_iter()
                .map(|v| match v {
                    v1_interpreter::Value::Str(s) => Ok(s.to_string()),
                    other => Err(format!(
                        "{qualified}: operations carries a non-string member: {}",
                        floor_value_shape(Some(other))
                    )),
                })
                .collect()
        }
        "OpaqueHostCallSurfaceUngrounded" => Err(format!(
            "{qualified}: the opaque-host-call surface is UNGROUNDED, so which arms are \
             unpollable is unreadable and no claim's preemption reachability can be observed. \
             The floor refuses rather than arming an empty surface, which would report every \
             completed-over-ceiling crossing as an ordinary overshoot. Repair the join in \
             gunbc.v1.v1_interpreter_opaque_host_call: unresolved={:?} not_free_call={:?} \
             wrong_dispatch_site={:?}",
            floor_value_shape(ctx.field(fields, "unresolved")),
            floor_value_shape(ctx.field(fields, "not_free_call")),
            // THE THIRD POPULATION, AND IT HAS ITS OWN REPAIR. `unresolved` is a roster row
            // pointing at nothing and `not_free_call` is an arm whose dispatch shape the
            // criterion does not admit; `wrong_dispatch_site` is an arm that IS a free call but
            // is dispatched somewhere other than `eval_builtin_inner`, so the recorder installed
            // there can never observe it. Reporting it as either of the other two would send a
            // repairer to the wrong authority.
            floor_value_shape(ctx.field(fields, "wrong_dispatch_site")),
        )),
        other => Err(format!(
            "{qualified}: unknown OpaqueHostCallSurface arm {other:?}"
        )),
    }
}

pub(crate) fn floor_git_diff_range() -> Result<String, String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_git_diff_unified_for_ci", false)
            .map_err(|e| format!("floor_observe_git_diff_unified_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffOk") => match ctx.field(fields, "text") {
            Some(Value::Str(s)) => Ok(s.to_string()),
            _ => Err("UnifiedDiffOk missing `text` field".to_string()),
        },
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffFail") => match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Err(r.to_string()),
            _ => Err("git diff observation failed (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_git_diff_unified_for_ci returned `{}`, expected FloorUnifiedDiffResult",
            ctx.format_value(other)
        )),
    }
}

/// Read back the baseline the affected-set diff was taken against — a PROJECTION of
/// `resolve_diff_baseline` (`v2.workflow.floor_diff_observe`
/// `floor_observe_diff_baseline_readout_for_ci`), never a second derivation. Used only
/// to LOCATE the no-observed-change diagnostic: a receipt that says "zero changed paths"
/// without naming the base it compared against cannot be acted on, because the
/// interesting case is precisely a base that names the head commit itself. Failure to
/// read it is not fatal here — the diagnostic degrades to an unnamed baseline and says
/// so, rather than suppressing the state.
/// Read the resolved COMPARISON WINDOW — base, head and relation — from
/// `v2.workflow.floor_diff_observe` `floor_observe_diff_comparison_readout_for_ci`. A projection of
/// `resolve_diff_baseline`, never a second derivation.
///
/// Distinct from `floor_diff_baseline_readout` beside it, and the distinction is the point: that one
/// answers "which ref" for a diagnostic, and a DECIDING consumer that takes it has to invent the
/// missing head and relation. Inventing them is what shipped the merge-base-on-every-arm fail-open,
/// so the deciding consumers read this one and a `ComparisonReadoutRefused` propagates.
pub(crate) fn floor_diff_comparison_readout() -> Result<FreezeBaselineComparison, String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_diff_comparison_readout_for_ci", false)
            .map_err(|e| format!("floor_observe_diff_comparison_readout_for_ci: {e}"))?;
    let str_field = |fields: &_, name: &str| -> Result<String, String> {
        match ctx.field(fields, name) {
            Some(Value::Str(s)) => Ok(s.to_string()),
            _ => Err(format!("comparison readout missing `{name}`")),
        }
    };
    // The baseline KIND is a closed coproduct in `gunbc.diff_baseline`, so an unrecognized arm is an
    // unmodeled state rather than a formatting question: it refuses instead of rendering a guess.
    let kind_name = |fields: &_| -> Result<String, String> {
        match ctx.field(fields, "kind") {
            Some(Value::Variant { variant_name, .. }) => {
                for name in [
                    "MergeTargetBaseline",
                    "ExactReplayBaseline",
                    "PushBeforeBaseline",
                    "PushParentBaseline",
                    "MergeGroupBaseBaseline",
                    "OperatorOverrideBaseline",
                ] {
                    if ctx.sym_eq(*variant_name, name) {
                        return Ok(name.to_string());
                    }
                }
                Err("comparison readout carries an unmodeled DiffBaselineKind arm".to_string())
            }
            _ => Err("comparison readout missing `kind`".to_string()),
        }
    };
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "DirectComparison") => {
            Ok(FreezeBaselineComparison::Direct {
                base: str_field(fields, "base")?,
                head: str_field(fields, "head")?,
                kind: kind_name(fields)?,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "MergeBaseComparison") => {
            Ok(FreezeBaselineComparison::MergeBase {
                base: str_field(fields, "base")?,
                head: str_field(fields, "head")?,
                kind: kind_name(fields)?,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "ComparisonReadoutRefused") => {
            match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => Err(r.to_string()),
                _ => Err("comparison readout refused (no reason)".to_string()),
            }
        }
        other => Err(format!(
            "floor_observe_diff_comparison_readout_for_ci returned `{}`, expected \
             FloorDiffComparisonReadout",
            ctx.format_value(other)
        )),
    }
}

/// THE FLOOR RUN'S BASE TREE, resolved at most once per run and lent to every base-tree reader.
/// `gunbc.recurring_failure_mode` `a_roster_edit_judged_against_the_base_tip_not_the_merge_base`:
/// the interface planner applied the merge-base arm while the unimported-bare-provider roster gate
/// and the cost-debt edit budget each read the base TIP, so on a branch behind main a row main
/// retired after the branch point read as this change's edit. The repair is not a merge-base call
/// at each site -- that leaves N relation choices to drift -- but one window, resolved by
/// `FreezeBaselineComparison::resolve_window`, that every reader receives. Lazy because a local
/// run without a diff baseline never needs a base side, and a refusal is held so every reader
/// reports the same cause rather than re-asking the resolver.
pub(crate) struct FloorBaseTree {
    /// `None` reads the run's own comparison (`floor_diff_comparison_readout`) at the process
    /// workspace; `Some` is a supplied comparison over a supplied repository, the same resolver.
    supplied: Option<(FreezeBaselineComparison, PathBuf)>,
    window: std::cell::OnceCell<
        Result<
            (
                FreezeBaselineComparison,
                comparison_window::ResolvedComparisonWindow,
            ),
            String,
        >,
    >,
}

impl FloorBaseTree {
    pub(crate) fn unresolved() -> Self {
        Self {
            supplied: None,
            window: std::cell::OnceCell::new(),
        }
    }

    /// A supplied comparison over a supplied repository, resolved by the same `resolve_window`.
    pub(crate) fn for_comparison(comparison: FreezeBaselineComparison, root: PathBuf) -> Self {
        Self {
            supplied: Some((comparison, root)),
            window: std::cell::OnceCell::new(),
        }
    }

    fn window(
        &self,
    ) -> Result<
        &(
            FreezeBaselineComparison,
            comparison_window::ResolvedComparisonWindow,
        ),
        String,
    > {
        self.window
            .get_or_init(|| {
                let (comparison, root) = match &self.supplied {
                    Some((comparison, root)) => (comparison.clone(), root.clone()),
                    None => (floor_diff_comparison_readout()?, process_workspace_root()),
                };
                let window = comparison.resolve_window(&root)?;
                Ok((comparison, window))
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    /// The commit whose tree is the base side of this run's comparison.
    pub(crate) fn commit(&self) -> Result<&comparison_window::BaseTreeCommit, String> {
        Ok(self.window()?.1.base_tree())
    }

    /// The resolved head commit of this run's comparison.
    pub(crate) fn head(&self) -> Result<&str, String> {
        Ok(self.window()?.1.head())
    }

    /// The base REF and the base-tree commit, for a located line. Never a tree route.
    pub(crate) fn located(&self) -> Result<String, String> {
        let (comparison, window) = self.window()?;
        Ok(format!(
            "base_ref={} base_tree={}",
            comparison.base_ref(),
            window.base_tree()
        ))
    }
}

pub(crate) fn floor_diff_baseline_readout() -> Result<(String, String), String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_diff_baseline_readout_for_ci", false)
            .map_err(|e| format!("floor_observe_diff_baseline_readout_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "BaselineReadout") => {
            let base = match ctx.field(fields, "ref") {
                Some(Value::Str(s)) => s.to_string(),
                _ => return Err("BaselineReadout missing `ref`".to_string()),
            };
            let event = match ctx.field(fields, "event_name") {
                Some(Value::Str(s)) => s.to_string(),
                _ => return Err("BaselineReadout missing `event_name`".to_string()),
            };
            Ok((base, event))
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "BaselineReadoutRefused") => match ctx.field(fields, "reason")
        {
            Some(Value::Str(r)) => Err(r.to_string()),
            _ => Err("baseline readout refused (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_diff_baseline_readout_for_ci returned `{}`, expected FloorDiffBaselineReadout",
            ctx.format_value(other)
        )),
    }
}

pub(crate) fn floor_git_diff_name_status_range() -> Result<(Vec<String>, HashSet<String>), String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_git_diff_name_status_for_ci", false)
            .map_err(|e| format!("floor_observe_git_diff_name_status_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "NameStatusDiffOk") => {
            let changed = match ctx.field(fields, "changed_paths") {
                Some(v) => string_list_from_value(v, "changed_paths")?,
                None => return Err("NameStatusDiffOk missing `changed_paths` field".to_string()),
            };
            let departed = match ctx.field(fields, "departed_paths") {
                Some(v) => string_list_from_value(v, "departed_paths")?,
                None => return Err("NameStatusDiffOk missing `departed_paths` field".to_string()),
            };
            Ok((
                changed.iter().map(|p| normalize_repo_path(p)).collect(),
                departed.iter().map(|p| normalize_repo_path(p)).collect(),
            ))
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "NameStatusDiffFail") => match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Err(r.to_string()),
            _ => Err("git diff --name-status observation failed (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_git_diff_name_status_for_ci returned `{}`, expected FloorNameStatusDiffResult",
            ctx.format_value(other)
        )),
    }
}

/// One file's content at the resolved diff base (`v2.workflow.floor_diff_observe`
/// `floor_run_base_file_read`): `Ok(None)` when the base does not carry the path, `Err` when the
/// listing or show refused -- a refusal is never read as an absent or empty file.
pub(crate) fn floor_base_file_read(
    base_tree: &FloorBaseTree,
    path: &str,
) -> Result<Option<String>, String> {
    use v1_interpreter::Value;
    let base = base_tree.commit()?.as_str();
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let args = [
        (Some("base".to_string()), str_value(base)),
        (Some("path".to_string()), str_value(path)),
    ];
    let result =
        v1_interpreter::run_in_context_with_args(&ctx, "floor_run_base_file_read", &args, false)
            .map_err(|e| format!("floor_run_base_file_read: {e}"))?;
    match &result {
        Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "BaseFileAbsent") => {
            Ok(None)
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "BaseFileRead") => match ctx.field(fields, "content") {
            Some(Value::Str(c)) => Ok(Some(c.to_string())),
            _ => Err("BaseFileRead missing `content`".to_string()),
        },
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "BaseFileReadRefused") => {
            match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => Err(format!("base read of {path} refused: {r}")),
                _ => Err(format!("base read of {path} refused (no reason)")),
            }
        }
        other => Err(format!(
            "floor_run_base_file_read returned an unexpected value: {}",
            ctx.format_value(other)
        )),
    }
}

pub(crate) fn floor_diff_edits_from_diff_text(
    index: &MultiEntryIndex,
    diff_text: &str,
) -> Result<FloorDiffEdits, String> {
    let line_ranges = parse_unified_diff_line_ranges(diff_text);
    let departed = parse_unified_diff_departed_paths(diff_text);
    let added = parse_unified_diff_added_paths(diff_text);
    // No census: `enrolled_test_fns` stays empty. Attribution tests read `edited_test_fns` /
    // `touched_entry_files` / data items from this wrapper; enrolment is only answered when
    // `floor_diff_edits_from_diff_text_with_base_names` (or production) supplies the census.
    floor_diff_edits_from_line_ranges(
        index,
        &line_ranges,
        &departed,
        &added,
        None,
        &std::collections::HashMap::new(),
    )
}

pub(crate) fn floor_diff_edits_from_diff_text_with_base_names(
    index: &MultiEntryIndex,
    diff_text: &str,
    base_test_decl_names: &std::collections::HashMap<String, HashSet<String>>,
) -> Result<FloorDiffEdits, String> {
    let line_ranges = parse_unified_diff_line_ranges(diff_text);
    let departed = parse_unified_diff_departed_paths(diff_text);
    let added = parse_unified_diff_added_paths(diff_text);
    let rename_from = parse_unified_diff_rename_sources(diff_text);
    floor_diff_edits_from_line_ranges(
        index,
        &line_ranges,
        &departed,
        &added,
        Some(base_test_decl_names),
        &rename_from,
    )
}

/// `floor_diff_edits_from_diff_text_with_base_names` with each changed path's new-side
/// content SUPPLIED rather than read from the working tree.
#[cfg(test)]
pub(crate) fn floor_diff_edits_from_diff_text_with_base_names_and_sources(
    index: &MultiEntryIndex,
    diff_text: &str,
    base_test_decl_names: &std::collections::HashMap<String, HashSet<String>>,
    sources: &std::collections::HashMap<String, String>,
) -> Result<FloorDiffEdits, String> {
    floor_diff_edits_from_line_ranges_reading(
        index,
        &parse_unified_diff_line_ranges(diff_text),
        &parse_unified_diff_departed_paths(diff_text),
        &parse_unified_diff_added_paths(diff_text),
        Some(base_test_decl_names),
        &parse_unified_diff_rename_sources(diff_text),
        &|path: &str| Ok(sources.get(path).cloned()),
    )
}

// Host realization under a declared scaffold: the governing row is the `SCAFFOLD (DESIGN
// §6–§7)` declaration above `FloorDiffEdits` in `cli_run`, which owns this function's
// reason, dissolve-on trigger and census. Read it there; it is not restated here.
pub(crate) fn floor_diff_edits_from_line_ranges(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
    departed_paths: &HashSet<String>,
    added_paths: &HashSet<String>,
    base_test_decl_names: Option<&std::collections::HashMap<String, HashSet<String>>>,
    rename_from: &std::collections::HashMap<String, String>,
) -> Result<FloorDiffEdits, String> {
    floor_diff_edits_from_line_ranges_reading(
        index,
        line_ranges_by_file,
        departed_paths,
        added_paths,
        base_test_decl_names,
        rename_from,
        &read_working_tree_source,
    )
}

/// The production reader: a changed path's content at the working tree. `Ok(None)` is
/// "absent from the tree", which the caller dispositions against the diff's departed set.
fn read_working_tree_source(file_norm: &str) -> Result<Option<String>, String> {
    let disk_path = process_workspace_root().join(file_norm);
    if !disk_path.is_file() {
        return Ok(None);
    }
    std::fs::read_to_string(&disk_path)
        .map(Some)
        .map_err(|e| format!("read failed for {file_norm}: {e}"))
}

/// Attribution over a supplied reader of each changed path's new-side content. The
/// interface is diff + census + content -> edits, so a witness supplies the content as a
/// value (DESIGN §3 witness rule) rather than reading whatever the live tree now holds.
#[allow(clippy::too_many_arguments)]
pub(crate) fn floor_diff_edits_from_line_ranges_reading(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
    departed_paths: &HashSet<String>,
    added_paths: &HashSet<String>,
    base_test_decl_names: Option<&std::collections::HashMap<String, HashSet<String>>>,
    rename_from: &std::collections::HashMap<String, String>,
    read_source: &dyn Fn(&str) -> Result<Option<String>, String>,
) -> Result<FloorDiffEdits, String> {
    let mut overlapping_data_items = HashSet::new();
    let mut edited_test_fns = HashSet::new();
    let mut enrolled_test_fns = HashSet::new();
    let mut touched_entry_files = HashSet::new();
    let mut touched_declarations: HashSet<(String, String)> = HashSet::new();
    // #6269 attributes src/v1/ .dag changes through a dedicated index; the structural-∅ fix
    // dropped the saw_non_dag/saw_dag refusal (a non-.dag-only diff is a nominal empty frontier,
    // handled by the `continue` arm below), so neither flag is needed here.
    let v1_attribution_index = if line_ranges_by_file
        .keys()
        .any(|p| normalize_repo_path(p).starts_with("src/v1/"))
    {
        Some(build_v1_attribution_multi_entry_index())
    } else {
        None
    };
    for (file_path, ranges) in line_ranges_by_file {
        if !file_path.ends_with(".dag") {
            // A non-.dag changed path is a structural-∅ for the .dag frontier: it declares no
            // .dag nodes, so there is nothing to attribute, and its coverage lives in the Rust
            // gates (rust_tests), not the .dag witnesses. Skipping it yields an empty .dag
            // frontier -- the SAME nominal outcome as an empty diff. This is NOT ignorance: the
            // only ignorance state is a failed git-diff observation (UnifiedDiffFail upstream,
            // floor_diff_observe.dag; operator ruling 2026-07-05). Structural-∅ and ignorance
            // are different states -- the mirror of the departed-.dag-path arm below.
            continue;
        }
        let file_norm = normalize_repo_path(file_path);
        let Some(content) = read_source(&file_norm)? else {
            if departed_paths.contains(&file_norm) {
                // Departed per the diff (deletion / rename-from): its decl set
                // is empty by construction — the file has no declarations to
                // attribute. The path-grain fact stays in changed_paths;
                // dependents that imported it fail loudly at their own resolve.
                continue;
            }
            // Absent from the tree but NOT marked departed by the diff: the
            // observation is incoherent (stale tree, quoting artifact, bogus
            // path). Structural-∅ and ignorance are different states — refuse.
            return Err(format!(
                "affected-set derivation refused: diff names {file_path} with \
                 content changes but the path is absent from the working tree \
                 and the diff does not mark it departed (deletion/rename)"
            ));
        };
        let resolve_index = if file_norm.starts_with("src/v1/") {
            v1_attribution_index.as_ref().expect("v1 attribution index")
        } else {
            index
        };
        // Attribution is a PARSE-grade fact: it needs each touched file's
        // declaration line map (names + spans + data/fn kind), never its typecheck.
        // The former full entry RESOLVE here made every touched file's typecheck
        // health gate the whole frontier — on a corpus-wide diff, one latent-red
        // module (a batch-2 debt row no gate compiles) dead-ended batch 2 at one
        // entry per CI cycle. Parse errors still refuse (typed, located); typecheck
        // reds surface where they belong — as that entry's own counted discovery row.
        let source = Rc::new(v1_compiler_compile::SourceFile {
            path: file_norm.clone(),
            content: content.clone(),
        });
        let (module_node, nl) = match parse_module_node_from_index_source(resolve_index, source) {
            Ok(pair) => pair,
            Err(e) => return Err(format!("parse failed for {file_path}: {e}")),
        };
        let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
            let mut m = HashMap::new();
            m.insert(file_norm.clone(), nl.clone());
            m
        });
        // A test is a declaration carrying the authored `test` marker, read off the item the
        // parse above already produced: no second reading of the file, and no line scan.
        let mut test_fn_names: HashSet<String> = HashSet::new();
        let mut decls: Vec<(i64, String, bool)> = Vec::new();
        for item in crate::v1_std_core::module_items(module_node.clone()).iter() {
            let line = byte_to_line_col(nl.clone(), item.span.start).line;
            let name = authored_name_at(single_si.clone(), item.clone());
            let is_data = item_kind(item.clone()) == ItemKind::DataItem;
            if item.declaration_marker == crate::v1_std_core::DeclarationMarker::TestMarked {
                test_fn_names.insert(name.clone());
            }
            decls.push((line, name, is_data));
        }
        if decls.is_empty() {
            // A declaration-less module (a lone `module` line — e.g. the
            // shadow-masked fixtures) has nothing to attribute at decl grain;
            // its only edit surface IS the file, so the file-grain touched set
            // carries it (dependents rerun via the import closure). Refusing
            // here dead-ended the frontier on a fixture that is legitimately
            // empty — not an incoherent observation.
            touched_entry_files.insert(file_norm.clone());
            continue;
        }
        decls.sort_by_key(|(line, _, _)| *line);
        // A PATH WHOSE DECLARATION SET IS ESTABLISHED FRESH CONTRIBUTES EVERY DECLARATION IT
        // CARRIES, and the line ranges do not establish that set — they only report which lines
        // the diff happened to print.
        //
        // `parse_unified_diff_added_paths` already rules that both wholly-added files and RENAME
        // DESTINATIONS are new-at-path ("its declaration set is established fresh at NEW"), and
        // for a `/dev/null` add the two agree by accident: every line is a `+` line, so line
        // attribution reaches every declaration anyway. For a rename destination they do not.
        // Git detects the rename and prints only the hunks that differ, so a moved file's
        // declarations are attributed by WHICH LINES THE MOVE HAPPENED TO EDIT — while every
        // identity at the new path is a NEW qualified identity (`<authored module>.<function>`)
        // that has never executed under that spelling, because the authored module name moved
        // with the file.
        //
        // MEASURED, NOT ANTICIPATED (2026-08-31, `git rev-list HEAD` = 90 commits): 8 of 8
        // rename-destination `.dag` files carrying test decls were under-enrolled, 87 of their
        // 103 identities missed. gunbc#9823 is the receipt-confirmed instance — it renamed
        // `machine_shape_construction_wall_test.dag` into `v2.test.` with two `test fn`s, git
        // printed the module line and the removed trailing blank line at EOF, the EOF hunk fell
        // inside the LAST declaration, and exactly one of the two siblings was selected. The
        // wall's DISCRIMINATING RED (`gate_red_synthetic_machine_shape_call`) was the one missed.
        // It executed anyway there, via the ordinary `Planned` arm — but changed-witness
        // membership is precisely what OVERRIDES the cost-debt withhold and the outside-gate
        // suppression below, so the same miss on a rostered identity is a silent decline of a
        // witness whose author is present. That is the state
        // `v2.workflow.floor_changed_witness` was written for.
        //
        // THIS NARROWS NOTHING AND WIDENS NOTHING BEYOND THE PATH'S OWN DECLARATIONS: the
        // universe is this file's parsed decl list, not the corpus, so it is the precise answer
        // to "what does this path declare", never an absorbing "rerun everything" (DESIGN §5).
        let charged: HashSet<usize> = if added_paths.contains(&file_norm) {
            (0..decls.len()).collect()
        } else {
            // A DECLARATION IS CHANGED WHEN ITS OWN TEXT DIFFERS AT BASE AND HEAD, joined by
            // name -- never by which lines a hunk names. Git's `-U0` alignment is free to print a
            // deletion between two similar declarations as a hunk that starts or ends inside the
            // neighbour, and any line-ownership rule then charges a declaration nobody touched
            // (`gunbc.recurring_failure_mode` `a_pure_deletion_force_runs_its_unchanged_neighbour`,
            // both the gap form of gunbc#13103 and the leading-comment form of gunbc#13126). The
            // base is reconstructed from the head and the diff's own removed lines, so this is
            // the same single observation, read without its alignment.
            let head_lines: Vec<&str> = content.lines().collect();
            let starts: Vec<(i64, String)> =
                decls.iter().map(|(l, n, _)| (*l, n.clone())).collect();
            let attribution = declaration_text_attribution(&head_lines, &starts, ranges);
            // Module-line edits stay fail-closed for modifies — renaming can change entry
            // identity. Wholly-added files necessarily touch line 1.
            if attribution.module_line_changed {
                return Err(format!("diff before first declaration in {file_path}"));
            }
            if attribution.pre_declaration_changed {
                touched_entry_files.insert(file_norm.clone());
            }
            attribution.changed
        };
        for (i, (_, name, is_data)) in decls.iter().enumerate() {
            if !charged.contains(&i) {
                continue;
            }
            if test_fn_names.contains(name) {
                edited_test_fns.insert((file_norm.clone(), name.clone()));
                // NEWLY ENROLLED = declared now and not declared at the resolved diff base.
                // The census is path-keyed. A rename destination is absent at the NEW path, so
                // looking up the dest would enrol every fn as new — the author's-only-moved
                // case. Enrolment therefore reads the SOURCE path when `rename_from` names one.
                // Without a census there is no enrolment answer: the added-path rule is the
                // superseded rung and must not remain as a silent alternative (review 64246).
                let newly_declared = match base_test_decl_names {
                    Some(at_base) => {
                        let census_path = rename_from.get(&file_norm).unwrap_or(&file_norm);
                        !at_base
                            .get(census_path)
                            .is_some_and(|names| names.contains(name))
                    }
                    None => false,
                };
                if newly_declared {
                    enrolled_test_fns.insert((file_norm.clone(), name.clone()));
                }
            } else if *is_data {
                overlapping_data_items.insert((file_norm.clone(), name.clone()));
                touched_declarations.insert((file_norm.clone(), name.clone()));
            } else {
                touched_entry_files.insert(file_norm.clone());
                touched_declarations.insert((file_norm.clone(), name.clone()));
            }
        }
    }
    // A present diff whose changed paths are all non-.dag lands here with an empty frontier
    // (structural-∅): it flows through as every row's not-affected skip -- nominal and
    // transparent, exactly like an empty diff, never a refusal. Observation failure (the only
    // ignorance state) is refused upstream in floor_git_diff_range; a successful observation
    // with an empty .dag subset is not ignorance.
    Ok(FloorDiffEdits {
        overlapping_data_items,
        edited_test_fns,
        enrolled_test_fns,
        touched_entry_files,
        touched_declarations,
    })
}

/// What one modified file's declarations look like once each is compared with itself at the
/// diff base. `changed` holds indices into the head declaration list the caller supplied.
struct DeclarationTextAttribution {
    module_line_changed: bool,
    pre_declaration_changed: bool,
    changed: HashSet<usize>,
}

/// A declaration's OWN text, ANNOTATION-ERASED: from its first line up to the next declaration,
/// with every `//` annotation line removed and the trailing blank lines dropped. DESIGN §4c:
/// semantic passes receive only the annotation-erased projection, so adding, deleting, editing or
/// moving a `//` block never changes this text and never charges the declaration. Whitespace is
/// otherwise compared as is.
///
/// A `.dag` string literal may span lines (`scan_string_body` does not stop at a newline), so a
/// line that begins with `//` INSIDE a string is program data, not an annotation, and is kept:
/// `annotation_line_mask` reads the lexical state at each line start.
fn declaration_own_text(lines: &[String], start: usize, next: usize) -> String {
    let span = &lines[start..next.min(lines.len()).max(start)];
    let mut kept: Vec<&str> = span
        .iter()
        .zip(annotation_line_mask(span))
        .filter(|(_, annotation)| !annotation)
        .map(|(l, _)| l.as_str())
        .collect();
    while kept.last().is_some_and(|l| l.trim().is_empty()) {
        kept.pop();
    }
    kept.join("\n")
}

/// For each line, whether it is a `//` annotation line: it starts (after indentation) with `//`
/// while the lexer is in code, not inside a string literal. Strings carry `\` escapes and `${ }`
/// interpolation whose code may itself open strings, so the state is a stack: `Str` inside a
/// literal, `Code(depth)` inside an interpolation's braces.
fn annotation_line_mask(lines: &[String]) -> Vec<bool> {
    enum Frame {
        Str,
        Code(usize),
    }
    let mut stack: Vec<Frame> = Vec::new();
    let mut mask = Vec::with_capacity(lines.len());
    for line in lines {
        let in_string = matches!(stack.last(), Some(Frame::Str));
        mask.push(!in_string && line.trim_start().starts_with("//"));
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match stack.last_mut() {
                Some(Frame::Str) => match c {
                    '\\' => i += 1,
                    '"' => {
                        stack.pop();
                    }
                    '$' if chars.get(i + 1) == Some(&'{') => {
                        stack.push(Frame::Code(0));
                        i += 1;
                    }
                    _ => {}
                },
                code => {
                    if c == '/' && chars.get(i + 1) == Some(&'/') {
                        break;
                    }
                    match (c, code) {
                        ('"', _) => stack.push(Frame::Str),
                        ('{', Some(Frame::Code(d))) => *d += 1,
                        ('}', Some(Frame::Code(0))) => {
                            stack.pop();
                        }
                        ('}', Some(Frame::Code(d))) => *d -= 1,
                        _ => {}
                    }
                }
            }
            i += 1;
        }
    }
    mask
}

/// Whether a line REMOVED by the diff opens a declaration at the base. Removed lines are not
/// parsed, so this reads layout: a column-0 line that is not a comment or a closing bracket. It
/// errs toward splitting -- a false split shortens a base declaration and so CHARGES its head
/// counterpart, and a missed one lengthens it and charges too; neither direction un-charges an
/// edited declaration.
fn removed_line_opens_declaration(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if first.is_whitespace() || matches!(first, '}' | ')' | ']' | '"') || line.starts_with("//") {
        return None;
    }
    const MARKERS: [&str; 6] = ["test", "pub", "fn", "data", "type", "probe"];
    line.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty() && !MARKERS.contains(w))
        .map(str::to_string)
        .next()
        .or_else(|| Some(String::new()))
}

/// Compare every head declaration with the base declaration of the same name. The base file is
/// the head with each hunk's new-side lines replaced by its removed lines, which `-U0` carries
/// in full, so the comparison needs no second read and no alignment.
fn declaration_text_attribution(
    head: &[&str],
    decls: &[(i64, String)],
    ranges: &[FileLineRange],
) -> DeclarationTextAttribution {
    let mut hunks: Vec<&FileLineRange> = ranges.iter().collect();
    hunks.sort_by_key(|r| r.new_start);
    let mut base: Vec<String> = Vec::new();
    let mut base_opens: Vec<(usize, String)> = Vec::new();
    let mut head_to_base: Vec<Option<usize>> = vec![None; head.len()];
    let mut h = 0usize;
    for r in hunks {
        let before = if r.new_count > 0 {
            r.new_start - 1
        } else {
            r.new_start
        };
        let before = (before.max(0) as usize).min(head.len());
        while h < before {
            head_to_base[h] = Some(base.len());
            base.push(head[h].to_string());
            h += 1;
        }
        for line in &r.removed {
            if let Some(name) = removed_line_opens_declaration(line) {
                base_opens.push((base.len(), name));
            }
            base.push(line.clone());
        }
        h = (h + r.new_count.max(0) as usize).min(head.len());
    }
    while h < head.len() {
        head_to_base[h] = Some(base.len());
        base.push(head[h].to_string());
        h += 1;
    }
    let head_owned: Vec<String> = head.iter().map(|l| l.to_string()).collect();
    for (line, name) in decls {
        if let Some(Some(b)) = head_to_base.get((*line - 1).max(0) as usize) {
            base_opens.push((*b, name.clone()));
        }
    }
    base_opens.sort();
    base_opens.dedup_by_key(|(b, _)| *b);
    let first_head = (decls[0].0 - 1).max(0) as usize;
    let first_base = base_opens.first().map(|(b, _)| *b).unwrap_or(base.len());
    let pre_declaration_changed = declaration_own_text(&head_owned, 0, first_head)
        != declaration_own_text(&base, 0, first_base);
    let module_line_changed = head_owned.first() != base.first();
    let mut changed = HashSet::new();
    for (i, (line, name)) in decls.iter().enumerate() {
        let start = (*line - 1).max(0) as usize;
        let next = decls
            .get(i + 1)
            .map(|(l, _)| (*l - 1).max(0) as usize)
            .unwrap_or(head_owned.len());
        let text = declaration_own_text(&head_owned, start, next);
        let unchanged = base_opens.iter().enumerate().any(|(j, (b, n))| {
            n == name && {
                let next_b = base_opens.get(j + 1).map(|(x, _)| *x).unwrap_or(base.len());
                declaration_own_text(&base, *b, next_b) == text
            }
        });
        if !unchanged {
            changed.insert(i);
        }
    }
    DeclarationTextAttribution {
        module_line_changed,
        pre_declaration_changed,
        changed,
    }
}

/// One projected row of the CHANGED-WITNESS PROJECTION. The standing vocabulary is
/// `v2.workflow.floor_changed_witness.ChangedWitnessExecutionStanding`; this struct is the
/// host's wire rendering of one arm applied to one changed identity, carrying the two receipt
/// facts the arm was derived from so the printed line never asserts a classification without
/// its inputs.
pub(crate) struct ChangedWitnessProjectionRow {
    pub identity: String,
    /// Wire name of the `ChangedWitnessExecutionStanding` arm: `planned-and-passed`,
    /// `planned-and-known-red-held`, `planned-without-terminal-verdict`, `declined`,
    /// `missing-disposition`.
    pub standing: &'static str,
    /// The disposition receipt's label for the identity (`required_floor_disposition_label`),
    /// or `absent` when no row exists.
    pub disposition: String,
    /// The terminal ledger's outcome wire (`claim_disposition_wire`), `not_executed` for a row
    /// that never ran, `absent` when there is no disposition row to speak for it.
    pub outcome: String,
    /// `changed_witness_standing_blocks` realized: passed, known-red-held, and a verdict-only
    /// pass with its cost published are green.
    pub blocks: bool,
    /// THE CAUSE THIS ROW HANDS ITS CONSUMER, and empty exactly on the rows that do not block.
    /// Mirror of `v2.workflow.floor_changed_witness` `changed_witness_blocking_cause`: it is
    /// chosen in the same expression that chooses `standing`, so the two cannot describe
    /// different rows. For a decline it is the disposition's own wire name, unprefixed, so this
    /// receipt and `required_floor_disposition.tsv` spell one fact one way.
    pub cause: String,
    /// The published cost observation, present exactly for a row that executed under
    /// `ChangedCostDebtVerdictOnly` and reached a verdict. `None` on every ordinary row, and on
    /// an override that published nothing — which is why that case reds rather than passing.
    pub cost: Option<ChangedWitnessCostObservation>,
}

/// The per-subject CPU line (`required_floor_per_subject_cpu_line_ms`), read out of the policy
/// model. One read shared by the enrolment-margin budget, which must sit strictly below it, and by
/// the Roster expensiveness ground, which admits only a live reading strictly above it.
pub(crate) fn floor_per_subject_cpu_line_ms(
    prepared: &crate::cli_run::PreparedRepository,
) -> Result<u64, String> {
    let policy_scope = claim_scope_for(prepared, REQUIRED_FLOOR_POLICY_MODULE)?;
    let policy_ctx = evaluation_frame(
        &policy_scope,
        v1_interpreter::ExecutionMode::Hermetic,
        None,
        None,
    );
    // THE CPU LINE, NOT THE WORK ENVELOPE (review 66007). This budget is compared against
    // an observed CPU reading, so the ceiling it must sit strictly below has to be
    // denominated on the same clock. `required_floor_claim_work_envelope_ms` is policy in
    // milliseconds of WORK, consumed only after conversion into eval steps; reading it here
    // judged a CPU figure against a line from another quantity because both are spelled in
    // milliseconds, which is the fusion `std.measure` `measure_clock_basis_note` forbids.
    // The two share the magnitude 500 and that coincidence is exactly how the fork got
    // written.
    floor_required_measure_count(&policy_ctx, "required_floor_per_subject_cpu_line_ms")
}

/// THE MARGIN BUDGET, READ OUT OF THE MODEL. Not a Rust literal and not arithmetic repeated here:
/// `v2.workflow.floor_enrolment_margin` `floor_enrolment_margin_budget_ms` derives it from the
/// ceiling authority and the measured p90 runner envelope, and this reads that derivation's own
/// answer. If the envelope is re-measured and the figure moves, the gate moves with it.
///
/// `v2.workflow.floor_enrolment_margin` is declared in `REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES`,
/// which is what makes evaluating it by name here legitimate rather than an undeclared reach.
pub(crate) fn floor_enrolment_margin_budget_ms(
    prepared: &crate::cli_run::PreparedRepository,
) -> Result<u64, String> {
    const MODULE: &str = "v2.workflow.floor_enrolment_margin";
    let scope = claim_scope_for(prepared, MODULE)?;
    let ctx = evaluation_frame(&scope, v1_interpreter::ExecutionMode::Hermetic, None, None);
    let qualified = format!("{MODULE}.floor_enrolment_margin_budget_ms_count");
    // STRICTLY POSITIVE AND STRICTLY BELOW THE CEILING. A zero or negative budget refuses every
    // witness before it evaluates one; a budget at or above the ceiling is the decoration the
    // model's own `the_enrolment_budget_is_strictly_below_the_ceiling` witness exists to forbid,
    // permanently green by construction while still being reported as coverage.
    let ceiling_ms = floor_per_subject_cpu_line_ms(prepared)?;
    match v1_interpreter::run_in_context(&ctx, &qualified, false) {
        Ok(v1_interpreter::Value::Int(n)) if n > 0 && (n as u64) < ceiling_ms => Ok(n as u64),
        Ok(v1_interpreter::Value::Int(n)) => Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=EnrolmentMarginBudgetUngrounded {qualified} returned \
             {n}ms against a {ceiling_ms}ms ceiling — a margin budget must be strictly positive \
             and strictly below the ceiling it sits inside, or it gates nothing"
        )),
        Ok(other) => Err(format!(
            "{qualified}: expected a positive Int, got {}",
            floor_value_shape(Some(&other))
        )),
        Err(e) => Err(format!("{qualified}: {e}")),
    }
}

/// `v2.workflow.floor_enrolment_margin` `enrolment_typed_cost_debt_identities`, decoded
/// from the frame the same way `floor_cost_debt_roster` is. Never a Rust-empty HashSet:
/// authoring a `floor_cost_debt_typed_admissions` row must reach this gate (review 65692).
pub(crate) fn floor_enrolment_typed_cost_debt_identities(
    prepared: &crate::cli_run::PreparedRepository,
) -> Result<HashSet<String>, String> {
    const MODULE: &str = "v2.workflow.floor_enrolment_margin";
    const QUALIFIED: &str =
        "v2.workflow.floor_enrolment_margin.enrolment_typed_cost_debt_identities";
    let scope = claim_scope_for(prepared, MODULE)?;
    let ctx = evaluation_frame(&scope, v1_interpreter::ExecutionMode::Hermetic, None, None);
    let value = v1_interpreter::run_in_context(&ctx, QUALIFIED, false)
        .map_err(|e| format!("{QUALIFIED}: {e}"))?;
    let items = floor_decode_list(&ctx, Some(&value)).map_err(|e| format!("{QUALIFIED}: {e}"))?;
    let mut out = HashSet::new();
    for item in items {
        match item {
            v1_interpreter::Value::Str(s) => {
                if !out.insert(s.to_string()) {
                    return Err(format!(
                        "REQUIRED-FLOOR REFUSAL cause=EnrolmentTypedCostDebtDuplicate identity={s}"
                    ));
                }
            }
            other => {
                return Err(format!(
                    "{QUALIFIED}: expected a qualified name, got {}",
                    floor_value_shape(Some(other))
                ));
            }
        }
    }
    Ok(out)
}

/// ONE FULL PARSE PER (INDEX, FILE) and ONE INDEX PER MODULE-NAME SET, asserted at `at`. Called
/// before the subject's strict compile, so a subject that refuses there has still been checked,
/// and again after the whole-pool warms, which read every pool file. Each named root set's
/// shared index must already exist: a control does not build what it inspects.
pub(crate) fn floor_index_controls(
    at: &str,
    root_sets: &[(&str, Vec<String>)],
) -> Result<(), String> {
    for (which, roots) in root_sets {
        // Looked up, never built: an absent index is a control with nothing to inspect, and
        // building one here would report on a population the control made itself.
        let index =
            super::entry_resolve::memoized_process_shared_index(roots).ok_or_else(|| {
                format!(
                "REQUIRED-FLOOR REFUSAL at={at} roots={which} cause=IndexControlsWithoutIndex -- \
                 no shared index was built over these roots before the control ran"
            )
            })?;
        let (files, parses) = super::reference_reading_parse_control(&index)
            .map_err(|e| format!("REQUIRED-FLOOR REFUSAL at={at} roots={which} {e}"))?;
        eprintln!(
            "[floor-phase] phase=reference-reading-parses at={at} roots={which} \
             index_generation={} files={files} full_parses={parses}",
            index.generation
        );
    }
    // The floor legitimately demands more than one name set (its source roots, the dag-only
    // environment closure, the v1 attribution roots when src/v1 is touched), so the invariant is
    // not a count: it is that no set is indexed twice.
    let builds = super::multi_entry_index_builds();
    let name_sets = super::multi_entry_index_sharing_control(&builds)
        .map_err(|e| format!("REQUIRED-FLOOR REFUSAL at={at} cause={e}"))?;
    let resident = super::entry_resolve::shared_index_residency_control()
        .map_err(|e| format!("REQUIRED-FLOOR REFUSAL at={at} cause={e}"))?;
    eprintln!("[floor-phase] phase=shared-index-residency at={at} resident_pools={resident}");
    eprintln!(
        "[floor-phase] phase=multi-entry-index-builds at={at} builds={} name_sets={name_sets} \
         sites={:?}",
        builds.len(),
        builds
            .iter()
            .map(|b| format!("{}@{}", b.modules, b.site))
            .collect::<Vec<_>>(),
    );
    Ok(())
}

/// Authored module names of `.dag` files whose non-data, non-test-fn declaration the diff
/// touched. The spelling is the `module` header, the same key `assemble_prepared_subject_closure`
/// matches with `starts_with`. A file with no module header refuses: dropping it would silently
/// exempt the malformed case from the compile seed, which is the class this seed exists to close.
///
/// A touched file OUTSIDE THE FLOOR'S SOURCE ROOTS (a `src/v1` `.dag` mirror under the parse
/// sweep but not under `--source-root dag --source-root src/v2`) names a module the prepared
/// subject cannot contain, so it is not a seed. It is returned as its own typed row rather
/// than dropped or counted: the reader sees which file seeded nothing and why.
pub(crate) fn module_seeds_from_touched_entry_files(
    base: &Path,
    touched_entry_files: &std::collections::HashSet<String>,
    source_roots: &[String],
) -> Result<(Vec<String>, Vec<(String, String)>, Vec<(String, String)>), String> {
    let roots = floor_source_roots_workspace_relative(source_roots);
    let mut modules: Vec<String> = Vec::new();
    let mut outside: Vec<(String, String)> = Vec::new();
    let mut seeded: Vec<(String, String)> = Vec::new();
    for file in touched_entry_files {
        let content = std::fs::read_to_string(base.join(file))
            .map_err(|e| format!("touched-entry compile-subject seed: read {file}: {e}"))?;
        let module = extract_module_path(&content).ok_or_else(|| {
            format!(
                "touched-entry compile-subject seed: {file} has a non-data, non-test-fn \
                 declaration edit but no module header, so it cannot be seeded into Strict \
                 preparation"
            )
        })?;
        let file_norm = normalize_repo_path(file);
        if path_under_floor_roots(&file_norm, &roots) {
            seeded.push((file_norm, module.clone()));
            modules.push(module);
        } else {
            outside.push((file_norm, module));
        }
    }
    modules.sort();
    modules.dedup();
    outside.sort();
    seeded.sort();
    Ok((modules, outside, seeded))
}

/// The floor's source roots as workspace-relative directory prefixes, the spelling a
/// `DeclarationIndex` record's `rel_path` and a diff path share.
fn floor_source_roots_workspace_relative(source_roots: &[String]) -> Vec<String> {
    source_roots
        .iter()
        .map(|root| {
            workspace_relative_repo_path(root)
                .trim_end_matches('/')
                .to_string()
        })
        .collect()
}

/// Whether a workspace-relative `.dag` path is indexed by the floor's prepared subject at all.
/// A path outside every root names a module preparation cannot contain, whatever seeds it.
fn path_under_floor_roots(rel_path: &str, roots: &[String]) -> bool {
    let rel = normalize_repo_path(rel_path);
    roots
        .iter()
        .any(|root| root == "." || root.is_empty() || rel.starts_with(&format!("{root}/")))
}

/// Is this identity's declared home under a root the tree declares NON-EXECUTING?
///
/// The authority is `gunbc.ci.ci_layer_roots` `non_executing_witness_module_prefixes`, read through the same
/// front-end projection as `witness_layer_roots`. Membership is the grant; everything else blocks.
/// The roster is grained in the MODULE NAMESPACE because this population is undiscovered by
/// definition -- no file was enumerated, so no path exists here to compare against a directory.
fn identity_home_is_declared_non_executing(module_path: &str) -> bool {
    non_executing_witness_module_prefixes()
        .iter()
        .any(|prefix| {
            !prefix.is_empty()
                && (module_path == prefix.as_str()
                    || module_path.starts_with(&format!("{prefix}.")))
        })
}

/// The host realization of `v2.workflow.floor_changed_witness`
/// `changed_witness_execution_standing`, one row per changed identity, joined against the two
/// receipt populations the run already holds: the disposition rows (the admission authority)
/// and the terminal rows read through `claim_disposition` — the SAME projection the disposition
/// TSV's outcome column uses, so this projection cannot disagree with the receipt about what a
/// row's outcome was. It realizes the arms the .dag fold names; it does not invent a fifth.
pub(crate) fn changed_witness_projection_rows(
    changed: &[String],
    disposition_rows: &[RequiredFloorDispositionRow],
    terminal: &[ClaimTerminalRow],
    verdict_only: &HashSet<String>,
    observations: &HashMap<String, ChangedWitnessCostObservation>,
    wet_lane: &LocalRepoWetLaneOutcome,
    candidate: &str,
) -> Vec<ChangedWitnessProjectionRow> {
    let dispositions: std::collections::HashMap<&str, &RequiredFloorDisposition> = disposition_rows
        .iter()
        .map(|r| (r.identity.as_str(), &r.disposition))
        .collect();
    let outcomes: std::collections::HashMap<&str, ClaimDisposition> = terminal
        .iter()
        .map(|row| (row.qualified.as_str(), claim_disposition(row)))
        .collect();
    changed
        .iter()
        .map(|identity| match dispositions.get(identity.as_str()) {
            None => ChangedWitnessProjectionRow {
                identity: identity.clone(),
                cost: None,
                standing: "missing-disposition",
                disposition: "absent".to_string(),
                outcome: "absent".to_string(),
                blocks: true,
                cause: "changed_witness_missing_disposition".to_string(),
            },
            Some(
                RequiredFloorDisposition::Planned
                | RequiredFloorDisposition::PlannedAsChangedWitness
                | RequiredFloorDisposition::PlannedAsReachConsumer,
            ) => {
                let outcome = outcomes.get(identity.as_str()).copied();
                // THE COST POLICY THIS IDENTITY EXECUTED UNDER, and the measurement published
                // for it. Under `ChangedCostDebtVerdictOnly` the CPU line was observed and not
                // gating, so a verdict that was reached is green and CARRIES its cost; an
                // override with no published measurement is the one new red
                // (`v2.workflow.floor_changed_witness` `changed_witness_planned_standing`).
                let policy = if verdict_only.contains(identity.as_str()) {
                    ChangedWitnessCostPolicy::ChangedCostDebtVerdictOnly
                } else {
                    ChangedWitnessCostPolicy::Ordinary
                };
                let observation = observations.get(identity.as_str()).copied();
                // The green set mirrors the .dag fold exactly: an ordinary Pass, an enrolled
                // expected-red failing as enrolled (§4b keeps discriminating REDs enrolled, so
                // touching one is sanctioned), and an enrolled row that passed (the stale
                // roster reds the floor at its own grain with its own remedy).
                let verdict_only_policy =
                    policy == ChangedWitnessCostPolicy::ChangedCostDebtVerdictOnly;
                // The verdict arms, before the cost policy is applied: an ordinary Pass, an
                // enrolled row that passed, and — ONLY under the override — a pass that ran past
                // the CPU line.
                let reached_pass = matches!(
                    outcome,
                    Some(ClaimDisposition::Passed | ClaimDisposition::KnownRedNowPassing)
                ) || (verdict_only_policy
                    && matches!(outcome, Some(ClaimDisposition::PassedOverBudget)));
                let cost_missing = verdict_only_policy && reached_pass && observation.is_none();
                // THE JOINED WET STANDING (`v2.workflow.floor_changed_witness`
                // `HermeticRouteGapHeldAndWetPassed`). A hermetic route gap stays nonterminal on
                // its own; it becomes admissible ONLY when the SAME identity has a wet terminal
                // that reached the expected verdict against THIS candidate, and only while the
                // lane's own schedule-to-terminal join holds.
                //
                // THE CANDIDATE IS COMPARED, NOT ASSUMED. Membership in a set of identities says
                // "it passed somewhere"; the pair the `.dag` evidence carries says where. The
                // lane records the subject it ran against, and an admission from any other
                // subject is refused here rather than silently accepted as this run's.
                let wet_joined = matches!(outcome, Some(ClaimDisposition::RouteGapBeforeVerdict))
                    && wet_lane.refusals.is_empty()
                    && wet_lane.candidate == candidate
                    && wet_lane.admitted.contains(identity.as_str());
                let green = (reached_pass && !cost_missing)
                    || matches!(outcome, Some(ClaimDisposition::KnownRedHeld))
                    || wet_joined;
                // THE STANDING AND ITS CAUSE ARE ONE CHOICE. Deriving the cause from the
                // standing string afterwards would be a second classification of the same fact
                // that a typo could silently desynchronize; the green arms carry no cause at all
                // rather than a plausible one nobody prints.
                let (standing, cause): (&'static str, &'static str) = match outcome {
                    Some(ClaimDisposition::KnownRedHeld) => ("planned-and-known-red-held", ""),
                    _ if cost_missing => (
                        "cost-observation-missing-under-verdict-only",
                        "changed_witness_cost_observation_missing_under_verdict_only",
                    ),
                    _ if reached_pass && verdict_only_policy => {
                        ("planned-and-passed-with-cost-debt-observed", "")
                    }
                    _ if reached_pass => ("planned-and-passed", ""),
                    _ if wet_joined => ("hermetic-route-gap-held-and-wet-passed", ""),
                    _ => {
                        // "No terminal Passed verdict stands", deliberately covering a failed
                        // or refused verdict too — each of those already reds the floor by its
                        // own mechanism, and this projection reds it again at the changed-set
                        // grain, naming the identity the change touched.
                        (
                            "planned-without-terminal-verdict",
                            "changed_witness_planned_without_terminal_verdict",
                        )
                    }
                };
                ChangedWitnessProjectionRow {
                    identity: identity.clone(),
                    cost: observation,
                    standing,
                    disposition: required_floor_disposition_label(dispositions[identity.as_str()])
                        .to_string(),
                    outcome: outcome
                        .map(claim_disposition_wire)
                        .unwrap_or("not_executed")
                        .to_string(),
                    blocks: !green,
                    cause: cause.to_string(),
                }
            }
            // A DECLINE BLOCKS UNLESS THE TREE GRANTS AN EXEMPTION, and the grant is membership in
            // a roster, never the absence of a match anywhere else.
            //
            // Blocking every decline is right for the silence the 2026-08-30 ruling closes: a PR
            // adds witnesses, they never run, the floor greens. `DeclinedOutsideGateClosure` is
            // that case. A root the fold structurally CANNOT reach is a different subject --
            // `gunbc.ci.ci_layer_roots` `non_executing_witness_module_prefixes` declares it, and
            // `non_executing_witness_module_prefixes_restoration` carries its 4b(2) trigger (bare references
            // binding by containment; vehicle, the namespace cut). Blocking there surfaces no new
            // silence: it forbids TOUCHING debt the tree has already declared, including by the
            // program whose trigger retires it. A rule that forbids the work its own trigger
            // requires is not enforcement.
            //
            // FAIL-CLOSED: the exemption requires the identity's declared home to BE in that
            // roster. An unreadable roster, an empty one, or a home nobody rostered all block. This
            // is deliberately not `!witness_layer_roots().contains(home)` -- that grants the
            // exemption by absence, so an unrostered tree or a typo'd path would go silently
            // non-blocking, and it substitutes "not declared executing" for "declared
            // non-executing", which are different claims.
            //
            // The exemption evaporates with the row: when `v1` leaves the roster, this arm stops
            // matching and needs no deletion.
            Some(RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery {
                module_path,
            }) if identity_home_is_declared_non_executing(module_path) => {
                ChangedWitnessProjectionRow {
                    identity: identity.clone(),
                    cost: None,
                    standing: "declined-in-declared-non-executing-root",
                    disposition: required_floor_disposition_label(
                        &RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery {
                            module_path: module_path.clone(),
                        },
                    )
                    .to_string(),
                    outcome: "not_executed".to_string(),
                    blocks: false,
                    cause: String::new(),
                }
            }
            // A CHANGED BinWitnessWet ROW: declined because no CI lane executes the class, which is
            // the declared loss `gunbc.rung_drop.edited_bin_witness_wet_rows_not_executed_by_ci`.
            // Non-blocking on that declaration, and never silent: the row prints with its pattern
            // and the summary line counts it beside the blocking and the declared-root declines.
            Some(declined @ RequiredFloorDisposition::DeclinedNoCiWetLane { .. }) => {
                ChangedWitnessProjectionRow {
                    identity: identity.clone(),
                    cost: None,
                    standing: "declined-no-ci-wet-lane",
                    disposition: format!(
                        "{} pattern={}",
                        required_floor_disposition_label(declined),
                        required_floor_disposition_matched_prefix(declined)
                    ),
                    outcome: "not_executed".to_string(),
                    blocks: false,
                    cause: String::new(),
                }
            }
            // THE DECLINE'S CAUSE IS ITS DISPOSITION, VERBATIM. This arm is the one the
            // `non_verdict_disposition_surfaces_as_refusal` receipt was measured on: a witness
            // enrolled because the DIFF touched it and declined because DISCOVERY reaches no
            // module still blocks, and must not reach its consumer wearing the flat name of the
            // population it belongs to. It is discharged by making the identity reachable or by
            // declaring it unreachable — never by a rerun, which is the only affordance one
            // undifferentiated cause can offer.
            Some(
                declined @ (RequiredFloorDisposition::DeclinedLongModule { .. }
                | RequiredFloorDisposition::DeclinedFixtureMember { .. }
                | RequiredFloorDisposition::DeclinedOutsideRequiredGate
                | RequiredFloorDisposition::DeclinedCostDebt
                | RequiredFloorDisposition::DeclinedOutsideGateClosure
                | RequiredFloorDisposition::DeclinedDiscoveryExcluded { .. }
                | RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery {
                    ..
                }),
            ) => ChangedWitnessProjectionRow {
                identity: identity.clone(),
                cost: None,
                standing: "declined",
                disposition: required_floor_disposition_label(declined).to_string(),
                outcome: "not_executed".to_string(),
                blocks: true,
                cause: required_floor_disposition_label(declined).to_string(),
            },
        })
        .collect()
}

/// ONE SCHEDULED ROW of the local-repo wet lane, decoded from
/// `v2.workflow.local_repo_wet_terminal.local_repo_wet_schedule`. The `.dag` declaration is the
/// authority for membership and for the verdict each member is expected to reach; this struct is
/// its host realization, never its origin.
#[derive(Debug, Clone)]
pub(crate) struct LocalRepoWetScheduledRow {
    pub identity: String,
    /// The AUTHORED source path. Kept and JOINED, not decoration: the lane refuses if the prepared
    /// subject resolved the module from a different file, so a stale entry cannot sit unread beside
    /// a module the executor reached some other way.
    pub entry: String,
    pub entry_module: String,
    pub function: String,
    /// THE DECLARED HOST PREMISE, if any, by its `v2.workflow.wet_evidence` `WetHostPremise` arm
    /// name, joined from `local_repo_wet_premise_roster`. The executor reads it BEFORE invoking
    /// the claim; the readback and its verdict are `.dag` (`gunbc.runner.wet_host_premise_readback`).
    pub premise: Option<String>,
}

/// The module whose `local_repo_wet_premise_readings_wet` answers every declared premise once.
pub(crate) const LOCAL_REPO_WET_PREMISE_READBACK_MODULE: &str =
    "gunbc.runner.wet_host_premise_readback";

/// WHAT THE LANE OBSERVED, kept as the two facts the changed-witness join needs and the refusals
/// that make the lane's own honesty checkable.
///
/// `admitted` is the conjunction the gate consults: a terminal for THIS candidate, for this exact
/// identity, whose observed verdict is the one the roster expected. `refusals` carries every way
/// the schedule and the receipts failed to agree — a scheduled member with no receipt, a receipt
/// for a member nobody scheduled, a verdict that was not the expected one. The lane reds the floor
/// on any of them, because a lane whose roster and receipts disagree cannot support the route claim
/// `std.witness_admission` makes for its cadence.
#[derive(Debug, Default)]
pub(crate) struct LocalRepoWetLaneOutcome {
    pub scheduled: usize,
    /// THE CANDIDATE EVERY ADMISSION IN THIS OUTCOME IS BOUND TO -- the prepared subject digest
    /// the lane actually ran against. Carried rather than assumed: the `.dag` evidence
    /// (`WetTerminalAdmitted`) is a pair, and a set of identities with no candidate beside it is
    /// the "it passed SOMEWHERE" standing this whole lane exists to refuse.
    pub candidate: String,
    pub admitted: HashSet<String>,
    pub refusals: Vec<String>,
}

/// Decode the lane's schedule from its `.dag` authority.
fn local_repo_wet_schedule(
    hermetic: &v1_interpreter::InterpContext,
) -> Result<Vec<LocalRepoWetScheduledRow>, String> {
    let value = v1_interpreter::run_in_context(
        hermetic,
        "v2.workflow.local_repo_wet_terminal.local_repo_wet_schedule",
        false,
    )
    .map_err(|e| format!("local_repo_wet_schedule: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|e| format!("local_repo_wet_schedule: {e}"))?;
    let mut out: Vec<LocalRepoWetScheduledRow> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for item in items {
        let v1_interpreter::Value::Record { type_name, fields } = item else {
            return Err(format!(
                "local_repo_wet_schedule: expected WetScheduledClaim, got {}",
                floor_value_shape(Some(&item))
            ));
        };
        if !hermetic.sym_eq(*type_name, "WetScheduledClaim") {
            return Err(format!(
                "local_repo_wet_schedule: expected WetScheduledClaim, got record {}",
                hermetic.resolve(*type_name)
            ));
        }
        let str_field = |name: &str| -> Result<String, String> {
            match hermetic.field(&fields, name) {
                Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                other => Err(format!(
                    "local_repo_wet_schedule: {name} must be String, got {}",
                    floor_value_shape(other)
                )),
            }
        };
        let function = str_field("function")?;
        let entry = str_field("entry")?;
        // THE IDENTITY IS A STRUCTURED `WitnessIdentity`, NOT A SPELLED NAME. It carries
        // `module_path` and `function` as separate fields, so the qualified name is DERIVED here
        // rather than parsed back out of a string — which is what the authority does in
        // `witness_identity_qualified_name`, and deriving it twice the same way is the point.
        let (module_path, identity_function) = match hermetic.field(&fields, "identity") {
            Some(v1_interpreter::Value::Record {
                type_name: id_type,
                fields: id_fields,
            }) => {
                if !hermetic.sym_eq(*id_type, "WitnessIdentity") {
                    return Err(format!(
                        "local_repo_wet_schedule: identity must be a WitnessIdentity, got record {}",
                        hermetic.resolve(*id_type)
                    ));
                }
                let id_str = |name: &str| -> Result<String, String> {
                    match hermetic.field(id_fields, name) {
                        Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                        other => Err(format!(
                            "local_repo_wet_schedule: identity.{name} must be String, got {}",
                            floor_value_shape(other)
                        )),
                    }
                };
                (id_str("module_path")?, id_str("function")?)
            }
            other => {
                return Err(format!(
                    "local_repo_wet_schedule: identity must be a WitnessIdentity record, got {}",
                    floor_value_shape(other)
                ));
            }
        };
        let identity = format!("{module_path}.{identity_function}");
        // THE ENTRY MODULE IS READ FROM THE IDENTITY, NOT PARSED OUT OF IT. `WitnessIdentity`
        // carries `module_path` as its own field, so what used to be a suffix-strip is now a
        // projection (DESIGN §3 — a path is a discriminator, the authored module name is the fact).
        //
        // THE AGREEMENT WALL SURVIVES AND GOT STRONGER RATHER THAN WEAKER. The old check inferred
        // disagreement from a failed suffix-strip, which could only notice it when the spelled
        // identity did not END in the function. The row carries the function name TWICE — once in
        // `identity.function`, once as its own `function` field — so the two are compared directly
        // and any disagreement refuses, not only the ones a suffix happens to expose. An empty
        // module_path is refused for the same reason the strip refused an empty prefix: a member
        // whose module is unnamed cannot key a prepared subject.
        if identity_function != function {
            return Err(format!(
                "local_repo_wet_schedule: identity.function {identity_function:?} and the row's \
                 function {function:?} disagree — one member, two names"
            ));
        }
        if module_path.is_empty() {
            return Err(format!(
                "local_repo_wet_schedule: identity {identity:?} carries an empty module_path"
            ));
        }
        let entry_module = module_path;
        // `ClaimExpectation` HAS TWO ARMS WHERE ITS PREDECESSOR HAD ONE, AND THIS LANE STILL
        // REALIZES EXACTLY ONE. That is the whole reason this refusal is kept rather than widened
        // with the type: the shared expectation vocabulary can now SAY `ExpectedRed`, and nothing
        // in this executor RUNS an expected-red member. A wider type is not a wider capability, and
        // defaulting here would let the authority claim a behaviour production cannot perform.
        // The widening that admits an expected-red member must arrive with the executor arm that
        // realizes it, and this line is what forces those two to land together.
        match hermetic.field(&fields, "expectation") {
            Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
                match hermetic.resolve(*variant_name).as_str() {
                    "ExpectedToHold" => {}
                    other => {
                        return Err(format!(
                            "local_repo_wet_schedule: expectation {other} has no executor arm in this \
                             lane; only ExpectedToHold is realizable today"
                        ));
                    }
                }
            }
            other => {
                return Err(format!(
                    "local_repo_wet_schedule: expectation must be a typed variant, got {}",
                    floor_value_shape(other)
                ));
            }
        }
        // A DUPLICATE REFUSES. The roster is joined to the receipts at identity grain, and a
        // repeated member would make one receipt answer for two rows.
        if !seen.insert(identity.clone()) {
            return Err(format!(
                "local_repo_wet_schedule: duplicate scheduled identity: {identity}"
            ));
        }
        out.push(LocalRepoWetScheduledRow {
            identity,
            entry,
            entry_module,
            function,
            premise: None,
        });
    }
    local_repo_wet_join_premises(hermetic, &mut out)?;
    Ok(out)
}

/// JOIN `v2.workflow.local_repo_wet_terminal.local_repo_wet_premise_roster` onto the schedule at
/// identity grain. A premise row naming an identity nobody scheduled, or naming one twice, REFUSES:
/// a premise that gates nothing is a roster defect, and two premises for one claim have no order.
fn local_repo_wet_join_premises(
    hermetic: &v1_interpreter::InterpContext,
    schedule: &mut [LocalRepoWetScheduledRow],
) -> Result<(), String> {
    let value = v1_interpreter::run_in_context(
        hermetic,
        "v2.workflow.local_repo_wet_terminal.local_repo_wet_premise_roster",
        false,
    )
    .map_err(|e| format!("local_repo_wet_premise_roster: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|e| format!("local_repo_wet_premise_roster: {e}"))?;
    for item in items {
        let v1_interpreter::Value::Record { fields, .. } = item else {
            return Err(format!(
                "local_repo_wet_premise_roster: expected WetPremisedClaim, got {}",
                floor_value_shape(Some(item))
            ));
        };
        let identity = match hermetic.field(fields, "identity") {
            Some(v1_interpreter::Value::Record {
                fields: id_fields, ..
            }) => {
                let id_str = |name: &str| -> Result<String, String> {
                    match hermetic.field(id_fields, name) {
                        Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                        other => Err(format!(
                            "local_repo_wet_premise_roster: identity.{name} must be String, got {}",
                            floor_value_shape(other)
                        )),
                    }
                };
                format!("{}.{}", id_str("module_path")?, id_str("function")?)
            }
            other => {
                return Err(format!(
                    "local_repo_wet_premise_roster: identity must be a WitnessIdentity record, \
                     got {}",
                    floor_value_shape(other)
                ));
            }
        };
        let premise = match hermetic.field(fields, "premise") {
            Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
                hermetic.resolve(*variant_name).to_string()
            }
            other => {
                return Err(format!(
                    "local_repo_wet_premise_roster: premise must be a WetHostPremise variant, \
                     got {}",
                    floor_value_shape(other)
                ));
            }
        };
        let Some(row) = schedule.iter_mut().find(|r| r.identity == identity) else {
            return Err(format!(
                "local_repo_wet_premise_roster: {identity} declares premise {premise} but is not \
                 scheduled"
            ));
        };
        if let Some(prior) = &row.premise {
            return Err(format!(
                "local_repo_wet_premise_roster: {identity} declares two premises ({prior}, \
                 {premise})"
            ));
        }
        row.premise = Some(premise);
    }
    Ok(())
}

/// Per-witness selection detail (the `SKIP`/`SKIP-RESOLVE`/`PREDICT` lines and the
/// per-resolve `[binding-fork-ledger]` census) is opt-in. The default floor output is the
/// upfront `[affected-set]` categorization plus the final `[measurement]` tally — a wide
/// corpus otherwise streams one skip line per unaffected witness (~1.7k lines), drowning the
/// signal. The counts survive on `DiscoverySummary`; only the per-row narration is gated.
pub(crate) fn floor_verbose() -> bool {
    std::env::var("GUNBC_FLOOR_VERBOSE")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

/// Wall-clock stamp `HH:MM:SS.mmm` (UTC) prefixed on the live floor lines so the stream reads as
/// a timeline and correlates with CI's wall-clock log — dependency-free (no chrono): seconds
/// since the epoch reduced to a 24h clock, plus millis.
pub(crate) fn floor_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

pub(crate) fn floor_color_enabled() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    if std::env::var("GUNBC_FLOOR_COLOR")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
    {
        return true;
    }
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
        || std::env::var("GITHUB_ACTIONS")
            .map(|v| v == "true")
            .unwrap_or(false)
}

/// Live realization view: stream affected witnesses to stderr as they finish, one colored
/// line per shard, so a run reads as "the affected set unrolling in real time" rather than a
/// silent wait then a summary. On by default (opt out with `GUNBC_FLOOR_QUIET=1`); color
/// auto-detected (a terminal or GitHub Actions), `NO_COLOR` honored, `GUNBC_FLOOR_COLOR=1`
/// forces it on. Only RUN witnesses reach the stream — skips are counted, not narrated.
pub(crate) fn floor_stream_enabled() -> bool {
    !std::env::var("GUNBC_FLOOR_QUIET")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

/// THE SEEDS OF THE REQUIRED FLOOR'S NOMINAL PREPARED SUBJECT -- what the floor prepares when a
/// run touches nothing: the gate's prefix roster and authored-module roster, both decoded from
/// `v2.workflow.required_floor` in a frame over that module's own closure, and the local-repo wet
/// schedule rows whose entry modules join the module seeds. The diff-derived seeds (changed
/// witnesses, touched entries, changed-interface consumers) are NOT here: they are a fact about one run's
/// diff, and this is the part of the subject that holds for every run.
///
/// ONE PRODUCER, TWO CONSUMERS, and that is the reason it is a function rather than a block in
/// `run_required_floor`. The floor consumes it to seed preparation; the required-lane resolution
/// census (`gunbc.required_lane_resolution_census`, `gunbc test
/// //gunbc/instruments:required-lane-resolution-census`) consumes it to answer, for every module
/// under the source roots, whether the floor's nominal subject resolves it. A census that
/// re-derived the seed list from the roster would be a second authority for what the floor
/// prepares, and it would drift the first time a seed kind was added here and not there.
pub struct RequiredFloorNominalSubjectSeeds {
    pub required_gate_prefixes: Vec<String>,
    pub required_gate_authored_modules: Vec<String>,
    pub(crate) local_repo_wet_schedule_rows: Vec<LocalRepoWetScheduledRow>,
}

/// THE LANE'S SCHEDULE IS DECODED HERE, IN THE POLICY CLOSURE, AND NOT LATER -- because its
/// modules must become SEEDS of the prepared subject. The first CI run of that lane refused
/// seven times with `EntryModuleOutsidePreparedSubject`: the prepared graph is the required-gate
/// closure plus the changed set, and the lane's members are on the discovery exclusion
/// frontier, so nothing pulled them in. A lane that cannot reach its own members cannot support
/// the route claim `std.witness_admission` makes for its cadence, so the schedule joins the seed
/// list rather than the executor learning to run outside the subject.
pub fn required_floor_nominal_subject_seeds(
    source_roots: &[String],
    gate_entry_index: &MultiEntryIndex,
) -> Result<RequiredFloorNominalSubjectSeeds, String> {
    let corpus = crate::cli_run::read_source_corpus_once(source_roots);
    required_floor_nominal_subject_seeds_from_corpus(&corpus, gate_entry_index)
}

/// The seed fold over a corpus the CALLER read. `run_required_floor` is the least common ancestor
/// of this prepare and the gate prepare below it, so it reads once and lends to both; the wrapper
/// above stays for callers with only one demand (the lane resolution census).
pub fn required_floor_nominal_subject_seeds_from_corpus(
    corpus: &crate::cli_run::SourceCorpusRead,
    gate_entry_index: &MultiEntryIndex,
) -> Result<RequiredFloorNominalSubjectSeeds, String> {
    let policy_seed = [REQUIRED_FLOOR_POLICY_MODULE.to_string()];
    let (policy_prepared, _) = crate::cli_run::prepare_repository_from_corpus(
        corpus,
        &floor_prepared_subject_exclusions(),
        Some((gate_entry_index, &[], &policy_seed)),
    )?;
    let policy_scope = claim_scope_for(&policy_prepared, REQUIRED_FLOOR_POLICY_MODULE)?;
    let policy_frame = evaluation_frame(
        &policy_scope,
        v1_interpreter::ExecutionMode::Hermetic,
        None,
        None,
    );
    let required_gate_prefixes = floor_decode_module_prefix_roster(
        &policy_frame,
        "v2.workflow.required_floor.required_gate_prefixes",
    )?;
    // THE GATE'S SECOND SELECTOR KIND (`v2.workflow.required_floor`
    // `required_gate_authored_modules`, `RequiredGateSelector`): authored module NAMES,
    // matched at segment boundaries -- the module-seed rule, not the prefix rule. They join
    // the module seeds, so preparation and site disposition admit them under one rule by
    // construction.
    let required_gate_authored_modules = floor_decode_module_prefix_roster(
        &policy_frame,
        "v2.workflow.required_floor.required_gate_authored_modules",
    )?;
    let local_repo_wet_schedule_rows = local_repo_wet_schedule(&policy_frame)?;
    Ok(RequiredFloorNominalSubjectSeeds {
        required_gate_prefixes,
        required_gate_authored_modules,
        local_repo_wet_schedule_rows,
    })
}

/// The nominal MODULE seeds (`v2.workflow.floor_subject_seed` `PreparedSubjectSeedGround`,
/// matched at segment boundaries): the floor's own runtime authorities, the gate's authored
/// modules, and the wet schedule's entry modules. The prefix seeds travel separately because
/// they are matched by a different rule.
pub(crate) fn required_floor_nominal_closure_module_seeds(
    required_gate_authored_modules: &[String],
    local_repo_wet_schedule_rows: &[LocalRepoWetScheduledRow],
) -> Vec<String> {
    REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES
        .iter()
        .map(|m| m.to_string())
        .chain(required_gate_authored_modules.iter().cloned())
        .chain(
            local_repo_wet_schedule_rows
                .iter()
                .map(|row| row.entry_module.clone()),
        )
        .chain(
            local_repo_wet_schedule_rows
                .iter()
                .any(|row| row.premise.is_some())
                .then(|| LOCAL_REPO_WET_PREMISE_READBACK_MODULE.to_string()),
        )
        .collect()
}

pub fn floor_prepared_subject_exclusions() -> Vec<String> {
    vec![
        // MUST-NOT-RESOLVE PROBES. Every module under dag/test/probe/ declares in its header that
        // it must not resolve: each is a designed refusal, consumed BY CLASS AND SUBJECT through
        // `gunbc.compile_diagnostic_census` `census_of(source: probe_source(name))` -- the source
        // text handed to the compiler by a fixture (DESIGN 4b), never a module in a closure.
        // They stayed out of the prepared subject before gunbc#11256 only by accident: no
        // importer, no gate prefix. The touched-entry seed (`module_seeds_from_touched_entry_files`)
        // seeds the authored module of every edited non-data, non-test-fn declaration, so a PR
        // that edits a probe (gunbc#11343 at 76923b733, run 34874099243:
        // `[floor-phase] phase=touched-entry-compile-subject seeds=6 modules=[...,
        // "test.probe.bare_string_from_boundary_probe"]` while
        // `phase=interface-changed-consumers ... consumers_added=0`) pulled its designed refusal
        // into Strict preparation as a blocker. This row is applied AFTER the seed walk, so the
        // seed still prints (as `TouchedEntryExcludedFromPreparation`, a typed row rather than a
        // vanished seed), and `ExclusionOrphansImporter` refuses the day anything imports one.
        "test/probe/".to_string(),
        "test/fixture/meta_exec_confinement_scan/".to_string(),
        // THE FLEET HEALTH FORBIDDEN PROGRAM. test.fixture.health_read_only.mutation_probe is the
        // program the no-mutation witness derives its findings from, consumed PARSE-ONLY: its
        // callees are undeclared spellings by design, so it must never resolve. Nothing imports it,
        // so it stayed out of preparation by that accident alone until gunbc#12761's widened
        // subject prepared it and it refused. Its designed refusal is asserted by class and subject
        // in test.claim.fleet.health_read_only_fixture_refusal_witness.
        "test/fixture/health_read_only/".to_string(),
        "test/manual/ownership_movable.dag".to_string(),
        // WET RECEIPT, AND IT HAS NO CI CONSUMER TODAY — stated plainly rather than dressed up
        // as an enrollment. case4_expansion_carrier_splices dispatches a real jq through
        // jq.Process.RunWithStdin, which carries no mock_response, so the hermetic floor refuses
        // it and one refusing member fails the run. Mocking is not the repair: the witness exists
        // to prove that a REAL jq exits 0 only on two argv words rather than one concatenated
        // one, and a mocked dispatch passes it without any process running.
        //
        // The first attempt at this fix added rows to gunbc.ci.ci_layer_roots
        // (witness_exclusion_frontier + bin_witness_wet_entries) and asserted they would take
        // effect. They did not: run_required_floor consults THIS list and nothing else, the CI
        // receipt was an unchanged modules_excluded=2, and the wet batches those rosters feed
        // were deleted with the old floor. Adding a row to a roster with no live consumer is
        // specification-without-execution, so both rows were reverted rather than left standing.
        //
        // What this exclusion buys is a green floor; what it does NOT buy is coverage. The claim
        // sits at UNEXECUTED-IN-CI with a local recipe recorded in the design note (§15), and
        // re-enrolls when a wet lane exists again — which required_floor.dag deliberately defers
        // until it can be "asked against a live consumer".
        "test/manual/process_argv_expansion_receipt.dag".to_string(),
        // Wet receipt for command_runner's transport-agnostic run site (the LocalExec/SshExec
        // match collapsed onto command_over_transport). Excluded for the same reason as the line
        // above and not a new class: hermetic evaluation replays an operation's declared
        // mock_response and never constructs an argv, so "the words reached the process unsplit"
        // is not observable hermetically. Enrolling it would assert the mock rather than the
        // behaviour -- specification without execution.
        "test/manual/command_runner_local_argv_receipt.dag".to_string(),
        // A ROW HERE MAY NOT NAME A MODULE ANYTHING IMPORTS. An exclusion is not "skip this
        // witness": prepare_repository_closure drops the path from the prepared graph, so an
        // importer cannot resolve and the whole subject refuses. That constraint is enforced --
        // `assemble_prepared_subject_closure` refuses with cause=ExclusionOrphansImporter, naming
        // the row beside each importer -- rather than asked for here.
        //
        // A row may also not be the way a CHANGED witness gets out of running. The floor admits a
        // changed witness regardless of any exclusion (cause=ChangedWitnessOutsidePreparedSubject),
        // which is what stops this list being an escape hatch a witness can opt into on the commit
        // that introduces it. A corpus-priced assertion therefore is not authored as a witness at
        // all; v2.lens.complexity_accumulator_copy.corpus_census is the worked example, and it
        // carries its own next-rung trigger.
    ]
}

pub fn floor_prepared_authority_active() -> bool {
    FLOOR_PREPARED_AUTHORITY.with(|cell| cell.borrow().is_some())
}

pub fn floor_prepared_inventory_snapshot() -> Option<Vec<PreparedSourceView>> {
    FLOOR_PREPARED_AUTHORITY.with(|cell| cell.borrow().as_ref().map(|auth| auth.inventory.clone()))
}

pub(crate) fn floor_prepared_inventory_digest() -> Option<String> {
    FLOOR_PREPARED_AUTHORITY.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|auth| auth.inventory_digest.clone())
    })
}

/// Resident kilobytes from `/proc/self/statm`, or `None` when the file will not read or parse.
/// `None` is the ONLY absent answer -- never `Some(0)`, which is a resident set a live process
/// cannot have and which read identically to an unreadable file before this was split out.
pub(crate) fn floor_statm_rss_kb() -> Option<u64> {
    const KB_PER_PAGE: u64 = 4;
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|m| {
            m.split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<u64>().ok())
        })
        .map(|pages| pages * KB_PER_PAGE)
}

/// ONE BEAT'S READINGS, TYPED, EACH `None` MEANING "THIS SOURCE DID NOT READ" and never zero.
/// Authority: `gunbc.observation_ci_render` `HeartbeatSample` -- this is the seed-side twin of
/// that record, at the grain the seed reads its sources: one `/proc/self/stat` read feeds `cpu_ms`
/// and `major_faults` together (a half-read stat cannot be summed, so the pair is all-or-nothing),
/// `/proc/self/statm` feeds `rss_bytes`, `memory.current`, `memory.events` and
/// `memory.events.local` are three files, and `/proc/vmstat` is one. The rendering is
/// `render_heartbeat_line_mirror`; nothing here formats.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FloorResourceSample {
    pub cpu_ms: Option<u64>,
    pub major_faults: Option<u64>,
    pub rss_bytes: Option<u64>,
    pub cgroup_charge_bytes: Option<u64>,
    /// `(high, max)` from `memory.events`, read together because they are one file.
    pub cgroup_events: Option<(u64, u64)>,
    pub cgroup_local_high_events: Option<u64>,
    /// `(pswpin, pgmajfault)` from `/proc/vmstat`, read together because they are one file.
    pub host_vmstat: Option<(u64, u64)>,
}

pub(crate) fn floor_resource_sample(cpu_baseline_ms: u64) -> FloorResourceSample {
    // ONE ABSENT ANSWER FOR ONE STATE. The cgroup and vmstat readers below answer `None` when
    // their file will not read; an earlier shape of this sampler answered a fabricated `0` for
    // three of these, so `rss_kb=0` rendered identically whether the process held no resident
    // pages -- which a live process cannot -- or `/proc/self/statm` was unreadable. Those are
    // different facts with opposite remedies, and this is the instrument that exists to settle a
    // memory contradiction, so a zero it invented is worse here than anywhere else in the line.
    // The cpu pair is all-or-nothing because a half-read stat cannot be summed.
    let stat = std::fs::read_to_string("/proc/self/stat").ok();
    let f: Vec<&str> = stat
        .as_deref()
        .unwrap_or("")
        .rsplit(')')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect();
    // Fields are indexed from the field AFTER comm: utime/stime are 12/13 here, majflt is 10.
    let tick = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok());
    let hz = 100u64;
    let (cpu_ms, major_faults) = match (tick(11), tick(12), tick(9)) {
        (Some(utime), Some(stime), Some(majflt)) => (
            Some(((utime + stime) * 1000 / hz).saturating_sub(cpu_baseline_ms)),
            Some(majflt),
        ),
        _ => (None, None),
    };
    let rss_bytes = floor_statm_rss_kb().map(|kb| kb * 1024);
    // THE CGROUP CHARGE AND THE THROTTLE EVENTS, on every beat, because the runs that most need
    // them are the ones that never reach an exit line. `floor_cgroup_envelope` reports the full
    // picture at entry; these carry the parts that CHANGE, so a killed run still leaves behind
    // what its envelope was doing when it died.
    //
    // rss and the cgroup charge are DIFFERENT COUNTERS and are printed side by side so they are
    // never silently substituted for one another: RSS is this process's resident anonymous +
    // mapped pages, while memory.current is the cgroup's total charge including page cache and
    // every other process in it. Comparing RSS against a cgroup limit is what produced the
    // standing contradiction this instrument exists to settle — a CI run observed at 14.69 GiB
    // RSS, above a declared 14.00 GiB max, that was not killed and ran 151 minutes more.
    //
    // high/max are the reclaim and kill counters for THIS level. Nonzero high is throttling
    // actually happening rather than inferred from a declared row; both zero beside a death
    // means the ceiling that killed it was somewhere else.
    //
    // READ THE LEAF, NOT THE ROOT. These fields first shipped reading `/sys/fs/cgroup/{name}`
    // directly, and on CI they printed unreadable on every single beat for a four-hour run: the
    // runner's leaf is
    // `/sys/fs/cgroup/system.slice/system-actions\x2drunner.slice/actions-runner@srv2-03.service`,
    // the root holds no `memory.current` this process may read, and the fallback fired every
    // time. The entry snapshot walked the path correctly while the sampler that runs
    // continuously did not — two readers at two different levels in one commit, and the wrong
    // one was the only one that would still be emitting when a run was cancelled.
    //
    // It passed locally because this container's `/proc/self/cgroup` is `0::/`, so leaf and root
    // are the same directory and the hardcoded path was accidentally correct. A degenerate
    // topology validated an instrument that had no chance of working anywhere else, which is why
    // the path is now taken from the same place `floor_cgroup_envelope` takes it.
    let cg_file =
        |name: &str| std::fs::read_to_string(format!("{}/{name}", floor_cgroup_dir())).ok();
    let cg_count = |name: &str| cg_file(name).and_then(|s| s.trim().parse::<u64>().ok());
    let cg_event = |body: &str, idx: usize| {
        body.lines()
            .nth(idx)
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u64>().ok())
    };
    let cgroup_charge_bytes = cg_count("memory.current");
    let cgroup_events =
        cg_file("memory.events").and_then(|body| Some((cg_event(&body, 1)?, cg_event(&body, 2)?)));
    // The LOCAL counter beside the hierarchical one, on every beat. `memory.events` at this
    // level already includes everything its descendants generated, so `high` rising says
    // "something in this subtree was throttled" and cannot say it was us. `local high` is
    // the same event restricted to this exact cgroup, and the pair is the only way to separate
    // our own reclaim from a neighbour's. Carried per beat rather than only in the periodic
    // envelope because a killed run keeps only what was already printed, and this is the field
    // the neighbour-pressure hypothesis is decided on.
    let cgroup_local_high_events =
        cg_file("memory.events.local").and_then(|body| cg_event(&body, 1));
    // HOST-WIDE SWAP-IN, because the fault storm has no local cause and this is what decides
    // whether it has ANY cause belonging to this fold.
    //
    // Established: ~7,416 major faults/s sustained for 151 minutes while the leaf cgroup read
    // high,0 max,0 oom_kill,0 — so the process was nowhere near its own limits and its own
    // cgroup did no reclaiming. A major fault is a page read from disk, and there are two ways
    // to take one without your own cgroup throttling you: the HOST reclaimed and swapped your
    // anonymous pages, or you are faulting file-backed pages in and out of a mapping.
    //
    // `pswpin` counts pages swapped IN; `pgmajfault` counts major faults. Differenced across
    // two beats they separate the cases exactly:
    //
    //   pswpin rises with pgmajfault   -> swap. The cause is host memory pressure, not the
    //                                     fold, and no amount of retention work here fixes it.
    //   pgmajfault rises, pswpin flat  -> file-backed mapping churn. That IS local to the fold
    //                                     and is a defect this lane owns.
    //   BOTH FLAT                      -> QUIET. Nothing is happening. This is a THIRD state
    //                                     and it is NOT the churn arm above.
    //
    // The third arm is written down because the two-arm form above was implemented faithfully
    // by two independent readers and both classified zero-and-zero as mapping churn -- 26
    // intervals in one reading, 6 in the other -- which inverts the conclusion, since churn is
    // a defect this lane owns and quiet is the absence of one.
    //
    // The structural reason both landed there: `pgmajfault rises` and `pswpin flat` are two
    // conditions, and only their CONJUNCTION is churn. Zero-and-zero satisfies the second and
    // not the first, so a reader matching on the cheaper condition alone selects churn for it.
    // Stated as an observation about THIS rule and these two readings -- not a claim about what
    // any future reader will do, which two readers cannot establish.
    //
    // Deliberately host-wide rather than cgroup-scoped: /proc/vmstat is not namespaced, and
    // host-level swap is precisely the thing a cgroup-scoped counter cannot see. That is also
    // why the leaf's PSI reading of 0.00% is not evidence of a quiet machine — it is this
    // cgroup's stall time, not the host's.
    let vmstat = std::fs::read_to_string("/proc/vmstat").ok();
    let vm = |key: &str| -> Option<u64> {
        vmstat.as_deref().and_then(|s| {
            s.lines().find_map(|l| {
                l.strip_prefix(key)
                    .and_then(|r| r.strip_prefix(' '))
                    .and_then(|v| v.trim().parse::<u64>().ok())
            })
        })
    };
    let host_vmstat = match (vm("pswpin"), vm("pgmajfault")) {
        (Some(swap_in), Some(major)) => Some((swap_in, major)),
        _ => None,
    };
    FloorResourceSample {
        cpu_ms,
        major_faults,
        rss_bytes,
        cgroup_charge_bytes,
        cgroup_events,
        cgroup_local_high_events,
        host_vmstat,
    }
}

/// THIS PROCESS'S OWN CGROUP DIRECTORY — the single answer both readers use.
///
/// Resolved once from `/proc/self/cgroup` and cached, because the alternative is what shipped
/// first: the entry snapshot walking the real path while the per-beat sampler read the root,
/// disagreeing silently for a whole run. Two readers of one fact is the duplication the repo's
/// own rules forbid, and here it cost every cgroup reading a four-hour CI run would have given.
///
/// Falls back to the cgroup root only when `/proc/self/cgroup` is unreadable — the same place a
/// container whose leaf IS the root legitimately lands.
pub(crate) fn floor_cgroup_dir() -> String {
    static DIR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let rel = std::fs::read_to_string("/proc/self/cgroup")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find_map(|l| l.rsplit("::").next().map(|p| p.to_string()))
            })
            .unwrap_or_default();
        let mut dir = std::path::PathBuf::from("/sys/fs/cgroup");
        for seg in rel.trim_matches('/').split('/').filter(|s| !s.is_empty()) {
            dir.push(seg);
        }
        dir.to_string_lossy().to_string()
    })
    .clone()
}

pub fn floor_seam(name: &str) {
    if let Ok(mut g) = FLOOR_SEAM.lock() {
        g.clear();
        g.push_str(name);
    }
    // EVERY SEAM IS A PHASE BOUNDARY, SO EVERY SEAM CARRIES A BEAT. The sixty-second watchdog
    // names the minute a phase was in, never where it began or ended, so a phase shorter than a
    // beat is invisible and a longer one is bounded only to the minute. A reading taken AT the
    // boundary is what lets gunbc.floor.floor_demand attribute a held-set delta to the phase between two
    // seams rather than to whichever phase the next tick happened to land in.
    floor_cgroup_stat_beat(&format!("seam-{name}"), None);
    floor_heap_beat(name);
    // THREAD CPU AT THE BOUNDARY, so a phase's cost is read as the floor thread's own CPU between
    // two seams rather than wall time, which on a shared host is mostly other jobs' load.
    eprintln!(
        "[floor-seam-cpu] seam={name} thread_cpu_ms={}",
        v1_interpreter::thread_cpu_nanos() / 1_000_000
    );
}

/// THE ALLOCATOR'S OWN SPLIT AT A SEAM: bytes live in allocations, and bytes the allocator holds
/// free. A resident set that stays high across a phase boundary has two causes with opposite
/// remedies -- state still owned by the program (shorten its ownership) or memory freed and kept
/// by glibc (an allocator behaviour, not retention) -- and RSS cannot tell them apart.
/// `malloc_trim` can, but it ACTS: it returns the free half, so the next phase would run on a
/// different heap than the one being measured. `mallinfo2` only reads. Seams only, never the
/// watchdog: it walks every arena, and a boundary is where the attribution is decided.
///
/// `in_use` is `uordblks` (allocated main-arena and thread-arena bytes) plus `hblkhd` (mmapped
/// chunks, which are always live); `free` is `fordblks`. glibc-only, and `na` elsewhere rather
/// than a zero, since zero free is a real reading.
fn floor_heap_beat(seam: &str) {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        // SAFETY: mallinfo2 takes no arguments and returns a struct by value; it reads allocator
        // bookkeeping under the arena locks and changes nothing.
        let mi = unsafe { libc::mallinfo2() };
        eprintln!(
            "[floor-heap] seam={seam} in_use={} free={} mmapped={} arena={} {}",
            mi.uordblks + mi.hblkhd,
            mi.fordblks,
            mi.hblkhd,
            mi.arena,
            crate::cli_run::entry_resolve::process_resolve_census(),
        );
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    eprintln!("[floor-heap] seam={seam} in_use=na free=na mmapped=na arena=na");
}

/// THE SEAM AS IT STANDS RIGHT NOW, so a reading can say WHERE IN THE RUN it was taken.
///
/// `floor_seam` writes this slot; the heartbeat has always read it for its human line. This
/// reader exists because `floor_cgroup_stat_beat` needs the same fact on its OWN line: a beat
/// and the seam it was sampled in were two separate stderr streams, joined only by the order
/// the lines happened to appear in, which is a positional citation of the kind DESIGN 3 forbids
/// — any line emitted between them by any other thread invalidates the join, and the reader
/// transcribing a receipt is the one who silently guesses. With the seam ON the beat, which
/// phase established a peak is a value a fold reads rather than an author's reading of a log.
///
/// THE READ IS THE SHARED FACT; THE RENDERING IS NOT. This returns the slot's own state and
/// renders nothing, because its two consumers owe DIFFERENT spellings of the same state and
/// collapsing them here would be a fork wearing a helper's clothes (review 69697 caught the
/// first version of this function doing exactly that).
///
/// `None` is a poisoned lock — no reading at all. `Some("")` is the genuinely unset slot, before
/// any `floor_seam` call. `Some(name)` is the seam.
///
/// Why the callers differ, and why this must not decide for them: the heartbeat's seam crosses
/// into `gunbc.observation_seed_render` `seed_heartbeat_subject`, which branches on `seam == ""`
/// to decide whether a beat carries a `PhaseSegment` at all — the empty string IS that model's
/// representation of "no phase", pinned by `test.claim.observation_seed_heartbeat_witness`.
/// Handing it the beat line's `none` would mint `PhaseSegment { name: "none" }`, fabricating a
/// phase named after the absence of one. The stat beat, whose line is transcribed into a typed
/// `gunbc.floor.floor_demand` `FloorSeam`, wants a single non-empty token it can map to
/// `SeamNotYetEntered`. Same fact, two boundaries, one read.
pub(crate) fn floor_seam_current() -> Option<String> {
    FLOOR_SEAM.lock().ok().map(|g| g.clone())
}

// THE CONSTRUCTOR A DECODE ACTUALLY OBSERVED, for refusals whose cause is a shape mismatch.
//
// A decode arm that reports only "not the expected shape" identifies its seam and nothing else:
// an absent field, a variant, a record and a scalar all render the same, and they have different
// remedies. This names the constructor and, where the constructor is a container, its arity --
// enough to discriminate "the field is missing" from "the field holds an empty list" from "the
// producer answered a variant" without dumping a value whose size is the reason the run is slow.
//
// It deliberately does NOT recurse or print contents. A shape reporter that renders values
// becomes a second serializer, and on this path the values in question are the ones large enough
// to have made the phase expensive in the first place.
pub(crate) fn floor_value_shape(v: Option<&v1_interpreter::Value>) -> String {
    match v {
        None => "<field absent>".to_string(),
        Some(v1_interpreter::Value::List(xs)) => format!("List(len={})", xs.len()),
        Some(v1_interpreter::Value::Record { fields, .. }) => {
            format!("Record(fields={})", fields.len())
        }
        Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
            format!("Variant({variant_name:?})")
        }
        Some(other) => floor_value_constructor(other).to_string(),
    }
}

// ONE CARRIER, TWO REALIZATIONS -- decoded here rather than assumed to be one of them.
//
// `v2.std.collection` declares `List<T> = std.algebra.FreeMonoid<T>`, an `Empty | Cons` chain.
// The interpreter ALSO represents some sequences natively as `Value::List`. Which one arrives at
// a given seam depends on whether the value was produced by a `.dag` fold or handed over by the
// host, and that is the model/realization fork DESIGN names as systemic and unfinished -- not a
// choice this decode gets to make.
//
// So the decode reads the carrier in either realization and REFUSES anything else, naming what
// it saw. What it must not do is assume one realization: the previous decode matched
// `Value::List` alone and rendered a perfectly well-formed `Cons` chain as "the attempt carries
// no claim list", which reports a representation mismatch as an absent population.
//
// The walk is ITERATIVE. A `Cons` chain of one element per claim is 10,444 deep on the measured
// subject, and the recursive spelling already in this file for dependency edges would recurse
// once per element -- fine for a handful of edges, a stack overflow here. Depth belongs on the
// heap when the depth is the population size.
/// One nullary `.dag` Int authority, decoded. The floor's per-claim thresholds are authored in
/// `v2.workflow.required_floor` and read here rather than re-spelled in Rust: the host owns when
/// a budget is applied, never what it is.
/// READ A `std.measure` MEASURE-TYPED CONSTANT AND RETURN ITS COUNT.
///
/// `Measure<Q, S, M> { count: M }` is a single-field record, so a `Millisecond` or `ByteSize`
/// constant arrives here as `Record { count: Int }` and `floor_required_int` — which demands a bare
/// `Int` — cannot read it. This is the host half of carrying units on the carrier rather than in a
/// field name: the unit lives in the `.dag` type, and the host unwraps exactly one level to get the
/// magnitude it compares.
///
/// SAME STRICTLY-POSITIVE WALL AS `floor_required_int`, and for the same reason: a zero ceiling is
/// not a lenient policy, it refuses every subject before it measures one. The refusal names the
/// shape it actually got, so a constant that stops being a Measure fails loudly here instead of
/// being read as some other number.
pub(crate) fn floor_required_measure_count(
    ctx: &v1_interpreter::InterpContext,
    func: &str,
) -> Result<u64, String> {
    let qualified = format!("v2.workflow.required_floor.{func}");
    let value = v1_interpreter::run_in_context(ctx, &qualified, false)
        .map_err(|e| format!("{qualified}: {e}"))?;
    let v1_interpreter::Value::Record { fields, .. } = &value else {
        return Err(format!(
            "{qualified}: expected a std.measure Measure record, got {}",
            floor_value_shape(Some(&value))
        ));
    };
    let Some(count) = ctx.field(fields, "count") else {
        return Err(format!(
            "{qualified}: std.measure Measure record carries no `count` field"
        ));
    };
    match count {
        v1_interpreter::Value::Int(n) if *n > 0 => Ok(*n as u64),
        other => Err(format!(
            "{qualified}: expected a positive Int count, got {}",
            floor_value_shape(Some(other))
        )),
    }
}

pub(crate) fn floor_required_int(
    ctx: &v1_interpreter::InterpContext,
    func: &str,
) -> Result<u64, String> {
    let qualified = format!("v2.workflow.required_floor.{func}");
    // STRICTLY POSITIVE, because the deleted admission decoder refused non-positive budgets and
    // a replacement that accepts zero is a weaker wall wearing the same name. A zero ceiling is
    // not a lenient policy — it refuses every witness before it evaluates one.
    match v1_interpreter::run_in_context(ctx, &qualified, false) {
        Ok(v1_interpreter::Value::Int(n)) if n > 0 => Ok(n as u64),
        Ok(other) => Err(format!(
            "{qualified}: expected a non-negative Int, got {}",
            floor_value_shape(Some(&other))
        )),
        Err(e) => Err(format!("{qualified}: {e}")),
    }
}

pub(crate) fn floor_decode_list<'a>(
    ctx: &v1_interpreter::InterpContext,
    v: Option<&'a v1_interpreter::Value>,
) -> Result<Vec<&'a v1_interpreter::Value>, String> {
    let mut out = Vec::new();
    let mut cursor = match v {
        None => return Err("<field absent>".to_string()),
        Some(v1_interpreter::Value::List(xs)) => {
            return Ok(xs.iter().collect());
        }
        Some(other) => other,
    };
    loop {
        let v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } = cursor
        else {
            return Err(format!(
                "expected a FreeMonoid Empty/Cons chain, observed {}",
                floor_value_shape(Some(cursor))
            ));
        };
        if ctx.sym_eq(*variant_name, "Empty") {
            return Ok(out);
        }
        if !ctx.sym_eq(*variant_name, "Cons") {
            return Err(format!(
                "expected a FreeMonoid Empty/Cons chain, observed Variant({variant_name:?})"
            ));
        }
        let Some(head) = ctx.field(fields, "head") else {
            return Err("Cons carries no head".to_string());
        };
        let Some(tail) = ctx.field(fields, "tail") else {
            return Err("Cons carries no tail".to_string());
        };
        out.push(head);
        cursor = tail;
    }
}

pub(crate) fn floor_value_constructor(v: &v1_interpreter::Value) -> &'static str {
    match v {
        v1_interpreter::Value::Int(_) => "Int",
        v1_interpreter::Value::Str(_) => "Str",
        v1_interpreter::Value::Bool(_) => "Bool",
        v1_interpreter::Value::Null => "Null",
        _ => "<other constructor>",
    }
}

/// DECODE ONE AUTHORED-MODULE-NAME PREFIX ROSTER from the `.dag` authority.
///
/// NO EMPTY-ROSTER REFUSAL, and the first version of this decode had one. It was backwards: an
/// empty exception roster is the DESIRED terminal state, so refusing on it would make "at least
/// one exclusion must exist forever" a structural requirement of the floor. Once a home is
/// discharged, admitting its witnesses to the ordinary population is the intended result, not an
/// error. A decode that FAILS still refuses — a failed read and a legitimately empty roster are
/// different states, and only the first is a defect.
/// A hermetic frame over ONE module's exact scope, for evaluating that module's own
/// declarations by name. The floor's rosters live in the policy module and are read from its
/// scope; every other authority the floor evaluates by name (`gunbc.output_policy`,
/// `v2.workflow.floor_naming_hygiene`) is read from its own module's scope, never from the
/// policy module's — whether the policy module's closure happens to reach a module is a fact
/// about the corpus, not about the question being asked.
pub(crate) fn floor_authority_frame(
    prepared: &PreparedRepository,
    module_path: &str,
) -> Result<v1_interpreter::InterpContext, String> {
    let scope = claim_scope_for(prepared, module_path)?;
    Ok(evaluation_frame(
        &scope,
        v1_interpreter::ExecutionMode::Hermetic,
        None,
        None,
    ))
}

/// Install the cross-claim pure-producer share for one prepared floor subject: decode the
/// declared roster, install admission and the shared-fill observer, and warm every nullary
/// row so its fill is billed to preparation. FAIL-CLOSED at every arm: the roster module is
/// a declared closure seed (`REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES`), so a subject that
/// cannot frame it, a roster that cannot decode, a warm row whose module is gone, a warm
/// that fails to evaluate, and a warm whose value the store refuses each stop the line — a
/// skip at any of these arms would leave admission empty while CI reads green, memoizing
/// nothing (a green over a flag that never ran).
// RETURNS ITS OBSERVATIONS RATHER THAN JUST ITS ERRORS, because the warm fills are a shared
// preparation build and the preparation refusal is denominated over the observations its caller
// collects. Before this, the warm ran, printed a wall figure and produced nothing the adjudicator
// could see: it sits OUTSIDE the per-claim ceiling by construction (the deadlines are armed inside
// `run_claim_measured`), and it was outside the preparation limits too, because those iterate a
// vector this function never contributed to. Bounded by neither is not the same as billed to
// preparation. One observation per warm row, measured on the same clock and RSS reads as every
// other shared build, so all five phases go through ONE refusal.
/// AT MOST ONE MODULE'S AUTHORITY FRAME RESIDENT DURING THE PURE-PRODUCER WARM. A frame is a claim
/// scope over its module's closure -- scoped graph plus scope indexes -- and a warm row needs it only
/// while that module's rows are looked up and warmed. Holding every producer module's frame for the
/// whole warm phase kept them all live at once beside the prepared graph; freed together at the end,
/// they left the allocator's arena grown by their SUM, so the floor's resident peak carried several
/// GB of freed-but-unreturned memory (the uncensored #12799 floor-memory-qualification: the
/// prepared-subject-warm seam's arena and free columns). What outlives a frame is what the floor
/// keeps on purpose: producer nodes (Rc into the prepared graph) and portable stored values.
/// `builds` counts frame constructions, so the CPU side of re-framing on a module change is
/// reported beside the memory it saves.
/// ONE ROW PER WARMED PRODUCER at identity grain, (producer, portable-value digest), for a
/// producer-grain differential between two runs. THE DIGEST IS ONLY RUN-COMPARABLE OVER A
/// CANONICAL PORTABLE ENCODING: until gunbc#12895 puts map entries in content order, a
/// map-bearing value digests in its process's hash order, so two runs can print different
/// digests for one value (recurring_failure_mode portable_value_map_order_is_process_random).
/// A value that stored nothing to digest prints `none`.
fn floor_warm_row_identity(qualified: &str) {
    let bare = qualified.rsplit('.').next().unwrap_or(qualified);
    let digest =
        v1_interpreter::take_cross_claim_store_digest(bare).unwrap_or_else(|| "none".to_string());
    eprintln!("[floor-warm-row] producer={qualified} digest={digest}");
}

#[derive(Default)]
struct WarmFrameSlot {
    module: Option<String>,
    frame: Option<v1_interpreter::InterpContext>,
    builds: usize,
    framed: std::collections::HashSet<String>,
    build_ms: u128,
    drop_ms: u128,
}

impl WarmFrameSlot {
    fn frame(
        &mut self,
        prepared: &PreparedRepository,
        corpus_modules: &std::collections::HashSet<String>,
        outside_subject: &mut std::collections::BTreeSet<String>,
        module: &str,
        row_kind: &str,
        row: &str,
    ) -> Result<Option<&v1_interpreter::InterpContext>, String> {
        if self.module.as_deref() == Some(module) {
            return Ok(self.frame.as_ref());
        }
        if outside_subject.contains(module) {
            return Ok(None);
        }
        // A MODULE ALREADY FRAMED IS NOT FRAMED AGAIN: the warm's rows are grouped by module, so a
        // second build means a row arrived out of its group -- memory traded for repeated work. This
        // refusal is the wall; the closing counts only report.
        if self.framed.contains(module) {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareFrameRebuilt module={module} \
                 {row_kind}={row} -- the warm's rows are not grouped by module, so one-frame-at-a-time \
                 would rebuild a frame it already built"
            ));
        }
        // The held frame goes BEFORE the next is built, so two are never resident together.
        let dropping = std::time::Instant::now();
        self.frame = None;
        self.module = None;
        self.drop_ms += dropping.elapsed().as_millis();
        let building = std::time::Instant::now();
        let built = floor_authority_frame(prepared, module);
        self.build_ms += building.elapsed().as_millis();
        match built {
            Ok(frame) => {
                self.builds += 1;
                self.framed.insert(module.to_string());
                self.module = Some(module.to_string());
                self.frame = Some(frame);
                Ok(self.frame.as_ref())
            }
            Err(_) if !corpus_modules.contains(module) => Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRowModuleAbsentFromCorpus \
                 {row_kind}={row} module={module} — the roster row names a module no source \
                 root carries; the row is stale: delete it or restore the module"
            )),
            Err(why) if why.contains("cause=EntryModuleOutsidePreparedSubject") => {
                outside_subject.insert(module.to_string());
                Ok(None)
            }
            Err(why) => Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRowModuleUnframeable \
                 {row_kind}={row} module={module} — the module is in the corpus and was not \
                 excluded as outside the prepared subject, yet it does not frame; the row is \
                 NOT shown stale by this, do not delete it to get green: {why}"
            )),
        }
    }
}

/// The one-pass warm's module order: `candidates` already sorted by (row kind, name), reordered so
/// every module comes after the modules whose acquisitions its carried rows read (`depends_on`).
/// Ties keep the candidate order, so the result is deterministic. A dependency on a module outside
/// `candidates` does not block. A cycle has no order and refuses.
fn order_warm_modules(
    candidates: &[String],
    depends_on: &std::collections::HashMap<String, std::collections::BTreeSet<String>>,
) -> Result<Vec<String>, String> {
    let mut placed: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut out: Vec<String> = Vec::with_capacity(candidates.len());
    while out.len() < candidates.len() {
        let next = candidates.iter().find(|m| {
            !placed.contains(m.as_str())
                && depends_on.get(*m).is_none_or(|deps| {
                    deps.iter()
                        .all(|d| placed.contains(d.as_str()) || !candidates.contains(d))
                })
        });
        match next {
            Some(m) => {
                placed.insert(m.as_str());
                out.push(m.clone());
            }
            None => {
                let stuck: Vec<&String> = candidates
                    .iter()
                    .filter(|m| !placed.contains(m.as_str()))
                    .collect();
                return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=PureProducerShareModuleOrderCycle modules={stuck:?} \
                     -- these modules' carried rows read each other's acquisitions, so no one-pass \
                     module order binds every carried input before the row that reads it"
                ));
            }
        }
    }
    Ok(out)
}

pub(crate) fn install_pure_producer_share(
    prepared: &PreparedRepository,
    corpus_modules: &std::collections::HashSet<String>,
) -> Result<Vec<(String, SharedBuildObservation)>, String> {
    const FLOOR_PURE_PRODUCER_SHARE_MODULE: &str = "v2.workflow.floor_pure_producer_share";
    const CROSS_CLAIM_SHARE_CACHE: &str = "cross_claim_pure_share";
    let roster_frame =
        floor_authority_frame(prepared, FLOOR_PURE_PRODUCER_SHARE_MODULE).map_err(|why| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRosterOutsidePreparedSubject \
                 module={FLOOR_PURE_PRODUCER_SHARE_MODULE} — the roster is a declared closure \
                 seed and must be in every required-floor subject: {why}"
            )
        })?;
    // The two carried-input rosters are decoded HERE, beside the other two, because their
    // producers are part of the ADMITTED population: admission is one node set, and a producer
    // that warms into the tier but is not admitted would store nothing while the receipt read
    // green. The roster's own wall refuses a producer standing in both a plain roster and a
    // carried-input row, so the union is unambiguous by construction.
    let prepared_inputs = floor_decode_prepared_effect_inputs(
        &roster_frame,
        &format!("{FLOOR_PURE_PRODUCER_SHARE_MODULE}.floor_cross_claim_prepared_effect_inputs"),
    )?;
    let carried_rows = floor_decode_carried_input_warm_rows(
        &roster_frame,
        &format!("{FLOOR_PURE_PRODUCER_SHARE_MODULE}.floor_cross_claim_carried_input_warm_rows"),
    )?;

    // Admission is by RESOLVED DECLARATION IDENTITY (review 57446 F1): each qualified
    // roster spelling resolves to its fn node in a frame over the prepared subject, and the
    // interpreter admits by that node set — a bare-name homonym in a non-rostered module is
    // a different node and never eligible.
    //
    // THE ROSTER IS GLOBAL AND THE SUBJECT IS LOCAL, so admission is scoped to the subject
    // actually prepared. Three dispositions, and only one of them is staleness:
    //   1. the module frames in the prepared subject -> resolve and admit (an unresolved
    //      spelling there IS stale and stops the line);
    //   2. the module exists in the corpus but outside the prepared closure -> NOT evaluated,
    //      counted and announced. This is not a widen: the subject is closed under every edge
    //      `claim_scope_for` follows, so no claim in it can demand that producer, and not
    //      warming it relocates no fill onto any claim. The whole-corpus run on main still
    //      evaluates every row, so a broken row cannot hide behind narrow subjects;
    //   3. the module exists nowhere in the corpus -> a stale row; stop the line and say so.
    // Pulling every rostered module into every subject instead would re-grow the per-PR
    // closure toward the corpus on every roster append — cost denominated in the corpus, not
    // the change (DESIGN section 5), for fills no claim in the subject can consume.
    // ONE FRAME RESIDENT AT A TIME (WarmFrameSlot): a frame is a claim scope over its module's
    // closure, needed only while that module's rows are looked up and warmed.
    let mut frames = WarmFrameSlot::default();
    let mut outside_subject: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    // Returns Ok(true) when the module framed, Ok(false) when it is outside this subject but
    // present in the corpus (disposition 2), Err on staleness or any other framing failure.
    // ADMISSION NEEDS NODE IDENTITY, NOT A FRAME. A producer's node is the declaration in its module's
    // typed items -- the same Rc the frame's scope indexes are built from, which the warm checks by
    // pointer -- so admission reads the prepared graph instead of framing every producer module up
    // front. A module the prepared graph lacks is outside the subject when the corpus carries it and
    // a stale row when it does not.
    let prepared_modules: std::collections::HashMap<
        &str,
        &crate::v1_compiler_infer_items::TypedModule,
    > = prepared
        .graph
        .modules
        .iter()
        .map(|m| (m.type_env.module_path.as_str(), m.as_ref()))
        .collect();
    let mut admitted_nodes = Vec::new();
    let mut admitted_qualified: Vec<String> = Vec::new();
    let carried_producers: Vec<String> = carried_rows.iter().map(|r| r.producer.clone()).collect();
    // The frozen runtime-identity residual: producers whose every call site carries arguments
    // computed inside the claim, so the derivation cannot join their demand before the run. The
    // population is closed in `.dag` (`FrozenRuntimeIdentityProducer`); the tier keys and verifies
    // their fills by declaration and argument row like any other.
    let residual_name = format!(
        "{FLOOR_PURE_PRODUCER_SHARE_MODULE}.floor_cross_claim_runtime_identity_residual_producers"
    );
    let residual_producers: Vec<String> = {
        let value = v1_interpreter::run_in_context(&roster_frame, &residual_name, false).map_err(
            |e| {
                format!(
                    "REQUIRED-FLOOR REFUSAL cause=RuntimeIdentityResidualUnreadable --                      {residual_name} did not evaluate: {e}"
                )
            },
        )?;
        let items = v1_interpreter::list_value_items(&roster_frame, &value).ok_or_else(|| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=RuntimeIdentityResidualUnreadable --                  {residual_name} is not a list"
            )
        })?;
        let mut out = Vec::new();
        for item in &items {
            let v1_interpreter::Value::Str(producer) = item else {
                return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=RuntimeIdentityResidualUnreadable --                      {residual_name} carries a member that is not a String"
                ));
            };
            out.push(producer.to_string());
        }
        out
    };
    // THE PRODUCERS ADMITTED HERE ARE THE CARRIED-INPUT ONES AND THE FROZEN RESIDUAL. Plain pure
    // producers are no longer a roster: they are derived from the planned claims' call-site
    // demand once planning has fixed the claims (`derive_and_install_cross_claim_share`).
    for qualified in carried_producers.iter().chain(residual_producers.iter()) {
        let (module, decl) = match qualified.rsplit_once('.') {
            Some((module, decl)) => (module.to_string(), decl),
            None => (qualified.clone(), qualified.as_str()),
        };
        let Some(typed) = prepared_modules.get(module.as_str()) else {
            if corpus_modules.contains(&module) {
                outside_subject.insert(module);
                continue;
            }
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRowModuleAbsentFromCorpus \
                 producer={qualified} module={module} — the roster row names a module no source \
                 root carries; the row is stale: delete it or restore the module"
            ));
        };
        let node = typed
            .items
            .iter()
            .find(|n| n.name == decl)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "REQUIRED-FLOOR REFUSAL cause=PureProducerShareProducerUnresolved \
                     producer={qualified} — the rostered spelling names no declaration in its \
                     module; fix or delete the roster row"
                )
            })?;
        admitted_nodes.push(node);
        admitted_qualified.push(qualified.clone());
    }
    let refused = floor_decode_refused_share_candidates(
        &roster_frame,
        &format!("{FLOOR_PURE_PRODUCER_SHARE_MODULE}.floor_cross_claim_refused_candidates"),
    )?;
    PURE_PRODUCER_SHARE_ROSTER.with(|r| {
        *r.borrow_mut() = Some(PureProducerShareRoster {
            admitted_qualified,
            refused,
        });
    });
    v1_interpreter::install_cross_claim_pure_share_roster(admitted_nodes);
    v1_interpreter::install_cross_claim_share_observer(Some(
        v1_interpreter::CrossClaimShareObserver {
            on_fill_begin: Box::new(crate::cli_run::shared_fill::begin_fill),
            on_fill: Box::new(|name, inclusive_wall, self_wall| {
                crate::cli_run::shared_fill::record_fill(
                    CROSS_CLAIM_SHARE_CACHE,
                    name,
                    inclusive_wall as u64,
                );
                crate::cli_run::record_shared_artifact_fill_wall(self_wall);
            }),
            on_fill_abandon: Box::new(crate::cli_run::shared_fill::abandon_fill),
            on_hit: Box::new(|name| {
                crate::cli_run::shared_fill::record_hit(CROSS_CLAIM_SHARE_CACHE, name)
            }),
        },
    ));
    let mut warm_observations: Vec<(String, SharedBuildObservation)> = Vec::new();

    // ── THE PREPARED EFFECT INPUTS, ACQUIRED ONCE AND CARRIED ────────────────────────────
    //
    // Ordered BEFORE the plain warm loop because a carried-input row cannot be warmed until its
    // input is bound, and every arm here stops the line: a row whose module the subject does not
    // carry, a spelling that resolves to nothing, an acquisition that is not nullary, an
    // acquisition that fails to evaluate, a value that is not portable. None of them has an empty
    // default — an empty carry would be served to every claim as though it were the carrier's
    // content, which is the fabricated-plausible-output failure with a cache in front of it.
    //
    // WHAT THE FLOOR DOES NOT DO IS DECIDE WHAT THE CARRIED VALUE MEANS. When the acquisition
    // SUCCEEDS and returns its own coproduct's refusal arm, that refusal is carried faithfully and
    // every claim receives the identical typed value it would have computed for itself — no claim
    // passes where it would otherwise have failed, which is what separates propagating a refusal
    // from widening one. The disposition is PRINTED (the carried variant name and content digest)
    // so a run whose carrier refused is diagnosable instead of reading as a corpus problem
    // (adjudicated 2026-09-14; bright-boar-435's condition on that ruling).
    let mut acquisition_nodes: std::collections::HashMap<
        String,
        std::rc::Rc<crate::v1_std_core::Node>,
    > = std::collections::HashMap::new();
    let mut inputs_outside_subject: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let module_of = |q: &str| {
        q.rsplit_once('.')
            .map(|(m, _)| m.to_string())
            .unwrap_or_else(|| q.to_string())
    };
    // ONE PASS, GROUPED BY MODULE. Every row a module owns -- its acquisitions, then its
    // carried-input producers, then its plain warms -- runs while that module's frame is the one
    // held, so each module is framed exactly once for the whole warm. Modules are ordered by the
    // DEPENDENCY the rows declare: a module whose carried row reads an acquisition another module
    // owns runs after that module (a topological order, ties broken by (row kind, name) so the order
    // is deterministic). Within a module its own acquisitions precede its carried rows, so every
    // carried row's input is bound before it runs. A dependency cycle across modules has no such
    // order and refuses (`PureProducerShareModuleOrderCycle`).
    let rank_of = |module: &str| -> u8 {
        if prepared_inputs
            .iter()
            .any(|i| module_of(&i.acquisition) == module)
        {
            0
        } else if carried_rows
            .iter()
            .any(|r| module_of(&r.producer) == module)
        {
            1
        } else {
            2
        }
    };
    let mut ordered_modules: Vec<String> = prepared_inputs
        .iter()
        .map(|i| module_of(&i.acquisition))
        .chain(carried_rows.iter().map(|r| module_of(&r.producer)))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    ordered_modules.sort_by_key(|m| (rank_of(m), m.clone()));
    let acquisition_module: std::collections::HashMap<&str, String> = prepared_inputs
        .iter()
        .map(|i| (i.acquisition.as_str(), module_of(&i.acquisition)))
        .collect();
    // module -> the other modules whose acquisitions its carried rows read.
    let mut depends_on: std::collections::HashMap<String, std::collections::BTreeSet<String>> =
        std::collections::HashMap::new();
    for row in &carried_rows {
        let producer_module = module_of(&row.producer);
        if let Some(input_module) = acquisition_module.get(row.carried_input.as_str()) {
            if *input_module != producer_module {
                depends_on
                    .entry(producer_module)
                    .or_default()
                    .insert(input_module.clone());
            }
        }
    }
    let ordered_modules = order_warm_modules(&ordered_modules, &depends_on)?;
    for group in &ordered_modules {
        for input in prepared_inputs
            .iter()
            .filter(|i| &module_of(&i.acquisition) == group)
        {
            let module = match input.acquisition.rsplit_once('.') {
                Some((module, _)) => module.to_string(),
                None => input.acquisition.clone(),
            };
            let Some(frame) = frames.frame(
                prepared,
                corpus_modules,
                &mut outside_subject,
                &module,
                "acquisition",
                &input.acquisition,
            )?
            else {
                inputs_outside_subject.insert(input.acquisition.clone());
                continue;
            };
            let node = frame.lookup_fn_node(&input.acquisition).ok_or_else(|| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=PreparedEffectInputUnresolved acquisition={} — the rostered spelling names no declaration in its module's frame; fix or delete the roster row",
                input.acquisition
            )
        })?;
            let (acquired, observation) = observe_shared_build(false, "floor-preparation", || {
                v1_interpreter::acquire_prepared_effect_input(frame, &input.acquisition)
            });
            let carry = acquired.map_err(|why| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=PreparedEffectInputAcquisitionFailed acquisition={} checkout_input={} — {why}",
                input.acquisition, input.checkout_input
            )
        })?;
            eprintln!(
            "[floor-phase] phase=prepared-effect-input-acquire state=completed acquisition={} checkout_input={} content_digest={} disposition={} cpu_ms={} wall_ms={} rss_growth_bytes={}",
            input.acquisition,
            input.checkout_input,
            carry.digest,
            carry.disposition(),
            observation.cpu_ms,
            observation.wall_ms,
            observation.rss_growth_bytes,
        );
            v1_interpreter::install_prepared_effect_input(&node, carry);
            acquisition_nodes.insert(input.acquisition.clone(), node);
            warm_observations.push((
                format!("PreparedEffectInputAcquire/{}", input.acquisition),
                observation,
            ));
        }
        for row in carried_rows
            .iter()
            .filter(|r| &module_of(&r.producer) == group)
        {
            let producer_module = match row.producer.rsplit_once('.') {
                Some((module, _)) => module,
                None => row.producer.as_str(),
            };
            if outside_subject.contains(producer_module) {
                continue;
            }
            // The producer is demandable in this subject but its input was not prepared here: the
            // claim would be served an uncarried value. Refuse, and name the real cause rather
            // than reporting the input as unknown to the roster.
            if inputs_outside_subject.contains(&row.carried_input) {
                return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=CarriedInputWarmRowInputOutsideSubject producer={} input={} — the producer's module is in the prepared subject but its carried input's module is not; the closure must carry the acquisition's module (an import from the producer's module is the edge preparation follows)",
                row.producer, row.carried_input
            ));
            }
            if !acquisition_nodes.contains_key(&row.carried_input)
                && prepared_inputs
                    .iter()
                    .any(|i| i.acquisition == row.carried_input)
            {
                return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=CarriedInputWarmRowInputNotYetBound producer={} input={} \
                 -- the one-pass warm reached this carried row before its acquisition's module, so the \
                 module order does not put every acquisition first",
                row.producer, row.carried_input
            ));
            }
            let acquisition_node = acquisition_nodes.get(&row.carried_input).ok_or_else(|| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=CarriedInputWarmRowInputUnknown producer={} input={} — the row declares a dependence on an input no floor_cross_claim_prepared_effect_inputs row prepares, so the value it names would never be bound",
                row.producer, row.carried_input
            )
        })?;
            let module = match row.producer.rsplit_once('.') {
                Some((module, _)) => module.to_string(),
                None => row.producer.clone(),
            };
            // Resolution above already framed every rostered producer's module, carried-input rows
            // included (they are part of the admitted population), so the frame is present.
            let Some(frame) = frames.frame(
                prepared,
                corpus_modules,
                &mut outside_subject,
                &module,
                "producer",
                &row.producer,
            )?
            else {
                continue;
            };
            let producer_node = frame.lookup_fn_node(&row.producer).ok_or_else(|| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareProducerUnresolved producer={} — the rostered spelling names no declaration in its module's frame",
                row.producer
            )
        })?;
            // The implicit binding is installed ONLY for the shape that needs it: a `BoundParameter`
            // row's caller passes the carried value itself, so its key already represents the
            // content and folding it in a second time would key one call two ways.
            if row.bound_parameter.is_none() {
                v1_interpreter::install_carried_input_producer(&producer_node, acquisition_node)
                    .map_err(|why| {
                        format!(
                            "REQUIRED-FLOOR REFUSAL cause=CarriedInputWarmRowInputUnknown \
                         producer={} input={} — {why}",
                            row.producer, row.carried_input
                        )
                    })?;
            }
            let (warm_result, warm_observation) =
                observe_shared_build(false, "floor-preparation", || {
                    v1_interpreter::warm_cross_claim_carried_input_producer(
                        frame,
                        &row.producer,
                        &row.carried_input,
                        row.bound_parameter.as_deref(),
                    )
                });
            match warm_result {
                Ok(outcome) => {
                    if !outcome.is_servable() {
                        let detail = match outcome.not_portable_detail() {
                            Some(refusal) => format!(
                                "{} path={} kind={}",
                                outcome.cause(),
                                if refusal.path_into_value.is_empty() {
                                    "<root>"
                                } else {
                                    refusal.path_into_value.as_str()
                                },
                                refusal.encountered_kind
                            ),
                            None => outcome.cause().to_string(),
                        };
                        return Err(format!(
                        "REQUIRED-FLOOR REFUSAL cause=PureProducerShareWarmNotStored producer={} — the carried-input producer evaluated but its value was refused by the cross-claim store: {detail}",
                        row.producer
                    ));
                    }
                    floor_warm_row_identity(&row.producer);
                    eprintln!(
                    "[floor-phase] phase=prepared-effect-input-warm state=completed producer={} input={} disposition={} cpu_ms={} wall_ms={} rss_growth_bytes={}",
                    row.producer,
                    row.carried_input,
                    outcome.cause(),
                    warm_observation.cpu_ms,
                    warm_observation.wall_ms,
                    warm_observation.rss_growth_bytes,
                );
                    warm_observations.push((
                        format!("CrossClaimCarriedInputWarm/{}", row.producer),
                        warm_observation,
                    ));
                }
                Err(why) => {
                    return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=PureProducerShareWarmFailed producer={} — {why}",
                    row.producer
                ));
                }
            }
        }
    }
    // Disposition 2 is COUNTED, one line, so a subject that quietly stopped evaluating rows
    // it should carry is visible in the run's own announcement.
    eprintln!(
        "[floor-phase] phase=pure-producer-share-scope state=completed \
         rostered_modules_outside_subject={} modules=[{}]",
        outside_subject.len(),
        outside_subject
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(",")
    );
    // REPORTED, NOT A CHECK: the rebuild refusal above enforces one frame per module. The two counts
    // come from different sources -- builds from the slot, roster modules from the rows -- so a module
    // whose rows never reached a frame shows as a shortfall rather than vanishing.
    let roster_modules_in_subject = ordered_modules
        .iter()
        .filter(|m| !outside_subject.contains(m.as_str()))
        .count();
    eprintln!(
        "[floor-phase] phase=pure-producer-share-frames state=completed frame_builds={} \
         roster_modules_in_subject={} (one frame resident at a time, rows grouped by module) \
         frame_build_ms={} frame_drop_ms={} last_frame_drop_ms={}",
        frames.builds,
        roster_modules_in_subject,
        frames.build_ms,
        frames.drop_ms,
        {
            let dropping = std::time::Instant::now();
            frames.frame = None;
            dropping.elapsed().as_millis()
        }
    );
    drop(frames);
    Ok(warm_observations)
}

/// THE DERIVED CROSS-CLAIM SHARE: admit, for this run, exactly the pure computations its PLANNED
/// claims demand from more than one claim with one closed identity.
///
/// The seed observes (`claim_call_site_demand`, one realization of the `.dag` row type
/// `CallSiteDemandObservation`) and `v2.workflow.floor_pure_producer_share`
/// `floor_cross_claim_share_derivation` decides; the seed then admits the decided rows at their call
/// sites. There is no warm: the first planned claim that evaluates an admitted site fills it (see the
/// install note at the end of this function). Every arm refuses: a planned claim the prepared subject does not carry, a fold
/// that does not evaluate or decode, a partition that does not reconcile, and an admitted producer
/// with no declaration node or an unparseable site.
pub(crate) fn derive_and_install_cross_claim_share(
    prepared: &PreparedRepository,
    claims: &[RequiredFloorClaim],
    declared: &[(String, String)],
    in_flight_wall_cap_ms: u64,
) -> Result<(), String> {
    use super::claim_call_site_demand::{
        CallSiteDemandObserver, CallSiteDemandRow, ConsumerRead, CLOSED_ARGUMENT_NORMALIZER,
    };
    use v1_interpreter::Value;
    const MODULE: &str = "v2.workflow.floor_pure_producer_share";
    let started = std::time::Instant::now();
    let mut population: Vec<(String, String)> = claims
        .iter()
        .map(|c| (c.module_path.clone(), c.function.clone()))
        .collect();
    let planned_count = population.len();
    let mut observer =
        CallSiteDemandObserver::new(&prepared.graph, prepared.source_indices.clone());
    // THE DECLARED POPULATION: every other claim declared in a module the prepared subject carries.
    // Recurrence is judged over it (so a claim's budget does not depend on which other claims a diff
    // plans); a declared claim whose module is outside the subject cannot be walked and is counted.
    let planned_set: std::collections::HashSet<(String, String)> =
        population.iter().cloned().collect();
    let mut declared_outside_subject = 0usize;
    for d in declared {
        if planned_set.contains(d) {
            continue;
        }
        if observer.decl(&d.0, &d.1).is_some() {
            population.push(d.clone());
        } else {
            declared_outside_subject += 1;
        }
    }
    let (rows, unresolved) = observer.observe(&population, planned_count);
    if !unresolved.is_empty() {
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=CallSiteDemandClaimUnresolved claims=[{}] -- a planned \
             claim's declaration is not in the prepared subject, so its call-site demand cannot be \
             observed and the derived share would omit it",
            unresolved.join(",")
        ));
    }
    let observe_ms = started.elapsed().as_millis();
    for (producer, cause, claims_n, sites_n) in observer.open_producers() {
        eprintln!(
            "[cross-claim-share-unadmissible] producer={producer} cause={} claims={claims_n} \
             sites={sites_n}",
            cause.variant()
        );
    }
    let frame = floor_authority_frame(prepared, MODULE).map_err(|why| {
        format!(
            "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRosterOutsidePreparedSubject \
             module={MODULE} -- {why}"
        )
    })?;
    let ctx = &frame;
    let sym = |s: &str| ctx.sym(s);
    let unit = |ty: &str, variant: &str| Value::Variant {
        type_name: sym(ty),
        variant_name: sym(variant),
        fields: std::rc::Rc::new(vec![]),
    };
    let closed_rows = rows
        .iter()
        .filter(|r| matches!(r, CallSiteDemandRow::Closed { .. }))
        .count();
    let values: Vec<Value> = rows
        .iter()
        .map(|row| match row {
            CallSiteDemandRow::Closed {
                producer,
                argument_preimage,
                claims,
                planned_claims,
                sites,
                reads,
            } => Value::Variant {
                type_name: sym("CallSiteDemandObservation"),
                variant_name: sym("ClosedCallSiteDemand"),
                fields: std::rc::Rc::new(vec![
                    (sym("producer"), str_value(producer)),
                    (sym("argument_preimage"), str_value(argument_preimage)),
                    (
                        sym("identity"),
                        Value::Variant {
                            type_name: sym("ComputationIdentity"),
                            variant_name: sym("NormalizedIdentical"),
                            fields: std::rc::Rc::new(vec![(
                                sym("normalizer"),
                                str_value(CLOSED_ARGUMENT_NORMALIZER),
                            )]),
                        },
                    ),
                    (
                        sym("claims"),
                        list_value_from_vec(claims.iter().map(str_value).collect()),
                    ),
                    (sym("planned_claims"), Value::Int(*planned_claims as i64)),
                    (
                        sym("sites"),
                        list_value_from_vec(sites.iter().map(str_value).collect()),
                    ),
                    (
                        sym("reads"),
                        list_value_from_vec(
                            reads
                                .iter()
                                .map(|(claim, read)| {
                                    let read = match read {
                                        ConsumerRead::WholeValue => {
                                            unit("ConsumerRead", "WholeValue")
                                        }
                                        ConsumerRead::ProjectedField(field) => Value::Variant {
                                            type_name: sym("ConsumerRead"),
                                            variant_name: sym("ProjectedField"),
                                            fields: std::rc::Rc::new(vec![(
                                                sym("field"),
                                                str_value(field),
                                            )]),
                                        },
                                        ConsumerRead::Unread(cause) => Value::Variant {
                                            type_name: sym("ConsumerRead"),
                                            variant_name: sym("ConsumerReadUnobserved"),
                                            fields: std::rc::Rc::new(vec![(
                                                sym("cause"),
                                                unit("ConsumerReadCause", cause.variant()),
                                            )]),
                                        },
                                    };
                                    Value::Record {
                                        type_name: sym("ClaimConsumerRead"),
                                        fields: std::rc::Rc::new(vec![
                                            (sym("claim"), str_value(claim)),
                                            (sym("read"), read),
                                        ]),
                                    }
                                })
                                .collect(),
                        ),
                    ),
                ]),
            },
            CallSiteDemandRow::Unadmissible {
                cause,
                claims,
                sites,
            } => Value::Variant {
                type_name: sym("CallSiteDemandObservation"),
                variant_name: sym("UnadmissibleCallSiteDemand"),
                fields: std::rc::Rc::new(vec![
                    (sym("cause"), unit("CallSiteDemandCause", cause.variant())),
                    (sym("claims"), Value::Int(*claims as i64)),
                    (sym("sites"), Value::Int(*sites as i64)),
                ]),
            },
        })
        .collect();
    let result = v1_interpreter::run_in_context_with_args(
        ctx,
        &format!("{MODULE}.floor_cross_claim_share_derivation"),
        &[(
            Some("observations".to_string()),
            list_value_from_vec(values),
        )],
        false,
    )
    .map_err(|e| {
        format!(
            "REQUIRED-FLOOR REFUSAL cause=CrossClaimShareDerivationFailed -- \
             floor_cross_claim_share_derivation did not evaluate: {e}"
        )
    })?;
    let derive_ms = started.elapsed().as_millis() - observe_ms;
    let malformed = |what: &str| {
        format!(
            "REQUIRED-FLOOR REFUSAL cause=CrossClaimShareDerivationUndecodable -- {what} (got `{}`)",
            ctx.format_value(&result)
        )
    };
    let Value::Record { fields, .. } = &result else {
        return Err(malformed("expected a CrossClaimShareDerivation record"));
    };
    // The fold's lists may be kernel lists or std.algebra FreeMonoid chains; both decode.
    let list_of = |name: &str| -> Result<Vec<Value>, String> {
        ctx.field(fields, name)
            .and_then(|v| v1_interpreter::list_value_items(ctx, v))
            .ok_or_else(|| malformed(&format!("no `{name}` list")))
    };
    let text_of = |row: &[(v1_interpreter::Symbol, Value)], name: &str| -> Result<String, String> {
        match ctx.field(row, name) {
            Some(Value::Str(s)) => Ok(s.to_string()),
            _ => Err(malformed(&format!("a row has no `{name}` String"))),
        }
    };
    let int_of = |row: &[(v1_interpreter::Symbol, Value)], name: &str| -> Result<i64, String> {
        match ctx.field(row, name) {
            Some(Value::Int(n)) => Ok(*n),
            _ => Err(malformed(&format!("a row has no `{name}` Int"))),
        }
    };
    let variant_of =
        |row: &[(v1_interpreter::Symbol, Value)], name: &str| -> Result<String, String> {
            match ctx.field(row, name) {
                Some(Value::Variant { variant_name, .. }) => Ok(ctx.resolve(*variant_name)),
                _ => Err(malformed(&format!("a row has no `{name}` variant"))),
            }
        };
    let admitted = list_of("admitted")?;
    let declined = list_of("declined")?;
    let unadmissible = list_of("unadmissible")?;
    if admitted.len() + declined.len() != closed_rows {
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=CrossClaimShareDerivationDoesNotReconcile \
             closed_observations={closed_rows} admitted={} declined={} -- every closed observation \
             must be admitted or declined exactly once",
            admitted.len(),
            declined.len()
        ));
    }
    let mut nodes = Vec::new();
    let mut sites: std::collections::HashSet<(String, i64, i64)> = std::collections::HashSet::new();
    let mut admitted_qualified = Vec::new();
    let mut billed_debt_claims: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    let mut net_only_sites: std::collections::HashSet<(String, i64, i64)> =
        std::collections::HashSet::new();
    for row in &admitted {
        let Value::Record { fields: r, .. } = row else {
            return Err(malformed("an admitted row is not a DerivedShareRow record"));
        };
        let producer = text_of(r, "producer")?;
        let preimage = text_of(r, "argument_preimage")?;
        let claims_n = int_of(r, "claims")?;
        let (module, decl) = producer.rsplit_once('.').ok_or_else(|| {
            malformed(&format!("admitted producer `{producer}` is not qualified"))
        })?;
        let node = observer.decl(module, decl).cloned().ok_or_else(|| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=DerivedShareProducerUnresolved producer={producer} \
                 -- the derivation admitted a producer the prepared subject carries no declaration for"
            )
        })?;
        let row_sites = ctx
            .field(r, "sites")
            .and_then(|v| v1_interpreter::list_value_items(ctx, v))
            .ok_or_else(|| malformed("an admitted row has no `sites` list"))?;
        let mut row_keys: Vec<(String, i64, i64)> = Vec::new();
        for site in &row_sites {
            let Value::Str(text) = site else {
                return Err(malformed("a site is not a String"));
            };
            let parsed = text.rsplit_once(':').and_then(|(file, span)| {
                let (start, end) = span.split_once('-')?;
                Some((file.to_string(), start.parse().ok()?, end.parse().ok()?))
            });
            let Some(key) = parsed else {
                return Err(malformed(&format!(
                    "site `{text}` is not <file>:<start>-<end>"
                )));
            };
            row_keys.push(key);
        }
        // WHY the row is admitted, printed so a single-claim fill debt is never read as sharing.
        let mut row_is_debt = false;
        let basis = match ctx.field(r, "basis") {
            Some(Value::Variant {
                variant_name,
                fields: b,
                ..
            }) => {
                let variant = ctx.resolve(*variant_name);
                if variant == "SingleClaimFillDebt" {
                    let claim = text_of(b, "claim")?;
                    billed_debt_claims.insert(claim.clone());
                    row_is_debt = true;
                    format!("SingleClaimFillDebt:{claim}")
                } else {
                    variant
                }
            }
            _ => return Err(malformed("an admitted row has no `basis` variant")),
        };
        // A debt row's sites are NET-ONLY: the fill is netted from its one claim and no value is
        // retained, since no other claim will ever ask for it.
        for key in row_keys {
            if row_is_debt {
                net_only_sites.insert(key.clone());
            }
            sites.insert(key);
        }
        eprintln!(
            "[cross-claim-share-admitted] producer={producer} basis={basis} claims={claims_n} \
             sites={} argument_preimage={preimage}",
            row_sites.len()
        );
        admitted_qualified.push(producer);
        nodes.push(node);
    }
    let mut decline_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for row in &declined {
        let Value::Record { fields: r, .. } = row else {
            return Err(malformed("a declined row is not a DeclinedShareRow record"));
        };
        *decline_counts.entry(variant_of(r, "decline")?).or_default() += 1;
        // EVERY decline is printed, single-claim ones included, so each producer's disposition is
        // readable by identity from the run's own log. A bundle names the slices only one claim
        // reads, and an unreadable consumer names its cause.
        let detail = match ctx.field(r, "decline") {
            Some(Value::Variant {
                variant_name,
                fields: d,
                ..
            }) => match ctx.resolve(*variant_name).as_str() {
                "BundleOfDisjointProjections" => {
                    let slices = ctx
                        .field(d, "sole_projections")
                        .and_then(|v| v1_interpreter::list_value_items(ctx, v))
                        .ok_or_else(|| malformed("a bundle decline has no `sole_projections`"))?;
                    let mut names = Vec::new();
                    for s in &slices {
                        let Value::Str(s) = s else {
                            return Err(malformed("a sole projection is not a String"));
                        };
                        names.push(s.to_string());
                    }
                    format!(" sole_projections=[{}]", names.join(","))
                }
                "ConsumerReadUnknown" => format!(" cause={}", variant_of(d, "cause")?),
                _ => String::new(),
            },
            _ => return Err(malformed("a declined row has no `decline` variant")),
        };
        eprintln!(
            "[cross-claim-share-declined] producer={} decline={}{detail} argument_preimage={}",
            text_of(r, "producer")?,
            variant_of(r, "decline")?,
            text_of(r, "argument_preimage")?
        );
    }
    let mut unadmissible_rendered = Vec::new();
    for row in &unadmissible {
        let Value::Record { fields: r, .. } = row else {
            return Err(malformed(
                "an unadmissible row is not an UnadmissibleDemandCount record",
            ));
        };
        unadmissible_rendered.push(format!(
            "{}:claims={}:sites={}",
            variant_of(r, "cause")?,
            int_of(r, "claims")?,
            int_of(r, "sites")?
        ));
    }
    eprintln!(
        "[floor-phase] phase=cross-claim-share-derivation state=completed planned_claims={} \
         declared_claims_observed={} declared_outside_subject={declared_outside_subject} \
         closed_identities={closed_rows} admitted={} admitted_sites={} declined=[{}] \
         unadmissible=[{}] observe_ms={observe_ms} derive_ms={derive_ms}",
        claims.len(),
        population.len(),
        admitted.len(),
        sites.len(),
        decline_counts
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(","),
        unadmissible_rendered.join(",")
    );
    // THE DEBT CONTRACT'S IDENTITY JOIN (v2.workflow.floor_pure_producer_share
    // floor_single_claim_fill_debt, a monotone debt set): every ACTIVE member this run PLANS must
    // have at least one single-claim fixture identity admitted on its behalf. A planned member with
    // none is STALE -- it was restructured, or its fixture became shared, or its cost is not a
    // closed fixture at all -- and a stale row would keep asserting a transfer that no longer
    // happens, so it stops the line and names the remedy.
    let active_debt = match v1_interpreter::run_in_context(
        ctx,
        &format!("{MODULE}.floor_single_claim_fill_debt_active_claims"),
        false,
    ) {
        Ok(v) => v1_interpreter::list_value_items(ctx, &v)
            .ok_or_else(|| malformed("floor_single_claim_fill_debt_active_claims is not a list"))?,
        Err(e) => {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=SingleClaimFillDebtUnreadable -- \
                 floor_single_claim_fill_debt_active_claims did not evaluate: {e}"
            ))
        }
    };
    let planned_identities: std::collections::HashSet<String> = claims
        .iter()
        .map(|c| format!("{}.{}", c.module_path, c.function))
        .collect();
    let mut active_planned = 0usize;
    let mut stale_debt: Vec<String> = Vec::new();
    for member in &active_debt {
        let Value::Str(member) = member else {
            return Err(malformed("a fill-debt member is not a String"));
        };
        let member = member.to_string();
        if planned_identities.contains(&member) {
            active_planned += 1;
            if !billed_debt_claims.contains(&member) {
                stale_debt.push(member);
            }
        }
    }
    eprintln!(
        "[floor-phase] phase=single-claim-fill-debt state=completed active_members={} \
         planned_members={active_planned} billed_members={} stale_members={}",
        active_debt.len(),
        billed_debt_claims.len(),
        stale_debt.len()
    );
    if !stale_debt.is_empty() {
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=SingleClaimFillDebtStale members=[{}] -- each is an ACTIVE \
             member of v2.workflow.floor_pure_producer_share floor_single_claim_fill_debt that this \
             run plans, and no single-claim fixture identity was admitted for it, so the row asserts \
             a cost transfer that no longer happens. Retire the row with its typed disposition \
             (RestructuredPerWitnessRule, BecameSharedByDemand or ClaimDeleted).",
            stale_debt.join(",")
        ));
    }
    v1_interpreter::install_cross_claim_derived_share_with_net_only(nodes, sites, net_only_sites);
    PURE_PRODUCER_SHARE_ROSTER.with(|r| {
        if let Some(roster) = r.borrow_mut().as_mut() {
            roster.admitted_qualified.extend(admitted_qualified);
        }
    });
    // NO WARM. An admitted site fills on the FIRST planned claim that actually evaluates it, so a
    // site the static reach over-approximates costs nothing, and no frame is built just to measure
    // a value (the first derived runs built one frame per site module -- 169 -- to warm 2523
    // identities and then discarded 2404 below the floor). The fill's evaluator steps are netted
    // from the paying claim's budget by the existing fill guard, so budgets stay deterministic; its
    // wall is excused from the claim's wall deadline only in proportion to the steps it has
    // performed (a declared ns-per-step ceiling), under the preparation wall safety limit as the
    // outer hard cap, so a stalled fill and a runaway one both still interrupt; and the cost floor
    // is applied to the fill's own measured steps at the moment it would be retained.
    let cost_floor_steps = match v1_interpreter::run_in_context(
        ctx,
        &format!("{MODULE}.floor_cross_claim_share_cost_floor_eval_steps"),
        false,
    ) {
        Ok(Value::Int(n)) if n > 0 => n as u64,
        other => {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=CrossClaimShareCostFloorUnreadable -- \
                 floor_cross_claim_share_cost_floor_eval_steps must be a positive Int, got {}",
                match other {
                    Ok(v) => ctx.format_value(&v),
                    Err(e) => e.to_string(),
                }
            ))
        }
    };
    v1_interpreter::install_cross_claim_cost_floor_steps(cost_floor_steps);
    // The floor has a CPU arm: a fill whose thread CPU reaches the declared floor is retained
    // however few steps it performed. A fill whose cost is native (keying a served producer's
    // argument row) is otherwise declined on its step count and re-paid by every claim.
    let cost_floor_cpu_ms = match v1_interpreter::run_in_context(
        ctx,
        &format!("{MODULE}.floor_cross_claim_share_cost_floor_cpu_millisecond_count"),
        false,
    ) {
        Ok(Value::Int(n)) if n > 0 => n as u64,
        other => {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=CrossClaimShareCostFloorCpuUnreadable -- \
                 floor_cross_claim_share_cost_floor_cpu_millisecond_count must be a positive \
                 Int, got {}",
                match other {
                    Ok(v) => ctx.format_value(&v),
                    Err(e) => e.to_string(),
                }
            ))
        }
    };
    v1_interpreter::install_cross_claim_cost_floor_cpu_ms(cost_floor_cpu_ms);
    let ns_per_step_ceiling = match v1_interpreter::run_in_context(
        ctx,
        &format!("{MODULE}.floor_cross_claim_fill_wall_per_step_ceiling_nanosecond_count"),
        false,
    ) {
        Ok(Value::Int(n)) if n > 0 => n as u64,
        other => {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=CrossClaimFillWallCeilingUnreadable -- \
                 floor_cross_claim_fill_wall_per_step_ceiling_nanosecond_count must be a positive Int, got {}",
                match other {
                    Ok(v) => ctx.format_value(&v),
                    Err(e) => e.to_string(),
                }
            ))
        }
    };
    v1_interpreter::install_cross_claim_in_flight_wall_bound(
        in_flight_wall_cap_ms,
        ns_per_step_ceiling,
    );
    eprintln!(
        "[floor-phase] phase=cross-claim-share-install state=completed \
         cost_floor_eval_steps={cost_floor_steps} fill_wall_ns_per_step_ceiling={ns_per_step_ceiling} \
         in_flight_wall_outer_cap_ms={in_flight_wall_cap_ms}"
    );
    Ok(())
}

/// One row of `v2.workflow.floor_pure_producer_share.floor_cross_claim_refused_candidates`:
/// a producer that was proposed for the cross-claim share tier, MEASURED, and refused.
#[derive(Clone)]
pub(crate) struct RefusedShareRow {
    producer: String,
    verdict: String,
    /// The consuming modules the refusal was measured over, read from the deciding run's own
    /// `[floor-shared-fill]` `modules=` field.
    carrier_modules: std::collections::BTreeSet<String>,
}

/// What the fold needs at the END of the run to adjudicate the refused roster: the admitted
/// spellings (to name the subject of an overlap) and the refused rows (to join against).
#[derive(Clone, Default)]
pub(crate) struct PureProducerShareRoster {
    admitted_qualified: Vec<String>,
    refused: Vec<RefusedShareRow>,
}

/// Decode the refused roster. Every arm refuses: a missing declaration, a non-record row, a
/// wrong record type, a non-String field, an unknown verdict variant. An unknown variant is
/// deliberately NOT tolerated — a fourth verdict arrives with a meaning this wall does not know
/// how to weigh, and treating it as "some refusal" would let the authority claim a judgement the
/// executor cannot perform.
/// One row of `floor_cross_claim_prepared_effect_inputs`: an input the floor acquires ONCE at
/// preparation and carries for the prepared subject's lifetime.
#[derive(Clone)]
pub(crate) struct PreparedEffectInputRow {
    acquisition: String,
    checkout_input: String,
}

/// One row of `floor_cross_claim_carried_input_warm_rows`: a producer warmed at preparation
/// whose value depends on a carried input. `ImplicitAcquisition` is the shape whose key must be
/// taught the carried content; `BoundParameter` is the shape whose caller already passes it.
#[derive(Clone)]
pub(crate) struct CarriedInputWarmRowDecoded {
    producer: String,
    carried_input: String,
    bound_parameter: Option<String>,
}

fn floor_decode_prepared_effect_inputs(
    hermetic: &v1_interpreter::InterpContext,
    qualified_name: &str,
) -> Result<Vec<PreparedEffectInputRow>, String> {
    let value = v1_interpreter::run_in_context(hermetic, qualified_name, false)
        .map_err(|e| format!("{qualified_name}: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|why| format!("{qualified_name} decode: {why}"))?;
    let mut out = Vec::new();
    for item in items {
        let v1_interpreter::Value::Record { type_name, fields } = &item else {
            return Err(format!(
                "{qualified_name}: expected PreparedEffectInput rows, got {}",
                floor_value_shape(Some(&item))
            ));
        };
        if !hermetic.sym_eq(*type_name, "PreparedEffectInput") {
            return Err(format!(
                "{qualified_name}: expected PreparedEffectInput, got record {}",
                hermetic.resolve(*type_name)
            ));
        }
        let field_str = |name: &str| -> Result<String, String> {
            match hermetic.field(fields, name) {
                Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                other => Err(format!(
                    "{qualified_name}: PreparedEffectInput.{name} must be a String, got {}",
                    floor_value_shape(other)
                )),
            }
        };
        out.push(PreparedEffectInputRow {
            acquisition: field_str("acquisition")?,
            checkout_input: field_str("checkout_input")?,
        });
    }
    Ok(out)
}

fn floor_decode_carried_input_warm_rows(
    hermetic: &v1_interpreter::InterpContext,
    qualified_name: &str,
) -> Result<Vec<CarriedInputWarmRowDecoded>, String> {
    let value = v1_interpreter::run_in_context(hermetic, qualified_name, false)
        .map_err(|e| format!("{qualified_name}: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|why| format!("{qualified_name} decode: {why}"))?;
    let mut out = Vec::new();
    for item in items {
        let v1_interpreter::Value::Record { type_name, fields } = &item else {
            return Err(format!(
                "{qualified_name}: expected CarriedInputWarmRow rows, got {}",
                floor_value_shape(Some(&item))
            ));
        };
        if !hermetic.sym_eq(*type_name, "CarriedInputWarmRow") {
            return Err(format!(
                "{qualified_name}: expected CarriedInputWarmRow, got record {}",
                hermetic.resolve(*type_name)
            ));
        }
        let field_str = |name: &str| -> Result<String, String> {
            match hermetic.field(fields, name) {
                Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                other => Err(format!(
                    "{qualified_name}: CarriedInputWarmRow.{name} must be a String, got {}",
                    floor_value_shape(other)
                )),
            }
        };
        // THE DEPENDENCE ARM IS READ, NOT ASSUMED. An unknown arm stops the line rather than
        // defaulting to either shape: defaulting to `ImplicitAcquisition` would teach the key a
        // content the producer never read, and defaulting to `BoundParameter` would key a
        // nullary call on the empty row — the stale serve this row kind exists to prevent.
        let bound_parameter = match hermetic.field(fields, "dependence") {
            Some(v1_interpreter::Value::Variant {
                variant_name,
                fields: dep_fields,
                ..
            }) => match hermetic.resolve(*variant_name).as_str() {
                "ImplicitAcquisition" => None,
                "BoundParameter" => match hermetic.field(dep_fields, "parameter") {
                    Some(v1_interpreter::Value::Str(s)) => Some(s.to_string()),
                    other => {
                        return Err(format!(
                            "{qualified_name}: BoundParameter.parameter must be a String, got {}",
                            floor_value_shape(other)
                        ))
                    }
                },
                other => {
                    return Err(format!(
                        "{qualified_name}: unknown CarriedInputDependence arm {other}"
                    ))
                }
            },
            other => {
                return Err(format!(
                    "{qualified_name}: CarriedInputWarmRow.dependence must be a \
                     CarriedInputDependence, got {}",
                    floor_value_shape(other)
                ))
            }
        };
        out.push(CarriedInputWarmRowDecoded {
            producer: field_str("producer")?,
            carried_input: field_str("carried_input")?,
            bound_parameter,
        });
    }
    Ok(out)
}

fn floor_decode_refused_share_candidates(
    hermetic: &v1_interpreter::InterpContext,
    qualified_name: &str,
) -> Result<Vec<RefusedShareRow>, String> {
    let value = v1_interpreter::run_in_context(hermetic, qualified_name, false)
        .map_err(|e| format!("{qualified_name}: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|why| format!("{qualified_name} decode: {why}"))?;
    let mut out = Vec::new();
    for item in items {
        let v1_interpreter::Value::Record { type_name, fields } = &item else {
            return Err(format!(
                "{qualified_name}: expected RefusedShareCandidate rows, got {}",
                floor_value_shape(Some(&item))
            ));
        };
        if !hermetic.sym_eq(*type_name, "RefusedShareCandidate") {
            return Err(format!(
                "{qualified_name}: expected RefusedShareCandidate, got record {}",
                hermetic.resolve(*type_name)
            ));
        }
        let producer = match hermetic.field(fields, "producer") {
            Some(v1_interpreter::Value::Str(s)) => s.to_string(),
            other => {
                return Err(format!(
                    "{qualified_name}: producer must be String, got {}",
                    floor_value_shape(other)
                ))
            }
        };
        let verdict = match hermetic.field(fields, "verdict") {
            Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
                let name = hermetic.resolve(*variant_name);
                match name.as_str() {
                    "MeasuredServeAboveRecompute"
                    | "NoMeasuredEffectOverItsConsumers"
                    | "SupersededBySingleAuthorityRepair"
                    | "KeyOmitsAnInputTheValueDependsOn" => name,
                    other => {
                        return Err(format!(
                            "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRefusalVerdictUnknown \
                             producer={producer} verdict={other} — the refused roster grew a \
                             verdict arm this executor cannot weigh; the arm and the handling \
                             land together or the wall is claiming a judgement it cannot make"
                        ));
                    }
                }
            }
            other => {
                return Err(format!(
                    "{qualified_name}: verdict must be a ShareRefusalVerdict variant, got {}",
                    floor_value_shape(other)
                ))
            }
        };
        let carriers = floor_decode_list(hermetic, hermetic.field(fields, "carrier_modules"))
            .map_err(|why| format!("{qualified_name}: {producer} carrier_modules: {why}"))?;
        let mut carrier_modules = std::collections::BTreeSet::new();
        for carrier in carriers {
            match carrier {
                v1_interpreter::Value::Str(s) => {
                    carrier_modules.insert(s.to_string());
                }
                other => {
                    return Err(format!(
                        "{qualified_name}: {producer} carrier_modules must be String rows, got {}",
                        floor_value_shape(Some(&other))
                    ))
                }
            }
        }
        // A REFUSED ROW WITH NO CARRIERS WOULD BE PERMANENTLY UNJOINABLE. It would sit in the
        // roster reading as covered while intersecting nothing, which is the vacuously-green
        // shape DESIGN §4b warns about, so it stops the line at decode rather than at nothing.
        if carrier_modules.is_empty() {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRefusedRowCarrierless \
                 producer={producer} — a refused row with an empty carrier_modules set can never \
                 join against an observed fill, so it would be enrolled coverage that cannot fire"
            ));
        }
        out.push(RefusedShareRow {
            producer,
            verdict,
            carrier_modules,
        });
    }
    Ok(out)
}

pub(crate) fn floor_decode_module_prefix_roster(
    hermetic: &v1_interpreter::InterpContext,
    qualified_name: &str,
) -> Result<Vec<String>, String> {
    let value = v1_interpreter::run_in_context(hermetic, qualified_name, false)
        .map_err(|e| format!("{qualified_name}: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|why| format!("{qualified_name} decode: {why}"))?;
    let mut out = Vec::new();
    for item in items {
        match item {
            v1_interpreter::Value::Str(s) => out.push(s.to_string()),
            other => {
                return Err(format!(
                    "{qualified_name}: expected String rows, got {}",
                    floor_value_shape(Some(other))
                ))
            }
        }
    }
    Ok(out)
}

/// THE ENVELOPE THE PROCESS ACTUALLY HAS, read from the kernel, at every visible level.
///
/// Every envelope figure this lane has reasoned with — 13.00 GiB high, 14.00 GiB max — came
/// from the DECLARED rows in `gunbc.runner.runner_slot_allocation`: a nominal limit, read before the
/// run, describing what was requested rather than what binds. Two measurements say that is not
/// good enough.
///
/// The first is a live contradiction in the timed-out CI run: it entered admission-decode at
/// rss_kb=15402396 (14.69 GiB), ABOVE both the declared high and the declared max, and was not
/// killed — it ran 151 more minutes. Either those rows are not what binds in that slot, or
/// process RSS overstates the cgroup charge enough to break the comparison, or both. Nobody
/// has measured which.
///
/// The second is that reading the nominal limit is a mistake with a receipt. A local run was
/// launched on the premise that this container's `memory.max` of 31.27 GiB and `memory.high`
/// of `max` made it a headroom arm; it was OOM-killed at 12.77 GB RSS. The counters afterwards
/// read `max 0`, `high 0`, `oom_kill 16` — its own limits were never reached, so the kill came
/// from an ancestor. The nominal limit described nothing that mattered.
///
/// Hence: the PATH as well as the values, and every readable ancestor, because a cgroup that
/// never hits its own maximum can still be killed from above and the honest reading requires
/// knowing where the ceiling lives. `memory.events` is the load-bearing field — `max`/`high`
/// nonzero means this level acted; all-zero beside a kill means some other level did.
///
/// Emitted at entry and again at exit so the peak and the event counters bound the whole run.
/// This reads `/proc` and `/sys` and writes nothing; an unreadable file is reported as absent
/// rather than defaulted, since a fabricated zero here would re-create the exact class of
/// error the function exists to end.
pub(crate) fn floor_cgroup_envelope(when: &str) {
    // One resolver for both readers — see `floor_cgroup_dir`. This function computing the path
    // itself while the sampler computed a different one is the defect that made every per-beat
    // cgroup field `na` for a four-hour run.
    let leaf = floor_cgroup_dir();
    eprintln!("[floor-cgroup] when={when} path={leaf}");
    let field = |dir: &str, name: &str| -> String {
        std::fs::read_to_string(format!("{dir}/{name}"))
            .map(|v| v.split_whitespace().collect::<Vec<_>>().join(","))
            .unwrap_or_else(|_| "<absent>".to_string())
    };
    // Walk from the leaf up to the cgroup root, reporting each level that is readable. Under a
    // container this is often just the root, and that itself is the finding: if no ancestor is
    // visible from inside, the process cannot observe the limit that binds it and the ceiling
    // has to be read from the host side.
    //
    // The walk is not decoration. On the runner the leaf recorded ZERO high events while its
    // parent slices recorded 141M against unlimited maxima, so the level doing the throttling
    // and the level holding the limit are different levels, and reading only our own would have
    // reported a process comfortably inside its envelope while it took 16M major faults.
    //
    // BOTH `memory.events` AND `memory.events.local`, because the first is HIERARCHICAL — an
    // ancestor's counters include every event generated anywhere beneath it — and only the
    // second reports events at that exact cgroup. Reading the hierarchical file alone is how a
    // parent slice's 141M-versus-476K was read as a difference in neighbour pressure when it is
    // an aggregate over each subtree's whole history: both parents carry `high=max` and so
    // cannot be throttled themselves, meaning every one of those events belongs to some
    // descendant. Local and hierarchical are the split that makes the question decidable at
    // all — a rising ancestor-hierarchical count beside a flat leaf-local count is the
    // signature of OTHER descendants being throttled, and without the pair a process cannot
    // tell its own reclaim from its neighbours'.
    //
    // `memory.pressure` (PSI) comes along because the events counters are cumulative totals
    // rather than rates: two snapshots of a monotone counter give an interval delta, but stall
    // time is what says whether that reclaim actually cost this process anything.
    let mut dir = std::path::PathBuf::from(&leaf);
    loop {
        let d = dir.to_string_lossy().to_string();
        if std::fs::metadata(format!("{d}/memory.current")).is_ok() {
            eprintln!(
                "[floor-cgroup] when={when} level={d} max={} high={} current={} peak={} \
                 events=[{}] events_local=[{}] pressure=[{}]",
                field(&d, "memory.max"),
                field(&d, "memory.high"),
                field(&d, "memory.current"),
                field(&d, "memory.peak"),
                field(&d, "memory.events"),
                field(&d, "memory.events.local"),
                field(&d, "memory.pressure"),
            );
        }
        if d == "/sys/fs/cgroup" || !dir.pop() {
            break;
        }
    }
}

/// THE LEAF'S memory.stat COUNTERS, RAW, ON EVERY BEAT, BECAUSE memory.peak IS NOT A DEMAND.
///
/// `memory.current` and `memory.peak` charge page cache alongside anonymous memory, and a run
/// that is being reclaimed down to its `memory.high` line finishes clean precisely because the
/// cache half of that charge is reclaimable. So a peak read at the throttle line is a CEILING
/// that includes cache, and sizing anything to it -- a cell row, a microVM guest whose kernel
/// can reclaim its own cache just as well -- treats a bet as a fact (DESIGN 4d). Demand is
/// therefore derived from `memory.stat`'s counters, and this reader prints those counters AS THE
/// KERNEL REPORTS THEM: one flat list, no grouping, no sum. `memory.stat` is not a partition --
/// `file` includes shmem, and `unevictable` is an LRU-list state over pages `anon` and `file`
/// already count -- so any grouping here would already be a derivation, and the derivation has
/// one home: `gunbc.floor.floor_demand` `beat_held_set`, which folds disjoint terms into a resident
/// held-set lower bound. `memory.current` is printed beside the counters so a derivation can be
/// checked against the charge it is drawn from.
///
/// Sampled every beat rather than every tenth, because `memory.stat` has no `.peak` and the
/// maximum over samples is the only bound available; a ten-minute cadence would undercount a
/// short anonymous spike by whatever it missed. One file read per beat, leaf only -- the
/// ancestors' counters cover their whole subtrees and say nothing about this run.
///
/// Every field is printed as read or as `na`; a missing key is never rendered as zero, for the
/// reason `floor_resource_sample` gives.
pub(crate) fn floor_cgroup_stat_beat(
    when: &str,
    stall: Option<&crate::memory_governor::MemoryStallObservation>,
) {
    let leaf = floor_cgroup_dir();
    // THE SEAM IS READ BEFORE AND AFTER THE SAMPLE, AND A TRANSITION IS NAMED RATHER THAN PICKED.
    //
    // Putting the seam on the beat's own line removed the LOG-ADJACENCY join; it did not bind the
    // two observations IN TIME, and those are different fixes. This function reads memory.stat and
    // memory.current from procfs and then renders; the floor thread can call `floor_seam` at any
    // point in between, so a single read taken after the sample would label a beat with a seam the
    // sampled memory was not taken under -- confidently, and with nothing in the line to say so.
    // A transition beat is exactly the one a reader most wants to trust, because it is where a
    // phase's cost is attributed.
    //
    // Bracketing is preferred over holding the seam lock across the reads: the lock is written by
    // the floor thread on every phase change, and blocking that thread on two procfs reads would
    // let the instrument perturb the workload it is measuring. Bracketing cannot do that, and it
    // makes the uncertainty EXPLICIT (`transition:A>B`) instead of resolving it silently -- which
    // is the difference between a typed refusal and a fabricated plausible reading (DESIGN 5).
    let seam_before = floor_seam_current();
    let body = std::fs::read_to_string(format!("{leaf}/memory.stat")).ok();
    let key = |k: &str| -> String {
        body.as_deref()
            .and_then(|b| {
                b.lines().find_map(|l| {
                    l.strip_prefix(k)
                        .and_then(|r| r.strip_prefix(' '))
                        .map(|v| v.trim().to_string())
                })
            })
            .unwrap_or_else(|| "na".to_string())
    };
    let raw = |name: &str| -> String {
        std::fs::read_to_string(format!("{leaf}/{name}"))
            .map(|v| v.split_whitespace().collect::<Vec<_>>().join(","))
            .unwrap_or_else(|_| "na".to_string())
    };
    let current = raw("memory.current");
    // THE WALL CLOCK, so a beat is placed in time by its own line rather than by the log
    // prefix the job runner happens to add, and a seam beat and a watchdog beat order by value.
    let unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "na".to_string());
    let seam_after = floor_seam_current();
    // The beat line's own spelling, one token each and no spaces, because this line is parsed
    // field-by-field into a typed receipt. `none` is the unset slot and maps to
    // `SeamNotYetEntered`; `unreadable` is a poisoned lock; `transition:A>B` is a seam that MOVED
    // while this sample was being taken, and maps to a `FloorSeam` arm that can neither size nor
    // attribute anything. Only a seam that was the SAME before and after the sample is reported
    // as that seam.
    let spell = |r: &Option<String>| -> String {
        match r {
            None => "unreadable".to_string(),
            Some(s) if s.is_empty() => "none".to_string(),
            Some(s) => s.clone(),
        }
    };
    let before = spell(&seam_before);
    let after = spell(&seam_after);
    let seam = if before == after {
        before
    } else {
        format!("transition:{before}>{after}")
    };
    // THE STALL CLAUSE ON THE BEAT'S OWN LINE, for the same reason as the seam. It is the
    // heartbeat's quantity and used to be readable only from the heartbeat line printed BESIDE
    // this one, so `gunbc.floor.floor_demand` `FloorMemoryStatBeat.stall` was transcribed by adjacency
    // -- and `receipt_last_unstalled_beat` derives the guest's cache allowance from that field,
    // so a mis-joined stall mis-sizes a guest. `na` where the window could not be read, never a
    // zero: a zero stall is the healthiest reading this line can carry, so fabricating one would
    // manufacture progress, which is the inverse of the error the counters exist to catch.
    let stall_text = match stall {
        Some(o) => crate::memory_governor::memory_stall_major_faults_per_minute(o).to_string(),
        None => "na".to_string(),
    };
    // SWAP, PSI AND THE LEAF'S OWN EVENTS BESIDE THE COUNTERS, on every beat. The 2026-09-19
    // receipt is censored on the held-set axis because anonymous pages were going to swap one beat
    // after its peak, and the swap figure that showed it was the HOST's swap-in, read beside the
    // leaf -- a different subject. `memory.swap.current` is this leaf's own; `memory.pressure` is
    // the stall time that says whether reclaim cost the run anything; `memory.events.local`
    // carries `oom` and `oom_kill` for this leaf alone. Printed raw, as the counters are.
    //
    // AND THE PROCESSES CHARGED TO THE LEAF, because a leaf charge is a sum over every process in
    // it: a floor that shares its slot with a build daemon or a previous step's straggler reads
    // their pages as its own, and that overlap is a different cause with a different remedy than
    // retained application state. `pid:comm:rss_kb` per member, read from procfs, nothing summed.
    let procs = std::fs::read_to_string(format!("{leaf}/cgroup.procs"))
        .map(|body| {
            body.lines()
                .filter_map(|pid| {
                    let pid = pid.trim();
                    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
                    let field = |k: &str| {
                        status.lines().find_map(|l| {
                            l.strip_prefix(k).map(|r| {
                                r.trim_start_matches(':')
                                    .trim()
                                    .trim_end_matches(" kB")
                                    .to_string()
                            })
                        })
                    };
                    Some(format!(
                        "{pid}:{}:{}",
                        field("Name").unwrap_or_else(|| "na".to_string()),
                        field("VmRSS").unwrap_or_else(|| "na".to_string())
                    ))
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_else(|_| "na".to_string());
    eprintln!(
        "[floor-cgroup] when={when} seam={seam} unix_ms={unix_ms} swap_current={} \
         events_local=[{}] pressure=[{}] procs=[{procs}]",
        raw("memory.swap.current"),
        raw("memory.events.local"),
        raw("memory.pressure"),
    );
    eprintln!(
        "[floor-cgroup] when={when} stat_level={leaf} seam={seam} stall_per_min={stall_text} current={current} \
         memory_stat=[anon,{},file,{},shmem,{},unevictable,{},slab_unreclaimable,{},\
         slab_reclaimable,{},kernel_stack,{},pagetables,{},percpu,{},sock,{},file_dirty,{}]",
        key("anon"),
        key("file"),
        key("shmem"),
        key("unevictable"),
        key("slab_unreclaimable"),
        key("slab_reclaimable"),
        key("kernel_stack"),
        key("pagetables"),
        key("percpu"),
        key("sock"),
        key("file_dirty"),
    );
}

pub(crate) fn required_floor_disposition_label(
    disposition: &RequiredFloorDisposition,
) -> &'static str {
    match disposition {
        RequiredFloorDisposition::Planned => "planned",
        RequiredFloorDisposition::PlannedAsChangedWitness => "planned_as_changed_witness",
        RequiredFloorDisposition::PlannedAsReachConsumer => "planned_as_reach_consumer",
        RequiredFloorDisposition::DeclinedLongModule { .. } => "declined_long_module",
        RequiredFloorDisposition::DeclinedFixtureMember { .. } => "declined_fixture_member",
        RequiredFloorDisposition::DeclinedOutsideRequiredGate => "declined_outside_required_gate",
        RequiredFloorDisposition::DeclinedCostDebt => "declined_cost_debt",
        RequiredFloorDisposition::DeclinedOutsideGateClosure => "declined_outside_gate_closure",
        RequiredFloorDisposition::DeclinedDiscoveryExcluded { .. } => "declined_discovery_excluded",
        RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery { .. } => {
            "declined_changed_witness_outside_discovery"
        }
        RequiredFloorDisposition::DeclinedNoCiWetLane { .. } => "declined_no_ci_wet_lane",
    }
}

pub(crate) fn required_floor_disposition_matched_prefix(
    disposition: &RequiredFloorDisposition,
) -> &str {
    match disposition {
        RequiredFloorDisposition::DeclinedLongModule { matched_prefix }
        | RequiredFloorDisposition::DeclinedFixtureMember { matched_prefix } => matched_prefix,
        // The excluded arm's payload is a SUBSTRING and not a module-name prefix, and it shares
        // this column because the column's meaning is "the authored text that matched", which is
        // the same question for all three. It is not folded into the prefix arms above: a
        // substring matched anywhere in a path is a different test from a prefix on a module
        // name, and the label column keeps them apart for any reader joining on it.
        RequiredFloorDisposition::DeclinedDiscoveryExcluded { matched_substring } => {
            matched_substring
        }
        // The module that declares the identity, which is the whole content of this row: the
        // reader needs to know WHERE the undeclarable selection is homed to see that it is
        // outside the run's roots.
        RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery { module_path } => {
            module_path
        }
        // The BinWitnessWet exclusion pattern that matched the file: the authored text that
        // placed this identity in the class no CI lane executes.
        RequiredFloorDisposition::DeclinedNoCiWetLane { pattern } => pattern,
        RequiredFloorDisposition::Planned
        | RequiredFloorDisposition::PlannedAsChangedWitness
        | RequiredFloorDisposition::PlannedAsReachConsumer
        | RequiredFloorDisposition::DeclinedOutsideRequiredGate
        | RequiredFloorDisposition::DeclinedOutsideGateClosure
        | RequiredFloorDisposition::DeclinedCostDebt => "",
    }
}

/// THE HEARTBEAT'S WINDOW OVER THE REFUSAL'S OWN METRIC.
///
/// The heartbeat printed `cpu_ms` as utime PLUS stime and no share at all, so every green run
/// carried no figure comparable to the one the stall refusal decides on, and the sub-threshold
/// population was unmeasurable at the grain the verdict is taken at. Reconstructing it from the
/// printed fields does not recover it: a census over 60 recent floor runs scored a REFUSING run
/// at a 31.6% compute share and a GREEN one at 19.7%, ranking the refusal as the healthier of the
/// two, because `utime + stime` reads a pure treadmill as roughly one-third computing -- exactly
/// what `MEMORY_STALL_PROGRESS_CPU_SHARE_FLOOR_BASIS_POINTS` says it does, with its own receipt.
///
/// So this is a SECOND READ OF ONE PRODUCER, never a second computation. The window is the
/// heartbeat's rather than the stall check's -- a different window over the same quantity -- and
/// the two figures come from `memory_stall_major_faults_per_minute` and
/// `memory_stall_self_cpu_share_basis_points`, the same mirrors `memory_stall_verdict` consumes.
/// A copy of either formula here would drift from the refusal the first time either was touched,
/// which is the defect this repair removes rather than one it must avoid introducing.
///
/// WHAT THIS METRIC IS SILENT ABOUT, stated because the opposite was proposed and measured false:
/// it reports the STALL condition and says nothing about the `cpu_deadline` population. Over the
/// same 60-run census, the 16 runs carrying an `interrupted_cpu_deadline` were indistinguishable
/// from the other 44 on both axes (median peak 341,656/min against 356,630; median minimum share
/// 0.537 against 0.504). That is a DATA result at n=16, not a claim about what a page fault does.
pub(crate) struct FloorStallWindow {
    pub started: std::time::Instant,
    pub major_faults: u64,
    pub self_user_cpu_ms: u64,
}

/// THE STALL WINDOW AS THE REFUSAL AUTHORITY'S OWN OBSERVATION, or `None` when either counter did
/// not read. `None` renders the sentinel rather than a fabricated zero, on the same all-or-nothing
/// rule as the cpu pair above: a share computed from half a reading is not a share, and a zero
/// share is the most severe reading the line can carry, so fabricating one manufactures a stall.
/// A zero-wall window is likewise `None`: a rate over no wall is not a rate. The rendering is
/// `render_heartbeat_line_mirror`, which projects this through `memory_governor`'s stall mirrors
/// -- the same two functions `memory_stall_verdict` consumes -- so the printed rate and share
/// cannot drift from the refusal's.
pub(crate) fn floor_stall_window_observation(
    window_wall_ms: u64,
    major_faults_in_window: Option<u64>,
    self_user_cpu_ms_in_window: Option<u64>,
) -> Option<crate::memory_governor::MemoryStallObservation> {
    let (Some(faults), Some(user_cpu_ms)) = (major_faults_in_window, self_user_cpu_ms_in_window)
    else {
        return None;
    };
    if window_wall_ms == 0 {
        return None;
    }
    Some(crate::memory_governor::MemoryStallObservation {
        window_wall_ms,
        major_faults_in_window: faults,
        self_user_cpu_ms_in_window: user_cpu_ms,
        // The cache counters are the refusal's diagnostic detail, not inputs to either figure
        // this line carries; zero here is a true statement about what this window measured, not
        // a stand-in for an unread counter.
        cache_evictions_in_window: 0,
        cache_readmissions_in_window: 0,
    })
}

thread_local! {
    /// Set by `install_pure_producer_share` and read once the fold is over. Thread-local for the
    /// same reason the shared-fill ledger is: it is the SAME thread's observation, and a state
    /// that could be written by one thread and read by another would let the wall adjudicate a
    /// roster that never governed the fills it is reading.
    static PURE_PRODUCER_SHARE_ROSTER: std::cell::RefCell<Option<PureProducerShareRoster>> =
        const { std::cell::RefCell::new(None) };
}
