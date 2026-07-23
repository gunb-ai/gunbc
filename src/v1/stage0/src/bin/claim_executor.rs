#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[cfg(test)]
use v1_compiler::cli_run::workspace_root;
use v1_compiler::cli_run::{
    compute_histogram_data, compute_witness_timing_rows, enable_floor_compile_clean_lazy_install,
    install_floor_compile_clean_receipt, make_eval_context, resolve_entry_graph,
    resolve_entry_graph_shared, run_claim, run_discovery_corpus_with_options, run_value, set_phase,
    top_n_slowest_witnesses, ClaimOutcome, DiscoveryCorpusOptions, DiscoverySummary,
    DiscoveryWidthPolicy, FloorPhase, HistogramData, NodeFrontierSelectionMode, PhaseProfile,
    TimingPercentiles, WitnessTimingRow, DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N,
};
use v1_compiler::memory_governor::{
    binding_cap_cgroup_dir, binding_high_cgroup_dir, floor_budget_below_minimum_footprint,
    leaf_cgroup_dir, mem_total_bytes, memory_events_field, memory_pressure_some_avg10,
    read_cgroup_raw, read_cgroup_u64, AdmittedSlot, MemoryGovernor,
};
use v1_compiler::v1_interpreter::{
    color_enabled, paint, run_in_context_with_args, sgr, ExecutionMode, InterpContext, Value,
};

#[derive(Clone, Copy, Default)]
struct FalsifierSelfHostWetBudgets {
    wall_budget_ms: Option<u64>,
    interp_eval_budget_ms: Option<u64>,
}

fn read_positive_budget_ms(
    plan_ctx: &InterpContext,
    function: &str,
) -> Result<Option<u64>, String> {
    match run_value(plan_ctx, function) {
        Ok(Value::Int(n)) if n > 0 => Ok(Some(n as u64)),
        Ok(other) => Err(format!(
            "claim_executor: {function} must be a positive Int, got {other:?} (fail-closed)"
        )),
        Err(msg) => Err(format!(
            "claim_executor: {function} is unavailable (fail-closed): {msg}"
        )),
    }
}

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority:
/// `gunbc.ci_spec.gunbc_ci_floor_batch_stop_policy_claim_executor_seed_note`,
/// type `gunbc.ci_spec.FloorBatchStopPolicy`):
/// seed-side enum mirror + `run_walk` consumer for the event-scoped batch halt;
/// policy mapping and plan roster enrollment are delegated to `.dag` eval
/// (`gunbc_ci_floor_batch_stop_policy_for_github_event`,
/// `gunbc_ci_floor_plan_uses_batch_stop_policy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FloorBatchStopPolicy {
    StopBeforeDependents,
    FullLedger,
}

fn plan_uses_floor_batch_stop_policy(plan_ctx: &InterpContext, plan_function: &str) -> bool {
    match run_in_context_with_args(
        plan_ctx,
        "gunbc_ci_floor_plan_uses_batch_stop_policy",
        &[(
            Some("plan_function".to_string()),
            Value::Str(plan_function.to_string()),
        )],
        true,
    ) {
        Ok(Value::Bool(b)) => b,
        Ok(other) => {
            eprintln!(
                "claim_executor: gunbc_ci_floor_plan_uses_batch_stop_policy returned \
                 non-bool {other:?} (treating as not enrolled)"
            );
            false
        }
        Err(msg) => {
            eprintln!(
                "claim_executor: gunbc_ci_floor_plan_uses_batch_stop_policy unavailable: {msg}"
            );
            false
        }
    }
}

fn resolve_floor_batch_stop_policy(
    plan_ctx: &InterpContext,
    plan_function: &str,
) -> FloorBatchStopPolicy {
    if !plan_uses_floor_batch_stop_policy(plan_ctx, plan_function) {
        return FloorBatchStopPolicy::StopBeforeDependents;
    }
    let event = std::env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    match run_in_context_with_args(
        plan_ctx,
        "gunbc_ci_floor_batch_stop_policy_for_github_event",
        &[(Some("event".to_string()), Value::Str(event))],
        true,
    ) {
        Ok(Value::Variant { variant_name, .. }) => {
            if plan_ctx.sym_eq(variant_name, "StopBeforeDependents") {
                FloorBatchStopPolicy::StopBeforeDependents
            } else if plan_ctx.sym_eq(variant_name, "FullLedger") {
                FloorBatchStopPolicy::FullLedger
            } else {
                eprintln!(
                    "claim_executor: unrecognized FloorBatchStopPolicy variant \
                     (defaulting to FullLedger, fail-closed)"
                );
                FloorBatchStopPolicy::FullLedger
            }
        }
        Ok(other) => {
            eprintln!(
                "claim_executor: floor batch stop policy returned non-variant {other:?} \
                 (defaulting to FullLedger, fail-closed)"
            );
            FloorBatchStopPolicy::FullLedger
        }
        Err(msg) => {
            eprintln!(
                "claim_executor: floor batch stop policy unavailable \
                 (defaulting to FullLedger, fail-closed): {msg}"
            );
            FloorBatchStopPolicy::FullLedger
        }
    }
}

#[derive(Clone)]
enum Runnable {
    SingleClaim {
        entry: String,
        function: String,
        use_walk_memo: bool,
        execution_mode: ExecutionMode,
    },
    DiscoveryBatch {
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        explicit_entries: Vec<(String, String)>,
        node_frontier_selection: NodeFrontierSelectionMode,
        exclude_substrings: Vec<String>,
        discovery_scope_dirs: Vec<String>,
        execution_mode: ExecutionMode,
    },
}

/// The runnable's declared effect envelope, read from its profile's `execution_mode`
/// field (`std.execution_mode`, carried on `RunnableResourceProfile`). Absent PROFILE
/// falls to Hermetic — the fail-closed direction: an undeclared runnable that needs
/// live effects refuses loudly at the effect boundary instead of silently dispatching
/// them (mirrors `runnable_resource_profile_negligible`). A profile that EXISTS but
/// lacks the field is a refusal, not a default (the `node_frontier_selection`
/// precedent: a stale plan must redeclare its semantics, never inherit them silently).
fn execution_mode_from_profile_field(
    fields: Option<&Value>,
    owner: &str,
    ctx: &InterpContext,
) -> Result<ExecutionMode, String> {
    let profile_fields = match fields {
        Some(Value::Record { fields: pf, .. }) | Some(Value::Variant { fields: pf, .. }) => pf,
        Some(other) => {
            return Err(format!(
                "{owner}.profile must be a RunnableResourceProfile record, got {}",
                other.type_label_public()
            ))
        }
        None => return Ok(ExecutionMode::Hermetic),
    };
    match ctx.field(profile_fields, "execution_mode") {
        Some(Value::Variant { variant_name, .. }) => {
            if ctx.sym_eq(*variant_name, "Hermetic") {
                Ok(ExecutionMode::Hermetic)
            } else if ctx.sym_eq(*variant_name, "Wet") {
                Ok(ExecutionMode::Wet)
            } else if ctx.sym_eq(*variant_name, "Record") {
                Ok(ExecutionMode::Record)
            } else {
                Err(format!(
                    "{owner}.profile.execution_mode: unknown ExecutionMode variant `{}`",
                    ctx.resolve(*variant_name)
                ))
            }
        }
        Some(other) => Err(format!(
            "{owner}.profile.execution_mode must be an ExecutionMode variant, got {}",
            other.type_label_public()
        )),
        None => Err(format!(
            "{owner}.profile is present but declares no execution_mode — the plan row \
             must declare its effect envelope (Hermetic / Wet / Record); no silent default"
        )),
    }
}

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("claim_executor: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

/// Path-valued arguments resolve against the PROCESS CWD at the CLI boundary, refusing
/// on a nonexistent path — never falling back to the compile-time-baked workspace root
/// (`v1_compiler::cli_run::resolve_cli_path_arg`; DESIGN §5 fail-open closed there).
fn require_path_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    let given = require_value(args, idx, flag)?;
    match v1_compiler::cli_run::resolve_cli_path_arg("claim_executor", flag, &given) {
        Ok(resolved) => Ok(resolved),
        Err(msg) => {
            eprintln!("{msg}");
            Err(ExitCode::from(2))
        }
    }
}

fn free_monoid_elems<'a>(value: &'a Value, ctx: &InterpContext) -> Result<Vec<&'a Value>, String> {
    let mut out = Vec::new();
    let mut cur = value;
    loop {
        match cur {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                let head = ctx
                    .field(fields, "head")
                    .ok_or_else(|| "Cons without `head` field".to_string())?;
                out.push(head);
                cur = ctx
                    .field(fields, "tail")
                    .ok_or_else(|| "Cons without `tail` field".to_string())?;
            }
            Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Empty") => {
                return Ok(out);
            }
            Value::List(items) => {
                out.extend(items.iter());
                return Ok(out);
            }
            other => {
                return Err(format!(
                    "expected a List (Cons/Empty), got {}",
                    other.type_label_public()
                ))
            }
        }
    }
}

fn str_list_from_value(value: &Value, ctx: &InterpContext) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for elem in free_monoid_elems(value, ctx)? {
        match elem {
            Value::Str(s) => out.push(s.clone()),
            other => {
                return Err(format!(
                    "expected a List<String> element, got {}",
                    other.type_label_public()
                ))
            }
        }
    }
    Ok(out)
}

fn str_field(
    fields: &[(v1_compiler::v1_interpreter::Symbol, Value)],
    name: &str,
    owner: &str,
    ctx: &InterpContext,
) -> Result<String, String> {
    match ctx.field(fields, name) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(other) => Err(format!(
            "{}.{} is {}, not String",
            owner,
            name,
            ctx.format_value(other)
        )),
        None => Err(format!("{} missing field `{}`", owner, name)),
    }
}

