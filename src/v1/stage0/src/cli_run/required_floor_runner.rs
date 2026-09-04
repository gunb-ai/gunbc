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

/// Clone of the leg's cost snapshot for `claim_executor`'s clamp + drift consumers.
/// `None` until `install_floor_compile_clean_receipt` ran a Compiled leg.
pub fn floor_compile_clean_cost_snapshot() -> Option<(u128, u128, Vec<(String, u64)>, String)> {
    FLOOR_COMPILE_CLEAN_COST.lock().ok().and_then(|guard| {
        guard.as_ref().map(|c| {
            (
                c.wall_ms,
                c.closure_units,
                c.module_typecheck_walls.clone(),
                c.pass_subject.to_string(),
            )
        })
    })
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
    // resolution refusal must not mask SourceAnnotationRefused in the rest of
    // the indexed pool. Compiled-file admission is the via-index parse itself
    // (`tokenize_artifact` + `admit_source_annotations`).
    let census_fill_diags = compile_clean_census_fill_hard_diagnostics(&census_only);
    let (graph, si, compile_clean_diags) = match resolved_graph_from_sources_with_index(
        &index,
        sources,
        ResolveTypecheckGate::Strict,
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

pub(crate) fn floor_route_gap_expectation_mismatch(
    expectation: Option<&FloorRouteGapExpectation>,
    operation: &str,
    ground: &v1_interpreter::HermeticEffectGround,
) -> Option<String> {
    let expectation = expectation?;
    let observed_ground = match ground {
        v1_interpreter::HermeticEffectGround::UnpublishedMockCase { .. } => {
            FloorRouteGapExpectedGround::UnpublishedMockCase
        }
        v1_interpreter::HermeticEffectGround::NoMockResponse => {
            FloorRouteGapExpectedGround::NoMockResponse
        }
        v1_interpreter::HermeticEffectGround::FilesystemRemoval => {
            FloorRouteGapExpectedGround::FilesystemRemoval
        }
    };
    if expectation.operation == operation && expectation.ground == observed_ground {
        None
    } else {
        Some(format!(
            "typed route-gap enrollment expected operation={} ground={:?}, observed operation={} ground={:?}",
            expectation.operation, expectation.ground, operation, observed_ground
        ))
    }
}

/// `v2.workflow.required_floor`'s claims execute Hermetic (pure in-process evaluation), so
/// CPU is the judged basis. A lane that later admits an execution mode whose purpose is
/// external or blocking interaction picks wall instead — but the choice is made here, by
/// declaring the lane's purpose, not derived from a measurement.
pub fn required_floor_cost_basis() -> RequiredFloorCostBasis {
    RequiredFloorCostBasis::CpuCost
}

pub fn floor_walk_attempt_id_from_env() -> String {
    std::env::var("GUNBC_FLOOR_WALK_ATTEMPT_ID")
        .or_else(|_| std::env::var("GUNBC_WALK_ATTEMPT_ID"))
        .unwrap_or_else(|_| "local".to_string())
}

pub(crate) fn floor_drain_retention_detail_enabled() -> bool {
    std::env::var("GUNBC_FLOOR_DRAIN_RETENTION")
        .ok()
        .as_deref()
        .map(|v| matches!(v, "1" | "true" | "TRUE"))
        .unwrap_or(false)
}

pub fn make_eval_context(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_fixture_store(graph, source_indices, execution_mode, None)
}

pub fn make_eval_context_with_fixture_store(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        None,
    )
}

pub fn make_eval_context_with_runtime_options(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
) -> v1_interpreter::InterpContext {
    v1_interpreter::InterpContext::with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        whole_tree_published_keys,
    )
}

/// Run a witness companion that returns `String` divergence detail (Lane B agreement loudness).
/// Empty string = no divergence detail (clean companion **or** companion not declared).
/// Non-empty refusal sentinel on wrong type / non-missing interpreter error — never silent
/// None when a companion *is* declared (review 41847, §5). A missing companion
/// (`NoSuchFunction` from the `_holds` → `_failure_receipt` naming convention) is
/// "not declared", not a refused receipt — the auto-derived name must not invent a
/// required loudness hook for every Bool(false) witness. This arm used to name a second
/// variant, `NoMainFunction`, because `run_in_context` reported EVERY missing named
/// function that way; that variant is deleted and the one honest arm now covers it.
pub fn run_claim_failure_receipt(ctx: &v1_interpreter::InterpContext, function: &str) -> String {
    match v1_interpreter::run_in_context(ctx, function, false) {
        Ok(Value::Str(s)) => s.to_string(),
        Ok(other) => format!(
            "failure_receipt_refused: {function} returned {}, expected String",
            ctx.format_value(&other)
        ),
        Err(v1_interpreter::InterpError::NoSuchFunction { .. }) => String::new(),
        Err(e) => format!("failure_receipt_refused: {function}: {e}"),
    }
}

/// THE UNWIND BOUNDARY. `catch_unwind` sits here, around the evaluation and nothing else, because
/// this is the seam that already turns "what the evaluator did" into "which terminal the claim
/// reached" — every other non-verdict is classified on the lines below, and a panic classified
/// anywhere else would be a second authority for the same question.
///
/// IT DOES NOT LICENSE CONTINUING. Capturing the payload lets the runner name the identity and
/// publish its ledger; the fold above still stops (operator ruling, 2026-08-26). Every other
/// non-verdict here is a state the producer DECIDED to return, so the process's invariants held
/// and the next claim starts from a known state; a panic is an invariant violated at an unknown
/// place, and a later row measured after it would carry an unstated precondition that no row can
/// express. Continuing would manufacture exactly the execution-provenance conflation this floor
/// exists to prevent — a green row after an unwind and a green row before it rendering alike.
///
/// The default hook still prints the panic and its location to stderr before this returns, so the
/// unwind stays as loud as it was; what changes is that it now also becomes a value.
pub(crate) fn run_claim_evaluation(
    ctx: &v1_interpreter::InterpContext,
    function: &str,
) -> Result<v1_interpreter::InterpResult<v1_interpreter::Value>, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        v1_interpreter::run_in_context(ctx, function, false)
    }))
    .map_err(|payload| panic_payload_text(&payload))
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
    let qualified = "gunbc.v1_interpreter_opaque_host_call.opaque_host_call_surface";
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
             gunbc.v1_interpreter_opaque_host_call: unresolved={:?} not_free_call={:?}",
            floor_value_shape(ctx.field(fields, "unresolved")),
            floor_value_shape(ctx.field(fields, "not_free_call")),
        )),
        other => Err(format!(
            "{qualified}: unknown OpaqueHostCallSurface arm {other:?}"
        )),
    }
}

/// The published spelling of a claim's observed `ClaimPreemptionReachability`.
///
/// THE OPERATIONS ARE CARRIED, NOT JUST THE ARM. A reader asking "why could the deadline not
/// fire here" needs the operation name to act, and re-deriving it from the claim body is the
/// positional citation DESIGN §3 forbids. They are joined with `+` because a claim may reach
/// more than one opaque arm and reporting only the first would under-state the surface a repair
/// has to cover.
pub(crate) fn preemption_reachability_label(reach: &v1_interpreter::OpaqueHostCallReach) -> String {
    match reach {
        v1_interpreter::OpaqueHostCallReach::SurfaceUnarmed => "surface_unarmed".to_string(),
        v1_interpreter::OpaqueHostCallReach::CooperativelyPollable => {
            "cooperatively_pollable".to_string()
        }
        v1_interpreter::OpaqueHostCallReach::OpaqueHostCallUnbounded { operations } => {
            format!("opaque_host_call_unbounded:{}", operations.join("+"))
        }
    }
}

pub fn run_claim_measured(
    ctx: &v1_interpreter::InterpContext,
    closure_subject_digest: &str,
    function: &str,
) -> (ClaimOutcome, v1_interpreter::PerformanceReceipt) {
    let subject_key =
        crate::resolved_graph_cache::witness_work_subject_key(closure_subject_digest, function);
    v1_interpreter::eval_profile_reset();
    v1_interpreter::eval_subject_set(subject_key.clone());
    // PER-CLAIM, NOT PER-RUN: reach is a fact about THIS claim's evaluation, so it is cleared
    // here beside the profile and the subject rather than at floor start. The armed surface
    // survives the reset -- disarming per claim would make every claim report `SurfaceUnarmed`.
    v1_interpreter::reset_opaque_host_call_reach();
    if let Some(budget_ms) = ctx.witness_eval_budget() {
        ctx.arm_eval_deadline(budget_ms);
    }
    if let Some(budget_ms) = ctx.witness_wall_budget() {
        // Kill-at-deadline: shell waits poll this and SIGKILL at the ceiling.
        // Completion-side `wall_budget_completion_outcome` stays as the backstop.
        ctx.arm_wall_deadline(budget_ms);
    }
    let started = std::time::Instant::now();
    let cpu_started_nanos = v1_interpreter::thread_cpu_nanos();
    let steps_started = v1_interpreter::evaluator_steps();
    let fill_steps_before = v1_interpreter::shared_artifact_fill_eval_steps();
    let fill_before_nanos = shared_artifact_fill_cpu_nanos();
    let fill_wall_before_nanos = shared_artifact_fill_wall_nanos();
    let outcome = run_claim(ctx, function);
    // CPU consumed by THIS (witness-eval) thread — the budget metric, so the completion-side
    // check matches the cooperative stride-poll and neither fires on cold-I/O or contention
    // wall time.
    let measured_cpu_nanos = v1_interpreter::thread_cpu_nanos().saturating_sub(cpu_started_nanos);
    // Sampled here rather than after the report so the wall clock can be split by the same rule
    // the CPU clock is, and so the reported line and the enforced quantity read one binding.
    let measured_wall_nanos = started.elapsed().as_nanos();
    // SHARED-ARTIFACT FILL IS NOT THIS CLAIM'S MARGINAL COST (operator-line ruling, 2026-08-27).
    // Whatever this claim spent filling a memo is consumed by every later claim naming the same
    // source — one of them measured at literally 0ms in the same run because this one paid — so
    // charging it here makes a merge-blocking ceiling a function of EXECUTION ORDER rather than of
    // the tree. The floor already bills its three `[floor-phase]` warm builds to preparation with
    // `provenance=built-by-preparation` for exactly this reason; this is that rule at memo grain.
    //
    // THE COST IS SPLIT, NEVER DROPPED: `fill_cpu_nanos` is reported per claim and the two halves
    // sum to `measured_cpu_nanos`. A runaway compile cannot hide behind "it was a miss", because
    // the fill is still counted, still attributed and still visible in the receipt.
    let fill_cpu_nanos = shared_artifact_fill_cpu_nanos().saturating_sub(fill_before_nanos);
    let cpu_nanos = measured_cpu_nanos.saturating_sub(fill_cpu_nanos);
    // THE SAME SPLIT ON THE WALL CLOCK. `wall_budget_completion_outcome` below is a
    // merge-blocking ceiling, so charging it the fill made it a function of execution order in
    // exactly the way the ruling above forbids — and unlike the CPU side it had no exemption
    // argument, only an omission. Split, never dropped: both halves are reported and they sum to
    // `measured_wall_nanos`.
    let fill_wall_nanos = shared_artifact_fill_wall_nanos().saturating_sub(fill_wall_before_nanos);
    let wall_nanos = marginal_wall_nanos(measured_wall_nanos, fill_wall_nanos);
    // THE WORK MEASURE, SPLIT BY THE SAME RULE AS THE TWO CLOCKS. Evaluator steps are counted
    // unconditionally by `eval_expr`, so this delta is a property of what the claim evaluated
    // and carries no term for the machine it evaluated on. Netting the stored fills is what
    // keeps that true across execution ORDER as well: without it the claim that happened to
    // fill a shared memo would carry the fill's steps and every later reader would carry none.
    let measured_eval_steps = v1_interpreter::evaluator_steps().wrapping_sub(steps_started);
    let fill_eval_steps =
        v1_interpreter::shared_artifact_fill_eval_steps().wrapping_sub(fill_steps_before);
    let eval_steps = measured_eval_steps.saturating_sub(fill_eval_steps);
    // EITHER clock, not the CPU one. A fill that blocked on I/O can spend wall time while
    // charging almost no CPU, and under a `fill_cpu_nanos > 0` guard that fill would be
    // subtracted from the enforced wall figure and reported nowhere — a cost dropped rather
    // than split, which is the one thing the ruling above forbids.
    if fill_cpu_nanos > 0 || fill_wall_nanos > 0 {
        // REPORTED, NOT ABSORBED. Printed on its own line, per claim, whenever a fill happened,
        // so the quantity the ceiling stops charging is visible at the same grain it was measured
        // — the difference between attributing a cost and losing one. `triggered_by` is this
        // claim, which is precisely what `SharedBuildProvenance::AlreadyWarmOnEntry` records for
        // the larger shared builds: every later claim reading this artifact reads it warm, and
        // this line names who paid.
        eprintln!(
            "[floor-shared-fill] claim={function} marginal_cpu_ms={} fill_cpu_ms={} \
             measured_cpu_ms={} marginal_wall_ms={} fill_wall_ms={} measured_wall_ms={} \
             marginal_eval_steps={} fill_eval_steps={} measured_eval_steps={} \
             provenance=filled-shared-artifact triggered_by={function}",
            cpu_nanos / 1_000_000,
            fill_cpu_nanos / 1_000_000,
            measured_cpu_nanos / 1_000_000,
            wall_nanos / 1_000_000,
            fill_wall_nanos / 1_000_000,
            measured_wall_nanos / 1_000_000,
            eval_steps,
            fill_eval_steps,
            measured_eval_steps,
        );
    }
    ctx.clear_eval_deadline();
    ctx.clear_wall_deadline();
    v1_interpreter::eval_subject_clear();
    let outcome = budget_completion_outcome(ctx.witness_eval_budget(), outcome, cpu_nanos);
    let outcome = wall_budget_completion_outcome(ctx.witness_wall_budget(), outcome, wall_nanos);
    // The receipt records BOTH clocks, and records the same `cpu_nanos` value that
    // `budget_completion_outcome` just enforced against — one binding feeding both, so the
    // enforced and the recorded quantity cannot drift apart. Previously `cpu_nanos` died on
    // the line above and only wall reached the receipt, which is why the cap enforced a
    // quantity no artifact carried.
    let receipt = v1_interpreter::performance_receipt_from_witness(
        subject_key,
        function,
        wall_nanos,
        cpu_nanos,
        eval_steps,
    );
    (outcome, receipt)
}

pub fn floor_discovery_path_excluded(path: &str) -> bool {
    matching_discovery_exclusion_substring(path).is_some()
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

pub(crate) fn floor_diff_edits_from_diff_text(
    index: &MultiEntryIndex,
    diff_text: &str,
) -> Result<FloorDiffEdits, String> {
    let line_ranges = parse_unified_diff_line_ranges(diff_text);
    let changed = parse_unified_diff_changed_new_lines(diff_text);
    let departed = parse_unified_diff_departed_paths(diff_text);
    let added = parse_unified_diff_added_paths(diff_text);
    floor_diff_edits_from_line_ranges(index, &line_ranges, &changed, &departed, &added)
}

// Host realization under a declared scaffold: the governing row is the `SCAFFOLD (DESIGN
// §6–§7)` declaration above `FloorDiffEdits` in `cli_run`, which owns this function's
// reason, dissolve-on trigger and census. Read it there; it is not restated here.
pub(crate) fn floor_diff_edits_from_line_ranges(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
    changed_new_lines_by_file: &HashMap<String, HashSet<i64>>,
    departed_paths: &HashSet<String>,
    added_paths: &HashSet<String>,
) -> Result<FloorDiffEdits, String> {
    let mut overlapping_data_items = HashSet::new();
    let mut edited_test_fns = HashSet::new();
    let mut touched_entry_files = HashSet::new();
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
        let disk_path = process_workspace_root().join(&file_norm);
        if !disk_path.is_file() {
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
        }
        let resolve_index = if file_norm.starts_with("src/v1/") {
            v1_attribution_index.as_ref().expect("v1 attribution index")
        } else {
            index
        };
        let content = match std::fs::read_to_string(&disk_path) {
            Ok(c) => c,
            Err(e) => return Err(format!("read failed for {file_path}: {e}")),
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
        let test_fn_names: HashSet<String> = scan_test_decl_names(&content).into_iter().collect();
        let mut decls: Vec<(i64, String, bool)> = Vec::new();
        for item in crate::v1_std_core::module_items(module_node.clone()).iter() {
            let line = byte_to_line_col(nl.clone(), item.span.start).line;
            let name = authored_name_at(single_si.clone(), item.clone());
            let is_data = item_kind(item.clone()) == ItemKind::DataItem;
            decls.push((line, name, is_data));
        }
        for (name, line) in scan_test_decl_lines(&content) {
            if !decls.iter().any(|(_, n, _)| n == &name) {
                decls.push((line, name, false));
            }
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
        let first_decl_line = decls[0].0;
        let mut changed =
            changed_new_lines_for_file(changed_new_lines_by_file, file_path, &file_norm);
        // Deletion-only hunks (`-` rows, zero `+` width) still carry a new-side anchor in the
        // hunk header; fall back to parsed ranges when no `+`/`-` rows were attributed.
        if changed.is_empty() {
            for r in ranges {
                let end = if r.end < r.start { r.start } else { r.end };
                for l in r.start..=end {
                    changed.insert(l);
                }
            }
        }
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
        if added_paths.contains(&file_norm) {
            for (line, _, _) in &decls {
                changed.insert(*line);
            }
        }
        // Module-line edits (line 1) stay fail-closed for modifies — renaming can
        // change entry identity. Wholly-added files necessarily touch line 1.
        if changed.contains(&1) && !added_paths.contains(&file_norm) {
            return Err(format!("diff before first declaration in {file_path}"));
        }
        let has_pre_decl = changed.iter().any(|&l| l < first_decl_line);
        let has_post_decl = changed.iter().any(|&l| l >= first_decl_line);
        if has_pre_decl {
            touched_entry_files.insert(file_norm.clone());
            if !has_post_decl {
                continue;
            }
        }
        for i in 0..decls.len() {
            let (line, name, is_data) = &decls[i];
            let decl_end = decls.get(i + 1).map(|(l, _, _)| l - 1).unwrap_or(i64::MAX);
            if !changed.iter().any(|&l| l >= *line && l <= decl_end) {
                continue;
            }
            if test_fn_names.contains(name) {
                edited_test_fns.insert((file_norm.clone(), name.clone()));
            } else if *is_data {
                overlapping_data_items.insert((file_norm.clone(), name.clone()));
            } else {
                touched_entry_files.insert(file_norm.clone());
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
        touched_entry_files,
    })
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
    /// The published cost observation, present exactly for a row that executed under
    /// `ChangedCostDebtVerdictOnly` and reached a verdict. `None` on every ordinary row, and on
    /// an override that published nothing — which is why that case reds rather than passing.
    pub cost: Option<ChangedWitnessCostObservation>,
}

/// THE CHANGED IDENTITIES, at the disposition receipt's own grain. The diff attribution is the
/// floor's existing authority (`floor_diff_edits_from_line_ranges` over the resolved comparison
/// baseline — the same observation every other diff consumer here makes), so "which test
/// declarations did this change touch" has one producer; this function only spells the result
/// as the qualified `module.function` identity the disposition receipt is keyed by. A wholly
/// added `.dag` file contributes every test declaration it carries; a modified file contributes
/// the declarations whose lines the diff reached. An observation or attribution failure
/// REFUSES — it never widens to "no changed witnesses".
pub(crate) fn changed_witness_identities(source_roots: &[String]) -> Result<Vec<String>, String> {
    let index = process_shared_index(source_roots);
    changed_witness_identities_with_index(&index)
}

fn changed_witness_identities_with_index(index: &MultiEntryIndex) -> Result<Vec<String>, String> {
    let diff_text = floor_git_diff_range()?;
    let (changed_paths, departed_paths) = floor_git_diff_name_status_range()?;
    let mut line_ranges_by_file = parse_unified_diff_line_ranges(&diff_text);
    for path in &changed_paths {
        line_ranges_by_file.entry(path.clone()).or_default();
    }
    let changed_new_lines_by_file = parse_unified_diff_changed_new_lines(&diff_text);
    let added_paths = parse_unified_diff_added_paths(&diff_text);
    let edits = floor_diff_edits_from_line_ranges(
        index,
        &line_ranges_by_file,
        &changed_new_lines_by_file,
        &departed_paths,
        &added_paths,
    )?;
    changed_witness_identities_from_edited_test_fns(
        &process_workspace_root(),
        &edits.edited_test_fns,
        &quarantine_probe_admitted_pairs(),
    )
}

/// The `(entry, function)` pairs whose admission says DO NOT SCHEDULE PER-PR, as a set at the grain
/// changed-witness selection decides at. Read once from the one function-grain authority.
fn quarantine_probe_admitted_pairs() -> std::collections::HashSet<(String, String)> {
    crate::cli_run::quarantine_probe_admission_pairs()
        .into_iter()
        .collect()
}

/// The identity spelling, separated from the observation so it is testable against fixture
/// files: `<authored module path>.<function>`, read from each touched file's own `module`
/// header — the same two facts the disposition loop joins into `RequiredFloorDispositionRow`'s
/// identity. A touched file that declares a test fn but no module header refuses: an identity
/// cannot be spelled for it, and dropping it would silently exempt exactly the malformed case.
///
/// A WITNESS CARRYING A `QuarantineProbeExpectRed` ADMISSION IS NOT SELECTED, and this is the one
/// place that decision belongs. Editing a quarantined witness's file does not un-quarantine it --
/// its dissolution trigger does -- so the override must not outrank an admission that already
/// answered whether this row is scheduled per-PR.
///
/// THE DEFECT THIS CLOSES, found by gunbc#10245 giving four such witnesses a declared subject. The
/// four `production_qualification_origin_probe_witness` frontier rows are admitted RED with
/// declared dissolutions and are declined by the ordinary floor. Touching their entry file planned
/// them anyway; they then failed exactly as admitted and were counted as UNEXPECTED failures,
/// refusing the floor (run 33793591024, `claims_failed=5`). The hold could not save them and must
/// not be taught to: `floor_expected_red_roster` means RUN-AND-EXPECT-RED while this admission
/// means DO-NOT-RUN-PER-PR, and making one consumer read the other's carrier is a §3 meaning fork.
/// The row was never supposed to execute here, so the repair is on the SELECTION side.
///
/// IT IS KEYED ON THE ADMISSION, NEVER ON THE LANE. Excluding by long-home prefix would wave a
/// touched witness through by virtue of the lane it happens to sit in, which is precisely what the
/// changed-witness override exists to prevent. A long-home witness with no quarantine admission is
/// selected, executes, and reds the floor exactly as before.
pub(crate) fn changed_witness_identities_from_edited_test_fns(
    base: &Path,
    edited_test_fns: &std::collections::HashSet<(String, String)>,
    quarantine_probe_admitted: &std::collections::HashSet<(String, String)>,
) -> Result<Vec<String>, String> {
    let mut identities: Vec<String> = Vec::new();
    for (file, function) in edited_test_fns {
        if quarantine_probe_admitted.contains(&(file.clone(), function.clone())) {
            continue;
        }
        let content = std::fs::read_to_string(base.join(file))
            .map_err(|e| format!("changed-witness identity derivation: read {file}: {e}"))?;
        let module = extract_module_path(&content).ok_or_else(|| {
            format!(
                "changed-witness identity derivation: {file} declares test fn `{function}` but \
                 no module header, so its qualified identity cannot be spelled"
            )
        })?;
        identities.push(format!("{module}.{function}"));
    }
    identities.sort();
    identities.dedup();
    Ok(identities)
}

/// Is this identity's declared home under a root the tree declares NON-EXECUTING?
///
/// The authority is `gunbc.ci_layer_roots` `non_executing_witness_module_prefixes`, read through the same
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
            },
            Some(
                RequiredFloorDisposition::Planned
                | RequiredFloorDisposition::PlannedAsChangedWitness,
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
                ChangedWitnessProjectionRow {
                    identity: identity.clone(),
                    cost: observation,
                    standing: match outcome {
                        Some(ClaimDisposition::KnownRedHeld) => "planned-and-known-red-held",
                        _ if cost_missing => "cost-observation-missing-under-verdict-only",
                        _ if reached_pass && verdict_only_policy => {
                            "planned-and-passed-with-cost-debt-observed"
                        }
                        _ if reached_pass => "planned-and-passed",
                        _ if wet_joined => "hermetic-route-gap-held-and-wet-passed",
                        _ => {
                            // "No terminal Passed verdict stands", deliberately covering a failed
                            // or refused verdict too — each of those already reds the floor by its
                            // own mechanism, and this projection reds it again at the changed-set
                            // grain, naming the identity the change touched.
                            "planned-without-terminal-verdict"
                        }
                    },
                    disposition: required_floor_disposition_label(dispositions[identity.as_str()])
                        .to_string(),
                    outcome: outcome
                        .map(claim_disposition_wire)
                        .unwrap_or("not_executed")
                        .to_string(),
                    blocks: !green,
                }
            }
            // A DECLINE BLOCKS UNLESS THE TREE GRANTS AN EXEMPTION, and the grant is membership in
            // a roster, never the absence of a match anywhere else.
            //
            // Blocking every decline is right for the silence the 2026-08-30 ruling closes: a PR
            // adds witnesses, they never run, the floor greens. `DeclinedOutsideGateClosure` is
            // that case. A root the fold structurally CANNOT reach is a different subject --
            // `gunbc.ci_layer_roots` `non_executing_witness_module_prefixes` declares it, and
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
                }
            }
            Some(declined) => ChangedWitnessProjectionRow {
                identity: identity.clone(),
                cost: None,
                standing: "declined",
                disposition: required_floor_disposition_label(declined).to_string(),
                outcome: "not_executed".to_string(),
                blocks: true,
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
}

/// WHAT ONE MEMBER ACTUALLY REACHED, at the width of the `.dag` authority's observed side.
///
/// This used to be a `bool`. Passed, Failed, Refused, Nonterminal and CompletedOverBudget are
/// different facts with different remedies, and folding them into "not passed" made a route with no arm
/// indistinguishable from a witness that ran and disagreed — a widened failure arm wearing a
/// precise one's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LocalRepoWetObserved {
    Passed,
    Failed,
    Refused(String),
    Nonterminal(String),
    CompletedOverBudget(String),
}

impl LocalRepoWetObserved {
    fn name(&self) -> &'static str {
        match self {
            LocalRepoWetObserved::Passed => "passed",
            LocalRepoWetObserved::Failed => "failed",
            LocalRepoWetObserved::Refused(_) => "refused",
            LocalRepoWetObserved::Nonterminal(_) => "nonterminal",
            LocalRepoWetObserved::CompletedOverBudget(_) => "completed-over-budget",
        }
    }

    fn detail(&self) -> &str {
        match self {
            LocalRepoWetObserved::Passed | LocalRepoWetObserved::Failed => "",
            LocalRepoWetObserved::Refused(d)
            | LocalRepoWetObserved::Nonterminal(d)
            | LocalRepoWetObserved::CompletedOverBudget(d) => d.as_str(),
        }
    }

    /// The host realization of `wet_disposition_is_agreement` for the one arm of
    /// `ClaimExpectation` this lane realizes. Exhaustive on the observed side by construction.
    fn meets_pass_expectation(&self) -> bool {
        matches!(self, LocalRepoWetObserved::Passed)
    }
}

/// EVERY ARM IS SPELLED, AND THE WILDCARD IS GONE ON PURPOSE.
///
/// This read `other => Refused(...)`, which quietly absorbed every FUTURE `ClaimOutcome` variant
/// into one class. A producer arm added upstream would have been classified by a catch-all that
/// never considered it, and nothing would have said so — the compiler's exhaustiveness check is
/// the only thing that can make a new outcome a decision rather than a default, and a wildcard
/// disables exactly that. A new variant must now fail this match until someone classifies it.
fn local_repo_wet_observed_from(outcome: &crate::cli_run::ClaimOutcome) -> LocalRepoWetObserved {
    use crate::cli_run::ClaimOutcome as O;
    match outcome {
        O::Pass => LocalRepoWetObserved::Passed,
        O::Fail => LocalRepoWetObserved::Failed,
        // NO VERDICT WAS REACHED. The claim was stopped or never started, which is not the same
        // as reaching a verdict this lane disagrees with.
        O::BudgetInterrupted { .. } => LocalRepoWetObserved::Nonterminal("budget".to_string()),
        O::NotAttempted { halted_by } => {
            LocalRepoWetObserved::Nonterminal(format!("not attempted: {halted_by}"))
        }
        // IT COMPLETED. Calling this nonterminal contradicted the producer: the claim reached its
        // verdict and then the run exceeded a declared line, which is a cost fact and not an
        // absence of one. The lane still refuses it — a member that only passes by running past
        // its budget is not a member this lane can carry — but it refuses it AS a completed
        // over-budget observation, so the reader is sent to the budget rather than to the route.
        O::CompletedOverBudget {
            elapsed_ms,
            budget_ms,
            kind,
        } => LocalRepoWetObserved::CompletedOverBudget(format!(
            "{kind:?} {elapsed_ms}ms over the {budget_ms}ms line"
        )),
        // THE ROUTE OR THE PROGRAM REFUSED. Each is a located typed refusal in its own right; the
        // lane keeps the class and hands the reader that diagnostic.
        O::NotBool { got } => LocalRepoWetObserved::Refused(format!("not a Bool: {got}")),
        O::RuntimeError { cause, message } => {
            LocalRepoWetObserved::Refused(format!("runtime error {cause:?}: {message}"))
        }
        O::HostToolUnresolved { name, probed } => LocalRepoWetObserved::Refused(format!(
            "host tool {name} unresolved after probing {} path(s)",
            probed.len()
        )),
        O::HostEffectRefused { operation, ground } => {
            LocalRepoWetObserved::Refused(format!("host effect {operation} refused: {ground:?}"))
        }
        O::Panicked { .. } => LocalRepoWetObserved::Refused("panicked".to_string()),
    }
}

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

/// ONE TERMINAL RECEIPT, at the width of `v2.workflow.local_repo_wet_terminal.LocalRepoWetTerminal`.
/// `entry` is the source the prepared subject RESOLVED for the module and `candidate` is the tree
/// the executor ran against — both recorded rather than restated from the roster, because the join
/// below decides whether they agree with what was scheduled.
#[derive(Debug, Clone)]
pub(crate) struct LocalRepoWetTerminalRow {
    pub identity: String,
    pub entry: String,
    pub function: String,
    pub candidate: String,
    pub observed: LocalRepoWetObserved,
}

/// WHETHER THE EXECUTOR RAN AT ALL, as a value the finalizer is handed.
///
/// Host realization of `v2.workflow.local_repo_wet_terminal.LocalRepoWetExecution`, and it carries
/// the same payload the modeled arm does: the candidate and the TERMINAL POPULATION. It carried a
/// pre-adjudicated `LocalRepoWetLaneOutcome` instead — a count, a candidate and an admitted set —
/// which meant the finalizer could only re-read the executor's own verdict, so a run that skipped a
/// scheduled member, mis-stated its count, or admitted an identity nobody scheduled finalized
/// green. The `.dag` fold refuses all three; a host arm that trusts its payload realizes none of it.
///
/// The join this lane already models is the completeness half of the wall; `NotInvoked` is the
/// liveness half. With no invocation there are no terminals, and a join over an unread schedule
/// holds vacuously — the deleted-`falsifier.yml` shape at this lane's grain, now that
/// `std.witness_admission` claims a live route for `LocalRepoWetLane`. So absence is an ARM, and
/// `Ran` has one producer: `run_local_repo_wet_lane` itself.
#[derive(Debug)]
pub(crate) enum LocalRepoWetExecution {
    NotInvoked,
    Ran {
        candidate: String,
        terminals: Vec<LocalRepoWetTerminalRow>,
    },
}

/// THE FLOOR'S WHOLE QUESTION ABOUT THIS LANE, asked once and unconditionally, with execution as an
/// argument. Host realization of `v2.workflow.local_repo_wet_terminal.local_repo_wet_finalize` and
/// of the `wet_evidence_validate` it defers to: the same three finalization causes and the same
/// bidirectional join, separated by remedy rather than collapsed into "the receipt is bad".
///
/// An empty schedule with no invocation HOLDS: a lane that scheduled nobody claims nobody. It is
/// `scheduled > 0` that turns absence into a refusal, and the schedule is non-empty by construction
/// on this tree, so production reaches only the refusing cell of that arm.
///
/// `LocalRepoWetLaneOutcome` is DERIVED from a join that held, never accepted from the executor.
pub(crate) fn finalize_local_repo_wet_lane(
    schedule: &[LocalRepoWetScheduledRow],
    execution: LocalRepoWetExecution,
    candidate: &str,
) -> Result<LocalRepoWetLaneOutcome, String> {
    let (ran_candidate, terminals) = match execution {
        LocalRepoWetExecution::NotInvoked => {
            if schedule.is_empty() {
                return Ok(LocalRepoWetLaneOutcome {
                    scheduled: 0,
                    candidate: candidate.to_string(),
                    admitted: HashSet::new(),
                    refusals: Vec::new(),
                });
            }
            return Err(format!(
                "local-repo wet lane: LocalRepoWetExecutorAbsent — the executor was not invoked \
                 while {} member(s) were scheduled, so the lane's schedule-to-terminal join had \
                 nothing to disagree with and `std.witness_admission`'s LocalRepoWetLane route \
                 claim would have been asserted rather than executed",
                schedule.len()
            ));
        }
        LocalRepoWetExecution::Ran {
            candidate,
            terminals,
        } => (candidate, terminals),
    };
    if ran_candidate != candidate {
        return Err(format!(
            "local-repo wet lane: LocalRepoWetExecutionForeignCandidate — the executor ran against \
             {ran_candidate} while this floor is evaluating {candidate}"
        ));
    }
    let mut refusals: Vec<String> = Vec::new();
    let mut admitted: HashSet<String> = HashSet::new();
    // FORWARD: every scheduled identity has exactly one terminal, for this candidate, naming the
    // entry and function that were scheduled, with the verdict the roster expects.
    for row in schedule {
        let hits: Vec<&LocalRepoWetTerminalRow> = terminals
            .iter()
            .filter(|t| t.identity == row.identity)
            .collect();
        match hits.len() {
            0 => refusals.push(format!(
                "{}: WetTerminalMissing — scheduled, and the executor produced no terminal \
                 for it",
                row.identity
            )),
            1 => {
                let t = hits[0];
                if t.candidate != candidate {
                    refusals.push(format!(
                        "{}: WetBindingPreparedSubjectDiffers — terminal names {}, this floor \
                         is evaluating {candidate}",
                        row.identity, t.candidate
                    ));
                } else if t.entry != row.entry {
                    refusals.push(format!(
                        "{}: WetTerminalForeignEntry — the roster's entry {:?} is not the \
                         source the prepared subject resolved ({})",
                        row.identity, row.entry, t.entry
                    ));
                } else if t.function != row.function {
                    refusals.push(format!(
                        "{}: WetTerminalForeignFunction — terminal names {}, the roster \
                         scheduled {}",
                        row.identity, t.function, row.function
                    ));
                } else if !t.observed.meets_pass_expectation() {
                    let detail = t.observed.detail();
                    let suffix = if detail.is_empty() {
                        String::new()
                    } else {
                        format!(" — {detail}")
                    };
                    refusals.push(format!(
                        "{}: WetTerminalVerdictNotExpected — expected passed, observed \
                         {}{suffix}",
                        row.identity,
                        t.observed.name()
                    ));
                } else {
                    admitted.insert(row.identity.clone());
                }
            }
            n => refusals.push(format!(
                "{}: WetTerminalDuplicated — {n} terminals claim this identity and neither \
                 can be trusted over the other",
                row.identity
            )),
        }
    }
    // REVERSE: a terminal for an identity nobody scheduled is a refusal, not a bonus. Without it an
    // executor could admit anything it liked as long as it also ran the roster.
    for t in &terminals {
        if !schedule.iter().any(|row| row.identity == t.identity) {
            refusals.push(format!(
                "{}: WetTerminalUnscheduled — a terminal for an identity this lane never \
                 scheduled",
                t.identity
            ));
        }
    }
    for refusal in &refusals {
        eprintln!("required-floor: LOCAL-REPO-WET REFUSAL {refusal}");
    }
    if !refusals.is_empty() {
        // THE CAUSES TRAVEL WITH THE COUNT. A count alone sends a reader back to the log to find
        // out WHICH member and which cause, and makes every join cell indistinguishable to any
        // caller that can only see the returned diagnostic.
        return Err(format!(
            "local-repo wet lane: {} refusal(s) — the lane's schedule and its terminals disagree: {}",
            refusals.len(),
            refusals.join("; ")
        ));
    }
    eprintln!(
        "[local-repo-wet] scheduled={} admitted={} refusals=0",
        schedule.len(),
        admitted.len()
    );
    Ok(LocalRepoWetLaneOutcome {
        scheduled: schedule.len(),
        candidate: candidate.to_string(),
        admitted,
        refusals: Vec::new(),
    })
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
        });
    }
    Ok(out)
}

/// RUN THE LANE, WET, AGAINST THIS CANDIDATE.
///
/// The frame is built over the SAME prepared subject the floor already holds, in
/// `ExecutionMode::Wet`, so the members execute their real effects against the very tree being
/// evaluated. That is what makes the terminal candidate-bound by construction: there is no receipt
/// to carry from elsewhere and no artifact handoff that could answer for another tree.
///
/// IT PRODUCES TERMINALS AND ADJUDICATES NOTHING. Every disagreement between the roster and what
/// ran — a member with no receipt, a receipt naming another entry, a verdict that was not the
/// expected one — is decided by `finalize_local_repo_wet_lane` against the SAME schedule, which is
/// the host realization of `wet_evidence_validate`. An executor that judged its own completeness
/// would be a second authority for the join, and the one whose payload the finalizer then trusted.
pub(crate) fn run_local_repo_wet_lane(
    prepared: &PreparedRepository,
    schedule: &[LocalRepoWetScheduledRow],
    published: Option<Rc<HashSet<String>>>,
) -> LocalRepoWetExecution {
    let mut terminals: Vec<LocalRepoWetTerminalRow> = Vec::new();
    // Group by module so one scope is prepared per entry rather than per witness.
    let mut by_module: Vec<(String, Vec<&LocalRepoWetScheduledRow>)> = Vec::new();
    for row in schedule {
        match by_module.iter_mut().find(|(m, _)| m == &row.entry_module) {
            Some((_, rows)) => rows.push(row),
            None => by_module.push((row.entry_module.clone(), vec![row])),
        }
    }
    for (module, rows) in &by_module {
        // THE TERMINAL CARRIES THE SOURCE PREPARATION ACTUALLY RESOLVED, never the path the roster
        // authored. That is what lets the join decide `ForeignEntry`: comparing the two here would
        // put the same comparison in two places, and the executor's copy would be the one deciding
        // what the finalizer never sees.
        let resolved_file = prepared
            .graph
            .modules
            .iter()
            .find(|m| m.func_env.name.as_str() == module.as_str())
            .map(|m| m.module.span.file.to_string());
        let scope = match claim_scope_for(prepared, module) {
            Ok(s) => s,
            Err(e) => {
                // A SCHEDULED MEMBER THE SUBJECT CANNOT REACH PRODUCES NO TERMINAL, and the join
                // then refuses it as missing — which is the truthful shape: nothing ran, so there
                // is nothing to report a verdict for. The reason is printed here, where it is
                // observed, so the identity the finalizer names has a located cause beside it.
                for row in rows {
                    eprintln!(
                        "[local-repo-wet] identity={} unreachable: scheduled module {module} has \
                         no scope in this subject: {e}",
                        row.identity
                    );
                }
                continue;
            }
        };
        let ctx = crate::cli_run::evaluation_frame(
            &scope,
            v1_interpreter::ExecutionMode::Wet,
            None,
            published.clone(),
        );
        for row in rows {
            let observed =
                local_repo_wet_observed_from(&crate::cli_run::run_claim(&ctx, &row.function));
            eprintln!(
                "[local-repo-wet] identity={} expected=passed observed={}",
                row.identity,
                observed.name(),
            );
            terminals.push(LocalRepoWetTerminalRow {
                identity: row.identity.clone(),
                entry: resolved_file.clone().unwrap_or_default(),
                function: row.function.clone(),
                candidate: prepared.subject_digest.clone(),
                observed,
            });
        }
    }
    eprintln!(
        "[local-repo-wet] scheduled={} terminals={}",
        schedule.len(),
        terminals.len()
    );
    // THE ONLY PRODUCER OF `Ran`. The floor holds `NotInvoked` until this function returns, so the
    // execution value the finalizer reads is a fact about whether this function ran.
    LocalRepoWetExecution::Ran {
        candidate: prepared.subject_digest.clone(),
        terminals,
    }
}

/// Print the projection — one `[changed-witness]` line per CHANGED identity (never one per
/// declined identity in the standing population; the §4b rung drop "Required gate reduced to
/// the compiler floor" owns that population and re-printing it would bury this signal), one
/// aggregate `required-floor:` line always, and the same rows as a markdown table appended to
/// `GITHUB_STEP_SUMMARY` when the environment provides one. A summary the environment asked
/// for that cannot be written REFUSES: evidence written only when convenient is the
/// instrumentation-optional shape the floor's other publications already forbid.
pub(crate) fn emit_changed_witness_projection(
    rows: &[ChangedWitnessProjectionRow],
) -> Result<(), String> {
    for row in rows {
        // THE COST TRAVELS WITH THE LINE THAT CLAIMS IT WAS OBSERVED. A standing named
        // "...with-cost-debt-observed" printed beside no figures would assert an observation the
        // reader cannot see; the receipt is the numbers, not the label.
        let cost = match row.cost {
            Some(observation) => format!(
                " marginal_cpu_ms={} wall_ms={} cpu_line_ms={}",
                observation.cpu_clock_nanos / 1_000_000,
                observation.wall_clock_nanos / 1_000_000,
                observation.cpu_line_ms
            ),
            None => String::new(),
        };
        eprintln!(
            "[changed-witness] identity={} standing={} disposition={} outcome={}{}",
            row.identity, row.standing, row.disposition, row.outcome, cost
        );
    }
    let blocking = rows.iter().filter(|r| r.blocks).count();
    // THE EXEMPTED POPULATION IS PRINTED BESIDE THE TWO IT SITS BETWEEN, because a non-blocking
    // decline that nobody counts is the silence the 2026-08-30 ruling forbids, merely relocated.
    // Countable and named is what separates a declared exemption from a hole.
    let declared_non_executing = rows
        .iter()
        .filter(|r| r.standing == "declined-in-declared-non-executing-root")
        .count();
    eprintln!(
        "required-floor: changed_witnesses={} changed_witness_blocking={} \
         changed_witness_declined_in_declared_nonexecuting_root={}",
        rows.len(),
        blocking,
        declared_non_executing
    );
    if let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY") {
        if !rows.is_empty() {
            let mut text = String::new();
            text.push_str("### Changed-witness execution standing\n\n");
            text.push_str(
                "One row per ADDED/MODIFIED witness identity in this change, at the disposition \
                 receipt's own grain (qualified `module.function`). Standing vocabulary: \
                 `v2.workflow.floor_changed_witness.ChangedWitnessExecutionStanding`; every \
                 standing except planned-and-passed and planned-and-known-red-held reds the \
                 required floor.\n\n",
            );
            text.push_str("| identity | disposition | outcome | standing |\n|---|---|---|---|\n");
            for row in rows {
                let mark = if row.blocks { "❌ " } else { "✅ " };
                text.push_str(&format!(
                    "| `{}` | {} | {} | {}{} |\n",
                    row.identity, row.disposition, row.outcome, mark, row.standing
                ));
            }
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| format!("changed-witness step summary: open {path}: {e}"))?;
            file.write_all(text.as_bytes())
                .map_err(|e| format!("changed-witness step summary: write {path}: {e}"))?;
        }
    }
    Ok(())
}

pub fn run_discovery_corpus_with_options(
    source_roots: &[String],
    scan_dirs: &[String],
    explicit_entries: &[(String, String)],
    execution_mode: v1_interpreter::ExecutionMode,
    width_policy: DiscoveryWidthPolicy,
    options: DiscoveryCorpusOptions,
) -> Result<DiscoverySummary, String> {
    let pump_started = std::time::Instant::now();
    let out = run_discovery_corpus_with_options_inner(
        source_roots,
        scan_dirs,
        explicit_entries,
        execution_mode,
        width_policy,
        options,
    );
    discovery_phase_totals::add(
        &discovery_phase_totals::PUMP_WALL_MS,
        pump_started.elapsed(),
    );
    out
}

pub(crate) fn run_discovery_corpus_with_options_inner(
    source_roots: &[String],
    scan_dirs: &[String],
    explicit_entries: &[(String, String)],
    execution_mode: v1_interpreter::ExecutionMode,
    width_policy: DiscoveryWidthPolicy,
    options: DiscoveryCorpusOptions,
) -> Result<DiscoverySummary, String> {
    let mut rows =
        if options.explicit_roster_only || (scan_dirs.is_empty() && !explicit_entries.is_empty()) {
            Vec::new()
        } else {
            let t = std::time::Instant::now();
            let walked = discover_floor_witness_roster(
                source_roots,
                scan_dirs,
                &options.exclude_substrings,
                &options.discovery_scope_dirs,
            );
            discovery_phase_totals::add(&discovery_phase_totals::ROSTER_WALK_MS, t.elapsed());
            walked?
        };
    let mut seen: std::collections::BTreeSet<(String, String)> = rows
        .iter()
        .map(|r| (r.entry.clone(), r.function.clone()))
        .collect();
    // U3 — empty function = file-grain: enumerate via the same test-decl scan discovery uses.
    let expanded_explicit = test_module_hygiene_bridge::expand_explicit_entries(explicit_entries)?;
    for (entry, function) in &expanded_explicit {
        if seen.insert((entry.clone(), function.clone())) {
            rows.push(DiscoveryRow {
                label: function.clone(),
                entry: entry.clone(),
                function: function.clone(),
                reads_live_tree: read_entry_live_tree_disposition(entry)?,
            });
        }
    }
    rows.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    if rows.is_empty() {
        return Err("discovery roster produced no rows (empty corpus → fail closed)".to_string());
    }
    let width_policy = match width_policy {
        DiscoveryWidthPolicy::DerivedSchedule => {
            let pairs: Vec<(String, String)> = rows
                .iter()
                .map(|r| (r.entry.clone(), r.function.clone()))
                .collect();
            let derived = crate::derived_realization_schedule::derive_discovery_schedule_width(
                source_roots,
                &pairs,
            )?;
            if let Some(msg) = derived.refuse_if_budget_unreadable() {
                return Err(msg);
            }
            eprintln!(
                "run_discovery_corpus: derived schedule width={} verdict={} max_derived_bound={}",
                derived.width,
                derived.verdict,
                derived
                    .max_derived_bound_bytes
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "unknown".into()),
            );
            DiscoveryWidthPolicy::FixedWidth(derived.width.max(1))
        }
        other => other,
    };
    // P4 advisory-first: predict the memory-packed width per witness from its derived
    // space bound, logged beside the governor — no scheduling change. Gated (opt-in).
    if std::env::var("GUNBC_REALIZE_ADVISORY").is_ok() {
        emit_realize_advisory_for_rows(source_roots, &rows);
    }
    let deferred_rows = if options.explicit_roster_only || scan_dirs.is_empty() {
        Vec::new()
    } else {
        collect_deferred_discovery_rows(source_roots, &options.exclude_substrings)?
    };
    let admission_orphans = collect_unexecuted_deferred_witnesses(&deferred_rows);
    refuse_unexecuted_deferred_witnesses(&admission_orphans)?;
    if !deferred_rows.is_empty() {
        refuse_stale_frozen_path_deferrals(&collect_stale_frozen_path_deferrals())?;
        refuse_frozen_path_deferral_additions(&collect_frozen_path_deferral_additions()?)?;
    }
    eprintln_deferred_discovery_rows(&deferred_rows);
    set_phase(FloorPhase::Discovery, "discovery-roster");
    if options.execution_authority_source_roots.is_empty() {
        return Err(
            "discovery execution requires an explicit executor-authority source-root universe"
                .to_string(),
        );
    }
    let execution_authority_is_subject = options.execution_authority_source_roots == source_roots;
    // Union-resolve S1 (resolver-graph-major-design (deleted) §7): ONE index for the whole
    // process step on the pump thread — prelude-warmed parse/typed caches instead of a
    // private cold build per consumer. S2a increment C (cross-worker-typecheck-share-
    // design.md §4): adaptive worker shards arm ONE process-scoped typed_module_cache
    // (serde byte transport). The pump thread keeps `process_shared_index` (private per-
    // index `Rc`) so prelude work does not duplicate into the shared store; workers alone
    // read/write the shared store as the typed-cache authority (no local Rc duplicate).
    // Store creation lives in the Adaptive match arm below — unrepresentable on Serial.
    let index = if p1_cohort_experiment_active()
        && matches!(width_policy, DiscoveryWidthPolicy::Serial)
        && p1_experimental_arm_shared_typed_store(1)
    {
        let store = new_shared_typecheck_caches();
        Rc::new(build_multi_entry_index_with_shared_caches(
            source_roots,
            store,
        ))
    } else {
        process_shared_index(source_roots)
    };
    // Calibration receipt, emitted BEFORE the heavy resolve so it survives a host-level
    // OOM kill (censored lower-bound pairs for the space-lens memory predictor — design
    // in flight on PR #6442; consumer binds to roster_import_closure_nodes_pre_resolve):
    // the transitive import-CLOSURE size — never the roster/entry count (pairing an
    // entry count against a whole-closure peak inflates bytes-per-node by the fan-in
    // factor). Skip-before-resolve (run_discovery_rows) elides cold resolve for
    // import-closure-unaffected entries while folding their module-graph closure into
    // the post-resolve union so this pre-resolve count stays paired with calibration.
    let preresolve_calibration_started = std::time::Instant::now();
    let pre_resolve_closure_nodes = {
        let n = roster_import_closure_nodes_pre_resolve(&rows, &[], &index)?;
        eprintln!(
            "[calibration] roster_import_closure_nodes={} rows={} (loader both-closure union, pre-resolve, no resolve/typecheck; pairs with the floor cgroup memory.peak steps — on a killed run this line plus the last [gantt] rss_mib sample are the lower-bound receipt)",
            n,
            rows.len()
        );
        n
    };
    discovery_phase_totals::add(
        &discovery_phase_totals::PRERESOLVE_CALIBRATION_MS,
        preresolve_calibration_started.elapsed(),
    );
    let whole_tree_published_keys = match precompute_whole_tree_published_mock_keys(source_roots) {
        Ok(keys) if keys.is_empty() => None,
        Ok(keys) => Some(keys),
        Err(e) => {
            return Err(format!(
                "whole-tree published mock corpus precompute failed: {e}"
            ));
        }
    };
    // Derive every leg for the WHOLE roster here, above the width dispatch, while this
    // thread's shared index is warm. At width > 1 the pool hands each worker its own chunk
    // of rows, so priming inside `run_discovery_rows` would build one interpreter context
    // per worker — and width is adaptive, so "n is small here" is not a fact that stays
    // true (§6). One build covers the run; workers only read the process-wide memo.
    prime_witness_execution_legs_from_authority(
        &index,
        (!execution_authority_is_subject)
            .then_some(options.execution_authority_source_roots.as_slice()),
        rows.iter().map(|row| row.entry.as_str()),
    );

    let floor_color = floor_color_enabled();
    let floor_stream = floor_stream_enabled();
    return match width_policy {
        DiscoveryWidthPolicy::DerivedSchedule => {
            unreachable!("DerivedSchedule is lowered to FixedWidth before the pool match")
        }
        DiscoveryWidthPolicy::Serial => {
            // Arm retention over the WHOLE serial schedule (all rows) before the single drain
            // call — a shared module stays resident until its last scheduled entry consumes it.
            index_arm_schedule_retention(&index, &rows);
            let summary = run_discovery_rows(
                &rows,
                &index,
                execution_mode,
                whole_tree_published_keys.clone(),
                options.witness_budget_policy(),
                ShardStyle {
                    shard_id: 0,
                    shard_count: 1,
                    color: floor_color,
                    stream: floor_stream,
                },
            )?;
            // Definition-drift oracle (single-authority reconciliation, executable): on a
            // COMPLETED serial run the pre-resolve import walk and the post-resolve
            // resolved-graph union must agree — resolve resolves exactly the transitive
            // imports. Serial only: the merged multi-worker field is max-over-workers, not
            // the process union, so the comparison is ill-posed there. A mismatch means one
            // closure definition is wrong (an implicit prelude module the walk missed, or a
            // resolve seeding change) and the space-lens calibration pair would silently
            // skew — refuse rather than emit a lying receipt.
            if summary.roster_closure_nodes != pre_resolve_closure_nodes {
                return Err(format!(
                    "[calibration] closure-definition drift: pre-resolve loader-closure union = {} nodes, \
                     post-resolve resolved union = {} — the two closure definitions diverged \
                     (loader fork or seeding change: resolve loaded a module set the loader \
                     both-closure fixpoint did not produce, or vice versa); reconcile the \
                     definitions before trusting bytes-per-node calibration \
                     (roster_import_closure_nodes_pre_resolve is the shared authority)",
                    pre_resolve_closure_nodes, summary.roster_closure_nodes
                ));
            }
            eprintln!(
                "[calibration] closure consistency: pre-resolve loader-closure union == post-resolve union == {} node(s)",
                pre_resolve_closure_nodes
            );
            Ok(finalize_discovery_summary(summary, &rows, deferred_rows))
        }
        DiscoveryWidthPolicy::ControlledWidthTwo => {
            const CONTROLLED_WIDTH: usize = 2;
            let arm_shared_store = if p1_cohort_experiment_active() {
                p1_experimental_arm_shared_typed_store(CONTROLLED_WIDTH)
            } else {
                true
            };
            let groups = entry_row_groups(&rows);
            eprintln!(
                "run_discovery_corpus: controlled width-2 pool over {} entry-group(s), {} row(s), shared_typed_store={}",
                groups.len(),
                rows.len(),
                arm_shared_store,
            );
            let cross_worker_store = arm_shared_store.then(new_shared_typecheck_caches);
            if floor_stream {
                eprintln!(
                    "{} [affected-set] controlled width-2 pool (fixed {} workers; shared typed-module store={})",
                    floor_ts(),
                    CONTROLLED_WIDTH,
                    arm_shared_store,
                );
            }
            let queue: std::sync::Arc<Mutex<VecDeque<Vec<DiscoveryRow>>>> =
                std::sync::Arc::new(Mutex::new(
                    groups
                        .into_iter()
                        .map(|g| g.iter().map(|&i| rows[i].clone()).collect())
                        .collect(),
                ));
            let abort = std::sync::Arc::new(AtomicBool::new(false));
            let source_roots_owned = source_roots.to_vec();
            let budget_policy_for_workers = options.witness_budget_policy();
            let mut handles = Vec::with_capacity(CONTROLLED_WIDTH);
            for worker_ordinal in 0..CONTROLLED_WIDTH {
                let queue_for_worker = queue.clone();
                let abort_for_worker = abort.clone();
                let roots = source_roots_owned.clone();
                let keys = whole_tree_published_keys.clone();
                let store = cross_worker_store.clone();
                let arm_shared_for_worker = arm_shared_store;
                let style = ShardStyle {
                    shard_id: worker_ordinal,
                    shard_count: CONTROLLED_WIDTH,
                    color: floor_color,
                    stream: floor_stream,
                };
                handles.push(std::thread::spawn(
                    move || -> Result<Vec<DiscoverySummary>, String> {
                        let index = if arm_shared_for_worker {
                            let store = store
                                .expect("shared typed store armed but cross_worker_store missing");
                            build_multi_entry_index_with_shared_caches(&roots, store)
                        } else {
                            build_multi_entry_index(&roots)
                        };
                        let mut worker_summaries = Vec::new();
                        loop {
                            if abort_for_worker.load(Ordering::SeqCst) {
                                break;
                            }
                            let Some(group_rows) = queue_for_worker.lock().unwrap().pop_front()
                            else {
                                break;
                            };
                            match run_discovery_rows(
                                &group_rows,
                                &index,
                                execution_mode,
                                keys.clone(),
                                budget_policy_for_workers,
                                style,
                            ) {
                                Ok(summary) => worker_summaries.push(summary),
                                Err(e) => {
                                    abort_for_worker.store(true, Ordering::SeqCst);
                                    return Err(e);
                                }
                            }
                        }
                        Ok(worker_summaries)
                    },
                ));
            }
            let mut summaries = Vec::new();
            let mut first_err: Option<String> = None;
            for handle in handles {
                match handle
                    .join()
                    .map_err(|_| "controlled-width discovery worker panicked".to_string())
                {
                    Ok(Ok(worker_summaries)) => summaries.extend(worker_summaries),
                    Ok(Err(e)) | Err(e) => first_err = first_err.or(Some(e)),
                }
            }
            if let Some(e) = first_err {
                return Err(e);
            }
            let leftover = queue.lock().unwrap().len();
            if leftover > 0 {
                return Err(format!(
                    "controlled width-2 pool exited with {leftover} undrained entry-group(s)"
                ));
            }
            Ok(finalize_discovery_summary(
                merge_discovery_summaries(summaries),
                &rows,
                deferred_rows,
            ))
        }
        DiscoveryWidthPolicy::FixedWidth(pool_width) => {
            // Derived schedule pool: entry-groups drain through a fixed worker count chosen
            // up front by std.realize_pack over the roster's derived space bounds.
            let groups = entry_row_groups(&rows);
            let spawn_target_width = pool_width;
            eprintln!(
                "run_discovery_corpus: derived schedule pool over {} entry-group(s), {} row(s) (scheduled width={})",
                groups.len(),
                rows.len(),
                spawn_target_width,
            );
            // Width=1: drain inline on the pump thread reusing `process_shared_index` (already
            // warmed for calibration + floor runner). Spawning a worker thread duplicates the
            // whole-tree index on a second thread-local cache — ~2× retention that OOM'd CI
            // batch-2 discovery (runs 29372308568 / 29373433928). Cross-worker store arms only
            // when plural workers run (below).
            //
            // This width read is deliberately SAMPLED ONCE, and at width 1 that makes the
            // window an absorbing state for this pool: the only path that grows it (a slot
            // completion) lives past the branch below, so the governor's AIMD controller is
            // not reachable from the corpus. That is a real defect in the controller — and
            // un-latching it is nonetheless a MEASURED LOSS, so the latch stays until the
            // cost it hides is gone. Same branch, same 621 entry-groups, same .rs-forced
            // whole-tree path: serial 11.75min GREEN (CI 29707161743 — max_width_reached=1,
            // admissions=1, peak 6.97 GB) vs un-latched 47min+ without finishing (CI
            // 29714863168), vs un-latched with per-unit window growth OOM-killed at
            // 101.6 GB in 11min (CI 29710324768).
            //
            // The reason is Amdahl, not a bug: a worker's front cost is its own whole-tree
            // index build (~10.7 GB, minutes) and the entire corpus is ~12 minutes of work,
            // so every added worker costs more setup than the parallelism it buys. Width is
            // not worth reaching for while the index is per-worker; the governor's job here
            // is to be correct when it IS reachable — see `CompletionKind` in
            // `memory_governor`, where the window tracks landed worker cost and never the
            // unit-completion rate.
            // 🟡 dissolve-on: Rc→Arc retires the width gate — sharing the index removes the
            // per-worker front cost, which is the thing that makes width unprofitable. Priced
            // FIRST by the share spike (cross-worker-typecheck-share-design (plan doc deleted 2026-08-28) §9
            // open decision 2), because that design's §7 warns a shared store also INCREASES
            // co-resident retention: the win is a crossover in width, not a given.
            if spawn_target_width <= 1 {
                eprintln!(
                    "run_discovery_corpus: width=1 inline drain — reusing process_shared_index (no worker duplicate index)"
                );
                eprintln!(
                    "run_discovery_corpus: cross_worker_store withheld (scheduled width={spawn_target_width}) — per-index typed cache until width > 1"
                );
                let style = ShardStyle {
                    shard_id: 0,
                    shard_count: 1,
                    color: floor_color,
                    stream: floor_stream,
                };
                let mut summaries = Vec::new();
                let drain_detail = floor_drain_retention_detail_enabled();
                let total_groups = groups.len();
                let mut drain_prev = index_retention_snapshot(&index);
                let mut drain_peaks = drain_prev;
                // Arm retention over the WHOLE batch schedule (every group's rows) ONCE, before
                // the inline drain — NOT per group. The drain reuses the one process-shared index
                // across all entry-groups, so a shared compiler-core module reached by many
                // entries keeps a refcount > 1 and stays resident until its LAST consumer's entry
                // completes; only an entry's genuinely-unique tail evicts when that entry finishes.
                // Per-group arming instead gave each entry a one-entry schedule (refcount 1 on the
                // whole closure), evicting and cold-recomputing the shared core once per entry.
                index_arm_schedule_retention(&index, &rows);
                let p1_cohort_detail = p1_cohort_receipt_enabled();
                let mut p1_cohort_seen_subjects: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for (group_idx, group_indices) in groups.into_iter().enumerate() {
                    let group_rows: Vec<DiscoveryRow> =
                        group_indices.iter().map(|&i| rows[i].clone()).collect();
                    let group_entry_label = group_rows
                        .first()
                        .map(|r| r.entry.clone())
                        .unwrap_or_default();
                    let group_wall_start = p1_cohort_detail.then(std::time::Instant::now);
                    let typecheck_misses_before = p1_cohort_detail.then(typecheck_compute_count);
                    let summary = run_discovery_rows(
                        &group_rows,
                        &index,
                        execution_mode,
                        whole_tree_published_keys.clone(),
                        options.witness_budget_policy(),
                        style,
                    )?;
                    // The rest of this block is P1 scaffold bookkeeping (per-group wall
                    // timing, typecheck-memo before/after, and the resolved-graph-hit
                    // subject-set scan) — computed only under the same opt-in gate as
                    // its emission (review 47844), not on the default production path.
                    let (group_wall_ms, typecheck_misses_after, resolved_graph_hit) =
                        if p1_cohort_detail {
                            let group_wall_ms = group_wall_start
                                .expect("set above under the same p1_cohort_detail gate")
                                .elapsed()
                                .as_millis();
                            let typecheck_misses_after = typecheck_compute_count();
                            // Cohort-scoped "have we already resolved a closure sharing this
                            // subject earlier in THIS run" fact — a resolved-graph-memo hit
                            // proxy at the granularity the P1 receipt needs (whether entry N
                            // is reusing prior entries' module universe), not a raw
                            // `resolved_graph_memo` cache-slot read (that memo is entry-scoped
                            // and evicted on completion by design, so a raw post-hoc read
                            // cannot distinguish "reused" from "inserted then evicted").
                            let resolved_graph_hit =
                                summary.entry_resolve_receipts.iter().any(|r| {
                                    !p1_cohort_seen_subjects.insert(r.closure_subject.clone())
                                });
                            for r in &summary.entry_resolve_receipts {
                                p1_cohort_seen_subjects.insert(r.closure_subject.clone());
                            }
                            (group_wall_ms, typecheck_misses_after, resolved_graph_hit)
                        } else {
                            (0, 0, false)
                        };
                    let resolve_ms = summary.total_resolve_nanos / 1_000_000;
                    let eval_ms = summary.total_measured_nanos / 1_000_000;
                    summaries.push(summary);
                    let snap = index_retention_snapshot(&index);
                    if drain_detail {
                        emit_floor_drain_group_line(
                            group_idx + 1,
                            total_groups,
                            &drain_prev,
                            &snap,
                        );
                    }
                    if p1_cohort_detail {
                        let modules_evicted = snap
                            .schedule_evictions
                            .saturating_sub(drain_prev.schedule_evictions);
                        let graphs_evicted = snap
                            .resolved_graph_evictions
                            .saturating_sub(drain_prev.resolved_graph_evictions);
                        let (cgroup_current, cgroup_peak) = p1_cohort_cgroup_memory();
                        emit_p1_cohort_entry_line(
                            group_idx + 1,
                            total_groups,
                            &group_entry_label,
                            group_wall_ms,
                            resolve_ms,
                            eval_ms,
                            Some(typecheck_misses_after) == typecheck_misses_before,
                            resolved_graph_hit,
                            modules_evicted,
                            graphs_evicted,
                            snap.peak_rss_bytes,
                            cgroup_current,
                            cgroup_peak,
                        );
                    }
                    drain_peaks = retention_snapshot_peak(&drain_peaks, &snap);
                    drain_prev = snap;
                }
                emit_floor_drain_receipt(&index, total_groups, &drain_peaks);
                return Ok(finalize_discovery_summary(
                    merge_discovery_summaries(summaries),
                    &rows,
                    deferred_rows,
                ));
            }
            let arm_shared_store = if p1_cohort_experiment_active() {
                p1_experimental_arm_shared_typed_store(spawn_target_width)
            } else {
                true
            };
            let cross_worker_store = arm_shared_store.then(new_shared_typecheck_caches);
            if floor_stream {
                eprintln!(
                    "{} [affected-set] streaming run-witnesses live across the derived schedule pool (width {}; ▎shard N, one color each)",
                    floor_ts(),
                    spawn_target_width,
                );
            }
            let queue: std::sync::Arc<Mutex<VecDeque<Vec<DiscoveryRow>>>> =
                std::sync::Arc::new(Mutex::new(
                    groups
                        .into_iter()
                        .map(|g| g.iter().map(|&i| rows[i].clone()).collect())
                        .collect(),
                ));
            let abort = std::sync::Arc::new(AtomicBool::new(false));
            let source_roots_owned = source_roots.to_vec();
            let budget_policy_for_workers = options.witness_budget_policy();
            let mut handles = Vec::with_capacity(spawn_target_width);
            for worker_ordinal in 0..spawn_target_width {
                let queue_for_worker = queue.clone();
                let abort_for_worker = abort.clone();
                let roots = source_roots_owned.clone();
                let keys = whole_tree_published_keys.clone();
                let store = cross_worker_store.clone();
                let arm_shared_for_worker = arm_shared_store;
                let style = ShardStyle {
                    shard_id: worker_ordinal,
                    shard_count: spawn_target_width,
                    color: floor_color,
                    stream: floor_stream,
                };
                handles.push(std::thread::spawn(
                    move || -> Result<Vec<DiscoverySummary>, String> {
                        let index = if arm_shared_for_worker {
                            let store = store
                                .expect("shared typed store armed but cross_worker_store missing");
                            build_multi_entry_index_with_shared_caches(&roots, store)
                        } else {
                            build_multi_entry_index(&roots)
                        };
                        let mut worker_summaries = Vec::new();
                        loop {
                            if abort_for_worker.load(Ordering::SeqCst) {
                                break;
                            }
                            let Some(group_rows) = queue_for_worker.lock().unwrap().pop_front()
                            else {
                                break;
                            };
                            match run_discovery_rows(
                                &group_rows,
                                &index,
                                execution_mode,
                                keys.clone(),
                                budget_policy_for_workers,
                                style,
                            ) {
                                Ok(summary) => worker_summaries.push(summary),
                                Err(e) => {
                                    abort_for_worker.store(true, Ordering::SeqCst);
                                    return Err(e);
                                }
                            }
                        }
                        Ok(worker_summaries)
                    },
                ));
            }
            let mut summaries = Vec::new();
            let mut first_err: Option<String> = None;
            for handle in handles {
                match handle
                    .join()
                    .map_err(|_| "discovery corpus worker thread panicked".to_string())
                {
                    Ok(Ok(worker_summaries)) => summaries.extend(worker_summaries),
                    Ok(Err(e)) | Err(e) => first_err = first_err.or(Some(e)),
                }
            }
            if let Some(e) = first_err {
                return Err(e);
            }
            // The pump exits when the queue is empty OR on abort; with no error the queue must be
            // fully drained (workers only exit early on retire/abort, and the pump re-admits while
            // items remain), so an undrained queue here is a scheduler bug — refuse, never under-run.
            let leftover = queue.lock().unwrap().len();
            if leftover > 0 {
                return Err(format!(
            "derived-schedule discovery pool exited with {leftover} undrained entry-group(s) and no \
             worker error — scheduler invariant violated; refusing a partial corpus"
        ));
            }
            Ok(finalize_discovery_summary(
                merge_discovery_summaries(summaries),
                &rows,
                deferred_rows,
            ))
        }
    };
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_discovery_rows(
    rows: &[DiscoveryRow],
    index: &MultiEntryIndex,
    execution_mode: v1_interpreter::ExecutionMode,
    whole_tree_published_keys: Option<std::collections::HashSet<String>>,
    budgets: WitnessBudgetPolicy,
    style: ShardStyle,
) -> Result<DiscoverySummary, String> {
    let mut summary = DiscoverySummary {
        total: rows.len(),
        passed: 0,
        skipped: 0,
        deferred_rows: Vec::new(),
        divergences: Vec::new(),
        failures: Vec::new(),
        witness_outcomes: Vec::with_capacity(rows.len()),
        entry_resolve_receipts: Vec::new(),
        total_resolve_nanos: 0,
        total_stage_nanos: ResolveStageNanos::default(),
        performance_receipts: Vec::new(),
        total_measured_nanos: 0,
        roster_closure_nodes: 0,
        total_entry_groups: 0,
        selected_entry_groups: 0,
    };
    // This shard's SUBJECT union closure, accumulated from the graphs it resolves as each
    // entry is loaded. It once also folded in a floor-runner prefix context, resolved before the
    // roster so the affected-set machinery was available to every row; that prefix is gone with
    // selection, so the closure is exactly the rows' own graphs.
    let mut closure_modules: HashSet<String> = HashSet::new();
    // Schedule-derived retention is armed by the CALLER over the WHOLE batch schedule
    // (Serial: the single call; Adaptive width=1: once before the entry-group loop), so a
    // shared module's refcount spans every entry that reaches it and it stays resident until
    // its genuinely-last consumer. Arming here (per `run_discovery_rows` call) would hand the
    // Adaptive inline drain a ONE-ENTRY schedule per group — refcount 1 on every module,
    // evicted the instant its entry finished — collapsing "keep shared state until last use"
    // into "cold-recompute the shared closure once per entry" (the entries=1 churn that held
    // batch-3 wall over budget while RSS was already bounded). Rows are sorted by entry, so an
    // entry's rows are contiguous and, once passed, the entry can never be read again; the
    // per-entry completion below decrements against the caller-armed refcount (a no-op when
    // unarmed — the plural/shared-store worker path).
    // The entry whose per-module state becomes unreachable once `row.entry` moves on.
    let mut schedule_prev_entry: Option<String> = None;
    let mut current_entry: Option<String> = None;
    let mut current_closure_subject: Option<String> = None;
    let mut ctx: Option<v1_interpreter::InterpContext> = None;
    let pool_roots = witness_layer_roots();
    let whole_tree_published_keys = whole_tree_published_keys.map(Rc::new);
    for row in rows {
        // Schedule-derived eviction: when the entry advances, the previous entry's
        // rows are all behind us (rows are sorted by entry), so its per-module state
        // can never be read again — drop everything no remaining entry's closure
        // reaches. A schedule underflow refuses here (typed, located).
        if schedule_prev_entry.as_deref() != Some(row.entry.as_str()) {
            if let Some(prev) = schedule_prev_entry.take() {
                // `current_closure_subject` still holds the PREVIOUS entry's subject here
                // (it is reassigned only in the resolve block below), so it keys the
                // previous entry's ResolvedGraph for eviction; None when that entry was
                // skip-before-resolved (no graph to drop).
                index_schedule_entry_completed(index, &prev, current_closure_subject.as_deref())?;
            }
            schedule_prev_entry = Some(row.entry.clone());
        }
        if current_entry.as_deref() != Some(row.entry.as_str()) {
            {
                let resolved = resolve_discovery_entry_for_corpus_row(
                    index,
                    &row.entry,
                    execution_mode,
                    whole_tree_published_keys.clone(),
                    &mut closure_modules,
                )?;
                summary.total_resolve_nanos += resolved.resolve_nanos;
                summary.total_stage_nanos.accumulate(&resolved.stage_nanos);
                summary.entry_resolve_receipts.push(EntryResolveReceipt {
                    entry: row.entry.clone(),
                    closure_subject: resolved.closure_subject.clone(),
                    resolve_nanos: resolved.resolve_nanos,
                    stage_nanos: resolved.stage_nanos,
                });
                current_closure_subject = Some(resolved.closure_subject);
                ctx = Some(resolved.ctx);
                if let Some(c) = ctx.as_ref() {
                    c.set_witness_eval_budget(budgets.cpu_eval_budget_ms);
                    c.set_witness_wall_budget(budgets.wet_receipt_wall_budget_ms);
                }
                current_entry = Some(row.entry.clone());
            }
        }
        if ctx.is_none() {
            let resolved = resolve_discovery_entry_for_corpus_row(
                index,
                &row.entry,
                execution_mode,
                whole_tree_published_keys.clone(),
                &mut closure_modules,
            )?;
            summary.total_resolve_nanos += resolved.resolve_nanos;
            summary.total_stage_nanos.accumulate(&resolved.stage_nanos);
            summary.entry_resolve_receipts.push(EntryResolveReceipt {
                entry: row.entry.clone(),
                closure_subject: resolved.closure_subject.clone(),
                resolve_nanos: resolved.resolve_nanos,
                stage_nanos: resolved.stage_nanos,
            });
            current_closure_subject = Some(resolved.closure_subject);
            ctx = Some(resolved.ctx);
            if let Some(c) = ctx.as_ref() {
                c.set_witness_eval_budget(budgets.cpu_eval_budget_ms);
                c.set_witness_wall_budget(budgets.wet_receipt_wall_budget_ms);
            }
        }
        let ctx_ref = ctx.as_ref().expect("ctx set above");
        let closure_subject = current_closure_subject
            .as_deref()
            .expect("closure subject set above");
        set_phase(
            FloorPhase::Eval,
            &format!("{}::{}", row.entry, row.function),
        );
        active_workset_admit(&row.entry, &row.function);
        let (outcome, receipt) = run_claim_measured(ctx_ref, closure_subject, &row.function);
        active_workset_complete(&row.entry, &row.function);
        let wall_nanos = receipt.wall_nanos;
        summary.total_measured_nanos += wall_nanos;
        summary.performance_receipts.push(receipt);
        let execution_leg = witness_execution_leg_label(&row.entry);
        let entry_repo_path = workspace_relative_repo_path(&row.entry);
        let module_path = index
            .module_graph_facts
            .path_to_module
            .get(&entry_repo_path)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "{}: discovery witness entry has no module identity in the live module graph (refuse; DeclarationRef cannot be fabricated)",
                    row.entry
                )
            })?;
        summary.witness_outcomes.push(DiscoveryWitnessOutcome {
            entry: row.entry.clone(),
            module_path: module_path.clone(),
            function: row.function.clone(),
            outcome: outcome.clone(),
            execution_leg: execution_leg.clone(),
        });
        // enrolled=false is a STATEMENT, not a default: this is the discovery/claim_batch path
        // and `floor_expected_red` is a required-floor roster that is not in scope here, so no
        // row on this path can be KNOWN-RED. The typed outcome still survives to the console,
        // which is the part that was being lost on both paths.
        style.stream_witness(
            &row.function,
            &module_path,
            &execution_leg,
            wall_nanos,
            CiWitnessVerdict::from_outcome(&outcome, false),
        );
        match outcome {
            ClaimOutcome::Pass => summary.passed += 1,
            ClaimOutcome::Fail => {
                let mut failure = format!("{} ({}) returned Bool(false)", row.function, row.entry);
                append_failure_receipt_companion_loudness(&mut failure, ctx_ref, &row.function);
                append_witness_verdict_diagnostic_loudness(&mut failure, ctx_ref, &row.function);
                summary.failures.push(failure);
            }
            ClaimOutcome::NotBool { got } => summary.failures.push(format!(
                "{} ({}) returned `{}`, not Bool",
                row.function, row.entry, got
            )),
            ClaimOutcome::RuntimeError { message, .. } => summary.failures.push(format!(
                "{} ({}) runtime error: {}",
                row.function, row.entry, message
            )),
            ClaimOutcome::HostToolUnresolved { name, probed } => summary.failures.push(format!(
                "{} ({}) host tool unresolved: {:?} (probed: {})",
                row.function,
                row.entry,
                name,
                probed.join(", ")
            )),
            ClaimOutcome::HostEffectRefused { operation, ground } => {
                summary.failures.push(format!(
                    "{} ({}) hermetic route has no arm for {}: {}",
                    row.function,
                    row.entry,
                    operation,
                    hermetic_effect_ground_label(&ground)
                ))
            }
            // BOTH BUDGET ARMS RENDER THROUGH `budget_figure_phrase` AND NEITHER SPELLS ITS
            // OWN SENTENCE. This site used to hand-write `cost is at least {n}ms against a
            // {budget}ms budget`, which is the bound-in-the-cost-field defect that renderer
            // exists to remove; keeping a local format string here would let this transport
            // disagree with the floor's about one outcome, which has happened before.
            ClaimOutcome::BudgetInterrupted { .. } | ClaimOutcome::CompletedOverBudget { .. } => {
                summary.failures.push(format!(
                    "{} ({}) {}",
                    row.function,
                    row.entry,
                    // SAFE BY CONSTRUCTION: the two arms matched here are exactly the two the
                    // renderer answers `Some` for. The fallback text is unreachable and says so
                    // rather than fabricating a figure.
                    outcome
                        .budget_figure_phrase()
                        .unwrap_or_else(|| "budget outcome carried no figure".to_string())
                ))
            }
            // THIS PATH DOES NOT STOP THE LINE ON AN UNWIND THE WAY THE REQUIRED FLOOR DOES, and
            // the difference is deliberate rather than an oversight: discovery runs rows across
            // worker threads with no single ordered fold to halt, and a `NotAttempted` population
            // here would have no ledger to be published into. What it does have is the same
            // obligation not to render an unwind as a witness answering false, so it goes to
            // `failures` naming what happened. The floor's stronger treatment is the floor's.
            ClaimOutcome::Panicked { payload } => summary.failures.push(format!(
                "{} ({}) PANICKED during evaluation: {}. The host unwound, so this is not a \
                 verdict and not a runtime error the evaluator raised.",
                row.function, row.entry, payload
            )),
            // Never produced on this path — nothing here mints not-attempted rows — and named
            // rather than wildcarded so a future producer cannot arrive silently.
            ClaimOutcome::NotAttempted { halted_by } => summary.failures.push(format!(
                "{} ({}) was published as not-attempted behind {}, which this path never mints",
                row.function, row.entry, halted_by
            )),
        }
    }
    // Per-shard input-size receipt: distinct modules in THIS shard's union closure, counted from the
    // graphs resolved above rather than from the thread's typecheck-miss counter (see the field doc
    // on `DiscoverySummary::roster_closure_nodes` for why the counter is not bounded to this window).
    // The final entry's rows are done — its state is now unreachable too.
    if let Some(prev) = schedule_prev_entry.take() {
        index_schedule_entry_completed(index, &prev, current_closure_subject.as_deref())?;
    }
    if let Some(ctx) = ctx.as_ref() {
        let stats = ctx.interner_stats_snapshot();
        eprintln!(
            "[floor-symbol-retention] canonical_entries={} retained_spelling_bytes={} spelling_cap_bytes={}",
            stats.canonical_entries,
            stats.canonical_retained_spelling_bytes,
            stats.canonical_spelling_cap_bytes,
        );
    }
    summary.roster_closure_nodes = closure_modules.len();
    Ok(summary)
}

pub fn floor_prepared_subject_exclusions() -> Vec<String> {
    vec![
        "test/fixture/meta_exec_confinement_scan/".to_string(),
        "test/manual/ownership_movable_test.dag".to_string(),
        // WET RECEIPT, AND IT HAS NO CI CONSUMER TODAY — stated plainly rather than dressed up
        // as an enrollment. case4_expansion_carrier_splices dispatches a real jq through
        // jq.Process.RunWithStdin, which carries no mock_response, so the hermetic floor refuses
        // it and one refusing member fails the run. Mocking is not the repair: the witness exists
        // to prove that a REAL jq exits 0 only on two argv words rather than one concatenated
        // one, and a mocked dispatch passes it without any process running.
        //
        // The first attempt at this fix added rows to gunbc.ci_layer_roots
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
        "test/manual/process_argv_expansion_receipt_test.dag".to_string(),
        // Wet receipt for command_runner's transport-agnostic run site (the LocalExec/SshExec
        // match collapsed onto command_over_transport). Excluded for the same reason as the line
        // above and not a new class: hermetic evaluation replays an operation's declared
        // mock_response and never constructs an argv, so "the words reached the process unsplit"
        // is not observable hermetically. Enrolling it would assert the mock rather than the
        // behaviour -- specification without execution.
        "test/manual/command_runner_local_argv_receipt_test.dag".to_string(),
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

/// ONE RENDERING FOR ONE STATE, for every numeric field of the heartbeat sample. A reading that
/// would not read is the sentinel; a reading that read is its number. There is no third answer,
/// and no field renders itself, so "fabricate a zero for this one field" is a change to a line
/// that names the field -- which is what makes it detectable rather than a silent substitution.
pub(crate) fn floor_sampled_field(v: Option<u64>) -> String {
    match v {
        Some(n) => n.to_string(),
        None => FLOOR_SAMPLE_UNREADABLE.to_string(),
    }
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

pub(crate) fn floor_resource_sample(cpu_baseline_ms: u64) -> String {
    // ONE SENTINEL FOR ONE STATE. The cgroup and vmstat readers below answer `na` when their
    // file will not read; these three answered a fabricated `0`, so `rss_kb=0` rendered
    // identically whether the process held no resident pages -- which a live process cannot --
    // or `/proc/self/statm` was unreadable. Those are different facts with opposite remedies,
    // and this is the instrument that exists to settle a memory contradiction, so a zero it
    // invented is worse here than anywhere else in the line. Same `na` convention now, and the
    // cpu pair is all-or-nothing because a half-read stat cannot be summed.
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
    let na = || FLOOR_SAMPLE_UNREADABLE.to_string();
    let cpu_ms = floor_sampled_field(match (tick(11), tick(12)) {
        (Some(utime), Some(stime)) => {
            Some(((utime + stime) * 1000 / hz).saturating_sub(cpu_baseline_ms))
        }
        _ => None,
    });
    let majflt = floor_sampled_field(tick(9));
    let rss_kb = floor_sampled_field(floor_statm_rss_kb());
    // THE CGROUP CHARGE AND THE THROTTLE EVENTS, on every beat, because the runs that most need
    // them are the ones that never reach an exit line. `floor_cgroup_envelope` reports the full
    // picture at entry; these three carry the parts that CHANGE, so a killed run still leaves
    // behind what its envelope was doing when it died.
    //
    // rss_kb and cur_kb are DIFFERENT COUNTERS and are printed side by side so they are never
    // silently substituted for one another: RSS is this process's resident anonymous + mapped
    // pages, while memory.current is the cgroup's total charge including page cache and every
    // other process in it. Comparing RSS against a cgroup limit is what produced the standing
    // contradiction this instrument exists to settle — a CI run observed at 14.69 GiB RSS,
    // above a declared 14.00 GiB max, that was not killed and ran 151 minutes more.
    //
    // ev_high/ev_max are the reclaim and kill counters for THIS level. Nonzero ev_high is
    // throttling actually happening rather than inferred from a declared row; both zero beside
    // a death means the ceiling that killed it was somewhere else.
    //
    // READ THE LEAF, NOT THE ROOT. These three fields first shipped reading
    // `/sys/fs/cgroup/{name}` directly, and on CI they printed `na` on every single beat for a
    // four-hour run: the runner's leaf is
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
    let cg = |name: &str, idx: usize| -> String {
        std::fs::read_to_string(format!("{}/{name}", floor_cgroup_dir()))
            .ok()
            .and_then(|s| {
                if idx == usize::MAX {
                    s.trim().parse::<u64>().ok().map(|v| (v / 1024).to_string())
                } else {
                    s.lines()
                        .nth(idx)
                        .and_then(|l| l.split_whitespace().nth(1).map(|v| v.to_string()))
                }
            })
            .unwrap_or_else(na)
    };
    let cur_kb = cg("memory.current", usize::MAX);
    let ev_high = cg("memory.events", 1);
    let ev_max = cg("memory.events", 2);
    // The LOCAL counter beside the hierarchical one, on every beat. `memory.events` at this
    // level already includes everything its descendants generated, so `ev_high` rising says
    // "something in this subtree was throttled" and cannot say it was us. `ev_local_high` is
    // the same event restricted to this exact cgroup, and the pair is the only way to separate
    // our own reclaim from a neighbour's. Carried per beat rather than only in the periodic
    // envelope because a killed run keeps only what was already printed, and this is the field
    // the neighbour-pressure hypothesis is decided on.
    let ev_local_high = cg("memory.events.local", 1);
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
    let vm = |key: &str| -> String {
        std::fs::read_to_string("/proc/vmstat")
            .ok()
            .and_then(|s| {
                s.lines().find_map(|l| {
                    l.strip_prefix(key)
                        .and_then(|r| r.strip_prefix(' '))
                        .map(|v| v.trim().to_string())
                })
            })
            .unwrap_or_else(na)
    };
    let pswpin = vm("pswpin");
    let pgmajfault = vm("pgmajfault");
    format!(
        "cpu_ms={cpu_ms} rss_kb={rss_kb} majflt={majflt} cur_kb={cur_kb} \
         ev_high={ev_high} ev_max={ev_max} ev_local_high={ev_local_high} \
         pswpin={pswpin} pgmajfault={pgmajfault}"
    )
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

/// One entry's enrolled witness names, exactly as `v2.workflow.floor_discovery_producer`
/// answered them; the site-projection loop's unit.
struct FloorDiscoveryFile {
    path: String,
    module_path: String,
    functions: Vec<String>,
}

/// The `.dag` module whose per-file fold IS the required floor's roster. Evaluated by qualified
/// name in its own exact scope, like every other floor authority; a closure seed of the
/// gate-bounded prepared subject (`REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES`).
const FLOOR_DISCOVERY_AUTHORITY_MODULE: &str = "v2.workflow.floor_discovery_producer";

pub fn floor_seam(name: &str) {
    if let Ok(mut g) = FLOOR_SEAM.lock() {
        g.clear();
        g.push_str(name);
    }
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
pub(crate) fn install_pure_producer_share(
    prepared: &PreparedRepository,
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
    let warm_rows = floor_decode_module_prefix_roster(
        &roster_frame,
        &format!("{FLOOR_PURE_PRODUCER_SHARE_MODULE}.floor_cross_claim_pure_producers_warm"),
    )?;
    let claim_forced_rows = floor_decode_module_prefix_roster(
        &roster_frame,
        &format!(
            "{FLOOR_PURE_PRODUCER_SHARE_MODULE}.floor_cross_claim_pure_producers_claim_forced"
        ),
    )?;
    // Admission is by RESOLVED DECLARATION IDENTITY (review 57446 F1): each qualified
    // roster spelling resolves to its fn node in a frame over the prepared subject, and the
    // interpreter admits by that node set — a bare-name homonym in a non-rostered module is
    // a different node and never eligible. A row whose module or declaration the subject
    // cannot resolve is a stale row and stops the line, exactly like a stale warm row.
    let mut resolution_frames: std::collections::HashMap<String, v1_interpreter::InterpContext> =
        std::collections::HashMap::new();
    let mut admitted_nodes = Vec::new();
    for qualified in warm_rows.iter().chain(claim_forced_rows.iter()) {
        let module = match qualified.rsplit_once('.') {
            Some((module, _)) => module.to_string(),
            None => qualified.clone(),
        };
        if !resolution_frames.contains_key(&module) {
            let frame = floor_authority_frame(prepared, &module).map_err(|why| {
                format!(
                    "REQUIRED-FLOOR REFUSAL cause=PureProducerShareProducerModuleOutsideSubject \
                     producer={qualified} — the rostered producer's module is not in the \
                     prepared subject; delete the stale roster row or restore its consumers: \
                     {why}"
                )
            })?;
            resolution_frames.insert(module.clone(), frame);
        }
        let node = resolution_frames[&module]
            .lookup_fn_node(qualified)
            .ok_or_else(|| {
                format!(
                    "REQUIRED-FLOOR REFUSAL cause=PureProducerShareProducerUnresolved \
                     producer={qualified} — the rostered spelling names no declaration in its \
                     module's frame; fix or delete the roster row"
                )
            })?;
        admitted_nodes.push(node);
    }
    let refused = floor_decode_refused_share_candidates(
        &roster_frame,
        &format!("{FLOOR_PURE_PRODUCER_SHARE_MODULE}.floor_cross_claim_refused_candidates"),
    )?;
    PURE_PRODUCER_SHARE_ROSTER.with(|r| {
        *r.borrow_mut() = Some(PureProducerShareRoster {
            admitted_qualified: warm_rows
                .iter()
                .chain(claim_forced_rows.iter())
                .cloned()
                .collect(),
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
    for qualified in &warm_rows {
        let module = match qualified.rsplit_once('.') {
            Some((module, _)) => module.to_string(),
            None => qualified.clone(),
        };
        // Resolution above already refused any row whose module the subject no longer
        // carries, so the frame is present; reuse it rather than re-preparing the module.
        let producer_frame = &resolution_frames[&module];
        // PROVENANCE IS DERIVED FROM THE TYPED OUTCOME, NOT ASSERTED BEFORE THE CALL, and the
        // first revision of this line got that wrong in the direction DESIGN section 4b names.
        // It passed `already_built: false` unconditionally, on the reasoning that the outcome
        // below is the authority for whether the value was already retained. THAT REASONING
        // FAILS BECAUSE THIS LOOP ALSO REPORTS A PROVENANCE: on the `AlreadyPresent` path the
        // receipt said `BuiltByPreparation` for an artifact preparation FOUND rather than built.
        // Two representations of one fact with one of them lying is worse than either alone, and
        // a fabricated provenance in a receipt is the fabricated-plausible-output failure applied
        // to this compiler's own self-description (review 59035, codex/gpt-5.6-sol).
        //
        // The flag cannot carry it: `observe_shared_build` is told before it runs, and the fact
        // does not exist until the call returns. So the observation is corrected AFTER the fact,
        // from the outcome that owns it.
        //
        // THE TRIGGER NAME STATES ONLY WHAT IS DECIDABLE. `AlreadyPresent` establishes PRESENCE
        // and not who caused it, so the label names the boundary that is knowable rather than
        // fabricating a call site -- inside this loop the only writer that can already have
        // stored a rostered producer's value is an earlier rostered producer whose traversal
        // reached it. That is the same discipline `warm_bare_reference_edge_index` uses when it
        // names `a-site-ahead-of-floor-preparation` instead of inventing an author, and it is
        // deliberately weaker than a call-site name because a call site is not recorded.
        let (warm_result, mut warm_observation) =
            observe_shared_build(false, "floor-preparation", || {
                v1_interpreter::warm_cross_claim_pure_producer(producer_frame, qualified)
            });
        if let Ok(outcome) = &warm_result {
            if matches!(
                outcome,
                v1_interpreter::CrossClaimStoreOutcome::AlreadyPresent
            ) {
                warm_observation.provenance = SharedBuildProvenance::AlreadyWarmOnEntry {
                    triggered_by: "an-earlier-rostered-producer-in-this-warm-loop",
                };
            }
        }
        match warm_result {
            Ok(outcome) => {
                // A NON-SERVABLE outcome means nothing is retained for later claims, so a
                // silent decline would relocate the fill onto the first toucher: stop the
                // line, naming the ONE cause rather than a disjunction of three. An
                // `AlreadyPresent` outcome is servable and therefore not a refusal — a
                // rostered producer reachable from an earlier rostered producer is stored
                // by that traversal, and its own warm correctly finds the work done.
                if !outcome.is_servable() {
                    // The located detail comes from the OUTCOME, so a cause can only ever be
                    // paired with its own evidence. Reading the retained slot here instead
                    // would decorate a byte-budget or entry-cap refusal with a stale path
                    // left by an earlier producer's unportable value (review 57554).
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
                        "REQUIRED-FLOOR REFUSAL cause=PureProducerShareWarmNotStored \
                         producer={qualified} — the rostered producer evaluated but its value \
                         was refused by the cross-claim store: {detail}"
                    ));
                }
                eprintln!(
                    "[floor-phase] phase=pure-producer-share-warm state=completed \
                     producer={qualified} disposition={} cpu_ms={} wall_ms={} \
                     rss_growth_bytes={} provenance={}",
                    outcome.cause(),
                    warm_observation.cpu_ms,
                    warm_observation.wall_ms,
                    warm_observation.rss_growth_bytes,
                    warm_observation.provenance.render(),
                );
                warm_observations.push((
                    format!("CrossClaimPureProducerWarm/{qualified}"),
                    warm_observation,
                ));
            }
            Err(why) => {
                return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=PureProducerShareWarmFailed \
                     producer={qualified} — {why}"
                ));
            }
        }
    }
    Ok(warm_observations)
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

thread_local! {
    /// Set by `install_pure_producer_share` and read once the fold is over. Thread-local for the
    /// same reason the shared-fill ledger is: it is the SAME thread's observation, and a state
    /// that could be written by one thread and read by another would let the wall adjudicate a
    /// roster that never governed the fills it is reading.
    static PURE_PRODUCER_SHARE_ROSTER: std::cell::RefCell<Option<PureProducerShareRoster>> =
        const { std::cell::RefCell::new(None) };
}

/// Decode the refused roster. Every arm refuses: a missing declaration, a non-record row, a
/// wrong record type, a non-String field, an unknown verdict variant. An unknown variant is
/// deliberately NOT tolerated — a fourth verdict arrives with a meaning this wall does not know
/// how to weigh, and treating it as "some refusal" would let the authority claim a judgement the
/// executor cannot perform.
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
                    | "SupersededBySingleAuthorityRepair" => name,
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

/// THE OVERLAP WALL, run once the fold is over and the shared-fill ledger is final.
///
/// The identity-grain check lives in the `.dag` and refuses a producer that is admitted and
/// refused at once. It is not sufficient, and this file's own roster contains the case it
/// misses: `rust_target_model_staging` measured clean while `rust_target_model` was enrolled,
/// then INHERITED that row's consumers and its regression the moment the wider row was
/// withdrawn. A distinct identity reaching the same measured neighbourhood is the same refusal
/// arriving by a second name, so the join is on the OBSERVED carriers, not on the spelling.
///
/// WHOSE RUN THIS FIRES ON, STATED BECAUSE IT IS NOT THE OBVIOUS ONE. The overlap can be created
/// by a diff that touches neither the admitted key nor its cost: withdrawing a wide row hands
/// its consumers to a narrower one, and the refusal then lands on the WITHDRAWER's run. That is
/// an externalized cost (DESIGN §5) unless the diagnostic says so, so it names three things —
/// the admitted key that now overlaps, the refused row it inherited the carriers from, and that
/// the trigger was a roster change rather than that key's own cost.
fn refuse_pure_producer_share_refused_carrier_overlap() -> Result<(), String> {
    const CROSS_CLAIM_SHARE_CACHE: &str = "cross_claim_pure_share";
    let observed = crate::cli_run::shared_fill::consumer_modules_by_key(CROSS_CLAIM_SHARE_CACHE);
    let roster = PURE_PRODUCER_SHARE_ROSTER.with(|r| r.borrow().clone());
    let roster = match roster {
        Some(roster) => roster,
        // NOT A SKIP. No roster installed and no fill observed is a lane that never entered the
        // share tier; no roster installed WITH fills observed means something filled the tier
        // outside the install this wall reads, and the wall would be adjudicating one run's
        // fills against another run's roster.
        None if observed.is_empty() => return Ok(()),
        None => {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareLedgerWithoutRoster keys={} — the \
                 cross-claim share ledger recorded fills but no roster was installed on this \
                 thread, so the refused-carrier join has no population to adjudicate",
                observed.len()
            ));
        }
    };
    // THE DOMAIN IS ASSERTED BEFORE THE VERDICT IS READ. An empty intersection is evidence only
    // when both operands are known non-empty, and this wall was itself built on a green that was
    // the identity element on an empty domain: the pre-merge join I ran by hand captured
    // `consumer_modules=53` instead of `modules=a,b,c` — the count, not the set — so every
    // intersection was empty and every key read clean while a real overlap stood in the same
    // ledger. The refused side is asserted at decode (a carrierless row refuses there); this is
    // the observed side. A SINGLE fill legitimately carries no modules — one filled outside the
    // fold and read by nobody has neither filler nor consumer — so the assertion is over the
    // whole ledger: fills recorded and not one module observed anywhere is an observation defect,
    // never a clean run. This is the rostered `predicate_vacuously_true_on_an_empty_domain`.
    if !observed.is_empty() && observed.iter().all(|(_, modules)| modules.is_empty()) {
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=PureProducerShareObservedCarriersVacuous keys={} — the \
             cross-claim share ledger recorded fills but not one of them carries a consuming \
             module, so the refused-carrier join would intersect against an empty domain and \
             report clean without comparing anything",
            observed.len()
        ));
    }
    // Bare ledger key -> the admitted spellings that end in it. The interpreter bills the share
    // ledger under the BARE function name, and the roster's whole admission discipline is that
    // identity is the resolved declaration, never the bare name — so an overlapping key whose
    // bare name is claimed by two admitted rows cannot be attributed, and naming either one
    // would be a fabricated subject.
    let mut by_bare: std::collections::BTreeMap<&str, Vec<&String>> =
        std::collections::BTreeMap::new();
    for qualified in &roster.admitted_qualified {
        let bare = qualified
            .rsplit_once('.')
            .map_or(qualified.as_str(), |(_, b)| b);
        by_bare.entry(bare).or_default().push(qualified);
    }
    for (key, modules) in &observed {
        for row in &roster.refused {
            // MIRROR of `v2.workflow.floor_pure_producer_share` `refused_row_carriers_transfer`,
            // which is the authority for WHICH verdicts transfer through carriers and carries the
            // reasoning. A verdict that measured NO effect has nothing for a later identity to
            // inherit, and a §3 supersession is a ruling about one spelling; refusing another
            // producer for reaching their modules would charge it with a harm the measurement
            // never found. Both rows the inheritance actually happened through are
            // MeasuredServeAboveRecompute and stay in this population. The match is exhaustive
            // over the decoded arms, so a fourth verdict cannot join by default — and the decode
            // refuses an arm it does not know before this point is reached.
            let carriers_transfer = match row.verdict.as_str() {
                "MeasuredServeAboveRecompute" => true,
                "NoMeasuredEffectOverItsConsumers" => false,
                "SupersededBySingleAuthorityRepair" => false,
                other => {
                    return Err(format!(
                        "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRefusalVerdictUnknown \
                         producer={} verdict={other} — the overlap wall cannot decide \
                         whether this verdict's carriers transfer",
                        row.producer
                    ));
                }
            };
            if !carriers_transfer {
                continue;
            }
            // THE TRIGGER IS COMPUTATION IDENTITY, NOT CO-LOCATION. An earlier revision of this
            // wall refused on module-set intersection alone, and that fabricates causal
            // equivalence from shared neighbourhood: two producers using one consuming module
            // establishes neither one computation identity nor any transfer of a measured cost,
            // so ANY unrelated admitted producer reaching that module could stop the required
            // floor. DESIGN §2 requires that one COMPUTATION IDENTITY join the demands to a
            // provider, and §4 refuses a heuristic where the richer source exists — it does
            // here, because the interpreter bills this ledger under the producing function's own
            // key. So the refusal fires when the ledger observes a fill under the BARE NAME OF A
            // REFUSED PRODUCER: that is the withdrawn computation recurring under a spelling the
            // static fold cannot see, since a transitively reached producer is in the tier and
            // not in the roster. Carrier overlap is retained as a REQUIRED CORROBORATION and
            // reported as evidence, never as the trigger on its own.
            //
            // WHAT THIS NARROWING GIVES UP, DECLARED RATHER THAN DISCOVERED LATER. The case that
            // motivated this wall is TWO IDENTITIES SHARING ONE EFFECT: `rust_target_model_staging`
            // was refused for the same measured effect as `rust_target_model`, and its consumer set
            // grew from 17 to 44 modules on the run where the first row was withdrawn and staging
            // inherited its consumers. That is a DIFFERENT computation identity, so this wall is
            // now green on it — deliberately, because module overlap cannot establish the transfer
            // and refusing on it charges a producer with a harm nothing measured. NEXT TRIGGER,
            // NAMED AS THE CAPABILITY: a modeled produces-the-same-value relation between two
            // producer identities, so inheritance is read off a declared equivalence rather than
            // inferred from co-location. A wider module heuristic is not that capability and must
            // not retire this gap.
            let refused_bare = row
                .producer
                .rsplit_once('.')
                .map_or(row.producer.as_str(), |(_, b)| b);
            if key.as_str() != refused_bare {
                continue;
            }
            let shared: Vec<&str> = modules
                .intersection(&row.carrier_modules)
                .map(|m| m.as_str())
                .collect();
            if shared.is_empty() {
                continue;
            }
            let subject = match by_bare.get(key.as_str()).map(|v| v.as_slice()) {
                Some([one]) => (*one).clone(),
                Some(many) => {
                    return Err(format!(
                        "REQUIRED-FLOOR REFUSAL cause=PureProducerShareOverlapSubjectAmbiguous \
                         key={key} candidates={} — this key's fills overlap the carriers of \
                         refused row {}, and the bare ledger key is claimed by more than one \
                         admitted spelling, so the overlap cannot be attributed to an identity",
                        many.iter()
                            .map(|q| q.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                        row.producer
                    ));
                }
                // Reachable and NOT an error: the store admits producers reached transitively
                // from a rostered one, so a key with no roster spelling is a real member of the
                // tier. It is named by its ledger key, which is what the operator can grep.
                _ => format!("<reached-from-roster>:{key}"),
            };
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=PureProducerShareRefusedCarrierOverlap \
                 admitted_key={key} admitted_producer={subject} refused_row={} \
                 refused_verdict={} shared_carrier_modules={} — THIS LEDGER KEY IS THE REFUSED \
                 PRODUCER'S OWN COMPUTATION IDENTITY, recurring under a spelling the roster \
                 does not name, and the shared modules corroborate that its measured carriers \
                 came with it. WHAT CHANGED IS THE ROSTER, NOT THIS KEY'S OWN COST. The named refused row was measured over those modules and \
                 withdrawn; this admitted key is now serving into the same measured \
                 neighbourhood, which is how a narrower identity inherits a withdrawn row's \
                 regression without any measurement of its own moving. Either re-measure this \
                 key over those consumers and carry the result, or withdraw it: \
                 v2.workflow.floor_pure_producer_share names the run pair and the next trigger \
                 on the refused row",
                row.producer,
                row.verdict,
                shared.join(",")
            ));
        }
    }
    Ok(())
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
/// from the DECLARED rows in `gunbc.runner_slot_allocation`: a nominal limit, read before the
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

pub fn run_required_floor(
    source_roots: &[String],
    commit: &str,
    style: ShardStyle,
) -> Result<RequiredFloorOutcome, String> {
    // HONEST SCOPE (review 53487): the caller marker below is a self-attested string, not
    // authentication — any caller able to set `_ONLY` can set `_ONLY_CALLER` too. What it
    // buys is exactly one thing: the witnesses CI env cannot gain `_ONLY` by a one-variable
    // edit or copy-paste; flipping the primary floor into join-only mode now takes a second,
    // deliberately named variable whose value documents where the mode is allowed to come
    // from. It is a tripwire against accident, not a wall against intent. The whole gate
    // dissolves with the `expected_red_roster_join` bin (registered scaffold) when the floor
    // emits the join report by default.
    const EXPECTED_RED_ROSTER_JOIN_ONLY_BIN: &str = "expected_red_roster_join_bin";
    let roster_join_only_requested = std::env::var("GUNBC_EXPECTED_RED_ROSTER_JOIN_ONLY")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if roster_join_only_requested
        && std::env::var("GUNBC_EXPECTED_RED_ROSTER_JOIN_ONLY_CALLER").as_deref()
            != Ok(EXPECTED_RED_ROSTER_JOIN_ONLY_BIN)
    {
        return Err(
            "REQUIRED-FLOOR REFUSAL cause=ExpectedRedRosterJoinOnlyUnauthorized — \
             GUNBC_EXPECTED_RED_ROSTER_JOIN_ONLY is admitted only from the \
             expected_red_roster_join stop-line-audit bin; witnesses CI must set \
             GUNBC_EXPECTED_RED_ROSTER_JOIN without _ONLY"
                .to_string(),
        );
    }
    floor_cgroup_envelope("floor-entry");
    spawn_floor_heartbeat();
    floor_seam("strict-preparation");
    eprintln!("[floor-phase] phase=strict-preparation state=started");
    // ── 1. read once, prepare once ────────────────────────────────────────────────────────
    set_phase(FloorPhase::Resolve, "required-floor preparation");
    let prepare_started = std::time::Instant::now();
    // THE ROSTER THAT BOUNDS PREPARATION IS READ BEFORE PREPARATION, from a frame over the
    // policy module's own closure -- a few dozen modules -- so the gate's authority is the .dag
    // roster and not a Rust mirror of it. The same roster is decoded again below from the full
    // hermetic frame for site disposition; the two reads are one function in one module.
    // ONE ENTRY INDEX FOR BOTH CLOSURES: building it is the expensive part (~75-110s on the
    // 4,260-module corpus, measured 2026-08-29), so it is built once here and lent to the
    // policy-closure prepare and the gate-closure prepare alike.
    let gate_entry_index = build_multi_entry_index(source_roots);
    // ONE DERIVATION, CONSUMED TWICE. #9717's changed-witness identity producer supplies both
    // the closure seeds that make these modules executable and the tail projection that judges
    // their terminal rows. Re-observing the diff after execution would create two authorities
    // over which identities this run promised to execute.
    let changed_witnesses = match changed_witness_identities_with_index(&gate_entry_index) {
        Ok(changed) => Some(changed),
        Err(e) if commit != "local" && !commit.is_empty() => {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=ChangedWitnessObservationFailed {e} — the \
                 changed-witness execution sublane could not observe or attribute the CI diff"
            ));
        }
        Err(e) => {
            eprintln!(
                "[changed-witness] EXECUTION SUBLANE NOT EVALUATED (no CI diff baseline on a local run): {e}"
            );
            None
        }
    };
    let changed_witness_set: HashSet<String> = changed_witnesses
        .iter()
        .flat_map(|rows| rows.iter().cloned())
        .collect();
    let changed_module_seeds: BTreeSet<String> = changed_witnesses
        .iter()
        .flat_map(|rows| rows.iter())
        .map(|identity| {
            identity
                .rsplit_once('.')
                .map(|(module, _)| module)
                .unwrap_or(identity.as_str())
                .to_string()
        })
        .collect();
    // THE LANE'S SCHEDULE IS DECODED HERE, IN THE POLICY CLOSURE, AND NOT LATER -- because its
    // modules must become SEEDS of the prepared subject below. The first CI run of this lane
    // refused seven times with `EntryModuleOutsidePreparedSubject`: the prepared graph is the
    // required-gate closure plus the changed set, and the lane's members are on the discovery
    // exclusion frontier, so nothing pulled them in. A lane that cannot reach its own members
    // cannot support the route claim `std.witness_admission` makes for its cadence, so the
    // schedule joins the seed list rather than the executor learning to run outside the subject.
    let (required_gate_prefixes, local_repo_wet_schedule_rows) = {
        let policy_seed = [REQUIRED_FLOOR_POLICY_MODULE.to_string()];
        let (policy_prepared, _) = prepare_repository_closure(
            source_roots,
            &floor_prepared_subject_exclusions(),
            Some((&gate_entry_index, &policy_seed)),
        )?;
        let policy_scope = claim_scope_for(&policy_prepared, REQUIRED_FLOOR_POLICY_MODULE)?;
        let policy_frame = evaluation_frame(
            &policy_scope,
            v1_interpreter::ExecutionMode::Hermetic,
            None,
            None,
        );
        let prefixes = floor_decode_module_prefix_roster(
            &policy_frame,
            "v2.workflow.required_floor.required_gate_prefixes",
        )?;
        let schedule = local_repo_wet_schedule(&policy_frame)?;
        (prefixes, schedule)
    };
    // THE FLOOR'S OWN AUTHORITIES ARE ALWAYS IN THE SUBJECT: the floor evaluates its rosters
    // (expected red, route gap, cost debt, the gate itself) in a frame over the prepared graph,
    // and a gate roster that happened not to reach `v2.workflow.required_floor` refused with
    // EntryModuleOutsidePreparedSubject (measured 2026-08-29). They are seeds, not gate rows:
    // their own witnesses are admitted or declined by the roster like every other module's.
    //
    // The roster names every module the floor's Rust reaches BY NAME outside the gate roster.
    // Under the whole-tree subject these resolved by pool-membership coincidence -- the
    // flat bare-name channel found `resolve_channel_policy` because the whole corpus was
    // loaded, not because anything in the policy closure references `gunbc.output_policy`
    // (measured 2026-08-29: the first gate-bounded run refused at output-policy install with
    // "no declaration named 'resolve_channel_policy' in this execution's loaded index").
    // Making the dependency a declared seed is the honest form; a module evaluated by name
    // and absent from this list refuses loudly at its own call site, never silently.
    let closure_seeds: Vec<String> = required_gate_prefixes
        .iter()
        .cloned()
        .chain(
            REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES
                .iter()
                .map(|m| m.to_string()),
        )
        .chain(changed_module_seeds.iter().cloned())
        .chain(
            local_repo_wet_schedule_rows
                .iter()
                .map(|row| row.entry_module.clone()),
        )
        .collect();
    let (mut prepared, prepared_sources) = prepare_repository_closure(
        source_roots,
        &floor_prepared_subject_exclusions(),
        Some((&gate_entry_index, &closure_seeds)),
    )?;
    drop(gate_entry_index);
    // THE FULL INDEX THE DISCOVERY AUTHORITY WILL JUDGE, captured here because the prepared
    // graph is intentionally only the required gate closure. Declaration discovery is a
    // corpus-wide question: fold the one modeled producer over every indexed source, finalize
    // once, then classify rows by whether preparation admitted their module.
    // `prepared_sources` is moved into the guard on the next line. Rc clones of what preparation
    // already holds -- path, module and bytes -- never a second corpus.
    let prepared_module_paths: HashSet<String> = prepared_sources
        .iter()
        .map(|view| view.module_path.clone())
        .collect();
    // BOUNDED RETENTION, NOT A CORPUS COPY HELD FOR THE CLAIM RUN. The full-index views are
    // taken OUT of `prepared` here and consumed, by value, inside the discovery-authority phase
    // below — no second vector is built from them, and the phase drops them before the claim
    // roster exists. Taking them out of `prepared` makes the lifetime structural: the
    // outside-closure source bytes cannot survive into claim execution through the repository
    // value every later phase borrows, and the discovery phase's own completion line measures
    // the release (review 57430).
    let discovery_exclusions = std::mem::take(&mut prepared.discovery_exclusions);
    let full_inventory = std::mem::take(&mut prepared.full_inventory);
    let _floor_prepared_guard = register_floor_prepared_authority_guard(prepared_sources);
    // WARM THE MODULE-PATH INDEX HERE, because otherwise ONE ARBITRARY CLAIM PAYS FOR IT.
    //
    // `compile_dag_rust_emit_check` (the emit witnesses' host arm) calls
    // `build_module_path_index_from_witness_roots`, which walks and PARSES every module under
    // the default source roots. It is thread-local cached, so exactly one claim per process
    // pays the build and every later one hits the cache. That claim is then billed ~45s
    // against a 1552ms per-claim ceiling and reports as a budget failure, while its identical
    // siblings run in ~760ms.
    //
    // MEASURED, three runs, same rows, evaluation order alphabetical and stable throughout:
    //
    //   run                     emitted_lib_rs...omits    emitter_nested...single
    //   pre-quarantine              739ms                     761ms
    //   32189985063               42647ms  <- billed           687ms
    //   32193032348 (main)        quarantined                45941ms  <- billed
    //
    // The bill is POSITIONAL, not a property of any witness: quarantining the victim hands it
    // to the next module in evaluation order. Three modules were quarantined down this chain
    // (dissolution_census 55.5s, emitted_lib_rs 42.6s, and the 83.0s
    // extdeps_scope_placement_gate row) before the pattern was read correctly -- each read as
    // a slow test, all three were the same one-time build landing on whoever touched it first.
    //
    // Paying it in preparation is where the cost BELONGS: it is a fact of the subject, not of
    // any claim, and the floor's whole design is one preparation serving every claim. Total
    // run wall is unchanged -- the same work happens once either way; what changes is that no
    // claim is charged for building the subject it was handed.
    //
    // dissolve-on: the index derives from the prepared inventory instead of a second disk
    // walk. The bytes are already in hand -- the emit memo is keyed on
    // `floor_inventory_content_digest` precisely because that index reads the same files --
    // and `languages_decl_records_from_inventory` is the existing precedent for the
    // inventory-sourced form of a census that used to scan. When that lands this warm call is
    // unnecessary rather than merely redundant, because there is no second authority to warm.
    // MEASURED AND ADJUDICATED like the edge-index warm below, because `FloorPreparationPhase`
    // declares this phase protected and a declared wall that nothing enforces is worse than an
    // absent one — it is cited as coverage (review 55338).
    let (warmed_modules, module_path_index_warm) =
        observe_shared_build(false, "floor-preparation", || {
            build_module_path_index_from_witness_roots().len()
        });
    eprintln!(
        "[floor-phase] phase=module-path-index-warm state=completed cpu_ms={} wall_ms={} \
         rss_growth_bytes={} modules={} provenance={}",
        module_path_index_warm.cpu_ms,
        module_path_index_warm.wall_ms,
        module_path_index_warm.rss_growth_bytes,
        warmed_modules,
        module_path_index_warm.provenance.render(),
    );
    // WARM THE SHARED MultiEntryIndex HERE, for the same reason as the module-path index
    // above: otherwise ONE ARBITRARY CLAIM PAYS FOR IT (witness cost class 2).
    //
    // `commit_witness_claim_pair_resolvable` (the commit-witness roster's host arm, and its
    // sibling `ci_floor_commit_witness_claim_pairs` / `commit_witness_claim_roster_defects`)
    // used to call `build_multi_entry_index(&roots)` directly instead of going through
    // `process_shared_index`, the existing thread-local shared-index cache every other
    // production call site already uses (`resolve_entry_graph_shared` and friends). That
    // meant every one of those functions re-walked and re-parsed the whole witness corpus on
    // EVERY call, uncached — even though `process_shared_index` would have served the same
    // `MultiEntryIndex` for free after the first build.
    //
    // MEASURED: `commit_witness_claim_pair_resolvable`, invoked exactly once by
    // `commit_witness_claim_roster_red_control_holds` (the fast-lane RED control for the
    // synthetic stale (entry, function) pair), cost 62.7-83.0s for that single call — the
    // full one-time corpus walk+parse billed against one witness with a ~1-5s ceiling. As
    // with the module-path index, quarantining the victim does not remove the cost, it only
    // relocates it onto whichever claim runs next.
    //
    // The three call sites were switched to `process_shared_index(&roots)` (this fix); this
    // warm call additionally ensures the shared cache is already hot before the per-claim
    // loop starts, so the cost is paid here, in preparation, rather than by whichever claim
    // happens to touch it first. `witness_layer_roots()` is the exact roots value all three
    // call sites resolve internally (not the `source_roots` parameter above), so the roots
    // key here must match theirs; `canonical_shared_index_roots` normalizes relative and
    // absolute forms to the same key regardless.
    //
    // dissolve-on: same as the module-path-index warm above — when the shared index derives
    // from the prepared inventory instead of a second disk walk, this warm call becomes
    // unnecessary rather than merely redundant, because there is no second authority to warm.
    let (warmed_shared_index_modules, shared_index_warm) =
        observe_shared_build(false, "floor-preparation", || {
            process_shared_index(&witness_layer_roots())
                .source_files
                .len()
        });
    eprintln!(
        "[floor-phase] phase=shared-index-warm state=completed cpu_ms={} wall_ms={} \
         rss_growth_bytes={} modules={} provenance={}",
        shared_index_warm.cpu_ms,
        shared_index_warm.wall_ms,
        shared_index_warm.rss_growth_bytes,
        warmed_shared_index_modules,
        shared_index_warm.provenance.render(),
    );
    // ── SHARED-BUILD ATTRIBUTION: the bare-reference edge index ───────────────────────────
    //
    // THIRD WARM, SAME REPAIR AS THE TWO ABOVE. The bare-reference edge index
    // (`both_closure_edge_index`, and through it `tree_bare_census_for_root` per root) is a fact
    // of the SUBJECT, not of any claim: memoized once per index — the census trace reports two
    // misses over two source roots against ONE index address, `edge_index_construction {
    // builds: 1 }` — so the work is already done exactly once per process. It was simply BILLED
    // to whichever claim resolved an entry first:
    // `test.claim.qualified_spelling_identity_witness_test.qualified_spelling_takes_the_shared_layer`
    // at 57193ms CPU against a 5000ms per-claim limit, while its sibling reaching the identical
    // computation milliseconds later measured 5ms.
    //
    // WHAT THIS DOES NOT DO: it does not make the build cheaper, and nothing here claims the run
    // gets faster. The same work happens once either way; what changes is WHO IS CHARGED.
    // Quarantining a first toucher would only hand the bill to the next claim in evaluation
    // order — three modules were quarantined down exactly that chain before the pattern was read
    // correctly (docs/plans/witness-cost-first-touch-attribution.md).
    //
    // PRODUCTION PRECEDES ADJUDICATION, and the placement is deliberate in both directions. The
    // build happens HERE, as early as any consumer could reach it and ahead of the published-mock
    // projection and the output-policy install, so no earlier phase can become an accidental
    // first toucher (measured in `claim_batch`, where exactly that happened on the PRE-FIX
    // installer: the warm placed after `install_output_policy` reported `already-warm-on-entry
    // cpu_ms=0` while a ~30.8s span sat billed to an output-policy read. That installer has since
    // been scoped to the policy's own import closure and no longer enters the shared index, so
    // that specific toucher is gone — the placement rule is not, because the next one will not
    // announce itself either). The REFUSAL is adjudicated further down, at the first point where
    // `hermetic` exists to read the three `.dag` limits — so every reported outcome carries the
    // observation it was computed against, and a refusal with no observation has no spelling.
    //
    // NO PRORATION. The cost is never divided across the claims that consume the artifact: a
    // per-row fraction would change whenever the roster changes, making a row's admissibility
    // depend on how many unrelated consumers happen to be enrolled.
    //
    // Both index identities the floor's own resolves can reach. `canonical_shared_index_roots`
    // normalizes the two spellings, so when the roots coincide the second call is a memo hit and
    // reports `provenance=already-warm-on-entry` — which is PROVENANCE (where the build was
    // triggered), never ownership (what is charged for it).
    // ONE COLLECTION, ONE REFUSAL, EVERY DECLARED PHASE. `FloorPreparationPhase` is closed and
    // every member of it is measured and adjudicated here. The two earliest warms were once
    // reported and never judged, which made two of the modeled walls decorative — permanently
    // green by construction and citable as coverage (review 55338).
    // THE POOL-ROOT INDEX, WARMED BY CALLING THE DECLARED PRODUCER ONCE. `module_path_index` is
    // also built per POOL ROOT on demand by the decl-facts reflection seam, keyed on the root a
    // claim asks for, so the witness-roots warm above cannot reach it. Under the gate-bounded
    // subject the `src/v2/lens` root's 1.07s fill landed on whichever
    // `lens_module_gate_witness` live claim ran first and interrupted it at the 500ms ceiling —
    // three claims in three consecutive runs as each victim was withheld (CI 92cc92e, 0829ad8,
    // 154fb1f; claim-cost receipts), the positional bill `floor_cost_debt`'s header describes.
    // The key is not a literal here: the warm evaluates the same declaration the consumers
    // evaluate, `lens_registry_completeness_live_facts`, in that module's own scope, so the
    // root comes from `lens_registry_completeness_pool_roots` and the key is identical by
    // construction. A subject that does not carry the module carries no consumer either, so the
    // warm is skipped there — printed, not silent — and a producer that fails to evaluate stops
    // the line rather than leaving the fill to a claim.
    const LENS_REGISTRY_COMPLETENESS_MODULE: &str = "v2.lens.registry.completeness";
    let lens_pool_root_warm = match floor_authority_frame(
        &prepared,
        LENS_REGISTRY_COMPLETENESS_MODULE,
    ) {
        Ok(frame) => {
            let (evaluated, warm) = observe_shared_build(false, "floor-preparation", || {
                v1_interpreter::run_in_context(
                    &frame,
                    "v2.lens.registry.completeness.lens_registry_completeness_live_facts",
                    false,
                )
            });
            if let Err(e) = evaluated {
                return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=PoolRootIndexWarmFailed \
                     producer=v2.lens.registry.completeness.lens_registry_completeness_live_facts \
                     — {e}"
                ));
            }
            eprintln!(
                "[floor-phase] phase=pool-root-index-warm state=completed cpu_ms={} wall_ms={} \
                 rss_growth_bytes={} producer={}.lens_registry_completeness_live_facts provenance={}",
                warm.cpu_ms,
                warm.wall_ms,
                warm.rss_growth_bytes,
                LENS_REGISTRY_COMPLETENESS_MODULE,
                warm.provenance.render(),
            );
            Some(warm)
        }
        Err(why) => {
            eprintln!(
                "[floor-phase] phase=pool-root-index-warm state=skipped module={} — the subject \
                 does not carry the producer, so it carries no consumer of that key either: {why}",
                LENS_REGISTRY_COMPLETENESS_MODULE
            );
            None
        }
    };
    // THE LANGUAGES CONSUMER CENSUS, WARMED BY CALLING THE DECLARED PRODUCER ONCE — the same
    // repair as the three above, and the fourth artifact this repository has measured being
    // billed to a first toucher. `languages_decl_records_cached` is a process-wide `OnceLock`
    // over a token scan of every `.dag` and `.rs` file in the tree, built once; on main runs
    // 33251451113 and 33246969960 (`required_floor_claim_cost.tsv`) the whole build landed on
    // `v2.test.languages_consumer_census.corpus.rust_language_external_consumer
    // corpus_rust_language_has_external_consumer` at 412ms against the 500ms
    // `required_floor_claim_cpu_safety_limit_ms` — red on any runner a fifth slower, which is the
    // class `gunbc.rung_drop floor_cost_claim_qualification_unavailable` now carries with its measurements and
    // its restoration trigger; this comment states the instance and does not restate the class — while its
    // sibling in the same file, reading the identical memo milliseconds later, measured 0ms.
    // The `OnceLock` miss is not bracketed by `record_shared_artifact_fill_cpu`, so
    // `run_claim_measured` could not net it either; paying it here is the ONE mechanism, and a
    // second bracket beside this warm would be the two-homes fork
    // `gunbc.census_memo_seed_growth` already refuses. Skipped, printed, when the subject does
    // not carry the authority the census reads (`std.languages`, `LANGUAGES_AUTHORITY_REL`): a subject without it
    // carries no consumer of it either, and the census panics on that absence by design.
    let languages_census_warm = if languages_census_subject_carries_authority() {
        let (decl_rows, warm) = observe_shared_build(
            languages_decl_records_already_built(),
            "floor-preparation",
            || languages_decl_records_cached().len(),
        );
        eprintln!(
            "[floor-phase] phase=languages-consumer-census-warm state=completed cpu_ms={} \
             wall_ms={} rss_growth_bytes={} decl_rows={} provenance={}",
            warm.cpu_ms,
            warm.wall_ms,
            warm.rss_growth_bytes,
            decl_rows,
            warm.provenance.render(),
        );
        Some(warm)
    } else {
        eprintln!(
            "[floor-phase] phase=languages-consumer-census-warm state=skipped — the subject does \
             not carry {}, so it carries no consumer of the census either",
            LANGUAGES_AUTHORITY_REL
        );
        None
    };
    // THE CROSS-CLAIM PURE-PRODUCER SHARE — install the declared roster, wire its fills and
    // hits into the shared-fill ledger, and warm the preparation-forceable rows so their
    // fills land outside the fold (`install_pure_producer_share`). The roster module is a
    // REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES closure seed, so its absence from the
    // prepared subject is drift, and the install REFUSES rather than skipping — a silent
    // skip would relocate every warm fill onto the first toucher, the exact
    // nondeterministic charge this mechanism deletes.
    // THE WARM OBSERVATIONS JOIN THE SAME VECTOR THE PREPARATION REFUSAL ITERATES, which is the
    // whole of this change: the fills were already outside the per-claim ceiling and were until
    // now outside the preparation bound as well, so a rostered producer could warm for any cost at
    // all and stop nothing. The label carries the producer because the refusal names a phase and
    // "which shared build" must resolve to one roster row, not to the roster.
    let pure_producer_warms = install_pure_producer_share(&prepared)?;
    let mut shared_build_warms: Vec<(String, SharedBuildObservation)> = vec![
        ("ModulePathIndexBuild".to_string(), module_path_index_warm),
        ("SharedModuleIndexBuild".to_string(), shared_index_warm),
    ];
    if let Some(warm) = lens_pool_root_warm {
        shared_build_warms.push(("ModulePathIndexBuild/lens-pool-roots".to_string(), warm));
    }
    if let Some(warm) = languages_census_warm {
        shared_build_warms.push(("LanguagesConsumerCensusBuild".to_string(), warm));
    }
    shared_build_warms.push((
        "BareReferenceEdgeIndexBuild/source-roots".to_string(),
        warm_bare_reference_edge_index(&process_shared_index(source_roots))?,
    ));
    shared_build_warms.push((
        "BareReferenceEdgeIndexBuild/witness-layer-roots".to_string(),
        warm_bare_reference_edge_index(&process_shared_index(&witness_layer_roots()))?,
    ));
    shared_build_warms.extend(pure_producer_warms);
    // The two earlier phases already printed their own lines at the point they ran; only the
    // edge-index entries are reported here, so a phase is reported exactly once and under its own
    // name. Every entry — all three phases — is adjudicated together further down.
    for (which, warm) in shared_build_warms
        .iter()
        .filter(|(which, _)| which.starts_with("BareReferenceEdgeIndexBuild"))
    {
        eprintln!(
            "[floor-phase] phase=bare-reference-edge-index-warm state=completed roots={which} \
             cpu_ms={} wall_ms={} rss_growth_bytes={} source_files={} bare_eligible={} \
             provenance={}",
            warm.cpu_ms,
            warm.wall_ms,
            warm.rss_growth_bytes,
            warm.source_files,
            warm.bare_eligible,
            warm.provenance.render(),
        );
    }
    let prepare_ms = prepare_started.elapsed().as_millis();
    eprintln!(
        "floor: active sources = {}",
        prepared.modules_resolved + prepared.modules_excluded
    );
    eprintln!(
        "[floor-phase] phase=strict-preparation state=completed wall_ms={} modules_resolved={} \
         modules_excluded={} digest={}",
        prepare_ms, prepared.modules_resolved, prepared.modules_excluded, prepared.subject_digest
    );
    // WHERE PREPARATION'S WALL AND POPULATION GO: `compile.reconcile`, measured 2026-08-16.
    //
    // No dump is emitted here, and that is the finding rather than an omission. A
    // `[floor-prepare-split]` line reading `resolve_stage_slot_add`'s accumulators was added
    // here and printed every stage as 0.0ms against a 569,079ms phase — because those
    // accumulators are written only by `resolved_graph_from_sources_with_index`, and
    // `prepare_repository_once` reaches `v1_compiler_compile::compile_to_resolved` through
    // `resolved_graph_from_sources`, which takes no index and no memo share and touches none
    // of them. The zeros meant "unwired path", not "free"; that reading was recorded before
    // the run, because `typecheck=0.0ms` on a typecheck-dominated phase is otherwise exactly
    // the shape that gets reported as a result.
    //
    // The attribution already existed, in `compile_to_resolved_with_options`'s `trace_mark`
    // pairs, and had been printing in every floor log:
    //
    //     compile.frontend    39s
    //     compile.normalize    2s
    //     compile.reconcile    8min      <- 520s of a 569s phase
    //     compile.analyses     1s
    //
    // READ THE FIRST SET, NOT THE FIRST MATCH. A floor run emits these marks TWICE: once for
    // this preparation, and again for the published-mock projection below, where the same four
    // names appear at 33ms/4ms/41ms/1ms. The small set is a second, tiny compile — not this one
    // — and grepping `compile.reconcile` finds whichever the reader looks at first. That is not
    // hypothetical: the 41ms reading is why these marks were dismissed as belonging to the mock
    // projection for weeks while they were the whole preparation answer.
    //
    // Cross-read against a 5s heartbeat (`GUNBC_FLOOR_HEARTBEAT_SECS`), reconcile spans
    // t=45s..565s and RSS 3.53 GB -> 9.28 GB, so it owns ~91% of the wall AND essentially all
    // of preparation's ~5.85 GiB. Everything else in preparation totals 42 seconds.
    //
    // So preparation is not diffusely large and does not need new instrumentation; it needs
    // `reconcile_with_census_extra` over 3,668 modules to get smaller. Anyone adding a probe
    // here should read the existing trace marks first.
    //
    // RECONCILE'S INTERIOR, at 5s resolution — four regions, NONE of them attributed:
    //
    //      45- 95s     50s   flat at 3.42 GB
    //      95-225s    130s   3.42 -> 6.13 GB   +2.71 GB
    //     225-505s    280s   flat at 6.15 GB   +0.01 GB     54% of the wall
    //     505-565s     60s   6.23 -> 9.28 GB   +3.05 GB     53% of the growth, 12% of the wall
    //
    // Wall cost and memory growth are largely SEPARATE: over half the wall allocates nothing,
    // and the largest growth arrives in the final eighth. Which of reconcile's six operations
    // owns which region is UNKNOWN — six operations against four regions, and a 20s grid cannot
    // see a sub-20s operation at all. Marks would have to be authored in `src/v1/04_infer.dag`
    // and regenerated, since the operations live in generated code.
    //
    // TWO ATTRIBUTIONS WERE PROPOSED FOR THESE REGIONS AND BOTH DIED. Recorded because the
    // reasoning that killed them is reusable and the regions are still open, so the same two
    // candidates will look attractive again:
    //
    //   - Function-parent double-flattening as the +5.75 GiB owner. Killed by a population
    //     bound: at most 3,668 x 3,667 Rc slots, ~102.6 MiB raw, ~205 MiB for two generations.
    //     Not gigabytes.
    //   - `corpus_has_v1_seed_source_indices` as the 280s plateau owner. Its qualitative
    //     signature fits perfectly — long plateau, allocation churn, flat RSS, and a result of
    //     one `Bool` — and it is still wrong. Measured below.
    //
    // `corpus_has_v1_seed_source_indices`, PRICED, so it is neither cited as the root cause nor
    // dismissed for not being it. On the floor's own source configuration it clones the same
    // ~3,670-key source-index set once per module — `map_keys` materialises the whole key vector
    // BEFORE the `any` can short-circuit — for 13.46M key clones. Of the 3,670 files under
    // `--source-root dag --source-root src/v2`, exactly ZERO contain `/v1/` or `src/v1`, so no
    // early break ever fires and the function deterministically returns false. A synthetic on
    // the faithful carriers (`im::HashMap` -> `im::Vector`; `v1_rt.rs` aliases `Vec` to
    // `im::Vector`, and a first attempt on std containers understated it by 1.5x) measures
    // 4.095s, 0.304us per key. Explaining the 280s plateau would need 20.8us per key — 68x the
    // measured cost. So: a real duplicate derivation worth deleting, priced at ~4s, and NOT the
    // plateau. The terminal shape derives the fact once from the source-set authority rather
    // than per typed module, which removes the module x source product instead of tuning it.
    //
    // THE METHOD RULE BOTH DEATHS PRODUCED, because each failed a different gate: a qualitative
    // shape match does not constrain a constant, so price the mechanism independently before
    // naming an owner — never solve the unit cost backward from the interval being explained.
    // And a benchmark licenses nothing until its carriers, control flow, scale and short-circuit
    // behaviour match the real thing; assuming fidelity is how the first synthetic above came to
    // be 1.5x wrong while reading as decisive.

    // ── 2. the evaluation frames ──────────────────────────────────────────────────────────
    //
    // The published-mock corpus is part of the ENVELOPE and therefore part of every frame.
    // Preparing without it produced a world in which every claim reading a published mock
    // resolved against nothing: 9,057 of 9,317 witnesses could not find their own code, at a
    // digest that claimed to name the same subject.
    // ── T2 ────────────────────────────────────────────────────────────────────────────────
    // The WHOLE projection, not the compiler phase lines printed inside it. Those lines
    // report the second compile's phases and exclude index construction, declarer discovery,
    // closure selection and extraction — so quoting them as this helper's cost is a derived
    // number wearing a raw one's label. It read as ~73ms and was never measured.
    floor_seam("published-mock-projection");
    let published_started = std::time::Instant::now();
    let published = match precompute_whole_tree_published_mock_keys(source_roots) {
        Ok(keys) if keys.is_empty() => None,
        Ok(keys) => Some(Rc::new(keys)),
        Err(e) => return Err(format!("published mock corpus precompute failed: {e}")),
    };
    eprintln!(
        "[floor-phase] phase=published-mock-projection state=completed wall_ms={} keys={}",
        published_started.elapsed().as_millis(),
        published.as_ref().map(|k| k.len()).unwrap_or(0)
    );
    let policy_scope = claim_scope_for(&prepared, REQUIRED_FLOOR_POLICY_MODULE)?;
    let hermetic = evaluation_frame(
        &policy_scope,
        v1_interpreter::ExecutionMode::Hermetic,
        None,
        published.clone(),
    );
    // THE SAFETY WALL MOVES WITH THE COST. Leaving the shared build outside the claim timer while
    // reporting it as merely informational would be accounting laundering — no row blocks, the
    // real cost still occurs, and nothing owns the refusal. So the shared unit gets its OWN
    // limits, from `v2.workflow.required_floor`, and its own blocking outcome. They are NOT the
    // per-claim 5000ms: that number answers a claim-level question, and a shared whole-corpus
    // build is a different unit whose limits are derived from the resources they protect (the CI
    // job budget and the host memory cap), never from what the build was measured to cost.
    //
    // THE NUMBERS HAVE ONE AUTHORITY (the `.dag` constants); the three-axis COMPARISON below is a
    // hand-written mirror of `floor_preparation_outcome`, which is *mitigatable* and carries its
    // own dissolution obligation (`floor_preparation_host_mirror_dissolve_on`).
    let preparation_cpu_limit_ms =
        floor_required_measure_count(&hermetic, "required_floor_preparation_cpu_safety_limit")?;
    let preparation_wall_limit_ms =
        floor_required_measure_count(&hermetic, "required_floor_preparation_wall_safety_limit")?;
    let preparation_rss_growth_limit_bytes =
        floor_required_measure_count(&hermetic, "required_floor_preparation_rss_growth_limit")?;
    // Any ONE axis crossing stops the line: three independent bounds on one unit, never tiers.
    // There is no warn arm — a shared build that "ran long but was allowed to continue" is an
    // observation with no remedy attached, which is exactly what the deleted per-claim warn tier
    // cost this module once. And because this returns before the claim loop, a refused
    // preparation produces NO per-witness sheet at all: a roster of zero-cost rows beside a
    // refused preparation would read as measured and passing while nothing had run them.
    for (which, warm) in &shared_build_warms {
        if warm.cpu_ms > preparation_cpu_limit_ms
            || warm.wall_ms > preparation_wall_limit_ms
            || warm.rss_growth_bytes > preparation_rss_growth_limit_bytes
        {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=FloorPreparationRefused \
                 phase={which} observed_cpu_ms={} \
                 observed_wall_ms={} observed_rss_growth_bytes={} cpu_limit_ms={} \
                 wall_limit_ms={} rss_growth_limit_bytes={} — the shared bare-reference edge \
                 index build exceeded its own preparation limits (v2.workflow.required_floor); \
                 no claim executed, so every claim in this run is NotExecuted rather than \
                 zero-cost",
                warm.cpu_ms,
                warm.wall_ms,
                warm.rss_growth_bytes,
                preparation_cpu_limit_ms,
                preparation_wall_limit_ms,
                preparation_rss_growth_limit_bytes,
            ));
        }
    }
    // The output policy is installed FROM the prepared subject. Resolving
    // `dag/gunbc/output_policy.dag` on its own cost a separate whole-entry resolve to read
    // five channel decisions out of a world this function had already built.
    //
    // IN ITS OWN MODULE'S SCOPE, not the policy module's. A by-name evaluation of
    // `gunbc.output_policy.resolve_channel_policy` is a question about that module, so the
    // frame it runs in is that module's exact scope. Under the whole-tree subject the policy
    // module's reference closure happened to reach `gunbc.output_policy`; under the
    // gate-bounded subject it does not, and the bare name refused with "no declaration named
    // 'resolve_channel_policy' in this execution's loaded index" (measured 2026-08-29, twice,
    // with the module present in the subject both times). The scope was the coincidence.
    let output_policy_frame = floor_authority_frame(&prepared, "gunbc.output_policy")?;
    install_output_policy_in(&output_policy_frame, source_roots);

    // ── 3. the claim roster, projected from the prepared inventory ───────────────────────
    //
    // T3/T4/T5 SPLIT THIS REGION because it is the whole blank interval. Between the
    // preparation line and the first `floor: claims = ...` line nothing is printed, so a run
    // cancelled in here reports one undifferentiated gap containing site/binding projection,
    // the .dag manifest evaluation and admission decoding. Four superlinear shapes are known
    // to live in the manifest, which is a ranked hypothesis list and not an attribution: the
    // seams below are what turn the gap into one located term.
    floor_seam("site-projection");
    let projection_started = std::time::Instant::now();
    // ONE DECISION, MADE ONCE, WHERE THE FACTS ALREADY ARE.
    //
    // WHAT THIS REPLACED, and why it was not an optimisation. The required outcome of this whole
    // region is exactly: exclude a witness whose AUTHORED module sits in the long home, exclude
    // a walk-plan fixture member already driven by its recipe, and plan everything else. That
    // decision used to be
    // made TWICE — once by `required_floor_manifest` over 10,498 records marshalled into the
    // interpreter, and again here in Rust, applying the same prefix test to explain the
    // difference between sites offered and claims returned. Between the two sat an interpreted
    // fold whose only product was a population this host could compute directly from facts it
    // already returned by the modeled discovery producer.
    //
    // THE AUTHORED FACTS STAY AUTHORED. `long_home_prefixes`, the claim budget and the warn
    // threshold are still read from `v2.workflow.required_floor`, so the prefix list and both
    // thresholds remain .dag authorities and are not re-spelled in Rust. What is deleted is the
    // reconstruction, never the fact.
    //
    // THE TEST IS ON THE MODULE'S AUTHORED NAME, never its path — the 2026-08-04 ruling's actual
    // requirement, and the reason the inventory carries `module_path` beside each file. A
    // directory deciding admission was that ruling's root cause; reading the declaration is what
    // fixes it, and that is preserved here exactly.
    //
    // THE PARTITION IS EXACT BY CONSTRUCTION IN THE ARMS, AND CHECKED AS AN IDENTITY JOIN AT THE
    // END. Every site takes exactly one arm below, which is what makes the COUNTS agree; it is
    // not what makes the POPULATION agree, and those are different claims. A row written for the
    // wrong identity, an identity discovered twice, or a preparation-side row for an identity the
    // tree does not declare all satisfy "one arm per site" and still break the projection — so
    // `FloorDispositionJoinInexact` below joins the declared identities against the rows they
    // produced, and the old count equality (`SitePartitionInexact`) is gone rather than kept as a
    // weaker restatement of it.
    //
    // DUPLICATE ENROLLMENT IS UNCHANGED AND UNMOVED, and this is the one invariant the deleted
    // fold genuinely carried. It is caught downstream by `receipt_identities`, a HashSet keyed on
    // the same qualified name: two claims sharing a name give executed=2, receipted=1, and
    // `ClaimIdentityCountsDisagree` refuses. The manifest's enrollment map was a second mechanism
    // for that one invariant — and, being a growing `Value::Map` re-hashed once per fold step,
    // was the whole of the phase's quadratic cost.
    // THREE INDEPENDENT NUMBERS, not one and not two. Operator ruling 2026-08-19 (BUDGET POLICY
    // CUT), superseding correction the same date: the CPU and wall safety deadlines are
    // independently derived — never one figure copied into both clocks — and the completed-cost
    // line is diagnostic only and reads from its own constant, unconsulted by admission.
    // Reading all three from separate `.dag` constants is what makes the fusion structurally
    // impossible to reintroduce here — there is no longer a single value a future edit could
    // hand to more than one role by accident.
    let claim_cpu_safety_limit_ms =
        floor_required_int(&hermetic, "required_floor_claim_cpu_safety_limit_ms")?;
    let claim_wall_safety_limit_ms =
        floor_required_int(&hermetic, "required_floor_claim_wall_safety_limit_ms")?;
    let claim_cost_line_ms = floor_required_int(&hermetic, "required_floor_claim_cost_line_ms")?;
    // ARM THE OPAQUE-HOST-CALL SURFACE FROM ITS ONE AUTHORITY, AND REFUSE IF IT IS UNGROUNDED.
    //
    // The roster is NOT restated in Rust: it is read out of
    // `gunbc.v1_interpreter_opaque_host_call.opaque_host_call_surface()`, the same module the
    // §4b drop's bounded population is derived from, so the seed and the model cannot drift into
    // two answers about which arms are unpollable.
    //
    // THE UNGROUNDED ARM REFUSES RATHER THAN ARMING AN EMPTY SURFACE. An empty roster would
    // classify every crossing as `cooperatively_pollable` -- a mere overshoot -- which is the
    // reassuring direction and precisely the absorbing fallback DESIGN §5 forbids: the run would
    // report a population it never observed and the deficit would stop being visible at the
    // moment the join broke. So a broken join stops the line here, typed and located, and the
    // remedy is named rather than left to the reader.
    let opaque_surface = floor_required_opaque_host_call_surface(&hermetic)?;
    v1_interpreter::set_opaque_host_call_surface(Some(opaque_surface));
    let long_home_prefixes = floor_decode_module_prefix_roster(
        &hermetic,
        "v2.workflow.required_floor.long_home_prefixes",
    )?;
    // ONE DECODE, TWO ROSTERS. The fixture-home prefixes are read through the same helper as
    // the long-home prefixes because they are the same kind of fact — an authored module-name
    // prefix the floor declines on — and a second hand-rolled decode beside it would be a
    // second authority for how such a roster is read.
    let fixture_home_prefixes = floor_decode_module_prefix_roster(
        &hermetic,
        "v2.workflow.required_floor.fixture_home_prefixes",
    )?;
    let required_gate_prefixes = floor_decode_module_prefix_roster(
        &hermetic,
        "v2.workflow.required_floor.required_gate_prefixes",
    )?;
    if required_gate_prefixes.is_empty() {
        return Err("REQUIRED-FLOOR REFUSAL cause=RequiredGateRosterEmpty — \
                    v2.workflow.required_floor.required_gate_prefixes admits nothing, so the \
                    floor would plan zero claims and green over an empty population"
            .to_string());
    }
    // ── the witness roster, as the `.dag` discovery authority answers it ──────────────────
    //
    // ONE AUTHORITY FOR "WHAT DOES THE FLOOR DISCOVER". The roster used to be projected by a
    // Rust text scan that was filename-blind, while
    // `v2.workflow.floor_discovery_producer` -- reached only by claim_batch -- refused a
    // `test`-marked decl outside a `*_test.dag` sidecar. Two answers to one question (DESIGN
    // section 3), measured on main 2026-08-29: five files, 39 test fns, refused on one path and
    // executed on the other. The floor now folds the producer's own per-file authority over the
    // full module index and takes that fold's rows as the declared roster; Rust threads values and
    // decides nothing. Every refusal the producer carries -- misplaced test decl, barren sidecar,
    // misplaced wire contract, malformed live_tree_disposition row -- therefore stops THIS line,
    // with the producer's own reason text.
    //
    // The subject is preparation's full index, not a filesystem walk or a Rust declaration scan.
    // The prepared closure and exclusion map then classify producer-returned identities without
    // rebuilding the population. Row admission (`witness_row_excluded_from_discovery`)
    // is the discovery-corpus mode's policy and is deliberately NOT applied here -- see
    // `floor_discovery_row_admission_policy_note` in the producer.
    floor_seam("discovery-authority");
    let discovery_started = std::time::Instant::now();
    let producer_frame = floor_authority_frame(&prepared, FLOOR_DISCOVERY_AUTHORITY_MODULE)?;
    let discovery_source_count = full_inventory.len();
    let mut discovery_outcomes: Vec<v1_interpreter::Value> =
        Vec::with_capacity(full_inventory.len());
    for src in &full_inventory {
        let args = [
            (
                Some("repo_path".to_string()),
                str_value(src.source.path.replace('\\', "/")),
            ),
            (
                Some("content".to_string()),
                str_value(src.source.content.clone()),
            ),
        ];
        let outcome = v1_interpreter::run_in_context_with_args(
            &producer_frame,
            "v2.workflow.floor_discovery_producer.discover_floor_rows_for_source",
            &args,
            false,
        )
        .map_err(|e| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=FloorDiscoveryAuthorityUnevaluable source={} — \
                 discover_floor_rows_for_source: {e}",
                src.source.path.replace('\\', "/")
            )
        })?;
        discovery_outcomes.push(outcome);
    }
    let finalized = v1_interpreter::run_in_context_with_args(
        &producer_frame,
        "v2.workflow.floor_discovery_producer.floor_discovery_finalize_source_outcomes",
        &[(
            Some("outcomes".to_string()),
            list_value_from_vec(discovery_outcomes),
        )],
        false,
    )
    .map_err(|e| {
        format!(
            "REQUIRED-FLOOR REFUSAL cause=FloorDiscoveryAuthorityUnevaluable — \
             floor_discovery_finalize_source_outcomes: {e}"
        )
    })?;
    let discovery_rows = parse_floor_discovery_producer_result(&producer_frame, &finalized)
        .map_err(|reason| {
            format!("REQUIRED-FLOOR REFUSAL cause=FloorDiscoveryRefused — {reason}")
        })?;
    // Rows are (entry, function); the disposition loop below needs the entry's AUTHORED module
    // name, which preparation read off the `module` line and holds beside the same path.
    let module_for_path: std::collections::HashMap<String, &str> = full_inventory
        .iter()
        .map(|src| (src.source.path.replace('\\', "/"), src.module_path.as_str()))
        .collect();
    let mut files: Vec<FloorDiscoveryFile> = Vec::new();
    for row in &discovery_rows {
        let Some(module_path) = module_for_path.get(row.entry.as_str()) else {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=FloorDiscoveryEntryOutsideSubject entry={} — the \
                 discovery authority enrolled an entry the prepared subject does not hold",
                row.entry
            ));
        };
        match files.last_mut() {
            Some(last) if last.path == row.entry => last.functions.push(row.function.clone()),
            _ => files.push(FloorDiscoveryFile {
                path: row.entry.clone(),
                module_path: module_path.to_string(),
                functions: vec![row.function.clone()],
            }),
        }
    }
    drop(module_for_path);
    // THE RELEASE IS MEASURED IN THE SAME MOTION AS IT HAPPENS, by exclusive drop against the
    // existing statm/malloc_trim instruments rather than a new one: rss before, drop the full
    // inventory (whose outside-closure `Rc<SourceFile>` bytes have no other owner once the
    // gate-cut index discarded them), trim the freed-but-retained arena, rss after. What the
    // trim gives back after this drop is exactly the full-index retention this phase held; a
    // reader of the run therefore sees the retention's size and its end on the phase's own
    // line, instead of trusting a comment that the bytes were dropped (review 57430).
    let full_inventory_rss_kb_before = floor_sampled_field(floor_statm_rss_kb());
    drop(full_inventory);
    let full_inventory_trim_reclaimed_kb = floor_sampled_field(trim_retained_heap());
    let full_inventory_rss_kb_after = floor_sampled_field(floor_statm_rss_kb());
    files.sort_by(|a, b| a.path.cmp(&b.path));
    eprintln!(
        "[floor-phase] phase=discovery-authority state=completed wall_ms={} authority={} \
         sources={} rows={} entries={} full_inventory_release_rss_kb_before={} \
         full_inventory_release_trim_reclaimed_kb={} full_inventory_release_rss_kb_after={}",
        discovery_started.elapsed().as_millis(),
        FLOOR_DISCOVERY_AUTHORITY_MODULE,
        discovery_source_count,
        discovery_rows.len(),
        files.len(),
        full_inventory_rss_kb_before,
        full_inventory_trim_reclaimed_kb,
        full_inventory_rss_kb_after
    );
    let files = &files;
    // THE COST-DEBT ROSTER, decoded before the claim-build loop below because it decides an
    // ADMISSION and not a verdict. It answers a FOURTH question: not which claims exist, not
    // which of them are known to fail, and not which have no route — but which PASS and cost
    // more than the per-claim ceiling allows, and are therefore WITHHELD FROM EXECUTION until
    // made cheap.
    //
    // Withheld is not deferred and is not covered. See `v2.workflow.floor_cost_debt` for the
    // monotone debt contract and the DESIGN §4b(3) rung drop; the short form is that a rostered
    // row does not run anywhere, its coverage is gone while it sits there, and it leaves only by
    // being deleted from the roster and then passing under the ordinary ceiling.
    let cost_debt_roster: HashSet<String> = {
        let value = v1_interpreter::run_in_context(
            &hermetic,
            "v2.workflow.floor_cost_debt.floor_cost_debt_roster",
            false,
        )
        .map_err(|e| format!("floor_cost_debt_roster: {e}"))?;
        let items = floor_decode_list(&hermetic, Some(&value))
            .map_err(|e| format!("floor_cost_debt_roster: {e}"))?;
        let mut out = HashSet::new();
        for item in items {
            match item {
                v1_interpreter::Value::Str(s) => {
                    // A DUPLICATE REFUSES, for the reason the expected-red roster gives above
                    // and one more that is specific to a debt this size: the roster's length is
                    // the debt, a repeated identity makes that length overstate what is
                    // withheld, and the second copy survives every removal of the first.
                    if !out.insert(s.to_string()) {
                        return Err(format!(
                            "floor_cost_debt_roster: duplicate withheld identity: {s}"
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "floor_cost_debt_roster: expected a qualified name, got {}",
                        floor_value_shape(Some(other))
                    ));
                }
            }
        }
        // NO EMPTY-ROSTER REFUSAL, and the asymmetry with the expected-red roster is deliberate
        // rather than an oversight. An empty expected-red roster makes its downstream partition
        // and did-not-execute joins VACUOUS, so nothing can fire. An empty cost-debt roster
        // makes nothing vacuous: it means every witness runs under the ceiling, which is the
        // terminal state this contract is shrinking toward, and the guard that matters — the
        // stale-row join below — gets STRICTER as the roster empties, never weaker.
        out
    };
    eprintln!(
        "[floor-cost-debt] roster withholds {} identity(ies) from execution",
        cost_debt_roster.len()
    );

    // EXACTLY ONE DECLARED MECHANISM HOLDS A ROW, and when cost debt withholds one it is the
    // holder. `gunbc.quarantine_probe_disposition` states the rule this implements: the question
    // is never WHICH roster names a row, it is whether exactly one MECHANISM holds it, and a row
    // claimed by two is the authority-substitution shape.
    //
    // WHY THIS IS NOT OPTIONAL, measured rather than anticipated: 6 identities sit in both this
    // roster and `floor_expected_red` at the restoration. Leaving both claims standing blocks the
    // run in EITHER direction, which is what makes this a real fork and not a tidy-up. Withheld
    // and still enrolled, the expected-red reverse join reports them as stale (enrolled but never
    // executed) and refuses. Un-withheld so the enrollment stays observable, they execute, exceed
    // the 500ms ceiling, and `ExpectedRedArm` explicitly REFUSES to hold an interrupted budget
    // outcome -- so they land as ordinary failures and refuse too. All 6 measure 521-3865ms of
    // MARGINAL cpu, so none of them can execute inside the ceiling.
    //
    // COST DEBT WINS, and the direction follows from what each roster asserts. Enrollment in
    // `floor_expected_red` asserts, in that module's own words, that the identity REACHES ITS
    // SUBJECT AND ANSWERS. A withheld row answers nothing, so while it is withheld that assertion
    // is not merely unobserved -- it is untrue, and the roster that must yield is the one making
    // a claim the run cannot support. The enrollment is dormant, not deleted: the moment the row
    // leaves the cost-debt roster its expected-red claim is live again and observed again.
    //
    // SUPPRESSION IS COUNTED, NEVER SILENT. A roster quietly shrinking is how a skip list is
    // born, so the count is printed per roster and the population is recoverable by intersecting
    // the two authorities. This is a narrowing of what those joins observe, declared as one.
    //
    // THE REQUIRED GATE IS THE SECOND SUCH MECHANISM, same shape, same accounting. A module
    // outside the gate roster is never loaded into the prepared subject, so none of its
    // identities can execute, so an enrollment naming one asserts something this run cannot
    // observe. Measured 2026-08-29 on the first gate-bounded fold: 39 expected-red rows, every
    // one in a module outside the gate, refused as ExpectedRedIdentityDidNotExecute. They are
    // not stale -- they are out of this run's scope -- and the receipts workflow that runs the
    // whole corpus is where their standing is decided. Withheld here, counted per roster, and
    // observable again the moment the gate roster admits their module.
    let identity_inside_required_gate = |identity: &str| -> bool {
        let module_path = match identity.rfind('.') {
            Some(dot) => &identity[..dot],
            None => identity,
        };
        required_gate_prefixes
            .iter()
            .any(|prefix| module_path.starts_with(prefix.as_str()))
    };
    // RETURNS WHAT IT REMOVED, at identity grain and carrying which of the two grounds removed
    // it. The counts were already printed; what did not exist was the per-identity fact, and the
    // expected-red roster join needs exactly that to account for every enrolled identity rather
    // than for the survivors. See the disposition note in v1.compiler.expected_red_roster_join.
    let suppress_withheld = |roster: &mut HashSet<String>,
                             name: &str|
     -> Vec<(String, SuppressionGround)> {
        let mut removed: Vec<(String, SuppressionGround)> = Vec::new();
        let before = roster.len();
        let mut withheld: Vec<String> = roster
            .iter()
            .filter(|identity| {
                !(changed_witness_set.contains(*identity) || !cost_debt_roster.contains(*identity))
            })
            .cloned()
            .collect();
        withheld.sort();
        removed.extend(
            withheld
                .into_iter()
                .map(|identity| (identity, SuppressionGround::WithheldCostDebt)),
        );
        roster.retain(|identity| {
            changed_witness_set.contains(identity) || !cost_debt_roster.contains(identity)
        });
        let suppressed = before - roster.len();
        if suppressed > 0 {
            eprintln!(
                "[floor-cost-debt] {name}: {suppressed} enrolled identity(ies) suppressed because \
                 the cost-debt roster withholds them; their enrollment is dormant, not deleted, \
                 and becomes observable again when they leave that roster"
            );
        }
        let before = roster.len();
        let mut outside: Vec<String> = roster
            .iter()
            .filter(|identity| {
                !(changed_witness_set.contains(*identity)
                    || identity_inside_required_gate(identity))
            })
            .cloned()
            .collect();
        outside.sort();
        removed.extend(
            outside
                .into_iter()
                .map(|identity| (identity, SuppressionGround::OutsideRequiredGate)),
        );
        roster.retain(|identity| {
            changed_witness_set.contains(identity) || identity_inside_required_gate(identity)
        });
        let outside_gate = before - roster.len();
        if outside_gate > 0 {
            eprintln!(
                "[floor-required-gate] {name}: {outside_gate} enrolled identity(ies) suppressed \
                 because their module is outside the required gate and was never loaded; their \
                 enrollment is dormant, not deleted, and becomes observable again when the gate \
                 roster admits the module or in the whole-corpus receipts run"
            );
        }
        removed
    };

    let mut claims: Vec<RequiredFloorClaim> = Vec::new();
    let mut planned_identities: HashSet<String> = HashSet::new();
    let mut cost_debt_seen: HashSet<String> = HashSet::new();
    // THE OVERRIDE POPULATION: enrolled identities this change touched, which therefore execute
    // for their verdict instead of being withheld. Kept beside `cost_debt_seen` rather than
    // inside it — a withhold and an override are different acts, and the reconciliation below
    // asserts that the withheld set and the `DeclinedCostDebt` dispositions name the same
    // identities, which an override is by construction not one of.
    let mut cost_debt_verdict_only: HashSet<String> = HashSet::new();
    // WHAT EACH OVERRIDDEN ROW ACTUALLY COST, keyed by the debt identity. Minted at execution,
    // consumed by the changed-witness projection and by the published receipt line; never read
    // to decide admission, and never written back onto the authored roster.
    let mut cost_debt_observations: HashMap<String, ChangedWitnessCostObservation> = HashMap::new();
    let mut outcome_withheld_cost_debt: Vec<String> = Vec::new();
    // THE POPULATION, AT IDENTITY GRAIN. The discovery authority above answered over the FULL
    // module index, so this loop sees every DECLARED witness identity in the tree and classifies
    // each one — the prepared closure and the exclusion map decide which are offered and which
    // carry a preparation-stage decline. Both counts (`sites_offered`, `declared_identities`) are
    // read off this set rather than maintained beside it: a count and a population kept in step
    // by hand are two computations of one fact, and the count is the weaker one.
    let mut declared_identity_set: HashSet<String> = HashSet::new();
    let mut sites_offered = 0usize;
    let mut disposition_rows: Vec<RequiredFloorDispositionRow> = Vec::new();
    let mut storage_agreement_rows: Vec<LongHomeStorageAgreementRow> = Vec::new();
    for file in files {
        let matched_prefix = long_home_prefixes
            .iter()
            .find(|prefix| file.module_path.starts_with(prefix.as_str()));
        let long_home = matched_prefix.is_some();
        // Diagnostic only -- computed once per file and never consulted by the admission
        // branching below. See `LongHomeStorageAgreement`'s doc comment.
        let fixture_prefix = fixture_home_prefixes
            .iter()
            .find(|prefix| file.module_path.starts_with(prefix.as_str()));
        let inside_required_gate = required_gate_prefixes
            .iter()
            .any(|prefix| file.module_path.starts_with(prefix.as_str()));
        let path_is_long = is_long_home_path(&file.path);
        let storage_agreement = long_home_storage_agreement(path_is_long, long_home);
        for function in &file.functions {
            let identity = format!("{}.{}", file.module_path, function);
            // ONE SITE PER QUALIFIED IDENTITY, REFUSED OVER THE WHOLE OFFERED POPULATION.
            //
            // This wall used to stand further down, guarding PLANNED claims only, so a duplicate
            // whose first arm was a decline never reached it: two sites sharing one identity took
            // two arms, wrote two disposition rows, and the partition — a count equality — added
            // up exactly. The identity join below cannot express "this witness has one
            // disposition" while the offered side is a multiset, so the uniqueness of the offered
            // side is established HERE, where the site is enumerated, and the join downstream is
            // then a statement about identities rather than about totals.
            if !declared_identity_set.insert(identity.clone()) {
                return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=DuplicateWitnessIdentity identity={identity} — \
                     one qualified declaration was discovered at more than one site, so it would \
                     carry more than one disposition; a witness identity names exactly one site"
                ));
            }
            let selected_as_changed_witness = changed_witness_set.contains(&identity);
            if selected_as_changed_witness && !prepared_module_paths.contains(&file.module_path) {
                return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=ChangedWitnessOutsidePreparedSubject \
                     identity={identity} module={} — the changed-witness sublane selected this \
                     exact identity, but its module closure was not prepared, so execution \
                     cannot be represented as a decline",
                    file.module_path
                ));
            }
            if !prepared_module_paths.contains(&file.module_path) {
                let disposition = match discovery_exclusions.get(&file.module_path) {
                    Some(matched_substring) => {
                        RequiredFloorDisposition::DeclinedDiscoveryExcluded {
                            matched_substring: matched_substring.clone(),
                        }
                    }
                    None => RequiredFloorDisposition::DeclinedOutsideGateClosure,
                };
                disposition_rows.push(RequiredFloorDispositionRow {
                    identity,
                    disposition,
                });
                continue;
            }
            sites_offered += 1;
            storage_agreement_rows.push(LongHomeStorageAgreementRow {
                identity: identity.clone(),
                agreement: storage_agreement,
            });
            if selected_as_changed_witness {
                planned_identities.insert(identity.clone());
                disposition_rows.push(RequiredFloorDispositionRow {
                    identity: identity.clone(),
                    disposition: RequiredFloorDisposition::PlannedAsChangedWitness,
                });
                // THE TYPED JOIN OF TWO AUTHORITIES, DERIVED HERE AND NOWHERE ELSE
                // (`v2.workflow.required_floor` `changed_witness_cost_policy`,
                // FLOOR-CHANGED-COST-0, operator ruling 2026-08-30). A changed identity the
                // cost-debt roster enrolls runs FOR ITS VERDICT: the same 500ms CPU figure is
                // measured against and published against the debt identity, and the wall
                // deadline stays armed. A changed identity the roster does not enroll takes the
                // ordinary policy and still reds when it crosses that line, so this is one
                // intersection rather than a widening of the floor.
                let cost_policy = if cost_debt_roster.contains(&identity) {
                    cost_debt_verdict_only.insert(identity.clone());
                    ChangedWitnessCostPolicy::ChangedCostDebtVerdictOnly
                } else {
                    ChangedWitnessCostPolicy::Ordinary
                };
                claims.push(RequiredFloorClaim {
                    qualified: identity,
                    module_path: file.module_path.clone(),
                    function: function.clone(),
                    execution_mode: v1_interpreter::ExecutionMode::Hermetic,
                    cpu_safety_limit_ms: claim_cpu_safety_limit_ms,
                    wall_safety_limit_ms: claim_wall_safety_limit_ms,
                    cost_line_ms: claim_cost_line_ms,
                    cost_policy,
                });
                continue;
            }
            if long_home {
                disposition_rows.push(RequiredFloorDispositionRow {
                    identity,
                    disposition: RequiredFloorDisposition::DeclinedLongModule {
                        matched_prefix: matched_prefix
                            .expect("long_home is true only when matched_prefix is Some")
                            .clone(),
                    },
                });
                continue;
            }
            if let Some(prefix) = fixture_prefix {
                disposition_rows.push(RequiredFloorDispositionRow {
                    identity,
                    disposition: RequiredFloorDisposition::DeclinedFixtureMember {
                        matched_prefix: prefix.clone(),
                    },
                });
                continue;
            }
            // THE THIRD DECLINE, APPLIED ONLY TO A SITE THE TWO ABOVE ADMITTED. The precedence
            // is `required_floor_site_disposition`'s, not this loop's — cost debt is last, so a
            // rostered identity that ALSO declines for a home reason keeps its home decline and
            // never lands here. That is what makes such a roster line show up as stale (it never
            // enters `cost_debt_seen`) instead of as a withhold nobody can tell from a real one.
            //
            // DECLINED AT BUILD, NEVER SKIPPED AT EXECUTION. A withheld row must not become a
            // planned claim: `claims_planned == claims_executed + not_attempted` and the terminal
            // ledger's identity join both require every planned identity to reach a verdict, so a
            // skip inside the fold would refuse on both. Declining here keeps all three partition
            // checks exact and costs the fold nothing — no scope is built and no frame allocated
            // for a row that will not run.
            if cost_debt_roster.contains(&identity) {
                cost_debt_seen.insert(identity.clone());
                outcome_withheld_cost_debt.push(identity.clone());
                disposition_rows.push(RequiredFloorDispositionRow {
                    identity,
                    disposition: RequiredFloorDisposition::DeclinedCostDebt,
                });
                continue;
            }
            // THE FOURTH DECLINE, AFTER COST DEBT so a rostered identity outside the gate still
            // enters `cost_debt_seen` and the roster's staleness check keeps its meaning.
            if !inside_required_gate {
                disposition_rows.push(RequiredFloorDispositionRow {
                    identity,
                    disposition: RequiredFloorDisposition::DeclinedOutsideRequiredGate,
                });
                continue;
            }
            // NO SECOND DUPLICATE WALL LIVES HERE. This arm used to re-test uniqueness over the
            // PLANNED identities only, which is the same invariant the offered-side insert above
            // now establishes over the whole discovered population — strictly wider, and reached
            // before any arm is taken, so a duplicate whose first site declines is caught too.
            // Keeping both would be one fact with two producers (DESIGN §3), and the narrower one
            // dissolves on the climb (§4b(4)). `planned_identities` remains, as the expected side
            // of the terminal ledger join; it is no longer a uniqueness mechanism.
            planned_identities.insert(identity.clone());
            disposition_rows.push(RequiredFloorDispositionRow {
                identity: identity.clone(),
                disposition: RequiredFloorDisposition::Planned,
            });
            claims.push(RequiredFloorClaim {
                qualified: identity,
                module_path: file.module_path.clone(),
                function: function.clone(),
                execution_mode: v1_interpreter::ExecutionMode::Hermetic,
                cpu_safety_limit_ms: claim_cpu_safety_limit_ms,
                wall_safety_limit_ms: claim_wall_safety_limit_ms,
                cost_line_ms: claim_cost_line_ms,
                cost_policy: ChangedWitnessCostPolicy::Ordinary,
            });
        }
    }
    // Taken BEFORE the declared population is folded in, so it is what the site loop offered and
    // not that number reconstructed by subtracting one population from another.
    // THE UNIVERSE IS THE DECLARED POPULATION, NOT THE OFFERED ONE.
    //
    // Preparation removes two populations before a site can be offered — modules excluded by
    // substring, and (since the 2026-08-29 gate cut) every module the gate closure does not
    // reach — and a witness declared in one of them used to be neither planned nor declined. It
    // was not in `offered`, so no partition over `offered` could say anything about it: the
    // denominator itself had already narrowed, which is the level ABOVE the partition where
    // `docs/plans/witness-execution-closure.md` found the last missing population. After the gate
    // cut that silence is most of the corpus.
    //
    // The modeled producer has already enumerated all identities from the full index. Rows whose
    // modules preparation did not admit were classified in that same loop, so there is no second
    // declaration scan or complement population to append here. `sites_offered` keeps its old
    // meaning — identities inside the prepared subject — beside the corpus-wide `declared`.
    // THE PARTITION OVER THE DECLARED POPULATION, CHECKED AS AN IDENTITY JOIN — not as a count
    // equality, and not merely reported for a reader to add up.
    //
    // `claims_planned` is the POST-decline number, and the terminal invariant downstream
    // (`ClaimIdentityCountsDisagree`) compares planned == executed == receipted. Every one of
    // those three is measured after the projection has already dropped whatever it dropped, so
    // the run's own honesty check could not see what it lost: a projection that declined a
    // thousand identities and one that declined none produce identically healthy-looking
    // triples. This is the missing invariant on the other side of that seam — the offered
    // population must be exactly the dispositioned population — and it is stated where the loop
    // that could violate it runs.
    //
    // WHY IT IS A JOIN AND NOT `offered == routed + declined_*`. The old form was that sum, and
    // DESIGN §5 names it: completeness is an identity join, not a count equality. The sum is
    // green over a projection that writes a row for the wrong identity, over one that drops
    // `m.c` while writing `m.a` twice, and over any pair of errors that cancels in the totals —
    // exactly the calibration pair `terminal_ledger_completeness_law` pins one seam downstream.
    // It is the SAME join, through the SAME function
    // (`reconcile_identity_population`), because it is the same fact asked at a different seam.
    //
    // AND THE COUNTERS ARE DERIVED FROM THE ROWS, not accumulated beside them. Four `+= 1`s used
    // to run in the arms that push the rows, so the reported `declined_long` and the row
    // population were two computations of one fact and could disagree with nothing to say so.
    // With the counts folded out of the rows there is one producer, and the join above is what
    // makes the rows themselves trustworthy.
    let (declared_without_disposition, dispositioned_without_declaration, disposition_duplicated) =
        reconcile_identity_population(
            &declared_identity_set,
            disposition_rows.iter().map(|r| r.identity.as_str()),
        );
    if !declared_without_disposition.is_empty()
        || !dispositioned_without_declaration.is_empty()
        || !disposition_duplicated.is_empty()
    {
        let sample = |rows: &[&str]| {
            if rows.is_empty() {
                "none".to_string()
            } else {
                rows.iter()
                    .take(10)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
        };
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=FloorDispositionJoinInexact declared={} rows={} \
             declared_without_disposition={} dispositioned_without_declaration={} duplicated={} \
             — every DECLARED witness identity must join to exactly one disposition; a gap here \
             is a roster that narrowed, widened or double-counted without saying so. \
             declared_without_disposition: {} :: dispositioned_without_declaration: {} :: \
             duplicated: {}",
            declared_identity_set.len(),
            disposition_rows.len(),
            declared_without_disposition.len(),
            dispositioned_without_declaration.len(),
            disposition_duplicated.len(),
            sample(&declared_without_disposition),
            sample(&dispositioned_without_declaration),
            sample(&disposition_duplicated),
        ));
    }
    // TWO RANGES OVER ONE POPULATION, HANDLED HERE AND NOT REPAIRED HERE.
    //
    // `changed_witness_set` is derived from the run's git diff and is ROOT-AGNOSTIC.
    // `declared_identity_set` is enumerated from the discovered index, which is scoped to this
    // run's `--source-root`s (`dag` and `src/v2`). The selector's range is therefore strictly
    // WIDER than the enumerator's, so a witness homed anywhere else — `src/v1/tests/claim/...`
    // is the live population — is SELECTABLE AND UNDECLARABLE by construction. Such an identity
    // reached neither side of the join above (it is in `declared_identity_set` for neither the
    // declared nor the dispositioned side, so `FloorDispositionJoinInexact` stayed silent about
    // it) and then failed the sublane join below naming identities that NO EDIT INSIDE THE
    // OFFENDING PR COULD EVER SATISFY.
    //
    // The guard that would have named it, `ChangedWitnessOutsidePreparedSubject`, cannot fire:
    // it sits inside the per-DISCOVERED-file loop, and these modules never enter that loop.
    //
    // SO THE UNHANDLED CASE BECOMES A TYPED DECLINE, and nothing else changes. Every identity
    // the enumerator DID declare keeps exactly the disposition it had, so the dispositioned
    // population over discovered files is unchanged; only selections that were previously
    // unrepresentable acquire a row. It is a decline rather than a filter on purpose: silently
    // dropping the selection would green the floor by making the over-selection invisible, which
    // is the absorbing arm — the selector keeps over-selecting and nobody ever learns.
    //
    // THIS HANDLES THE MISMATCH AND DOES NOT RETIRE IT. NEXT-RUNG TRIGGER, named as the
    // capability: SELECTION AND DISPOSITION CONSUME ONE RANGE. A green floor over these rows
    // means the mismatch is represented, never that the two denominators have been reconciled.
    let undeclarable_changed: Vec<String> = {
        let mut v: Vec<String> = changed_witness_set
            .difference(&declared_identity_set)
            .cloned()
            .collect();
        v.sort();
        v
    };
    for identity in &undeclarable_changed {
        let module_path = identity
            .rsplit_once('.')
            .map(|(module, _)| module.to_string())
            .unwrap_or_else(|| identity.clone());
        disposition_rows.push(RequiredFloorDispositionRow {
            identity: identity.clone(),
            disposition: RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery {
                module_path,
            },
        });
    }
    // The set the sublane join is entitled to expect: everything the selector chose MINUS the
    // selections the enumerator could never declare, each of which now carries its own row.
    let changed_witness_expected: HashSet<String> = changed_witness_set
        .iter()
        .filter(|identity| !undeclarable_changed.contains(identity))
        .cloned()
        .collect();
    if !undeclarable_changed.is_empty() {
        println!(
            "required-ci: floor changed-witness selections outside discovery: {} identity(ies) \
             declined — the changed-witness selector ranges over the whole diff and the declared \
             population ranges over the discovery roots, so these are selectable and \
             undeclarable; they are represented, NOT reconciled: {}",
            undeclarable_changed.len(),
            undeclarable_changed.join(", ")
        );
    }

    // EXACTNESS OF THE SUBLANE, as an identity join rather than a count. The left side is the
    // single #9717 diff derivation captured before preparation; the right side is what this site
    // projection actually marked for changed execution. A missing, foreign, or duplicated row
    // cannot be repaired by the aggregate counts coincidentally agreeing.
    let changed_disposition_set: HashSet<String> = disposition_rows
        .iter()
        .filter(|row| {
            matches!(
                row.disposition,
                RequiredFloorDisposition::PlannedAsChangedWitness
            )
        })
        .map(|row| row.identity.clone())
        .collect();
    if changed_disposition_set != changed_witness_expected {
        let mut selected_without_disposition: Vec<&str> = changed_witness_expected
            .difference(&changed_disposition_set)
            .map(String::as_str)
            .collect();
        let mut disposition_without_selection: Vec<&str> = changed_disposition_set
            .difference(&changed_witness_expected)
            .map(String::as_str)
            .collect();
        selected_without_disposition.sort();
        disposition_without_selection.sort();
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=ChangedWitnessSublaneJoinInexact \
             selected_without_disposition=[{}] disposition_without_selection=[{}] — the \
             changed-witness execution sublane must execute exactly the one derived identity set",
            selected_without_disposition.join(", "),
            disposition_without_selection.join(", ")
        ));
    }
    // ONE PRODUCER FOR THE COUNTS: the joined row population, folded once per arm.
    let disposition_count = |select: fn(&RequiredFloorDisposition) -> bool| {
        disposition_rows
            .iter()
            .filter(|row| select(&row.disposition))
            .count()
    };
    let declared_identities = declared_identity_set.len();
    let long_declined =
        disposition_count(|d| matches!(d, RequiredFloorDisposition::DeclinedLongModule { .. }));
    let fixture_declined =
        disposition_count(|d| matches!(d, RequiredFloorDisposition::DeclinedFixtureMember { .. }));
    let outside_gate_declined =
        disposition_count(|d| matches!(d, RequiredFloorDisposition::DeclinedOutsideRequiredGate));
    let cost_debt_declined =
        disposition_count(|d| matches!(d, RequiredFloorDisposition::DeclinedCostDebt));
    let gate_closure_declined =
        disposition_count(|d| matches!(d, RequiredFloorDisposition::DeclinedOutsideGateClosure));
    let discovery_excluded_declined = disposition_count(|d| {
        matches!(
            d,
            RequiredFloorDisposition::DeclinedDiscoveryExcluded { .. }
        )
    });
    eprintln!(
        "[floor-phase] phase=site-projection state=completed wall_ms={} declared={} sites={} \
         files={} claims={} declined_long={} declined_fixture={} declined_outside_gate={} \
         declined_gate_closure={} declined_discovery_excluded={} declined_cost_debt={}",
        projection_started.elapsed().as_millis(),
        declared_identities,
        sites_offered,
        files.len(),
        claims.len(),
        long_declined,
        fixture_declined,
        outside_gate_declined,
        gate_closure_declined,
        discovery_excluded_declined,
        cost_debt_declined
    );

    // THE COST-DEBT ROSTER'S STANDING, JOINED AGAINST THE DECLARED UNIVERSE RATHER THAN AGAINST
    // THE SPELLING OF A NAME. Computed here because this is the last point at which the declared
    // set, the withheld set and the disposition rows are all live and unmoved; the reverse join
    // below consumes the result.
    //
    // WHAT THIS REPLACES AND WHY IT WAS A FAIL-OPEN. The reverse join used to decide "is this
    // unseen roster row merely outside the gate" by prefix-matching the identity's module path
    // against `required_gate_prefixes`. A module NAME cannot distinguish "declared, but this
    // run's gate never loaded it" from "no such declaration anywhere" — so an enrolled identity
    // that does not exist (a typo, a renamed or deleted witness, a fabricated line) prefix-missed
    // the gate, was counted as outside-the-gate, and refused nothing. That is precisely the hole
    // the reverse join exists to close, in its own words below: "the cheapest way to fake a green
    // run: enrolling an identity that does not exist would otherwise cost nothing." The
    // 2026-08-29 gate cut reopened it across the whole non-gate namespace, which is now the
    // majority of the corpus — 122 enrolled rows sat in that arm on the first gate-bounded fold,
    // and nothing could have told a real one from a fabricated one.
    //
    // gunbc#9684 is what makes the repair possible: every DECLARED identity now carries exactly
    // one disposition, so preparation's own account of what it loaded is available here as a
    // population rather than as a name test.
    let cost_debt_disposition_index: HashMap<String, RequiredFloorDisposition> = disposition_rows
        .iter()
        .map(|row| (row.identity.clone(), row.disposition.clone()))
        .collect();
    let cost_debt_standings =
        partition_cost_debt_roster(&cost_debt_roster, &cost_debt_disposition_index);
    // THE WITHHELD SET AND THE DISPOSITION ROWS MUST NAME THE SAME IDENTITIES, and a disagreement
    // REFUSES rather than being resolved in either direction. They are two observations of one
    // act — the build loop's own accounting, and the projection of that same loop — so they agree
    // by construction or the loop is wrong. An earlier draft of this change consulted the
    // withheld set as a FIRST classifier and fell back to the disposition, which made two
    // structures answer one question and would have silently preferred one; that is the §3 defect
    // this repair exists to close, and rebuilding it inside the repair is the failure mode worth
    // refusing loudly for.
    {
        let (withheld_without_disposition, dispositioned_without_withhold) =
            reconcile_withheld_against_dispositions(&cost_debt_seen, &cost_debt_disposition_index);
        if !withheld_without_disposition.is_empty() || !dispositioned_without_withhold.is_empty() {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=CostDebtWithholdDispositionDisagreement — the \
                 build loop's withheld set and its disposition projection are two observations of \
                 one act and must name the same identities. withheld with no DeclinedCostDebt \
                 disposition: [{}]; DeclinedCostDebt disposition with no withhold: [{}]",
                withheld_without_disposition.join(", "),
                dispositioned_without_withhold.join(", ")
            ));
        }
    }
    let cost_debt_undeclared: Vec<&str> = cost_debt_standings
        .iter()
        .filter(|(_, s)| *s == CostDebtRosterStanding::Undeclared)
        .map(|(q, _)| *q)
        .collect();
    let cost_debt_outside_universe: Vec<&str> = cost_debt_standings
        .iter()
        .filter(|(_, s)| *s == CostDebtRosterStanding::OutsideThisRunsUniverse)
        .map(|(q, _)| *q)
        .collect();
    let cost_debt_declared_not_withheld: Vec<&str> = cost_debt_standings
        .iter()
        .filter(|(_, s)| *s == CostDebtRosterStanding::DeclaredButNotWithheld)
        .map(|(q, _)| *q)
        .collect();
    let cost_debt_withheld_count = cost_debt_standings
        .iter()
        .filter(|(_, s)| *s == CostDebtRosterStanding::Withheld)
        .count();
    // THE OVERRIDDEN POPULATION IS REPORTED SEPARATELY FROM BOTH NEIGHBOURS, because it is
    // neither (FLOOR-CHANGED-COST-0). Counted as withheld it would overstate what this run
    // actually froze; counted as declared-not-withheld it would be told to delete a roster line
    // that still describes the tree.
    let cost_debt_overridden: Vec<&str> = cost_debt_standings
        .iter()
        .filter(|(_, s)| *s == CostDebtRosterStanding::WithholdOverriddenForChangedVerdict)
        .map(|(q, _)| *q)
        .collect();
    // THE PARTITION IS THE RECEIPT, AT IDENTITY GRAIN AND ON ONE LINE. The previous report was a
    // bare count of the outside-gate arm; a count cannot be joined against a roster, and the arm
    // it counted silently contained both legitimate rows and unrefusable fabrications.
    eprintln!(
        "[floor-cost-debt] roster standing: enrolled={} withheld={} \
         withhold_overridden_for_changed_verdict={} outside_this_runs_universe={} \
         undeclared={} declared_not_withheld={}",
        cost_debt_standings.len(),
        cost_debt_withheld_count,
        cost_debt_overridden.len(),
        cost_debt_outside_universe.len(),
        cost_debt_undeclared.len(),
        cost_debt_declared_not_withheld.len()
    );
    if !cost_debt_overridden.is_empty() {
        eprintln!(
            "[floor-cost-debt] withhold overridden for a changed verdict, still valid debt and \
             NOT stale (this change touched them, so they execute for their verdict while the \
             500ms CPU line is observed and published rather than gating): {}",
            cost_debt_overridden.join(", ")
        );
    }
    if !cost_debt_outside_universe.is_empty() {
        eprintln!(
            "[floor-cost-debt] outside this run's universe, kept as record and NOT counted as \
             debt (declared, but preparation never offered them — gate closure or discovery \
             exclusion): {}",
            cost_debt_outside_universe.join(", ")
        );
    }

    // THE EXPECTED-RED ROSTER, read from its .dag authority in the policy module's frame — it
    // must be decoded BEFORE that frame is dropped below, and it is a separate evaluation from
    // the manifest because it answers a different question: the manifest says which claims
    // exist, this says which of them are known to fail while someone fixes them.
    let expected_red_roster: HashSet<String> = {
        let value = v1_interpreter::run_in_context(
            &hermetic,
            "v2.workflow.floor_expected_red.floor_expected_red_roster",
            false,
        )
        .map_err(|e| format!("floor_expected_red_roster: {e}"))?;
        let items = floor_decode_list(&hermetic, Some(&value))
            .map_err(|e| format!("floor_expected_red_roster: {e}"))?;
        let mut out = HashSet::new();
        for item in items {
            match item {
                v1_interpreter::Value::Str(s) => {
                    // A DUPLICATE REFUSES. The roster's length is read as the debt, and a
                    // repeated identity makes that length lie in the direction that flatters:
                    // 820 rows naming 819 identities reports one more fixed row than exists,
                    // and the second copy survives every removal of the first. The set would
                    // absorb it silently, so the refusal has to be here rather than in the set.
                    if !out.insert(s.to_string()) {
                        return Err(format!(
                            "floor_expected_red_roster: duplicate enrolled identity: {s}"
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "floor_expected_red_roster: expected a qualified name, got {}",
                        floor_value_shape(Some(other))
                    ));
                }
            }
        }
        // AN EMPTY ROSTER REFUSES, because it is indistinguishable from a roster that could
        // not be read. Every downstream guard here is a join over this set: the partition-sum
        // check compares four counters against `len()`, and the did-not-execute check walks
        // the roster looking for identities no claim reported. At zero, all of them are
        // VACUOUSLY TRUE -- 0+0+0+0 == 0 passes, and nothing is missing from an empty set. So
        // the one shape that disables every check is the one shape nothing was checking.
        //
        // This is the empty-observation narrow, and it is the mirror of the absorbing fallback
        // rather than an instance of it: a widen is merely expensive, a narrow is silently
        // uncovered. It ran live on main. #8437 flipped prepared-floor scope to bind bare
        // helper names last-write-wins, `floor_expected_red_roster` began evaluating to the
        // empty list, and the run reported `roster carries 0 enrolled identity(ies)` followed
        // by 469 ordinary FAILs -- 469 enrolled rows each re-labelled a regression, with the
        // remainder absorbed as passes. The immediately preceding commit reported 661.
        //
        // The roster is a debt ledger shrinking toward zero, so an empty one WILL eventually
        // be legitimate. It is not legitimate SILENTLY: the day the last row is removed, this
        // refusal is what makes someone delete it deliberately and say so, rather than a read
        // failure quietly wearing the same face as success.
        if out.is_empty() {
            return Err("REQUIRED-FLOOR REFUSAL cause=ExpectedRedRosterEmpty — \
                 v2.workflow.floor_expected_red.floor_expected_red_roster evaluated to zero \
                 identities. An empty roster makes the partition-sum and did-not-execute \
                 checks vacuous, so every enrolled row reports as an ordinary failure and no \
                 guard can fire. If the roster is genuinely empty, delete this refusal in the \
                 same change that empties it."
                .to_string());
        }
        out
    };
    let mut expected_red_roster = expected_red_roster;
    let expected_red_suppressed = suppress_withheld(&mut expected_red_roster, "floor_expected_red");
    eprintln!(
        "[floor-known-red] roster carries {} enrolled identity(ies)",
        expected_red_roster.len()
    );

    // THE ROUTE-GAP ROSTER, read the same way and for the same reason: it must be decoded while
    // the policy frame is alive. It answers a THIRD question, distinct from both of the two
    // above — not which claims exist, and not which of them are known to fail, but which of
    // them the floor currently has no route that can RUN. See
    // `v2.workflow.floor_route_gap` for the contract; the short form is that enrollment
    // changes which outcome counts as agreement and nothing else, and that an unenrolled route
    // gap reds the build.
    //
    // NO EMPTY-ROSTER REFUSAL HERE, and the asymmetry with the expected-red roster above is
    // deliberate rather than an omission. That refusal exists because an empty expected-red
    // roster makes its OWN downstream guards vacuous — a partition sum of zero against zero,
    // a did-not-execute walk over nothing. This roster has no such guard to disable: an
    // identity that is not enrolled BLOCKS, so an empty roster is the strictest possible
    // state, not the most permissive one. A read failure here therefore cannot flatter a run;
    // it can only red one that would otherwise be green.
    let route_gap_roster: HashSet<String> = {
        let value = v1_interpreter::run_in_context(
            &hermetic,
            "v2.workflow.floor_route_gap.floor_route_gap_roster",
            false,
        )
        .map_err(|e| format!("floor_route_gap_roster: {e}"))?;
        let items = floor_decode_list(&hermetic, Some(&value))
            .map_err(|e| format!("floor_route_gap_roster: {e}"))?;
        let mut out = HashSet::new();
        for item in items {
            match item {
                v1_interpreter::Value::Str(s) => {
                    // A DUPLICATE REFUSES, for the same reason it does above: the roster's
                    // length is read as the debt, and a repeated identity makes that length
                    // report one more supplied route than exists.
                    if !out.insert(s.to_string()) {
                        return Err(format!(
                            "floor_route_gap_roster: duplicate enrolled identity: {s}"
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "floor_route_gap_roster: expected a qualified name, got {}",
                        floor_value_shape(Some(other))
                    ));
                }
            }
        }
        out
    };
    let mut route_gap_roster = route_gap_roster;
    let _ = suppress_withheld(&mut route_gap_roster, "floor_route_gap");
    eprintln!(
        "[floor-route-gap] roster carries {} enrolled identity(ies)",
        route_gap_roster.len()
    );

    // New enrollments carry the operation and the closed remedy-ground observed at the
    // interpreter boundary. Their identities are projected into `floor_route_gap_roster` by
    // the .dag authority; this decode reads the detail rather than authoring a second identity
    // list in Rust. A changed operation or ground blocks below instead of being silently held
    // by an identity-only row whose original fact no longer applies.
    let mut expectations_outside_gate = 0usize;
    let mut expectations_withheld_cost_debt = 0usize;
    let route_gap_expectations: HashMap<String, FloorRouteGapExpectation> = {
        let value = v1_interpreter::run_in_context(
            &hermetic,
            "v2.workflow.floor_route_gap.floor_route_gap_expectations",
            false,
        )
        .map_err(|e| format!("floor_route_gap_expectations: {e}"))?;
        let items = floor_decode_list(&hermetic, Some(&value))
            .map_err(|e| format!("floor_route_gap_expectations: {e}"))?;
        let mut out = HashMap::new();
        for item in items {
            let v1_interpreter::Value::Record { type_name, fields } = item else {
                return Err(format!(
                    "floor_route_gap_expectations: expected FloorRouteGapExpectation, got {}",
                    floor_value_shape(Some(&item))
                ));
            };
            if !hermetic.sym_eq(*type_name, "FloorRouteGapExpectation") {
                return Err(format!(
                    "floor_route_gap_expectations: expected FloorRouteGapExpectation, got record {}",
                    hermetic.resolve(*type_name)
                ));
            }
            let identity = match hermetic.field(&fields, "identity") {
                Some(v1_interpreter::Value::Str(s)) => s.to_string(),
                other => {
                    return Err(format!(
                        "floor_route_gap_expectations: identity must be String, got {}",
                        floor_value_shape(other)
                    ));
                }
            };
            let operation = match hermetic.field(&fields, "operation") {
                Some(v1_interpreter::Value::Str(s)) => s.to_string(),
                other => {
                    return Err(format!(
                        "floor_route_gap_expectations: operation must be String, got {}",
                        floor_value_shape(other)
                    ));
                }
            };
            let ground = match hermetic.field(&fields, "ground") {
                Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
                    match hermetic.resolve(*variant_name).as_str() {
                        "UnpublishedMockCase" => FloorRouteGapExpectedGround::UnpublishedMockCase,
                        "NoMockResponse" => FloorRouteGapExpectedGround::NoMockResponse,
                        "FilesystemRemoval" => FloorRouteGapExpectedGround::FilesystemRemoval,
                        other => {
                            return Err(format!(
                                "floor_route_gap_expectations: unknown ground {other}"
                            ));
                        }
                    }
                }
                other => {
                    return Err(format!(
                        "floor_route_gap_expectations: ground must be a typed variant, got {}",
                        floor_value_shape(other)
                    ));
                }
            };
            // SAME WITHHOLDING AS THE ROSTER IT JOINS. The roster above has already had its
            // outside-gate rows withheld (counted), so an expectation located on one of them
            // is out of scope for this run, not absent from the roster.
            if !identity_inside_required_gate(identity.as_str()) {
                expectations_outside_gate += 1;
                continue;
            }
            // COST DEBT WINS here too: the roster this joins has already had its cost-debt
            // rows withheld (`suppress_withheld`), so an expectation located on one of them is
            // dormant with its row, not absent from the roster.
            if cost_debt_roster.contains(identity.as_str()) {
                expectations_withheld_cost_debt += 1;
                continue;
            }
            if !route_gap_roster.contains(identity.as_str()) {
                return Err(format!(
                    "floor_route_gap_expectations: located identity is absent from derived roster: {identity}"
                ));
            }
            if out
                .insert(
                    identity.clone(),
                    FloorRouteGapExpectation { operation, ground },
                )
                .is_some()
            {
                return Err(format!(
                    "floor_route_gap_expectations: duplicate enrolled identity: {identity}"
                ));
            }
        }
        out
    };
    if expectations_withheld_cost_debt > 0 {
        eprintln!(
            "[floor-cost-debt] floor_route_gap_expectations: {expectations_withheld_cost_debt} \
             expectation(s) dormant because the cost-debt roster withholds their identity"
        );
    }
    if expectations_outside_gate > 0 {
        eprintln!(
            "[floor-required-gate] floor_route_gap_expectations: {expectations_outside_gate} \
             expectation(s) withheld because their identity's module is outside the required gate"
        );
    }

    // THE TWO ROSTERS MAY NOT NAME THE SAME IDENTITY, and this refusal is the reason the split
    // between them stays a split rather than decaying back into the conflation it was created
    // to undo.
    //
    // They make CONTRADICTORY claims. Enrollment in `floor_expected_red` asserts that an
    // identity REACHES ITS SUBJECT AND ANSWERS FALSE — a statement about a verdict. Enrollment
    // in `floor_route_gap` asserts that it never reaches its subject at all. Both cannot be
    // true of one identity, and the failure mode is not hypothetical: 101 identities sat in the
    // expected-red roster for exactly this reason, held as agreed failures while producing no
    // verdict, until the typed outcome made the difference observable. Having paid to separate
    // them once, leaving nothing to stop them merging again would be the same defect with a
    // longer fuse.
    //
    // It refuses by NAME rather than by count, because the remedy is per identity: decide which
    // fact is true of it and delete the other row.
    {
        let mut both: Vec<&String> = route_gap_roster
            .iter()
            .filter(|q| expected_red_roster.contains(q.as_str()))
            .collect();
        both.sort();
        if !both.is_empty() {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=RosterClaimsContradict count={} — these \
                 identities are enrolled BOTH in v2.workflow.floor_expected_red (which asserts \
                 the witness reaches its subject and answers false) AND in \
                 v2.workflow.floor_route_gap (which asserts it never reaches its subject). Both \
                 cannot be true. Decide which one is, and delete the other row: {}",
                both.len(),
                both.iter()
                    .map(|q| q.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // THE NON-VERDICT ROSTER. See `v2.workflow.floor_non_verdict` for the contract; the short
    // form is that an enrolled expected-red identity which produces NO VERDICT — it throws, or
    // answers a non-Bool — must be enrolled here, and an unenrolled one STOPS THE LINE.
    //
    // THE POLARITY IS THE CONSTRUCTION. An identity that is not enrolled BLOCKS, so an EMPTY
    // roster is the STRICTEST state rather than the most permissive one, and a read failure
    // here cannot flatter a run — it can only red one that would otherwise be green. The
    // opposite polarity (a roster of things to gate ON) would have made losing the roster
    // produce a greener answer, which is the absorbing-fallback shape rebuilt inside the
    // mechanism written to close an absorbing fallback.
    let non_verdict_roster: HashSet<String> = {
        let value = v1_interpreter::run_in_context(
            &hermetic,
            "v2.workflow.floor_non_verdict.floor_non_verdict_roster",
            false,
        )
        .map_err(|e| format!("floor_non_verdict_roster: {e}"))?;
        let items = floor_decode_list(&hermetic, Some(&value))
            .map_err(|e| format!("floor_non_verdict_roster: {e}"))?;
        let mut out = HashSet::new();
        for item in items {
            match item {
                v1_interpreter::Value::Str(s) => {
                    if !out.insert(s.to_string()) {
                        return Err(format!(
                            "floor_non_verdict_roster: duplicate enrolled identity: {s}"
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "floor_non_verdict_roster: expected a qualified name, got {}",
                        floor_value_shape(Some(other))
                    ));
                }
            }
        }
        out
    };
    let mut non_verdict_roster = non_verdict_roster;
    let _ = suppress_withheld(&mut non_verdict_roster, "floor_non_verdict");
    eprintln!(
        "[floor-non-verdict] roster carries {} enrolled identity(ies)",
        non_verdict_roster.len()
    );

    // AND THIS ROSTER NEEDS NO SEPARATE FREEZE-DISJOINTNESS CHECK, which is worth saying because
    // the wall below now covers three rosters and this is a fourth. The refusal immediately
    // beneath enforces non-verdict SUBSET-OF expected-red, and expected-red is already
    // cross-referenced against `frozen_path_deferrals` there — so an identity claiming both
    // "produced no verdict while executing" and "never executes" is already refused, transitively
    // and by construction. A fourth arm would be a second representation of a fact the subset
    // relation already carries.

    // A NON-VERDICT ROW THAT IS NOT ALSO EXPECTED-RED CAN NEVER FIRE, so it is refused rather
    // than left in the tree. Only an ENROLLED identity reaches the arms this roster classifies;
    // an unenrolled one that throws goes through the ordinary failure path. So such a row is not
    // a guard sitting quiet — the mechanism cannot produce the state it names — and DESIGN's
    // reachability rule is explicit that unreachable is not empty: a check whose red cannot be
    // authored is a decoration, worse than absent, because it is cited as coverage.
    {
        let mut orphans: Vec<&String> = non_verdict_roster
            .iter()
            .filter(|q| !expected_red_roster.contains(q.as_str()))
            .collect();
        orphans.sort();
        if !orphans.is_empty() {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=NonVerdictRowUnreachable count={} — these \
                 identities are enrolled in v2.workflow.floor_non_verdict but NOT in \
                 v2.workflow.floor_expected_red. Only an enrolled identity can reach the \
                 non-verdict arms, so these rows can never classify anything: they read as debt \
                 while being incapable of being debt. Enroll the identity as expected-red, or \
                 delete the row: {}",
                orphans.len(),
                orphans
                    .iter()
                    .map(|q| q.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // CONTRADICTORY-INTERSECTION WALL: both executing-roster classifications and
    // `witness_deferral_freeze`'s
    // `frozen_path_deferrals` (`LegacyFrozenPathDeferral` — admitted as NEVER EXECUTED) make
    // opposite claims about the same identity. Neither can coexist with the freeze: expected-red
    // requires a verdict, while a typed route-gap requires an attempted execution that produced
    // a pre-verdict route receipt. Either proves the freeze's no-executing-consumer classification
    // stale. Construction, not validation (DESIGN.md §5): all three rosters
    // are cross-referenced from their own source authorities on every required-floor run, so the
    // contradiction cannot re-accumulate silently the way it did before this wall existed.
    {
        let freeze_content = std::fs::read_to_string(
            workspace_root().join(WITNESS_DEFERRAL_FREEZE_AUTHORITY_REL),
        )
        .map_err(|e| {
            format!("witness deferral freeze: failed to read {WITNESS_DEFERRAL_FREEZE_AUTHORITY_REL}: {e}")
        })?;
        let frozen_qualified = frozen_path_deferral_qualified_identities_from_source(
            &freeze_content,
            &workspace_root(),
        );
        let colliding = expected_red_freeze_intersection(&frozen_qualified, &expected_red_roster);
        if !colliding.is_empty() {
            let head = current_git_head_or_unresolved();
            return Err(format_expected_red_freeze_intersection_refusal(
                &colliding, &head,
            ));
        }
        let colliding = route_gap_freeze_intersection(&frozen_qualified, &route_gap_roster);
        if !colliding.is_empty() {
            let head = current_git_head_or_unresolved();
            return Err(format_route_gap_freeze_intersection_refusal(
                &colliding, &head,
            ));
        }
    }

    let roster_join_path = std::env::var("GUNBC_EXPECTED_RED_ROSTER_JOIN").ok();
    let roster_join_only = std::env::var("GUNBC_EXPECTED_RED_ROSTER_JOIN_ONLY")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let roster_join_active = roster_join_path.is_some() || roster_join_only;
    let required_floor_disposition_path = std::env::var("GUNBC_REQUIRED_FLOOR_DISPOSITION").ok();
    let long_home_storage_agreement_path = std::env::var("GUNBC_LONG_HOME_STORAGE_AGREEMENT").ok();
    let claim_cost_path = std::env::var("GUNBC_REQUIRED_FLOOR_CLAIM_COST").ok();
    let cross_claim_demand_path = std::env::var("GUNBC_REQUIRED_FLOOR_CROSS_CLAIM_DEMAND").ok();
    let mut roster_join_report = if roster_join_active {
        // THE DENOMINATOR IS THE ENROLLED ROSTER, NOT THE SURVIVORS. Until 2026-09-01 this took
        // the post-suppression roster, so the report described only identities the fold could
        // reach while its own run_note said it described every enrolled one. The suppressed rows
        // now enter the report and carry their ground, so the sentence is true and a reader can
        // see the two populations apart.
        let mut roster_identities: Vec<String> = expected_red_roster
            .iter()
            .cloned()
            .chain(expected_red_suppressed.iter().map(|(id, _)| id.clone()))
            .collect();
        roster_identities.sort();
        roster_identities.dedup();
        let run_head = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .filter(|s| !s.is_empty());
        let run_note = if roster_join_only {
            "join-only mode: evaluates enrolled identities present in the manifest; do not \
             prune the roster from this output until the rebase wave (#8420) restores host-tool \
             verdicts"
                .to_string()
        } else {
            "full-floor join: every enrolled identity receives still_red | now_passes | \
             not_evaluated | suppressed; suppressed rows were removed before the fold and carry \
             the ground that removed them"
                .to_string()
        };
        let mut report =
            crate::v1_compiler_expected_red_roster_join::new_expected_red_roster_join_report(
                run_head,
                run_note,
                std::rc::Rc::new(roster_identities.into_iter().collect::<im::Vector<_>>()),
            );
        for (identity, ground) in expected_red_suppressed.iter() {
            report = crate::v1_compiler_expected_red_roster_join::record_suppressed(
                report,
                identity.clone(),
                *ground,
            );
        }
        Some(report)
    } else {
        None
    };

    // THE MANIFEST'S WORLD DIES HERE, before the fold rather than at the end of the function.
    //
    // `hermetic` is the frame the manifest was folded in. It owns a scope AND the mutable
    // evaluation caches that scope accumulated while folding 10,114 sites — call memo, data
    // cache, name caches — and `admission` is the decoded Value it produced. Neither is read
    // again: `claims` is the projection of both, and the fold consumes only `claims`.
    //
    // Left to ordinary scope they live until the function returns, which is to say through the
    // entire fold, and every byte they hold is a byte the fold does not have. That matters more
    // than it looks: the runner throttles at `memory.high` and the last run sat within ~1MB of
    // the watermark, so retained-but-unread state is not slack — it is wall time, because under
    // throttling the kernel reclaims continuously and every task in the cgroup stalls.
    //
    // Measured before this change, published-mock-projection stepped the resident set 6.58GB ->
    // 9.15GB and it stayed there for the run. 90 mock keys do not cost 2.6GB; a retained frame
    // over a folded manifest plausibly does. Whether that is what this recovers is exactly what
    // the next run says, and if the step survives then the cost is elsewhere and this was still
    // correct — an unread value held across the longest phase of the program has no defence.
    drop(hermetic);
    drop(policy_scope);

    // ── 4. fold the manifest ──────────────────────────────────────────────────────────────
    eprintln!("floor: claims = {}", claims.len());
    if roster_join_only {
        let before = claims.len();
        claims.retain(|c| expected_red_roster.contains(c.qualified.as_str()));
        eprintln!(
            "floor: expected-red-roster-join-only retained {} of {} claim(s)",
            claims.len(),
            before
        );
    }
    let claims_planned = claims.len();
    let mut outcome = RequiredFloorOutcome {
        subject_digest: prepared.subject_digest.clone(),
        modules_resolved: prepared.modules_resolved,
        modules_excluded: prepared.modules_excluded,
        sites_offered,
        declined_long_module: long_declined,
        declined_fixture_member: fixture_declined,
        declined_outside_required_gate: outside_gate_declined,
        declared_identities,
        declined_outside_gate_closure: gate_closure_declined,
        declined_discovery_excluded: discovery_excluded_declined,
        claims_planned,
        claims_executed: 0,
        receipt_identities: 0,
        not_attempted_after_abort: 0,
        passed: 0,
        known_red_held: 0,
        route_gap_held: 0,
        known_red_now_passing: 0,
        known_red_budget_refused: 0,
        known_red_passed_over_budget: 0,
        known_red_host_tool_unresolved_held: 0,
        known_red_host_effect_refused: 0,
        stale_quarantine: Vec::new(),
        interrupted_before_verdict: Vec::new(),
        completed_over_cost_requirement: Vec::new(),
        withheld_cost_debt: outcome_withheld_cost_debt,
        stale_cost_debt: Vec::new(),
        known_red_runtime_errored: Vec::new(),
        non_verdict_unenrolled: Vec::new(),
        stale_non_verdict: Vec::new(),
        known_red_observation_unreadable: Vec::new(),
        host_tool_unresolved: Vec::new(),
        route_gap: Vec::new(),
        stale_route_gap: Vec::new(),
        over_cost_line_diagnostic: 0,
        claim_cost: Vec::new(),
        failures: Vec::new(),
        required_floor_disposition: disposition_rows,
        long_home_storage_agreement: storage_agreement_rows,
        changed_witness_rows: 0,
        changed_witness_blocking: Vec::new(),
    };
    let mut receipted: HashSet<String> = HashSet::new();

    // SCOPES ARE DERIVED ONCE FROM THE EXACT MANIFEST, as an explicit table rather than a lazy
    // cache filled during the fold. A lazy cache would make "how many scopes exist" a question
    // about execution history instead of about the manifest, and the acceptance census asks for
    // distinct scopes constructed to EQUAL the manifest's distinct scope identities — which is
    // only checkable if the table is built before anything runs.
    floor_seam("claim-scope-projection");
    let scope_start = std::time::Instant::now();
    let distinct_scopes: std::collections::BTreeSet<&str> =
        claims.iter().map(|c| c.module_path.as_str()).collect();
    // SCOPE SIZE IS REPORTED, because it is the quantity every per-claim cost is proportional to.
    //
    // A context's lazily-built indexes are derived from the scope's module population and rebuilt
    // per claim, so a scope's module count multiplies by the number of claims that share it. When
    // the scope was an import closure that product was small; the reference closure changed the
    // multiplicand and the fold's cost moved with it. Reporting only the projection's own
    // duration hides that entirely: the projection is where scopes are BUILT, and the fold is
    // where their size is PAID.
    // SCOPES ARE NO LONGER RETAINED, and the census that required retaining them never did.
    //
    // The table above held one `PreparedClaimScope` per distinct claim module — 1,383 of them
    // over a 3,646-module corpus at a mean of 511.5 modules each, so every module's structures
    // were materialized around 192 times. Measured, that table WAS the floor's memory: manifest
    // evaluation sat flat at 9.94 GB and scope projection added 25 GB before a single witness
    // ran. It was built eagerly on the stated ground that "how many scopes exist" must be a
    // question about the manifest rather than about execution history.
    //
    // That ground is sound and the table was not what established it. The manifest's distinct
    // scope identities are exactly the distinct module paths its claims name, which is a fact
    // about `claims` — countable without constructing anything, and counted here BEFORE the
    // fold, so the census keeps the property it was built for.
    //
    // What replaces the table is a stream: the fold holds at most one scope, rebuilding when the
    // claim module changes. Claims arrive grouped by module, so the number of constructions
    // tracks the number of distinct scopes rather than the number of claims — and because that
    // is a property of the manifest's order rather than a guarantee, the fold COUNTS its
    // constructions and reports them against this number. A grouping that degrades shows up as
    // constructions far above distinct scopes, loudly, instead of as silent rebuilding.
    eprintln!(
        "floor: {} distinct claim scope(s) named by the manifest, counted in {}ms \
         (0 reads, 0 parses, 0 resolves, 0 scopes retained) corpus={}",
        distinct_scopes.len(),
        scope_start.elapsed().as_millis(),
        prepared.graph.modules.len()
    );

    floor_seam("claim-evaluation-fold");
    let eval_started = std::time::Instant::now();
    // AT MOST ONE SCOPE IS ALIVE. Rebuilt when the claim module changes, dropped when it is
    // replaced. See the projection note above for why the table it replaces was the floor's
    // memory and why its census survives without it.
    let mut current_scope: Option<(String, PreparedClaimScope)> = None;
    let mut scope_constructions: usize = 0;
    // TIME BESIDE MEMORY, BECAUSE THE MEMORY HALF ALONE COULD NOT SIZE A FIX. The scope note
    // below reads RSS either side of the one construction, so scope COST was observable and
    // scope TIME was not; likewise the per-claim frame was untimed entirely. Measured on run
    // 33366134453 that left ~150s of a 336.5s fold attributable to neither the shared-fill
    // ledger (~85s, and nine fills are almost all of it) nor the per-claim cost receipt
    // (~100s) -- a plurality of the fold's wall with no instrument pointing at it. These two
    // counters close that, and they are deliberately the same shape as the RSS pair: read
    // either side of the one call, so the delta is that call and not a sample near it.
    let mut scope_build_nanos: u128 = 0;
    let mut frame_build_nanos: u128 = 0;
    let mut frame_constructions: usize = 0;
    let mut scope_module_total: usize = 0;
    let mut scope_module_max: usize = 0;
    let mut terminal_rows: Vec<ClaimTerminalRow> = Vec::new();
    // `Some(identity)` exactly when a claim's evaluation unwound and stopped the fold.
    let mut halted_by: Option<String> = None;
    let mut known_red_held: usize = 0;
    let mut known_red_now_passing: usize = 0;
    let mut known_red_budget_refused: usize = 0;
    let mut known_red_passed_over_budget: usize = 0;
    let mut known_red_host_tool_unresolved: usize = 0;
    let mut known_red_host_effect_refused: usize = 0;
    let mut known_red_runtime_errored_count: usize = 0;
    // THE CAUSE CENSUS, GROUPED IN-PROCESS. A count of 172 throws is not 172 defects until
    // something says whether they share a root, and rendering one concentrated cause as
    // distributed debt is worse than the absorbing counter it replaces: it reads as honest
    // accounting while being wrong about the SHAPE of the problem, and a later reader prices
    // N repairs against what may be one fix. Grouping here costs a HashMap and answers it on
    // the same run that produces the count.
    let mut known_red_runtime_error_causes: HashMap<&'static str, usize> = HashMap::new();
    let mut known_red_observation_unreadable_count: usize = 0;
    // WHICH ENROLLED ROUTE-GAP IDENTITIES ACTUALLY GAPPED, for the reverse join below. Without
    // it the roster is a one-way lookup that only ever asks "is this gap enrolled" and never
    // "is this enrollment still real", which is exactly how a skip list rots.
    let mut route_gap_seen: HashSet<String> = HashSet::new();
    let mut route_gap_held: usize = 0;
    // IDENTITY GRAIN, NEVER COUNTS. The whole point of the roster is the case where one
    // identity is repaired while a different one begins throwing and the COUNT DOES NOT MOVE.
    let mut non_verdict_seen: HashSet<String> = HashSet::new();
    let mut non_verdict_detail: BTreeMap<String, String> = BTreeMap::new();
    let mut expected_red_seen: HashSet<String> = HashSet::new();
    let mut claim_rss_kb_max: u64 = 0;
    let mut claim_rss_kb_max_row = String::new();
    let mut trim_reclaimed_kb_total: u64 = 0;
    let mut trim_reclaimed_kb_max: u64 = 0;
    let mut trims_performed: u64 = 0;
    let mut scope_kb_total: u64 = 0;
    let mut scope_kb_max: u64 = 0;
    let mut scope_kb_max_module = String::new();
    let mut scope_kb_max_modules: usize = 0;
    let mut scopes_with_ambiguity: usize = 0;
    let mut scope_build_split = crate::cli_run::ScopeBuildSplit::default();
    let mut ambiguous_total: usize = 0;
    let mut ambiguous_max: usize = 0;
    let mut final_symbol_retention = None;
    for (index, claim) in claims.iter().enumerate() {
        if index % 1000 == 0 {
            eprintln!("floor: evaluating {index} / {claims_planned}");
        }
        if current_scope.as_ref().map(|(module, _)| module.as_str())
            != Some(claim.module_path.as_str())
        {
            // Dropped before the next is built, not after: holding both would put two scopes
            // resident at the seam and defeat the point of streaming them.
            drop(current_scope.take());
            // SCOPE COST, MEASURED AT THE SCOPE. The fold's resident set swings from 9.26GB to
            // 14.86GB and back down, which correlates with scope size but only correlates: the
            // heartbeat samples on a timer, so it cannot say whether the swing IS the scope or
            // something else moving alongside it. Read either side of the one construction and
            // the question stops being inferential. Taken with the old scope already dropped, so
            // the delta is this scope's cost and not the difference between two.
            let rss_before = current_rss_bytes().unwrap_or(0) / 1024;
            let scope_build_started = std::time::Instant::now();
            let built = claim_scope_for(&prepared, &claim.module_path)?;
            scope_build_nanos += scope_build_started.elapsed().as_nanos();
            let rss_after = current_rss_bytes().unwrap_or(0) / 1024;
            let scope_kb = rss_after.saturating_sub(rss_before);
            if scope_kb > scope_kb_max {
                scope_kb_max = scope_kb;
                scope_kb_max_module = claim.module_path.clone();
                scope_kb_max_modules = built.indexes.modules.len();
            }
            scope_kb_total += scope_kb;
            scope_constructions += 1;
            scope_build_split.accumulate(&built.build_split);
            scope_module_total += built.indexes.modules.len();
            scope_module_max = scope_module_max.max(built.indexes.modules.len());
            if built.ambiguous_bare_names > 0 {
                scopes_with_ambiguity += 1;
                ambiguous_total += built.ambiguous_bare_names;
                ambiguous_max = ambiguous_max.max(built.ambiguous_bare_names);
            }
            current_scope = Some((claim.module_path.clone(), built));
        }
        let scope = &current_scope
            .as_ref()
            .expect("a scope was just built for this claim's module")
            .1;
        // FRESH PER CLAIM. Claims sharing one immutable scope must not share the mutable
        // evaluation caches a context owns, or one witness contaminates the next through a
        // memo rather than through anything it declared.
        let frame_build_started = std::time::Instant::now();
        let frame = evaluation_frame(scope, claim.execution_mode, None, published.clone());
        frame_build_nanos += frame_build_started.elapsed().as_nanos();
        frame_constructions += 1;
        // ARM THE WALL CEILING, which is what the operator's rule has always been about and
        // what this path was not doing. `run_claim_measured` already arms the deadline and
        // applies a completion-side backstop when a wall budget is set -- the mechanism was
        // complete in the interpreter and simply never switched on here, so a CPU budget stood
        // in for it while printing the wall rule's own error text.
        //
        // Both clocks are armed deliberately, and INDEPENDENTLY (operator ruling 2026-08-19,
        // BUDGET POLICY CUT, superseding correction — "DO NOT set CPU and wall to the same
        // 5000ms"). CPU catches a spin; wall catches a witness that is slow because of what it
        // reaches for, which CPU cannot see: the worst row measured burned 504 SECONDS of wall
        // under a 5-second ceiling and returned an ordinary Bool, because its time was
        // filesystem reads and its CPU never approached the limit. The wall limit is
        // deliberately looser than the CPU limit so ordinary host scheduling delay on a pure
        // in-process claim cannot itself trip an interrupt while the claim is still within its
        // CPU envelope.
        //
        // WHICH CLOCK IS ARMED IS THE CLAIM'S COST POLICY, and only the CPU one moves
        // (`v2.workflow.required_floor` `changed_witness_cpu_deadline`, FLOOR-CHANGED-COST-0).
        // Under `ChangedCostDebtVerdictOnly` the CPU deadline is NOT armed, so the interrupt
        // cannot preempt the verdict the changed set exists to learn; the same 500ms figure is
        // still carried on the claim and is published against the debt identity below. The wall
        // budget is armed identically under both policies — a claim that is blocked or stuck
        // still reaches no verdict, and that is still a red.
        frame.set_witness_eval_budget(match claim.cost_policy {
            ChangedWitnessCostPolicy::Ordinary => Some(claim.cpu_safety_limit_ms),
            ChangedWitnessCostPolicy::ChangedCostDebtVerdictOnly => None,
        });
        frame.set_witness_wall_budget(Some(claim.wall_safety_limit_ms));
        // NAME WHO IS RUNNING, so a shared computation filled during this claim is attributed to
        // it rather than to nobody. The wall time this row is about to be charged is not
        // necessarily its own; `shared_fill` records which part was a first touch of a
        // process-global corpus cache and which later claims read that same fill for free.
        shared_fill::set_current_claim(Some(&claim.qualified));
        set_phase(FloorPhase::Eval, &claim.qualified);
        // WHAT ONE CLAIM COSTS IN MEMORY, for the same reason the scope is measured beside it:
        // the fold's resident set swings ~5.6GB and the scope turned out to account for 0.02GB
        // of it, so the remainder is unattributed and the only honest way to attribute it is to
        // read it where it happens. A claim's frame owns mutable evaluation caches (call memo,
        // data cache, name caches) that live exactly as long as the claim, so a row that walks
        // a large population can cost far more than the scope it walks.
        //
        // This matters beyond memory now: the runner throttles at `memory.high`, and under
        // throttling WALL time inflates while thread-CPU does not — so a row that drives the
        // resident set into the watermark inflates the wall measurement of every row near it,
        // including its own. Naming the rows that cost the most is therefore the first step in
        // separating expensive witnesses from witnesses that merely ran next to one.
        let claim_rss_before = current_rss_bytes().unwrap_or(0) / 1024;
        let (result, receipt) =
            run_claim_measured(&frame, &prepared.subject_digest, &claim.qualified);
        final_symbol_retention = Some(frame.interner_stats_snapshot());
        // FOLD THIS CLAIM'S RECOMPUTE LEDGER INTO THE CROSS-CLAIM CENSUS BEFORE THE FRAME DIES.
        // The ledger is per-frame and reports only keys re-hit WITHIN one claim, so a producer
        // this closure re-derives exactly once per claim is invisible to it in every claim
        // separately — which is the population `v2.workflow.floor_pure_producer_share` is
        // enrolled from. Absorbed here, one line after the measurement and before the next
        // frame is built, so the census's claim grain is the loop's own.
        v1_interpreter::absorb_claim_recompute_demand(&frame, &claim.qualified, &claim.module_path);
        let claim_rss_after = current_rss_bytes().unwrap_or(0) / 1024;
        let claim_rss_kb = claim_rss_after.saturating_sub(claim_rss_before);
        if claim_rss_kb > claim_rss_kb_max {
            claim_rss_kb_max = claim_rss_kb;
            claim_rss_kb_max_row = claim.qualified.clone();
        }
        // A quarter of a gigabyte in ONE row is loud, because a row that costs that much is
        // both a memory subject in its own right and the reason its neighbours' wall times
        // cannot be trusted. Threshold is a reporting choice, not a verdict: nothing refuses on
        // it, it only makes the population visible so it can be ranked.
        if claim_rss_kb > 262_144 {
            eprintln!(
                "[floor-claim-memory] {} grew rss by {:.2}GB (to {:.2}GB)",
                claim.qualified,
                claim_rss_kb as f64 / 1048576.0,
                claim_rss_after as f64 / 1048576.0
            );
        }
        // DIAGNOSTIC-ONLY COST-LINE COMPARISON. This is the executing consumer of
        // `claim_terminality`/`exceeds_completed_cost_line`/`ClaimTerminality`/
        // `RequiredFloorCostBasis` (BUDGET POLICY CUT, 2026-08-19) — reported and counted so the
        // types are not review §6 scaffolds, never gated on: `over_cost_line_diagnostic` has no
        // reader that fails the run, unlike `completed_over_cost_requirement` above, which blocks
        // on the safety limit and is computed independently of this comparison.
        let terminality = claim_terminality(
            &result,
            &receipt,
            WitnessSafetyPolicy {
                cpu_ms: claim.cpu_safety_limit_ms,
                wall_ms: claim.wall_safety_limit_ms,
            },
        );
        // MINT THE OCCURRENCE HERE, before any classification can diverge from it. The
        // over-cost population, its count and the identity-grain receipt are all folds over
        // `outcome.claim_cost` computed after the loop -- so the summary cannot disagree with
        // its own members, and there is no separately incremented tally to drift.
        outcome.claim_cost.push(WitnessExecutionOccurrence {
            identity: claim.qualified.clone(),
            module_path: claim.module_path.clone(),
            outcome: witness_execution_outcome_label(&result).to_string(),
            // MINT THE READING FROM THE TERMINALITY, not from the receipt. The receipt's two
            // clocks are the same numbers either way; what `claim_terminality` adds — and what
            // reading them straight off the receipt DISCARDED — is whether a deadline preempted
            // the subject, which is the only thing that decides if those numbers are
            // measurements or lower bounds.
            reading: ClaimCostReading::of(&terminality),
            eval_steps: receipt.eval_steps,
            verdict_reached: matches!(terminality, ClaimTerminality::VerdictReached { .. }),
            cost_line_ms: claim.cost_line_ms,
            preemption_reachability: preemption_reachability_label(&receipt.opaque_host_call_reach),
        });
        // PUBLISH THE COST AGAINST THE DEBT IDENTITY (FLOOR-CHANGED-COST-0, operator ruling
        // 2026-08-30). Standing down a gate without putting a measurement in its place is the
        // absorbing-fallback shape DESIGN §5 forbids — the deficit stops being counted at the
        // moment it stops blocking — so an override that publishes nothing is refused by the
        // projection below (`CostObservationMissingUnderVerdictOnly`) rather than passing as an
        // ordinary green. The figures are the marginal ones `run_claim_measured` netted, which
        // is the same quantity the CPU deadline would have enforced against.
        if claim.cost_policy == ChangedWitnessCostPolicy::ChangedCostDebtVerdictOnly {
            let observation = ChangedWitnessCostObservation {
                cpu_clock_nanos: receipt.cpu_nanos,
                wall_clock_nanos: receipt.wall_nanos,
                cpu_line_ms: claim.cpu_safety_limit_ms,
            };
            eprintln!(
                "[floor-cost-debt-observation] identity={} standing={} marginal_cpu_ms={} \
                 wall_ms={} cpu_line_ms={} verdict={} roster_row=retained",
                claim.qualified,
                cost_debt_roster_standing_label(
                    &CostDebtRosterStanding::WithholdOverriddenForChangedVerdict
                ),
                observation.cpu_clock_nanos / 1_000_000,
                observation.wall_clock_nanos / 1_000_000,
                observation.cpu_line_ms,
                witness_execution_outcome_label(&result),
            );
            cost_debt_observations.insert(claim.qualified.clone(), observation);
        }
        // RETURN WHAT THE FRAME NO LONGER OWNS, on a cadence rather than every row.
        //
        // The frame is dropped at the end of this iteration, so by the next trim its caches are
        // free as far as Rust is concerned; whether they are free as far as the KERNEL is
        // concerned is the question above. Every 200 claims is a deliberate compromise: often
        // enough that the resident set cannot drift far between samples, rare enough that the
        // trim's own cost (it walks the arena) is not paid ten thousand times. Nothing refuses
        // on the result and nothing is skipped because of it — this only gives memory back and
        // says how much, so a run that reclaims nothing is strictly the run we already had.
        if index % 200 == 199 {
            if let Some(reclaimed_kb) = trim_retained_heap() {
                trim_reclaimed_kb_total += reclaimed_kb;
                trim_reclaimed_kb_max = trim_reclaimed_kb_max.max(reclaimed_kb);
                trims_performed += 1;
            }
        }
        outcome.claims_executed += 1;
        receipted.insert(claim.qualified.clone());
        // ROSTER MEMBERSHIP IS READ HERE, BEFORE THE LINE PRINTS. It used to be computed a few
        // dozen lines below, purely to pick a counter -- so the console had already committed to
        // FAILED by the time anything knew the row was enrolled. Moving the read above the print
        // is the fix; the branch below still owns the counters and the receipts, and reads the
        // same `expected_red_roster` set, so there is one authority and two consumers rather
        // than two answers.
        style.stream_witness(
            &claim.function,
            &claim.module_path,
            "PreparedSubject",
            receipt.wall_nanos,
            CiWitnessVerdict::from_outcome(
                &result,
                expected_red_roster.contains(claim.qualified.as_str()),
            ),
        );
        // THE EXPECTED-RED JOIN. A quarantined identity is one this branch KNOWS fails; it is
        // enrolled by exact qualified name in `v2.workflow.floor_expected_red`, and the
        // difference from an exclusion is that it still RUNS and its outcome is still asserted.
        //
        // Three things follow, and they are the whole reason this is admissible where a skip
        // list is not. A failure outside the roster still reds the build, so the roster covers
        // named debt rather than a category. A quarantined row that PASSES also reds, so a fix
        // is counted the moment it lands instead of being silently absorbed into a green run —
        // which is what makes the roster monotonically shrinking rather than a place things go
        // to be forgotten. And every row is an executing claim with a receipt, so nothing in
        // here reads as covered when it is not.
        let expected_red = expected_red_roster.contains(claim.qualified.as_str());
        // THE LINE STOPS HERE, ahead of every branch below it.
        //
        // BREAKING RATHER THAN CONTINUING IS THE RULING (operator, 2026-08-26), and the argument
        // is not caution. Every other non-verdict in this fold is a state the producer DECIDED to
        // return, so the process's invariants held and the next claim starts from a known state;
        // a panic is an invariant violated at an unknown place, and every row measured after it
        // would carry an unstated precondition — measured after an unwind of unknown extent —
        // that no row in this ledger can express. A green row after the unwind and a green row
        // before it would render identically, which is the execution-provenance conflation this
        // artifact exists to prevent. DESIGN §5's factory model settles the direction: a deficit
        // stops the line, and the stopped-line audit reports rather than greens.
        //
        // ITS TERMINAL ROW IS MINTED HERE rather than at the shared site below, because breaking
        // skips that site. The identity is therefore IN the ledger, and where the rows end is not
        // something a reader has to infer.
        //
        // AHEAD OF THE EXPECTED-RED JOIN, deliberately: a panic is not the enrolled failure, so
        // it must not reach an arm that could hold it — the same rule this fold already applies
        // to a runtime error, a budget interruption, an unresolved host tool and a route gap.
        // ONE CONSEQUENCE IS DECLARED RATHER THAN HIDDEN: an ENROLLED identity that panics never
        // reaches `record_observed`, so the expected-red roster join reports it as
        // `NotEvaluated { reason: "not_observed" }`. The DISPOSITION is exactly right — no
        // verdict was observed — and the REASON is generic where "panicked" would be specific.
        // Naming the cause there needs a `Panicked` arm on `WitnessEvalVerdict`, which is a
        // GENERATED carrier (`src/v1/expected_red_roster_join.dag` → its committed mirror), so it
        // lands with that module's next regeneration and not by hand here. The cause is not lost
        // meanwhile: the terminal ledger carries it as the row's detail, and `failures` names it.
        if let ClaimOutcome::Panicked { payload } = &result {
            terminal_rows.push(ClaimTerminalRow {
                qualified: claim.qualified.clone(),
                expected_red,
                outcome: result.clone(),
            });
            outcome.failures.push(format!(
                "{} PANICKED during evaluation: {payload}. This is not a verdict and not a \
                 runtime error the evaluator raised — the host unwound, so the claim produced \
                 nothing to judge. The fold STOPS here: every later row would be measured in a \
                 process that unwound through an unknown place, and no row could say so. Every \
                 planned identity behind this one is published as a not-attempted row naming \
                 this identity as what halted the line.",
                claim.qualified
            ));
            halted_by = Some(claim.qualified.clone());
            break;
        }
        if expected_red {
            expected_red_seen.insert(claim.qualified.clone());
            if let Some(ref mut join) = roster_join_report {
                if let Some(verdict) = witness_eval_verdict_from_claim_outcome(&result) {
                    *join = crate::v1_compiler_expected_red_roster_join::record_observed(
                        join.clone(),
                        claim.qualified.clone(),
                        std::rc::Rc::new(verdict),
                    );
                }
            }
        }
        // ONE ROW PER PLANNED IDENTITY, MINTED HERE BECAUSE THIS IS THE ONLY POINT BEFORE THE
        // BRANCHING. Every expected-red arm below `continue`s, and the ordinary match below has
        // its own arms; minting in either place would silently omit whichever population took
        // the other path, and an omission is exactly what this ledger exists to make visible.
        terminal_rows.push(ClaimTerminalRow {
            qualified: claim.qualified.clone(),
            expected_red,
            outcome: result.clone(),
        });
        let passed = matches!(result, ClaimOutcome::Pass);
        if expected_red {
            // ONE DISPATCH. Every arm does its own work here rather than classifying once and
            // re-deriving the answer below: two dispatches over one value agree only as long
            // as nobody adds a variant, and the second test is always the narrower one, so the
            // new variant reaches the fallthrough and is silently held. That is precisely the
            // absorption this join exists to remove, and it would read as correct in review
            // because the helper LOOKS like it classifies. With one match the compiler makes
            // the next variant get classified here or not compile, and the caller's sum check
            // is then checking three counters produced by one mechanism rather than two.
            match expected_red_arm(&result) {
                ExpectedRedArm::NowPassing => {
                    known_red_now_passing += 1;
                    outcome.stale_quarantine.push(format!(
                        "{} is enrolled as expected-red and PASSED — remove it from \
                         v2.workflow.floor_expected_red",
                        claim.qualified
                    ));
                    continue;
                }
                // A BUDGET REFUSAL IS NOT AN ENROLLED FAILURE, and conflating the two is what
                // let the most expensive row in the corpus hide behind its own enrollment.
                // Enrollment records that this branch expects the claim to FAIL — a statement
                // about the witness's verdict. A budget refusal is not a verdict: it is an
                // interruption plus a measured lower bound on cost, so the enrolled claim was
                // never decided at all. Holding it reports agreement about a failure that
                // nobody observed.
                ExpectedRedArm::BudgetRefused => {
                    known_red_budget_refused += 1;
                    // ONE RENDERER, NOT A LOCAL FORMAT STRING. The arm reached here is the
                    // interrupted one BY CONSTRUCTION — `ExpectedRedArm::BudgetRefused` has
                    // exactly one producer — so `budget_figure_phrase` answers `Some` and
                    // renders `cost=UNMEASURED` with the interrupt point in its own field. The
                    // local string this replaced said `cost at least {n}ms against {budget}ms`,
                    // which is the bound-in-the-cost-field reading the renderer exists to make
                    // unwritable; the debug fallback stays for the same reason it always did.
                    let detail = result
                        .budget_figure_phrase()
                        .unwrap_or_else(|| format!("{result:?}"));
                    // TWO DERIVATIONS OVER ONE CLAIM, AND A DISAGREEMENT REFUSES. This arm is
                    // reached only for `ClaimOutcome::BudgetInterrupted`, and `claim_terminality`
                    // maps exactly that outcome to `SafetyInterrupted` — so `None` here means the
                    // outcome classifier and the terminality classifier disagree about whether
                    // this claim was interrupted at all. Substituting zeroes would publish a
                    // fabricated reading; the line stops instead, typed and located (DESIGN §5).
                    let Some(interrupt) = safety_interrupt_reading(&terminality) else {
                        outcome.failures.push(format!(
                            "{} classified as BUDGET-REFUSED by ClaimOutcome but its \
                             ClaimTerminality is not SafetyInterrupted. The two derivations \
                             over one claim disagree, so no interrupt reading can be published \
                             for it. This is a defect in the classifiers, not in the witness.",
                            claim.qualified
                        ));
                        continue;
                    };
                    outcome
                        .interrupted_before_verdict
                        .push(InterruptedBeforeVerdict {
                            qualified: claim.qualified.clone(),
                            interrupt,
                            enrolled_expected_red: true,
                            // THE IDENTITY IS THE ROW'S FIELD AND IS NOT RESTATED HERE. It used
                            // to lead this sentence, which made `qualified` a second
                            // representation of a string a reader would otherwise have to
                            // recover by parsing prose. The printer renders the field.
                            detail: format!(
                                "is enrolled as expected-red but was BUDGET-REFUSED, not failed. \
                         {}. Enrollment asserts an expected verdict and a budget refusal \
                         produces none, so the enrolled claim went undecided — THIS ROW'S \
                         CORRECTNESS IS UNKNOWN, not merely expensive: the refusal preempted \
                         the verdict, so a content defect here would be indistinguishable from \
                         the enrolled failure. Reducing the row's cost, or moving it to a lane \
                         that declares its own ceiling, is what lets it reach a verdict at all; \
                         removing it from the roster would not help, because it is not passing \
                         either.",
                                detail
                            ),
                        });
                    continue;
                }
                // NOT COUNTED AS A PASS. A held row did not pass — it failed as enrolled, and
                // agreement about a failure is not the same fact as a passing witness. Folding
                // it into `passed` would make the headline number rise as debt is ADDED, which
                // is the direction that flatters, and would leave no count that falls when the
                // debt is repaid. The identity accounting (planned = executed = receipted) is
                // unaffected because it counts receipts, not verdicts.
                // BOTH REMEDIES, because the row is true on both axes. The semantic half goes
                // to stale_quarantine (the claim passed, so the roster row must come out) and
                // the cost half to completed_over_cost_requirement (an exact overrun that still
                // has to be paid down). Choosing one would silently drop the other, and the one the code used
                // to drop was the repaid-debt signal the roster exists to surface.
                ExpectedRedArm::PassedOverBudget => {
                    known_red_passed_over_budget += 1;
                    // ONE RENDERER HERE TOO, though this arm's figure was never a bound: a
                    // completed row has an exact cost. It routes through
                    // `budget_figure_phrase` anyway so that the completed and interrupted
                    // sentences keep being authored in one place — a local "exactly" template
                    // beside a shared renderer is how the two readings drift back into one
                    // shape.
                    let cost = result
                        .budget_figure_phrase()
                        .unwrap_or_else(|| format!("{result:?}"));
                    outcome.stale_quarantine.push(format!(
                        "{} is enrolled as expected-red and PASSED (then exceeded its budget: \
                         {}) — remove it from v2.workflow.floor_expected_red; the cost debt is \
                         reported separately and is not a reason to keep the row",
                        claim.qualified, cost
                    ));
                    outcome.completed_over_cost_requirement.push(format!(
                        "{} PASSED but exceeded its budget. {}. Reduce it, or move the row to a \
                         lane declaring its own ceiling.",
                        claim.qualified, cost
                    ));
                    continue;
                }
                ExpectedRedArm::HostToolUnresolved => {
                    known_red_host_tool_unresolved += 1;
                    let detail = match &result {
                        ClaimOutcome::HostToolUnresolved { name, probed } => format!(
                            "host tool unresolved: {name:?} (probed: {})",
                            probed.join(", ")
                        ),
                        other => format!("{other:?}"),
                    };
                    outcome.host_tool_unresolved.push(format!(
                        "{} is enrolled as expected-red but HOST-TOOL-UNRESOLVED, not failed \
                         and not budget-refused: {}. Enrollment asserts an expected verdict; \
                         missing host tooling produces none. Fix the tool chain or run on a host \
                         that provides it — do not chase witness cost on an infra gap.",
                        claim.qualified, detail
                    ));
                    continue;
                }
                // A ROUTE GAP IS NOT AGREEMENT, for the same reason the two arms above are not.
                // Enrollment asserts an expected VERDICT; a claim whose route had no arm for a
                // host effect never reached its subject and produced none. Holding it would let
                // an enrollment silently cover a witness that has not run since the day it was
                // enrolled — the failure this whole lane exists to close.
                ExpectedRedArm::HostEffectRefused => {
                    known_red_host_effect_refused += 1;
                    let (operation, ground, detail) = match &result {
                        ClaimOutcome::HostEffectRefused { operation, ground } => (
                            operation.as_str(),
                            ground,
                            format!(
                                "hermetic route has no arm for {operation}: {}",
                                hermetic_effect_ground_label(ground)
                            ),
                        ),
                        other => {
                            return Err(format!(
                                "required-floor expected-red arm/result disagreement for {}: {other:?}",
                                claim.qualified
                            ));
                        }
                    };
                    // THE TWO ROSTERS ARE DIFFERENT AXES, AND THIS ROW SITS ON BOTH. Being
                    // enrolled as expected-red says nothing about whether the floor has a route
                    // that can run the identity, so the route-gap roster is consulted here
                    // exactly as it is for an unenrolled row — the expected-red enrollment does
                    // not cover the gap, and the gap does not discharge the enrollment.
                    route_gap_seen.insert(claim.qualified.clone());
                    if route_gap_roster.contains(claim.qualified.as_str()) {
                        if let Some(mismatch) = floor_route_gap_expectation_mismatch(
                            route_gap_expectations.get(claim.qualified.as_str()),
                            operation,
                            ground,
                        ) {
                            outcome.route_gap.push(format!(
                                "{} is enrolled as a route gap, but its observed typed route changed: {}. Update the enrollment only after deciding which route the witness actually requires.",
                                claim.qualified, mismatch
                            ));
                        } else {
                            route_gap_held += 1;
                        }
                    } else {
                        outcome.route_gap.push(format!(
                            "{} is enrolled as expected-red but ROUTE-GAPPED, not failed: {}. \
                             Enrollment asserts an expected verdict; a claim that never reached \
                             its subject produced none. Supply the route (publish the mock case, \
                             author the mock_response, or supply a lane that can run the effect) \
                             — do not read this as the enrolled failure.",
                            claim.qualified, detail
                        ));
                    }
                    continue;
                }
                // A THROW IS NOT THE ENROLLED FAILURE. Same shape as the three arms above and
                // for the same reason: enrollment asserts an expected VERDICT, and a claim that
                // threw produced none. Held here would keep a witness enrolled forever on the
                // strength of an error, which is the exact rot this lane exists to surface.
                ExpectedRedArm::RuntimeErrored => {
                    known_red_runtime_errored_count += 1;
                    // KEYED ON THE TYPED CAUSE, NOT ON THE PROSE. The previous key was the
                    // first twelve whitespace-separated words of the message — which EMBEDS THE
                    // MISSING NAME, so `no declaration named X` and `no declaration named Y`
                    // counted as two distinct causes. Measured on main `f9963a762`: 65 distinct
                    // "signatures" across 142 identities, for a population with four actual
                    // causes. That is not an imprecise number, it is an inverted one — it told
                    // every reader "many roots" where the truth is "one root, many names", and
                    // pointed them away from the single repair that closes most of the
                    // population. The comment it replaced described the key as normalizing away
                    // per-row identities; it did the opposite.
                    let detail = match &result {
                        ClaimOutcome::RuntimeError { cause, message } => {
                            *known_red_runtime_error_causes
                                .entry(cause.token())
                                .or_insert(0) += 1;
                            format!("runtime error [{}]: {message}", cause.token())
                        }
                        // NOT a fallback that guesses. Only `RuntimeErrored` reaches this arm and
                        // only `RuntimeError` produces it, so this is unreachable in fact; it is
                        // kept, and kept LOUD, because unreachable is not the same as absent and
                        // a silent `_ => ()` here would hide a real routing defect.
                        other => {
                            *known_red_runtime_error_causes
                                .entry("routing-defect-non-runtime-error-in-runtime-errored-arm")
                                .or_insert(0) += 1;
                            format!("{other:?}")
                        }
                    };
                    non_verdict_seen.insert(claim.qualified.clone());
                    non_verdict_detail.insert(claim.qualified.clone(), detail.clone());
                    outcome.known_red_runtime_errored.push(format!(
                        "{} is enrolled as expected-red but RUNTIME-ERRORED, not failed: {}. \
                         Enrollment asserts an expected verdict; a claim that threw \
                         produced none. Repair the witness or its subject, then \
                         re-read the enrollment — do not read this as the enrolled \
                         failure.",
                        claim.qualified, detail
                    ));
                    continue;
                }
                ExpectedRedArm::ObservationUnreadable => {
                    known_red_observation_unreadable_count += 1;
                    non_verdict_seen.insert(claim.qualified.clone());
                    non_verdict_detail.insert(
                        claim.qualified.clone(),
                        format!("returned {result:?}, which is not a Bool"),
                    );
                    outcome.known_red_observation_unreadable.push(format!(
                        "{} is enrolled as expected-red but returned something that is NOT A \
                         VERDICT ({:?}), so it is neither the enrolled failure nor a \
                         repayment. Enrollment asserts an expected verdict; an \
                         unreadable observation is none. Make the witness return a \
                         Bool, then re-read the enrollment.",
                        claim.qualified, result
                    ));
                    continue;
                }
                ExpectedRedArm::Held => {
                    known_red_held += 1;
                    continue;
                }
                // QUIET, NOT DEAD — see `ExpectedRedArm::Aborted`. The fold breaks on a panic
                // above this match, so nothing reaches here today; if a future edit reorders
                // those two blocks this reports the fact rather than holding the enrollment on
                // a claim that produced no verdict, which is the absorption every arm above
                // exists to refuse.
                ExpectedRedArm::Aborted => {
                    outcome.failures.push(format!(
                        "{} is enrolled as expected-red and its attempt was ABORTED (the host \
                         unwound, or the fold had already stopped). Enrollment asserts an \
                         expected verdict; an aborted attempt produced none, so this is not the \
                         enrolled failure.",
                        claim.qualified
                    ));
                    continue;
                }
            }
        }
        match result {
            // `outcome.passed += 1` USED TO LIVE HERE AND IS DELETED, not left beside the
            // derivation. Two authorities over one fact is what this change removes; keeping the
            // counter here "as a cross-check" would preserve exactly the disagreement the
            // derivation makes unrepresentable. `passed` is now counted from the terminal rows
            // after the fold.
            ClaimOutcome::Pass => {}
            ClaimOutcome::Fail => outcome
                .failures
                .push(format!("{} returned Bool(false)", claim.qualified)),
            // Every non-pass arm is reported with the fact that distinguishes it. A
            // collapsed "failed" would make a budget refusal, a runtime error and a
            // witness that answered false read alike, and those three have different
            // remedies.
            ClaimOutcome::NotBool { got } => outcome
                .failures
                .push(format!("{} answered {got}, not a Bool", claim.qualified)),
            ClaimOutcome::RuntimeError { message, .. } => outcome
                .failures
                .push(format!("{} errored: {message}", claim.qualified)),
            ClaimOutcome::HostToolUnresolved { name, probed } => outcome.failures.push(format!(
                "{} host tool unresolved: {:?} (probed: {})",
                claim.qualified,
                name,
                probed.join(", ")
            )),
            // NOT A FAILURE, AND NOT GREEN. A route gap goes to its own blocking collection
            // rather than to `failures`, because reporting it as a failure says the witness is
            // wrong about its subject when the witness was never given a way to reach it — and
            // the two have different remedies. It still stops the line.
            ClaimOutcome::HostEffectRefused { operation, ground } => {
                route_gap_seen.insert(claim.qualified.clone());
                if route_gap_roster.contains(claim.qualified.as_str()) {
                    if let Some(mismatch) = floor_route_gap_expectation_mismatch(
                        route_gap_expectations.get(claim.qualified.as_str()),
                        &operation,
                        &ground,
                    ) {
                        outcome.route_gap.push(format!(
                            "{} is enrolled as a route gap, but its observed typed route changed: {}. Update the enrollment only after deciding which route the witness actually requires.",
                            claim.qualified, mismatch
                        ));
                    } else {
                        route_gap_held += 1;
                    }
                } else {
                    outcome.route_gap.push(format!(
                        "{} never reached its subject: the hermetic route has no arm for {} \
                         ({}). Supply the route — publish the mock case, author the \
                         mock_response, or supply a lane that can run the effect. Enrolling the \
                         identity in v2.workflow.floor_route_gap records the gap as known debt; \
                         it does not make the gap acceptable.",
                        claim.qualified,
                        operation,
                        hermetic_effect_ground_label(&ground)
                    ));
                }
            }
            // AND AN UNENROLLED BUDGET REFUSAL IS NOT A DEFECT EITHER. The enrolled arm above
            // already rules that a budget refusal produces no verdict and therefore is not a
            // failure; that fact is a property of the interruption, not of the roster, so it
            // holds identically for a row nobody enrolled. Reporting it in `failures` said the
            // opposite — `failures` is the channel whose remedy is "fix the defect", and it is
            // what the alert signature reads to distinguish a regression from a cost debt. A
            // row that was preempted before answering has no defect to fix and may well be
            // passing, so routing it here made an unmeasured cost indistinguishable from a
            // broken witness, in the direction that manufactures alarm.
            //
            // The consequence this closes is concrete: a row that PASSES and exceeds its budget
            // had no honest state anywhere. Enrolled, it asserted an expected failure that does
            // not occur and reported twice. Unenrolled, it landed here and read as a defect.
            // Cost is not a verdict, so the verdict channels cannot carry it — and now they do
            // not. The line still stops, because the cost is still owed; it stops saying the
            // true thing about why.
            // TWO READINGS, AND THEY ARE NOT THE SAME CLAIM. `Interrupted` means the deadline
            // fired before the witness answered: no verdict exists, the figure is a LOWER BOUND
            // and the row's real cost is unmeasured. `CompletedOverBudget` means the witness ran
            // to completion and then was found over budget: the verdict IS known and the figure
            // is EXACT. Printing one sentence for both would repeat the conflation this arm
            // exists to remove — asserting "correctness unknown" over a row that demonstrably
            // answered is as wrong as calling a cost a defect.
            // THE FIGURES COME FROM `budget_figure_phrase` AND THE PROSE NO LONGER RESTATES
            // THEM. What used to stand here was `cost at least {n}ms against {budget}ms`
            // followed by a caveat saying that number was not a cost — a caveat that competes
            // with a number already read in the cost position, and demonstrably loses. The
            // renderer does not put the bound there at all, so the remedy sentences below need
            // only say what to DO, which is the part a caveat was never the right carrier for.
            ClaimOutcome::BudgetInterrupted { .. } => {
                let figure = result
                    .budget_figure_phrase()
                    .unwrap_or_else(|| format!("{result:?}"));
                // SAME REFUSAL AS THE ENROLLED ARM, and for the same reason — see there.
                let Some(interrupt) = safety_interrupt_reading(&terminality) else {
                    outcome.failures.push(format!(
                        "{} classified as BUDGET-REFUSED by ClaimOutcome but its \
                         ClaimTerminality is not SafetyInterrupted. The two derivations over \
                         one claim disagree, so no interrupt reading can be published for it. \
                         This is a defect in the classifiers, not in the witness.",
                        claim.qualified
                    ));
                    continue;
                };
                outcome
                    .interrupted_before_verdict
                    .push(InterruptedBeforeVerdict {
                        qualified: claim.qualified.clone(),
                        interrupt,
                        enrolled_expected_red: false,
                        detail: format!(
                            "was BUDGET-REFUSED and went UNDECIDED. {}. Not enrolled as \
                     expected-red, so nothing claims it is broken — but the deadline preempted \
                     the verdict, so whether it PASSES is UNKNOWN too. Reduce the cost, or move \
                     it to a lane declaring its own ceiling, so the witness reaches a verdict.",
                            figure
                        ),
                    })
            }
            ClaimOutcome::CompletedOverBudget { .. } => {
                let figure = result
                    .budget_figure_phrase()
                    .unwrap_or_else(|| format!("{result:?}"));
                // WHICH POPULATION THIS CROSSING IS IN, NAMED IN THE SENTENCE THAT BLOCKS.
                //
                // The two are different facts with different remedies and they were arriving
                // here as one bucket. For a `cooperatively_pollable` claim the deadline had
                // every stride point available and the claim finished anyway, so the crossing is
                // an overshoot between two polls -- a charge. For an
                // `opaque_host_call_unbounded` claim no stride point falls inside the operation,
                // so the deadline OBSERVED NOTHING and the crossing is a missed interrupt, which
                // is the case `claim_safety_outcome_blocks` exists for.
                //
                // BOTH STILL BLOCK, IDENTICALLY. This names the population; it does not decide
                // admission, and no arm below returns early or skips the push. Whether the
                // pollable population should keep blocking is the open question this column
                // exists to make answerable with counts instead of argument -- it stays with the
                // operator, and `gunbc.rung_drop` `floor_cost_claim_qualification_unavailable`
                // stays standing until then.
                let reachability = outcome
                    .claim_cost
                    .iter()
                    .find(|row| row.identity == claim.qualified)
                    .map(|row| row.preemption_reachability.clone())
                    .unwrap_or_else(|| "surface_unarmed".to_string());
                let population = if reachability.starts_with("opaque_host_call_unbounded") {
                    format!(
                        "MISSED INTERRUPT: the deadline could not fire — this claim's cost \
                         accrued inside {}, where no stride poll lands, so completing under the \
                         ceiling was never something the limit could enforce",
                        reachability
                            .strip_prefix("opaque_host_call_unbounded:")
                            .unwrap_or(&reachability)
                    )
                } else if reachability == "cooperatively_pollable" {
                    "OVERSHOOT: every stride poll was reachable and the claim completed anyway, \
                     so the deadline missed nothing — what this crossing reports is a charge, \
                     not a failed interrupt"
                        .to_string()
                } else {
                    "REACHABILITY UNOBSERVED: the opaque-host-call surface was not armed for \
                     this run, so which population this crossing belongs to is unknown and is \
                     NOT assumed to be the benign one"
                        .to_string()
                };
                outcome.completed_over_cost_requirement.push(format!(
                    "{} reached its verdict and then exceeded its budget. {}. {}. The cost is \
                     therefore known and actionable. This is a cost debt only — it is not a \
                     defect and it does not belong on the expected-red roster, which asserts an \
                     expected FAILURE this row does not exhibit.",
                    claim.qualified, figure, population
                ))
            }
            // NEITHER REACHES THIS MATCH, and both are named rather than wildcarded. A panic
            // breaks the loop above this point, and a not-attempted row is minted after the loop
            // and never classified here at all. Naming them keeps this match total over the
            // outcome vocabulary, so a future arm cannot arrive through a `_` that reports it as
            // one of the classes above.
            ClaimOutcome::Panicked { payload } => outcome.failures.push(format!(
                "{} PANICKED and reached the post-loop classification, which is unreachable \
                 while the fold breaks on a panic: {payload}",
                claim.qualified
            )),
            ClaimOutcome::NotAttempted { halted_by } => outcome.failures.push(format!(
                "{} was classified as not-attempted inside the fold (halted_by={halted_by}), \
                 which only mints such rows after it",
                claim.qualified
            )),
        }
    }
    outcome.receipt_identities = receipted.len();
    // THE LEDGER IS PUBLISHED OVER THE PLANNED POPULATION, NEVER OVER THE PREFIX THAT RAN.
    //
    // A stopped fold that simply ended would publish a short ledger, and a short ledger is
    // indistinguishable from a narrowed roster: `stopped at claim 400 of 10439` and `10039 rows
    // quietly missing` are the same artifact. So every planned identity the fold never reached
    // gets its own terminal, naming the identity whose unwind halted the line. These rows are NOT
    // executions — nothing evaluated them, they hold no receipt, and `claims_executed` is
    // deliberately not incremented for them — which is why the count check below compares
    // `planned` against `executed + not_attempted` rather than against `executed` alone.
    if let Some(halted) = halted_by.clone() {
        let reached: HashSet<String> = terminal_rows
            .iter()
            .map(|row| row.qualified.clone())
            .collect();
        for claim in claims.iter() {
            if reached.contains(&claim.qualified) {
                continue;
            }
            terminal_rows.push(ClaimTerminalRow {
                qualified: claim.qualified.clone(),
                expected_red: expected_red_roster.contains(claim.qualified.as_str()),
                outcome: ClaimOutcome::NotAttempted {
                    halted_by: halted.clone(),
                },
            });
            outcome.not_attempted_after_abort += 1;
        }
        eprintln!(
            "[floor-abort] {} halted the fold; {} planned identity(ies) were never attempted \
             and are published as not-attempted rows naming it",
            halted, outcome.not_attempted_after_abort
        );
    }
    // THE FOLD IS OVER, so nothing after this point is any witness's cost. Cleared before the
    // report is rendered rather than after, so a gate that fills a cache on its way out cannot
    // be charged to the last row that happened to run.
    shared_fill::set_current_claim(None);
    eprintln!(
        "floor: evaluating {claims_planned} / {claims_planned} ({}ms)",
        eval_started.elapsed().as_millis()
    );
    if let Some(stats) = final_symbol_retention {
        eprintln!(
            "[floor-symbol-retention] canonical_entries={} retained_spelling_bytes={} spelling_cap_bytes={}",
            stats.canonical_entries,
            stats.canonical_retained_spelling_bytes,
            stats.canonical_spelling_cap_bytes,
        );
    }
    // WHAT THE PER-ROW NUMBERS ABOVE DO NOT SAY. Each line names one shared computation, what
    // its fill cost, which claim paid it, and how many claims and modules read that same fill
    // for free. A `shared` fill does not go away when its payer is removed from the floor — the
    // next claim to touch it pays the same seconds — so a paring decision that reads only the
    // per-row wall time is deciding on an attribution artifact.
    eprint!("{}", shared_fill::report());
    // AND THE LEDGER IS ADJUDICATED, NOT ONLY RENDERED. The lines above are what a paring
    // decision reads; this call is what refuses one. It runs after the render so an operator has
    // the whole ledger in the log above the refusal that cites two of its rows.
    refuse_pure_producer_share_refused_carrier_overlap()?;
    // CONSTRUCTIONS AGAINST DISTINCT SCOPES. Equal means the manifest's claim order was grouped
    // by module and each scope was built exactly once; higher means it was not, and the excess
    // is rebuilding this reports rather than absorbs. `modules_per_scope` is carried here now
    // because the sizes are observed as scopes are built, not read off a retained table.
    eprintln!(
        "floor: {} scope construction(s) for {} distinct scope(s) \
         modules_per_scope mean={:.1} max={} corpus={}",
        scope_constructions,
        distinct_scopes.len(),
        if scope_constructions == 0 {
            0.0
        } else {
            scope_module_total as f64 / scope_constructions as f64
        },
        scope_module_max,
        prepared.graph.modules.len()
    );
    // THE SILENT PICK, COUNTED. Each of these is a bare name two transitively-reached modules
    // both spell, resolved by scope precedence because a registry keyed on bare names cannot
    // hold both — a resolution nothing the author wrote authorizes. Reported, not refused: the
    // honest arm is to refuse the ambiguous lookup, and whether that is affordable is a question
    // about this population, which until now nobody had measured. Zero here would mean the
    // reference closure never donates a colliding name and the flat registry is adequate in
    // practice; anything else sizes the terminal per-module-environment correction.
    eprintln!(
        "[floor-bare-name-ambiguity] scopes_affected={} of {} names_total={} worst_scope={}",
        scopes_with_ambiguity, scope_constructions, ambiguous_total, ambiguous_max
    );
    // WHAT ONE SCOPE COSTS. `mean` divides only by constructions that measured a rise, so it is
    // the mean cost of a scope that cost anything; a scope whose modules were all resident from
    // the previous one reads as free and would otherwise drag the mean toward zero. This is the
    // quantity the terminal one-corpus-index correction removes entirely — a scope that is a
    // view rather than a rebuild costs nothing to enter.
    // THE FOLD'S TIME, PARTITIONED AT THE TWO CONSTRUCTIONS THE OTHER RECEIPTS CANNOT SEE.
    // The shared-fill ledger above accounts for the fills; the per-claim cost receipt accounts
    // for the claims. Neither can see scope or frame construction, so this line is what makes
    // the fold's wall add up. It reports both totals AND their per-construction means, because
    // the remedies differ: a large scope total over 656 constructions is a per-MODULE cost that
    // tracks distinct scopes, while a large frame total over 3146 is a per-CLAIM cost that
    // tracks the manifest's length -- and only the second would grow by adding witnesses to a
    // module that already has some.
    eprintln!(
        "[floor-fold-time] scope_build={:.1}s over {} construction(s) (mean={:.0}ms) \
         frame_build={:.1}s over {} construction(s) (mean={:.1}ms)",
        scope_build_nanos as f64 / 1_000_000_000.0,
        scope_constructions,
        if scope_constructions == 0 {
            0.0
        } else {
            scope_build_nanos as f64 / scope_constructions as f64 / 1_000_000.0
        },
        frame_build_nanos as f64 / 1_000_000_000.0,
        frame_constructions,
        if frame_constructions == 0 {
            0.0
        } else {
            frame_build_nanos as f64 / frame_constructions as f64 / 1_000_000.0
        }
    );
    // AND WHICH OF THE THREE CONSTRUCTIONS INSIDE A SCOPE BUILD THAT TIME IS. This line is the
    // DECOMPOSITION of `[floor-fold-time]`'s `scope_build` term, not a second measurement of
    // it: `[floor-scope-cost]` prices a scope in resident bytes and cannot distinguish an
    // order walk from a registry union from an index rebuild, and those are three different
    // constructions whose replacements are three different changes. It deliberately carries NO
    // total and NO overall mean — the total is `scope_build` one line up, and printing a second
    // nearly-equal figure beside it would be two spellings of one number, differing only by the
    // few instructions between the timer around the call and the timers inside it.
    {
        let n = scope_constructions.max(1) as f64;
        eprintln!(
            "[floor-scope-split] order_ms={} registry_ms={} indexes_ms={} \
             per_scope_ms(order={:.1} registry={:.1} indexes={:.1})",
            scope_build_split.order_nanos / 1_000_000,
            scope_build_split.registry_nanos / 1_000_000,
            scope_build_split.indexes_nanos / 1_000_000,
            scope_build_split.order_nanos as f64 / 1_000_000.0 / n,
            scope_build_split.registry_nanos as f64 / 1_000_000.0 / n,
            scope_build_split.indexes_nanos as f64 / 1_000_000.0 / n,
        );
    }
    eprintln!(
        "[floor-scope-cost] max={:.2}GB at={} ({} modules) total_built={:.2}GB over {} construction(s)",
        scope_kb_max as f64 / 1048576.0,
        if scope_kb_max_module.is_empty() {
            "-"
        } else {
            scope_kb_max_module.as_str()
        },
        scope_kb_max_modules,
        scope_kb_total as f64 / 1048576.0,
        scope_constructions
    );
    // THE DEBT, REPORTED EVERY RUN. `held` is the enrolled population that behaved as enrolled
    // — the number that must fall. `now_passing` is enrolled rows that PASSED, which is a build
    // failure by design: a fix has landed and the roster is stale, and the run says so rather
    // than quietly absorbing it.
    eprintln!(
        "[floor-known-red] {} enrolled identity(ies) held as expected-red; {} enrolled \
         identity(ies) now PASS and must be removed from the roster; {} enrolled \
         identity(ies) were BUDGET-REFUSED and so went undecided; {} PASSED but exceeded \
         budget (stale roster row AND a real cost debt); {} HOST-TOOL-UNRESOLVED (infra, \
         not budget); {} RUNTIME-ERRORED (threw, so never decided); {} returned a \
         NON-VERDICT (unreadable, so never decided)",
        known_red_held,
        known_red_now_passing,
        known_red_budget_refused,
        known_red_passed_over_budget,
        known_red_host_tool_unresolved,
        known_red_runtime_errored_count,
        known_red_observation_unreadable_count
    );
    // THE CAUSE CENSUS, PRINTED WHETHER OR NOT ANYTHING REFUSES. Largest class first, so the
    // first line answers the only question the raw count raises: is this one root or many?
    // Printed here rather than in the partition refusal because that refusal returns before the
    // per-identity report runs — on precisely the run where the evidence matters, it would be
    // computed and dropped.
    if !known_red_runtime_error_causes.is_empty() {
        let mut causes: Vec<(&&'static str, &usize)> =
            known_red_runtime_error_causes.iter().collect();
        causes.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        eprintln!(
            "[floor-known-red-causes] {} distinct cause(s) across {} non-verdict enrolled \
             identity(ies)",
            causes.len(),
            known_red_runtime_errored_count + known_red_observation_unreadable_count
        );
        // THE CAP IS PRINTED, NOT RAISED, AND IT IS PRINTED WHETHER OR NOT IT BIT. Truncating at
        // 20 was not wrong because 20 is small; it was wrong because nothing said anything had
        // been dropped, so a reader tallying the printed rows got a short total with no way to
        // know it — 93 of 142 on main `f9963a762`, and an experienced reader looked straight at
        // that line without noticing the rows did not sum. Raising the cap would fix one run and
        // leave the same silence for the next.
        //
        // `not_listed=0` IS THE LOAD-BEARING CASE. A drop notice that appears only when
        // something drops requires the reader to know the field exists in order to miss it,
        // which is the same silence one step quieter. Printing it always makes a complete
        // listing say so, in the same place and the same shape as a truncated one.
        const CAUSE_ROWS: usize = 20;
        let dropped_rows = causes.len().saturating_sub(CAUSE_ROWS);
        let dropped_identities: usize = causes.iter().skip(CAUSE_ROWS).map(|(_, c)| **c).sum();
        let listed_identities: usize = causes.iter().take(CAUSE_ROWS).map(|(_, c)| **c).sum();
        for (cause, count) in causes.iter().take(CAUSE_ROWS) {
            eprintln!("[floor-known-red-causes] {count} × {cause}");
        }
        eprintln!(
            "[floor-known-red-causes] listed={} listing_cap={CAUSE_ROWS} \
             not_listed={dropped_rows} not_listed_identities={dropped_identities} \
             listed_identities={listed_identities}",
            causes.len().min(CAUSE_ROWS)
        );
    }
    eprintln!(
        "[floor-claim-memory] worst single claim grew rss by {:.2}GB at={}",
        claim_rss_kb_max as f64 / 1048576.0,
        if claim_rss_kb_max_row.is_empty() {
            "-"
        } else {
            claim_rss_kb_max_row.as_str()
        }
    );
    // WHAT THE ALLOCATOR WAS HOLDING. Read this beside the claim-memory line above: together
    // they split the fold's resident growth into the part one expensive row caused and the part
    // that was merely never returned. A large total here means the growth was glibc keeping
    // arenas and the fold's own retention is small; a total near zero means the memory is LIVE
    // and the next question is what holds it — the trim cannot release live memory, so it
    // cannot flatter that answer.
    if trims_performed > 0 {
        eprintln!(
            "[floor-heap-trim] {} trim(s) returned {:.2}GB total, {:.2}GB worst single trim",
            trims_performed,
            trim_reclaimed_kb_total as f64 / 1048576.0,
            trim_reclaimed_kb_max as f64 / 1048576.0
        );
    }
    outcome.known_red_held = known_red_held;
    outcome.route_gap_held = route_gap_held;
    outcome.known_red_now_passing = known_red_now_passing;
    outcome.known_red_budget_refused = known_red_budget_refused;
    outcome.known_red_passed_over_budget = known_red_passed_over_budget;
    outcome.known_red_host_tool_unresolved_held = known_red_host_tool_unresolved;
    outcome.known_red_host_effect_refused = known_red_host_effect_refused;
    eprintln!(
        "[floor-route-gap] {} enrolled identity(ies) held as route-gapped; {} unenrolled route \
         gap(s) reported",
        route_gap_held,
        outcome.route_gap.len()
    );
    // THE ROUTE-GAP ROSTER IS A TWO-WAY JOIN, exactly as the expected-red roster is, and for
    // exactly the same reason. Enrollment above only ever asks "is this gap enrolled". The
    // reverse question — is every enrolled identity STILL gapping — has no consumer unless it
    // is asked here, and without it a row survives its own repair: a route lands, the identity
    // starts passing, and the roster keeps counting a debt that was paid.
    //
    // Both directions of staleness are one refusal because both have one remedy — delete the
    // row — and separating them would ask the reader to learn two names for it. An identity
    // that executed and did not gap, and an identity that did not execute at all (renamed,
    // deleted, or declined), are distinguished in the message rather than in the mechanism.
    // EVERY REVERSE JOIN BELOW IS SUSPENDED WHEN THE FOLD HALTED, AND THIS IS THE ONE PREDICATE
    // THAT DECIDES IT. A reverse join asks "is every enrolled identity still executing", and it
    // answers by subtracting what was OBSERVED from the roster. That subtraction is sound only
    // when the observation ran over the whole routed roster. After a halting panic it ran over a
    // PREFIX, so an enrolled identity in the unrun suffix is not stale — it is NOT-ATTEMPTED, and
    // this run has exactly one row for it saying so. Answering "stale, delete the row" over a
    // truncated denominator is the empty-observation narrow DESIGN names: ⊥-as-ignorance
    // ("nothing observed it") rendered as ⊥-as-answer ("nothing exercises it"), and its remedy —
    // delete the roster row — is the opposite of the correct one.
    //
    // THIS DOES NOT FAIL OPEN, and the ordering is the reason it does not. The panic itself is
    // already in `outcome.failures`, and `required_floor_outcome_is_clean` is false whenever that
    // is non-empty, so the run refuses either way; what changes is WHICH refusal it carries and
    // whether the ledger is reached. Before this predicate the truncated join returned `Err`
    // ahead of publication, so a halted run refused with `ExpectedRedIdentityDidNotExecute` —
    // naming rows that are fine — and published NO ledger, which is precisely the artifact the
    // halt exists to produce. The line still stops; the stopped-line audit now survives it.
    let reverse_joins_answerable = halted_by.is_none();
    if !reverse_joins_answerable {
        eprintln!(
            "[floor-halt] reverse roster joins SUSPENDED: the fold halted at {}, so the observed \
             population is a prefix of the routed roster and staleness is not decidable over it. \
             expected_red_roster={} route_gap_roster={} non_verdict_roster={} not_attempted={}. \
             The run refuses on the panic; every unrun enrolled identity carries a not-attempted \
             terminal row rather than a staleness verdict.",
            halted_by.as_deref().unwrap_or(""),
            expected_red_roster.len(),
            route_gap_roster.len(),
            non_verdict_roster.len(),
            outcome.not_attempted_after_abort
        );
    }
    // ONE DECISION POINT, TAKEN AFTER THE FOLD, over the two identity sets. The arms above only
    // RECORD what they observed; nothing there decides admission, so there is no second place
    // where the rule could drift from the one written in
    // `v2.workflow.floor_non_verdict_admission`.
    {
        let admission = non_verdict_admission(&non_verdict_seen, &non_verdict_roster);
        // `added` STAYS ARMED ON A HALT, and the asymmetry with `repaid` below is the whole
        // content of the predicate. `added` is a FORWARD observation — this identity RAN and
        // produced no verdict — so it is decidable over the prefix and suppressing it would hide
        // an observed failure. `repaid` is the reverse question, and it is not.
        for identity in &admission.added {
            let detail = non_verdict_detail
                .get(identity)
                .map(|d| d.as_str())
                .unwrap_or("no verdict");
            outcome.non_verdict_unenrolled.push(format!(
                "{identity} is enrolled as expected-red and produced NO VERDICT ({detail}), and \
                 it is NOT enrolled in v2.workflow.floor_non_verdict. Enrollment as expected-red \
                 admits a known SEMANTIC VERDICT -- this witness reaches its subject and answers \
                 false -- and is not permission for the subject to stop evaluating. Repair the \
                 witness or its subject. Enrolling the identity records the debt; it does not \
                 make the missing verdict acceptable, and the roster is frozen against growth."
            ));
        }
        // REFUSED, NOT MERELY REPORTED. A repaid row left standing is a live exemption: the
        // identity is fixed today, and if it regresses tomorrow it is already rostered and the
        // wall admits it. So repayment and roster deletion are one act.
        for identity in admission.repaid.iter().filter(|_| reverse_joins_answerable) {
            let ran = receipted.contains(identity.as_str());
            outcome.stale_non_verdict.push(if ran {
                format!(
                    "{identity} is enrolled in v2.workflow.floor_non_verdict but PRODUCED A \
                     VERDICT this run -- it reaches its subject again. Delete the row; the debt \
                     is repaid."
                )
            } else {
                format!(
                    "{identity} is enrolled in v2.workflow.floor_non_verdict but did not \
                     execute at all, so no non-verdict could be observed. It was renamed, \
                     deleted, or declined. Delete the row or restore the identity to the routed \
                     roster."
                )
            });
        }
    }

    {
        let mut stale: Vec<&String> = route_gap_roster
            .iter()
            .filter(|q| reverse_joins_answerable && !route_gap_seen.contains(*q))
            .collect();
        stale.sort();
        for identity in stale {
            let ran = receipted.contains(identity.as_str());
            outcome.stale_route_gap.push(if ran {
                format!(
                    "{identity} is enrolled in v2.workflow.floor_route_gap but its route did NOT \
                     gap — the route was supplied or the witness stopped reaching for the \
                     effect. Delete the row; the debt is repaid."
                )
            } else {
                format!(
                    "{identity} is enrolled in v2.workflow.floor_route_gap but did not execute \
                     at all, so no gap could be observed. It was renamed, deleted, or declined. \
                     Delete the row or restore the identity to the routed roster — an \
                     enrollment nothing observes is a row that can never ask to be removed."
                )
            });
        }
    }

    // THE COST-DEBT ROSTER IS A TWO-WAY JOIN FOR THE SAME REASON, and the reverse direction is
    // the whole thing that stops this contract becoming a permission slip. Forward, the roster
    // only ever answers "is this planned claim withheld". Reverse, it must answer "does every
    // withheld identity still exist as a planned claim" — because a rostered row whose witness
    // was renamed, deleted, or declined by home policy withholds NOTHING while still counting
    // toward the debt, which overstates what is frozen in the direction that flatters and
    // survives every repair of the rows around it.
    //
    // BLOCKING, unlike the withhold itself. A withheld row is declared debt; a stale row is a
    // roster that has stopped describing the tree, and the contract's monotone claim is only
    // worth anything while its universe is the discovered one.
    //
    // OUTSIDE THIS RUN'S UNIVERSE IS NOT STALE, BUT UNDECLARED IS. A cost-debt row whose module
    // preparation never offered was never planned, for the same reason the expected-red and
    // route-gap rows above were withheld: this run's universe is the gate closure, not the tree.
    // Those rows are kept as record and reported at identity grain above, never counted as debt.
    //
    // A row the tree DOES NOT DECLARE is a different fact and refuses. The two were conflated
    // while this arm asked a name test; the partition above separates them on preparation's own
    // account of what it loaded. See `partition_cost_debt_roster`.
    //
    // GUARDED BY `reverse_joins_answerable` EXACTLY AS BEFORE. Both arms below REFUSE, and a
    // halted run's population is truncated, so answering "delete the row" over it would be the
    // empty-observation narrow this file already refuses to commit for the sibling rosters. The
    // partition's inputs are planning-time facts, so they survive an execution halt — but a halt
    // during preparation truncates them too, and this arm must not be the one place that assumes
    // otherwise. The report line above is diagnostic and prints regardless.
    {
        for identity in cost_debt_declared_not_withheld
            .iter()
            .filter(|_| reverse_joins_answerable)
        {
            outcome.stale_cost_debt.push(format!(
                "{identity} is enrolled in v2.workflow.floor_cost_debt but was not planned, so \
                 nothing was withheld for it. It was renamed, deleted, or declined by home \
                 policy. Delete the row — a withhold over an identity the tree does not carry \
                 counts as debt while costing nothing and can never ask to be removed."
            ));
        }
        for identity in cost_debt_undeclared
            .iter()
            .filter(|_| reverse_joins_answerable)
        {
            outcome.stale_cost_debt.push(format!(
                "{identity} is enrolled in v2.workflow.floor_cost_debt but the tree DECLARES NO \
                 SUCH IDENTITY — it is not merely outside this run's gate closure, it does not \
                 exist. Delete the row. Until gunbc#9684 gave the floor a declared-identity \
                 universe this row was indistinguishable from a legitimate outside-the-gate \
                 enrollment and refused nothing, which is the cheapest way to fake a green run."
            ));
        }
    }
    // THE ROSTER IS A TWO-WAY JOIN, NOT A ONE-WAY LOOKUP. Enrollment as written above only ever
    // asks "is this executing claim enrolled". The reverse question — is every enrolled identity
    // still executing — has no consumer unless it is asked here, and without it the roster rots
    // in exactly the way that makes a skip list a skip list: an identity that is renamed,
    // deleted, moved under a declined path, or dropped from discovery stays on the roster
    // forever, is never observed, never passes, and therefore never asks to be removed. The
    // debt count would keep counting rows that no longer exist.
    //
    // So a roster entry that did not execute REFUSES, and it refuses by name. That also closes
    // the cheapest way to fake a green run: enrolling an identity that does not exist would
    // otherwise cost nothing.
    let expected_red_missing: Vec<&String> = {
        let mut missing: Vec<&String> = expected_red_roster
            .iter()
            .filter(|q| reverse_joins_answerable && !expected_red_seen.contains(*q))
            .collect();
        missing.sort();
        missing
    };
    if !expected_red_missing.is_empty() && !roster_join_only {
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=ExpectedRedIdentityDidNotExecute count={} — every \
             identity enrolled in v2.workflow.floor_expected_red must be observed among the \
             executed claims; these were not, so they are stale and must be removed from the \
             roster or restored to discovery: {}",
            expected_red_missing.len(),
            expected_red_missing
                .iter()
                .map(|q| q.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    // AND THE PARTITION MUST BE EXACT. With the reverse join above, every enrolled identity was
    // observed exactly once, so it landed in precisely one of the two arms. Checking the sum is
    // therefore checking that the two arms are the whole roster and do not overlap — cheap, and
    // it fails loudly if a later edit adds a third arm that quietly swallows rows — which is
    // exactly what it did to the edit that added `runtime_errored` and `observation_unreadable`,
    // catching an incomplete change one site short. The sum is the reason those two arms could
    // not be added quietly, and it is why this invariant is worth more than the six names in it.
    // The three-outcome roster join relaxes this to still_red | now_passes | not_evaluated and
    // is the authority for pruning — not the failure-log subset.
    // THE PARTITION SUM IS SUSPENDED ON A HALT FOR THE SAME REASON, and it is a stronger case
    // than the joins above rather than a weaker one: the sum's premise is stated in its own
    // comment — "with the reverse join above, every enrolled identity was observed exactly once".
    // A halted fold observes a prefix, so the premise is false by construction and the arms
    // cannot sum to the roster. Left armed it would refuse with `ExpectedRedPartitionInexact`,
    // ahead of publication, over an inexactness the halt guarantees.
    if !roster_join_only
        && reverse_joins_answerable
        && known_red_held
            + known_red_now_passing
            + known_red_budget_refused
            + known_red_passed_over_budget
            + known_red_host_tool_unresolved
            + known_red_host_effect_refused
            + known_red_runtime_errored_count
            + known_red_observation_unreadable_count
            != expected_red_roster.len()
    {
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=ExpectedRedPartitionInexact held={} now_passing={} \
             budget_refused={} passed_over_budget={} host_tool_unresolved={} \
             host_effect_refused={} runtime_errored={} observation_unreadable={} roster={} — \
             every enrolled identity must be exactly one of held, now-passing, budget-refused, \
             passed-over-budget, host-tool-unresolved, host-effect-refused, runtime-errored, or \
             observation-unreadable. First non-verdict rows: {}",
            known_red_held,
            known_red_now_passing,
            known_red_budget_refused,
            known_red_passed_over_budget,
            known_red_host_tool_unresolved,
            known_red_host_effect_refused,
            known_red_runtime_errored_count,
            known_red_observation_unreadable_count,
            expected_red_roster.len(),
            // A BOUNDED SAMPLE, BECAUSE THIS REFUSAL RETURNS BEFORE THE PER-IDENTITY REPORT
            // RUNS. The rows naming WHICH identities live in `outcome.known_red_runtime_errored`
            // and are printed by `report_required_floor_outcome` — which the caller reaches only
            // on the Ok path. So on precisely the run where the partition is wrong, the evidence
            // that says why was computed and then dropped, and the reader is left with counts
            // and no identities. Ten rows, not all of them: enough to see whether the population
            // shares one cause, without turning a refusal into a log dump.
            outcome
                .known_red_runtime_errored
                .iter()
                .chain(outcome.known_red_observation_unreadable.iter())
                .take(10)
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    // THE THREE IDENTITY COUNTS MUST AGREE, and they are compared here rather than reported
    // for a reader to compare. A run that planned more claims than it executed has silently
    // narrowed, which is the failure this whole path exists to make unwritable; reporting the
    // pair and letting a human notice is exactly how the deferred bucket survived.
    // WITHHELD ROWS DO NOT APPEAR IN THIS SUM, and that is a consequence of where the withhold
    // happens rather than an exemption carved into it. A cost-debt row is declined at claim
    // BUILD time, alongside the long-home and live-tree declines, so it never becomes a planned
    // claim and this partition never sees it. The site-level population join upstream
    // (`FloorDispositionJoinInexact`) is what accounts for it.
    //
    // A FIRST CUT OF THIS CHANGE WITHHELD INSIDE THE EXECUTION LOOP INSTEAD, and it would have
    // red the floor on this very check -- 320 planned claims that never executed and never
    // published as not-attempted. Recorded because the lesson generalises past this diff: a
    // `continue` at the top of a fold is exactly the shape that leaves a partition behind, and
    // the three partition checks in this file are the only thing that says so. A code review
    // approved the loop-skip form; the partition would have refused it on the first run.
    if outcome.claims_planned != outcome.claims_executed + outcome.not_attempted_after_abort
        || outcome.claims_executed != outcome.receipt_identities
    {
        return Err(format!(
            "REQUIRED-FLOOR REFUSAL cause=ClaimIdentityCountsDisagree planned={} executed={} \
             not_attempted={} receipted={} — every planned claim must either execute or be \
             published as not-attempted behind a halting panic, and every execution must land \
             one receipt identity; a gap here is a narrowed roster reported as a roster",
            outcome.claims_planned,
            outcome.claims_executed,
            outcome.not_attempted_after_abort,
            outcome.receipt_identities
        ));
    }
    // THE CONSUMER. Reconciliation is an IDENTITY JOIN, and the counts above cannot replace it.
    //
    // The check immediately preceding this one compares three COUNTS, and it is the reason this
    // one is not redundant: a ledger that omits one identity and duplicates another satisfies
    // every count in the run — planned == executed == receipted, and `passed` unchanged — while
    // the identity population is short by one. `ClaimIdentityCountsDisagree` is green over that.
    // Completeness is an identity join, not a count equality.
    //
    // Three sets, computed independently and reported together rather than as a coproduct: the
    // omission-plus-duplicate case fails in TWO of them at once, and an arm that reported only
    // the first would hide the second — which is precisely the pair that cancels in the totals.
    {
        let (planned_without_terminal, terminal_without_planned, terminal_duplicated) =
            reconcile_identity_population(
                &planned_identities,
                terminal_rows.iter().map(|r| r.qualified.as_str()),
            );
        if !planned_without_terminal.is_empty()
            || !terminal_without_planned.is_empty()
            || !terminal_duplicated.is_empty()
        {
            return Err(format!(
                "REQUIRED-FLOOR REFUSAL cause=TerminalLedgerIncomplete \
                 planned_without_terminal={:?} terminal_without_planned={:?} \
                 terminal_duplicated={:?} — every planned identity must appear exactly once in \
                 the terminal population; counts agreeing does not establish this, because an \
                 omission and a duplicate cancel in every total",
                planned_without_terminal, terminal_without_planned, terminal_duplicated
            ));
        }
    }
    // `passed` IS DERIVED FROM THE ROWS, and the branch that used to increment it is deleted
    // rather than kept as a cross-check. A cross-check would preserve the disagreement this
    // derivation makes unrepresentable.
    outcome.passed = terminal_rows
        .iter()
        .filter(|row| claim_disposition(row) == ClaimDisposition::Passed)
        .count();
    // PUBLISH THE EVIDENCE, OR REFUSE. Unconditional: there is no env var gating it and no arm
    // that returns having written nothing. Evidence that is written only when convenient is the
    // instrumentation-optional shape — a green run whose ledger is silently absent is the exact
    // state this artifact exists to prevent — so a publication failure is a floor refusal, and a
    // refusal by the grammar still leaves every row under the diagnosis name.
    //
    // THE BINDING CARRIES WHAT THE FLOOR ACTUALLY HOLDS AND NOTHING INVENTED: the prepared
    // subject digest, and the commit — which is `GITHUB_SHA` on CI and the literal "local"
    // otherwise. "local" is not a commit id and is not rendered as one; it is mapped to the
    // module's declared unpublished token, so a local run's ledger is shaped as NOT
    // candidate-bound rather than carrying a commit-shaped lie. The roster identity is derived
    // inside the module from the identities being published, so no value here can disagree with
    // the population it names.
    {
        let snapshot_wire = if commit == "local" || commit.is_empty() {
            "unpublished"
        } else {
            commit
        };
        let seed_rows: Vec<terminal_ledger_publish::SeedLedgerRow> =
            terminal_rows.iter().map(seed_ledger_row).collect();
        match terminal_ledger_publish::publish_terminal_ledger(
            source_roots,
            snapshot_wire,
            &prepared.subject_digest,
            terminal_ledger_publish::TERMINAL_LEDGER_PATH,
            terminal_ledger_publish::TERMINAL_LEDGER_DIAGNOSIS_PATH,
            &seed_rows,
        )? {
            terminal_ledger_publish::LedgerPublication::Published { path, bytes } => {
                eprintln!("required-floor: terminal ledger published path={path} bytes={bytes}");
            }
            terminal_ledger_publish::LedgerPublication::RefusedWithDiagnosis {
                reason,
                offending,
                path,
            } => {
                return Err(format!(
                    "REQUIRED-FLOOR REFUSAL cause=TerminalLedgerUnrenderable \
                     reason={reason} offending={offending} diagnosis={path} — the ledger's \
                     grammar refused to render this run's evidence. Every row the fold produced \
                     is preserved at the diagnosis path, in a format the ledger reader refuses, \
                     so it cannot be cited as a ledger."
                ));
            }
        }
    }
    if let Some(join) = roster_join_report {
        let join = crate::v1_compiler_expected_red_roster_join::finalize_not_observed(join);
        emit_expected_red_roster_join_summary(&join);
        if let Some(path) = roster_join_path {
            write_expected_red_roster_join_tsv(&path, &join)?;
        }
    }
    // THE DIAGNOSTIC IS A FOLD OVER THE RETAINED ROWS, not a tally kept beside them. The
    // millisecond projection here is exactly `claim_terminality`'s (`nanos / 1_000_000`), so
    // this reproduces `exceeds_completed_cost_line` on the same quantities rather than
    // approximating it.
    let cost_basis = required_floor_cost_basis();
    let over_cost_members: Vec<&WitnessExecutionOccurrence> = outcome
        .claim_cost
        .iter()
        // `exact_cost_ms` IS THE FILTER'S OWN REFUSAL. It answers `None` for a right-censored
        // row, so a bound cannot reach the comparison at all — where the predicate previously
        // relied on the `verdict_reached` conjunct being remembered, the type now supplies it.
        // The conjunct stays because it is a DIFFERENT fact (an unwound claim has an exact cost
        // and no completion to judge), and dropping it would judge a panicked row against a
        // completion line.
        .filter(|row| {
            row.verdict_reached
                && exact_cost_ms(row, cost_basis).is_some_and(|cost| cost > row.cost_line_ms)
        })
        .collect();
    outcome.over_cost_line_diagnostic = over_cost_members.len();
    // THE WARNING TIER, RESTORED AT 100ms BY OPERATOR RULING (2026-08-27) AND PRINTED RANKED.
    //
    // WHY THIS IS NOT THE WARN TIER DELETED ON 2026-08-20, because it will otherwise be deleted
    // again by a reader who recognises the shape. That tier was a second threshold ON THE SAFETY
    // DEADLINE -- an admission mechanism -- and "over a threshold, reported, allowed to finish"
    // is the widen DESIGN section 5 forbids of a FAILURE ARM. This line is not a failure arm and
    // sits nowhere near admission: `exceeds_completed_cost_line` judges only COMPLETED claims,
    // nothing reads `over_cost_line_diagnostic` to fail a run, and a row named here has already
    // been admitted and has already answered. The pairing the operator asked for is a warning at
    // 100ms and a hard error at 500ms, and those are two different mechanisms rather than two
    // tiers of one: the hard error is `required_floor_claim_cpu_safety_limit_ms`, which refuses.
    //
    // RANKED AND BOUNDED, WITH THE REMAINDER STATED. At a 100ms line the population is ~924 rows
    // on the measured corpus, and this file already carries the receipt for what that does to a
    // log -- the prior 5000ms/10000ms pair "fired on the MEDIAN witness (several hundred
    // `[floor-witness-slow]` lines)", and a signal at that volume is read by nobody. So the print
    // is the 25 most expensive rows, which is the actionable head of a distribution whose median
    // is ~1ms.
    //
    // THE CAP IS NOT SILENT (DESIGN section 5, no silent caps): the dropped count is printed and
    // the FULL population is in the per-claim cost TSV, which is written unconditionally a few
    // lines below and uploaded as a run artifact. A reader who needs row 26 has it; a reader
    // skimming the log gets the head instead of 835 lines that hide every other diagnostic.
    let mut ranked: Vec<&WitnessExecutionOccurrence> = over_cost_members.clone();
    // Every member reached the filter above through `Some`, so this cannot rank a bound; the
    // `unwrap_or(0)` is unreachable rather than a substituted figure, and it sorts such a row to
    // the BOTTOM where it would be visible rather than into the actionable head.
    ranked.sort_by_key(|row| std::cmp::Reverse(exact_cost_ms(row, cost_basis).unwrap_or(0)));
    const OVER_COST_PRINT_LIMIT: usize = 25;
    for row in ranked.iter().take(OVER_COST_PRINT_LIMIT) {
        // NAMED `observed_*`, and destructured rather than accessed, for the same reason the
        // artifact's columns are disjoint: this listing is a COMPLETED-cost ranking, and the
        // former `cpu_ms=` label was a slot a censored figure could be dropped into by a later
        // edit without anything reading differently. The censored arm is unreachable here — every
        // member passed `exact_cost_ms` — and it renders the bound's own field names rather than
        // a cost if that ever stops being true.
        let (observed_wall_ms, observed_cpu_ms) = match &row.reading {
            ClaimCostReading::Observed {
                observed_cpu_ms,
                observed_wall_ms,
            } => (observed_wall_ms.to_string(), observed_cpu_ms.to_string()),
            ClaimCostReading::RightCensored(reading) => (
                format!("at_least_{}", reading.elapsed_wall_at_least_ms),
                format!("at_least_{}", reading.elapsed_cpu_at_least_ms),
            ),
        };
        eprintln!(
            "[over-cost] {} observed_wall_ms={} observed_cpu_ms={} eval_steps={} line_ms={} \
             outcome={}",
            row.identity,
            observed_wall_ms,
            observed_cpu_ms,
            row.eval_steps,
            row.cost_line_ms,
            row.outcome
        );
    }
    if ranked.len() > OVER_COST_PRINT_LIMIT {
        eprintln!(
            "[over-cost] ... and {} further row(s) over the {}ms line, not printed. The complete \
             population is in the per-claim cost TSV uploaded by this run; this list is the {} \
             most expensive, ranked on the declared cost basis.",
            ranked.len() - OVER_COST_PRINT_LIMIT,
            over_cost_members
                .first()
                .map(|row| row.cost_line_ms)
                .unwrap_or(0),
            OVER_COST_PRINT_LIMIT
        );
    }
    if let Some(path) = claim_cost_path {
        write_required_floor_claim_cost_tsv(&path, &outcome.claim_cost, cost_basis)?;
    }
    // THE CROSS-CLAIM DEMAND CENSUS. The per-claim cost receipt above says WHAT each claim was
    // charged; this says WHICH PRODUCER IDENTITY the run re-derived across claims, which is the
    // question the charge cannot answer and the one `v2.workflow.floor_pure_producer_share`
    // enrolls from. It gates nothing and it enrols nothing: a row here is a candidate whose
    // serve cost is still unmeasured, and that roster's own header records the case where the
    // serve lost to the recompute.
    {
        let rows = v1_interpreter::cross_claim_demand_rows();
        let disclosure = v1_interpreter::cross_claim_demand_disclosure();
        // THE CEILING ANY TRUE TOTAL MUST SIT UNDER, carried beside the rows because the cost
        // column is inclusive of callees and therefore NOT additive. Without it the first thing a
        // reader does is sum the column, which on the first artifact gave ~14x the run's entire
        // claim-side CPU.
        // SUMMED OVER OBSERVED ROWS ONLY, AND THE EXCLUSION IS COUNTED RATHER THAN SILENT. A
        // right-censored row has no cost to contribute: adding its lower bound would understate
        // the total by an unbounded amount while the figure kept reading as a ceiling, which is
        // the one direction this value must not fail in — it exists so a reader can tell that the
        // census's inclusive-of-callees column has been over-summed, and a ceiling that is itself
        // too low cannot do that job. Excluding them makes the ceiling a ceiling ON THE MEASURED
        // POPULATION, so the count travels beside it and the label below says which population.
        let claim_cpu_total_ms: u128 = outcome
            .claim_cost
            .iter()
            .filter_map(|row| match &row.reading {
                ClaimCostReading::Observed {
                    observed_cpu_ms, ..
                } => Some(*observed_cpu_ms as u128),
                ClaimCostReading::RightCensored(_) => None,
            })
            .sum();
        let claim_cpu_censored_rows = outcome
            .claim_cost
            .iter()
            .filter(|row| matches!(row.reading, ClaimCostReading::RightCensored(_)))
            .count();
        // A COPY, SORTED FOR A READER. The artifact leaves in identity order; this preview
        // orders by cross-claim recomputation and says so in band, so no consumer can read a
        // log head as the census's own ranking or as a candidate roster.
        let mut shared: Vec<&v1_interpreter::CrossClaimDemandRow> =
            rows.iter().filter(|row| row.claims > 1).collect();
        shared.sort_by_key(|row| std::cmp::Reverse(row.cross_claim_wasted_ns()));
        eprintln!(
            "[cross-claim-demand] claims_absorbed={} retained_keys={} shared_keys={} \
             omitted_under_floor={} omitted_under_floor_ms={} key_cap_overflow={} \
             absorb_ms={} absorb_max_ms={} claim_cpu_observed_total_ms={} \
             claim_cpu_censored_rows_excluded={} (observation only; nothing \
             refuses on these figures. The absorb runs AFTER each claim's measurement returns, so \
             it is outside every claim's charged window and cannot trip the deadline; absorb_ms \
             is what this instrument cost the run in total. DO NOT SUM THE COST COLUMN: durations \
             are inclusive of callees, so a producer and its callees overlap and the sum counts \
             the same nanoseconds once per level of nesting -- claim_cpu_observed_total_ms is \
             the ceiling over the MEASURED rows only, with the censored rows counted beside it \
             because their cost is a lower bound and summing bounds into a ceiling would push \
             the ceiling down; it is the ceiling \
             any true total must sit under.)",
            disclosure.claims_absorbed,
            rows.len(),
            shared.len(),
            disclosure.omitted_keys,
            disclosure.omitted_ns / 1_000_000,
            disclosure.overflow_keys,
            disclosure.absorb_ns_total / 1_000_000,
            disclosure.absorb_ns_max / 1_000_000,
            claim_cpu_total_ms,
            claim_cpu_censored_rows
        );
        const CROSS_CLAIM_DEMAND_PRINT_LIMIT: usize = 25;
        eprintln!(
            "[cross-claim-demand] the {} lines below are a PREVIEW ordered by cross-claim \
             recomputation, not a candidate roster and not the artifact's order: a row is a \
             producer whose SERVE cost is unmeasured, and enrolment is decided only by \
             v2.workflow.floor_pure_producer_share, which has removed a top-ranking pair before \
             on a measured serve-versus-recompute experiment.",
            CROSS_CLAIM_DEMAND_PRINT_LIMIT.min(shared.len())
        );
        for row in shared.iter().take(CROSS_CLAIM_DEMAND_PRINT_LIMIT) {
            eprintln!(
                "[cross-claim-demand] producer={} args={} claims={} evals={} total_ms={} \
                 cross_claim_ms={} modules={} sample={} @{}",
                row.producer,
                row.arg_shape,
                row.claims,
                row.evals,
                row.total_ns / 1_000_000,
                row.cross_claim_wasted_ns() / 1_000_000,
                row.modules,
                row.module_sample.join(","),
                row.decl_site
            );
        }
        if shared.len() > CROSS_CLAIM_DEMAND_PRINT_LIMIT {
            eprintln!(
                "[cross-claim-demand] ... and {} further shared producer identit(ies), not \
                 printed. This list is the {} largest by cross-claim recomputation; the complete \
                 retained population is in the cross-claim demand TSV this run uploads, and the \
                 omitted-under-floor counters above bound what no artifact retains.",
                shared.len() - CROSS_CLAIM_DEMAND_PRINT_LIMIT,
                CROSS_CLAIM_DEMAND_PRINT_LIMIT
            );
        }
        if let Some(path) = cross_claim_demand_path {
            write_required_floor_cross_claim_demand_tsv(
                &path,
                &rows,
                &disclosure,
                claim_cpu_total_ms,
                claim_cpu_censored_rows,
            )?;
        }
    }
    if let Some(path) = required_floor_disposition_path {
        write_required_floor_disposition_tsv(
            &path,
            &outcome.required_floor_disposition,
            &terminal_rows,
        )?;
    }
    if let Some(path) = long_home_storage_agreement_path {
        write_long_home_storage_agreement_tsv(&path, &outcome.long_home_storage_agreement)?;
    }
    // THE CHANGED-WITNESS PROJECTION (authority: `v2.workflow.floor_changed_witness`; operator-
    // relayed ruling 2026-08-30). Two PRs added witness identities and this floor reported green
    // while every one of them was DeclinedOutsideGateClosure — the disposition receipt above had
    // the facts, and nothing projected them for the changed set. So: derive the ADDED/MODIFIED
    // test-declaration identities from the run's own diff observation, join each against the
    // disposition and terminal populations this function just published, print one line per
    // CHANGED identity, and let any standing except passed or enrolled-held red the required
    // context
    // through `changed_witness_blocking` (read by `required_floor_outcome_is_clean`).
    //
    // ON CI THE OBSERVATION IS REQUIRED: a diff that cannot be observed means the floor cannot
    // say whether the change's own witnesses ran, which is the exact silence this projection
    // closes — refusing is the only arm that does not widen. A local run (no GITHUB_SHA, so no
    // CI diff baseline to resolve) reports the projection NOT EVALUATED, loudly, rather than
    // fabricating an empty changed set.
    // THE LOCAL-REPO WET LANE RUNS BEFORE THE CHANGED-SET PROJECTION READS IT, in this same
    // process and over this same prepared subject, which is what binds its terminals to the
    // candidate under evaluation rather than to whatever tree produced them elsewhere. Its
    // refusals red the floor on their own: a lane whose roster and receipts disagree cannot
    // support the route claim `std.witness_admission` makes for its cadence.
    //
    // THE INVOCATION IS ITSELF OBSERVED, by both routes the ruling that ordered this wall names.
    // Finalization is unconditional and takes execution as an ARGUMENT, so it runs whether or not
    // the executor did -- it lives outside the function that would be deleted. DELETING the call
    // below leaves `wet_execution` unbound and the floor does not compile; SUPPRESSING it, by
    // handing the finalizer `NotInvoked` from anywhere, reaches the named
    // `LocalRepoWetExecutorAbsent` refusal. That is the tooth the schedule-to-terminal join cannot
    // have: with nothing invoked there are no terminals, and a join over zero terminals and an
    // unread schedule holds vacuously while `std.witness_admission` goes on claiming the route.
    let wet_execution: LocalRepoWetExecution =
        run_local_repo_wet_lane(&prepared, &local_repo_wet_schedule_rows, published.clone());
    let wet_lane = finalize_local_repo_wet_lane(
        &local_repo_wet_schedule_rows,
        wet_execution,
        &prepared.subject_digest,
    )?;
    if let Some(changed_witnesses) = changed_witnesses {
        let rows = changed_witness_projection_rows(
            &changed_witnesses,
            &outcome.required_floor_disposition,
            &terminal_rows,
            &cost_debt_verdict_only,
            &cost_debt_observations,
            &wet_lane,
            &prepared.subject_digest,
        );
        emit_changed_witness_projection(&rows)?;
        outcome.changed_witness_rows = rows.len();
        outcome.changed_witness_blocking = rows
            .iter()
            .filter(|r| r.blocks)
            .map(|r| r.identity.clone())
            .collect();
    }
    Ok(outcome)
}

/// The wire label of one `CostDebtRosterStanding` arm. Modeled authority:
/// `v2.workflow.required_floor` `CostDebtRosterStanding`; one spelling per arm, so the published
/// receipt line and the roster report cannot name the same standing two ways.
pub(crate) fn cost_debt_roster_standing_label(standing: &CostDebtRosterStanding) -> &'static str {
    match standing {
        CostDebtRosterStanding::Withheld => "withheld",
        CostDebtRosterStanding::WithholdOverriddenForChangedVerdict => {
            "withhold-overridden-for-changed-verdict"
        }
        CostDebtRosterStanding::OutsideThisRunsUniverse => "outside-this-runs-universe",
        CostDebtRosterStanding::Undeclared => "undeclared",
        CostDebtRosterStanding::DeclaredButNotWithheld => "declared-not-withheld",
    }
}

pub(crate) fn required_floor_disposition_label(
    disposition: &RequiredFloorDisposition,
) -> &'static str {
    match disposition {
        RequiredFloorDisposition::Planned => "planned",
        RequiredFloorDisposition::PlannedAsChangedWitness => "planned_as_changed_witness",
        RequiredFloorDisposition::DeclinedLongModule { .. } => "declined_long_module",
        RequiredFloorDisposition::DeclinedFixtureMember { .. } => "declined_fixture_member",
        RequiredFloorDisposition::DeclinedOutsideRequiredGate => "declined_outside_required_gate",
        RequiredFloorDisposition::DeclinedCostDebt => "declined_cost_debt",
        RequiredFloorDisposition::DeclinedOutsideGateClosure => "declined_outside_gate_closure",
        RequiredFloorDisposition::DeclinedDiscoveryExcluded { .. } => "declined_discovery_excluded",
        RequiredFloorDisposition::DeclinedChangedWitnessOutsideDiscovery { .. } => {
            "declined_changed_witness_outside_discovery"
        }
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
        RequiredFloorDisposition::Planned
        | RequiredFloorDisposition::PlannedAsChangedWitness
        | RequiredFloorDisposition::DeclinedOutsideRequiredGate
        | RequiredFloorDisposition::DeclinedOutsideGateClosure
        | RequiredFloorDisposition::DeclinedCostDebt => "",
    }
}

#[cfg(test)]
mod pure_producer_share_tests {
    use super::*;
    use std::rc::Rc;

    /// A minimal PreparedRepository over in-memory sources — the same graph shape the floor
    /// prepares, without the corpus. Only the fields `install_pure_producer_share` reaches
    /// carry content.
    fn prepared_from(sources: &[(&str, &str)]) -> PreparedRepository {
        let files: Vec<Rc<crate::v1_compiler_compile::SourceFile>> = sources
            .iter()
            .map(|(path, content)| {
                Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: path.to_string(),
                    content: content.to_string(),
                })
            })
            .collect();
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(files.into()));
        let graph = result.graph.as_ref().expect("fixture graph").clone();
        PreparedRepository {
            graph,
            source_indices: result.source_indices.clone(),
            subject_digest: "fixture".to_string(),
            modules_resolved: sources.len(),
            modules_excluded: 0,
            full_inventory: Vec::new(),
            discovery_exclusions: HashMap::new(),
        }
    }

    /// THE CLOSURE RED the review asked for: a prepared subject WITHOUT the roster module
    /// must REFUSE, never skip — a skip leaves admission empty while the floor reads green,
    /// memoizing nothing.
    #[test]
    fn a_subject_without_the_roster_module_refuses_never_skips() {
        v1_interpreter::clear_cross_claim_pure_memos();
        let prepared = prepared_from(&[(
            "workspace/src/other.dag",
            "module fixture.other\nfn check() -> Bool { true }\n",
        )]);
        let err = install_pure_producer_share(&prepared)
            .expect_err("a subject without the roster must refuse");
        assert!(
            err.contains("PureProducerShareRosterOutsidePreparedSubject"),
            "refusal must name the cause: {err}"
        );
        v1_interpreter::clear_cross_claim_pure_memos();
    }

    /// Positive control: a subject carrying the roster module warms its nullary rows into
    /// the cross-claim store.
    #[test]
    fn a_carried_roster_warms_and_stores_its_nullary_rows() {
        v1_interpreter::clear_cross_claim_pure_memos();
        let prepared = prepared_from(&[(
            "workspace/src/v2/workflow/floor_pure_producer_share.dag",
            "module v2.workflow.floor_pure_producer_share\n\
             fn tm_local() -> Bool { true }\n\
             data floor_cross_claim_pure_producers_warm: List<String> = [\"v2.workflow.floor_pure_producer_share.tm_local\"]\n\
             data floor_cross_claim_pure_producers_claim_forced: List<String> = [\"v2.workflow.floor_pure_producer_share.tm_local\"]\n\
             type ShareRefusalVerdict =\n\
                 MeasuredServeAboveRecompute\n\
               | NoMeasuredEffectOverItsConsumers\n\
             type RefusedShareCandidate {\n\
               producer: String\n\
               verdict: ShareRefusalVerdict\n\
               carrier_modules: List<String>\n\
               measurement: String\n\
               next_trigger: String\n\
             }\n\
             data floor_cross_claim_refused_candidates: List<RefusedShareCandidate> = [\n\
               RefusedShareCandidate {\n\
                 producer: \"v2.workflow.floor_pure_producer_share.tm_refused\",\n\
                 verdict: MeasuredServeAboveRecompute,\n\
                 carrier_modules: [\"v2.test.fixture.a_consumer\"],\n\
                 measurement: \"fixture\",\n\
                 next_trigger: \"fixture\"\n\
               }\n\
             ]\n",
        )]);
        let observations =
            install_pure_producer_share(&prepared).expect("carried roster installs and warms");
        let (stores, overflow) = v1_interpreter::cross_claim_pure_memo_counts();
        assert_eq!(overflow, 0);
        assert!(stores >= 1, "the warm must land in the store, got {stores}");
        // THE DISCRIMINATING ASSERTION, and it is why this control is no longer only positive:
        // the warm is a shared preparation build, and a shared build that produces no
        // observation is bounded by nothing -- the preparation refusal is denominated over the
        // observations collected here. Before the observation existed this function returned
        // `()`, so this assertion could not be written at all, which is precisely the shape of
        // the gap: the cost was real, printed, and invisible to the only wall that could stop it.
        assert_eq!(
            observations.len(),
            1,
            "one observation per warm row, got {observations:?}"
        );
        let (label, observed) = &observations[0];
        assert_eq!(
            label, "CrossClaimPureProducerWarm/v2.workflow.floor_pure_producer_share.tm_local",
            "the label must name the ROW, since the refusal names one phase and a reader must \
             reach one roster row from it"
        );
        // The three axes the preparation refusal reads. Asserting they are PRESENT rather than
        // asserting a magnitude: a fixture's absolute cost is a property of the fixture and the
        // runner it ran on, and a threshold copied from this tree would be the measurement-as-
        // oracle DESIGN section 5 refuses.
        let _: u64 = observed.cpu_ms;
        let _: u64 = observed.wall_ms;
        let _: u64 = observed.rss_growth_bytes;
        v1_interpreter::clear_cross_claim_pure_memos();
    }

    /// THE `AlreadyPresent` PATH REPORTS THAT IT FOUND THE VALUE, NOT THAT IT BUILT IT.
    /// The discriminating red for review 59035: before the fix this asserted
    /// `BuiltByPreparation` on a warm that built nothing, so the receipt claimed preparation
    /// produced an artifact it merely found. Running the install TWICE without clearing the
    /// memos in between is what puts the second warm on that path, and nothing else in this
    /// module reaches it — which is why the defect survived the first round of tests.
    #[test]
    fn a_second_warm_of_the_same_producer_reports_that_it_was_found_not_built() {
        v1_interpreter::clear_cross_claim_pure_memos();
        let prepared = prepared_from(&[(
            "workspace/src/v2/workflow/floor_pure_producer_share.dag",
            "module v2.workflow.floor_pure_producer_share\n\
             fn tm_local() -> Bool { true }\n\
             data floor_cross_claim_pure_producers_warm: List<String> = [\"v2.workflow.floor_pure_producer_share.tm_local\"]\n\
             data floor_cross_claim_pure_producers_claim_forced: List<String> = []\n\
             type ShareRefusalVerdict =\n\
                 MeasuredServeAboveRecompute\n\
               | NoMeasuredEffectOverItsConsumers\n\
             type RefusedShareCandidate {\n\
               producer: String\n\
               verdict: ShareRefusalVerdict\n\
               carrier_modules: List<String>\n\
               measurement: String\n\
               next_trigger: String\n\
             }\n\
             data floor_cross_claim_refused_candidates: List<RefusedShareCandidate> = []\n",
        )]);

        let first = install_pure_producer_share(&prepared).expect("first install warms");
        assert!(
            matches!(
                first[0].1.provenance,
                SharedBuildProvenance::BuiltByPreparation
            ),
            "the first warm BUILDS it: {:?}",
            first[0].1.provenance
        );

        // No `clear_cross_claim_pure_memos()` here, deliberately: the retained value is the
        // whole subject of this test.
        let second = install_pure_producer_share(&prepared).expect("second install re-warms");
        match &second[0].1.provenance {
            SharedBuildProvenance::AlreadyWarmOnEntry { triggered_by } => {
                // The label names a BOUNDARY and not a call site, because `AlreadyPresent`
                // establishes presence and not cause. Asserting the exact string keeps a
                // future edit from quietly upgrading it into a fabricated attribution.
                assert_eq!(
                    *triggered_by,
                    "an-earlier-rostered-producer-in-this-warm-loop"
                );
            }
            other => panic!("a warm that found the value must not claim it built it: {other:?}"),
        }
        v1_interpreter::clear_cross_claim_pure_memos();
    }

    /// A warm row naming a producer the subject cannot resolve stops the line.
    #[test]
    fn a_stale_warm_row_stops_the_line() {
        v1_interpreter::clear_cross_claim_pure_memos();
        let prepared = prepared_from(&[(
            "workspace/src/v2/workflow/floor_pure_producer_share.dag",
            "module v2.workflow.floor_pure_producer_share\n\
             data floor_cross_claim_pure_producers_warm: List<String> = [\"v2.workflow.floor_pure_producer_share.tm_gone\"]\n\
             data floor_cross_claim_pure_producers_claim_forced: List<String> = []\n",
        )]);
        let err = install_pure_producer_share(&prepared)
            .expect_err("a stale warm row must stop the line");
        // The stop now lands at roster RESOLUTION (admission is by resolved declaration
        // identity), before any warm runs — same line-stop, more precisely located.
        assert!(
            err.contains("PureProducerShareProducerUnresolved"),
            "refusal must name the cause: {err}"
        );
        v1_interpreter::clear_cross_claim_pure_memos();
    }
}

#[cfg(test)]
mod scope_fragment_memo_equivalence {
    use super::*;
    use std::rc::Rc;

    /// A minimal `PreparedRepository` over in-memory sources — the same shape the floor
    /// prepares, without the corpus.
    fn prepared_from(sources: &[(&str, &str)]) -> PreparedRepository {
        let files: Vec<Rc<crate::v1_compiler_compile::SourceFile>> = sources
            .iter()
            .map(|(path, content)| {
                Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: path.to_string(),
                    content: content.to_string(),
                })
            })
            .collect();
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(files.into()));
        let graph = result.graph.as_ref().expect("fixture graph").clone();
        PreparedRepository {
            graph,
            source_indices: result.source_indices.clone(),
            // A DISTINCT DIGEST PER FIXTURE, because the fragment memo is keyed by it: a shared
            // spelling would let one fixture's fragments answer another's scopes, which is the
            // cross-subject leak the key exists to prevent — and a test that shared it would be
            // measuring the leak instead of the memo.
            subject_digest: format!("fixture-{}", sources.len()),
            modules_resolved: sources.len(),
            modules_excluded: 0,
            full_inventory: Vec::new(),
            discovery_exclusions: HashMap::new(),
        }
    }

    /// TWO MODULES CLAIMING ONE BARE NAME, reached from two different entries — so the two
    /// scopes over this corpus have DIFFERENT precedence orders over the SAME modules, and the
    /// colliding name must resolve to a different declaration in each. That is the property a
    /// per-module memo could destroy if it ever cached anything order-dependent: the second
    /// scope would be served the first scope's winner and the fingerprints would diverge.
    fn colliding_corpus() -> PreparedRepository {
        prepared_from(&[
            (
                "workspace/src/alpha.dag",
                "module fixture.alpha\n\
                 import fixture.shared { helper }\n\
                 fn shared_name() -> Bool { true }\n\
                 fn alpha_entry() -> Bool { shared_name() }\n",
            ),
            (
                "workspace/src/beta.dag",
                "module fixture.beta\n\
                 import fixture.shared { helper }\n\
                 fn shared_name() -> Bool { false }\n\
                 fn beta_entry() -> Bool { shared_name() }\n",
            ),
            (
                "workspace/src/shared.dag",
                "module fixture.shared\n\
                 fn helper() -> Bool { true }\n\
                 data shared_datum: Bool = true\n",
            ),
        ])
    }

    /// THE EQUIVALENCE WITNESS. Every scope of the corpus, built with the per-module fragment
    /// memo and built without it, must answer identically at identity grain — same fn_nodes
    /// down to WHICH declaration node each name resolves to, same ambiguity set, same
    /// file/import binding tiers, same service ops, same item registry, same precedence
    /// identity and module count.
    #[test]
    fn a_memoized_scope_answers_exactly_as_an_unmemoized_one() {
        let prepared = colliding_corpus();
        for entry in ["fixture.alpha", "fixture.beta", "fixture.shared"] {
            let memoized = claim_scope_for(&prepared, entry).expect("memoized scope");
            let control = claim_scope_for_without_memos(&prepared, entry).expect("control scope");
            assert_eq!(
                memoized.scope_identity, control.scope_identity,
                "{entry}: precedence identity diverged"
            );
            assert_eq!(
                memoized.module_count, control.module_count,
                "{entry}: module population diverged"
            );
            assert_eq!(
                memoized.ambiguous_bare_names, control.ambiguous_bare_names,
                "{entry}: ambiguity population diverged"
            );
            assert_eq!(
                memoized.resolution_fingerprint(),
                control.resolution_fingerprint(),
                "{entry}: scope answers diverged between the memoized and unmemoized folds"
            );
        }
    }

    /// THE DISCRIMINATING RED'S SUBJECT, asserted rather than assumed: the two entries really
    /// do resolve the colliding bare name to DIFFERENT declarations. Without this the
    /// equivalence assertion above could pass over a corpus where order never decided
    /// anything, which is the absence-evidence trap — a clean comparison on a sample that
    /// lacks the discriminating case.
    #[test]
    fn the_two_scopes_really_do_disagree_about_the_colliding_name() {
        let prepared = colliding_corpus();
        let alpha = claim_scope_for(&prepared, "fixture.alpha").expect("alpha scope");
        let beta = claim_scope_for(&prepared, "fixture.beta").expect("beta scope");
        let bare_line = |scope: &PreparedClaimScope| -> String {
            scope
                .resolution_fingerprint()
                .into_iter()
                .find(|line| line.starts_with("fn\tshared_name\t"))
                .expect("both scopes bind the colliding bare name")
        };
        assert_ne!(
            bare_line(&alpha),
            bare_line(&beta),
            "the fixture must exercise order-dependent resolution, or the equivalence \
             witness proves nothing about precedence"
        );
    }

    /// THE MEMOS ARE ACTUALLY REUSED ACROSS SCOPES, asserted rather than assumed. Without this
    /// the equivalence test above could be green because the "memoized" path memoized nothing:
    /// a per-scope cache would answer identically to the control and prove only that two
    /// unmemoized folds agree.
    #[test]
    fn a_second_scope_of_one_subject_reuses_the_first_scopes_memos() {
        let prepared = colliding_corpus();
        claim_scope_for(&prepared, "fixture.alpha").expect("first scope");
        let filled = crate::cli_run::scope_memo_population_for_test(&prepared);
        assert!(
            filled.0 > 0 && filled.1 > 0,
            "the first scope must leave fragments and reached-lists behind: {filled:?}"
        );
        claim_scope_for(&prepared, "fixture.beta").expect("second scope");
        let after = crate::cli_run::scope_memo_population_for_test(&prepared);
        assert!(
            after.0 >= filled.0 && after.1 >= filled.1,
            "the second scope must add to the SAME memos, not start new ones: \
             {filled:?} then {after:?}"
        );
    }

    /// The memo is keyed inside one prepared subject. Two subjects sharing a module NAME must
    /// not share its fragment: the declarations are different nodes, and serving one for the
    /// other is a silently wrong resolution rather than a slow one.
    #[test]
    fn a_second_subject_does_not_inherit_the_first_subjects_fragments() {
        let first = colliding_corpus();
        let second = prepared_from(&[
            (
                "workspace/src/alpha.dag",
                "module fixture.alpha\nfn shared_name() -> Bool { false }\n",
            ),
            (
                "workspace/src/second_only.dag",
                "module fixture.second_only\nfn only_here() -> Bool { true }\n",
            ),
        ]);
        let from_first = claim_scope_for(&first, "fixture.alpha").expect("first subject scope");
        let from_second = claim_scope_for(&second, "fixture.alpha").expect("second subject scope");
        let control = claim_scope_for_without_memos(&second, "fixture.alpha").expect("control");
        assert_eq!(
            from_second.resolution_fingerprint(),
            control.resolution_fingerprint(),
            "the second subject's scope must answer from its OWN modules"
        );
        assert_ne!(
            from_first.resolution_fingerprint(),
            from_second.resolution_fingerprint(),
            "two subjects declaring the same module name must not produce one answer"
        );
    }
}

#[cfg(test)]
mod changed_witness_projection_tests {
    use super::*;

    /// THE STATE EVERY CHANGED IDENTITY OUTSIDE THE LOCAL-REPO WET LANE IS IN: the lane ran, held,
    /// and admitted nobody. Named once so the wet witnesses below differ from the others by
    /// exactly the fact under test.
    fn no_wet_lane() -> LocalRepoWetLaneOutcome {
        LocalRepoWetLaneOutcome {
            scheduled: 0,
            candidate: TEST_CANDIDATE.to_string(),
            admitted: HashSet::new(),
            refusals: Vec::new(),
        }
    }

    /// One scheduled row, named once so every cell below differs from the positive by exactly the
    /// fact under test.
    fn scheduled_row(identity: &str) -> LocalRepoWetScheduledRow {
        LocalRepoWetScheduledRow {
            identity: identity.to_string(),
            entry: "dag/test/claim/x_test.dag".to_string(),
            entry_module: "test.claim.x".to_string(),
            function: "w_holds".to_string(),
        }
    }

    fn terminal_row(identity: &str, candidate: &str) -> LocalRepoWetTerminalRow {
        LocalRepoWetTerminalRow {
            identity: identity.to_string(),
            entry: "dag/test/claim/x_test.dag".to_string(),
            function: "w_holds".to_string(),
            candidate: candidate.to_string(),
            observed: LocalRepoWetObserved::Passed,
        }
    }

    fn finalize(
        schedule: &[LocalRepoWetScheduledRow],
        terminals: Vec<LocalRepoWetTerminalRow>,
    ) -> Result<LocalRepoWetLaneOutcome, String> {
        finalize_local_repo_wet_lane(
            schedule,
            LocalRepoWetExecution::Ran {
                candidate: TEST_CANDIDATE.to_string(),
                terminals,
            },
            TEST_CANDIDATE,
        )
    }

    /// THE OPPOSITE CONTROL FOR THE INVOCATION WALL, run through the SAME finalizer the required
    /// floor calls. A nonempty schedule with the executor never invoked reds by name, and the
    /// message names the arm and the count so the reader is sent to the call site rather than to
    /// the roster.
    ///
    /// It is deliberately not a grep for `run_local_repo_wet_lane` and not a `.dag`-only
    /// comparison: those establish that a call is spelled somewhere, never that finalization
    /// refuses when it is not.
    #[test]
    fn a_nonempty_schedule_with_no_executor_invocation_refuses_at_finalization() {
        let refused = finalize_local_repo_wet_lane(
            &[scheduled_row("test.claim.x.w_holds")],
            LocalRepoWetExecution::NotInvoked,
            TEST_CANDIDATE,
        )
        .expect_err("a scheduled lane whose executor never ran must refuse");
        assert!(
            refused.contains("LocalRepoWetExecutorAbsent") && refused.contains("1 member(s)"),
            "the refusal must name the arm and how many members went unrun, got: {refused}"
        );
    }

    /// THE PAIRED POSITIVE. The same finalizer, the same schedule, an executor that ran this
    /// candidate and produced the terminal the schedule demands: it holds, and the admitted set is
    /// DERIVED from that join rather than accepted from the executor.
    #[test]
    fn an_executor_that_ran_this_candidate_completely_finalizes() {
        let schedule = [scheduled_row("test.claim.x.w_holds")];
        let outcome = finalize(
            &schedule,
            vec![terminal_row("test.claim.x.w_holds", TEST_CANDIDATE)],
        )
        .expect("a complete run against this candidate must finalize");
        assert_eq!(outcome.scheduled, 1);
        assert!(outcome.admitted.contains("test.claim.x.w_holds"));
    }

    /// A RECEIPT FROM ANOTHER TREE IS REFUSED BEFORE THE ROSTER IS BLAMED. The executor ran, and
    /// the schedule is satisfied on its own terms -- what fails is provenance, which is the
    /// standing this lane's candidate binding exists to make unavailable.
    #[test]
    fn an_executor_that_ran_another_candidate_refuses_at_finalization() {
        let refused = finalize_local_repo_wet_lane(
            &[],
            LocalRepoWetExecution::Ran {
                candidate: "subject-another-tree".to_string(),
                terminals: Vec::new(),
            },
            TEST_CANDIDATE,
        )
        .expect_err("a receipt bound to another candidate must refuse");
        assert!(
            refused.contains("LocalRepoWetExecutionForeignCandidate"),
            "got: {refused}"
        );
    }

    /// THE FORWARD DIRECTION OF THE JOIN, IN THE HOST. An executor that ran and skipped a scheduled
    /// member must refuse HERE, not merely in the `.dag` fold: the carrier this finalizer reads
    /// used to be the executor's own admitted set, so a skipped member finalized green while the
    /// modeled authority refused it.
    #[test]
    fn an_executor_that_ran_but_skipped_a_scheduled_member_refuses_as_missing() {
        let schedule = [
            scheduled_row("test.claim.x.w_holds"),
            scheduled_row("test.claim.x.w_other"),
        ];
        let refused = finalize(
            &schedule,
            vec![terminal_row("test.claim.x.w_holds", TEST_CANDIDATE)],
        )
        .expect_err("a scheduled member with no terminal must refuse");
        assert!(
            refused.contains("1 refusal(s)")
                && refused.contains("WetTerminalMissing")
                && refused.contains("w_other"),
            "exactly the skipped member refuses, by name, got: {refused}"
        );
    }

    /// THE REVERSE DIRECTION. A terminal for an identity nobody scheduled is a refusal, not a
    /// bonus admission -- without this an executor could admit anything at all as long as it also
    /// ran the roster.
    #[test]
    fn a_terminal_for_an_unscheduled_identity_refuses_at_finalization() {
        let schedule = [scheduled_row("test.claim.x.w_holds")];
        let refused = finalize(
            &schedule,
            vec![
                terminal_row("test.claim.x.w_holds", TEST_CANDIDATE),
                terminal_row("test.claim.x.w_nobody_scheduled", TEST_CANDIDATE),
            ],
        )
        .expect_err("an unscheduled terminal must refuse");
        assert!(
            refused.contains("1 refusal(s)")
                && refused.contains("WetTerminalUnscheduled")
                && refused.contains("w_nobody_scheduled"),
            "got: {refused}"
        );
    }

    /// THE VERDICT CELL. The member ran, for this candidate, from the scheduled source -- and did
    /// not pass. The lane's one expectation is not met, so it cannot be admitted.
    #[test]
    fn a_member_that_ran_and_failed_refuses_at_finalization() {
        let schedule = [scheduled_row("test.claim.x.w_holds")];
        let mut terminal = terminal_row("test.claim.x.w_holds", TEST_CANDIDATE);
        terminal.observed = LocalRepoWetObserved::Failed;
        let refused =
            finalize(&schedule, vec![terminal]).expect_err("a failed member must not be admitted");
        assert!(
            refused.contains("WetTerminalVerdictNotExpected")
                && refused.contains("observed failed"),
            "got: {refused}"
        );
    }

    /// THE ENTRY CELL, which is why the schedule keeps a third writable field. The roster's entry
    /// must be the source the prepared subject actually resolved for the module; a stale one is a
    /// refusal rather than an unread string beside a module the executor reached another way.
    #[test]
    fn a_terminal_resolved_from_another_source_refuses_as_foreign_entry() {
        let schedule = [scheduled_row("test.claim.x.w_holds")];
        let mut terminal = terminal_row("test.claim.x.w_holds", TEST_CANDIDATE);
        terminal.entry = "dag/test/claim/somewhere_else_test.dag".to_string();
        let refused = finalize(&schedule, vec![terminal])
            .expect_err("a roster entry that is not the resolved source must refuse");
        assert!(
            refused.contains("WetTerminalForeignEntry")
                && refused.contains("somewhere_else_test.dag"),
            "got: {refused}"
        );
    }

    const TEST_CANDIDATE: &str = "subject-8993beb0d2808db6";

    fn disposition(
        identity: &str,
        disposition: RequiredFloorDisposition,
    ) -> RequiredFloorDispositionRow {
        RequiredFloorDispositionRow {
            identity: identity.to_string(),
            disposition,
        }
    }

    /// Every pre-FLOOR-CHANGED-COST-0 witness projects under the ORDINARY cost policy: no
    /// identity is in the verdict-only population and nothing is published. The override's own
    /// witnesses call `changed_witness_projection_rows` directly with those populations
    /// non-empty, so the two shapes stay distinguishable here rather than sharing a default.
    fn ordinary_projection(
        changed: &[String],
        dispositions: &[RequiredFloorDispositionRow],
        terminal: &[ClaimTerminalRow],
    ) -> Vec<ChangedWitnessProjectionRow> {
        changed_witness_projection_rows(
            changed,
            dispositions,
            terminal,
            &HashSet::new(),
            &HashMap::new(),
            &no_wet_lane(),
            TEST_CANDIDATE,
        )
    }

    fn terminal(identity: &str, outcome: ClaimOutcome) -> ClaimTerminalRow {
        ClaimTerminalRow {
            qualified: identity.to_string(),
            expected_red: false,
            outcome,
        }
    }

    /// Positive control: a planned changed identity with a terminal Pass is the ONE green
    /// standing — this is the seed-prefix arm of the falsifier set (a witness added under a
    /// gate prefix executes and projects green).
    #[test]
    fn planned_and_passed_changed_identity_is_green() {
        let rows = ordinary_projection(
            &["m.a".to_string()],
            &[disposition("m.a", RequiredFloorDisposition::Planned)],
            &[terminal("m.a", ClaimOutcome::Pass)],
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].standing, "planned-and-passed");
        assert!(!rows[0].blocks, "planned-and-passed must not block");
    }

    #[test]
    fn changed_sublane_pass_is_green_and_keeps_its_selector_disposition() {
        let rows = ordinary_projection(
            &["m.a".to_string()],
            &[disposition(
                "m.a",
                RequiredFloorDisposition::PlannedAsChangedWitness,
            )],
            &[terminal("m.a", ClaimOutcome::Pass)],
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].standing, "planned-and-passed");
        assert_eq!(rows[0].disposition, "planned_as_changed_witness");
        assert_eq!(rows[0].outcome, "passed");
        assert!(!rows[0].blocks);
    }

    /// THE INCIDENT ARM: a changed identity whose module name is outside the gate closure is
    /// Declined, blocking, and the row NAMES the identity — the state PRs #9672/#9675 shipped
    /// green.
    #[test]
    fn declined_outside_gate_closure_changed_identity_blocks_and_names_itself() {
        let rows = ordinary_projection(
            &["outside.gate.witness".to_string()],
            &[disposition(
                "outside.gate.witness",
                RequiredFloorDisposition::DeclinedOutsideGateClosure,
            )],
            &[],
        );
        assert_eq!(rows[0].standing, "declined");
        assert_eq!(rows[0].disposition, "declined_outside_gate_closure");
        assert!(rows[0].blocks, "a declined changed identity must block");
        assert_eq!(rows[0].identity, "outside.gate.witness");
    }

    /// A changed identity absent from the disposition receipt is MissingDisposition, blocking.
    #[test]
    fn changed_identity_absent_from_receipt_is_missing_disposition() {
        let rows = ordinary_projection(
            &["m.gone".to_string()],
            &[disposition("m.other", RequiredFloorDisposition::Planned)],
            &[],
        );
        assert_eq!(rows[0].standing, "missing-disposition");
        assert!(rows[0].blocks, "a missing disposition must block");
    }

    /// A planned changed identity with no terminal row (`not_executed` in the receipt) carries
    /// no terminal verdict — blocking.
    #[test]
    fn planned_changed_identity_without_terminal_row_blocks() {
        let rows = ordinary_projection(
            &["m.a".to_string()],
            &[disposition("m.a", RequiredFloorDisposition::Planned)],
            &[],
        );
        assert_eq!(rows[0].standing, "planned-without-terminal-verdict");
        assert_eq!(rows[0].outcome, "not_executed");
        assert!(rows[0].blocks);
    }

    /// A planned changed identity whose terminal verdict is a FAIL is not planned-and-passed:
    /// no terminal Passed verdict stands, and the changed-set grain reds it by name.
    #[test]
    fn planned_changed_identity_with_failed_verdict_blocks() {
        let rows = ordinary_projection(
            &["m.a".to_string()],
            &[disposition("m.a", RequiredFloorDisposition::Planned)],
            &[terminal("m.a", ClaimOutcome::Fail)],
        );
        assert_eq!(rows[0].standing, "planned-without-terminal-verdict");
        assert_eq!(rows[0].outcome, "failed");
        assert!(rows[0].blocks);
    }

    /// An enrolled expected-red failing exactly as enrolled reached its terminal verdict —
    /// green at this grain, so the sanctioned add-a-discriminating-RED move is not vetoed.
    #[test]
    fn known_red_held_changed_identity_is_green() {
        let rows = ordinary_projection(
            &["m.red".to_string()],
            &[disposition("m.red", RequiredFloorDisposition::Planned)],
            &[ClaimTerminalRow {
                qualified: "m.red".to_string(),
                expected_red: true,
                outcome: ClaimOutcome::Fail,
            }],
        );
        assert_eq!(rows[0].standing, "planned-and-known-red-held");
        assert_eq!(rows[0].outcome, "known-red-held");
        assert!(
            !rows[0].blocks,
            "an enrolled expected-red held must not block"
        );
    }

    /// Positive control: a diff with no changed witness identities projects ZERO rows — zero
    /// lines, nothing blocking, the required context stays green.
    #[test]
    fn empty_changed_set_projects_zero_rows() {
        let rows = ordinary_projection(
            &[],
            &[disposition(
                "m.a",
                RequiredFloorDisposition::DeclinedOutsideGateClosure,
            )],
            &[],
        );
        assert!(
            rows.is_empty(),
            "unchanged identities must project no lines"
        );
    }

    /// UNCHANGED identities never project: only the changed set is joined, so the standing
    /// declined corpus cannot red a PR that did not touch it.
    #[test]
    fn unchanged_declined_identities_project_nothing() {
        let rows = ordinary_projection(
            &["m.mine".to_string()],
            &[
                disposition("m.mine", RequiredFloorDisposition::Planned),
                disposition(
                    "corpus.debt",
                    RequiredFloorDisposition::DeclinedOutsideGateClosure,
                ),
            ],
            &[terminal("m.mine", ClaimOutcome::Pass)],
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].identity, "m.mine");
        assert!(!rows[0].blocks);
    }

    /// The identity spelling is `<authored module path>.<function>` — the disposition
    /// receipt's own grain — read from each touched file's `module` header.
    #[test]
    fn identity_spelling_joins_module_header_and_function() {
        let dir = std::env::temp_dir().join(format!(
            "changed_witness_identity_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("fixture dir");
        std::fs::write(
            dir.join("added_test.dag"),
            "module fixture.changed_witness_spelling\n\ntest fn added_holds() -> Bool { true }\n",
        )
        .expect("fixture write");
        let mut edited = std::collections::HashSet::new();
        edited.insert(("added_test.dag".to_string(), "added_holds".to_string()));
        let identities =
            changed_witness_identities_from_edited_test_fns(&dir, &edited, &Default::default())
                .expect("identities");
        assert_eq!(
            identities,
            vec!["fixture.changed_witness_spelling.added_holds".to_string()]
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    // THE FOUR CONTROLS ON QUARANTINE-KEYED SELECTION, and (a) is deliberately written first
    // because it is the arm that stops this degrading into holding everything.
    //
    // The subject is `changed_witness_identities_from_edited_test_fns`, whose output IS the
    // selection: an identity it returns is planned, one it omits is not. So these assert
    // NOT-SCHEDULED rather than not-failing, which are different claims and only the first is
    // what the admission promises.

    /// Build a two-witness fixture in one file: both are long-home, only one is admitted.
    fn quarantine_selection_fixture(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gunbc-quarantine-selection-{tag}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("fixture dir");
        std::fs::write(
            dir.join("probe_test.dag"),
            "module v2.test.long.quarantine_selection_fixture

             test fn admitted_reds() -> Bool { false }
             test fn unadmitted_reds() -> Bool { false }
",
        )
        .expect("fixture write");
        dir
    }

    fn edited(pairs: &[(&str, &str)]) -> std::collections::HashSet<(String, String)> {
        pairs
            .iter()
            .map(|(f, n)| (f.to_string(), n.to_string()))
            .collect()
    }

    /// (a) A LONG-HOME WITNESS WITH NO ADMISSION IS STILL SELECTED. This is the discriminating
    /// RED for the whole change: an exclusion keyed on the lane rather than on the admission
    /// would drop this row too, and the floor would stop noticing a failing witness the PR
    /// touched. Sitting in `v2.test.long.` earns nothing by itself.
    #[test]
    fn an_unadmitted_long_home_witness_is_still_selected_as_a_changed_witness() {
        let dir = quarantine_selection_fixture("unadmitted");
        let admitted: std::collections::HashSet<(String, String)> =
            [("probe_test.dag".to_string(), "admitted_reds".to_string())]
                .into_iter()
                .collect();
        let identities = changed_witness_identities_from_edited_test_fns(
            &dir,
            &edited(&[("probe_test.dag", "unadmitted_reds")]),
            &admitted,
        )
        .expect("identities");
        assert_eq!(
            identities,
            vec!["v2.test.long.quarantine_selection_fixture.unadmitted_reds".to_string()],
            "a long-home witness with no quarantine admission must still be planned"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// (b) AN ADMITTED WITNESS THE DIFF EDITED IS NOT SELECTED. Not merely "does not fail" —
    /// it is absent from the identities, so nothing plans it.
    #[test]
    fn a_quarantine_admitted_witness_is_not_selected_even_when_edited() {
        let dir = quarantine_selection_fixture("admitted");
        let admitted: std::collections::HashSet<(String, String)> =
            [("probe_test.dag".to_string(), "admitted_reds".to_string())]
                .into_iter()
                .collect();
        let identities = changed_witness_identities_from_edited_test_fns(
            &dir,
            &edited(&[("probe_test.dag", "admitted_reds")]),
            &admitted,
        )
        .expect("identities");
        assert!(
            identities.is_empty(),
            "an edited quarantine-admitted witness must not be scheduled: {identities:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// (c) THE KEY IS THE EXACT CADENCE, NOT "HAS AN ADMISSION". A row admitted under a
    /// different cadence is a different obligation, so it is absent from the quarantine set and
    /// stays selected. Asserted through the real authority rather than the fixture set, since
    /// the cadence read is where a widened head would leak.
    #[test]
    fn only_quarantine_probe_cadence_rows_reach_the_selection_exclusion() {
        let quarantine: std::collections::HashSet<(String, String)> =
            crate::cli_run::quarantine_probe_admission_pairs()
                .into_iter()
                .collect();
        let all: std::collections::HashSet<(String, String)> =
            crate::cli_run::explicit_witness_admission_pairs()
                .into_iter()
                .collect();
        assert!(
            !quarantine.is_empty(),
            "fixture drift: the authority declares quarantine probes"
        );
        assert!(
            quarantine.is_subset(&all),
            "every quarantine row is an admission row"
        );
        // The positive control: some admitted row is NOT a quarantine probe. Without this a
        // reader that returned every admission would satisfy the subset check above.
        assert!(
            all.len() > quarantine.len(),
            "the authority carries admissions under other cadences; if this ever became an \
             equality the exclusion would have widened to every admitted witness"
        );
        std::fs::remove_dir_all(std::env::temp_dir().join("gunbc-quarantine-noop")).ok();
    }

    /// The four rows that produced the incident are actually in the set this reads — an exclusion
    /// keyed on a head that parsed nothing would pass every test above while changing nothing.
    #[test]
    fn the_origin_probe_frontier_rows_are_read_as_quarantine_admissions() {
        let quarantine: std::collections::HashSet<(String, String)> =
            crate::cli_run::quarantine_probe_admission_pairs()
                .into_iter()
                .collect();
        let entry = "src/v2/test/claim/long/production_qualification_origin_probe_witness_test.dag";
        for f in [
            "witness_direct_door_smoke_requires_closure_or_call_occurrence_frontier_reds",
            "witness_ingest_mint_surface_requires_call_occurrence_frontier_reds",
            "witness_live_scope_fixture_origin_discoverable_in_entry_closure_reds",
            "witness_live_structural_fixture_mint_site_discovered_reds",
        ] {
            assert!(
                quarantine.contains(&(entry.to_string(), f.to_string())),
                "{f} must be read as a quarantine admission"
            );
        }
    }

    /// A touched file declaring a test fn without a module header REFUSES — the identity
    /// cannot be spelled, and dropping it would exempt exactly the malformed case.
    #[test]
    fn missing_module_header_refuses_identity_derivation() {
        let dir = std::env::temp_dir().join(format!(
            "changed_witness_headerless_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("fixture dir");
        std::fs::write(
            dir.join("bare_test.dag"),
            "test fn nameless() -> Bool { true }\n",
        )
        .expect("fixture write");
        let mut edited = std::collections::HashSet::new();
        edited.insert(("bare_test.dag".to_string(), "nameless".to_string()));
        let err =
            changed_witness_identities_from_edited_test_fns(&dir, &edited, &Default::default())
                .expect_err("headerless file must refuse");
        assert!(err.contains("no module header"), "cause named: {err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// FLOOR-CHANGED-COST-0 host arms. The model fold is witnessed in
    /// `v2.test.floor_changed_witness`; these assert that the HOST realization joins the same
    /// two populations onto it, which is the seam the .dag fold cannot reach.
    fn observation(cpu: u64) -> HashMap<String, ChangedWitnessCostObservation> {
        let mut map = HashMap::new();
        map.insert(
            "m.a".to_string(),
            ChangedWitnessCostObservation {
                cpu_clock_nanos: u128::from(cpu) * 1_000_000,
                wall_clock_nanos: u128::from(cpu + 15) * 1_000_000,
                cpu_line_ms: 500,
            },
        );
        map
    }

    fn verdict_only_set() -> HashSet<String> {
        let mut set = HashSet::new();
        set.insert("m.a".to_string());
        set
    }

    /// A changed cost-debt identity that PASSED past the CPU line is green, and the row carries
    /// the published measurement rather than laundering it into an ordinary pass.
    #[test]
    fn verdict_only_pass_over_the_line_is_green_and_carries_its_cost() {
        let rows = changed_witness_projection_rows(
            &["m.a".to_string()],
            &[disposition(
                "m.a",
                RequiredFloorDisposition::PlannedAsChangedWitness,
            )],
            &[terminal(
                "m.a",
                ClaimOutcome::CompletedOverBudget {
                    elapsed_ms: 505,
                    budget_ms: 500,
                    kind: BudgetKind::Cpu,
                },
            )],
            &verdict_only_set(),
            &observation(505),
            &no_wet_lane(),
            TEST_CANDIDATE,
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].standing,
            "planned-and-passed-with-cost-debt-observed"
        );
        assert!(!rows[0].blocks);
        assert_eq!(
            rows[0].cost.expect("cost published").cpu_clock_nanos,
            505_000_000
        );
    }

    /// THE DISCRIMINATING CONTROL: the identical terminal outcome for an identity the cost-debt
    /// roster does NOT enroll still reds. One population membership separates the two.
    #[test]
    fn ordinary_changed_identity_over_the_line_still_reds() {
        let rows = ordinary_projection(
            &["m.a".to_string()],
            &[disposition(
                "m.a",
                RequiredFloorDisposition::PlannedAsChangedWitness,
            )],
            &[terminal(
                "m.a",
                ClaimOutcome::CompletedOverBudget {
                    elapsed_ms: 505,
                    budget_ms: 500,
                    kind: BudgetKind::Cpu,
                },
            )],
        );
        assert_eq!(rows[0].standing, "planned-without-terminal-verdict");
        assert!(rows[0].blocks);
        assert!(rows[0].cost.is_none());
    }

    /// A stood-down gate with nothing published in its place REFUSES.
    #[test]
    fn verdict_only_without_a_published_cost_reds() {
        let rows = changed_witness_projection_rows(
            &["m.a".to_string()],
            &[disposition(
                "m.a",
                RequiredFloorDisposition::PlannedAsChangedWitness,
            )],
            &[terminal("m.a", ClaimOutcome::Pass)],
            &verdict_only_set(),
            &HashMap::new(),
            &no_wet_lane(),
            TEST_CANDIDATE,
        );
        assert_eq!(
            rows[0].standing,
            "cost-observation-missing-under-verdict-only"
        );
        assert!(rows[0].blocks);
    }

    /// The override moves the COST arm and nothing else: a semantic failure is still red.
    #[test]
    fn verdict_only_semantic_failure_still_reds() {
        let rows = changed_witness_projection_rows(
            &["m.a".to_string()],
            &[disposition(
                "m.a",
                RequiredFloorDisposition::PlannedAsChangedWitness,
            )],
            &[terminal("m.a", ClaimOutcome::Fail)],
            &verdict_only_set(),
            &observation(505),
            &no_wet_lane(),
            TEST_CANDIDATE,
        );
        assert_eq!(rows[0].standing, "planned-without-terminal-verdict");
        assert!(rows[0].blocks);
    }

    /// A CHANGED IDENTITY WHOSE HERMETIC ROUTE GAPPED IS ADMITTED ONLY WITH ITS WET TERMINAL, and
    /// this pair is the discrimination: the SAME row, the same outcome, differing only in whether
    /// the lane admitted the identity against this candidate. The projection realizes
    /// `v2.workflow.floor_changed_witness` `HermeticRouteGapHeldAndWetPassed`, whose own arms
    /// execute in `v2/test/floor_changed_witness_test.dag`; this control is the host half.
    #[test]
    fn a_route_gap_is_admitted_with_a_wet_terminal_and_blocks_without_one() {
        let changed = ["m.a".to_string()];
        let dispositions = [disposition(
            "m.a",
            RequiredFloorDisposition::PlannedAsChangedWitness,
        )];
        // THE OUTCOME MUST BE A REAL ROUTE GAP, not a pass: `claim_disposition` maps
        // `HostEffectRefused` onto `RouteGapBeforeVerdict`, and that is the only outcome the joined
        // arm applies to. A fixture that passed here would assert nothing about the join.
        let terminal = [terminal(
            "m.a",
            ClaimOutcome::HostEffectRefused {
                operation: "Dir".to_string(),
                ground: v1_interpreter::HermeticEffectGround::NoMockResponse,
            },
        )];
        let admitting = |candidate: &str, refusals: Vec<String>| LocalRepoWetLaneOutcome {
            scheduled: 1,
            candidate: candidate.to_string(),
            admitted: ["m.a".to_string()].into_iter().collect(),
            refusals,
        };
        let project = |lane: &LocalRepoWetLaneOutcome| {
            changed_witness_projection_rows(
                &changed,
                &dispositions,
                &terminal,
                &HashSet::new(),
                &HashMap::new(),
                lane,
                TEST_CANDIDATE,
            )
        };

        let with_wet = project(&admitting(TEST_CANDIDATE, Vec::new()));
        assert_eq!(with_wet.len(), 1);
        assert_eq!(
            with_wet[0].standing,
            "hermetic-route-gap-held-and-wet-passed"
        );
        assert!(!with_wet[0].blocks);

        // NO TERMINAL AT ALL.
        let without_wet = project(&no_wet_lane());
        assert_eq!(without_wet[0].standing, "planned-without-terminal-verdict");
        assert!(
            without_wet[0].blocks,
            "a route gap with no wet terminal must still block"
        );

        // A TERMINAL FROM ANOTHER TREE. Set membership alone would accept this, which is exactly
        // the "it passed somewhere" standing the candidate binding exists to refuse.
        let foreign = project(&admitting("subject-0000000000000000", Vec::new()));
        assert!(
            foreign[0].blocks,
            "an admission bound to another candidate must not green this run"
        );

        // THE LANE ITSELF DISAGREED. One member admitted while the lane's schedule and terminals
        // do not join is a broken roster supplying evidence for its intact rows.
        let refused = project(&admitting(
            TEST_CANDIDATE,
            vec!["m.b: wet terminal missing".to_string()],
        ));
        assert!(
            refused[0].blocks,
            "an admission under a refused lane join must not green this run"
        );
    }

    /// A cost-debt row planned by the changed override is NOT stale, and an ordinarily planned
    /// one still is — the pair is what keeps the new arm from silencing the refusal.
    #[test]
    fn overridden_roster_row_is_not_stale_but_an_ordinary_planned_one_is() {
        let mut roster = HashSet::new();
        roster.insert("m.a".to_string());
        roster.insert("m.b".to_string());
        let mut dispositions = HashMap::new();
        dispositions.insert(
            "m.a".to_string(),
            RequiredFloorDisposition::PlannedAsChangedWitness,
        );
        dispositions.insert("m.b".to_string(), RequiredFloorDisposition::Planned);
        let rows = partition_cost_debt_roster(&roster, &dispositions);
        assert_eq!(
            rows.iter().find(|(q, _)| *q == "m.a").map(|(_, s)| *s),
            Some(CostDebtRosterStanding::WithholdOverriddenForChangedVerdict)
        );
        assert_eq!(
            rows.iter().find(|(q, _)| *q == "m.b").map(|(_, s)| *s),
            Some(CostDebtRosterStanding::DeclaredButNotWithheld)
        );
    }
}

#[cfg(test)]
mod expected_red_roster_join_suppression_tests {
    use crate::v1_compiler_expected_red_roster_join::{
        disposition_label, disposition_reason, expected_red_roster_join_not_evaluated,
        expected_red_roster_join_roster_len, expected_red_roster_join_suppressed,
        finalize_not_observed, new_expected_red_roster_join_report, record_suppressed,
        ExpectedRedSuppressionGround,
    };
    use std::rc::Rc;

    fn report_over(
        identities: &[&str],
    ) -> Rc<crate::v1_compiler_expected_red_roster_join::ExpectedRedRosterJoinReport> {
        new_expected_red_roster_join_report(
            Some("fixture-head".to_string()),
            "fixture".to_string(),
            Rc::new(
                identities
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<im::Vector<String>>(),
            ),
        )
    }

    /// A suppressed identity is IN the report and carries the ground that removed it. This is the
    /// repair's whole subject: before it, a suppressed identity was absent from the roster the
    /// report was built over, so the run_note's "every enrolled identity" sentence described a
    /// population the artifact did not contain.
    #[test]
    fn a_suppressed_identity_is_reported_with_its_ground() {
        let report = report_over(&["a.b.c", "d.e.f"]);
        let report = record_suppressed(
            report,
            "a.b.c".to_string(),
            ExpectedRedSuppressionGround::OutsideRequiredGate,
        );
        assert_eq!(expected_red_roster_join_roster_len(report.clone()), 2);
        assert_eq!(expected_red_roster_join_suppressed(report.clone()), 1);
        let row = report
            .rows
            .iter()
            .find(|r| r.identity == "a.b.c")
            .expect("suppressed identity is in the report");
        assert_eq!(disposition_label(row.disposition.clone()), "suppressed");
        assert_eq!(
            disposition_reason(row.disposition.clone()),
            "suppressed_outside_required_gate"
        );
    }

    /// THE DISCRIMINATING RED, and it is not the arm above. `finalize_not_observed` rewrites every
    /// row still carrying the initial placeholder into `not_in_executed_manifest` -- which is TRUE
    /// of a row nobody executed and FALSE of a row nobody attempted. If it reached suppressed rows,
    /// the artifact would report the 39 outside-gate identities as "no matching claim executed",
    /// losing the only fact that distinguishes a dormant enrolment from an unobserved one, and
    /// every assertion in the test above would still pass because it runs before finalize.
    #[test]
    fn finalize_does_not_overwrite_a_suppressed_row_with_not_in_executed_manifest() {
        let report = report_over(&["a.b.c", "d.e.f"]);
        let report = record_suppressed(
            report,
            "a.b.c".to_string(),
            ExpectedRedSuppressionGround::WithheldCostDebt,
        );
        let report = finalize_not_observed(report);
        let suppressed = report
            .rows
            .iter()
            .find(|r| r.identity == "a.b.c")
            .expect("suppressed identity survives finalize");
        assert_eq!(
            disposition_label(suppressed.disposition.clone()),
            "suppressed"
        );
        assert_eq!(
            disposition_reason(suppressed.disposition.clone()),
            "suppressed_withheld_cost_debt"
        );
        // The positive control: an ordinary unobserved row IS finalized, so the assertion above
        // cannot be satisfied by finalize doing nothing at all.
        let unobserved = report
            .rows
            .iter()
            .find(|r| r.identity == "d.e.f")
            .expect("the other identity is present");
        assert_eq!(
            disposition_label(unobserved.disposition.clone()),
            "not_evaluated"
        );
        assert_eq!(
            disposition_reason(unobserved.disposition.clone()),
            "not_in_executed_manifest"
        );
        assert_eq!(expected_red_roster_join_not_evaluated(report.clone()), 1);
        assert_eq!(expected_red_roster_join_suppressed(report), 1);
    }
}

#[cfg(test)]
mod pure_producer_share_refused_carrier_overlap_tests {
    use super::*;
    use crate::cli_run::shared_fill;

    const CACHE: &str = "cross_claim_pure_share";

    fn refused_row(producer: &str, carriers: &[&str]) -> RefusedShareRow {
        RefusedShareRow {
            producer: producer.to_string(),
            verdict: "MeasuredServeAboveRecompute".to_string(),
            carrier_modules: carriers.iter().map(|m| (*m).to_string()).collect(),
        }
    }

    /// Bill one fill of `key` to a claim in `module`, exactly as the interpreter's share
    /// observer does: the ledger key is the BARE function name.
    fn fill_from(module: &str, key: &str) {
        shared_fill::set_current_claim(Some(&format!("{module}.a_witness")));
        shared_fill::begin_fill();
        shared_fill::record_fill(CACHE, key, 0);
        shared_fill::set_current_claim(None);
    }

    fn install(admitted: &[&str], refused: Vec<RefusedShareRow>) {
        PURE_PRODUCER_SHARE_ROSTER.with(|r| {
            *r.borrow_mut() = Some(PureProducerShareRoster {
                admitted_qualified: admitted.iter().map(|q| (*q).to_string()).collect(),
                refused,
            });
        });
    }

    /// THE DISCRIMINATING RED. A distinct admitted identity, whose own cost nothing measured,
    /// serving into a module a refused row was measured over and withdrawn from — the
    /// `rust_target_model_staging` shape, in miniature.
    #[test]
    fn an_admitted_key_serving_a_refused_rows_carriers_stops_the_line() {
        install(
            &["v2.extdeps.languages.rust.rust_target_model_staging"],
            vec![refused_row(
                "v2.extdeps.languages.rust.rust_target_model",
                &["v2.test.emit.produced_decl_two_target"],
            )],
        );
        // THE LEDGER OBSERVES A FILL UNDER THE REFUSED PRODUCER'S OWN KEY. That is the
        // withdrawn computation recurring, not a neighbour sharing a module, and it is the only
        // thing this wall is entitled to charge. The admitted spelling is reached from the
        // roster rather than named by it, which is why the static fold cannot see it.
        fill_from("v2.test.emit.produced_decl_two_target", "rust_target_model");
        let why = refuse_pure_producer_share_refused_carrier_overlap()
            .expect_err("a refused producer's own key recurring over its carriers must refuse");
        // The three things the diagnostic owes: the admitted subject, the row it inherited the
        // carriers from, and that the trigger was the roster rather than this key's own cost.
        assert!(
            why.contains("cause=PureProducerShareRefusedCarrierOverlap"),
            "{why}"
        );
        assert!(
            why.contains("admitted_producer=<reached-from-roster>:rust_target_model"),
            "{why}"
        );
        assert!(
            why.contains("refused_row=v2.extdeps.languages.rust.rust_target_model"),
            "{why}"
        );
        assert!(
            why.contains("WHAT CHANGED IS THE ROSTER, NOT THIS KEY'S OWN COST"),
            "{why}"
        );
        assert!(
            why.contains("shared_carrier_modules=v2.test.emit.produced_decl_two_target"),
            "{why}"
        );
    }

    /// THE POSITIVE CONTROL, varying exactly the consuming module. Same roster, same refused
    /// row, same admitted key — a fill that reaches none of the measured carriers is ordinary.
    #[test]
    fn the_same_key_serving_modules_no_refusal_measured_is_ordinary() {
        install(
            &["v2.extdeps.languages.rust.rust_target_model_staging"],
            vec![refused_row(
                "v2.extdeps.languages.rust.rust_target_model",
                &["v2.test.emit.produced_decl_two_target"],
            )],
        );
        fill_from(
            "v2.test.claim.bash_command_fold",
            "rust_target_model_staging",
        );
        assert!(refuse_pure_producer_share_refused_carrier_overlap().is_ok());
    }

    /// THE VACUITY REFUSAL. Fills recorded with no consuming module anywhere is the shape that
    /// produced this wall's own false green: a join whose domain is empty answers clean without
    /// comparing anything. Recorded outside any claim, so the ledger has fills and no modules.
    #[test]
    fn a_ledger_whose_fills_carry_no_modules_refuses_instead_of_reading_clean() {
        install(
            &["v2.extdeps.languages.rust.rust_target_model_staging"],
            vec![refused_row(
                "v2.extdeps.languages.rust.rust_target_model",
                &["v2.test.emit.produced_decl_two_target"],
            )],
        );
        shared_fill::set_current_claim(None);
        shared_fill::begin_fill();
        shared_fill::record_fill(CACHE, "rust_target_model_staging", 0);
        let why = refuse_pure_producer_share_refused_carrier_overlap()
            .expect_err("an empty observed domain must refuse rather than read clean");
        assert!(
            why.contains("cause=PureProducerShareObservedCarriersVacuous"),
            "{why}"
        );
    }

    /// THE NARROWING, EXECUTED. A refused row that measured NO effect over its consumers has
    /// nothing for a later identity to inherit, so an admitted key reaching those same modules is
    /// ordinary. This is the arm that run 33696651737 proved was needed: it refused
    /// grammar_relation_row_for_emitted over rust_add_emit_translate, an overlap with a row whose
    /// own measurement found zero.
    #[test]
    fn a_refused_row_that_measured_no_effect_transfers_nothing_through_its_carriers() {
        install(
            &["v2.compiler.translate.grammar_relation_row_for_emitted"],
            vec![RefusedShareRow {
                producer: "v2.extdeps.languages.rust.rust_target_model_core_edges".to_string(),
                verdict: "NoMeasuredEffectOverItsConsumers".to_string(),
                carrier_modules: ["v2.test.manual.rust_add_emit_translate".to_string()]
                    .into_iter()
                    .collect(),
            }],
        );
        fill_from(
            "v2.test.manual.rust_add_emit_translate",
            "grammar_relation_row_for_emitted",
        );
        assert!(refuse_pure_producer_share_refused_carrier_overlap().is_ok());
    }

    /// AND THE SAME OVERLAP UNDER A MEASURED-COST VERDICT STILL REFUSES, so the narrowing is to
    /// the verdict and not to the join. Identical roster, identical key, identical module — only
    /// the refused row's verdict differs.
    #[test]
    fn the_same_overlap_under_a_measured_cost_verdict_still_stops_the_line() {
        install(
            &["v2.compiler.translate.grammar_relation_row_for_emitted"],
            vec![RefusedShareRow {
                producer: "v2.extdeps.languages.rust.rust_target_model_core_edges".to_string(),
                verdict: "MeasuredServeAboveRecompute".to_string(),
                carrier_modules: ["v2.test.manual.rust_add_emit_translate".to_string()]
                    .into_iter()
                    .collect(),
            }],
        );
        fill_from(
            "v2.test.manual.rust_add_emit_translate",
            "rust_target_model_core_edges",
        );
        let why = refuse_pure_producer_share_refused_carrier_overlap()
            .expect_err("a measured-cost row's carriers still transfer");
        assert!(
            why.contains("cause=PureProducerShareRefusedCarrierOverlap"),
            "{why}"
        );
    }

    /// CO-LOCATION IS NOT CAUSAL TRANSFER, AND THIS IS THE CONTROL THAT SAYS SO BY EXECUTION.
    /// An earlier revision of this wall refused here, on module-set intersection alone: a
    /// DIFFERENT admitted producer whose fills reach a module the refused row was measured over.
    /// Sharing a consuming module establishes neither one computation identity nor any transfer
    /// of a measured cost, so refusing on it charges an unrelated producer with a harm nothing
    /// measured — and would let any admitted producer reaching that module stop the required
    /// floor. Raised as review 59213 finding 2 against gunbc#10141 and enrolled here rather than
    /// fixed silently, because this exact fixture was GREEN under the old trigger.
    #[test]
    fn a_different_producer_merely_sharing_a_carrier_module_does_not_refuse() {
        install(
            &["v2.compiler.translate.grammar_relation_row_for_emitted"],
            vec![RefusedShareRow {
                producer: "v2.extdeps.languages.rust.rust_target_model_core_edges".to_string(),
                verdict: "MeasuredServeAboveRecompute".to_string(),
                carrier_modules: ["v2.test.manual.rust_add_emit_translate".to_string()]
                    .into_iter()
                    .collect(),
            }],
        );
        fill_from(
            "v2.test.manual.rust_add_emit_translate",
            "grammar_relation_row_for_emitted",
        );
        assert!(
            refuse_pure_producer_share_refused_carrier_overlap().is_ok(),
            "co-location must not be read as inheritance"
        );
    }

    /// AN OVERLAP THE WALL CANNOT ATTRIBUTE REFUSES RATHER THAN PICKING ONE. The ledger key is
    /// the bare function name while admission is by resolved declaration, so two admitted rows
    /// spelling one bare name leave the overlap real and its subject unknown — and a diagnostic
    /// that named either would be naming a fabricated subject.
    #[test]
    fn an_overlap_whose_bare_key_two_admitted_rows_claim_refuses_as_unattributable() {
        install(
            &["a.module.shared_spelling", "b.module.shared_spelling"],
            vec![refused_row(
                "v2.extdeps.languages.rust.shared_spelling",
                &["v2.test.emit.produced_decl_two_target"],
            )],
        );
        // The refused producer's own key recurs over its own carriers, so the identity trigger
        // fires — and then the bare key is claimed by two admitted spellings, so naming either
        // one would be a fabricated subject.
        fill_from("v2.test.emit.produced_decl_two_target", "shared_spelling");
        let why = refuse_pure_producer_share_refused_carrier_overlap()
            .expect_err("an unattributable overlap must refuse");
        assert!(
            why.contains("cause=PureProducerShareOverlapSubjectAmbiguous"),
            "{why}"
        );
        assert!(
            why.contains("a.module.shared_spelling, b.module.shared_spelling"),
            "{why}"
        );
    }

    /// A LEDGER WITH NO ROSTER IS NOT A CLEAN RUN. The tier filled under a roster this wall
    /// cannot read, so there is no population to adjudicate and the arm refuses rather than
    /// reporting the absence as no overlap.
    #[test]
    fn fills_without_an_installed_roster_refuse_rather_than_read_as_clean() {
        PURE_PRODUCER_SHARE_ROSTER.with(|r| *r.borrow_mut() = None);
        fill_from("v2.test.claim.bash_command_fold", "bash_fold_lex");
        let why = refuse_pure_producer_share_refused_carrier_overlap()
            .expect_err("fills with no installed roster must refuse");
        assert!(
            why.contains("cause=PureProducerShareLedgerWithoutRoster"),
            "{why}"
        );
    }
}