fn runnable_from_value(value: &Value, ctx: &InterpContext) -> Result<Runnable, String> {
    match value {
        Value::Record { type_name, fields } if ctx.sym_eq(*type_name, "ClaimRef") => {
            Ok(Runnable::SingleClaim {
                entry: str_field(fields, "entry", "ClaimRef", ctx)?,
                function: str_field(fields, "function", "ClaimRef", ctx)?,
                use_walk_memo: false,
                // ClaimRef carries no profile: fail-closed envelope (see
                // execution_mode_from_profile_field).
                execution_mode: ExecutionMode::Hermetic,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableSingleClaim") => {
            let entry = str_field(fields, "entry", "RunnableSingleClaim", ctx)?;
            let function = str_field(fields, "function", "RunnableSingleClaim", ctx)?;
            let profile = ctx.field(fields, "profile");
            let use_walk_memo = match profile {
                Some(Value::Record { fields: pf, .. })
                | Some(Value::Variant { fields: pf, .. }) => {
                    matches!(
                        ctx.field(pf, "heavy_whole_tree_resolve"),
                        Some(Value::Bool(true))
                    )
                }
                _ => false,
            };
            let execution_mode =
                execution_mode_from_profile_field(profile, "RunnableSingleClaim", ctx)?;
            Ok(Runnable::SingleClaim {
                entry,
                function,
                use_walk_memo,
                execution_mode,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableDiscoveryBatch") => {
            let source_roots = match ctx.field(fields, "source_roots") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => {
                    return Err("RunnableDiscoveryBatch missing field `source_roots`".to_string())
                }
            };
            let scan_dirs = match ctx.field(fields, "scan_dirs") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => return Err("RunnableDiscoveryBatch missing field `scan_dirs`".to_string()),
            };
            let explicit_entries = match ctx.field(fields, "explicit_entries") {
                Some(v) => {
                    let mut out = Vec::new();
                    for elem in free_monoid_elems(v, ctx)? {
                        let efields = match elem {
                            Value::Record { fields, .. } => fields,
                            Value::Variant { fields, .. } => fields,
                            other => {
                                return Err(format!(
                                    "RunnableDiscoveryBatch.explicit_entries element is {}, not a record",
                                    other.type_label_public()
                                ))
                            }
                        };
                        out.push((
                            str_field(efields, "entry", "explicit_entries", ctx)?,
                            str_field(efields, "function", "explicit_entries", ctx)?,
                        ));
                    }
                    out
                }
                None => Vec::new(),
            };
            let node_frontier_selection = match ctx.field(fields, "node_frontier_selection") {
                Some(Value::Variant { variant_name, .. }) => {
                    if ctx.sym_eq(*variant_name, "SelectionOff") {
                        NodeFrontierSelectionMode::Off
                    } else if ctx.sym_eq(*variant_name, "SelectionApplied") {
                        NodeFrontierSelectionMode::Applied
                    } else if ctx.sym_eq(*variant_name, "SelectionPredictOnly") {
                        NodeFrontierSelectionMode::PredictOnly
                    } else {
                        return Err(format!(
                            "RunnableDiscoveryBatch.node_frontier_selection: unknown \
                                 NodeFrontierSelection variant `{}`",
                            ctx.resolve(*variant_name)
                        ));
                    }
                }
                Some(other) => {
                    return Err(format!(
                        "RunnableDiscoveryBatch.node_frontier_selection must be a \
                             NodeFrontierSelection variant, got {}",
                        other.type_label_public()
                    ))
                }
                // Absent is a REFUSAL, not a default: the former Bool defaulted to false
                // when the field was missing — a fail-open where a stale plan silently
                // ran without its declared selection semantics.
                None => {
                    return Err(
                        "RunnableDiscoveryBatch.node_frontier_selection is absent — the \
                             plan row must declare its selection mode (SelectionOff / \
                             SelectionApplied / SelectionPredictOnly); no silent default"
                            .to_string(),
                    )
                }
            };
            let exclude_substrings = match ctx.field(fields, "exclude_substrings") {
                Some(v) => str_list_from_value(v, ctx)?,
                // Field absent means the plan author specified no exclusions — default is empty,
                // not the Rust constant (the model is the sole authority on the plan path).
                None => Vec::new(),
            };
            let discovery_scope_dirs = match ctx.field(fields, "discovery_scope_dirs") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => Vec::new(),
            };
            let execution_mode = execution_mode_from_profile_field(
                ctx.field(fields, "profile"),
                "RunnableDiscoveryBatch",
                ctx,
            )?;
            Ok(Runnable::DiscoveryBatch {
                source_roots,
                scan_dirs,
                explicit_entries,
                node_frontier_selection,
                exclude_substrings,
                discovery_scope_dirs,
                execution_mode,
            })
        }
        other => Err(format!(
            "expected a ClaimRef record or Runnable variant, got {}",
            ctx.format_value(other)
        )),
    }
}

fn batches_from_plan(plan: &Value, ctx: &InterpContext) -> Result<Vec<Vec<Runnable>>, String> {
    let mut batches = Vec::new();
    for batch_val in free_monoid_elems(plan, ctx)? {
        let mut batch = Vec::new();
        for elem in free_monoid_elems(batch_val, ctx)? {
            batch.push(runnable_from_value(elem, ctx)?);
        }
        batches.push(batch);
    }
    Ok(batches)
}

struct ClaimResult {
    function: String,
    ok: bool,
    detail: String,
    /// Wall-clock eval time for this single claim (0 for discovery aggregate).
    wall_nanos: u128,
    /// Resolve time charged to this result; non-zero only on the first claim in a
    /// SharedClaims group (the group resolves once, cost attributed to first claim).
    resolve_nanos: u128,
    /// For discovery batch nodes: sum of per-file resolve times (serial sum, not wall).
    corpus_resolve_nanos: u128,
    /// For discovery batch nodes: sum of per-witness eval times (serial sum, not wall).
    corpus_eval_nanos: u128,
    /// Number of discovery witnesses (non-zero only for discovery batch nodes).
    corpus_witnesses: usize,
}

/// A batch is partitioned into resolve-groups before scheduling. SingleClaims that share one
/// `entry` collapse into a single `SharedClaims` group: the entry's import closure is resolved
/// (and typechecked) EXACTLY ONCE and every claim runs on that one shared interpreter context,
/// instead of each claim re-resolving the identical graph on its own thread. This is the floor's
/// dominant footprint win — batch-2's gate witnesses all live in one file
/// (`dag/tools/floor_effect_gate_witness.dag`, ~0.9 GiB / 106 modules per resolve), so the
/// per-thread-resolve scheme held that graph ~6x concurrently (~4.5 GiB of pure duplication,
/// roughly half the self-RSS). Resolve is a pure function of `(source_roots, entry)`, so sharing
/// the graph across same-entry claims is semantically identical — correctness by construction
/// (DESIGN §2: duplicated work removed; §4: realization may share what the pure spec models apart).
enum BatchUnit {
    SharedClaims {
        entry: String,
        functions: Vec<String>,
        use_walk_memo: bool,
        execution_mode: ExecutionMode,
    },
    UnrunnableSentinel {
        function: String,
    },
    Discovery {
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        explicit_entries: Vec<(String, String)>,
        node_frontier_selection: NodeFrontierSelectionMode,
        exclude_substrings: Vec<String>,
        discovery_scope_dirs: Vec<String>,
        execution_mode: ExecutionMode,
    },
}

/// Partition a batch's runnables into resolve-groups, preserving first-appearance order so the
/// PASS/FAIL log stays stable. SingleClaims with a non-empty `entry` coalesce by `entry`;
/// empty-entry sentinels and DiscoveryBatch nodes stay their own units (each resolves apart).
fn group_batch_units(batch: &[Runnable]) -> Vec<BatchUnit> {
    let mut units: Vec<BatchUnit> = Vec::new();
    // Same-entry claims share one resolved context, so the shared context's
    // execution mode is part of the group key: a Wet gate and a Hermetic gate on
    // the same entry resolve apart rather than silently sharing an envelope.
    let mut entry_to_unit: std::collections::HashMap<(String, ExecutionMode), usize> =
        std::collections::HashMap::new();
    for runnable in batch {
        match runnable {
            Runnable::SingleClaim {
                entry, function, ..
            } if entry.is_empty() => {
                units.push(BatchUnit::UnrunnableSentinel {
                    function: function.clone(),
                });
            }
            Runnable::SingleClaim {
                entry,
                function,
                use_walk_memo,
                execution_mode,
            } => {
                let unit_key = (entry.clone(), *execution_mode);
                if let Some(&idx) = entry_to_unit.get(&unit_key) {
                    if let BatchUnit::SharedClaims {
                        functions,
                        use_walk_memo: existing_memo,
                        ..
                    } = &mut units[idx]
                    {
                        functions.push(function.clone());
                        // A memo claim (heavy whole-tree resolve) may merge into a
                        // group created from a non-memo claim first; OR here so the
                        // group gets the memo path if any member is heavy.
                        *existing_memo |= use_walk_memo;
                    }
                } else {
                    entry_to_unit.insert(unit_key, units.len());
                    units.push(BatchUnit::SharedClaims {
                        entry: entry.clone(),
                        functions: vec![function.clone()],
                        use_walk_memo: *use_walk_memo,
                        execution_mode: *execution_mode,
                    });
                }
            }
            Runnable::DiscoveryBatch {
                source_roots,
                scan_dirs,
                explicit_entries,
                node_frontier_selection,
                exclude_substrings,
                discovery_scope_dirs,
                execution_mode,
            } => units.push(BatchUnit::Discovery {
                source_roots: source_roots.clone(),
                scan_dirs: scan_dirs.clone(),
                explicit_entries: explicit_entries.clone(),
                node_frontier_selection: *node_frontier_selection,
                exclude_substrings: exclude_substrings.clone(),
                discovery_scope_dirs: discovery_scope_dirs.clone(),
                execution_mode: *execution_mode,
            }),
        }
    }
    units
}

fn claim_result_for_outcome(
    function: String,
    outcome: ClaimOutcome,
    wall_nanos: u128,
    resolve_nanos: u128,
) -> ClaimResult {
    match outcome {
        ClaimOutcome::Pass => ClaimResult {
            function,
            ok: true,
            detail: String::new(),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
        ClaimOutcome::Fail => ClaimResult {
            function,
            ok: false,
            detail: "returned Bool(false)".to_string(),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
        ClaimOutcome::NotBool { got } => ClaimResult {
            function,
            ok: false,
            detail: format!("returned `{}`, not Bool", got),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
        ClaimOutcome::RuntimeError { message } => ClaimResult {
            function,
            ok: false,
            detail: format!("runtime error: {}", message),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
    }
}

fn run_batch_unit(
    source_roots: Vec<String>,
    unit: BatchUnit,
    governor: Arc<MemoryGovernor>,
    fast_lane_eval_budget_ms: Option<u64>,
    falsifier_self_host_wet_budgets: FalsifierSelfHostWetBudgets,
) -> Vec<ClaimResult> {
    match unit {
        BatchUnit::UnrunnableSentinel { function } => vec![ClaimResult {
            function,
            ok: false,
            detail: "unrunnable sentinel (unmapped node or non-complete plan) — failing closed"
                .to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        }],
        BatchUnit::Discovery {
            source_roots: roots,
            scan_dirs,
            explicit_entries,
            node_frontier_selection,
            exclude_substrings,
            discovery_scope_dirs,
            execution_mode,
        } => {
            // The fast-lane eval budget (operator 5s rule) governs the HERMETIC per-PR
            // discovery corpus — witnesses whose own eval must stay cheap or move to a
            // long/ lane. A Wet execution batch (the bin-witness roster: compile-clean
            // seam checks, floor-skip/interp-fixture drivers) spends its wall time in
            // declared subprocess I/O, not in eval, and legitimately runs for minutes;
            // the budget's completion-side wall check would kill it for doing exactly its
            // declared job. So the budget applies only to a hermetic batch. This is not a
            // silent widen: the Wet roster is a small explicit set with its own resource
            // profile, and the eval-wedge risk the budget guards (the s1_closure class)
            // lives in the wide hermetic corpus, not here.
            let (effective_fast_lane, wet_wall_budget_ms, wet_interp_budget_ms) =
                if execution_mode.is_hermetic() {
                    (fast_lane_eval_budget_ms, None, None)
                } else {
                    (
                        None,
                        falsifier_self_host_wet_budgets.wall_budget_ms,
                        falsifier_self_host_wet_budgets.interp_eval_budget_ms,
                    )
                };
            vec![run_discovery_batch_node(
                roots,
                scan_dirs,
                explicit_entries,
                node_frontier_selection,
                exclude_substrings,
                discovery_scope_dirs,
                governor,
                execution_mode,
                effective_fast_lane,
                wet_wall_budget_ms,
                wet_interp_budget_ms,
            )]
        }
        BatchUnit::SharedClaims {
            entry,
            functions,
            execution_mode,
            ..
        } => {
            // A gate unit's resolved graph is a real memory resident: take a governor
            // slot for the unit's lifetime so gate threads and discovery workers draw
            // from the same admission window instead of stacking unbounded.
            let mut slot = AdmittedSlot::acquire_blocking(&governor, &format!("gate-unit {entry}"));
            let results =
                run_shared_entry_claims(&source_roots, &entry, &functions, execution_mode);
            slot.note_unit_complete();
            results
        }
    }
}

fn run_shared_entry_claims(
    source_roots: &[String],
    entry: &str,
    functions: &[String],
    execution_mode: ExecutionMode,
) -> Vec<ClaimResult> {
    let resolve_start = Instant::now();
    let (graph, source_indices) = match resolve_entry_graph(source_roots, entry) {
        Ok(pair) => pair,
        Err(msg) => {
            return functions
                .iter()
                .map(|function| ClaimResult {
                    function: function.clone(),
                    ok: false,
                    detail: format!("resolve failed for {}: {}", entry, msg),
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                })
                .collect();
        }
    };
    let resolve_nanos = resolve_start.elapsed().as_nanos();
    let ctx = make_eval_context(&graph, source_indices, execution_mode);
    let mut first = true;
    functions
        .iter()
        .map(|function| {
            set_phase(FloorPhase::Gate, &format!("{entry}::{function}"));
            let claim_start = Instant::now();
            let outcome = run_claim(&ctx, function);
            // Witness frame exit: the memo must not retain values across
            // witnesses sharing this ctx (byte-unbounded, 20GiB-class kills).
            v1_compiler::v1_interpreter::eval_call_memo_frame_exit(&ctx);
            let wall_nanos = claim_start.elapsed().as_nanos();
            let rn = if first {
                first = false;
                resolve_nanos
            } else {
                0
            };
            claim_result_for_outcome(function.clone(), outcome, wall_nanos, rn)
        })
        .collect()
}

/// Run claims whose entry-graph is already resolved and cached in `memo`, resolving on first miss.
/// Runs on the main thread so `InterpContext` (which contains `Rc` fields) never crosses thread
/// boundaries. Subsequent callers for the same entry share the cached context — resolve runs once.
fn run_memo_shared_claims(
    source_roots: &[String],
    entry: &str,
    functions: &[String],
    execution_mode: ExecutionMode,
    memo: &mut std::collections::HashMap<(String, ExecutionMode), InterpContext>,
) -> Vec<ClaimResult> {
    let resolve_start = Instant::now();
    let mut fresh_resolve = false;
    // The cached context carries its execution mode, so the memo key must too —
    // same-entry claims with different declared envelopes resolve apart.
    let memo_key = (entry.to_string(), execution_mode);
    if !memo.contains_key(&memo_key) {
        let (graph, source_indices) = match resolve_entry_graph(source_roots, entry) {
            Ok(pair) => pair,
            Err(msg) => {
                return functions
                    .iter()
                    .map(|function| ClaimResult {
                        function: function.clone(),
                        ok: false,
                        detail: format!("resolve failed for {}: {}", entry, msg),
                        wall_nanos: 0,
                        resolve_nanos: 0,
                        corpus_resolve_nanos: 0,
                        corpus_eval_nanos: 0,
                        corpus_witnesses: 0,
                    })
                    .collect();
            }
        };
        memo.insert(
            memo_key.clone(),
            make_eval_context(&graph, source_indices, execution_mode),
        );
        fresh_resolve = true;
    }
    let resolve_nanos = if fresh_resolve {
        resolve_start.elapsed().as_nanos()
    } else {
        0
    };
    let ctx = memo.get(&memo_key).expect("memo populated above");
    let mut first = fresh_resolve;
    functions
        .iter()
        .map(|function| {
            set_phase(FloorPhase::Gate, &format!("{entry}::{function}"));
            let claim_start = Instant::now();
            let outcome = run_claim(ctx, function);
            // Witness frame exit — this memoized ctx outlives whole entry
            // groups, so per-witness release matters here most of all.
            v1_compiler::v1_interpreter::eval_call_memo_frame_exit(ctx);
            let wall_nanos = claim_start.elapsed().as_nanos();
            let rn = if first {
                first = false;
                resolve_nanos
            } else {
                0
            };
            claim_result_for_outcome(function.clone(), outcome, wall_nanos, rn)
        })
        .collect()
}

/// The medium's `Viewport.width` for boxed output. The seed sources the runtime value from the host
/// (`COLUMNS` when present) and passes it into the `.dag` render model; the model owns how width
/// shapes the box. Conservative 88-col default fits common CI-log and chat viewers; clamped so a
/// hostile `COLUMNS` cannot produce a degenerate box.
fn histogram_output_width() -> i64 {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|w| *w >= 48)
        .unwrap_or(88)
        .min(120)
}

fn clamp_nanos_to_i64(n: u128) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// Render the timing histogram boxes through `dag/gunbc/ci_render.dag` (`render_percentile_box`),
/// the single authority for boxed-Frame width. The seed only supplies measured data + the host
/// viewport width; all layout (borders, padding, duration formatting) lives in `.dag`.
fn render_timing_histogram(
    source_roots: &[String],
    data: &HistogramData,
) -> Result<String, String> {
    let entry = "dag/gunbc/ci_render.dag";
    let (graph, indices) = resolve_entry_graph(source_roots, entry)
        .map_err(|m| format!("resolve failed for {entry}:\n{m}"))?;
    // Pure render evaluation (ci_render.dag fns over measured data) — no effects, so
    // the hermetic envelope is exact; a service call sneaking in refuses loudly.
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let width = histogram_output_width();
    let color = color_enabled();

    let render_box = |title: &str, p: &TimingPercentiles| -> Result<String, String> {
        let value = run_in_context_with_args(
            &ctx,
            "render_percentile_box",
            &[
                (Some("title".to_string()), Value::Str(title.to_string())),
                (
                    Some("p50".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p50)),
                ),
                (
                    Some("p90".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p90)),
                ),
                (
                    Some("p95".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p95)),
                ),
                (
                    Some("p99".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p99)),
                ),
                (
                    Some("p100".to_string()),
                    Value::Int(clamp_nanos_to_i64(p.p100)),
                ),
                (Some("width".to_string()), Value::Int(width)),
                (Some("color".to_string()), Value::Bool(color)),
            ],
            false,
        )
        .map_err(|e| format!("render_percentile_box eval failed: {e}"))?;
        match value {
            Value::Str(s) => Ok(s),
            other => Err(format!(
                "render_percentile_box returned non-string: {other}"
            )),
        }
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Total witnesses: {} (included in histogram); {} skipped (no entry-resolve timing)\n",
        data.included, data.skipped
    ));
    out.push_str(
        "Note: Resolve times are per-entry-amortized (witnesses in an entry share its resolve cost); eval times are per-witness.\n\n",
    );
    out.push_str(&render_box("TOTAL TIME (Resolve + Eval)", &data.total)?);
    out.push('\n');
    out.push_str(&render_box("RESOLVE TIME", &data.resolve)?);
    out.push('\n');
    out.push_str(&render_box("EVAL TIME", &data.eval)?);
    Ok(out)
}

fn slowest_witness_attribution_n() -> usize {
    std::env::var("GUNBC_FLOOR_SLOWEST_N")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N)
}

fn str_list_value(lines: &[String]) -> Value {
    Value::List(Rc::new(
        lines
            .iter()
            .cloned()
            .map(Value::Str)
            .collect::<Vec<_>>()
            .into(),
    ))
}

/// Render the top-N slowest witnesses through `dag/gunbc/ci_render.dag`.
fn render_slowest_witnesses(
    source_roots: &[String],
    rows: &[WitnessTimingRow],
) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let entry = "dag/gunbc/ci_render.dag";
    let (graph, indices) = resolve_entry_graph(source_roots, entry)
        .map_err(|m| format!("resolve failed for {entry}:\n{m}"))?;
    // Pure render evaluation (ci_render.dag fns over measured data) — no effects, so
    // the hermetic envelope is exact; a service call sneaking in refuses loudly.
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let width = histogram_output_width();
    let color = color_enabled();

    let mut body_lines: Vec<String> = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let line = run_in_context_with_args(
            &ctx,
            "slowest_witness_row",
            &[
                (Some("rank".to_string()), Value::Int((i + 1) as i64)),
                (
                    Some("function".to_string()),
                    Value::Str(row.function.clone()),
                ),
                (Some("entry".to_string()), Value::Str(row.entry.clone())),
                (
                    Some("eval_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.eval_nanos)),
                ),
                (
                    Some("resolve_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.resolve_nanos)),
                ),
                (
                    Some("total_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.total_nanos)),
                ),
            ],
            false,
        )
        .map_err(|e| format!("slowest_witness_row eval failed: {e}"))?;
        match line {
            Value::Str(s) => body_lines.push(s),
            other => return Err(format!("slowest_witness_row returned non-string: {other}")),
        }
    }

    // Call out the longest-running witnesses with the .dag box's hot styling (fire glyph + red
    // timer). How many rows count as "hot" is a rendering policy owned by the .dag authority
    // (`slowest_witness_default_hot_count`), not a magic constant here — Rust just feeds the rows.
    let value = run_in_context_with_args(
        &ctx,
        "render_slowest_witnesses_box_default",
        &[
            (Some("body".to_string()), str_list_value(&body_lines)),
            (Some("width".to_string()), Value::Int(width)),
            (Some("color".to_string()), Value::Bool(color)),
        ],
        false,
    )
    .map_err(|e| format!("render_slowest_witnesses_box eval failed: {e}"))?;
    match value {
        Value::Str(s) => Ok(s),
        other => Err(format!(
            "render_slowest_witnesses_box returned non-string: {other}"
        )),
    }
}

fn emit_slowest_witness_attribution(source_roots: &[String], summary: &DiscoverySummary) {
    match compute_witness_timing_rows(summary) {
        Ok(rows) => {
            let n = slowest_witness_attribution_n().min(rows.len());
            if n == 0 {
                return;
            }
            let top = top_n_slowest_witnesses(&rows, n);
            match render_slowest_witnesses(source_roots, &top) {
                Ok(boxed) => {
                    eprintln!("{boxed}");
                    let tail_eval_ms: u128 =
                        top.iter().map(|r| r.eval_nanos).sum::<u128>() / 1_000_000;
                    let total_eval_ms = summary.total_measured_nanos / 1_000_000;
                    let pct = if total_eval_ms == 0 {
                        0.0
                    } else {
                        100.0 * tail_eval_ms as f64 / total_eval_ms as f64
                    };
                    eprintln!(
                        "[attribution] top-{n} slowest witnesses: eval serial-sum {tail_eval_ms}ms ({pct:.1}% of corpus eval)"
                    );
                }
                Err(e) => eprintln!("[attribution] render failed (timings unaffected): {e}"),
            }
        }
        Err(msg) => eprintln!("{msg}"),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_discovery_batch_node(
    source_roots: Vec<String>,
    scan_dirs: Vec<String>,
    explicit_entries: Vec<(String, String)>,
    node_frontier_selection: NodeFrontierSelectionMode,
    exclude_substrings: Vec<String>,
    discovery_scope_dirs: Vec<String>,
    governor: Arc<MemoryGovernor>,
    execution_mode: ExecutionMode,
    fast_lane_eval_budget_ms: Option<u64>,
    wet_receipt_wall_budget_ms: Option<u64>,
    wet_receipt_interp_eval_budget_ms: Option<u64>,
) -> ClaimResult {
    set_phase(FloorPhase::Discovery, "discovery-corpus");
    let label = format!(
        "discovery-corpus[{} root(s)+{} explicit, adaptive width]",
        source_roots.len(),
        explicit_entries.len(),
    );
    match run_discovery_corpus_with_options(
        &source_roots,
        &scan_dirs,
        &explicit_entries,
        execution_mode,
        DiscoveryWidthPolicy::Adaptive(governor),
        DiscoveryCorpusOptions {
            node_frontier_selection,
            explicit_roster_only: false,
            exclude_substrings,
            discovery_scope_dirs,
            fast_lane_eval_budget_ms,
            wet_receipt_wall_budget_ms,
            wet_receipt_interp_eval_budget_ms,
        },
    ) {
        Ok(summary) if summary.failures.is_empty() => {
            eprintln!(
                "[measurement] discovery corpus: {} witness(es) ({} skipped, {} deferred), resolve {:.3}ms, evalu {:.3}ms, CostAccount.time basis=Measured {}ns, roster-closure {} nodes (max shard)",
                summary.total,
                summary.skipped,
                summary.deferred_rows.len(),
                summary.total_resolve_nanos as f64 / 1.0e6,
                summary.total_measured_nanos as f64 / 1.0e6,
                summary.total_measured_nanos,
                summary.roster_closure_nodes,
            );
            let st = &summary.total_stage_nanos;
            let ms = |n: u128| n as f64 / 1.0e6;
            eprintln!(
                "[resolve-split] load={:.1}ms parse={:.1}ms resolve={:.1}ms normalize={:.1}ms typecheck={:.1}ms parent_envs={:.1}ms reconcile_assembly={:.1}ms ownership={:.1}ms other={:.1}ms",
                ms(st.load),
                ms(st.parse),
                ms(st.resolve),
                ms(st.normalize),
                ms(st.typecheck_compute),
                ms(st.parent_envs),
                ms(st.reconcile_assembly),
                ms(st.ownership),
                ms(summary
                    .total_resolve_nanos
                    .saturating_sub(st.attributed_total())),
            );
            match compute_histogram_data(&summary) {
                Ok(data) => match render_timing_histogram(&source_roots, &data) {
                    Ok(histogram) => eprintln!("{histogram}"),
                    Err(e) => eprintln!("[histogram] render failed (timings unaffected): {e}"),
                },
                Err(msg) => eprintln!("{msg}"),
            }
            emit_slowest_witness_attribution(&source_roots, &summary);
            ClaimResult {
                function: format!("{label} ({} witnesses)", summary.total),
                ok: true,
                detail: String::new(),
                wall_nanos: 0,
                resolve_nanos: 0,
                corpus_resolve_nanos: summary.total_resolve_nanos,
                corpus_eval_nanos: summary.total_measured_nanos,
                corpus_witnesses: summary.total,
            }
        }
        Ok(summary) => ClaimResult {
            function: label,
            ok: false,
            detail: format!(
                "{} of {} discovery witness(es) failed: {}",
                summary.failures.len(),
                summary.total,
                summary.failures.join("; ")
            ),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: summary.total_resolve_nanos,
            corpus_eval_nanos: summary.total_measured_nanos,
            corpus_witnesses: summary.total,
        },
        Err(msg) => ClaimResult {
            function: label,
            ok: false,
            detail: format!("discovery corpus failed: {msg}"),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
        },
    }
}

fn eval_plan_in_ctx(
    plan_ctx: &InterpContext,
    plan_entry: &str,
    plan_function: &str,
) -> Result<Vec<Vec<Runnable>>, String> {
    set_phase(FloorPhase::Gate, &format!("{plan_entry}::{plan_function}"));
    let plan_value = run_value(plan_ctx, plan_function).map_err(|msg| {
        format!(
            "plan eval failed ({}::{}): {}",
            plan_entry, plan_function, msg
        )
    })?;
    let batches = batches_from_plan(&plan_value, plan_ctx)
        .map_err(|msg| format!("malformed plan value: {}", msg))?;
    Ok(batches)
}

fn eval_plan(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<Vec<Vec<Runnable>>, String> {
    let (plan_graph, plan_indices) = resolve_entry_graph(source_roots, plan_entry)
        .map_err(|msg| format!("resolve failed for plan {}:\n{}", plan_entry, msg))?;
    let plan_ctx = make_eval_context(&plan_graph, plan_indices, ExecutionMode::Hermetic);
    eval_plan_in_ctx(&plan_ctx, plan_entry, plan_function)
}

struct WalkOutcome {
    any_failed: bool,
    batches_run: usize,
}

fn peak_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

fn heartbeat_field(v: Option<u64>) -> String {
    v.map(|b| b.to_string())
        .unwrap_or_else(|| "unreadable".into())
}

/// Floor memory heartbeat — FIDELITY ONLY (reads state, changes no behavior; §5 stopped-line
/// analysis needs the line's memory story to exist in the log). The 2026-07-11 wedge was
/// invisible precisely here: `memory.high` throttles instead of killing, so the only log
/// evidence was a post-hoc `memory.peak` pinned at `high + <1MiB` after a 30-49min silent
/// tail. One synchronous regime-disclosure line at floor start (which limits bind, where),
/// then one line per minute from a detached thread (dies with the process): `memory.current`,
/// `memory.swap.current`, `memory.events` high-throttle count, PSI `some avg10`. The wedge
/// signature becomes: current pinned at high, swap climbing, high-events exploding, PSI avg10
/// double digits — attributable to a 60s window instead of a forensic reconstruction.
/// Sampling denominator = the binding-high dir when set (the slot slice that throttles), else
/// the binding-cap dir, else the leaf (whole-machine regimes); absence of all three refuses
/// loudly and the floor proceeds unmonitored — never a fabricated zero.
fn spawn_floor_memory_heartbeat() {
    let high_dir = binding_high_cgroup_dir();
    let cap_dir = binding_cap_cgroup_dir();
    let leaf = leaf_cgroup_dir();
    let describe = |label: &str, d: &Option<std::path::PathBuf>, file: &str| match d {
        Some(dir) => format!(
            "{label}={} ({})",
            read_cgroup_raw(dir, file).unwrap_or_else(|| "unreadable".into()),
            dir.display()
        ),
        None => format!("{label}=none"),
    };
    eprintln!(
        "[floor-memory] regime: {}; {}",
        describe("memory.high", &high_dir, "memory.high"),
        describe("memory.max", &cap_dir, "memory.max"),
    );
    let Some(dir) = high_dir.or(cap_dir).or(leaf) else {
        eprintln!(
            "[floor-memory] heartbeat unavailable: no readable cgroup (refusing to fabricate; floor proceeds unmonitored)"
        );
        return;
    };
    let spawned = std::thread::Builder::new()
        .name("floor-memory-heartbeat".into())
        .spawn(move || {
            let mut minute: u64 = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                minute += 1;
                let psi = read_cgroup_raw(&dir, "memory.pressure")
                    .and_then(|c| memory_pressure_some_avg10(&c))
                    .unwrap_or_else(|| "unreadable".into());
                let high_events = read_cgroup_raw(&dir, "memory.events")
                    .and_then(|c| memory_events_field(&c, "high"));
                eprintln!(
                    "[floor-memory] t={minute}m current={} swap={} high_events={} psi_some_avg10={psi}",
                    heartbeat_field(read_cgroup_u64(&dir, "memory.current")),
                    heartbeat_field(read_cgroup_u64(&dir, "memory.swap.current")),
                    heartbeat_field(high_events),
                );
            }
        });
    if let Err(e) = spawned {
        eprintln!(
            "[floor-memory] heartbeat thread failed to spawn: {e} (floor proceeds unmonitored)"
        );
    }
}

/// One read of the whole-tree job footprint and its budget context, taken in a single cgroup walk so
/// a consumer (placement divisor, compile-jobs derive) can parse usage AND budget from one emitted
/// line without re-walking. `cap_bytes` is the tightest numeric `memory.max` on the leaf→root walk
/// (`None` = uncapped / RAM-bound); `host_ram` is the physical-RAM budget that binds when uncapped;
/// `sccache_under_leaf` classifies the sccache server's cgroup as a descendant (already counted in
/// `leaf_peak`) vs a sibling (must be subtracted as `host_fixed_overhead`) — the "accounted exactly
/// once" decision, read from paths rather than guessed.
struct CgroupMeasurement {
    leaf_peak: u64,
    leaf_rel: String,
    cap_bytes: Option<u64>,
    host_ram: Option<u64>,
    pids_current: u64,
    pids_max: String,
    sccache_rel: Option<String>,
    sccache_under_leaf: bool,
}

fn cgroup_job_measurement() -> Option<CgroupMeasurement> {
    let leaf = leaf_cgroup_dir()?;
    let leaf_peak = fs::read_to_string(leaf.join("memory.peak"))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    let leaf_rel = leaf
        .strip_prefix("/sys/fs/cgroup")
        .map(|p| format!("/{}", p.to_string_lossy().trim_start_matches('/')))
        .unwrap_or_else(|_| leaf.to_string_lossy().into_owned());
    let cap_bytes = binding_cap_cgroup_dir()
        .and_then(|d| fs::read_to_string(d.join("memory.max")).ok())
        .and_then(|s| s.trim().parse::<u64>().ok());
    let pids_current = fs::read_to_string(leaf.join("pids.current"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let pids_max = fs::read_to_string(leaf.join("pids.max"))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let sccache_rel = sccache_server_cgroup_rel();
    // Descendant iff sccache's cgroup is the leaf or strictly under it — a PATH-COMPONENT prefix,
    // not a bare string prefix, so a sibling like `<leaf>-other.service` is NOT misclassified as a
    // descendant (that would under-count host overhead — the fail-OPEN direction). When the leaf is
    // the cgroup root (`leaf_r` empty — e.g. a single-cgroup container) everything is a descendant.
    // On the real fleet the leaf is the runner-service cgroup and sccache is a sibling service
    // cgroup, so this is `false` (subtract as host_fixed_overhead).
    let leaf_r = leaf_rel.trim_start_matches('/').to_string();
    let sccache_under_leaf = sccache_rel
        .as_deref()
        .map(|r| {
            let r = r.trim_start_matches('/');
            leaf_r.is_empty() || r == leaf_r || r.starts_with(&format!("{leaf_r}/"))
        })
        .unwrap_or(false);
    Some(CgroupMeasurement {
        leaf_peak,
        leaf_rel,
        cap_bytes,
        host_ram: mem_total_bytes(),
        pids_current,
        pids_max,
        sccache_rel,
        sccache_under_leaf,
    })
}

/// Verify each declared release artifact exists, is executable, and is non-empty — failing CLOSED
/// (exit 1) with the GitHub-Actions `::error::` annotation on the first violation. This is the
/// in-binary home of what was previously inline `[ -x ]` / `[ -s ]` shell in the generated ci.yml
/// (DESIGN §5 fail-open guard: an sccache-served truncated/empty cached artifact after a
/// `successful` build). The artifact paths are authored by the .dag spec and passed positionally.
fn verify_build_artifacts(paths: &[String]) -> Result<ExitCode, ExitCode> {
    use std::os::unix::fs::PermissionsExt;
    if paths.is_empty() {
        eprintln!("claim_executor: --verify-build-artifacts requires at least one artifact path");
        return Err(ExitCode::from(2));
    }
    for path in paths {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path.as_str());
        match std::fs::metadata(path) {
            Ok(meta) => {
                let executable = meta.permissions().mode() & 0o111 != 0;
                if !meta.is_file() || !executable {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' absent or not \
                         executable after a 'successful' build (sccache/cache corruption — DESIGN \
                         §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
                if meta.len() == 0 {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' is zero-byte after \
                         a 'successful' build (sccache served a truncated/empty cached artifact — \
                         DESIGN §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
            }
            Err(_) => {
                eprintln!(
                    "::error::build verification: declared artifact '{name}' absent or not \
                     executable after a 'successful' build (sccache/cache corruption — DESIGN §5 \
                     fail-open); failing closed: {path}"
                );
                return Err(ExitCode::from(1));
            }
        }
    }
    eprintln!(
        "claim_executor: build-artifact verification passed ({} declared release binar{} present \
         + non-empty)",
        paths.len(),
        if paths.len() == 1 { "y" } else { "ies" }
    );
    Ok(ExitCode::SUCCESS)
}

/// Single authority for the one-line whole-tree cgroup measurement, shared by the floor run and the
/// standalone `--measure-cgroup-peak` mode so the `ci` and `rust_tests` jobs report an
/// identically-shaped line. `context` distinguishes the call site.
fn emit_cgroup_measurement(context: &str) {
    match cgroup_job_measurement() {
        Some(m) => {
            let cap = match m.cap_bytes {
                Some(b) => format!("{b} bytes"),
                None => "uncapped(RAM-bound)".to_string(),
            };
            let host_ram = m
                .host_ram
                .map(|b| format!("{b} bytes"))
                .unwrap_or_else(|| "unknown".to_string());
            let sccache = match (&m.sccache_rel, m.sccache_under_leaf) {
                (Some(r), true) => format!("{r} (descendant: counted in memory.peak)"),
                (Some(r), false) => format!("{r} (sibling: subtract as host_fixed_overhead)"),
                (None, _) => "not-found (treat as fixed host overhead)".to_string(),
            };
            eprintln!(
                "[measurement] cgroup peak: {peak} bytes (memory.peak @ {rel}) memory.max={cap} host_ram={host_ram} pids.current={pc} pids.max={pm} sccache-server-cgroup={sccache} context={context}",
                peak = m.leaf_peak,
                rel = m.leaf_rel,
                pc = m.pids_current,
                pm = m.pids_max
            );
        }
        None => eprintln!(
            "[measurement] cgroup peak: unavailable (no leaf cgroup or memory.peak unreadable; kernel < 5.19?) context={context}"
        ),
    }
}

/// Best-effort cgroup-v2 relative path of the sccache SERVER daemon (a `sccache` comm), scanned
/// from `/proc`. Emitted beside the binding-cap ancestor path so the analysis can classify the
/// "accounted exactly once" case: a path UNDER the ancestor → sccache is inside `memory.peak`
/// (don't subtract); a SIBLING path → subtract it as `host_fixed_overhead`. Returns `None` when
/// no sccache server process is found (then it's fixed host overhead by default, fail-closed).
fn sccache_server_cgroup_rel() -> Option<String> {
    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let p = entry.path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        if fs::read_to_string(p.join("comm"))
            .unwrap_or_default()
            .trim()
            != "sccache"
        {
            continue;
        }
        if let Some(rel) = fs::read_to_string(p.join("cgroup")).ok().and_then(|cg| {
            cg.lines()
                .find_map(|l| l.strip_prefix("0::"))
                .map(|s| s.trim().to_string())
        }) {
            return Some(rel);
        }
    }
    None
}

/// Per-batch timing record collected during run_walk for Gantt emission.
struct BatchRecord {
    batch_index: usize,
    wall_nanos: u128,
    /// Flattened results from all units in this batch (order: unit by unit).
    results: Vec<ClaimResult>,
}

/// Materialization-ladder receipt: how many entry resolves this floor run actually
/// paid (walk_memo hits charge resolve_nanos == 0 and are excluded — this counts the
/// duplicated work across DISTINCT entries, the cross-entry share the memo cannot do).
/// Discovery corpus resolve time is reported on its own key, never folded into the
/// entry count (different grain; conflating them would mask either regression).
/// Consumed by the ci.yml resolve-receipt gate emitted from dag/gunbc/ci_materialization.dag.
/// Returns false on a write error — the walk fails closed at the point of
/// failure rather than relying only on the downstream missing-file gate.
fn write_resolve_receipt(batch_records: &[BatchRecord]) -> bool {
    let mut resolves_total: u64 = 0;
    let mut resolve_ms_total: u128 = 0;
    let mut discovery_corpus_resolve_ms: u128 = 0;
    for rec in batch_records {
        for result in &rec.results {
            if result.resolve_nanos > 0 {
                resolves_total += 1;
                resolve_ms_total += result.resolve_nanos / 1_000_000;
            }
            discovery_corpus_resolve_ms += result.corpus_resolve_nanos / 1_000_000;
        }
    }
    let body = format!(
        "resolves_total={resolves_total}\nresolve_ms_total={resolve_ms_total}\ndiscovery_corpus_resolve_ms={discovery_corpus_resolve_ms}\n"
    );
    let path = std::path::Path::new("target/floor-resolve-receipt.txt");
    if let Err(e) = std::fs::create_dir_all("target").and_then(|_| std::fs::write(path, &body)) {
        eprintln!(
            "claim_executor: failed to write resolve receipt {}: {e} — walk fails closed here (and the gate downstream fails closed on the missing file)",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor resolves: {resolves_total} entry resolve(s), {resolve_ms_total}ms (receipt: {})",
        path.display()
    );
    true
}

/// The materialization demand receipt at the eval-frame grain: process-wide
/// ledger totals accumulated by every InterpContext on Drop (threads included),
/// written once at walk end. Determinism, as measured (2026-07-10, 5 receipts):
/// unkeyed_calls is corpus-deterministic (identical across schedules, machines,
/// and debug/release); keyed/distinct/duplicated jitter a few counts because
/// they sum PER-CTX numbers and witness→ctx grouping is a thread-pool accident
/// — so counts disclose, they do not pin, until the frame grain is structural.
/// wasted_ms lines are observational and must never gate. The derived ci.yml
/// gate fails closed on a missing/malformed file or zeroed keyed_calls (a
/// floor that evaluated nothing is a lie, so disabling the trace cannot
/// silently green the gate). Returns false on a write error — the walk fails
/// closed here, not only at the downstream missing-file gate.
fn write_materialization_receipt() -> bool {
    let t = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
    let body = format!(
        "keyed_calls={}\nunkeyed_calls={}\noverflow_calls={}\ndistinct_keys={}\nduplicated_keys={}\nsingle_site_keys={}\nmulti_site_keys={}\nwasted_ms_total={}\nwasted_ms_single_site={}\nwasted_ms_multi_site={}\nmemo_hits={}\nmemo_misses={}\nmemo_overflow={}\n",
        t.keyed_calls,
        t.unkeyed_calls,
        t.overflow_calls,
        t.distinct_keys,
        t.duplicated_keys,
        t.single_site_keys,
        t.multi_site_keys,
        t.wasted_ns_total / 1_000_000,
        t.wasted_ns_single_site / 1_000_000,
        t.wasted_ns_multi_site / 1_000_000,
        t.memo_hits,
        t.memo_misses,
        t.memo_overflow
    );
    let path = std::path::Path::new("target/floor-materialization-receipt.txt");
    if let Err(e) = std::fs::create_dir_all("target").and_then(|_| std::fs::write(path, &body)) {
        eprintln!(
            "claim_executor: failed to write materialization receipt {}: {e} — walk fails closed here (and the gate downstream fails closed on the missing file)",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor materialization: keyed_calls={} unkeyed_calls={} duplicated_keys={} (single_site={} multi_site={}) wasted_ms={} memo_hits={} memo_misses={} (receipt: {})",
        t.keyed_calls,
        t.unkeyed_calls,
        t.duplicated_keys,
        t.single_site_keys,
        t.multi_site_keys,
        t.wasted_ns_total / 1_000_000,
        t.memo_hits,
        t.memo_misses,
        path.display()
    );
    true
}

/// Emit a fractal Gantt tree to stderr when GUNBC_FLOOR_GANTT=1.
fn emit_gantt(batch_records: &[BatchRecord], total_wall_nanos: u128) {
    let gantt_enabled = std::env::var("GUNBC_FLOOR_GANTT")
        .map(|v| v == "1")
        .unwrap_or(false);
    if !gantt_enabled {
        return;
    }
    let total_ms = total_wall_nanos / 1_000_000;
    eprintln!("[gantt] claim_executor wall: {}ms", total_ms);
    for rec in batch_records {
        let batch_ms = rec.wall_nanos / 1_000_000;
        let pct = if total_ms == 0 {
            0.0
        } else {
            100.0 * batch_ms as f64 / total_ms as f64
        };
        eprintln!(
            "[gantt]   batch {} wall: {}ms ({:.1}%)",
            rec.batch_index + 1,
            batch_ms,
            pct,
        );
        for result in &rec.results {
            let batch_pct = |ns: u128| -> f64 {
                if rec.wall_nanos == 0 {
                    0.0
                } else {
                    100.0 * ns as f64 / rec.wall_nanos as f64
                }
            };
            if result.corpus_witnesses > 0 {
                // Discovery batch: show serial-sum breakdown.
                let corpus_resolve_ms = result.corpus_resolve_nanos / 1_000_000;
                let corpus_eval_ms = result.corpus_eval_nanos / 1_000_000;
                eprintln!(
                    "[gantt]     {} ({} witnesses)",
                    result.function, result.corpus_witnesses
                );
                eprintln!(
                    "[gantt]       resolve (serial sum): {}ms  ({:.1}% of batch wall)",
                    corpus_resolve_ms,
                    batch_pct(result.corpus_resolve_nanos),
                );
                eprintln!(
                    "[gantt]       eval    (serial sum): {}ms  ({:.1}% of batch wall)",
                    corpus_eval_ms,
                    batch_pct(result.corpus_eval_nanos),
                );
            } else {
                // Single claim: show resolve (if charged) + eval.
                if result.resolve_nanos > 0 {
                    let resolve_ms = result.resolve_nanos / 1_000_000;
                    eprintln!(
                        "[gantt]     resolve (entry): {}ms  ({:.1}% of batch wall)",
                        resolve_ms,
                        batch_pct(result.resolve_nanos),
                    );
                }
                let wall_ms = result.wall_nanos / 1_000_000;
                let ok = if result.ok { "PASS" } else { "FAIL" };
                eprintln!(
                    "[gantt]     {}: {}ms  [{ok}]  ({:.1}% of batch wall)",
                    result.function,
                    wall_ms,
                    batch_pct(result.wall_nanos),
                );
            }
        }
    }
}

fn run_walk(
    source_roots: &[String],
    batches: &[Vec<Runnable>],
    governor: &Arc<MemoryGovernor>,
    fast_lane_eval_budget_ms: Option<u64>,
    falsifier_self_host_wet_budgets: FalsifierSelfHostWetBudgets,
    stop_policy: FloorBatchStopPolicy,
) -> WalkOutcome {
    let mut any_failed = false;
    let mut batches_run = 0usize;
    let walk_start = Instant::now();
    let mut batch_records: Vec<BatchRecord> = Vec::new();
    // Cross-batch resolve memo: SharedClaims whose runnable does a heavy whole-tree resolve
    // run on the main thread and share a single resolved InterpContext per entry across all batches.
    // Rc<ResolvedGraph> is !Send so these units cannot run on spawned threads; they
    // run sequentially here after the spawned (non-memo) threads in each batch are joined.
    // Key invariant: source_roots is constant for the lifetime of a run_walk call, so
    // keying the memo by (entry, execution_mode) is sufficient — a given entry always
    // resolves against the same source_roots here. If this function ever accepts multiple
    // source_root sets, the key must grow a source_roots_hash component.
    let mut walk_memo: std::collections::HashMap<(String, ExecutionMode), InterpContext> =
        std::collections::HashMap::new();
    for (bi, batch) in batches.iter().enumerate() {
        batches_run = bi + 1;
        let units = group_batch_units(batch);
        eprintln!(
            "claim_executor: batch {} — {} node(s) in {} resolve-group(s), governor target_width={}",
            bi + 1,
            batch.len(),
            units.len(),
            governor.current_target_width()
        );
        let batch_start = Instant::now();
        // Partition units: memo units stay on the main thread; others spawn.
        // A unit goes to the memo path if (a) its profile declares heavy whole-tree
        // resolve, or (b) its entry is already in walk_memo from a prior batch — in
        // which case re-resolving would be redundant regardless of profile.
        let mut memo_units: Vec<BatchUnit> = Vec::new();
        let mut main_thread_units: Vec<BatchUnit> = Vec::new();
        let mut thread_units: Vec<BatchUnit> = Vec::new();
        for unit in units {
            match &unit {
                BatchUnit::SharedClaims {
                    use_walk_memo: true,
                    ..
                } => memo_units.push(unit),
                BatchUnit::SharedClaims {
                    entry,
                    execution_mode,
                    ..
                } if walk_memo.contains_key(&(entry.clone(), *execution_mode)) => {
                    memo_units.push(unit)
                }
                // Discovery pumps run on the MAIN thread: `process_shared_index` is
                // thread-local (Rc-based, !Send), and the eagerly-installed compile-clean
                // receipt (see the install before run_walk) warmed THIS thread's index —
                // routing the pump here is what lets batch-2's witness resolves read the
                // gate's content-keyed typed store instead of re-typechecking the tree
                // (lever 1, PR #6766). No wall-clock loss: the main thread previously
                // idled at the join while the one Discovery unit ran on a spawned thread.
                BatchUnit::Discovery { .. } => main_thread_units.push(unit),
                _ => thread_units.push(unit),
            }
        }
        // Bracket the parallel walk in a host-effect group: the `[file]`/`[rest]`/
        // `[shell]` trace lines stream to stderr from the worker threads INSIDE the
        // group, while the scannable PASS/FAIL summary is deferred to AFTER the group
        // closes (below) so it stays outside the collapsed section. GitHub Actions
        // renders this as a collapsible `::group::`; a plain terminal as a header.
        // Threads can't interleave group markers (one open/close on the main thread
        // spans the whole batch), so it is sound under parallel unit threads.
        let grouped = v1_compiler::v1_interpreter::host_trace_grouping_active();
        if grouped {
            set_phase(
                FloorPhase::HostEffect,
                &format!("batch-{}-host-effects", bi + 1),
            );
            v1_compiler::v1_interpreter::group_begin(&format!("batch {} host-effects", bi + 1));
        }
        let handles: Vec<_> = thread_units
            .into_iter()
            .map(|unit| {
                let roots = source_roots.to_vec();
                let unit_governor = governor.clone();
                let wet_budgets = falsifier_self_host_wet_budgets;
                thread::spawn(move || {
                    run_batch_unit(
                        roots,
                        unit,
                        unit_governor,
                        fast_lane_eval_budget_ms,
                        wet_budgets,
                    )
                })
            })
            .collect();
        // Run memo units on the main thread while spawned threads are working.
        let mut memo_results: Vec<ClaimResult> = Vec::new();
        for unit in memo_units {
            if let BatchUnit::SharedClaims {
                entry,
                functions,
                execution_mode,
                ..
            } = unit
            {
                let results = run_memo_shared_claims(
                    source_roots,
                    &entry,
                    &functions,
                    execution_mode,
                    &mut walk_memo,
                );
                memo_results.extend(results);
            }
        }
        // Discovery pumps on the main thread (see the partition above): the pump's
        // `process_shared_index` is this thread's — the one the eager compile-clean
        // receipt install warmed — so the corpus reads the gate's typed store.
        for unit in main_thread_units {
            memo_results.extend(run_batch_unit(
                source_roots.to_vec(),
                unit,
                governor.clone(),
                fast_lane_eval_budget_ms,
                falsifier_self_host_wet_budgets,
            ));
        }
        // Collect all results before printing any PASS/FAIL — the prints must land
        // after `group_end`, and a thread panic still has to close the group.
        let mut batch_results: Vec<ClaimResult> = memo_results;
        let mut thread_panicked = false;
        for handle in handles {
            match handle.join() {
                Ok(results) => batch_results.extend(results),
                Err(_) => thread_panicked = true,
            }
        }
        if grouped {
            v1_compiler::v1_interpreter::group_end();
        }
        for result in &batch_results {
            if result.ok {
                println!(
                    "{}",
                    paint(
                        &format!("✓ PASS [batch {}] {}", bi + 1, result.function),
                        sgr::SUCCESS
                    )
                );
            } else {
                println!(
                    "{}",
                    paint(
                        &format!(
                            "✗ FAIL [batch {}] {} ({})",
                            bi + 1,
                            result.function,
                            result.detail
                        ),
                        sgr::ERROR
                    )
                );
                any_failed = true;
            }
        }
        if thread_panicked {
            println!(
                "{}",
                paint(
                    &format!("✗ FAIL [batch {}] <claim thread panicked>", bi + 1),
                    sgr::ERROR
                )
            );
            any_failed = true;
        }
        let batch_wall_nanos = batch_start.elapsed().as_nanos();
        batch_records.push(BatchRecord {
            batch_index: bi,
            wall_nanos: batch_wall_nanos,
            results: batch_results,
        });
        if any_failed {
            match stop_policy {
                FloorBatchStopPolicy::StopBeforeDependents => {
                    eprintln!(
                        "claim_executor: batch {} had failures — stopping before dependent batches",
                        bi + 1
                    );
                    break;
                }
                FloorBatchStopPolicy::FullLedger => {
                    eprintln!(
                        "claim_executor: batch {} had failures — continuing (FullLedger stop policy)",
                        bi + 1
                    );
                }
            }
        }
    }
    let total_wall_nanos = walk_start.elapsed().as_nanos();
    emit_gantt(&batch_records, total_wall_nanos);
    let resolve_receipt_ok = write_resolve_receipt(&batch_records);
    // Memo contexts absorb their ledger totals into the process accumulator on
    // Drop, so they must die before the materialization receipt is written.
    drop(walk_memo);
    let materialization_receipt_ok = write_materialization_receipt();
    WalkOutcome {
        any_failed: any_failed || !resolve_receipt_ok || !materialization_receipt_ok,
        batches_run,
    }
}

fn remap_entry_for_temp(source_root: &str, temp_src: &Path, entry: &str) -> PathBuf {
    let prefix = format!("{source_root}/");
    if let Some(suffix) = entry.strip_prefix(&prefix) {
        temp_src.join(suffix)
    } else if let Some(suffix) = entry.strip_prefix("src/v2/") {
        temp_src.join(suffix)
    } else {
        PathBuf::from(entry)
    }
}

fn copy_dir_all(from: &Path, to: &Path) -> Result<(), String> {
    if !from.is_dir() {
        return Err(format!("{} is not a directory", from.display()));
    }
    fs::create_dir_all(to).map_err(|e| format!("mkdir {}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("read_dir {}: {e}", from.display()))? {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let ft = entry.file_type().map_err(|e| format!("file_type: {e}"))?;
        let dest = to.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest).map_err(|e| {
                format!("copy {} -> {}: {e}", entry.path().display(), dest.display())
            })?;
        }
    }
    Ok(())
}

fn perturb_function_to_false(path: &Path, function: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let needle_fn = format!("fn {function}(");
    let needle_func = format!("func {function}(");
    let start = match (text.find(&needle_func), text.find(&needle_fn)) {
        (Some(f), _) => f,
        (None, Some(f)) => f,
        (None, None) => return Err(format!("{}: missing function {function}", path.display())),
    };
    let brace = start
        + text[start..]
            .find('{')
            .ok_or_else(|| format!("{}: missing body for {function}", path.display()))?;
    let mut depth = 0;
    let mut end = None;
    for (i, ch) in text[brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(brace + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.ok_or_else(|| format!("{}: unterminated body for {function}", path.display()))?;
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..brace]);
    out.push_str("{\n  false\n}");
    out.push_str(&text[end..]);
    fs::write(path, out).map_err(|e| format!("write {}: {e}", path.display()))
}

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority:
/// `v2.workflow.ci_floor_plan.gunbc_floor_arm_time_budget_refusal_plan_functions_claim_executor_seed`,
/// symbols `gunbc.ci_spec.floor_plan_function` / `plan_artifact_plan_function` /
/// `gunbc.falsifier_workflow.falsifier_plan_function`):
/// plan functions whose schedule is a floor walk and therefore subject to arm-time
/// `FloorBudgetBelowMinimumFootprint` refusal in claim_executor.
const FLOOR_ARM_TIME_BUDGET_REFUSAL_PLAN_FUNCTIONS: &[&str] = &[
    "gunbc_ci_floor_batches",
    "gunbc_ci_plan_artifact_batches",
    "gunbc_falsifier_batches",
];

fn plan_requires_floor_arm_time_budget_refusal(plan_function: &str) -> bool {
    FLOOR_ARM_TIME_BUDGET_REFUSAL_PLAN_FUNCTIONS.contains(&plan_function)
}

fn run_perturb_check(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<ExitCode, ExitCode> {
    let batches = match eval_plan(source_roots, plan_entry, plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: --perturb-check: {msg}");
            return Err(ExitCode::from(2));
        }
    };
    if batches.len() < 2 {
        eprintln!(
            "claim_executor: --perturb-check needs a plan with >= 2 batches to witness the \
             walk halt (got {})",
            batches.len()
        );
        return Err(ExitCode::from(2));
    }
    let (gating_entry, gating_function) = match batches[0].first() {
        Some(Runnable::SingleClaim {
            entry, function, ..
        }) if !entry.is_empty() => (entry.clone(), function.clone()),
        _ => {
            eprintln!(
                "claim_executor: --perturb-check: batch 1 has no plantable SingleClaim gating node"
            );
            return Err(ExitCode::from(2));
        }
    };

    let primary = &source_roots[0];
    let tmp = std::env::temp_dir().join(format!("claim-executor-perturb-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let temp_src = tmp.join("src");
    if let Err(e) = copy_dir_all(Path::new(primary), &temp_src) {
        eprintln!("claim_executor: --perturb-check: {e}");
        return Err(ExitCode::from(2));
    }

    let gating_path = remap_entry_for_temp(primary, &temp_src, &gating_entry);
    if let Err(e) = perturb_function_to_false(&gating_path, &gating_function) {
        let _ = fs::remove_dir_all(&tmp);
        eprintln!("claim_executor: --perturb-check: plant gating->false failed: {e}");
        return Err(ExitCode::from(2));
    }

    let temp_root = temp_src.to_string_lossy().into_owned();
    let remap_root = |root: &str| -> String {
        if root == primary.as_str() {
            temp_root.clone()
        } else {
            root.to_string()
        }
    };
    let remapped: Vec<Vec<Runnable>> = batches
        .iter()
        .map(|batch| {
            batch
                .iter()
                .map(|r| match r {
                    Runnable::SingleClaim {
                        entry,
                        function,
                        use_walk_memo,
                        execution_mode,
                    } => Runnable::SingleClaim {
                        entry: if entry.is_empty() {
                            entry.clone()
                        } else {
                            remap_entry_for_temp(primary, &temp_src, entry)
                                .to_string_lossy()
                                .into_owned()
                        },
                        function: function.clone(),
                        use_walk_memo: *use_walk_memo,
                        execution_mode: *execution_mode,
                    },
                    Runnable::DiscoveryBatch {
                        source_roots: roots,
                        scan_dirs,
                        explicit_entries,
                        node_frontier_selection,
                        exclude_substrings,
                        discovery_scope_dirs,
                        execution_mode,
                    } => Runnable::DiscoveryBatch {
                        source_roots: roots.iter().map(|r| remap_root(r)).collect(),
                        scan_dirs: scan_dirs.iter().map(|d| remap_root(d)).collect(),
                        explicit_entries: explicit_entries.clone(),
                        node_frontier_selection: *node_frontier_selection,
                        exclude_substrings: exclude_substrings.clone(),
                        discovery_scope_dirs: discovery_scope_dirs.clone(),
                        execution_mode: *execution_mode,
                    },
                })
                .collect()
        })
        .collect();

    eprintln!(
        "claim_executor: --perturb-check: planted batch-1 gating witness `{}` -> false; re-walking",
        gating_function
    );
    // The perturb re-walk is a small diagnostics pass: a max_width=1 governor keeps it
    // serial, matching the prior fixed width-1 semantics.
    let outcome = run_walk(
        &[temp_root],
        &remapped,
        &Arc::new(MemoryGovernor::from_environment(1)),
        None,
        FalsifierSelfHostWetBudgets::default(),
        FloorBatchStopPolicy::StopBeforeDependents,
    );
    let _ = fs::remove_dir_all(&tmp);

    if outcome.any_failed && outcome.batches_run == 1 {
        eprintln!(
            "claim_executor: --perturb-check OK: gating batch-1 false -> run failed closed AND \
             walk halted before batch 2 (batches_run=1 of {})",
            batches.len()
        );
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!(
            "claim_executor: --perturb-check FAIL: expected fail-closed + halt-at-batch-1, got \
             any_failed={} batches_run={} (of {})",
            outcome.any_failed,
            outcome.batches_run,
            batches.len()
        );
        Ok(ExitCode::from(1))
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut plan_entry: Option<String> = None;
    let mut plan_function = "bre_claim_batches".to_string();
    let mut notice_title: Option<String> = None;
    let mut perturb_check = false;
    let mut measure_cgroup_peak = false;
    let mut verify_artifacts: Vec<String> = Vec::new();
    let mut verify_artifacts_mode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--verify-build-artifacts" => {
                // All remaining positional args are declared release-artifact paths to verify.
                verify_artifacts_mode = true;
                i += 1;
                while i < args.len() {
                    verify_artifacts.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            "--source-root" => {
                i += 1;
                source_roots.push(require_path_value(&args, i, "--source-root")?);
            }
            "--plan-entry" => {
                i += 1;
                plan_entry = Some(require_path_value(&args, i, "--plan-entry")?);
            }
            "--plan-function" => {
                i += 1;
                plan_function = require_value(&args, i, "--plan-function")?;
            }
            "--notice-title" => {
                i += 1;
                notice_title = Some(require_value(&args, i, "--notice-title")?);
            }
            "--perturb-check" => perturb_check = true,
            "--measure-cgroup-peak" => measure_cgroup_peak = true,
            other => {
                eprintln!("claim_executor: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    // Build-artifact verification (no plan run): the floor's bootstrap `cargo build` is followed by
    // this check so a `successful` build that nonetheless produced a missing/zero-byte binary
    // (sccache serving a truncated/empty cached artifact — DESIGN §5 fail-open) fails CLOSED before
    // the floor runs. The LOGIC lives here (the floor binary) instead of inline shell in ci.yml; the
    // declared artifact paths are still authored by the .dag spec and passed as positional args.
    // Short-circuits before the plan-arg requirements so it needs no `--plan-entry`.
    if verify_artifacts_mode {
        return verify_build_artifacts(&verify_artifacts);
    }

    // Standalone whole-tree cgroup measurement (no plan run): the `rust_tests` job invokes this
    // after its gate to emit ITS leaf cgroup peak (a separate ephemeral runner cgroup from the `ci`
    // job's), reusing the same single-authority walk/emit. Short-circuits before the plan-arg
    // requirements so it needs no `--source-root`/`--plan-entry`.
    if measure_cgroup_peak {
        let job = std::env::var("GITHUB_JOB").unwrap_or_else(|_| "standalone".to_string());
        emit_cgroup_measurement(&format!("job={job} (--measure-cgroup-peak)"));
        return Ok(ExitCode::SUCCESS);
    }

    if source_roots.is_empty() {
        eprintln!("claim_executor: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let _phase_profile = PhaseProfile::install_from_env();
    let plan_entry = match plan_entry {
        Some(e) => e,
        None => {
            eprintln!("claim_executor: --plan-entry <file.dag> is required");
            return Err(ExitCode::from(2));
        }
    };

    // Coarse wall-clock phase marks: the pre-walk phases (hygiene walk, policy
    // install, plan resolve, plan eval, width eval) are interpreter-heavy and used
    // to be SILENT — a 30-minute prelude looked identical to a hang. Every phase
    // now stamps a line so the floor log itemizes its own time.
    let floor_started = Instant::now();
    let phase_mark = |label: &str| {
        eprintln!(
            "claim_executor: [t+{:.1}s] {label}",
            floor_started.elapsed().as_secs_f64()
        );
    };

    // Under the opt-in inversion the plan's DiscoveryBatches carry explicit entries
    // only (or are absent entirely on an empty roster), and the explicit-only path
    // skips the tree-walk naming hygiene (`test fn` outside `*_test.dag`, `__`
    // basenames) that glob discovery used to run. A witness must stay NAMEABLE even
    // when not enrolled (an unnameable witness could never be opted in), so the plan
    // path always runs the fail-closed walk once up front — before the (expensive)
    // plan evaluation, so a naming violation is the cheapest possible failure.
    {
        let excludes = v1_compiler::cli_run::witness_exclusion_substrings();
        if let Err(msg) = v1_compiler::cli_run::check_floor_filename_hygiene(&source_roots)
            .and_then(|_| {
                v1_compiler::cli_run::discover_floor_corpus_rows(&source_roots, &[], &excludes)
                    .map(|_| ())
            })
        {
            eprintln!("claim_executor: witness naming hygiene (pre-plan walk): {msg}");
            return Err(ExitCode::from(1));
        }
    }
    phase_mark("naming-hygiene walk complete");

    // Install the host-effect trace policy from the .dag authority once, before
    // discovery threads spawn, so `[file] read` / `[rest]` / `[hermetic:mock]` etc.
    // are funnelled per `gunbc.output_policy` instead of flooding the floor log.
    v1_compiler::cli_run::install_output_policy(&source_roots);
    // Install the per-target group-marker syntax (GitHub Actions `::group::` vs a
    // plain-terminal header) from the .dag authority, so the parallel walk folds each
    // batch's host-effect traces into a collapsible group.
    v1_compiler::cli_run::install_group_syntax(&source_roots);
    phase_mark("output policy + group syntax installed");

    if perturb_check {
        return run_perturb_check(&source_roots, &plan_entry, &plan_function);
    }

    // Resolve the plan entry ONCE and evaluate both the batches (hermetic) and the
    // spawn width (wet) from the same resolved graph — this resolve was previously
    // paid twice back-to-back (the §2 double-paid-compute trap, at minutes each).
    let (plan_graph, plan_indices) = match resolve_entry_graph_shared(&source_roots, &plan_entry) {
        Ok(resolved) => resolved,
        Err(msg) => {
            eprintln!("claim_executor: resolve failed for plan {plan_entry}:\n{msg}");
            return Err(ExitCode::from(1));
        }
    };
    phase_mark("plan entry resolved");

    let plan_ctx = make_eval_context(&plan_graph, plan_indices.clone(), ExecutionMode::Hermetic);
    let batches = match eval_plan_in_ctx(&plan_ctx, &plan_entry, &plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    };
    // Fast-lane 5s rule (operator 2026-07-12): a plan that schedules a discovery batch
    // must declare the per-witness eval budget; a missing/mistyped row refuses the run
    // (fail-closed), while discovery-free plans (regen, plan-artifact) never read it.
    let schedules_discovery = batches
        .iter()
        .flatten()
        .any(|r| matches!(r, Runnable::DiscoveryBatch { .. }));
    let fast_lane_eval_budget_ms: Option<u64> = if schedules_discovery {
        match run_value(&plan_ctx, "gunbc_ci_fast_lane_eval_budget_ms") {
            Ok(Value::Int(n)) if n > 0 => Some(n as u64),
            Ok(other) => {
                eprintln!(
                    "claim_executor: gunbc_ci_fast_lane_eval_budget_ms must be a positive Int, got {other:?} (fail-closed)"
                );
                return Err(ExitCode::from(1));
            }
            Err(msg) => {
                eprintln!(
                    "claim_executor: plan schedules a discovery batch but gunbc_ci_fast_lane_eval_budget_ms is unavailable (fail-closed): {msg}"
                );
                return Err(ExitCode::from(1));
            }
        }
    } else {
        None
    };
    let falsifier_self_host_wet_budgets = if plan_function == "gunbc_falsifier_batches" {
        FalsifierSelfHostWetBudgets {
            wall_budget_ms: match read_positive_budget_ms(
                &plan_ctx,
                "gunbc_falsifier_self_host_wet_receipt_wall_budget_ms",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            interp_eval_budget_ms: match read_positive_budget_ms(
                &plan_ctx,
                "gunbc_falsifier_self_host_wet_interp_eval_budget_ms",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
        }
    } else {
        FalsifierSelfHostWetBudgets::default()
    };
    let batch_stop_policy = resolve_floor_batch_stop_policy(&plan_ctx, &plan_function);
    drop(plan_ctx);
    phase_mark("plan evaluated");

    eprintln!(
        "claim_executor: [{}] executor plan = {} batch(es) from {}::{}",
        notice_title.as_deref().unwrap_or("ci floor"),
        batches.len(),
        plan_entry,
        plan_function
    );

    if batches.is_empty() {
        eprintln!("claim_executor: executor plan produced 0 batches — failing closed");
        return Err(ExitCode::from(1));
    }

    drop(plan_graph);
    // Adaptive width: no plan-evaluated spawn width and no pinned per-shard constants —
    // the governor admits workers against the slot's own declared budget (AIMD), so the
    // width story for the run is its announce line here plus its end-of-run receipt.
    let governor = Arc::new(MemoryGovernor::from_environment(
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    ));
    if plan_requires_floor_arm_time_budget_refusal(&plan_function) {
        if let Some(msg) = floor_budget_below_minimum_footprint(governor.budget_bytes()) {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    }
    phase_mark("memory governor armed; starting batch walk");
    spawn_floor_memory_heartbeat();

    // Plans whose schedule carries the compile-clean gate node: the gate only CONSUMES the
    // in-run whole-tree compile receipt, so these plans must arm the lazy install.
    // `gunbc_ci_plan_artifact_batches` is batch 1 of the floor schedule (the docs-only
    // shortcut) — same gate node, same receipt dependency.
    if plan_function == "gunbc_ci_floor_batches"
        || plan_function == "gunbc_ci_plan_artifact_batches"
    {
        enable_floor_compile_clean_lazy_install(&source_roots);
        // Eager install on the MAIN thread (lever 1, PR #6766): the receipt compile
        // rides this thread's `process_shared_index` — the same thread-local universe
        // the Discovery pump (main-thread lane in run_walk) resolves batch-2 witnesses
        // against — so the typed store the gate fills is the one the corpus reads.
        // Plan resolve completed above, matching the lazy path's ordering. The lazy
        // consume fallback stays armed; with the receipt installed here it never fires.
        // A refused install is loud AND fail-closed: the gate node consumes the
        // installed receipt (including a Refused one) and reds the floor.
        if let Err(msg) = install_floor_compile_clean_receipt() {
            eprintln!("claim_executor: eager compile-clean receipt install refused: {msg}");
        }
    }

    let outcome = run_walk(
        &source_roots,
        &batches,
        &governor,
        fast_lane_eval_budget_ms,
        falsifier_self_host_wet_budgets,
        batch_stop_policy,
    );
    match peak_rss_bytes() {
        Some(bytes) => {
            eprintln!("[measurement] floor peak RSS: {bytes} bytes (VmHWM) (adaptive width)");
        }
        None => eprintln!(
            "[measurement] floor peak RSS: unavailable (no /proc/self/status) (adaptive width)"
        ),
    }
    // The governor receipt is the §5-counted degradation story for the run: every graceful
    // hold, hard back-off, and forced-serial admission, beside the width actually reached.
    eprintln!("{}", governor.receipt_line());
    // [measurement] WHOLE-TREE cgroup peak — the SOUND placement divisor input (SELF-RSS above omits
    // child rustc/sccache PIDs; cgroup-v2 `memory.peak` at the leaf job cgroup is hierarchical and
    // captures them). Single authority `emit_cgroup_measurement` so the `ci` and `rust_tests` jobs
    // report an identically-shaped line. Runtime-harmless read-only.
    emit_cgroup_measurement("floor adaptive-width");
    if outcome.any_failed {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn main() -> ExitCode {
    // The materialization demand receipt is mandatory on the floor: enable the
    // interpreter's recompute-trace ledger unless the environment already set
    // it. An explicit =0 zeroes the receipt, and the derived ci.yml gate fails
    // closed on keyed_calls=0 — disabling is loud, never silent.
    if std::env::var_os("GUNBC_RECOMPUTE_TRACE").is_none() {
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
    }
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_arm_time_budget_refusal_plan_functions_match_dag_seed_roster() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("src/v2/workflow/ci_floor_plan.dag")
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph(&roots, &entry).expect("resolve ci_floor_plan");
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let value = run_value(
            &ctx,
            "gunbc_floor_arm_time_budget_refusal_plan_function_roster",
        )
        .expect("evaluate materialized plan roster");
        let dag_roster =
            str_list_from_value(&value, &ctx).expect("plan roster must be List<String>");
        assert_eq!(
            dag_roster.as_slice(),
            FLOOR_ARM_TIME_BUDGET_REFUSAL_PLAN_FUNCTIONS,
            "claim_executor seed const must match ci_floor_plan materialized roster \
             (dissolve-on: v2 emit of stage0 host constants)"
        );
    }

    // The materialization-receipt chain by execution: a real entry resolves, a
    // claim evaluates on its InterpContext, and the ctx Drop absorbs ledger
    // totals into the process accumulator. The env latch is process-global and
    // sticky (OnceLock), so under plain `cargo test` sibling tests in this
    // binary share it and their ctx drops may also absorb — every assertion
    // here is therefore monotone under concurrent absorbs (siblings can only
    // ADD totals; nothing here asserts the accumulator is empty). Drain-once
    // is Option::take by construction, not asserted through the shared global.
    #[test]
    fn materialization_receipt_totals_absorb_on_ctx_drop() {
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/materialization_ladder_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let _ = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
        {
            let (graph, indices) =
                resolve_entry_graph(&roots, &entry).expect("resolve ladder witness entry");
            // Pure render evaluation (ci_render.dag fns over measured data) — no effects, so
            // the hermetic envelope is exact; a service call sneaking in refuses loudly.
            let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
            let outcome = run_claim(&ctx, "single_pure_demand_is_accepted_recompute");
            assert!(
                matches!(outcome, ClaimOutcome::Pass),
                "claim must pass for the receipt to be meaningful"
            );
        }
        let totals = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
        assert!(
            totals.keyed_calls > 0,
            "ctx Drop must absorb ledger totals into the process accumulator"
        );
    }

    // The eval-frame memo by execution: the same pure claim evaluated twice on
    // one ctx must (a) produce identical values — the memo-vs-recompute
    // equivalence oracle at the value grain — and (b) record verified hits, so
    // "the cache worked" is a counted fact, never an assumption. Assertions
    // are per-ctx (eval_call_memo_counters), immune to test-process sharing.
    #[test]
    fn eval_call_memo_serves_verified_hits_with_identical_values() {
        // Every ledger-touching test must set the trace var BEFORE its first
        // eval: the enablement latch is process-wide and initialized once, so
        // whichever test evaluates first fixes it for every sibling (this
        // exact ordering red-failed the receipt test when this test ran
        // first without the var).
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/materialization_ladder_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let (graph, indices) =
            resolve_entry_graph(&roots, &entry).expect("resolve ladder witness entry");
        // Pure render evaluation (ci_render.dag fns over measured data) — no effects, so
        // the hermetic envelope is exact; a service call sneaking in refuses loudly.
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let first = run_value(
            &ctx,
            "cross_frame_duplicate_discharged_by_covering_provider",
        )
        .expect("first evaluation");
        let (_, misses_after_first, _) = v1_compiler::v1_interpreter::eval_call_memo_counters(&ctx);
        let second = run_value(
            &ctx,
            "cross_frame_duplicate_discharged_by_covering_provider",
        )
        .expect("second evaluation");
        assert!(
            first == second,
            "memo-served evaluation must equal the recomputed one"
        );
        let (hits, misses, overflow) = v1_compiler::v1_interpreter::eval_call_memo_counters(&ctx);
        assert!(
            hits > 0,
            "second identical evaluation must serve verified hits from the eval memo"
        );
        assert!(
            misses >= misses_after_first,
            "miss counter is monotone (counted, never reset)"
        );
        assert_eq!(overflow, 0, "tiny workload must not hit the entry cap");
    }

    fn single(entry: &str, function: &str) -> Runnable {
        Runnable::SingleClaim {
            entry: entry.to_string(),
            function: function.to_string(),
            use_walk_memo: false,
            execution_mode: ExecutionMode::Hermetic,
        }
    }

    fn discovery() -> Runnable {
        Runnable::DiscoveryBatch {
            source_roots: vec!["src/v2".to_string()],
            scan_dirs: vec![],
            explicit_entries: vec![],
            node_frontier_selection: NodeFrontierSelectionMode::Applied,
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: ExecutionMode::Hermetic,
        }
    }

    /// Flatten the grouped units back to the (entry, function) claims they will execute, in the
    /// order each unit runs them. `Discovery` contributes no SingleClaim pairs; an empty-entry
    /// sentinel surfaces as `("", function)`. This is the verdict-preservation oracle: grouping is
    /// only allowed to change scheduling, never which claims run.
    fn executed_claims(units: &[BatchUnit]) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for unit in units {
            match unit {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } => {
                    for f in functions {
                        out.push((entry.clone(), f.clone()));
                    }
                }
                BatchUnit::UnrunnableSentinel { function } => {
                    out.push((String::new(), function.clone()));
                }
                BatchUnit::Discovery { .. } => {}
            }
        }
        out
    }

    #[test]
    fn same_entry_claims_coalesce_into_one_resolve_group() {
        // The floor's batch-2 shape: many gate witnesses share one file. They must collapse to a
        // single resolve-group (one resolve) carrying every function, in input order.
        let batch = vec![
            single("gate.dag", "rust_gate"),
            single("gate.dag", "emit_gate"),
            single("gate.dag", "layering_gate"),
        ];
        let units = group_batch_units(&batch);
        assert_eq!(
            units.len(),
            1,
            "three same-entry claims => one resolve-group"
        );
        match &units[0] {
            BatchUnit::SharedClaims {
                entry, functions, ..
            } => {
                assert_eq!(entry, "gate.dag");
                assert_eq!(functions, &["rust_gate", "emit_gate", "layering_gate"]);
            }
            _ => panic!("expected a SharedClaims unit"),
        }
    }

    #[test]
    fn grouping_preserves_every_claim_exactly_once() {
        // Verdict preservation: no claim dropped, duplicated, or invented — grouping only reorders
        // by coalescing same-entry claims (keeping their relative order). Mixed entries interleave.
        let batch = vec![
            single("a.dag", "a1"),
            single("b.dag", "b1"),
            single("a.dag", "a2"),
            discovery(),
            single("b.dag", "b2"),
            single("", "__unmapped__"),
        ];
        let units = group_batch_units(&batch);

        // a.dag coalesces (a1 before a2), b.dag coalesces (b1 before b2), discovery + sentinel stay.
        let mut got = executed_claims(&units);
        let mut want = vec![
            ("a.dag".to_string(), "a1".to_string()),
            ("a.dag".to_string(), "a2".to_string()),
            ("b.dag".to_string(), "b1".to_string()),
            ("b.dag".to_string(), "b2".to_string()),
            (String::new(), "__unmapped__".to_string()),
        ];
        got.sort();
        want.sort();
        assert_eq!(got, want, "exact same claim set runs after grouping");

        // Same-entry relative order is preserved within each coalesced group.
        let a = units
            .iter()
            .find_map(|u| match u {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } if entry == "a.dag" => Some(functions),
                _ => None,
            })
            .expect("a.dag group present");
        assert_eq!(a, &["a1", "a2"]);
    }

    #[test]
    fn empty_entry_sentinel_is_its_own_failing_unit() {
        // The unmapped-node sentinel (empty entry) must never coalesce into a real resolve-group;
        // it is a stand-alone fail-closed unit so a non-complete plan stays red (DESIGN §5).
        let batch = vec![single("", "__gunbc_ci_floor_unmapped__")];
        let units = group_batch_units(&batch);
        assert_eq!(units.len(), 1);
        match &units[0] {
            BatchUnit::UnrunnableSentinel { function } => {
                assert_eq!(function, "__gunbc_ci_floor_unmapped__");
            }
            _ => panic!("empty-entry claim must be an UnrunnableSentinel, not a resolve-group"),
        }

        // And it fails closed when run.
        let unit = units.into_iter().next().unwrap();
        let results = run_batch_unit(
            vec!["src/v2".to_string()],
            unit,
            Arc::new(MemoryGovernor::from_environment(1)),
            None,
            FalsifierSelfHostWetBudgets::default(),
        );
        assert_eq!(results.len(), 1);
        assert!(!results[0].ok, "unmapped sentinel must fail closed");
    }

    #[test]
    fn discovery_batch_stays_an_isolated_unit() {
        // A discovery corpus node never merges with SingleClaims — it keeps its own shard-parallel
        // resolve path.
        let batch = vec![
            single("gate.dag", "g1"),
            discovery(),
            single("gate.dag", "g2"),
        ];
        let units = group_batch_units(&batch);
        assert_eq!(units.len(), 2, "gate group + discovery");
        assert!(units
            .iter()
            .any(|u| matches!(u, BatchUnit::Discovery { .. })));
        let gate = units
            .iter()
            .find_map(|u| match u {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } if entry == "gate.dag" => Some(functions),
                _ => None,
            })
            .expect("gate group present");
        assert_eq!(gate, &["g1", "g2"], "both gate claims kept in one group");
    }

    // --- build-artifact verification teeth (DESIGN §5 fail-open guard) ---

    fn write_exec(dir: &std::path::Path, name: &str, bytes: &[u8]) -> String {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join(name);
        std::fs::write(&p, bytes).expect("write artifact");
        let mut perms = std::fs::metadata(&p).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&p, perms).expect("chmod");
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn verify_build_artifacts_accepts_nonempty_executables() {
        let dir = std::env::temp_dir().join(format!("cev-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"\x7fELF-not-really-but-nonempty");
        let b = write_exec(&dir, "gunbc", b"binary");
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Ok(c) if c == ExitCode::SUCCESS),
            "real bins pass"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_zero_byte() {
        let dir = std::env::temp_dir().join(format!("cev-zero-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"ok");
        let b = write_exec(&dir, "gunbc", b""); // sccache served a truncated/empty cached artifact
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "zero-byte artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_missing() {
        let missing = std::env::temp_dir()
            .join(format!("cev-missing-{}/gunbc", std::process::id()))
            .to_string_lossy()
            .into_owned();
        let r = verify_build_artifacts(&[missing]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "absent artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_empty_arglist() {
        let r = verify_build_artifacts(&[]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(2)),
            "no declared artifacts is a usage error, fail closed"
        );
    }

    /// Warm==cold purity oracle for the cross-batch resolve memo (DESIGN §5).
    ///
    /// Proves that `run_memo_shared_claims` (warm path: 2nd+ calls share the cached
    /// `InterpContext`) produces byte-identical `(function, ok, detail)` results vs
    /// `run_shared_entry_claims` (cold path: fresh resolve each time). Goes RED if
    /// the shared `InterpContext` has interior mutability that contaminates results
    /// across claims — the failure mode the declaration-validity lenses cannot catch.
    #[test]
    fn memo_warm_cold_results_are_identical() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let functions: &[String] = &[
            "witness_negligible_profile_is_not_heavy".to_string(),
            "witness_substantial_memory_forbids_corpus_co_residence".to_string(),
        ];

        // Cold path: two independent resolves, one function each.
        let cold: Vec<(String, bool, String)> = functions
            .iter()
            .flat_map(|f| {
                run_shared_entry_claims(
                    &source_roots,
                    &entry,
                    std::slice::from_ref(f),
                    ExecutionMode::Hermetic,
                )
                .into_iter()
                .map(|r| (r.function, r.ok, r.detail))
            })
            .collect();

        // Warm path: single memo, first call resolves fresh, second hits the cache.
        let mut memo = std::collections::HashMap::new();
        let warm: Vec<(String, bool, String)> = functions
            .iter()
            .flat_map(|f| {
                run_memo_shared_claims(
                    &source_roots,
                    &entry,
                    std::slice::from_ref(f),
                    ExecutionMode::Hermetic,
                    &mut memo,
                )
                .into_iter()
                .map(|r| (r.function, r.ok, r.detail))
            })
            .collect();

        assert_eq!(
            warm, cold,
            "memo warm path must be byte-identical to cold path — shared InterpContext is pure"
        );
    }

    /// Resolve-count oracle: proves the memo fires resolve_entry_graph exactly once
    /// per distinct entry per walk (DESIGN §2 — no redundant resolve).
    ///
    /// Goes RED if the memo is bypassed: both calls would have resolve_nanos > 0,
    /// meaning resolve_entry_graph fired twice for the same entry instead of once.
    #[test]
    fn memo_deduplicates_resolve_count() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();

        let mut memo = std::collections::HashMap::new();

        // First call for this entry: must resolve (resolve_nanos > 0).
        let first = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_negligible_profile_is_not_heavy".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
        );
        assert!(
            first[0].resolve_nanos > 0,
            "first call must pay the resolve cost (resolve_entry_graph fires); ok={} detail={}",
            first[0].ok,
            first[0].detail
        );

        // Second call for the same entry AND mode: must cache-hit (resolve_nanos == 0).
        let second = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_substantial_memory_forbids_corpus_co_residence".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
        );
        assert_eq!(
            second[0].resolve_nanos, 0,
            "second call must cache-hit — resolve_entry_graph must NOT fire again"
        );
    }
}
