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
    compute_histogram_data, enable_floor_compile_clean_lazy_install, heartbeat_feed_enter_batch,
    heartbeat_feed_entry_completed, heartbeat_feed_snapshot, install_floor_compile_clean_receipt,
    make_eval_context, project_witness_cost_receipt, resolve_entry_graph,
    resolve_entry_graph_shared, run_claim, run_discovery_corpus_with_options, run_value, set_phase,
    top_n_slowest_witnesses, BudgetKind, ClaimOutcome, DiscoveryCorpusOptions, DiscoverySummary,
    DiscoveryWidthPolicy, FloorPhase, HistogramData, NodeFrontierSelectionMode, PhaseProfile,
    TimingPercentiles, DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N,
};
use v1_compiler::memory_governor::{
    binding_cap_cgroup_dir, binding_high_cgroup_dir, floor_budget_below_minimum_footprint,
    leaf_cgroup_dir, mem_total_bytes, memory_pressure_some_avg10, read_cgroup_raw, read_cgroup_u64,
    AdmittedSlot, MemoryGovernor,
};
use v1_compiler::v1_interpreter::{
    color_enabled, paint, run_in_context_with_args, sgr, ExecutionMode, InterpContext, Value,
};

/// Per-LANE budgets for the falsifier's rostered batches, each keyed by the lane's own
/// roster so a batch draws exactly the ceiling its lane declares. Self-host wet (green +
/// known-red quarantine) share the 600s wall budget; silent-pick owns
/// `gunbc_falsifier_silent_pick_gate_receipt_wall_budget` (900s); the Hermetic substrate
/// long lane owns `gunbc_falsifier_substrate_long_lane_witness_eval_budget`. No lane
/// inherits another's ceiling — the prior mis-scopes reddened silent-pick against the
/// 600s self-host wall (2026-07-25) and the substrate long lane against the 5s per-PR
/// fast-lane eval budget (run 30176416535, 7 of 10 rows killed at ~5001ms).
#[derive(Clone, Default)]
struct FalsifierSelfHostWetBudgets {
    wall_budget_ms: Option<u64>,
    interp_eval_budget_ms: Option<u64>,
    /// Entry paths from `falsifier_self_host_wet_entries` (green wet roster).
    roster_entry_paths: Vec<String>,
    /// Entry paths from `falsifier_self_host_wet_known_red_entries` — Wet expect_red
    /// quarantine; arms the same self-host wall budget + expect_red invert.
    known_red_entry_paths: Vec<String>,
    /// Hermetic known-red probe paths (`known_red_probe_entries`) — expect_red only.
    hermetic_known_red_entry_paths: Vec<String>,
    silent_pick_wall_budget_ms: Option<u64>,
    silent_pick_entry_paths: Vec<String>,
    /// Hermetic substrate long-lane paths (`falsifier_substrate_long_lane_entries`) —
    /// rows deliberately excluded from per-PR discovery precisely because their eval
    /// exceeds the fast-lane budget, so they arm their own dated eval ceiling instead.
    substrate_long_lane_entry_paths: Vec<String>,
    substrate_long_lane_eval_budget_ms: Option<u64>,
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

/// THE COST WALL (Piece 3 derived clamp — authority `gunbc.ci_spec.gunbc_ci_floor_batch_clamp_params`
/// + `gunbc_ci_floor_batch_clamp_note`): the per-batch clamp is `overhead_seconds*1000 +
/// runtime_unit_count * per_unit_ms`. This reads the two index-aligned param lists fail-closed at
/// arm time (the fast-lane-budget pattern); the clamp itself is computed at enforcement, because the
/// affected-set-selected unit count is a runtime datum the schedule does not hold. Clamps are
/// admission/scheduling facts at the walk grain — witness verdicts never carry a wall-clock term
/// (the ruling split reconciled in the carrier note).
fn read_floor_batch_clamp_params(
    plan_ctx: &InterpContext,
    batch_count: usize,
) -> Result<Vec<(u128, u128)>, String> {
    let overhead_items = match run_value(plan_ctx, "gunbc_ci_floor_batch_clamp_overhead_seconds") {
        Ok(Value::List(items)) => items,
        Ok(other) => {
            return Err(format!(
                "claim_executor: gunbc_ci_floor_batch_clamp_overhead_seconds must be a List<Int>, got {other:?} (fail-closed)"
            ));
        }
        Err(msg) => {
            return Err(format!(
                "claim_executor: floor plan schedules batches but gunbc_ci_floor_batch_clamp_overhead_seconds is unavailable (fail-closed): {msg}"
            ));
        }
    };
    let mut overheads_ms: Vec<u128> = Vec::new();
    for item in overhead_items.iter() {
        match item {
            Value::Int(n) if *n > 0 => overheads_ms.push(*n as u128 * 1000),
            other => {
                return Err(format!(
                    "claim_executor: gunbc_ci_floor_batch_clamp_overhead_seconds rows must be positive Ints, got {other:?} (fail-closed)"
                ));
            }
        }
    }
    let rate_items = match run_value(plan_ctx, "gunbc_ci_floor_batch_clamp_per_unit_ms") {
        Ok(Value::List(items)) => items,
        Ok(other) => {
            return Err(format!(
                "claim_executor: gunbc_ci_floor_batch_clamp_per_unit_ms must be a List<Int>, got {other:?} (fail-closed)"
            ));
        }
        Err(msg) => {
            return Err(format!(
                "claim_executor: floor plan schedules batches but gunbc_ci_floor_batch_clamp_per_unit_ms is unavailable (fail-closed): {msg}"
            ));
        }
    };
    let mut rates_ms: Vec<u128> = Vec::new();
    for item in rate_items.iter() {
        match item {
            Value::Int(n) if *n >= 0 => rates_ms.push(*n as u128),
            other => {
                return Err(format!(
                    "claim_executor: gunbc_ci_floor_batch_clamp_per_unit_ms rows must be non-negative Ints, got {other:?} (fail-closed)"
                ));
            }
        }
    }
    if overheads_ms.len() != batch_count || rates_ms.len() != batch_count {
        return Err(format!(
            "claim_executor: floor batch clamp params (overhead {} row(s), rate {} row(s)) must each cover the {} scheduled batch(es) exactly (fail-closed; update gunbc.ci_spec beside the schedule change)",
            overheads_ms.len(),
            rates_ms.len(),
            batch_count
        ));
    }
    Ok(overheads_ms.into_iter().zip(rates_ms).collect())
}

/// The RED-control fault injection (`GUNBC_FLOOR_BATCH_BUDGET_TIGHTEN_MS`): lowers the COMPUTED
/// per-batch clamp (min) at enforcement, so it can force a FLOOR-BATCH-OVER-BUDGET refusal for a
/// control run but can never open the gate — tighten-only by construction, never an escape hatch.
fn read_floor_batch_budget_tighten_ms() -> Result<Option<u128>, String> {
    match std::env::var("GUNBC_FLOOR_BATCH_BUDGET_TIGHTEN_MS") {
        Ok(t) => match t.parse::<u128>() {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(format!(
                "claim_executor: GUNBC_FLOOR_BATCH_BUDGET_TIGHTEN_MS must parse as milliseconds, got {t:?} (fail-closed)"
            )),
        },
        Err(_) => Ok(None),
    }
}

/// RED-control fault injection for the compile-clean leg clamp
/// (`gunbc_ci_compile_clean_clamp_note`): lowers the COMPUTED clamp (min), never
/// raises — the same posture as `GUNBC_FLOOR_BATCH_BUDGET_TIGHTEN_MS`.
fn read_compile_clean_budget_tighten_ms() -> Result<Option<u128>, String> {
    match std::env::var("GUNBC_FLOOR_COMPILE_CLEAN_BUDGET_TIGHTEN_MS") {
        Ok(t) => match t.parse::<u128>() {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(format!(
                "claim_executor: GUNBC_FLOOR_COMPILE_CLEAN_BUDGET_TIGHTEN_MS must parse as milliseconds, got {t:?} (fail-closed)"
            )),
        },
        Err(_) => Ok(None),
    }
}

/// The compile-clean leg's clamp constants (authority `gunbc.ci_spec.gunbc_ci_compile_clean_clamp`
/// + `gunbc_ci_compile_clean_clamp_note`, projected through the floor plan like the batch clamp
/// lists). Read fail-closed at arm time; returns (overhead_ms, per_unit_ms).
fn read_compile_clean_clamp(plan_ctx: &InterpContext) -> Result<(u128, u128), String> {
    let overhead_s = match run_value(plan_ctx, "gunbc_ci_compile_clean_clamp_overhead_seconds") {
        Ok(Value::Int(v)) if v > 0 => v as u128,
        Ok(other) => {
            return Err(format!(
                "claim_executor: gunbc_ci_compile_clean_clamp_overhead_seconds must be a positive Int, got {other:?} (fail-closed)"
            ))
        }
        Err(msg) => {
            return Err(format!(
                "claim_executor: plan schedules the compile-clean gate but gunbc_ci_compile_clean_clamp_overhead_seconds is unavailable (fail-closed): {msg}"
            ))
        }
    };
    let rate_ms = match run_value(plan_ctx, "gunbc_ci_compile_clean_clamp_rate_per_unit_ms") {
        Ok(Value::Int(v)) if v >= 0 => v as u128,
        Ok(other) => {
            return Err(format!(
                "claim_executor: gunbc_ci_compile_clean_clamp_rate_per_unit_ms must be a non-negative Int, got {other:?} (fail-closed)"
            ))
        }
        Err(msg) => {
            return Err(format!(
                "claim_executor: plan schedules the compile-clean gate but gunbc_ci_compile_clean_clamp_rate_per_unit_ms is unavailable (fail-closed): {msg}"
            ))
        }
    };
    Ok((overhead_s * 1000, rate_ms))
}

/// Clamp verdict for the compile-clean leg (prelude coverage follow-up (a), first slice —
/// authority `gunbc_ci_compile_clean_clamp_note`). Runs POST-WALK so it covers both the eager
/// and the lazy install path. Returns true when the walk must red (OverBudget). Admission
/// grain only: the compile receipt's ok is untouched (the signed admission/verdict split).
/// Also writes `target/floor-compile-clean-wall-receipt.txt` (mirrors the batch-wall body):
/// Unbudgeted when no clamp params were read, WithinBudget/OverBudget otherwise.
fn enforce_floor_compile_clean_clamp(
    clamp: Option<(u128, u128)>,
    tighten_ms: Option<u128>,
) -> bool {
    let Some((wall_ms, units, _rows, subject)) =
        v1_compiler::cli_run::floor_compile_clean_cost_snapshot()
    else {
        // Skipped / refused / never-armed legs record no cost — nothing to clamp; the
        // gate's own receipt consumption already carries those arms loudly.
        return false;
    };
    let mut body = String::new();
    body.push_str(&format!("compile_clean_wall_ms={wall_ms}\n"));
    body.push_str(&format!("compile_clean_units={units}\n"));
    body.push_str(&format!("compile_clean_scope={subject}\n"));
    let mut over = false;
    match clamp {
        None => {
            body.push_str("compile_clean_verdict=Unbudgeted\n");
        }
        Some((overhead_ms, rate_ms)) => {
            let mut clamp_ms = overhead_ms + units * rate_ms;
            if let Some(t) = tighten_ms {
                clamp_ms = clamp_ms.min(t);
            }
            let verdict = if wall_ms > clamp_ms {
                over = true;
                "OverBudget"
            } else {
                "WithinBudget"
            };
            body.push_str(&format!("compile_clean_clamp_ms={clamp_ms}\n"));
            body.push_str(&format!("compile_clean_verdict={verdict}\n"));
            if over {
                println!(
                    "{}",
                    paint(
                        &format!(
                            "✗ FLOOR-COMPILE-CLEAN-OVER-BUDGET wall_ms={wall_ms} clamp_ms={clamp_ms} units={units} scope={subject}                                  (clamp = overhead + units*rate; authority gunbc.ci_spec                                  gunbc_ci_compile_clean_clamp; raising an overhead or rate requires                                  an operator-signed line per gunbc_ci_compile_clean_clamp_note — a refusal,                                  never a widen)"
                        ),
                        sgr::ERROR
                    )
                );
            }
        }
    }
    let path = std::path::Path::new("target").join("floor-compile-clean-wall-receipt.txt");
    if let Err(e) =
        std::fs::create_dir_all("target").and_then(|_| std::fs::write(&path, body.as_bytes()))
    {
        eprintln!(
            "claim_executor: failed to write compile-clean wall receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return true;
    }
    eprintln!(
        "[receipt] floor compile-clean wall: wall_ms={wall_ms} units={units} scope={subject} (receipt: {})",
        path.display()
    );
    over
}

/// Drift comparison for the compile-clean cost rows (pass + per-module typecheck walls)
/// against `dag/gunbc/compile_clean_cost_basis.tsv` — the counted growth-detector half of
/// the family margin ruling, sharing the witness row-cost model wholesale: same basis
/// schema and parser (key = first two columns, here kind/subject), same host-class and
/// zero-basis refusals, same 2× authority `witness_row_cost_exceeds_basis`, same
/// three-valued verdict with BasisAbsent counted loudly (an unseeded basis must never
/// read as no-drift — the #7475 lesson). Runs on every floor walk that produced a cost
/// snapshot: the leg's row identities are stable (unlike the affected-set-selected
/// witness rows that keep witness drift on the falsifier cadence).
fn write_compile_clean_cost_drift_receipt_at(
    base: &std::path::Path,
    basis_path: &std::path::Path,
    source_roots: &[String],
) -> bool {
    let Some((wall_ms, _units, module_rows, subject)) =
        v1_compiler::cli_run::floor_compile_clean_cost_snapshot()
    else {
        return true; // no leg, no rows — nothing to compare, nothing to hide
    };
    let entry = "dag/gunbc/witness_row_cost.dag";
    let (graph, indices) = match resolve_entry_graph(source_roots, entry) {
        Ok(v) => v,
        Err(m) => {
            eprintln!(
                "claim_executor: failed to resolve {entry} for compile-clean drift comparator (fail-closed):\n{m}"
            );
            return false;
        }
    };
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);

    let mut basis: std::collections::HashMap<(String, String), WitnessRowCostBasisRow> =
        std::collections::HashMap::new();
    if let Ok(text) = std::fs::read_to_string(basis_path) {
        for line in text.lines().skip(1) {
            match parse_witness_row_cost_basis_line(line) {
                Ok(None) => {}
                Ok(Some((key, row))) => {
                    basis.insert(key, row);
                }
                Err(msg) => {
                    eprintln!("claim_executor: compile-clean basis: {msg}");
                }
            }
        }
    } else {
        eprintln!(
            "claim_executor: compile-clean cost basis file missing at {} — every row records BasisAbsent",
            basis_path.display()
        );
    }

    let mut rows: Vec<(String, String, u128)> = vec![("pass".to_string(), subject, wall_ms)];
    for (module, ms) in &module_rows {
        rows.push(("module_typecheck".to_string(), module.clone(), *ms as u128));
    }

    let mut body =
        String::from("kind\tsubject\tobserved_wall_ms\tbasis_wall_ms\tverdict\trun_ref\n");
    let mut drift_count = 0usize;
    let mut basis_absent_count = 0usize;
    for (kind, subj, observed) in &rows {
        match basis.get(&(kind.clone(), subj.clone())) {
            None => {
                basis_absent_count += 1;
                body.push_str(&format!("{kind}\t{subj}\t{observed}\t\tBasisAbsent\t\n"));
            }
            Some(b) => {
                let exceeds = match witness_row_cost_exceeds_basis_via_authority(
                    &ctx,
                    *observed,
                    b.eval_ms_basis,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!(
                            "claim_executor: compile-clean drift comparator refused for {kind}::{subj}: {e} — walk fails closed here"
                        );
                        return false;
                    }
                };
                let verdict = if exceeds {
                    drift_count += 1;
                    "DriftExceeded"
                } else {
                    "WithinBasis"
                };
                body.push_str(&format!(
                    "{kind}\t{subj}\t{observed}\t{}\t{verdict}\t{}\n",
                    b.eval_ms_basis, b.run_ref
                ));
            }
        }
    }
    eprintln!(
        "[compile-clean-cost-drift] basis_absent={basis_absent_count} drift_exceeded={drift_count}"
    );
    for line in body.lines() {
        eprintln!("[compile-clean-cost-drift] {line}");
    }
    let path = base.join("floor-compile-clean-cost-drift-receipt.tsv");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write compile-clean cost drift receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor compile-clean cost drift: basis_absent={basis_absent_count} drift_exceeded={drift_count} (TSV: {})",
        path.display()
    );
    true
}

/// Runtime per-batch unit count for the derived clamp: a discovery aggregate result contributes
/// its post-selection witness count (`corpus_witnesses`); every single-claim gate row contributes 1.
fn batch_runtime_unit_count(results: &[ClaimResult]) -> u128 {
    results
        .iter()
        .map(|r| {
            if r.corpus_witnesses > 0 {
                r.corpus_witnesses as u128
            } else {
                1u128
            }
        })
        .sum()
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

/// The residency class a runnable declares (`std.realization_schedule`
/// `RunnableMemoryClass`). A structural marker, not a quantity — the operator ruling
/// that retired predicted-peak byte constants stands, and this carries only the
/// Negligible/Substantial fact co-residence structure keys on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ParsedMemoryClass {
    Negligible,
    Substantial,
}

/// THE WHOLE PROFILE, retained. `std.realization_schedule.RunnableResourceProfile`
/// declares four facts; the parser used to keep two — `heavy_whole_tree_resolve` (as
/// `use_walk_memo`) and `execution_mode` — and drop `spawns_host_compiler` and the
/// memory class at parse time. That was invisible while only the ordinary batch path
/// consumed profiles, because the ordinary path happens to need exactly the two that
/// survived. It stops being invisible the moment ONE executor serves both populations:
/// a shared `run_stage` cannot enforce a stage's declared resource contract against
/// facts the parse threw away, so the arm-time validator could wall the heavy-resolve
/// case and nothing else (review 2026-07-30, naming this the run_stage prerequisite).
///
/// Retained as a unit rather than as four sibling fields on the runnable so that adding
/// a fifth profile fact is one edit here, not a fifth thing to remember to thread.
/// Whether the plan actually DESCRIBED this runnable's resources, or the parse supplied
/// fail-closed values because no profile was present. Hermetic is genuinely conservative
/// for effects, but `heavy_whole_tree_resolve: false`, `spawns_host_compiler: false` and
/// `memory: Negligible` are OPTIMISTIC assertions about work nobody described — and once
/// a profileless ClaimRef becomes the same SingleClaim variant an explicitly-profiled one
/// does, the stage validator cannot tell "declared Negligible" from "nothing declared"
/// (review 2026-07-31). A pure but whole-tree-heavy ClaimRef would enter a stage looking
/// negligible. Absence is therefore its own state, and stages refuse it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ParsedProfileProvenance {
    Declared,
    Undeclared,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ParsedRunnableProfile {
    provenance: ParsedProfileProvenance,
    heavy_whole_tree_resolve: bool,
    spawns_host_compiler: bool,
    memory: ParsedMemoryClass,
    execution_mode: ExecutionMode,
}

impl ParsedRunnableProfile {
    /// The fail-closed profile for a runnable that declares none (`ClaimRef`, which
    /// carries no profile field at all). Mirrors
    /// `runnable_resource_profile_negligible`: Hermetic envelope, no host compiler, no
    /// heavy resolve. An undeclared runnable that actually needs live effects refuses
    /// loudly at the effect boundary rather than silently dispatching them.
    fn undeclared() -> Self {
        ParsedRunnableProfile {
            provenance: ParsedProfileProvenance::Undeclared,
            heavy_whole_tree_resolve: false,
            spawns_host_compiler: false,
            memory: ParsedMemoryClass::Negligible,
            execution_mode: ExecutionMode::Hermetic,
        }
    }

    /// True when this runnable is heavier than the negligible-admission class stages are
    /// currently sized for — either it spawns a host compiler or it declares substantial
    /// residency. Read by the stage admissibility validator.
    fn is_substantial_or_spawns_compiler(&self) -> bool {
        self.spawns_host_compiler || matches!(self.memory, ParsedMemoryClass::Substantial)
    }
}

#[derive(Clone)]
enum Runnable {
    SingleClaim {
        entry: String,
        function: String,
        profile: ParsedRunnableProfile,
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

/// Parse the whole `RunnableResourceProfile`. An ABSENT profile field yields the
/// fail-closed undeclared profile; a profile that EXISTS but omits a field is a
/// REFUSAL, not a default — the `node_frontier_selection` precedent, so a stale plan
/// must redeclare its semantics rather than inherit them silently. `execution_mode`
/// keeps its own parser because its absent-vs-malformed split is already stated there.
fn parsed_runnable_profile_from_field(
    profile: Option<&Value>,
    owner: &str,
    ctx: &InterpContext,
) -> Result<ParsedRunnableProfile, String> {
    let execution_mode = execution_mode_from_profile_field(profile, owner, ctx)?;
    let fields = match profile {
        Some(Value::Record { fields, .. }) | Some(Value::Variant { fields, .. }) => fields,
        // No profile at all: every axis takes its fail-closed value. execution_mode
        // above has already resolved to Hermetic for the same reason.
        _ => return Ok(ParsedRunnableProfile::undeclared()),
    };
    let heavy_whole_tree_resolve = match ctx.field(fields, "heavy_whole_tree_resolve") {
        Some(Value::Bool(b)) => *b,
        Some(other) => {
            return Err(format!(
                "{owner}.profile.heavy_whole_tree_resolve must be a Bool, got {}",
                ctx.format_value(other)
            ))
        }
        None => {
            return Err(format!(
                "{owner}.profile omits `heavy_whole_tree_resolve` — a profile that exists \
                 declares every axis; a stale plan redeclares rather than inheriting"
            ))
        }
    };
    let spawns_host_compiler = match ctx.field(fields, "spawns_host_compiler") {
        Some(Value::Bool(b)) => *b,
        Some(other) => {
            return Err(format!(
                "{owner}.profile.spawns_host_compiler must be a Bool, got {}",
                ctx.format_value(other)
            ))
        }
        None => {
            return Err(format!(
                "{owner}.profile omits `spawns_host_compiler` — a profile that exists \
                 declares every axis; a stale plan redeclares rather than inheriting"
            ))
        }
    };
    let memory = match ctx.field(fields, "memory") {
        Some(Value::Variant { variant_name, .. })
            if ctx.sym_eq(*variant_name, "RunnableMemoryNegligible") =>
        {
            ParsedMemoryClass::Negligible
        }
        Some(Value::Variant { variant_name, .. })
            if ctx.sym_eq(*variant_name, "RunnableMemorySubstantial") =>
        {
            ParsedMemoryClass::Substantial
        }
        Some(other) => {
            return Err(format!(
                "{owner}.profile.memory must be RunnableMemoryNegligible or \
                 RunnableMemorySubstantial, got {}",
                ctx.format_value(other)
            ))
        }
        None => {
            return Err(format!(
                "{owner}.profile omits `memory` — a profile that exists declares every \
                 axis; a stale plan redeclares rather than inheriting"
            ))
        }
    };
    Ok(ParsedRunnableProfile {
        provenance: ParsedProfileProvenance::Declared,
        heavy_whole_tree_resolve,
        spawns_host_compiler,
        memory,
        execution_mode,
    })
}

fn runnable_from_value(value: &Value, ctx: &InterpContext) -> Result<Runnable, String> {
    match value {
        Value::Record { type_name, fields } if ctx.sym_eq(*type_name, "ClaimRef") => {
            Ok(Runnable::SingleClaim {
                entry: str_field(fields, "entry", "ClaimRef", ctx)?,
                function: str_field(fields, "function", "ClaimRef", ctx)?,
                // ClaimRef carries no profile at all: fail-closed on every axis
                // (ParsedRunnableProfile::undeclared).
                profile: ParsedRunnableProfile::undeclared(),
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableSingleClaim") => {
            let entry = str_field(fields, "entry", "RunnableSingleClaim", ctx)?;
            let function = str_field(fields, "function", "RunnableSingleClaim", ctx)?;
            let profile_value = ctx.field(fields, "profile");
            let profile =
                parsed_runnable_profile_from_field(profile_value, "RunnableSingleClaim", ctx)?;
            Ok(Runnable::SingleClaim {
                entry,
                function,
                profile,
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

/// The parsed form of `std.realization_schedule.WalkPlan` — the ONE plan shape every
/// plan function returns (see `walk_plan_note` on the carrier). Two populations with
/// different ordering laws: `batches` are the ordinary floor under the walk's
/// `FloorBatchStopPolicy`; `on_success_stages` run only after the ordinary floor
/// completed AND its receipts wrote, each stage a barrier, always fail-fast between
/// stages regardless of the ordinary stop policy.
struct ParsedWalkPlan {
    batches: Vec<Vec<Runnable>>,
    finalization: Option<FloorFinalization>,
    on_success_stages: Vec<Vec<Runnable>>,
}

/// Parse the plan-carried finalization VALUE (walk_finalization_note). The executor
/// never selects finalization by a plan function's spelling, and — since the carrier
/// became `WalkPlan<F>` — it does not need to care which instantiation produced the
/// value either: ONE parser reads both. An unrecognized shape is a hard error, and that
/// refusal is load-bearing rather than belt-and-braces: the plan function's declared
/// return type does NOT bound what its body returns (the typechecker does not check
/// return position), so this parser and the enrolled value witnesses are what actually
/// stop a plan from carrying the wrong finalization family.
///
/// `Nat` reaches the interpreter as a native `Int` (the numeric tower is grounded), so
/// a negative value cannot arrive from a well-typed plan; it is still refused rather
/// than carried into a count comparison it could only lose.
fn finalization_from_value(
    v: &Value,
    ctx: &InterpContext,
) -> Result<Option<FloorFinalization>, String> {
    // The two inhabitants have DIFFERENT runtime shapes, and that is a consequence of
    // the carrier split rather than an accident to paper over: FloorFinalization is a
    // standalone record in gunbc.ci_materialization (Value::Record), while
    // NoFinalizationDeclared is the nullary variant of std's NoWalkFinalization sum
    // (Value::Variant). Both are matched by TYPE NAME — never by "has a field called
    // declared_resolve_count", which would admit any record that happened to carry one.
    let floor_from_fields =
        |fields: &[(v1_compiler::v1_interpreter::Symbol, Value)]| -> Result<i64, String> {
            match ctx.field(fields, "declared_resolve_count") {
                Some(Value::Int(n)) if *n >= 0 => Ok(*n),
                Some(Value::Int(n)) => Err(format!(
                    "FloorFinalization.declared_resolve_count is Nat, got {n}"
                )),
                other => Err(format!(
                    "FloorFinalization.declared_resolve_count must be a Nat, got {other:?}"
                )),
            }
        };
    match v {
        Value::Record {
            type_name, fields, ..
        } if ctx.sym_eq(*type_name, "FloorFinalization") => Ok(Some(FloorFinalization {
            declared_resolve_count: floor_from_fields(fields)?,
        })),
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if ctx.sym_eq(*variant_name, "NoFinalizationDeclared") {
                Ok(None)
            } else if ctx.sym_eq(*variant_name, "FloorFinalization") {
                // Retained because the same declaration can reach the interpreter as a
                // record OR as a variant depending on how it was constructed; refusing
                // one spelling of a value the model does admit would be a false wall.
                Ok(Some(FloorFinalization {
                    declared_resolve_count: floor_from_fields(fields)?,
                }))
            } else {
                Err(
                    "unknown variant (expected NoFinalizationDeclared or FloorFinalization)"
                        .to_string(),
                )
            }
        }
        other => Err(format!(
            "must be a NoFinalizationDeclared or FloorFinalization value, got {}",
            ctx.format_value(other)
        )),
    }
}

/// Strict WalkPlan parser. Deliberately NO fallback from a failed record parse to a
/// bare-`List<List<Runnable>>` reading: that fallback would let a malformed plan run
/// with its success stages silently dropped — the silent-widen arm §5 forbids. A plan
/// with no postconditions declares `on_success_stages: []`; it never omits the field.
fn walk_plan_from_plan(plan: &Value, ctx: &InterpContext) -> Result<ParsedWalkPlan, String> {
    let fields = match plan {
        Value::Record { fields, .. } => fields,
        Value::Variant { fields, .. } => fields,
        other => {
            return Err(format!(
                "expected a WalkPlan record {{ batches, finalization, on_success_stages }}, \
                 got {} — every plan function returns WalkPlan<F> (std.realization_schedule \
                 walk_plan_note); there is deliberately no bare-list fallback",
                ctx.format_value(other)
            ))
        }
    };
    let batches_val = ctx
        .field(fields, "batches")
        .ok_or_else(|| "WalkPlan missing field `batches`".to_string())?;
    let stages_val = ctx.field(fields, "on_success_stages").ok_or_else(|| {
        "WalkPlan missing field `on_success_stages` — a plan with no postconditions \
         declares an empty list, never omits the field"
            .to_string()
    })?;
    let finalization_val = ctx.field(fields, "finalization").ok_or_else(|| {
        "WalkPlan missing field `finalization` — a plan with no finalization declares \
         NoFinalizationDeclared, never omits the field"
            .to_string()
    })?;
    Ok(ParsedWalkPlan {
        batches: batches_from_plan(batches_val, ctx)
            .map_err(|msg| format!("WalkPlan.batches: {msg}"))?,
        finalization: finalization_from_value(finalization_val, ctx)
            .map_err(|msg| format!("WalkPlan.finalization: {msg}"))?,
        on_success_stages: batches_from_plan(stages_val, ctx)
            .map_err(|msg| format!("WalkPlan.on_success_stages: {msg}"))?,
    })
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
    /// Per-witness eval+resolve identity preserved from discovery (empty for gate/single-claim rows).
    witness_row_costs: Vec<(String, String, u128, u128, u128, String, String)>,
    /// Set only when this row was killed at a budget, carrying the pair that explains it.
    ///
    /// `ok`/`detail` are a lossy flattening of `ClaimOutcome`, so without this the batch's
    /// failure mode has to be recovered by substring-matching `detail` — which is what
    /// `falsifier_failure_mode` did, and its fallback arm is `WitnessRed`, so a reworded
    /// message silently demoted a budget refusal to a witness failure. This keeps the fact
    /// as data on the path that needs it. It is a projection of `ClaimOutcome::TimedOut`,
    /// not a second authority: nothing sets it except the `TimedOut` arm below.
    budget_refusal: Option<BudgetRefusal>,
}

/// The pair explaining a budget kill, kept alongside the flattened `ok`/`detail`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BudgetRefusal {
    elapsed_ms: u64,
    budget_ms: u64,
    kind: BudgetKind,
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
                profile,
            } => {
                let use_walk_memo = &profile.heavy_whole_tree_resolve;
                let execution_mode = &profile.execution_mode;
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
            witness_row_costs: Vec::new(),
            budget_refusal: None,
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
            witness_row_costs: Vec::new(),
            budget_refusal: None,
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
            witness_row_costs: Vec::new(),
            budget_refusal: None,
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
            witness_row_costs: Vec::new(),
            budget_refusal: None,
        },
        ClaimOutcome::TimedOut {
            elapsed_ms,
            budget_ms,
            kind,
        } => ClaimResult {
            function,
            ok: false,
            // The detail still names the budget in prose for the human reading a log, but
            // `budget_refusal` beside it is what classification reads — so the mode no
            // longer depends on this wording. "ceiling" is deliberate: the row was killed
            // AT the budget, so elapsed bounds the cost, it does not measure it.
            detail: format!(
                "killed at its {} budget: {}ms elapsed > {}ms budget (elapsed is a ceiling, \
                 not a completed duration)",
                kind.label(),
                elapsed_ms,
                budget_ms
            ),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            witness_row_costs: Vec::new(),
            budget_refusal: Some(BudgetRefusal {
                elapsed_ms,
                budget_ms,
                kind,
            }),
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
            witness_row_costs: Vec::new(),
            budget_refusal: None,
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
            let DiscoveryBatchBudgets {
                eval_budget_ms: effective_fast_lane,
                wet_wall_budget_ms,
                wet_interp_budget_ms,
            } = select_discovery_batch_budgets(
                execution_mode,
                &explicit_entries,
                fast_lane_eval_budget_ms,
                &falsifier_self_host_wet_budgets,
            );
            let expect_red = discovery_entries_are_expect_red(
                &explicit_entries,
                &falsifier_self_host_wet_budgets.hermetic_known_red_entry_paths,
                &falsifier_self_host_wet_budgets.known_red_entry_paths,
            );
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
                expect_red,
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
                    witness_row_costs: Vec::new(),
                    budget_refusal: None,
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
                        witness_row_costs: Vec::new(),
                        budget_refusal: None,
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
    rows: &[(String, String, u128, u128, u128, String, String)],
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
                (Some("function".to_string()), Value::Str(row.1.clone())),
                (Some("entry".to_string()), Value::Str(row.0.clone())),
                (
                    Some("eval_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.2)),
                ),
                (
                    Some("resolve_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.3)),
                ),
                (
                    Some("total_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.4)),
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

fn emit_slowest_witness_attribution(
    source_roots: &[String],
    rows: &[(String, String, u128, u128, u128, String, String)],
) {
    let n = slowest_witness_attribution_n().min(rows.len());
    if n == 0 {
        return;
    }
    let top = top_n_slowest_witnesses(rows, n);
    match render_slowest_witnesses(source_roots, &top) {
        Ok(boxed) => {
            eprintln!("{boxed}");
            let tail_eval_ms: u128 = top.iter().map(|r| r.2).sum::<u128>() / 1_000_000;
            let total_eval_ms = rows.iter().map(|r| r.2).sum::<u128>() / 1_000_000;
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

/// Known-red probe cadence (gunbc.ci_layer_roots known_red_probe_entries) and
/// Wet assemble-red quarantine (falsifier_self_host_wet_known_red_entries): the
/// batch EXPECTS RED. Greening is the un-quarantine event — so inverted verdict:
/// still-red (Bool(false) / resolve refuse) ⇒ PASS; unexpected green ⇒ FAIL.
fn discovery_entries_are_expect_red(
    explicit_entries: &[(String, String)],
    hermetic_known_red_paths: &[String],
    wet_known_red_paths: &[String],
) -> bool {
    if explicit_entries.is_empty() {
        return false;
    }
    explicit_entries.iter().all(|(entry, _)| {
        hermetic_known_red_paths.iter().any(|p| p == entry)
            || wet_known_red_paths.iter().any(|p| p == entry)
    })
}

/// True when this discovery batch's explicit entries intersect `roster_entry_paths`.
/// Path equality is the gate — substring heuristics would re-fork the roster.
/// The budgets a single discovery batch runs under.
struct DiscoveryBatchBudgets {
    /// Cooperative per-witness CPU eval deadline.
    eval_budget_ms: Option<u64>,
    /// Whole-receipt wall budget (Wet lanes only).
    wet_wall_budget_ms: Option<u64>,
    /// Secondary interpreter-wedge eval deadline for Wet self-host receipts.
    wet_interp_budget_ms: Option<u64>,
}

/// Budgets are scoped by LANE, never by witness kind.
///
/// The fast-lane eval budget (operator 5s rule) governs the per-PR discovery corpus and its
/// cold replays — witnesses whose own eval must stay cheap or move to a `long/` lane. A
/// Hermetic batch that carries its own lane roster draws that lane's dated ceiling instead:
/// selecting on `is_hermetic()` alone armed the 5s per-PR budget on the substrate long lane,
/// whose entire purpose is hosting rows OVER that budget (run 30176416535 — 7 of 10 rows
/// killed at ~5001ms by a refusal whose own text says the row belongs on its dedicated lane).
///
/// The residual (Hermetic, no lane roster) keeps the fast lane, so a new lane that forgets to
/// declare a ceiling reds loudly at 5s rather than running unbounded — the failure arm
/// narrows, never widens (DESIGN §5). For the same reason the declared ceiling is part of the
/// condition, not the payload: a rostered lane whose budget row is missing falls back to the
/// fast lane rather than to no budget at all. The plan read refuses a missing/mistyped budget
/// first, so that arm is unreachable today; it is written so the unreachable direction is the
/// narrow one.
///
/// Wet batches skip the eval budget (subprocess I/O is their job) and take whole-receipt wall
/// budgets the same roster-scoped way: self-host wet (green OR known-red quarantine) → 600s;
/// silent-pick → 900s dated ceiling; rehomed bin inherits neither (receipt: silent-pick 707s
/// red under a mis-scoped 600s self-host budget while batch walls stayed green — 2026-07-25).
fn select_discovery_batch_budgets(
    execution_mode: ExecutionMode,
    explicit_entries: &[(String, String)],
    fast_lane_eval_budget_ms: Option<u64>,
    budgets: &FalsifierSelfHostWetBudgets,
) -> DiscoveryBatchBudgets {
    let fast_lane = DiscoveryBatchBudgets {
        eval_budget_ms: fast_lane_eval_budget_ms,
        wet_wall_budget_ms: None,
        wet_interp_budget_ms: None,
    };
    if execution_mode.is_hermetic() {
        return match budgets.substrate_long_lane_eval_budget_ms {
            Some(ms)
                if discovery_entries_intersect_roster(
                    explicit_entries,
                    &budgets.substrate_long_lane_entry_paths,
                ) =>
            {
                DiscoveryBatchBudgets {
                    eval_budget_ms: Some(ms),
                    wet_wall_budget_ms: None,
                    wet_interp_budget_ms: None,
                }
            }
            _ => fast_lane,
        };
    }
    if discovery_entries_intersect_roster(explicit_entries, &budgets.roster_entry_paths)
        || discovery_entries_intersect_roster(explicit_entries, &budgets.known_red_entry_paths)
    {
        return DiscoveryBatchBudgets {
            eval_budget_ms: None,
            wet_wall_budget_ms: budgets.wall_budget_ms,
            wet_interp_budget_ms: budgets.interp_eval_budget_ms,
        };
    }
    if discovery_entries_intersect_roster(explicit_entries, &budgets.silent_pick_entry_paths) {
        return DiscoveryBatchBudgets {
            eval_budget_ms: None,
            wet_wall_budget_ms: budgets.silent_pick_wall_budget_ms,
            wet_interp_budget_ms: None,
        };
    }
    DiscoveryBatchBudgets {
        eval_budget_ms: None,
        wet_wall_budget_ms: None,
        wet_interp_budget_ms: None,
    }
}

fn discovery_entries_intersect_roster(
    explicit_entries: &[(String, String)],
    roster_entry_paths: &[String],
) -> bool {
    if roster_entry_paths.is_empty() || explicit_entries.is_empty() {
        return false;
    }
    explicit_entries
        .iter()
        .any(|(entry, _)| roster_entry_paths.iter().any(|p| p == entry))
}

fn read_schedule_witness_entry_paths(
    plan_ctx: &InterpContext,
    function: &str,
) -> Result<Vec<String>, String> {
    match run_value(plan_ctx, function) {
        Ok(v) => {
            let mut out = Vec::new();
            for elem in free_monoid_elems(&v, plan_ctx)? {
                let fields = match elem {
                    Value::Record { fields, .. } => fields,
                    Value::Variant { fields, .. } => fields,
                    other => {
                        return Err(format!(
                            "claim_executor: {function} element is {}, not a record (fail-closed)",
                            other.type_label_public()
                        ))
                    }
                };
                out.push(str_field(fields, "entry", function, plan_ctx)?);
            }
            Ok(out)
        }
        Err(msg) => Err(format!(
            "claim_executor: {function} is unavailable (fail-closed): {msg}"
        )),
    }
}

/// The first budget kill among this discovery batch's witnesses, if any.
///
/// Discovery is the falsifier path — it is where the incident that motivated the typed
/// `TimedOut` actually lives (`resolution_divergence_silent_pick_gate_keystone_holds` is a
/// discovery row). A discovery batch flattens N witness outcomes into one `ok`/`detail`
/// pair, so without lifting the refusal out of `witness_outcomes` the batch would carry
/// `budget_refusal: None`, miss the structural path in `batch_failure_mode_and_detail`, and
/// fall back to substring-matching a detail string that no longer contains the old budget
/// prose — reporting `WitnessRed` for a budget kill on the exact path this change exists to
/// fix (review 45220).
fn discovery_budget_refusal(summary: &DiscoverySummary) -> Option<BudgetRefusal> {
    summary
        .witness_outcomes
        .iter()
        .find_map(|w| match w.outcome {
            ClaimOutcome::TimedOut {
                elapsed_ms,
                budget_ms,
                kind,
            } => Some(BudgetRefusal {
                elapsed_ms,
                budget_ms,
                kind,
            }),
            _ => None,
        })
}

fn discovery_claim_result(
    function: String,
    ok: bool,
    detail: String,
    summary: &DiscoverySummary,
    projected: Result<Vec<(String, String, u128, u128, u128, String, String)>, String>,
) -> ClaimResult {
    // Per-row identity is load-bearing for the receipt spine: a compute failure OR an
    // incomplete row set must refuse the discovery claim (typed/located), never silently
    // emit a partial receipt as complete (§5 / review 43261 + review 43274).
    match projected {
        Ok(witness_row_costs) => ClaimResult {
            function,
            ok,
            detail,
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: summary.total_resolve_nanos,
            corpus_eval_nanos: summary.total_measured_nanos,
            corpus_witnesses: summary.total,
            witness_row_costs,
            budget_refusal: discovery_budget_refusal(summary),
        },
        Err(msg) => {
            eprintln!("[witness-row-cost] refused: {msg}");
            // Preserve the caller's failure context (e.g. "N of M discovery witness(es)
            // failed: …") — receipt refusal is an additional located cause, never a
            // replacement that erases the discovery diagnostic (DESIGN §5 / review 43284).
            let receipt = format!("witness row-cost receipt refused: {msg}");
            let detail = if detail.is_empty() {
                receipt
            } else {
                format!("{detail}; {receipt}")
            };
            ClaimResult {
                function,
                ok: false,
                detail,
                wall_nanos: 0,
                resolve_nanos: 0,
                corpus_resolve_nanos: summary.total_resolve_nanos,
                corpus_eval_nanos: summary.total_measured_nanos,
                corpus_witnesses: summary.total,
                witness_row_costs: Vec::new(),
                // Same rule as the detail above: a receipt refusal ADDS a cause, it does not
                // erase the one already established. A batch that blew its budget and then
                // failed to project its receipt is still a budget kill, and dropping that
                // here would hand it back to the substring classifier.
                budget_refusal: discovery_budget_refusal(summary),
            }
        }
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
    expect_red: bool,
) -> ClaimResult {
    set_phase(FloorPhase::Discovery, "discovery-corpus");
    let label = format!(
        "discovery-corpus[{} root(s)+{} explicit, adaptive width{}]",
        source_roots.len(),
        explicit_entries.len(),
        if expect_red { ", expect_red" } else { "" },
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
            eprintln!(
                "[assembly-split] schedule={:.1}ms probe={:.1}ms registry={:.1}ms services={:.1}ms rewire={:.1}ms emit_info={:.1}ms residue={:.1}ms",
                ms(st.assembly_schedule),
                ms(st.assembly_probe),
                ms(st.assembly_registry),
                ms(st.assembly_services),
                ms(st.assembly_rewire),
                ms(st.assembly_emit_info),
                ms(st.reconcile_assembly),
            );
            // Both halves come from the SAME per-entry spans here (`total_resolve_nanos`
            // and `total_stage_nanos` are accumulated entry by entry at the same two call
            // sites), so the discovery corpus can be partitioned against its own parent
            // rather than the thread-local account, which would see only the pump thread.
            {
                let span_rows: Vec<(String, u64, u128, v1_compiler::cli_run::ResolveStageNanos)> = {
                    let mut by_entry: std::collections::HashMap<
                        String,
                        (u64, u128, v1_compiler::cli_run::ResolveStageNanos),
                    > = std::collections::HashMap::new();
                    for r in &summary.entry_resolve_receipts {
                        let slot = by_entry.entry(r.entry.clone()).or_default();
                        slot.0 += 1;
                        slot.1 += r.resolve_nanos;
                        slot.2.accumulate(&r.stage_nanos);
                    }
                    let mut rows: Vec<(
                        String,
                        u64,
                        u128,
                        v1_compiler::cli_run::ResolveStageNanos,
                    )> = by_entry
                        .into_iter()
                        .map(|(k, (n, ns, st))| (k, n, ns, st))
                        .collect();
                    rows.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
                    rows
                };
                let partition = v1_compiler::cli_run::exclusive_cost_partition_from(
                    st,
                    "summed_per_entry_discovery_resolve_span_nanos",
                    summary.total_resolve_nanos,
                    summary.entry_resolve_receipts.len() as u64,
                    0,
                    span_rows,
                );
                eprintln!(
                    "[cost-partition] {}",
                    v1_compiler::cli_run::render_exclusive_cost_partition_json(
                        &partition,
                        &[(
                            "witness_eval_measured_nanos",
                            summary.total_measured_nanos as u128
                        )],
                    )
                );
            }
            let projected = project_witness_cost_receipt(&source_roots, &summary);
            match &projected {
                Ok(rows) => {
                    match render_timing_histogram(&source_roots, &compute_histogram_data(rows)) {
                        Ok(histogram) => eprintln!("{histogram}"),
                        Err(e) => eprintln!("[histogram] render failed (timings unaffected): {e}"),
                    }
                }
                Err(msg) => eprintln!("{msg}"),
            }
            if let Ok(rows) = &projected {
                emit_slowest_witness_attribution(&source_roots, rows);
            }
            if expect_red && summary.total > 0 {
                // All green on an expect-red probe = stale quarantine (dissolve-on fired).
                discovery_claim_result(
                    label,
                    false,
                    format!(
                        "expect_red probe unexpectedly green: {} witness(es) passed — un-quarantine (delete known_red_probe_entries / falsifier_self_host_wet_known_red_entries rows) or restore the discriminating red",
                        summary.total
                    ),
                    &summary,
                    projected,
                )
            } else {
                discovery_claim_result(
                    format!("{label} ({} witnesses)", summary.total),
                    true,
                    String::new(),
                    &summary,
                    projected,
                )
            }
        }
        Ok(summary) => {
            let projected = project_witness_cost_receipt(&source_roots, &summary);
            if expect_red {
                eprintln!(
                    "[expect-red] known-red probe still red: {} of {} failed (agreement — quarantine holds)",
                    summary.failures.len(),
                    summary.total
                );
                discovery_claim_result(
                    format!("{label} (expect_red still-red OK)"),
                    true,
                    String::new(),
                    &summary,
                    projected,
                )
            } else {
                discovery_claim_result(
                    label,
                    false,
                    format!(
                        "{} of {} discovery witness(es) failed: {}",
                        summary.failures.len(),
                        summary.total,
                        summary.failures.join("; ")
                    ),
                    &summary,
                    projected,
                )
            }
        }
        Err(msg) => {
            if expect_red {
                // Resolve-refuse is the documented known-red shape for logic_ground_truth
                // (imported bare variants unbound in expression position).
                eprintln!(
                    "[expect-red] known-red probe still red via resolve/eval refuse (agreement — quarantine holds): {msg}"
                );
                ClaimResult {
                    function: format!("{label} (expect_red still-red OK)"),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    witness_row_costs: Vec::new(),
                    budget_refusal: None,
                }
            } else {
                ClaimResult {
                    function: label,
                    ok: false,
                    detail: format!("discovery corpus failed: {msg}"),
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    witness_row_costs: Vec::new(),
                    budget_refusal: None,
                }
            }
        }
    }
}

fn eval_plan_in_ctx(
    plan_ctx: &InterpContext,
    plan_entry: &str,
    plan_function: &str,
) -> Result<ParsedWalkPlan, String> {
    set_phase(FloorPhase::Gate, &format!("{plan_entry}::{plan_function}"));
    let plan_value = run_value(plan_ctx, plan_function).map_err(|msg| {
        format!(
            "plan eval failed ({}::{}): {}",
            plan_entry, plan_function, msg
        )
    })?;
    walk_plan_from_plan(&plan_value, plan_ctx)
        .map_err(|msg| format!("malformed plan value ({plan_function}): {msg}"))
}

fn eval_plan(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<ParsedWalkPlan, String> {
    let (plan_graph, plan_indices) = resolve_entry_graph(source_roots, plan_entry)
        .map_err(|msg| format!("resolve failed for plan {}:\n{}", plan_entry, msg))?;
    let plan_ctx = make_eval_context(&plan_graph, plan_indices, ExecutionMode::Hermetic);
    eval_plan_in_ctx(&plan_ctx, plan_entry, plan_function)
}

/// An infrastructure fault the walk observed directly, kept as data.
///
/// The walk KNOWS a claim thread panicked — `handle.join()` returned `Err`. That fact used
/// to survive only as the rendered line `"batch=N infra=thread_panic"`, which
/// `falsifier_failure_mode` then recovered by matching its own `"infra="` prefix: a typed
/// fact formatted into prose and grepped back out, the same round-trip the budget refusal
/// used to make. The panic PAYLOAD is genuinely lost (`Err(_)` discards it), but *that a
/// thread panicked* is not lost — so the classification reads this, and the rendered line
/// goes back to being for humans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InfraFault {
    ClaimThreadPanicked { batch_index: usize },
}

struct WalkOutcome {
    any_failed: bool,
    batches_run: usize,
    /// Failed claim details collected across the walk (for typed terminal classification).
    failure_details: Vec<String>,
    /// Infra faults observed structurally, never re-derived from `failure_details` text.
    infra_faults: Vec<InfraFault>,
}

/// Render a floor phase-completion line through the single-authority observation
/// renderer (`gunbc.observation_ci_render`, via the seed boundary
/// `gunbc.observation_seed_render`) instead of a raw `[t+…]` byte string in the seed
/// — the format lives in `.dag`, this only transports primitives across the boundary,
/// exactly as `cli_run::install_output_policy` calls `output_policy.resolve_channel_policy`.
/// The seed occurrence "a floor phase concluded in `elapsed_ms`" is modelled as a
/// `Concluded` event on a `PhaseSegment` subject and projected by
/// `ci_event_line ∘ ci_render_line`. Resolve is memoized in the process resolve store,
/// so the render module is resolved once and every later mark is a cache hit + a cheap
/// eval. Returns `None` only if the renderer cannot be resolved/evaluated; the caller
/// degrades loudly and never reproduces the old marker (§5: a failure arm refuses, it
/// does not widen). The entry is located under whichever source root holds it as an
/// absolute path, so resolution does not depend on the process CWD (production runs
/// from the repo root; `cargo test` runs from the crate dir).
fn render_phase_concluded_line(
    source_roots: &[String],
    phase: &str,
    elapsed_ms: u64,
    overhead_ms: u64,
    emoji: bool,
) -> Option<String> {
    let entry = source_roots
        .iter()
        .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
        .find(|p| p.exists())?
        .to_string_lossy()
        .into_owned();
    let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let out = run_in_context_with_args(
        &ctx,
        "phase_concluded_line",
        &[
            (Some("phase".to_string()), Value::Str(phase.to_string())),
            (
                Some("elapsed_ms".to_string()),
                Value::Int(elapsed_ms as i64),
            ),
            (
                Some("overhead_ms".to_string()),
                Value::Int(overhead_ms as i64),
            ),
            (Some("emoji".to_string()), Value::Bool(emoji)),
        ],
        false,
    )
    .ok()?;
    match out {
        Value::Str(s) => Some(s),
        _ => None,
    }
}

fn peak_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// Mirror of `gunbc.observation_ci_render.ci_minute_switch_seconds` — where the
/// sentence form switches from seconds to minutes (display policy, not a unit).
const CI_MINUTE_SWITCH_SECONDS: u64 = 90;

/// Mirror of `gunbc.observation_seed_render.seed_heartbeat_unreadable_cause`.
const SEED_HEARTBEAT_UNREADABLE_CAUSE: &str = "cgroup field unreadable";

/// Pure Rust mirror of `gunbc.observation_seed_render.seed_heartbeat_line` —
/// `ci_heartbeat_line ∘ ci_render_line` over the seed's real input space. The
/// heartbeat thread cannot call the interpreter (would build a duplicate module
/// index under the memory envelope it watches — DESIGN §2); this mirror is proven
/// byte-equal to the `.dag` oracle by `render_heartbeat_line_mirror_matches_seed_oracle`.
/// Subject is batch-grain only (parallel entries → no fabricated per-module detail).
fn render_heartbeat_line_mirror(
    elapsed_ms: u64,
    batch_label: &str,
    entry_index: u64,
    entry_total: u64,
    rss_bytes: Option<u64>,
    swap_bytes: Option<u64>,
    pressure_bp: Option<u64>,
    emoji: bool,
) -> String {
    let glyph = if emoji { "🕐" } else { "◷" };
    let duration = mirror_ci_human_duration(elapsed_ms);
    let rss = mirror_ci_measured_bytes(rss_bytes);
    let swap = mirror_ci_measured_bytes(swap_bytes);
    let pressure = mirror_ci_measured_percent(pressure_bp);
    format!(
        "{glyph} {duration} in — still in {batch_label}: entry {entry_index} of {entry_total}. memory {rss}, swap {swap}, pressure {pressure}"
    )
}

fn mirror_ci_human_duration(ms: u64) -> String {
    if ms < 1_000 {
        format!("{ms}ms")
    } else if ms < CI_MINUTE_SWITCH_SECONDS * 1_000 {
        format!("{} seconds", ms / 1_000)
    } else {
        format!("{} minutes", ms / 60_000)
    }
}

fn mirror_ci_tenths_text(tenths: u64) -> String {
    format!("{}.{}", tenths / 10, tenths % 10)
}

fn mirror_ci_human_bytes(bytes: u64) -> String {
    // Mirror of ci_gibibyte_tenths: (bytes * 10) / gibibyte_scale_factor_bytes (2^30).
    let tenths = (bytes.saturating_mul(10)) / 1_073_741_824;
    format!("{} GiB", mirror_ci_tenths_text(tenths))
}

fn mirror_ci_human_percent(bp: u64) -> String {
    // Mirror of ci_human_percent: tenths = bp / 10 → "9.0%".
    format!("{}%", mirror_ci_tenths_text(bp / 10))
}

fn mirror_ci_measured_bytes(v: Option<u64>) -> String {
    match v {
        Some(b) => mirror_ci_human_bytes(b),
        None => format!("unreadable ({SEED_HEARTBEAT_UNREADABLE_CAUSE})"),
    }
}

fn mirror_ci_measured_percent(v: Option<u64>) -> String {
    match v {
        Some(bp) => mirror_ci_human_percent(bp),
        None => format!("unreadable ({SEED_HEARTBEAT_UNREADABLE_CAUSE})"),
    }
}

/// PSI `avg10` is a percent with one decimal (e.g. `"9.01"`). One basis point is
/// 0.01 percentage points, so percent × 100 = bp (9.01 → 901) — the same scale
/// `std.observation` carries for `PsiPressure.avg10`.
fn psi_avg10_to_basis_points(avg10: &str) -> Option<u64> {
    let pct: f64 = avg10.parse().ok()?;
    if !pct.is_finite() || pct < 0.0 {
        return None;
    }
    Some((pct * 100.0).round() as u64)
}

fn batch_heartbeat_label(batch: &[Runnable]) -> String {
    if batch
        .iter()
        .any(|r| matches!(r, Runnable::DiscoveryBatch { .. }))
    {
        // Canonical crawl-window subject — matches the seed oracle's batch label.
        "witness discovery".to_string()
    } else if let Some(Runnable::SingleClaim { function, .. }) = batch.first() {
        function.clone()
    } else {
        "batch".to_string()
    }
}

/// Floor memory heartbeat — FIDELITY ONLY (reads state, changes no behavior; §5 stopped-line
/// analysis needs the line's memory story to exist in the log). The 2026-07-11 wedge was
/// invisible precisely here: `memory.high` throttles instead of killing, so the only log
/// evidence was a post-hoc `memory.peak` pinned at `high + <1MiB` after a 30-49min silent
/// tail. One synchronous regime-disclosure line at floor start (which limits bind, where),
/// then one line per minute from a detached thread (dies with the process), projected
/// through `render_heartbeat_line_mirror` — identity-first, human units, subject from the
/// HeartbeatFeed. The raw floor-memory byte dump (minute counter + raw current/swap
/// integers) is deleted (census negative example). Sampling denominator = the binding-high
/// dir when set (the slot slice that throttles), else the binding-cap dir, else the leaf
/// (whole-machine regimes); absence of all three refuses loudly and the floor proceeds
/// unmonitored — never a fabricated zero.
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
    let emoji = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");
    let spawned = std::thread::Builder::new()
        .name("floor-memory-heartbeat".into())
        .spawn(move || {
            let started = Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                // Skip until a batch has armed the feed with a real entry_total —
                // never a fabricated 0-of-0 during prelude / roster assembly.
                let Some(feed) = heartbeat_feed_snapshot() else {
                    continue;
                };
                let rss = read_cgroup_u64(&dir, "memory.current");
                let swap = read_cgroup_u64(&dir, "memory.swap.current");
                let pressure = read_cgroup_raw(&dir, "memory.pressure")
                    .and_then(|c| memory_pressure_some_avg10(&c))
                    .as_deref()
                    .and_then(psi_avg10_to_basis_points);
                let elapsed_ms = started.elapsed().as_millis() as u64;
                let line = render_heartbeat_line_mirror(
                    elapsed_ms,
                    &feed.batch_label,
                    feed.entry_done,
                    feed.entry_total,
                    rss,
                    swap,
                    pressure,
                    emoji,
                );
                eprintln!("{line}");
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
    let emoji = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");
    match cgroup_job_measurement() {
        Some(m) => {
            let label = format!("cgroup peak @ {} ({context})", m.leaf_rel);
            eprintln!(
                "{}",
                v1_compiler::cli_run::render_peak_rss_line_mirror(&label, Some(m.leaf_peak), emoji)
            );
            // Diagnostic companions (cap/pids/sccache) stay as Ambient detail beside the
            // Measured peak — not the old raw-byte `[measurement]` dump. Placement still
            // reads the typed `cgroup_job_measurement` fact, not this prose.
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
                "  memory.max={cap} host_ram={host_ram} pids_current={pc} pids_max={pm} sccache-server-cgroup={sccache}",
                pc = m.pids_current,
                pm = m.pids_max
            );
        }
        None => {
            let label = format!("cgroup peak ({context})");
            eprintln!(
                "{}",
                v1_compiler::cli_run::render_peak_rss_line_mirror_with_cause(
                    &label,
                    None,
                    "no leaf cgroup or memory.peak unreadable; kernel < 5.19?",
                    emoji,
                )
            );
        }
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
    /// The derived clamp computed for this batch (overhead + unit_count*rate, tightened), or None
    /// for budget-less plans (falsifier/regen). Recorded so the receipt never recomputes it.
    clamp_ms: Option<u128>,
    /// The runtime unit count the clamp used (discovery witnesses + gate rows).
    unit_count: u128,
    /// Flattened results from all units in this batch (order: unit by unit).
    results: Vec<ClaimResult>,
    /// Heartbeat label for this batch — carried so the component receipt names the
    /// component the same way the progress lines did.
    label: String,
    /// The batch's node-frontier selection role, the STRUCTURAL identity the component
    /// receipt keys on (`gunbc.floor_component_receipt` role note): the affected-set
    /// cold control is the `predict_only` component, never "batch 1" — indices shift
    /// when a `gunbc_falsifier_batches` enrollment flag flips.
    selection_tag: &'static str,
}

/// Materialization-ladder receipt: how many entry resolves this floor run actually
/// paid (walk_memo hits charge resolve_nanos == 0 and are excluded — this counts the
/// duplicated work across DISTINCT entries, the cross-entry share the memo cannot do).
/// Discovery corpus resolve time is reported on its own key, never folded into the
/// entry count (different grain; conflating them would mask either regression).
/// Consumed by the ci.yml resolve-receipt gate emitted from dag/gunbc/ci_materialization.dag.
/// Returns false on a write error — the walk fails closed at the point of
/// failure rather than relying only on the downstream missing-file gate.
/// The batch's node-frontier selection role, as the tag vocabulary
/// `gunbc.floor_component_receipt.floor_component_selection_of_tag` admits.
/// Two or more DISTINCT selections in one batch returns `"ambiguous"`, which that
/// function refuses — the role would not identify a unique component, and a silently
/// picked first one is exactly the kind of guess the receipt exists to remove.
fn batch_selection_tag(batch: &[Runnable]) -> &'static str {
    let mut seen: Vec<&'static str> = Vec::new();
    for r in batch {
        if let Runnable::DiscoveryBatch {
            node_frontier_selection,
            ..
        } = r
        {
            let tag = match node_frontier_selection {
                NodeFrontierSelectionMode::Off => "off",
                NodeFrontierSelectionMode::Applied => "applied",
                NodeFrontierSelectionMode::PredictOnly => "predict_only",
            };
            if !seen.contains(&tag) {
                seen.push(tag);
            }
        }
    }
    match seen.len() {
        0 => "off",
        1 => seen[0],
        _ => "ambiguous",
    }
}

/// One batch's failure MODE and detail — the seed's existing `falsifier_failure_mode`
/// vocabulary verbatim, with `"none"` for a clean batch. This function deliberately
/// makes NO judgment: the mode-to-`ObservationOutcome` mapping lives in
/// `gunbc.floor_component_receipt.floor_component_outcome_of_failure_mode`, under a
/// witness, and refuses an unmodelled mode. An earlier version of this mapped the mode
/// to an outcome tag here with a `_ => "failed"` arm, which would have absorbed a newly
/// added mode into `failed` with no diagnostic — a silent widen (DESIGN §5).
fn batch_failure_mode_and_detail(rec: &BatchRecord) -> (&'static str, String) {
    let details: Vec<String> = rec
        .results
        .iter()
        .filter(|r| !r.ok)
        .map(|r| format!("fn={} detail={}", r.function, r.detail))
        .collect();
    if details.is_empty() {
        return ("none", "no failures recorded".to_string());
    }
    let joined = details.join(" | ");
    // Read the budget refusal off the VALUE, never off the prose. `falsifier_failure_mode`
    // recovers "BudgetExceeded" by substring-matching the message the seed had already
    // formatted from the typed pair — one fact in two representations, the second guessed
    // back from the first — and its fallback arm is "WitnessRed", so rewording an error
    // silently demoted a budget refusal to a witness failure (two different remedies:
    // re-basis a dated ceiling vs. fix the witness). A structurally-known refusal wins
    // outright; the string classifier stays only for the paths that still arrive as
    // RuntimeError prose (interpreter-raised budgets, infra strings) and dissolves as
    // those are typed at their own seams.
    if rec.results.iter().any(|r| r.budget_refusal.is_some()) {
        return (
            "BudgetExceeded",
            joined.chars().take(600).collect::<String>(),
        );
    }
    (
        falsifier_failure_mode(&details),
        joined.chars().take(600).collect::<String>(),
    )
}

fn batch_witness_count(rec: &BatchRecord) -> u128 {
    let corpus: u128 = rec.results.iter().map(|r| r.corpus_witnesses as u128).sum();
    if corpus > 0 {
        corpus
    } else {
        rec.results.len() as u128
    }
}

/// Write the floor's per-component receipt, the machine-readable state
/// `.github/workflows/falsifier-alert.yml` reads instead of inferring one hardcoded
/// causal story from the overall workflow conclusion. Authority for shape and content
/// is `gunbc.floor_component_receipt`; this only transports primitives across the seed
/// boundary, exactly as `render_phase_concluded_line` does for progress lines. Returns
/// false on any refusal or write failure so the walk fails closed — a receipt that
/// silently did not appear is the blindness the alert defect was made of.
fn write_floor_component_receipt_at(
    base: &std::path::Path,
    source_roots: &[String],
    batch_records: &[BatchRecord],
    total_batches: usize,
) -> bool {
    let Some(entry) = source_roots
        .iter()
        .map(|r| Path::new(r).join("gunbc/floor_component_receipt.dag"))
        .find(|p| p.exists())
        .map(|p| p.to_string_lossy().into_owned())
    else {
        eprintln!(
            "claim_executor: floor component receipt REFUSED — gunbc/floor_component_receipt.dag \
             not found under any source root {source_roots:?}"
        );
        return false;
    };
    let (graph, indices) = match resolve_entry_graph_shared(source_roots, &entry) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("claim_executor: floor component receipt REFUSED — resolve {entry}: {e}");
            return false;
        }
    };
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let run_id = std::env::var("GITHUB_RUN_ID").unwrap_or_else(|_| "local".to_string());

    let mut rows: Vec<Value> = Vec::new();
    for rec in batch_records {
        let (failure_mode, detail) = batch_failure_mode_and_detail(rec);
        match floor_component_row_value(
            &ctx,
            &run_id,
            rec.batch_index as i64 + 1,
            &rec.label,
            rec.selection_tag,
            batch_witness_count(rec) as i64,
            failure_mode,
            &detail,
            (rec.wall_nanos / 1_000_000) as i64,
        ) {
            Some(v) => rows.push(v),
            None => return false,
        }
    }
    // Batches the stop policy never reached are Skipped with their cause — a named
    // state, never an absent row the alert would have to guess about.
    for bi in batch_records.len()..total_batches {
        match floor_component_row_value(
            &ctx,
            &run_id,
            bi as i64 + 1,
            "not reached",
            "off",
            0,
            "not_reached",
            "batch not reached — an earlier batch failed under the stop policy",
            0,
        ) {
            Some(v) => rows.push(v),
            None => return false,
        }
    }

    let doc = match run_in_context_with_args(
        &ctx,
        "floor_component_receipt_document",
        &[
            (Some("run_id".to_string()), Value::Str(run_id.clone())),
            (Some("rows".to_string()), Value::List(Rc::new(rows.into()))),
        ],
        false,
    ) {
        Ok(Value::Str(s)) => s,
        Ok(other) => {
            eprintln!(
                "claim_executor: floor component receipt REFUSED — \
                 floor_component_receipt_document returned {other:?}, not Str"
            );
            return false;
        }
        Err(e) => {
            eprintln!(
                "claim_executor: floor component receipt REFUSED — \
                 floor_component_receipt_document eval: {e}"
            );
            return false;
        }
    };

    let path = base.join("floor-component-receipt.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&path, doc) {
        Ok(()) => {
            eprintln!(
                "[receipt] floor component receipt: {} component(s) ({})",
                total_batches,
                path.display()
            );
            true
        }
        Err(e) => {
            eprintln!(
                "claim_executor: floor component receipt REFUSED — write {}: {e}",
                path.display()
            );
            false
        }
    }
}

/// One `FloorComponentRow`, built by the `.dag` authority from primitives. `None` is a
/// refusal the caller propagates: the only way the constructor returns absent is an
/// outcome or selection tag outside its vocabulary, which is a defect in this seed's
/// mapping and must stop the line rather than drop a component from the receipt.
#[allow(clippy::too_many_arguments)]
fn floor_component_row_value(
    ctx: &InterpContext,
    run_id: &str,
    index: i64,
    label: &str,
    selection_tag: &str,
    witnesses: i64,
    failure_mode: &str,
    detail: &str,
    wall_ms: i64,
) -> Option<Value> {
    // The duration crosses as the std.measure carrier, not a bare scalar: `millisecond`
    // is called across the boundary so the constructor stays the single authority. A
    // `Value::Record { type_name: "Millisecond", .. }` built here would fork it, and a
    // bare `wall_ms: Nat` parameter would be the flat-scalar unit the standing
    // unit-modeling hold forbids (`floor_component_receipt_unit_surface_note`).
    let wall = match run_in_context_with_args(
        ctx,
        "millisecond",
        &[(Some("count".to_string()), Value::Int(wall_ms))],
        false,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "claim_executor: floor component receipt REFUSED — millisecond(count: {wall_ms}) \
                 eval for batch {index}: {e}"
            );
            return None;
        }
    };
    let constructor = "floor_component_row_of_failure_mode";
    let args: Vec<(Option<String>, Value)> = vec![
        (Some("run_id".to_string()), Value::Str(run_id.to_string())),
        (Some("index".to_string()), Value::Int(index)),
        (Some("label".to_string()), Value::Str(label.to_string())),
        (
            Some("selection_tag".to_string()),
            Value::Str(selection_tag.to_string()),
        ),
        (Some("witnesses".to_string()), Value::Int(witnesses)),
        (
            Some("failure_mode".to_string()),
            Value::Str(failure_mode.to_string()),
        ),
        (Some("detail".to_string()), Value::Str(detail.to_string())),
        (Some("wall".to_string()), wall),
    ];
    let out = run_in_context_with_args(ctx, constructor, &args, false);
    match out {
        Ok(Value::Variant {
            ref variant_name,
            ref fields,
            ..
        }) if ctx.sym_eq(*variant_name, "Present") => fields
            .iter()
            .find(|(n, _)| ctx.sym_eq(*n, "value"))
            .map(|(_, v)| v.clone())
            .or_else(|| {
                eprintln!(
                    "claim_executor: floor component receipt REFUSED — \
                     Present row without a `value` field (batch {index})"
                );
                None
            }),
        Ok(Value::Null) => {
            eprintln!(
                "claim_executor: floor component receipt REFUSED — {constructor} returned \
                 absent for batch {index}: selection_tag={selection_tag:?} \
                 failure_mode={failure_mode:?} are outside the .dag vocabulary"
            );
            None
        }
        Ok(other) => {
            eprintln!(
                "claim_executor: floor component receipt REFUSED — {constructor} returned {other:?} for batch {index}"
            );
            None
        }
        Err(e) => {
            eprintln!(
                "claim_executor: floor component receipt REFUSED — {constructor} eval for batch {index}: {e}"
            );
            None
        }
    }
}

fn write_resolve_receipt(batch_records: &[BatchRecord]) -> bool {
    write_resolve_receipt_at(std::path::Path::new("target"), batch_records)
}

/// Per-batch wall receipt (THE COST WALL, Piece 3 derived clamp): typed rows — one
/// wall/units/clamp/verdict group per batch — so the floor's time story is a receipt, not a log
/// archaeology exercise. `OverBudget` rows correspond one-to-one with the FLOOR-BATCH-OVER-BUDGET
/// refusals the walk printed; a clamp-less run (falsifier/regen plans) records walls with
/// `verdict=Unbudgeted`. Returns false on a write error — the walk fails closed here.
fn write_batch_wall_receipt(batch_records: &[BatchRecord]) -> bool {
    write_batch_wall_receipt_at(std::path::Path::new("target"), batch_records)
}

fn write_batch_wall_receipt_at(base: &std::path::Path, batch_records: &[BatchRecord]) -> bool {
    let mut body = String::new();
    let mut over_budget = 0usize;
    for rec in batch_records {
        let n = rec.batch_index + 1;
        let wall_ms = rec.wall_nanos / 1_000_000;
        body.push_str(&format!("batch_{n}_wall_ms={wall_ms}\n"));
        match rec.clamp_ms {
            Some(clamp_ms) => {
                let verdict = if wall_ms > clamp_ms {
                    over_budget += 1;
                    "OverBudget"
                } else {
                    "WithinBudget"
                };
                body.push_str(&format!("batch_{n}_units={}\n", rec.unit_count));
                body.push_str(&format!("batch_{n}_clamp_ms={clamp_ms}\n"));
                body.push_str(&format!("batch_{n}_verdict={verdict}\n"));
            }
            None => {
                body.push_str(&format!("batch_{n}_verdict=Unbudgeted\n"));
            }
        }
    }
    body.push_str(&format!("over_budget_batches={over_budget}\n"));
    let path = base.join("floor-batch-wall-receipt.txt");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write batch-wall receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor batch walls: {} batch(es), {} over budget (receipt: {})",
        batch_records.len(),
        over_budget,
        path.display()
    );
    true
}

fn write_gate_warm_cost_receipt(batch_records: &[BatchRecord], emit_full_tsv_log: bool) -> bool {
    write_gate_warm_cost_receipt_at(
        std::path::Path::new("target"),
        batch_records,
        emit_full_tsv_log,
    )
}

/// Per-gate warm-cost TSV (D2 placement probe, ci-two-tier-placement-redesign §9.1): one row per
/// gate/claim carrying its warm eval wall, resolve time, and combined warm cost — the PLACEMENT
/// ROSTER's measurement basis. A gate rides PrTier only if its measured warm cost is within the
/// 5s fast-lane budget (else fail-closed to Gauntlet — v2.workflow.ci_placement). Denominated PER
/// GATE (placement is per-gate), reusing the ClaimResult timings the floor already records
/// (operator ruling 2026-07-24: instrument the existing floor, do not add a throwaway probe
/// workflow). For a single-claim gate `wall_nanos` IS the eval wall (== thread-CPU on its one
/// thread); the discovery row carries the per-witness rate (serial-sum eval over the witness
/// count — the parallel batch wall is the batch-wall receipt's, not a per-witness figure). The
/// probe reads a WARM run's rows; run cold-then-warm on >=2 hosts and the roster records value +
/// host basis. Fail-closed on a write error (shares target/ with the gated receipts, so a write
/// failure here is the same disk fault that fails them); never a verdict term.
fn write_gate_warm_cost_receipt_at(
    base: &std::path::Path,
    batch_records: &[BatchRecord],
    emit_full_tsv_log: bool,
) -> bool {
    let mut body =
        String::from("gate\tbatch\teval_ms\tresolve_ms\twarm_ms\twitnesses\ts_per_witness_us\n");
    for rec in batch_records {
        let n = rec.batch_index + 1;
        for result in &rec.results {
            if result.corpus_witnesses > 0 {
                let eval_ms = result.corpus_eval_nanos / 1_000_000;
                let resolve_ms = result.corpus_resolve_nanos / 1_000_000;
                let warm_ms = (result.corpus_eval_nanos + result.corpus_resolve_nanos) / 1_000_000;
                let per_witness_us =
                    result.corpus_eval_nanos / (result.corpus_witnesses as u128) / 1_000;
                body.push_str(&format!(
                    "discovery\t{n}\t{eval_ms}\t{resolve_ms}\t{warm_ms}\t{}\t{per_witness_us}\n",
                    result.corpus_witnesses
                ));
            } else {
                let eval_ms = result.wall_nanos / 1_000_000;
                let resolve_ms = result.resolve_nanos / 1_000_000;
                let warm_ms = (result.wall_nanos + result.resolve_nanos) / 1_000_000;
                body.push_str(&format!(
                    "{}\t{n}\t{eval_ms}\t{resolve_ms}\t{warm_ms}\t0\t0\n",
                    result.function
                ));
            }
        }
    }
    // The falsifier cadence mirrors the TSV into the log (prefixed, grep-collectable) so the
    // placement probe can lift it from get_job_logs without an artifact-upload step. Per-PR
    // runs keep only the summary below; the complete file remains available on every cadence.
    if emit_full_tsv_log {
        for line in body.lines() {
            eprintln!("[gate-warm-cost] {line}");
        }
    }
    let path = base.join("floor-gate-warm-cost-receipt.tsv");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write gate warm-cost receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor gate warm-cost: {} batch(es) (TSV receipt: {})",
        batch_records.len(),
        path.display()
    );
    true
}

/// Per-witness cost receipt (Piece #5 spine): one row per discovery witness preserving
/// `(entry, function, eval_ms, resolve_ms)` identity — the grain falsifier_cadence_surface_note
/// requires before per-row placement is admissible. The complete machine-readable record is
/// the TSV file; rendered streams may project a subset later (W2 ruling: one record, two
/// projections). Fail-closed on write error.
fn write_witness_row_cost_receipt(batch_records: &[BatchRecord], emit_full_tsv_log: bool) -> bool {
    write_witness_row_cost_receipt_at(
        std::path::Path::new("target"),
        batch_records,
        emit_full_tsv_log,
    )
}

fn write_witness_row_cost_receipt_at(
    base: &std::path::Path,
    batch_records: &[BatchRecord],
    emit_full_tsv_log: bool,
) -> bool {
    let mut body = String::from("batch\tentry\tfunction\teval_ms\tresolve_ms\twarm_ms\n");
    let mut row_count = 0usize;
    for rec in batch_records {
        let n = rec.batch_index + 1;
        for result in &rec.results {
            for row in &result.witness_row_costs {
                let eval_ms = row.2 / 1_000_000;
                let resolve_ms = row.3 / 1_000_000;
                let warm_ms = row.4 / 1_000_000;
                body.push_str(&format!(
                    "{n}\t{}\t{}\t{eval_ms}\t{resolve_ms}\t{warm_ms}\n",
                    row.0, row.1
                ));
                row_count += 1;
            }
        }
    }
    // Task #20's falsifier reproduction protocol consumes these grep-collectable lines. The
    // per-PR floor keeps the identical TSV file plus the summary below without duplicating the
    // full body on stderr.
    if emit_full_tsv_log {
        for line in body.lines() {
            eprintln!("[witness-row-cost] {line}");
        }
    }
    let path = base.join("floor-witness-row-cost-receipt.tsv");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write witness row-cost receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor witness row-cost: {row_count} row(s) (TSV receipt: {})",
        path.display()
    );
    true
}

/// Required `host_class` on every signed basis row (`witness_row_cost_basis_host_class_note`).
const WITNESS_ROW_COST_BASIS_HOST_CLASS: &str = "srv_fleet_arm64";

#[derive(Debug)]
struct WitnessRowCostBasisRow {
    eval_ms_basis: u128,
    run_ref: String,
}

/// Parse one TSV body line from `witness_row_cost_basis.tsv`.
/// Returns `Ok(None)` for blank/comment lines; `Err` for malformed or wrong-host-class rows
/// (caller must not insert them — a wrong host class would poison the 2× comparator).
fn parse_witness_row_cost_basis_line(
    line: &str,
) -> Result<Option<((String, String), WitnessRowCostBasisRow)>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 5 {
        return Err(format!(
            "malformed witness-row-cost basis line (need 5 cols: entry function eval_ms_basis run_ref host_class): {line}"
        ));
    }
    let host_class = parts[4];
    if host_class != WITNESS_ROW_COST_BASIS_HOST_CLASS {
        return Err(format!(
            "witness-row-cost basis row host_class={host_class:?} refused (required {WITNESS_ROW_COST_BASIS_HOST_CLASS}; wrong host class poisons the 2× comparator): {line}"
        ));
    }
    let eval_ms_basis = parts[2].parse::<u128>().unwrap_or(0);
    if eval_ms_basis == 0 {
        return Err(format!(
            "witness-row-cost basis row has zero eval_ms_basis (refused as basis): {line}"
        ));
    }
    Ok(Some((
        (parts[0].to_string(), parts[1].to_string()),
        WitnessRowCostBasisRow {
            eval_ms_basis,
            run_ref: parts[3].to_string(),
        },
    )))
}

fn millisecond_value(ctx: &InterpContext, count_ms: u128) -> Result<Value, String> {
    let count = i64::try_from(count_ms).map_err(|_| {
        format!("witness-row-cost: millisecond count {count_ms} exceeds i64 (fail-closed)")
    })?;
    match run_in_context_with_args(
        ctx,
        "millisecond",
        &[(Some("count".to_string()), Value::Int(count))],
        false,
    ) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("millisecond({count}): {e}")),
    }
}

/// Single-authority projection: evaluate `gunbc.witness_row_cost.witness_row_cost_exceeds_basis`
/// rather than re-implementing `observed > basis * 2` in the seed (review 43261).
fn witness_row_cost_exceeds_basis_via_authority(
    ctx: &InterpContext,
    observed_ms: u128,
    basis_ms: u128,
) -> Result<bool, String> {
    let observed = millisecond_value(ctx, observed_ms)?;
    let basis = millisecond_value(ctx, basis_ms)?;
    match run_in_context_with_args(
        ctx,
        "witness_row_cost_exceeds_basis",
        &[
            (Some("observed".to_string()), observed),
            (Some("basis".to_string()), basis),
        ],
        false,
    ) {
        Ok(Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "witness_row_cost_exceeds_basis returned {other}, expected Bool (fail-closed)"
        )),
        Err(e) => Err(format!("witness_row_cost_exceeds_basis: {e}")),
    }
}

/// Drift comparison on the falsifier cadence only (margin ruling: row grew >2× against its
/// dated basis = counted drift receipt, never merge-refusing). A row with no dated basis is
/// BasisAbsent — typed, located, counted; never assume fine. Comparator authority is
/// `dag/gunbc/witness_row_cost.dag` (resolved once; per-row exceeds_basis calls).
fn write_witness_row_cost_drift_receipt_at(
    base: &std::path::Path,
    batch_records: &[BatchRecord],
    basis_path: &std::path::Path,
    source_roots: &[String],
) -> bool {
    let entry = "dag/gunbc/witness_row_cost.dag";
    let (graph, indices) = match resolve_entry_graph(source_roots, entry) {
        Ok(v) => v,
        Err(m) => {
            eprintln!(
                "claim_executor: failed to resolve {entry} for drift comparator (fail-closed):\n{m}"
            );
            return false;
        }
    };
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);

    let mut basis: std::collections::HashMap<(String, String), WitnessRowCostBasisRow> =
        std::collections::HashMap::new();
    if let Ok(text) = std::fs::read_to_string(basis_path) {
        for line in text.lines().skip(1) {
            match parse_witness_row_cost_basis_line(line) {
                Ok(None) => {}
                Ok(Some((key, row))) => {
                    basis.insert(key, row);
                }
                Err(msg) => {
                    // Skip — never insert a wrong-host / malformed / zero-eval row
                    // (would poison the 2× comparator; review 43284). Loud + counted via log.
                    eprintln!("claim_executor: {msg}");
                }
            }
        }
    } else {
        eprintln!(
            "claim_executor: witness-row-cost basis file missing at {} — every row records BasisAbsent",
            basis_path.display()
        );
    }

    let mut body =
        String::from("batch\tentry\tfunction\tobserved_eval_ms\tbasis_eval_ms\tverdict\trun_ref\n");
    let mut drift_count = 0usize;
    let mut basis_absent_count = 0usize;
    for rec in batch_records {
        let n = rec.batch_index + 1;
        for result in &rec.results {
            for row in &result.witness_row_costs {
                let observed = row.2 / 1_000_000;
                let key = (row.0.clone(), row.1.clone());
                match basis.get(&key) {
                    None => {
                        basis_absent_count += 1;
                        body.push_str(&format!(
                            "{n}\t{}\t{}\t{observed}\t\tBasisAbsent\t\n",
                            row.0, row.1
                        ));
                    }
                    Some(b) => {
                        let exceeds = match witness_row_cost_exceeds_basis_via_authority(
                            &ctx,
                            observed,
                            b.eval_ms_basis,
                        ) {
                            Ok(v) => v,
                            Err(e) => {
                                eprintln!(
                                    "claim_executor: drift comparator refused for {}::{}: {e} — walk fails closed here",
                                    row.0, row.1
                                );
                                return false;
                            }
                        };
                        let verdict = if exceeds {
                            drift_count += 1;
                            "DriftExceeded"
                        } else {
                            "WithinBasis"
                        };
                        body.push_str(&format!(
                            "{n}\t{}\t{}\t{observed}\t{}\t{verdict}\t{}\n",
                            row.0, row.1, b.eval_ms_basis, b.run_ref
                        ));
                    }
                }
            }
        }
    }
    eprintln!(
        "[witness-row-cost-drift] basis_absent={basis_absent_count} drift_exceeded={drift_count}"
    );
    for line in body.lines() {
        eprintln!("[witness-row-cost-drift] {line}");
    }
    let path = base.join("floor-witness-row-cost-drift-receipt.tsv");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write witness row-cost drift receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor witness row-cost drift: basis_absent={basis_absent_count} drift_exceeded={drift_count} (TSV: {})",
        path.display()
    );
    true
}

/// ONE stage's own receipt, written before the next stage begins. The aggregate receipt
/// below cannot substitute for it: written only after the whole sequence, it does not
/// exist for stage N while stage N+1 is running, and a process death between the two
/// loses every stage that had in fact completed. Per-stage, the disk state answers
/// "which stages ran, and what did each cost" at any instant.
///
/// SCAFFOLD (§7 seed-retained HAND-RUST — authority: `std.types` `path_segment_is_safe`,
/// the single law for "safe as ONE path segment"). This mirrors that predicate clause for
/// clause because the executor is the Rust seed and cannot call the `.dag` surface at the
/// point it observes the environment. It is a REALIZATION of that authority, not a second
/// rule: if the two ever disagree, the `.dag` predicate is right and this is the defect.
/// dissolve-on: the executor's env observation running as a modeled effect, at which point
/// the branding constructor `gunbc.merge_admission.walk_attempt_id` is the only gate and
/// this function deletes.
fn walk_attempt_id_segment_is_safe(raw: &str) -> bool {
    !(raw.is_empty()
        || raw == "."
        || raw == ".."
        || raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\n')
        || raw.contains('\r')
        || raw.contains('\0'))
}

/// Observe the walk-attempt identity, per `gunbc.merge_admission`
/// `merge_admission_attempt_scope_note`: derived from GITHUB_RUN_ID + GITHUB_RUN_ATTEMPT +
/// GITHUB_JOB on GitHub, or supplied explicitly as GUNBC_WALK_ATTEMPT_ID off it.
///
/// REFUSES rather than defaulting. The ruling names the exact failure this prevents: "never
/// a silent constant like a bare local, which would make every local run one attempt and
/// the wrong-attempt refusal unreachable off CI". A default here would not be a convenience,
/// it would disable the identity check everywhere except GitHub — the absorbing fallback in
/// its purest form, since nothing would ever report that identity had been fabricated.
///
/// GUNBC_WALK_ATTEMPT_ID is NOT an escape hatch: it supplies a required input that the
/// environment did not, and a value that fails the segment law still refuses. There is no
/// value of it that makes a refusal not fire.
fn observe_walk_attempt_id() -> Result<String, String> {
    compose_walk_attempt_id(
        &std::env::var("GUNBC_WALK_ATTEMPT_ID").unwrap_or_default(),
        &std::env::var("GITHUB_RUN_ID").unwrap_or_default(),
        &std::env::var("GITHUB_RUN_ATTEMPT").unwrap_or_default(),
        &std::env::var("GITHUB_JOB").unwrap_or_default(),
    )
}

/// The PURE half, split from the observation above for the same reason
/// `gunbc.merge_admission_produce` splits them ("Pure composition of the walk-attempt
/// identity from its parts; the ENV OBSERVATION lives with the wet entry"). It is also what
/// makes the refusals reachable by a test: process env is global, so a test that set it
/// would race every other test in the binary, and a rule that can only be exercised by a
/// racing test is a rule nobody checks.
fn compose_walk_attempt_id(
    explicit: &str,
    run_id: &str,
    run_attempt: &str,
    job: &str,
) -> Result<String, String> {
    if !explicit.trim().is_empty() {
        return if walk_attempt_id_segment_is_safe(explicit) {
            Ok(explicit.to_string())
        } else {
            Err(format!(
                "GUNBC_WALK_ATTEMPT_ID={explicit:?} is not a safe path segment (std.types path_segment_is_safe: non-empty, not `.`/`..`, no `/` `\\` CR LF NUL)"
            ))
        };
    }
    if run_id.is_empty() || run_attempt.is_empty() || job.is_empty() {
        return Err(
            "no walk-attempt identity: GITHUB_RUN_ID/GITHUB_RUN_ATTEMPT/GITHUB_JOB are not all \
             present and GUNBC_WALK_ATTEMPT_ID was not supplied. On-success stages write \
             attempt-scoped receipts, so an unidentified walk refuses here rather than \
             stamping a receipt no consumer could tell apart from another run's"
                .to_string(),
        );
    }
    let composed = format!("{run_id}-{run_attempt}-{job}");
    if walk_attempt_id_segment_is_safe(&composed) {
        Ok(composed)
    } else {
        Err(format!(
            "composed walk-attempt identity {composed:?} is not a safe path segment (std.types path_segment_is_safe)"
        ))
    }
}

fn write_on_success_stage_receipt(
    stage_index: usize,
    passed: bool,
    run: &StageRun,
    declared_stage_count: usize,
    attempt_id: &str,
) -> bool {
    let mut resolves: u64 = 0;
    for r in &run.results {
        if r.resolve_nanos > 0 {
            resolves += 1;
        }
    }
    let mut body = String::new();
    // IDENTITY FIRST, IN THE PAYLOAD. `gunbc.merge_admission`
    // `merge_admission_attempt_scope_note`: "the receipt carries the walk-attempt identity
    // in the PAYLOAD, not just the path ... path identity alone is not enough, because a
    // misrouted read must fail on the content too". A receipt from another attempt is not
    // stale-or-fresh, it is NOT THE SUBJECT, and a consumer that reads this file by path
    // must be able to discover that from the bytes it just read.
    body.push_str(&format!("attempt_id={attempt_id}\n"));
    body.push_str(&format!("stage_index={}\n", stage_index + 1));
    body.push_str(&format!("stage_count={declared_stage_count}\n"));
    body.push_str(&format!(
        "outcome={}\n",
        if passed { "passed" } else { "failed" }
    ));
    body.push_str(&format!("claims={}\n", run.results.len()));
    body.push_str(&format!("unit_count={}\n", run.unit_count));
    body.push_str(&format!("resolves={resolves}\n"));
    body.push_str(&format!("wall_ms={}\n", run.wall_nanos / 1_000_000));
    // Recorded rather than omitted so a future DECLARED stage clamp shows up as a value
    // change here instead of a new field a reader has to notice.
    body.push_str(&format!(
        "clamp_ms={}\n",
        match run.clamp_ms {
            Some(ms) => ms.to_string(),
            None => "none".to_string(),
        }
    ));
    for r in &run.results {
        body.push_str(&format!(
            "claim\t{}\t{}\t{}\n",
            r.function,
            if r.ok { "passed" } else { "failed" },
            r.wall_nanos / 1_000_000
        ));
    }
    // Path scoping is HYGIENE, not the wall — the same ruling is explicit that "correctness
    // never depends on cleaning a shared path". It earns its keep by keeping two attempts on
    // one reused workspace (self-hosted runners do reuse `target/`) from overwriting each
    // other's evidence; the payload check above is what makes a misroute detectable.
    let base = std::path::Path::new("target").join(format!("floor-attempt-{attempt_id}"));
    let base = base.as_path();
    let path = base.join(format!("on-success-stage-{}-receipt.tsv", stage_index + 1));
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write on-success stage {} receipt {}: {e} — stage fails closed here",
            stage_index + 1,
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] on-success stage {}: {} claim(s), {} resolve(s), {}ms (receipt: {})",
        stage_index + 1,
        run.results.len(),
        resolves,
        run.wall_nanos / 1_000_000,
        path.display()
    );
    true
}

/// The on-success stage receipt — a SEPARATE receipt class from the ordinary floor
/// receipts, by lifecycle (ruling 2026-07-30): stages run only after the ordinary
/// receipts finalized, so their resolves and walls are not part of the population
/// `ci_floor_declared_resolve_count` measures, and folding them in would re-open the
/// count-collision this separation exists to close. On a skip (ordinary floor failed)
/// the receipt still writes, LOUDLY, with skipped=ordinary_floor_failed and zero
/// stages run — a typed diagnostic, never an admission artifact.
fn write_on_success_receipt(
    stage_rows: &[(usize, bool)],
    resolves_total: u64,
    skipped_ordinary_failed: bool,
    declared_stage_count: usize,
) -> bool {
    let mut body = String::new();
    if skipped_ordinary_failed {
        body.push_str("skipped=ordinary_floor_failed\n");
    }
    body.push_str(&format!(
        "on_success_stages_declared={declared_stage_count}\n"
    ));
    body.push_str(&format!("on_success_stages_run={}\n", stage_rows.len()));
    body.push_str(&format!("on_success_resolves_total={resolves_total}\n"));
    for (stage_index, passed) in stage_rows {
        body.push_str(&format!(
            "on_success_stage_{}={}\n",
            stage_index + 1,
            if *passed { "passed" } else { "failed" }
        ));
    }
    let base = std::path::Path::new("target");
    let path = base.join("floor-on-success-receipt.txt");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write on-success receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] on-success stages: {} of {} run, {} resolve(s) (receipt: {})",
        stage_rows.len(),
        declared_stage_count,
        resolves_total,
        path.display()
    );
    true
}

fn write_resolve_receipt_at(base: &std::path::Path, batch_records: &[BatchRecord]) -> bool {
    let mut resolves_total: u64 = 0;
    let mut resolve_ms_total: u128 = 0;
    let mut discovery_corpus_resolve_ms: u128 = 0;
    let mut discovery_corpus_eval_ms: u128 = 0;
    for rec in batch_records {
        for result in &rec.results {
            if result.resolve_nanos > 0 {
                resolves_total += 1;
                resolve_ms_total += result.resolve_nanos / 1_000_000;
            }
            discovery_corpus_resolve_ms += result.corpus_resolve_nanos / 1_000_000;
            discovery_corpus_eval_ms += result.corpus_eval_nanos / 1_000_000;
        }
    }
    let discovery_phases = v1_compiler::cli_run::take_discovery_phase_totals_receipt_rows();
    let body = format!(
        "resolves_total={resolves_total}\nresolve_ms_total={resolve_ms_total}\ndiscovery_corpus_resolve_ms={discovery_corpus_resolve_ms}\ndiscovery_corpus_eval_ms={discovery_corpus_eval_ms}\n{discovery_phases}"
    );
    let path = base.join("floor-resolve-receipt.txt");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
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

/// Emit a fractal post-walk tree to stderr when GUNBC_FLOOR_GANTT=1.
/// Wired (gantt flip): each row is a PhaseSegment Concluded projection via the
/// observation mirror — the raw `[gantt] … wall: {}ms` key=value tree is gone.
fn emit_gantt(batch_records: &[BatchRecord], total_wall_nanos: u128) {
    let gantt_enabled = std::env::var("GUNBC_FLOOR_GANTT")
        .map(|v| v == "1")
        .unwrap_or(false);
    if !gantt_enabled {
        return;
    }
    let emoji = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");
    let total_ms = (total_wall_nanos / 1_000_000) as u64;
    eprintln!(
        "{}",
        v1_compiler::v1_rt::render_phase_concluded_line_mirror("claim_executor", total_ms, emoji)
    );
    for rec in batch_records {
        let batch_ms = (rec.wall_nanos / 1_000_000) as u64;
        let batch_label = format!("batch {}", rec.batch_index + 1);
        eprintln!(
            "{}",
            v1_compiler::v1_rt::render_phase_concluded_line_mirror(&batch_label, batch_ms, emoji)
        );
        for result in &rec.results {
            if result.corpus_witnesses > 0 {
                let corpus_resolve_ms = (result.corpus_resolve_nanos / 1_000_000) as u64;
                let corpus_eval_ms = (result.corpus_eval_nanos / 1_000_000) as u64;
                let name = format!(
                    "{} ({} witnesses)",
                    result.function, result.corpus_witnesses
                );
                eprintln!(
                    "{}",
                    v1_compiler::v1_rt::render_phase_concluded_line_mirror(
                        &name,
                        corpus_resolve_ms + corpus_eval_ms,
                        emoji
                    )
                );
                eprintln!(
                    "{}",
                    v1_compiler::v1_rt::render_phase_concluded_line_mirror(
                        &format!("{}.resolve", result.function),
                        corpus_resolve_ms,
                        emoji
                    )
                );
                eprintln!(
                    "{}",
                    v1_compiler::v1_rt::render_phase_concluded_line_mirror(
                        &format!("{}.eval", result.function),
                        corpus_eval_ms,
                        emoji
                    )
                );
            } else {
                if result.resolve_nanos > 0 {
                    let resolve_ms = (result.resolve_nanos / 1_000_000) as u64;
                    eprintln!(
                        "{}",
                        v1_compiler::v1_rt::render_phase_concluded_line_mirror(
                            "resolve (entry)",
                            resolve_ms,
                            emoji
                        )
                    );
                }
                let wall_ms = (result.wall_nanos / 1_000_000) as u64;
                let label = if result.ok {
                    result.function.clone()
                } else {
                    format!("{} [FAIL]", result.function)
                };
                eprintln!(
                    "{}",
                    v1_compiler::v1_rt::render_phase_concluded_line_mirror(&label, wall_ms, emoji)
                );
            }
        }
    }
}

/// The (entry, execution_mode) pairs some runnable resolves on the memo path (heavy
/// whole-tree profile) in ANY batch of the walk. The partition routes every
/// SharedClaims group matching one of these keys to the memo path too — negligible
/// profile or not — so one entry resolves exactly once per walk. Derived from the
/// plan's own declared profiles; no schedule fact is added or reordered here.
fn memo_path_entry_keys(
    batches: &[Vec<Runnable>],
) -> std::collections::HashSet<(String, ExecutionMode)> {
    batches
        .iter()
        .flatten()
        .filter_map(|r| match r {
            Runnable::SingleClaim { entry, profile, .. }
                if profile.heavy_whole_tree_resolve && !entry.is_empty() =>
            {
                Some((entry.clone(), profile.execution_mode))
            }
            _ => None,
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnitLane {
    Memo,
    MainThread,
    Spawned,
}

/// Lane decision for one resolve-group. Memo lane (main thread, shared
/// `walk_memo`/`process_shared_index`) if (a) the group's profile declares heavy
/// whole-tree resolve, (b) its entry is already in `walk_memo` from a prior batch,
/// or (c) some unit in ANY batch resolves the same (entry, mode) on the memo path
/// (`memo_path_entry_keys`) — one entry, one resolve, across batches. Discovery
/// pumps run on the MAIN thread: `process_shared_index` is thread-local (Rc-based,
/// !Send), and the eagerly-installed compile-clean receipt (see the install before
/// run_walk) warmed THIS thread's index — routing the pump here is what lets
/// batch-2's witness resolves read the gate's content-keyed typed store instead of
/// re-typechecking the tree (lever 1, PR #6766). No wall-clock loss: the main
/// thread previously idled at the join while the one Discovery unit ran on a
/// spawned thread. Everything else spawns.
fn batch_unit_lane(
    unit: &BatchUnit,
    walk_memo: &std::collections::HashMap<(String, ExecutionMode), InterpContext>,
    memo_path_entries: &std::collections::HashSet<(String, ExecutionMode)>,
) -> UnitLane {
    match unit {
        BatchUnit::SharedClaims {
            use_walk_memo: true,
            ..
        } => UnitLane::Memo,
        BatchUnit::SharedClaims {
            entry,
            execution_mode,
            ..
        } if walk_memo.contains_key(&(entry.clone(), *execution_mode))
            || memo_path_entries.contains(&(entry.clone(), *execution_mode)) =>
        {
            UnitLane::Memo
        }
        BatchUnit::Discovery { .. } => UnitLane::MainThread,
        _ => UnitLane::Spawned,
    }
}

/// Arm-time admissibility of the success-stage population. Called immediately after
/// the plan parses, BEFORE the governor arms and before any ordinary batch runs: a
/// plan-shape error is knowable at parse time, and discovering it after a 20-30 minute
/// floor would spend the whole walk to report something the parse already had in hand.
///
/// Stages admit `RunnableSingleClaim` only. A discovery batch has no defined green-only
/// meaning, and — until stages execute through the ordinary unit-lane partition — a
/// Substantial or host-compiler-spawning claim would run outside governor admission and
/// batch clamps, so those profiles refuse here rather than running through a weaker
/// route (the executor gap is named on `std.realization_schedule.walk_plan_note`).
fn validate_on_success_stage_admissibility(stages: &[Vec<Runnable>]) -> Vec<String> {
    let mut refusals = Vec::new();
    for (si, stage) in stages.iter().enumerate() {
        for runnable in stage {
            match runnable {
                Runnable::SingleClaim {
                    entry,
                    function,
                    profile,
                } => {
                    if entry.trim().is_empty() || function.trim().is_empty() {
                        refusals.push(format!(
                            "on-success stage {} declares a claim with an empty entry or function",
                            si + 1
                        ));
                    }
                    // THE PROFILE WALL. Two distinct classes refuse, for two distinct
                    // reasons; neither is the reason an earlier draft of this function
                    // gave, and the correction matters because it changed the code.
                    //
                    // (1) HEAVY WHOLE-TREE RESOLVE still refuses, and the claim that
                    // run_stage made this safe was WRONG (review 2026-07-31). The
                    // reasoning was "stages now go through the same lane partition, so a
                    // heavy claim takes the memo lane exactly as it would in a batch."
                    // The partition is shared; the ADMISSION is not. `batch_unit_lane`
                    // routes a heavy unit — and every unit sharing its entry — to
                    // UnitLane::Memo, and the memo lane runs through
                    // `run_memo_shared_claims`, which takes no governor and acquires no
                    // AdmittedSlot. Only `run_batch_unit`, on the spawned lane, does
                    // (the single acquire_blocking site in this file). So a heavy stage
                    // claim would resolve and evaluate on the main thread, unadmitted,
                    // while spawned units hold slots — exactly the unbounded stacking
                    // the governor exists to prevent.
                    //
                    // Wrapping the memo call in an ordinary slot would NOT be the fix
                    // either: the slot would release while the resolved InterpContext
                    // stays resident in `stage_memo` for every later stage.
                    //
                    // AND THE OBVIOUS REPAIR — hold an AdmittedSlot for the memoized
                    // context's lifetime, released on drop — DEADLOCKS. An earlier draft
                    // of this comment named exactly that as "the real fix", which
                    // understated it in the same direction as the mistake above
                    // (operator review 2026-07-31, probed against `decide_admission`).
                    // `AdmittedSlot` is a CONCURRENCY slot, not a memory reservation:
                    // it increments `active`, which is compared against `target_width`.
                    // A resident hold pins `active >= 1` forever, so the progress floor
                    // (`active == 0` admits unconditionally) never fires and every later
                    // admission returns Hold(WindowFull). `target_width` only grows in
                    // `note_completion`, which needs a completion, which needs an
                    // admission — nothing breaks the cycle. This is not a corner case:
                    // the runner starts at `target_width=1`. A resident hold that also
                    // skips `note_first_cost_paid` leaves `undigested > 0`, which holds
                    // admissions even after width grows.
                    //
                    // So the dissolve-on is NOT "a lease" as a standalone change. It is
                    // SPLITTING the governor's single `active` counter into two
                    // resources — an execution slot (paced, width-bounded) and a
                    // resident memory reservation (counted against the memory budget,
                    // NOT against width) — after which a lease is expressible. That
                    // touches the path standing between the floor and the exit-137 OOM
                    // kills, so it is its own work with its own receipt, not a step
                    // inside a feature lane. Until then this refuses.
                    // (0) NO PROFILE AT ALL refuses first: the values below would be the
                    // parse's fail-closed fillers, not the plan's statements, and reading
                    // a wall off invented facts is worse than having no wall.
                    if matches!(profile.provenance, ParsedProfileProvenance::Undeclared) {
                        refusals.push(format!(
                            "on-success stage {} claim {} carries no resource profile — \
                             stages admit only claims whose resources the plan declared; \
                             an absent profile is not a negligible one",
                            si + 1,
                            function
                        ));
                    }
                    if profile.heavy_whole_tree_resolve {
                        refusals.push(format!(
                            "on-success stage {} claim {} declares a heavy whole-tree resolve — \
                             the memo lane it would take runs unadmitted (no AdmittedSlot), so \
                             it refuses rather than resolving a whole tree outside governor \
                             admission; dissolves on a resident lease tied to the memoized \
                             context's lifetime",
                            si + 1,
                            function
                        ));
                    }
                    // (2) HOST-COMPILER OR SUBSTANTIAL RESIDENCY refuses because stages
                    // carry no declared cost clamp — `gunbc_ci_floor_batch_clamp_params`
                    // indexes the ORDINARY batches — so such a claim would run unclamped
                    // until the outer workflow timeout. This wall is newly expressible:
                    // the parse used to discard both facts.
                    if profile.is_substantial_or_spawns_compiler() {
                        refusals.push(format!(
                            "on-success stage {} claim {} declares spawns_host_compiler={} \
                             memory={:?} — stages carry no declared cost clamp, so a \
                             host-compiler or substantial-residency claim refuses rather \
                             than running unclamped",
                            si + 1,
                            function,
                            profile.spawns_host_compiler,
                            profile.memory
                        ));
                    }
                }
                Runnable::DiscoveryBatch { .. } => refusals.push(format!(
                    "on-success stage {} contains a RunnableDiscoveryBatch — stages admit single \
                     claims only; a discovery batch has no defined green-only meaning",
                    si + 1
                )),
            }
        }
    }
    refusals
}

/// The floor plan's finalization laws, carried in the PARSED PLAN VALUE
/// (`WalkPlan.finalization`, authority `gunbc.ci_materialization.FloorFinalization`)
/// and enforced INSIDE the walk as ordinary-floor finalization (ruling 2026-07-30).
/// These were two GitHub shell steps — the resolve-receipt gate and the
/// materialization-receipt gate — which ran AFTER the floor step, so a receipt-law
/// violation could red the job after admission had already stamped. In here, a
/// violation is an ordinary-floor failure and blocks every on-success stage.
///
/// BOTH laws are intrinsic: inhabiting `FloorFinalization` obligates the resolve count
/// AND materialization disclosure. The disclosure Bool that used to sit beside the
/// count is gone — it was a writable bypass whose `false` arm skipped the
/// materialization check while the success line still reported that disclosure held
/// (review 2026-07-30). A plan DECLARES that it carries these laws by returning
/// `WalkPlan<FloorFinalization>` where regen/falsifier/plan-artifact return
/// `WalkPlan<NoWalkFinalization>` — a declaration, not a guarantee: the typechecker
/// does not check return position, so the value is what decides, and the enrolled
/// witnesses in v2.test.claim.ci_floor_plan_witness are what check the value.
struct FloorFinalization {
    declared_resolve_count: i64,
}

/// Validate the floor's finalization laws against the walk's own records. Returns the
/// list of typed refusals (empty = contract satisfied). The receipt FILES keep being
/// written by the walk — they are observability — this validates the laws the deleted
/// shell steps used to re-derive from those files.
///
/// `plan_site` is the `<entry>::<function>` the value was read from, so a refusal
/// locates the field it actually consumed rather than always pointing at the
/// production authority — the fixture and any future plan declare their own counts.
fn validate_floor_finalization(
    fin: &FloorFinalization,
    plan_site: &str,
    batch_records: &[BatchRecord],
) -> Vec<String> {
    let mut refusals = Vec::new();
    // Law 1 — the declared cold-resolve count (gunbc.ci_materialization
    // ci_floor_resolve_receipt_note): resolves_total is recomputed from the SAME rule
    // write_resolve_receipt_at uses, never re-parsed from the file we just wrote.
    let mut resolves_total: u64 = 0;
    for rec in batch_records {
        for result in &rec.results {
            if result.resolve_nanos > 0 {
                resolves_total += 1;
            }
        }
    }
    if resolves_total as i64 != fin.declared_resolve_count {
        refusals.push(format!(
            "floor resolve count {} differs from declared {}: duplicate-computation debt \
             changed - update the count this plan declared, at \
             {plan_site} WalkPlan.finalization.declared_resolve_count \
             (the production floor projects it from gunbc.ci_materialization \
             ci_floor_declared_resolve_count; a fixture or another plan declares its own)",
            resolves_total, fin.declared_resolve_count
        ));
    }
    // Law 2 — materialization disclosure (ci_floor_materialization_receipt_note):
    // receipt exists, keyed/unkeyed/duplicated parse, keyed nonzero. Read from the
    // file because the accumulator is process-global and already harvested into it.
    // Unconditional: disclosure is intrinsic to FloorFinalization, so there is no
    // early return that could skip this law while the caller reports it held.
    let path = std::path::Path::new("target/floor-materialization-receipt.txt");
    match std::fs::read_to_string(path) {
        Err(_) => refusals.push("floor materialization receipt missing - fail closed".to_string()),
        Ok(body) => {
            let field = |key: &str| -> Option<u64> {
                body.lines()
                    .find_map(|l| l.strip_prefix(key))
                    .and_then(|v| v.trim().parse::<u64>().ok())
            };
            match (
                field("keyed_calls="),
                field("unkeyed_calls="),
                field("duplicated_keys="),
            ) {
                (Some(k), Some(_u), Some(_d)) => {
                    if k == 0 {
                        refusals.push(
                            "floor evaluated zero keyed calls - ledger disabled or floor empty - \
                             fail closed"
                                .to_string(),
                        );
                    }
                }
                _ => refusals
                    .push("floor materialization receipt malformed - fail closed".to_string()),
            }
        }
    }
    refusals
}

/// THE CONCURRENCY PRIMITIVE, extracted so the contract is reachable by a test.
///
/// `walk_plan_note` states that members WITHIN a stage run concurrently — that is why
/// anything sequential must be one claim whose body sequences its steps, or two
/// singleton stages. While the spawn and the join were inlined in the batch loop, that
/// promise was checkable only by reading the code, and the first attempt at success
/// stages promised it while executing serially. Split out, the two halves each get a
/// latch control: members really do overlap, and the join really does wait for all of
/// them.
///
/// The join half is what makes the stage BARRIER real. Stage N+1 cannot begin early
/// because `run_walk`'s stage loop is sequential by construction — each iteration takes
/// `&mut stage_memo`, so two iterations cannot overlap — which reduces "stage N+1 waits
/// for every stage-N member" to "this join returns only after every member completed".
fn spawn_units(
    work: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>>,
) -> Vec<thread::JoinHandle<Vec<ClaimResult>>> {
    work.into_iter()
        .map(|w| thread::spawn(move || w()))
        .collect()
}

/// Join every spawned unit, returning its results and whether any thread panicked. A
/// panic is collected rather than propagated so the caller can still close its
/// host-effect group and report the fault as infra rather than as a claim verdict.
fn join_units(handles: Vec<thread::JoinHandle<Vec<ClaimResult>>>) -> (Vec<ClaimResult>, bool) {
    let mut results = Vec::new();
    let mut panicked = false;
    for handle in handles {
        match handle.join() {
            Ok(unit_results) => results.extend(unit_results),
            Err(_) => panicked = true,
        }
    }
    (results, panicked)
}

/// Which of the walk's two populations a stage belongs to. It selects LABELS only —
/// the execution path below is byte-identical for both, which is the entire point of
/// the extraction (`std.realization_schedule.walk_plan_note`). Ordering and failure
/// policy live in the callers, where the two populations genuinely differ.
#[derive(Clone, Copy, PartialEq, Eq)]
enum StagePopulation {
    OrdinaryBatch,
    OnSuccessStage,
}

impl StagePopulation {
    /// Prose label used in progress lines and PASS/FAIL prefixes.
    fn label(self) -> &'static str {
        match self {
            StagePopulation::OrdinaryBatch => "batch",
            StagePopulation::OnSuccessStage => "on-success stage",
        }
    }

    /// Hyphenated form for phase names, which carry no spaces.
    fn phase_slug(self) -> &'static str {
        match self {
            StagePopulation::OrdinaryBatch => "batch",
            StagePopulation::OnSuccessStage => "on-success-stage",
        }
    }
}

/// What one stage produced. Verdicts are deliberately NOT classified here: the caller
/// decides what a failure means, because that is precisely where the two populations
/// differ (`FloorBatchStopPolicy` for the ordinary floor, unconditional fail-fast
/// between stages).
struct StageRun {
    results: Vec<ClaimResult>,
    /// The heartbeat label this stage armed the feed with. Returned rather than
    /// recomputed by the caller: one derivation, one call.
    label: String,
    wall_nanos: u128,
    /// Runtime unit count the clamp used (discovery witnesses + gate rows).
    unit_count: u128,
    /// The derived clamp actually computed, or None when the plan declares no clamp
    /// params for this population.
    clamp_ms: Option<u128>,
    /// A worker thread panicked — an infra fault, distinct from a claim verdict.
    thread_panicked: bool,
    /// The stage exceeded its derived clamp. Already printed as a typed refusal.
    over_budget: bool,
}

/// ONE stage of a walk, and the SINGLE execution path both populations share.
///
/// Groups the runnables into resolve-units, partitions them across the unit lanes
/// (`batch_unit_lane`), spawns the eligible ones — where `run_batch_unit` takes a
/// governor slot for the unit's lifetime — runs the memo and main-thread lanes on this
/// thread while the spawned ones work, joins everything, and applies the derived cost
/// clamp. It returns only once every member has finished, so the barrier is structural
/// rather than a convention each caller has to remember.
///
/// It deliberately does NOT print PASS/FAIL, classify failures, or write receipts.
/// Folding either population's ordering law in here would put one population's policy
/// inside the other's executor, which is the fork this extraction exists to close.
#[allow(clippy::too_many_arguments)]
fn run_stage(
    source_roots: &[String],
    stage: &[Runnable],
    population: StagePopulation,
    index: usize,
    feed_index: u64,
    memo: &mut std::collections::HashMap<(String, ExecutionMode), InterpContext>,
    memo_path_entries: &std::collections::HashSet<(String, ExecutionMode)>,
    governor: &Arc<MemoryGovernor>,
    fast_lane_eval_budget_ms: Option<u64>,
    falsifier_self_host_wet_budgets: &FalsifierSelfHostWetBudgets,
    clamp_params: Option<(u128, u128)>,
    budget_tighten_ms: Option<u128>,
) -> StageRun {
    let units = group_batch_units(stage);
    // Arm the observation heartbeat feed at stage-enter: discovery leaves entry_total
    // pending (filled when the roster's entry-group count is known); SingleClaim arms
    // immediately with the claim count. Never a fabricated 0-of-0.
    let label = batch_heartbeat_label(stage);
    let entry_total = if stage
        .iter()
        .any(|r| matches!(r, Runnable::DiscoveryBatch { .. }))
    {
        None
    } else if stage.is_empty() {
        None
    } else {
        Some(stage.len() as u64)
    };
    heartbeat_feed_enter_batch(feed_index, &label, entry_total);
    eprintln!(
        "claim_executor: {} {} — {} node(s) in {} resolve-group(s), governor target_width={}",
        population.label(),
        index + 1,
        stage.len(),
        units.len(),
        governor.current_target_width()
    );
    let stage_start = Instant::now();
    // Partition units into lanes (decision table: `batch_unit_lane`): memo and Discovery
    // units stay on the main thread; others spawn.
    let mut memo_units: Vec<BatchUnit> = Vec::new();
    let mut main_thread_units: Vec<BatchUnit> = Vec::new();
    let mut thread_units: Vec<BatchUnit> = Vec::new();
    for unit in units {
        match batch_unit_lane(&unit, memo, memo_path_entries) {
            UnitLane::Memo => memo_units.push(unit),
            UnitLane::MainThread => main_thread_units.push(unit),
            UnitLane::Spawned => thread_units.push(unit),
        }
    }
    // Bracket the parallel walk in a host-effect group: the `[file]`/`[rest]`/`[shell]`
    // trace lines stream to stderr from the worker threads INSIDE the group, while the
    // scannable PASS/FAIL summary is deferred to AFTER the group closes (the caller
    // prints it) so it stays outside the collapsed section. GitHub Actions renders this
    // as a collapsible `::group::`; a plain terminal as a header. Threads cannot
    // interleave group markers (one open/close on the main thread spans the whole
    // stage), so it is sound under parallel unit threads.
    let grouped = v1_compiler::v1_interpreter::host_trace_grouping_active();
    if grouped {
        set_phase(
            FloorPhase::HostEffect,
            &format!("{}-{}-host-effects", population.phase_slug(), index + 1),
        );
        v1_compiler::v1_interpreter::group_begin(&format!(
            "{} {} host-effects",
            population.label(),
            index + 1
        ));
    }
    let spawned: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>> = thread_units
        .into_iter()
        .map(|unit| {
            let roots = source_roots.to_vec();
            let unit_governor = governor.clone();
            let wet_budgets = falsifier_self_host_wet_budgets.clone();
            let boxed: Box<dyn FnOnce() -> Vec<ClaimResult> + Send> = Box::new(move || {
                run_batch_unit(
                    roots,
                    unit,
                    unit_governor,
                    fast_lane_eval_budget_ms,
                    wet_budgets,
                )
            });
            boxed
        })
        .collect();
    let handles = spawn_units(spawned);
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
            let results =
                run_memo_shared_claims(source_roots, &entry, &functions, execution_mode, memo);
            memo_results.extend(results);
        }
    }
    // Discovery pumps on the main thread (see the partition above): the pump's
    // `process_shared_index` is this thread's — the one the eager compile-clean receipt
    // install warmed — so the corpus reads the gate's typed store.
    for unit in main_thread_units {
        memo_results.extend(run_batch_unit(
            source_roots.to_vec(),
            unit,
            governor.clone(),
            fast_lane_eval_budget_ms,
            falsifier_self_host_wet_budgets.clone(),
        ));
    }
    // Collect all results before returning — the caller's PASS/FAIL prints must land
    // after `group_end`, and a thread panic still has to close the group.
    let mut results: Vec<ClaimResult> = memo_results;
    let (joined, thread_panicked) = join_units(handles);
    results.extend(joined);
    if grouped {
        v1_compiler::v1_interpreter::group_end();
    }
    // SingleClaim path: discovery advances the feed via `index_schedule_entry_completed`;
    // gate stages have no schedule retention, so each claim result is the per-entry tick.
    if !stage
        .iter()
        .any(|r| matches!(r, Runnable::DiscoveryBatch { .. }))
    {
        for _ in &results {
            heartbeat_feed_entry_completed();
        }
    }
    let wall_nanos = stage_start.elapsed().as_nanos();
    // THE COST WALL (Piece 3 derived clamp): the per-stage clamp is overhead + runtime
    // unit count * rate, computed HERE where the affected-set-selected count is known
    // (the schedule holds one opaque discovery runnable; the count is runtime).
    // Over-clamp is a typed, located refusal that reds the walk; it never widens (no
    // rerun, no scope change, no cap raise). Witness verdicts inside the stage stand as
    // evaluated — the clamp is an admission/scheduling fact, not a verdict term. A
    // population whose plan declares no clamp params gets None and is not clamped;
    // nothing is fabricated for it.
    let unit_count = batch_runtime_unit_count(&results);
    let clamp_ms: Option<u128> = clamp_params.map(|(overhead_ms, rate_ms)| {
        let mut clamp = overhead_ms + unit_count * rate_ms;
        if let Some(t) = budget_tighten_ms {
            clamp = clamp.min(t);
        }
        clamp
    });
    let mut over_budget = false;
    if let Some(clamp) = clamp_ms {
        let wall_ms = wall_nanos / 1_000_000;
        if wall_ms > clamp {
            over_budget = true;
            println!(
                "{}",
                paint(
                    &format!(
                        "✗ FLOOR-BATCH-OVER-BUDGET {}={} wall_ms={} clamp_ms={} units={}                                  (clamp = overhead + units*rate; authority gunbc.ci_spec                                  gunbc_ci_floor_batch_clamp_params[{}]; raising an overhead or rate requires                                  an operator-signed line per gunbc_ci_floor_batch_clamp_note — a refusal,                                  never a widen)",
                        population.phase_slug(),
                        index + 1,
                        wall_ms,
                        clamp,
                        unit_count,
                        index
                    ),
                    sgr::ERROR
                )
            );
        }
    }
    StageRun {
        results,
        label,
        wall_nanos,
        unit_count,
        clamp_ms,
        thread_panicked,
        over_budget,
    }
}

fn run_walk(
    source_roots: &[String],
    plan_site: &str,
    batches: &[Vec<Runnable>],
    on_success_stages: &[Vec<Runnable>],
    floor_finalization: Option<&FloorFinalization>,
    governor: &Arc<MemoryGovernor>,
    fast_lane_eval_budget_ms: Option<u64>,
    falsifier_self_host_wet_budgets: FalsifierSelfHostWetBudgets,
    stop_policy: FloorBatchStopPolicy,
    batch_clamp_params: Option<&[(u128, u128)]>,
    budget_tighten_ms: Option<u128>,
    falsifier_cadence: bool,
    witness_row_cost_basis_path: &Path,
    // The observed walk-attempt identity. `None` is legitimate ONLY for a plan with no
    // on-success stages: nothing writes an attempt-scoped receipt, so nothing needs the
    // identity. With stages present the arm-time check has already refused an unidentified
    // walk, so `None` here is unreachable — and the stage loop still treats it as a typed
    // refusal rather than unwrapping, because "unreachable by an invariant elsewhere" is
    // the assumption that stops being true when someone adds a second caller.
    walk_attempt_id: Option<&str>,
) -> WalkOutcome {
    let mut any_failed = false;
    let mut batches_run = 0usize;
    let mut failure_details: Vec<String> = Vec::new();
    let mut infra_faults: Vec<InfraFault> = Vec::new();
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
    let memo_path_entries = memo_path_entry_keys(batches);
    for (bi, batch) in batches.iter().enumerate() {
        batches_run = bi + 1;
        let StageRun {
            results: batch_results,
            label,
            wall_nanos: batch_wall_nanos,
            unit_count: batch_unit_count,
            clamp_ms: batch_clamp_ms,
            thread_panicked,
            over_budget,
        } = run_stage(
            source_roots,
            batch,
            StagePopulation::OrdinaryBatch,
            bi,
            bi as u64,
            &mut walk_memo,
            &memo_path_entries,
            governor,
            fast_lane_eval_budget_ms,
            &falsifier_self_host_wet_budgets,
            batch_clamp_params.and_then(|p| p.get(bi)).copied(),
            budget_tighten_ms,
        );
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
                failure_details.push(format!(
                    "batch={} fn={} detail={}",
                    bi + 1,
                    result.function,
                    result.detail
                ));
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
            failure_details.push(format!("batch={} infra=thread_panic", bi + 1));
            infra_faults.push(InfraFault::ClaimThreadPanicked {
                batch_index: bi + 1,
            });
            any_failed = true;
        }
        // The clamp refusal itself printed inside `run_stage`; the verdict term belongs
        // to the ordinary floor, so it is applied here where that failure state lives.
        if over_budget {
            any_failed = true;
        }
        batch_records.push(BatchRecord {
            batch_index: bi,
            wall_nanos: batch_wall_nanos,
            clamp_ms: batch_clamp_ms,
            unit_count: batch_unit_count,
            results: batch_results,
            label: label.clone(),
            selection_tag: batch_selection_tag(batch),
        });
        // LOCAL, not aggregate. This used to test the cumulative `any_failed`, so under
        // FullLedger every batch after the first failure was announced as "batch N had
        // failures" whether or not it had any — the same local-versus-aggregate
        // conflation that made the falsifier alert misattribute green components
        // (review 2026-07-31). The stop DECISION is still the walk's, because
        // StopBeforeDependents must halt on any prior failure; only the message is local.
        let this_batch_failed = thread_panicked
            || over_budget
            || batch_records
                .last()
                .map(|r| r.results.iter().any(|x| !x.ok))
                .unwrap_or(false);
        if any_failed {
            match stop_policy {
                FloorBatchStopPolicy::StopBeforeDependents => {
                    eprintln!(
                        "claim_executor: stopping before dependent batches (batch {} {})",
                        bi + 1,
                        if this_batch_failed {
                            "failed"
                        } else {
                            "green; an earlier batch failed"
                        }
                    );
                    break;
                }
                FloorBatchStopPolicy::FullLedger => {
                    eprintln!(
                        "claim_executor: continuing (FullLedger stop policy) — batch {} {}",
                        bi + 1,
                        if this_batch_failed {
                            "failed"
                        } else {
                            "green; an earlier batch failed"
                        }
                    );
                }
            }
        }
    }
    let total_wall_nanos = walk_start.elapsed().as_nanos();
    emit_gantt(&batch_records, total_wall_nanos);
    let resolve_receipt_ok = write_resolve_receipt(&batch_records);
    let batch_wall_receipt_ok = write_batch_wall_receipt(&batch_records);
    let gate_warm_cost_receipt_ok = write_gate_warm_cost_receipt(&batch_records, falsifier_cadence);
    let witness_row_cost_receipt_ok =
        write_witness_row_cost_receipt(&batch_records, falsifier_cadence);
    let witness_row_cost_drift_receipt_ok = if falsifier_cadence {
        write_witness_row_cost_drift_receipt_at(
            std::path::Path::new("target"),
            &batch_records,
            witness_row_cost_basis_path,
            source_roots,
        )
    } else {
        true
    };
    // Written on EVERY exit path, red included — a red run is precisely when the
    // alert needs to know which component failed (gunbc.floor_component_receipt).
    let floor_component_receipt_ok = write_floor_component_receipt_at(
        std::path::Path::new("target"),
        source_roots,
        &batch_records,
        batches.len(),
    );
    // Memo contexts absorb their ledger totals into the process accumulator on
    // Drop, so they must die before the materialization receipt is written.
    drop(walk_memo);
    let materialization_receipt_ok = write_materialization_receipt();
    // Ordinary-floor verdict INCLUDING receipt construction: a receipt-write failure
    // is an ordinary-floor failure, so success stages are gated on the whole floor
    // contract this process owns, not just the batch verdicts (ruling 2026-07-30).
    // #7467's per-component receipt joins the ordinary verdict: its construction or
    // write failure is an ORDINARY-FLOOR failure and must therefore block admission,
    // not merely red the walk after a stamp. Preserved explicitly through the rebase
    // because dropping it would reopen the ordering defect this branch exists to close.
    let mut ordinary_failed = any_failed
        || !resolve_receipt_ok
        || !batch_wall_receipt_ok
        || !gate_warm_cost_receipt_ok
        || !witness_row_cost_receipt_ok
        || !witness_row_cost_drift_receipt_ok
        || !floor_component_receipt_ok
        || !materialization_receipt_ok;
    // Floor finalization laws (the in-executor form of the deleted resolve/
    // materialization gate steps): validated AFTER the receipts wrote and BEFORE the
    // on-success stages, so a violation blocks admission instead of post-dating it.
    if let Some(fin) = floor_finalization {
        let refusals = validate_floor_finalization(fin, plan_site, &batch_records);
        if refusals.is_empty() {
            if !ordinary_failed {
                eprintln!(
                    "claim_executor: floor contract finalized — resolve count matches declared {} \
                     and materialization disclosure holds",
                    fin.declared_resolve_count
                );
            }
        } else {
            ordinary_failed = true;
            for msg in refusals {
                eprintln!("claim_executor: FLOOR-FINALIZATION-REFUSED: {msg}");
                failure_details.push(format!("floor finalization refused: {msg}"));
            }
        }
    }
    // On-success stages: run only on a fully-green ordinary floor; each stage is a
    // barrier and stage-to-stage execution is ALWAYS fail-fast — FloorBatchStopPolicy
    // is an ordinary-floor policy and never applies between stages. Members run through
    // `run_stage`, the SAME executor the ordinary batches use — same unit grouping, same
    // lane partition, same governor admission — so a stage's members are concurrent
    // exactly as the contract says, and nothing reaches a weaker route. The two
    // populations differ here only in ordering and failure policy, which is the whole
    // claim of the extraction. Their memo contexts drop AFTER
    // write_materialization_receipt above, so stage materialization is structurally NOT
    // folded into the floor receipt.
    let mut on_success_failed = false;
    if !on_success_stages.is_empty() {
        if ordinary_failed {
            eprintln!(
                "claim_executor: ordinary floor failed — {} on-success stage(s) NOT run \
                 (green-only by construction)",
                on_success_stages.len()
            );
            let _ = write_on_success_receipt(&[], 0, true, on_success_stages.len());
        } else {
            let mut stage_rows: Vec<(usize, bool)> = Vec::new();
            let mut stage_resolves: u64 = 0;
            let mut stage_memo: std::collections::HashMap<(String, ExecutionMode), InterpContext> =
                std::collections::HashMap::new();
            // The stage population's own memo-path set. The stage memo is SEPARATE from
            // `walk_memo` by lifecycle, not by accident: stage contexts must drop after
            // `write_materialization_receipt` above so stage materialization is
            // structurally not folded into the floor receipt. One entry shared by several
            // stages still resolves once, within that separate map.
            let stage_memo_path_entries = memo_path_entry_keys(on_success_stages);
            for (si, stage) in on_success_stages.iter().enumerate() {
                let run = run_stage(
                    source_roots,
                    stage,
                    StagePopulation::OnSuccessStage,
                    si,
                    (batches.len() + si) as u64,
                    &mut stage_memo,
                    &stage_memo_path_entries,
                    governor,
                    fast_lane_eval_budget_ms,
                    &falsifier_self_host_wet_budgets,
                    // Stages declare no clamp params: `gunbc_ci_floor_batch_clamp_params`
                    // indexes the ORDINARY batches, and reusing an ordinary batch's
                    // overhead/rate for a stage would fabricate a budget nobody declared.
                    // The arm-time validator refuses the profiles that would need one.
                    None,
                    budget_tighten_ms,
                );
                // A supplied clamp must have teeth. Stages pass None today (see the
                // call above), so this is currently unreachable — it is wired now so that
                // adding a declared stage clamp is a one-line change rather than a
                // one-line change plus remembering this fold, which is exactly the kind of
                // omission the ordinary path's own clamp handling had to be corrected for.
                let mut stage_failed = run.thread_panicked || run.over_budget;
                if run.over_budget {
                    failure_details.push(format!(
                        "on-success stage {} exceeded its declared clamp",
                        si + 1
                    ));
                }
                if run.thread_panicked {
                    println!(
                        "{}",
                        paint(
                            &format!(
                                "✗ FAIL [on-success stage {}] <claim thread panicked>",
                                si + 1
                            ),
                            sgr::ERROR
                        )
                    );
                    failure_details.push(format!("on-success stage {} infra=thread_panic", si + 1));
                    infra_faults.push(InfraFault::ClaimThreadPanicked {
                        batch_index: batches.len() + si + 1,
                    });
                }
                for r in &run.results {
                    if r.resolve_nanos > 0 {
                        stage_resolves += 1;
                    }
                    if r.ok {
                        println!(
                            "{}",
                            paint(
                                &format!("✓ PASS [on-success stage {}] {}", si + 1, r.function),
                                sgr::SUCCESS
                            )
                        );
                    } else {
                        stage_failed = true;
                        println!(
                            "{}",
                            paint(
                                &format!(
                                    "✗ FAIL [on-success stage {}] {} ({})",
                                    si + 1,
                                    r.function,
                                    r.detail
                                ),
                                sgr::ERROR
                            )
                        );
                        failure_details.push(format!(
                            "on-success stage {} fn={} detail={}",
                            si + 1,
                            r.function,
                            r.detail
                        ));
                    }
                }
                stage_rows.push((si, !stage_failed));
                // THIS stage's receipt, written BEFORE the next stage begins. The
                // aggregate receipt below cannot substitute: written only after the whole
                // sequence, it does not exist for stage N while stage N+1 runs, and a
                // process death between the two loses every stage that had in fact
                // completed. A receipt that cannot be written is itself a stage failure.
                let receipt_written = match walk_attempt_id {
                    Some(attempt) => write_on_success_stage_receipt(
                        si,
                        !stage_failed,
                        &run,
                        on_success_stages.len(),
                        attempt,
                    ),
                    None => {
                        eprintln!(
                            "claim_executor: on-success stage {} has no walk-attempt identity — \
                             refusing to write an unidentifiable receipt (arm-time observation \
                             should have refused this walk already; reaching here means a caller \
                             ran stages without observing identity)",
                            si + 1
                        );
                        false
                    }
                };
                if !receipt_written {
                    stage_failed = true;
                    failure_details
                        .push(format!("on-success stage {} receipt write failed", si + 1));
                    if let Some(last) = stage_rows.last_mut() {
                        last.1 = false;
                    }
                }
                if stage_failed {
                    on_success_failed = true;
                    eprintln!(
                        "claim_executor: on-success stage {} failed — remaining stage(s) NOT run \
                         (stages are fail-fast regardless of the ordinary stop policy)",
                        si + 1
                    );
                    break;
                }
            }

            if !write_on_success_receipt(
                &stage_rows,
                stage_resolves,
                false,
                on_success_stages.len(),
            ) {
                on_success_failed = true;
            }
        }
    }
    WalkOutcome {
        any_failed: ordinary_failed || on_success_failed,
        batches_run,
        failure_details,
        infra_faults,
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
    "gunbc_ci_floor_plan",
    "gunbc_ci_plan_artifact_plan",
    "gunbc_falsifier_plan",
];

fn plan_requires_floor_arm_time_budget_refusal(plan_function: &str) -> bool {
    FLOOR_ARM_TIME_BUDGET_REFUSAL_PLAN_FUNCTIONS.contains(&plan_function)
}

fn run_perturb_check(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<ExitCode, ExitCode> {
    let walk_plan = match eval_plan(source_roots, plan_entry, plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: --perturb-check: {msg}");
            return Err(ExitCode::from(2));
        }
    };
    // --perturb-check exercises the ordinary gating batch only; success stages are
    // out of its subject (they cannot run when the planted gate fails, by construction).
    let batches = walk_plan.batches;
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
                        profile,
                    } => Runnable::SingleClaim {
                        entry: if entry.is_empty() {
                            entry.clone()
                        } else {
                            remap_entry_for_temp(primary, &temp_src, entry)
                                .to_string_lossy()
                                .into_owned()
                        },
                        function: function.clone(),
                        profile: *profile,
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
        &format!("{plan_entry}::{plan_function} (perturb re-walk)"),
        &remapped,
        &[],
        None,
        &Arc::new(MemoryGovernor::from_environment(1)),
        None,
        FalsifierSelfHostWetBudgets::default(),
        FloorBatchStopPolicy::StopBeforeDependents,
        None,
        None,
        false,
        Path::new("dag/gunbc/witness_row_cost_basis.tsv"),
        // The perturb re-walk passes `&[]` for stages, so no attempt-scoped receipt is
        // written and no identity is owed.
        None,
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

    // Coarse phase marks for the pre-walk prelude (hygiene walk, policy install,
    // plan resolve/eval, governor arm) — interpreter-heavy phases that used to be
    // SILENT, so a 30-minute prelude looked identical to a hang. These now render
    // through the single-authority observation renderer (`gunbc.observation_ci_render`,
    // via the `gunbc.observation_seed_render` boundary) as `✅ <phase> done in <human
    // duration>` instead of a raw `[t+…]` byte string the seed would fork the format
    // into. Each mark carries its OWN wall (delta since the last mark), not a running
    // `t+`, so the log itemizes which prelude phase is slow — the step toward the
    // per-phase receipt keys the ci_spec prelude-coverage-hole follow-up calls for.
    let floor_started = Instant::now();
    let phase_last = std::cell::Cell::new(floor_started);
    // Emoji glyphs under GitHub Actions, Unicode on a plain terminal (the same medium
    // split `install_group_syntax` makes for `::group::` markers).
    let phase_glyph_emoji = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");
    // Placement basis only (see `gunbc.observation_seed_render`): the clamp overhead
    // `gunbc.ci_spec` budgets for non-per-unit floor time, so a prelude phase completing
    // within it reads Ambient. Coarse + placement-only — `ci_render_line` reads only the
    // glyph and text — and dissolves when the stream driver reads the basis from ci_spec.
    const PHASE_BASIS_OVERHEAD_MS: u64 = 300_000;
    let phase_mark_roots = source_roots.clone();
    let phase_mark = |label: &str| {
        let now = Instant::now();
        let delta_ms = now.saturating_duration_since(phase_last.get()).as_millis() as u64;
        phase_last.set(now);
        match render_phase_concluded_line(
            &phase_mark_roots,
            label,
            delta_ms,
            PHASE_BASIS_OVERHEAD_MS,
            phase_glyph_emoji,
        ) {
            Some(line) => eprintln!("{line}"),
            // §5: refuse to fabricate the pretty format when the renderer is unreachable,
            // and never reproduce the deleted `[t+…]` marker — name the degradation loudly.
            None => eprintln!(
                "claim_executor: phase {label} (+{:.1}s) [observation renderer unavailable]",
                (delta_ms as f64) / 1000.0
            ),
        }
    };

    // Install the host-effect trace policy from the .dag authority FIRST — before the
    // naming-hygiene walk below and every subsequent corpus read — so `[file] read` /
    // `[rest]` / `[hermetic:mock]` etc. are funnelled per `gunbc.output_policy`
    // (Instrumentation is Suppressed at Normal, the CI default) instead of flooding the
    // floor log. Installing AFTER the walk (the prior order) left the walk's whole-tree
    // read at the `Full` default — ~2.3k `[file] read` lines, the firehose the
    // observation-emit census (`gunbc.observation_emit_census`) targets. The walk still
    // runs before plan evaluation, so a naming violation stays the cheapest failure.
    v1_compiler::cli_run::install_output_policy(&source_roots);
    // Install the per-target group-marker syntax (GitHub Actions `::group::` vs a
    // plain-terminal header) from the .dag authority, so the parallel walk folds each
    // batch's host-effect traces into a collapsible group.
    v1_compiler::cli_run::install_group_syntax(&source_roots);
    phase_mark("output-policy + group-syntax install");

    // Under the opt-in inversion the plan's DiscoveryBatches carry explicit entries
    // only (or are absent entirely on an empty roster), and the explicit-only path
    // skips the tree-walk naming hygiene (`test fn` outside `*_test.dag`, `__`
    // basenames) that glob discovery used to run. A witness must stay NAMEABLE even
    // when not enrolled (an unnameable witness could never be opted in), so the plan
    // path always runs the fail-closed walk once up front — before the (expensive)
    // plan evaluation, so a naming violation is the cheapest possible failure.
    {
        let excludes = v1_compiler::cli_run::witness_exclusion_substrings();
        if let Err(msg) =
            v1_compiler::cli_run::discover_floor_witness_roster(&source_roots, &[], &excludes, &[])
                .map(|_| ())
        {
            eprintln!("claim_executor: witness naming hygiene (pre-plan walk): {msg}");
            return Err(ExitCode::from(1));
        }
    }
    phase_mark("naming-hygiene walk");

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
    phase_mark("plan resolve");

    let plan_ctx = make_eval_context(&plan_graph, plan_indices.clone(), ExecutionMode::Hermetic);
    let walk_plan = match eval_plan_in_ctx(&plan_ctx, &plan_entry, &plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    };
    let batches = walk_plan.batches;
    let on_success_stages = walk_plan.on_success_stages;
    // Plan-shape validation BEFORE the governor arms and before any batch runs.
    {
        let refusals = validate_on_success_stage_admissibility(&on_success_stages);
        if !refusals.is_empty() {
            for msg in &refusals {
                eprintln!("claim_executor: ON-SUCCESS-STAGE-REFUSED: {msg}");
            }
            return Err(ExitCode::from(1));
        }
    }
    // Walk-attempt identity, observed at the SAME altitude and for the same reason as the
    // shape refusal above: a walk that cannot identify itself cannot write a receipt anyone
    // can attribute, and that is knowable now rather than after a 20-30 minute floor. Only
    // demanded when stages exist — a plan with no on-success stages writes no attempt-scoped
    // receipt, so requiring identity of it would be a refusal with no subject.
    let walk_attempt_id: Option<String> = if on_success_stages.is_empty() {
        None
    } else {
        match observe_walk_attempt_id() {
            Ok(id) => Some(id),
            Err(msg) => {
                eprintln!("claim_executor: ON-SUCCESS-STAGE-REFUSED: {msg}");
                return Err(ExitCode::from(1));
            }
        }
    };
    // Floor finalization is carried BY the plan value (walk_finalization_note) — the
    // name-keyed read this replaces was the same seed-roster convention the carrier
    // was built to remove, reintroduced and then caught in review.
    let floor_finalization: Option<FloorFinalization> = walk_plan.finalization;
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
    let falsifier_self_host_wet_budgets = if plan_function == "gunbc_falsifier_plan" {
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
            roster_entry_paths: match read_schedule_witness_entry_paths(
                &plan_ctx,
                "falsifier_self_host_wet_entries",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            known_red_entry_paths: match read_schedule_witness_entry_paths(
                &plan_ctx,
                "falsifier_self_host_wet_known_red_roster",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            hermetic_known_red_entry_paths: match read_schedule_witness_entry_paths(
                &plan_ctx,
                "known_red_probe_roster",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            silent_pick_wall_budget_ms: match read_positive_budget_ms(
                &plan_ctx,
                "gunbc_falsifier_silent_pick_gate_receipt_wall_budget_ms",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            silent_pick_entry_paths: match read_schedule_witness_entry_paths(
                &plan_ctx,
                "falsifier_silent_pick_gate_roster",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            substrate_long_lane_entry_paths: match read_schedule_witness_entry_paths(
                &plan_ctx,
                "falsifier_substrate_long_lane_entries",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            substrate_long_lane_eval_budget_ms: match read_positive_budget_ms(
                &plan_ctx,
                "gunbc_falsifier_substrate_long_lane_eval_budget_ms",
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
    // THE COST WALL (Piece 3 derived clamp): the floor plan's per-batch clamp params, read
    // fail-closed at arm time (the fast-lane-budget pattern). Scoped to the full floor plan only:
    // the plan-artifact shortcut runs a single batch of the same schedule and the falsifier
    // carries its own receipt budgets, so neither reads these lists.
    let batch_clamp_params: Option<Vec<(u128, u128)>> = if plan_function == "gunbc_ci_floor_plan" {
        match read_floor_batch_clamp_params(&plan_ctx, batches.len()) {
            Ok(v) => Some(v),
            Err(msg) => {
                eprintln!("{msg}");
                return Err(ExitCode::from(1));
            }
        }
    } else {
        None
    };
    let budget_tighten_ms: Option<u128> = match read_floor_batch_budget_tighten_ms() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{msg}");
            return Err(ExitCode::from(1));
        }
    };
    // Compile-clean leg clamp constants (prelude coverage follow-up (a)): read for the
    // two plan shapes that arm the compile-clean gate, at the same fail-closed arm point
    // as the batch clamps; enforcement runs post-walk where the leg's cost snapshot exists.
    let compile_clean_clamp: Option<(u128, u128)> = if plan_function == "gunbc_ci_floor_plan"
        || plan_function == "gunbc_ci_plan_artifact_plan"
    {
        match read_compile_clean_clamp(&plan_ctx) {
            Ok(v) => Some(v),
            Err(msg) => {
                eprintln!("{msg}");
                return Err(ExitCode::from(1));
            }
        }
    } else {
        None
    };
    let compile_clean_tighten_ms: Option<u128> = match read_compile_clean_budget_tighten_ms() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{msg}");
            return Err(ExitCode::from(1));
        }
    };
    drop(plan_ctx);
    phase_mark("plan eval");

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
    phase_mark("memory-governor arm");
    spawn_floor_memory_heartbeat();

    // Plans whose schedule carries the compile-clean gate node: the gate only CONSUMES the
    // in-run whole-tree compile receipt, so these plans must arm the lazy install.
    // `gunbc_ci_plan_artifact_plan` is batch 1 of the floor schedule (the docs-only
    // shortcut) — same gate node, same receipt dependency.
    if plan_function == "gunbc_ci_floor_plan" || plan_function == "gunbc_ci_plan_artifact_plan" {
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
        &format!("{plan_entry}::{plan_function}"),
        &batches,
        &on_success_stages,
        floor_finalization.as_ref(),
        &governor,
        fast_lane_eval_budget_ms,
        falsifier_self_host_wet_budgets,
        batch_stop_policy,
        batch_clamp_params.as_deref(),
        budget_tighten_ms,
        plan_function == "gunbc_falsifier_plan",
        Path::new("dag/gunbc/witness_row_cost_basis.tsv"),
        walk_attempt_id.as_deref(),
    );
    // Floor receipts block — data, not outcomes. One named group; pulse glyphs only
    // (operator live-log 2026-07-25: outcome glyphs for outcomes only).
    v1_compiler::v1_interpreter::group_begin("floor receipts");
    match peak_rss_bytes() {
        Some(bytes) => {
            eprintln!(
                "{}",
                v1_compiler::cli_run::render_peak_rss_line_mirror(
                    "floor peak RSS (adaptive width)",
                    Some(bytes),
                    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true"),
                )
            );
        }
        None => eprintln!(
            "{}",
            v1_compiler::cli_run::render_peak_rss_line_mirror(
                "floor peak RSS (adaptive width)",
                None,
                std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true"),
            )
        ),
    }
    // The governor receipt is the §5-counted degradation story for the run: every graceful
    // hold, hard back-off, and forced-serial admission, beside the width actually reached.
    eprintln!("{}", governor.receipt_line());
    // WHOLE-TREE cgroup peak — the SOUND placement divisor input (SELF-RSS above omits
    // child rustc/sccache PIDs; cgroup-v2 `memory.peak` at the leaf job cgroup is hierarchical and
    // captures them). Single authority `emit_cgroup_measurement` so the `ci` and `rust_tests` jobs
    // report an identically-shaped line. Runtime-harmless read-only.
    emit_cgroup_measurement("floor adaptive-width");
    // Compile-clean leg cost gates (prelude coverage follow-up (a)): the enforced clamp
    // and the counted basis drift, both over the leg's cost snapshot. Post-walk so both
    // the eager and the lazy install path are covered; no snapshot (skipped/refused leg)
    // means nothing to clamp and nothing to compare.
    let compile_clean_over_budget =
        enforce_floor_compile_clean_clamp(compile_clean_clamp, compile_clean_tighten_ms);
    let compile_clean_drift_receipt_ok = write_compile_clean_cost_drift_receipt_at(
        std::path::Path::new("target"),
        std::path::Path::new("dag/gunbc/compile_clean_cost_basis.tsv"),
        &source_roots,
    );
    v1_compiler::v1_interpreter::group_end();
    if outcome.any_failed {
        emit_falsifier_failure_class(&outcome.failure_details, &outcome.infra_faults);
    }
    floor_terminal_fast_exit(walk_exit_code(
        outcome.any_failed || compile_clean_over_budget || !compile_clean_drift_receipt_ok,
    ))
}

/// Typed terminal failure class for the falsifier/floor walk (brief Step 2, 2026-07-25):
/// names BudgetExceeded{wall,budget} vs WitnessRed{claims} vs Infra{spawn/toolchain/eviction}
/// so "falsifier dark" is one of three modes, never an undifferentiated exit 1.
fn falsifier_failure_mode(details: &[String]) -> &'static str {
    if details.iter().any(|d| {
        d.contains("BudgetExceeded{")
            || d.contains("witness receipt wall budget exceeded")
            || d.contains("wet self-host receipt wall budget exceeded")
            || d.contains("eval budget exceeded")
    }) {
        "BudgetExceeded"
    } else if details.iter().any(|d| {
        // THIS LIST IS A FORK, and that is the defect — not the individual substrings.
        //
        // `gunbc.ci_failure_class` already models this properly: `InfraSignature` is a
        // closed sum whose match text is DERIVED from cited upstream authorities —
        // `errno_strerror(EAGAIN)`, `spawn_outcome_log_fragment(SpawnFailed)`, the sccache
        // fragments, `runner_lifecycle_log_fragment(ShutdownReceived)` — each carrying an
        // `infra_signature_origin` naming where the text comes from. That authority has no
        // production consumer today (`classify_failure_reason` is reached only by witness
        // tests), while this hand-typed list is the one that actually runs.
        //
        // So the fix is to make this consume `gunbc.ci_failure_class`, not to hand-edit the
        // fork. An earlier revision of this change deleted the "failed to spawn" arm on the
        // grounds that no producer reachable from THIS input emits it — the interpreter's
        // spawn failure says "failed to execute". That evidence was about this surface only;
        // the model declares `ProcessSpawnFailure` a live signature whose site is the build
        // log. Editing one side of a fork on surface-local evidence widens the divergence
        // instead of closing it, so the arm is restored and the fork is recorded here.
        d.contains("Resource temporarily unavailable")
            || d.contains("failed to spawn")
            || d.contains("sccache")
    }) {
        "Infra"
    } else {
        "WitnessRed"
    }
}

/// Classification with the walk's structurally-observed faults taking precedence.
///
/// An observed `InfraFault` settles the mode outright: the walk saw the panic, so nothing
/// downstream needs to infer it from rendered text. The string path below it remains for
/// genuinely external message text only.
fn falsifier_failure_mode_with_faults(details: &[String], faults: &[InfraFault]) -> &'static str {
    if !faults.is_empty() {
        return "Infra";
    }
    falsifier_failure_mode(details)
}

/// Projection of a failure mode into the `gunbc.ci_failure_class` vocabulary. `Structural`
/// is `Structural { reason: String }`, so an arm printed WITHOUT its reason is a lossy
/// rendering of the typed value, not the value: it collapses BudgetExceeded and WitnessRed
/// — which have different remedies (re-basis the lane's dated ceiling vs. fix the witness)
/// — into one indistinguishable string, discarding the `mode` computed beside it (receipt:
/// run 30176416535 printed mode=BudgetExceeded next to arm=FloorFailed{class:Structural}).
/// Both non-Infra modes stay Structural — an over-budget witness is a real, non-retryable
/// failure that blocks merge — but the reason rides along. Enumerated, not defaulted, so a
/// new mode cannot fall through into an unlabelled Structural (the OOM-reclassification
/// consumer stays on its own design doc).
fn ci_failure_class_arm(mode: &str) -> String {
    match mode {
        "Infra" => "FloorFailed{class:Infra}".to_string(),
        "BudgetExceeded" => "FloorFailed{class:Structural{reason:BudgetExceeded}}".to_string(),
        "WitnessRed" => "FloorFailed{class:Structural{reason:WitnessRed}}".to_string(),
        other => format!("FloorFailed{{class:Structural{{reason:{other}}}}}"),
    }
}

fn emit_falsifier_failure_class(details: &[String], faults: &[InfraFault]) {
    let joined = details.join(" | ");
    let mode = falsifier_failure_mode_with_faults(details, faults);
    eprintln!("[falsifier-failure-class] mode={mode}");
    if details.is_empty() {
        eprintln!("[falsifier-failure-class] detail=<receipt/write failure or empty ledger>");
    } else {
        for d in details {
            eprintln!("[falsifier-failure-class] {d}");
        }
    }
    let ci_arm = ci_failure_class_arm(mode);
    eprintln!(
        "[falsifier-failure-class] ci_failure_class_arm={ci_arm} reason_preview={}",
        {
            let preview: String = joined.chars().take(240).collect();
            preview
        }
    );
}

/// The walk outcome's process exit code — extracted so the fast-exit wiring is testable
/// without exiting the test process.
fn walk_exit_code(any_failed: bool) -> i32 {
    if any_failed {
        1
    } else {
        0
    }
}

/// Terminal fast-exit for the floor walk (CI floor endgame D2): the executor retains a
/// ~16GB store (process-shared index, typed caches, interner) whose Drop walk at process
/// end measured 2.5–3.1 minutes, twice-confirmed (Pi bench: swap grows DURING Drop;
/// run 30009199696: +3.1min between the last batch verdict and step end) — pure teardown
/// of memory the process is about to abandon to the kernel anyway. After the receipts are
/// written (inside `run_walk`, before it returns — their write failures are already folded
/// into `any_failed`, so a truncated/unwritable receipt reds, never vanishes) and both
/// streams are flushed, `process::exit` skips the Drop walk. This is the common tail of
/// `run()`'s walk path — every plan walk (floor, plan-artifact, regen, falsifier) exits
/// through it; every refusal/error path above returns normally. The exit CODE is exactly
/// `walk_exit_code(any_failed)` — behavior-identical to the ExitCode return it replaces.
fn floor_terminal_fast_exit(code: i32) -> ! {
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    std::process::exit(code)
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

    fn finalization_record(resolve_counts: &[u64]) -> BatchRecord {
        BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            label: "finalization-fixture".to_string(),
            selection_tag: "fixture",
            results: resolve_counts
                .iter()
                .map(|n| ClaimResult {
                    function: "fixture".to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: *n as u128,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    witness_row_costs: Vec::new(),
                    budget_refusal: None,
                })
                .collect(),
        }
    }

    /// The `<entry>::<function>` a refusal locates itself at — the same shape the
    /// production caller passes.
    const TEST_PLAN_SITE: &str = "src/v2/workflow/ci_floor_plan.dag::gunbc_ci_floor_plan";

    // Floor finalization law 1 (in-executor form of the deleted resolve-receipt gate
    // step): the count is recomputed from batch records by the same nonzero-
    // resolve_nanos rule the receipt writer uses. Declared-vs-actual mismatch refuses
    // in BOTH directions; match passes. The materialization arm is exercised through
    // the missing-file refusal (no target/ receipt exists under cargo test), which
    // also pins that law 2 cannot silently pass when the receipt is absent.
    #[test]
    fn floor_finalization_refuses_on_resolve_count_mismatch_both_directions() {
        let fin = FloorFinalization {
            declared_resolve_count: 1,
        };
        let over = [finalization_record(&[5, 7])];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &over);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("floor resolve count 2 differs from declared 1")),
            "over-count must refuse: {refusals:?}"
        );
        let under = [finalization_record(&[])];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &under);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("floor resolve count 0 differs from declared 1")),
            "under-count must refuse: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_matching_count_leaves_only_the_materialization_arm() {
        let fin = FloorFinalization {
            declared_resolve_count: 2,
        };
        let records = [finalization_record(&[3, 0, 9])];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records);
        // resolve law satisfied (two nonzero-resolve results); under cargo test no
        // materialization receipt file exists, so exactly the missing-receipt refusal
        // remains — proving law 2 fails closed on absence rather than passing.
        assert!(
            !refusals.iter().any(|m| m.contains("differs from declared")),
            "resolve law must be satisfied: {refusals:?}"
        );
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("floor materialization receipt missing")),
            "absent receipt must refuse, never pass: {refusals:?}"
        );
    }

    // D2 wiring pin: the fast-exit consumes exactly this mapping, so the terminal
    // process code stays behavior-identical to the ExitCode return it replaced.
    #[test]
    fn walk_exit_code_maps_failure_to_one_success_to_zero() {
        assert_eq!(walk_exit_code(true), 1);
        assert_eq!(walk_exit_code(false), 0);
    }

    #[test]
    fn known_red_probe_entry_paths_detect_expect_red_batch() {
        let hermetic = vec![
            "src/v2/test/claim/emit/logic_ground_truth_test.dag".into(),
            "src/v2/test/claim/manual/english_emit_add_test.dag".into(),
            "src/v2/test/claim/self_host/compiler_closure_emit_from_ingest_test.dag".into(),
        ];
        let wet_known_red =
            vec!["dag/test/claim/self_host_03_normalize_behavioral_witness_test.dag".into()];
        assert!(discovery_entries_are_expect_red(
            &[(
                "src/v2/test/claim/emit/logic_ground_truth_test.dag".into(),
                "logic_complement_truth_table".into()
            )],
            &hermetic,
            &[]
        ));
        assert!(discovery_entries_are_expect_red(
            &[(
                "dag/test/claim/self_host_03_normalize_behavioral_witness_test.dag".into(),
                "self_host_03_normalize_behavioral_receipt_holds".into()
            )],
            &[],
            &wet_known_red
        ));
        assert!(!discovery_entries_are_expect_red(
            &[(
                "dag/test/claim/design_register_lift_parity_witness_test.dag".into(),
                "design_register_lift_parity_holds".into()
            )],
            &hermetic,
            &wet_known_red
        ));
        assert!(!discovery_entries_are_expect_red(
            &[],
            &hermetic,
            &wet_known_red
        ));
    }

    #[test]
    fn self_host_wet_roster_intersection_scopes_wall_budget() {
        let roster = vec![
            "dag/test/claim/self_host_logic_behavioral_witness_test.dag".into(),
            "dag/test/claim/namespace_import_closure_witness_test.dag".into(),
        ];
        assert!(discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/self_host_logic_behavioral_witness_test.dag".into(),
                "self_host_logic_behavioral_receipt_holds".into()
            )],
            &roster
        ));
        // Silent-pick is Wet but not on the self-host roster — must not inherit
        // the 600s whole-receipt ceiling (mis-scope receipt 2026-07-25).
        assert!(!discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/resolution_divergence_silent_pick_gate_witness_test.dag".into(),
                "resolution_divergence_silent_pick_gate_keystone_holds".into()
            )],
            &roster
        ));
        assert!(!discovery_entries_intersect_roster(&[], &roster));
        assert!(!discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/self_host_logic_behavioral_witness_test.dag".into(),
                "self_host_logic_behavioral_receipt_holds".into()
            )],
            &[]
        ));
    }

    #[test]
    fn silent_pick_roster_intersection_scopes_own_wall_budget() {
        let silent_pick =
            vec!["dag/test/claim/resolution_divergence_silent_pick_gate_witness_test.dag".into()];
        assert!(discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/resolution_divergence_silent_pick_gate_witness_test.dag".into(),
                "resolution_divergence_silent_pick_gate_keystone_holds".into()
            )],
            &silent_pick
        ));
        assert!(!discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/self_host_logic_behavioral_witness_test.dag".into(),
                "self_host_logic_behavioral_receipt_holds".into()
            )],
            &silent_pick
        ));
    }

    fn batch_record_for_test(results: Vec<ClaimResult>) -> BatchRecord {
        BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: results.len() as u128,
            results,
            label: "batch-under-test".to_string(),
            selection_tag: "off",
        }
    }

    /// A budget kill must classify as BudgetExceeded from the VALUE, not from the message.
    ///
    /// RED control: the detail string here deliberately contains none of the substrings
    /// `falsifier_failure_mode` looks for, so under the old string-sniffing path this row
    /// would fall through to the `else` arm and report `WitnessRed` — silently swapping
    /// "re-basis a dated ceiling" for "fix the witness". Passing proves the mode is read
    /// off `budget_refusal`, so message wording can change without moving the class.
    #[test]
    fn budget_kill_classifies_structurally_not_by_message_text() {
        let timed_out = ClaimResult {
            function: "some_witness".to_string(),
            ok: false,
            detail: "wording that mentions no budget phrase at all".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            witness_row_costs: Vec::new(),
            budget_refusal: Some(BudgetRefusal {
                elapsed_ms: 900_001,
                budget_ms: 900_000,
                kind: BudgetKind::Wall,
            }),
        };
        // Sanity: the string classifier alone really would misclassify this detail.
        assert_eq!(
            falsifier_failure_mode(&[timed_out.detail.clone()]),
            "WitnessRed",
            "control is only meaningful if the string path would get this wrong"
        );

        let rec = batch_record_for_test(vec![timed_out]);
        let (mode, _detail) = batch_failure_mode_and_detail(&rec);
        assert_eq!(mode, "BudgetExceeded");

        // And an ordinary witness red must NOT be dragged into BudgetExceeded.
        let plain = ClaimResult {
            function: "other_witness".to_string(),
            ok: false,
            detail: "returned Bool(false)".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            witness_row_costs: Vec::new(),
            budget_refusal: None,
        };
        let (mode, _) = batch_failure_mode_and_detail(&batch_record_for_test(vec![plain]));
        assert_eq!(mode, "WitnessRed");
    }

    #[test]
    fn falsifier_failure_mode_classifies_three_arms() {
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=1 BudgetExceeded{wall_ms=900000,budget_ms=600000}".into()
            ]),
            "BudgetExceeded"
        );
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=3 fn=resolution_divergence_silent_pick_gate_keystone_holds detail=witness receipt wall budget exceeded: 707687ms elapsed > 600000ms whole-receipt budget".into()
            ]),
            "BudgetExceeded"
        );
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=3 fn=resolution_divergence_silent_pick_gate_keystone_holds detail=wet self-host receipt wall budget exceeded: 707687ms elapsed > 600000ms whole-receipt budget".into()
            ]),
            "BudgetExceeded"
        );
        // The walk's own infra fact is now read STRUCTURALLY, not grepped out of the line
        // the walk itself formatted. Both directions are asserted so the move from text to
        // value is proven rather than assumed: the same text alone no longer classifies as
        // Infra, and the observed fault classifies as Infra regardless of the text.
        let panic_text: Vec<String> = vec![
            "batch=2 infra=thread_panic".into(),
            "batch=1 fn=x detail=stale digest".into(),
        ];
        assert_eq!(
            falsifier_failure_mode(&panic_text),
            "WitnessRed",
            "the rendered line is for humans now — it must not carry the classification"
        );
        assert_eq!(
            falsifier_failure_mode_with_faults(
                &panic_text,
                &[InfraFault::ClaimThreadPanicked { batch_index: 2 }]
            ),
            "Infra"
        );
        assert_eq!(
            falsifier_failure_mode_with_faults(
                &["batch=1 fn=x detail=nothing resembling infra".into()],
                &[InfraFault::ClaimThreadPanicked { batch_index: 1 }]
            ),
            "Infra",
            "an observed fault settles the mode even when no text hints at it"
        );
        // And no fault means no Infra, whatever the prose says.
        assert_eq!(
            falsifier_failure_mode_with_faults(&panic_text, &[]),
            "WitnessRed"
        );
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=1 fn=design_register_lift_parity_holds detail=false".into()
            ]),
            "WitnessRed"
        );
        // BudgetExceeded wins when co-present with witness detail (honest wall kill).
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=2 fn=wet detail=cargo refuse".into(),
                "batch=2 BudgetExceeded{wall_ms=601000,budget_ms=600000}".into()
            ]),
            "BudgetExceeded"
        );
    }

    // The lane-scoping wiring itself, both directions. The receipted defect (run
    // 30176416535) was that EVERY Hermetic batch drew the 5s per-PR fast-lane budget,
    // including the substrate long lane whose rows measure 12–135s. Discriminating: the
    // same execution_mode with different rosters must yield different ceilings.
    #[test]
    fn hermetic_eval_budget_is_scoped_by_lane_not_by_witness_kind() {
        let long_lane_entry =
            "src/v2/test/claim/long/edit_locus_source_provenance_witness_test.dag";
        let budgets = FalsifierSelfHostWetBudgets {
            substrate_long_lane_entry_paths: vec![long_lane_entry.to_string()],
            substrate_long_lane_eval_budget_ms: Some(180_000),
            ..Default::default()
        };
        let on_lane = vec![(
            long_lane_entry.to_string(),
            "lens_edit_locus_source_provenance_affected_set_wire_holds".to_string(),
        )];
        let off_lane = vec![(
            "src/v2/test/claim/self_host/some_fast_witness_test.dag".to_string(),
            "some_fast_holds".to_string(),
        )];

        // On the lane: its own dated ceiling, NOT the 5s per-PR budget.
        let picked = select_discovery_batch_budgets(
            ExecutionMode::Hermetic,
            &on_lane,
            Some(5_000),
            &budgets,
        );
        assert_eq!(picked.eval_budget_ms, Some(180_000));
        assert_eq!(picked.wet_wall_budget_ms, None);

        // Same execution_mode, different roster: the per-PR fast lane is the residual.
        let residual = select_discovery_batch_budgets(
            ExecutionMode::Hermetic,
            &off_lane,
            Some(5_000),
            &budgets,
        );
        assert_eq!(residual.eval_budget_ms, Some(5_000));

        // The whole point: witness kind alone must not decide the ceiling.
        assert_ne!(
            picked.eval_budget_ms, residual.eval_budget_ms,
            "two Hermetic batches on different lanes must not share one ceiling"
        );

        // A rostered lane with no declared ceiling narrows to the fast lane (a loud,
        // named red), never to an unbudgeted eval.
        let unbudgeted = FalsifierSelfHostWetBudgets {
            substrate_long_lane_eval_budget_ms: None,
            ..budgets.clone()
        };
        assert_eq!(
            select_discovery_batch_budgets(
                ExecutionMode::Hermetic,
                &on_lane,
                Some(5_000),
                &unbudgeted
            )
            .eval_budget_ms,
            Some(5_000)
        );

        // Wet lanes are untouched by the long-lane roster: no eval budget, and the
        // self-host wall/interp pair still arms off its own roster.
        let wet = FalsifierSelfHostWetBudgets {
            wall_budget_ms: Some(600_000),
            interp_eval_budget_ms: Some(120_000),
            roster_entry_paths: vec!["dag/test/claim/wet_receipt_test.dag".to_string()],
            ..budgets.clone()
        };
        let wet_pick = select_discovery_batch_budgets(
            ExecutionMode::Wet,
            &[(
                "dag/test/claim/wet_receipt_test.dag".to_string(),
                "receipt_holds".to_string(),
            )],
            Some(5_000),
            &wet,
        );
        assert_eq!(wet_pick.eval_budget_ms, None);
        assert_eq!(wet_pick.wet_wall_budget_ms, Some(600_000));
        assert_eq!(wet_pick.wet_interp_budget_ms, Some(120_000));
    }

    // The mode must survive the projection into the ci_failure_class vocabulary. Before
    // this, both non-Infra modes rendered as the reason-less `FloorFailed{class:Structural}`
    // — the receipted mis-map (run 30176416535: mode=BudgetExceeded, arm=…Structural).
    // Discriminating: a budget kill and a witness red must NOT print the same arm.
    #[test]
    fn ci_failure_class_arm_carries_the_structural_reason() {
        assert_eq!(ci_failure_class_arm("Infra"), "FloorFailed{class:Infra}");
        assert_eq!(
            ci_failure_class_arm("BudgetExceeded"),
            "FloorFailed{class:Structural{reason:BudgetExceeded}}"
        );
        assert_eq!(
            ci_failure_class_arm("WitnessRed"),
            "FloorFailed{class:Structural{reason:WitnessRed}}"
        );
        assert_ne!(
            ci_failure_class_arm("BudgetExceeded"),
            ci_failure_class_arm("WitnessRed"),
            "a budget kill and a witness red must be distinguishable in the emitted arm"
        );
        // End-to-end through the mode classifier: the batch-6 eval-budget detail shape.
        let detail: Vec<String> = vec![
            "batch=6 fn=discovery-corpus detail=7 of 10 discovery witness(es) failed: \
             lens_affected_set_provenance_producer_coverage_receipt_holds runtime error: \
             eval budget exceeded: 5001ms elapsed > 5000ms fast-lane budget"
                .into(),
        ];
        assert_eq!(
            ci_failure_class_arm(falsifier_failure_mode(&detail)),
            "FloorFailed{class:Structural{reason:BudgetExceeded}}"
        );
    }

    // D2 RED control: a receipt that cannot be written REDS the walk (returns false,
    // folded into any_failed → nonzero exit) — it never vanishes behind the fast exit.
    // The base path is a FILE, so create_dir_all refuses.
    #[test]
    fn unwritable_receipt_base_reds_not_vanishes() {
        let base =
            std::env::temp_dir().join(format!("claim-executor-receipt-red-{}", std::process::id()));
        let _ = fs::remove_file(&base);
        fs::write(&base, b"a file where the receipt dir should be").unwrap();
        assert!(!write_resolve_receipt_at(&base, &[]));
        assert!(!write_batch_wall_receipt_at(&base, &[]));
        let _ = fs::remove_file(&base);
    }

    // D5 receipt rows, both directions: an over-budget batch records OverBudget and is
    // counted; a within-budget batch records WithinBudget; a budget-less walk records
    // Unbudgeted (falsifier/regen plans).
    #[test]
    fn batch_wall_receipt_verdicts_both_directions() {
        let base =
            std::env::temp_dir().join(format!("claim-executor-batch-wall-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        // Clamped: batch 0 wall 5s > clamp 2s (OverBudget); batch 1 wall 1s < clamp 2s (WithinBudget).
        let clamped_records = vec![
            BatchRecord {
                batch_index: 0,
                wall_nanos: 5_000_000_000, // 5s
                clamp_ms: Some(2_000),     // 2s
                unit_count: 3,
                results: Vec::new(),
                label: "batch-0".to_string(),
                selection_tag: "off",
            },
            BatchRecord {
                batch_index: 1,
                wall_nanos: 1_000_000_000, // 1s
                clamp_ms: Some(2_000),     // 2s
                unit_count: 0,
                results: Vec::new(),
                label: "batch-1".to_string(),
                selection_tag: "off",
            },
        ];
        assert!(write_batch_wall_receipt_at(&base, &clamped_records));
        let body = fs::read_to_string(base.join("floor-batch-wall-receipt.txt")).unwrap();
        assert!(body.contains("batch_1_wall_ms=5000"));
        assert!(body.contains("batch_1_units=3"));
        assert!(body.contains("batch_1_clamp_ms=2000"));
        assert!(body.contains("batch_1_verdict=OverBudget"));
        assert!(body.contains("batch_2_verdict=WithinBudget"));
        assert!(body.contains("over_budget_batches=1"));
        // Clamp-less (falsifier/regen plans): records carry clamp_ms None -> Unbudgeted.
        let unbudgeted_records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 5_000_000_000,
            clamp_ms: None,
            unit_count: 0,
            results: Vec::new(),
            label: "batch-0".to_string(),
            selection_tag: "off",
        }];
        assert!(write_batch_wall_receipt_at(&base, &unbudgeted_records));
        let body = fs::read_to_string(base.join("floor-batch-wall-receipt.txt")).unwrap();
        assert!(body.contains("batch_1_verdict=Unbudgeted"));
        assert!(body.contains("over_budget_batches=0"));
        let _ = fs::remove_dir_all(&base);
    }

    // One entry, one resolve (lane clause (c)): a negligible-profile SharedClaims
    // group whose (entry, mode) some batch resolves on the memo path is itself
    // routed to the memo lane, so the entry's closure resolves once on the main
    // thread (thread-local `process_shared_index`) instead of cold on a spawned
    // thread — the #7088 cheap-gate batch-0 shape (resolve receipt 4).
    #[test]
    fn same_entry_shared_claims_promote_to_memo_lane_across_batches() {
        let cheap = |f: &str| Runnable::SingleClaim {
            entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
            function: f.to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        };
        let heavy = Runnable::SingleClaim {
            entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
            function: "dag_compile_clean_gate_passes".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: true,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        };
        let batches = vec![
            vec![
                cheap("extdeps_external_authority_gate_passes"),
                cheap("generated_artifact_drift_gate_passes"),
            ],
            vec![heavy],
        ];
        let keys = memo_path_entry_keys(&batches);
        assert!(keys.contains(&(
            "dag/tools/floor_effect_gate_witness.dag".to_string(),
            ExecutionMode::Wet
        )));
        let empty_memo = std::collections::HashMap::new();
        let cheap_units = group_batch_units(&batches[0]);
        assert_eq!(
            cheap_units.len(),
            1,
            "same-entry claims coalesce into one resolve-group"
        );
        assert_eq!(
            batch_unit_lane(&cheap_units[0], &empty_memo, &keys),
            UnitLane::Memo,
            "batch-0 group must ride the memo lane when a later batch resolves the same entry there"
        );
        // RED control — the promotion is exactly the heavy same-entry declaration:
        // without it the group spawns (the pre-fix behavior).
        let no_heavy = memo_path_entry_keys(&batches[..1]);
        assert!(no_heavy.is_empty());
        assert_eq!(
            batch_unit_lane(&cheap_units[0], &empty_memo, &no_heavy),
            UnitLane::Spawned
        );
    }

    // Mode is part of the promotion key: a Hermetic group must not share a Wet
    // entry's memo context (the cached InterpContext carries its effect envelope).
    #[test]
    fn memo_lane_promotion_is_mode_keyed() {
        let batches = vec![vec![Runnable::SingleClaim {
            entry: "dag/x.dag".to_string(),
            function: "f".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: true,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        }]];
        let keys = memo_path_entry_keys(&batches);
        let hermetic_units = group_batch_units(&[Runnable::SingleClaim {
            entry: "dag/x.dag".to_string(),
            function: "g".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Hermetic,
            },
        }]);
        let empty_memo = std::collections::HashMap::new();
        assert_eq!(
            batch_unit_lane(&hermetic_units[0], &empty_memo, &keys),
            UnitLane::Spawned,
            "a Wet memo entry must not promote a Hermetic group on the same entry"
        );
    }

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

    // The seed→.dag render boundary by execution: `render_phase_concluded_line`
    // resolves `gunbc.observation_seed_render` and projects a floor phase mark through
    // the single-authority renderer, so the seed speaks the observation vocabulary
    // rather than a raw `[t+…]` byte string. Discriminating RED: a helper that fell
    // back to the raw marker, dropped the phase, or forked the duration format fails
    // one of the three asserts below (human units, completed glyph, no old marker).
    #[test]
    fn phase_mark_renders_through_the_observation_render_authority() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // Emoji tier (the CI target): 48s concludes as `✅ naming-hygiene walk done in
        // 48 seconds`. overhead_ms is the placement basis only (the 300s clamp overhead).
        let line =
            render_phase_concluded_line(&roots, "naming-hygiene walk", 48_000, 300_000, true)
                .expect("observation_seed_render must resolve and render a phase line");
        assert!(
            line.contains("naming-hygiene walk") && line.contains("done in 48 seconds"),
            "phase line must name the phase in human units: {line:?}"
        );
        assert!(
            line.starts_with('✅'),
            "a Done outcome concludes with the completed glyph at the Emoji tier: {line:?}"
        );
        assert!(
            !line.contains("[t+"),
            "the projection must not carry the deleted raw phase marker: {line:?}"
        );
    }

    fn run_seed_phase_begin_line(
        source_roots: &[String],
        phase: &str,
        overhead_ms: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "phase_begin_line",
            &[
                (Some("phase".to_string()), Value::Str(phase.to_string())),
                (
                    Some("overhead_ms".to_string()),
                    Value::Int(overhead_ms as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    // Gantt flip byte-oracle: compile-path mirrors of phase_begin_line /
    // phase_concluded_line must stay byte-equal to the .dag seed (justified
    // divergence — interpreter render from inside compile would recurse).
    #[test]
    fn gantt_phase_mirrors_match_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let begin_oracle = run_seed_phase_begin_line(&roots, "compile.frontend", 300_000, true)
            .expect("phase_begin_line must resolve and render");
        let begin_mirror =
            v1_compiler::v1_rt::render_phase_begin_line_mirror("compile.frontend", true);
        assert_eq!(
            begin_oracle, begin_mirror,
            "begin mirror must be byte-equal to seed oracle"
        );
        assert!(
            begin_oracle.starts_with('🔄') && begin_oracle.contains("started compile.frontend"),
            "begin line shape: {begin_oracle:?}"
        );

        let done_oracle =
            render_phase_concluded_line(&roots, "compile.frontend", 12_000, 300_000, true)
                .expect("phase_concluded_line must resolve and render");
        let done_mirror = v1_compiler::v1_rt::render_phase_concluded_line_mirror(
            "compile.frontend",
            12_000,
            true,
        );
        assert_eq!(
            done_oracle, done_mirror,
            "concluded mirror must be byte-equal to seed oracle"
        );
        assert!(
            done_oracle.contains("compile.frontend done in 12 seconds"),
            "concluded line shape: {done_oracle:?}"
        );
    }

    fn run_seed_psi_hold_line(
        source_roots: &[String],
        avg10_bp: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_psi_hold_line",
            &[
                (Some("avg10_bp".to_string()), Value::Int(avg10_bp as i64)),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    fn run_seed_high_water_hold_line(
        source_roots: &[String],
        current_bytes: u64,
        high_water_bytes: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_high_water_hold_line",
            &[
                (
                    Some("current_bytes".to_string()),
                    Value::Int(current_bytes as i64),
                ),
                (
                    Some("high_water_bytes".to_string()),
                    Value::Int(high_water_bytes as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    // Governor flip byte-oracle: HoldReason → ci_hold_cause_text mirrors must stay
    // byte-equal to seed_psi_hold_line / seed_high_water_hold_line.
    #[test]
    fn governor_hold_mirrors_match_seed_oracle() {
        use v1_compiler::memory_governor::{render_governor_hold_line_mirror, HoldReason};
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // 37.5% → 3750 basis points
        let psi_oracle = run_seed_psi_hold_line(&roots, 3750, true)
            .expect("seed_psi_hold_line must resolve and render");
        let psi_mirror =
            render_governor_hold_line_mirror(&HoldReason::PsiPressure { avg10: 37.5 }, true);
        assert_eq!(
            psi_oracle, psi_mirror,
            "psi hold mirror must be byte-equal to seed oracle"
        );
        assert!(
            psi_oracle.starts_with('⏳') && psi_oracle.contains("blocked on memory reclaim"),
            "psi hold shape: {psi_oracle:?}"
        );

        let hw_oracle = run_seed_high_water_hold_line(&roots, 8_589_934_592, 10_737_418_240, true)
            .expect("seed_high_water_hold_line must resolve and render");
        let hw_mirror = render_governor_hold_line_mirror(
            &HoldReason::CurrentHighWater {
                current: 8_589_934_592,
                high_water: 10_737_418_240,
            },
            true,
        );
        assert_eq!(
            hw_oracle, hw_mirror,
            "high-water hold mirror must be byte-equal to seed oracle"
        );
        assert!(
            hw_oracle.contains("blocked on the memory high-water line"),
            "high-water hold shape: {hw_oracle:?}"
        );
    }

    fn run_seed_typecheck_concluded_line(
        source_roots: &[String],
        module_path: &str,
        elapsed_ms: u64,
        overhead_ms: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "typecheck_concluded_line",
            &[
                (
                    Some("module_path".to_string()),
                    Value::Str(module_path.to_string()),
                ),
                (
                    Some("elapsed_ms".to_string()),
                    Value::Int(elapsed_ms as i64),
                ),
                (
                    Some("overhead_ms".to_string()),
                    Value::Int(overhead_ms as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    #[test]
    fn typecheck_attribution_mirrors_match_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let oracle = run_seed_typecheck_concluded_line(
            &roots,
            "v2.compiler.normalized_tree",
            606_984,
            300_000,
            true,
        )
        .expect("typecheck_concluded_line must resolve and render");
        let mirror = v1_compiler::cli_run::render_typecheck_concluded_line_mirror(
            "v2.compiler.normalized_tree",
            606_984,
            true,
        );
        assert_eq!(
            oracle, mirror,
            "typecheck concluded mirror must be byte-equal to seed oracle"
        );
        assert!(
            oracle.starts_with('✅')
                && oracle.contains("typecheck v2.compiler.normalized_tree done in 10 minutes"),
            "typecheck concluded shape: {oracle:?}"
        );
        let begin_mirror = v1_compiler::cli_run::render_typecheck_begin_line_mirror(
            "v2.compiler.normalized_tree",
            true,
        );
        assert_eq!(
            begin_mirror,
            "🔄 started typecheck v2.compiler.normalized_tree"
        );
    }

    fn run_seed_peak_rss_line(
        source_roots: &[String],
        label: &str,
        rss_bytes: u64,
        rss_available: bool,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_peak_rss_line",
            &[
                (Some("label".to_string()), Value::Str(label.to_string())),
                (Some("rss_bytes".to_string()), Value::Int(rss_bytes as i64)),
                (
                    Some("rss_available".to_string()),
                    Value::Bool(rss_available),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    #[test]
    fn peak_rss_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // 1 GiB exact → "1.0 GiB"
        let oracle = run_seed_peak_rss_line(&roots, "floor peak RSS", 1_073_741_824, true, true)
            .expect("seed_peak_rss_line must resolve and render");
        let mirror = v1_compiler::cli_run::render_peak_rss_line_mirror(
            "floor peak RSS",
            Some(1_073_741_824),
            true,
        );
        assert_eq!(oracle, mirror, "peak RSS mirror must be byte-equal to seed");
        assert_eq!(oracle, "🕐 floor peak RSS — 1.0 GiB");

        // 1536 MiB → "1.5 GiB"
        let oracle15 = run_seed_peak_rss_line(&roots, "cgroup peak", 1_610_612_736, true, false)
            .expect("seed");
        let mirror15 = v1_compiler::cli_run::render_peak_rss_line_mirror(
            "cgroup peak",
            Some(1_610_612_736),
            false,
        );
        assert_eq!(oracle15, mirror15);
        assert_eq!(oracle15, "◷ cgroup peak — 1.5 GiB");

        let unread = run_seed_peak_rss_line(&roots, "floor peak RSS", 0, false, true)
            .expect("unreadable seed");
        let unread_mirror =
            v1_compiler::cli_run::render_peak_rss_line_mirror("floor peak RSS", None, true);
        assert_eq!(unread, unread_mirror);
        assert!(unread.contains("unreadable (no /proc/self/status)"));
    }

    fn run_seed_shell_effect_failed_line(
        source_roots: &[String],
        intent: &str,
        argv_collapsed: &str,
        exit_code: u64,
        elapsed_ms: u64,
        overhead_ms: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "shell_effect_failed_line",
            &[
                (Some("intent".to_string()), Value::Str(intent.to_string())),
                (
                    Some("argv_collapsed".to_string()),
                    Value::Str(argv_collapsed.to_string()),
                ),
                (Some("exit_code".to_string()), Value::Int(exit_code as i64)),
                (
                    Some("elapsed_ms".to_string()),
                    Value::Int(elapsed_ms as i64),
                ),
                (
                    Some("overhead_ms".to_string()),
                    Value::Int(overhead_ms as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    #[test]
    fn shell_effect_failed_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let oracle = run_seed_shell_effect_failed_line(
            &roots,
            "shell.Exec.Run",
            "echo hi",
            1,
            2000,
            300_000,
            true,
        )
        .expect("shell_effect_failed_line must resolve and render");
        let mirror = v1_compiler::v1_interpreter::render_shell_effect_failed_line_mirror(
            "shell.Exec.Run",
            "echo hi",
            1,
            2000,
            true,
        );
        assert_eq!(
            oracle, mirror,
            "shell Failed mirror must be byte-equal to seed oracle"
        );
        assert_eq!(
            oracle,
            "❌ shell.Exec.Run failed: $ echo hi (exit=1) in 2 seconds"
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn run_seed_heartbeat_line(
        source_roots: &[String],
        elapsed_ms: u64,
        batch_label: &str,
        entry_index: u64,
        entry_total: u64,
        rss: Option<u64>,
        swap: Option<u64>,
        pressure: Option<u64>,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_heartbeat_line",
            &[
                (Some("elapsed_ms".into()), Value::Int(elapsed_ms as i64)),
                (Some("batch_index".into()), Value::Int(0)),
                (
                    Some("batch_label".into()),
                    Value::Str(batch_label.to_string()),
                ),
                (Some("entry_index".into()), Value::Int(entry_index as i64)),
                (Some("entry_total".into()), Value::Int(entry_total as i64)),
                (
                    Some("rss_bytes".into()),
                    Value::Int(rss.unwrap_or(0) as i64),
                ),
                (Some("rss_available".into()), Value::Bool(rss.is_some())),
                (
                    Some("swap_bytes".into()),
                    Value::Int(swap.unwrap_or(0) as i64),
                ),
                (Some("swap_available".into()), Value::Bool(swap.is_some())),
                (
                    Some("pressure_bp".into()),
                    Value::Int(pressure.unwrap_or(0) as i64),
                ),
                (
                    Some("pressure_available".into()),
                    Value::Bool(pressure.is_some()),
                ),
                (Some("emoji".into()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    // The floor-memory heartbeat's seed→.dag boundary, proven by execution:
    // seed_heartbeat_line takes the primitives the heartbeat thread has (elapsed,
    // batch label, entry position, memory vitals) and projects them through the one
    // renderer — identity first, human units, no raw byte dump. These golden strings
    // are the oracle the Rust mirror is proven byte-equal to
    // (`render_heartbeat_line_mirror_matches_seed_oracle`); the subject is batch-grain
    // (parallel entries → no fabricated per-module detail).
    #[test]
    fn seed_heartbeat_line_renders_identity_first_in_human_units() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // The captured crawl window's memory beat: identity first, human units, the
        // raw byte value absent.
        let line = run_seed_heartbeat_line(
            &roots,
            1_980_000,
            "witness discovery",
            214,
            602,
            Some(16_107_200_512),
            Some(34_359_738_368),
            Some(901),
            true,
        )
        .expect("seed_heartbeat_line must resolve and render");
        assert_eq!(
            line,
            "🕐 33 minutes in — still in witness discovery: entry 214 of 602. memory 15.0 GiB, swap 32.0 GiB, pressure 9.0%"
        );
        assert!(
            !line.contains("16107200512"),
            "raw bytes must not appear: {line}"
        );

        // An unreadable cgroup field names its cause, never a fabricated zero.
        let unreadable = run_seed_heartbeat_line(
            &roots,
            500,
            "self-host fixed-point",
            0,
            2,
            None,
            Some(0),
            None,
            true,
        )
        .expect("resolve");
        assert_eq!(
            unreadable,
            "🕐 500ms in — still in self-host fixed-point: entry 0 of 2. memory unreadable (cgroup field unreadable), swap 0.0 GiB, pressure unreadable (cgroup field unreadable)"
        );
    }

    // Wiring flip 4b: the Rust mirror is proven byte-equal to the 4a seed oracle.
    // The heartbeat thread cannot call the interpreter (duplicate module index under
    // the memory envelope it watches); this pin is what keeps the mirror honest.
    #[test]
    fn render_heartbeat_line_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let crawl = run_seed_heartbeat_line(
            &roots,
            1_980_000,
            "witness discovery",
            214,
            602,
            Some(16_107_200_512),
            Some(34_359_738_368),
            Some(901),
            true,
        )
        .expect("seed oracle");
        let mirror = render_heartbeat_line_mirror(
            1_980_000,
            "witness discovery",
            214,
            602,
            Some(16_107_200_512),
            Some(34_359_738_368),
            Some(901),
            true,
        );
        assert_eq!(
            mirror, crawl,
            "mirror must be byte-equal to the seed oracle"
        );
        assert_eq!(
            mirror,
            "🕐 33 minutes in — still in witness discovery: entry 214 of 602. memory 15.0 GiB, swap 32.0 GiB, pressure 9.0%"
        );

        let unreadable_oracle = run_seed_heartbeat_line(
            &roots,
            500,
            "self-host fixed-point",
            0,
            2,
            None,
            Some(0),
            None,
            true,
        )
        .expect("seed oracle");
        let unreadable_mirror = render_heartbeat_line_mirror(
            500,
            "self-host fixed-point",
            0,
            2,
            None,
            Some(0),
            None,
            true,
        );
        assert_eq!(
            unreadable_mirror, unreadable_oracle,
            "unreadable fields must stay byte-equal"
        );
        assert!(
            !mirror.contains("16107200512") && !unreadable_mirror.contains("[floor-memory]"),
            "mirror must not carry the deleted byte-dump shape"
        );
    }

    #[test]
    fn psi_avg10_converts_to_basis_points_at_observation_scale() {
        assert_eq!(psi_avg10_to_basis_points("9.01"), Some(901));
        assert_eq!(psi_avg10_to_basis_points("37.5"), Some(3750));
        assert_eq!(psi_avg10_to_basis_points("0.0"), Some(0));
        assert_eq!(psi_avg10_to_basis_points("garbage"), None);
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
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Hermetic,
            },
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

    /// The committed basis file must actually load. A basis that parses to zero rows
    /// pins `drift_exceeded` at 0 by construction and reports it as "no drift" — the
    /// state this file spent its whole life in (`basis_absent=5344` on run 30550328673),
    /// where the loud missing-file diagnostic never fired because the file *existed* and
    /// was merely empty of data. So this asserts the artifact is non-vacuous, not that
    /// the parser works in the abstract.
    #[test]
    fn committed_witness_row_cost_basis_loads_and_excludes_the_censored_row() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("dag/gunbc/witness_row_cost_basis.tsv");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("basis file unreadable at {}: {e}", path.display()));

        let mut loaded = 0usize;
        for line in text.lines().skip(1) {
            match parse_witness_row_cost_basis_line(line) {
                Ok(Some(_)) => loaded += 1,
                Ok(None) => {}
                // A refused row is silently skipped by the loader, so a malformed commit
                // would degrade to BasisAbsent instead of failing. Catch it here instead.
                Err(msg) => panic!("committed basis row refused by the loader: {msg}"),
            }
        }
        assert!(
            loaded >= 1000,
            "committed basis loaded only {loaded} row(s) — a near-empty basis reports \
             drift_exceeded=0 by construction, never by measurement"
        );

        // The corpus's most expensive row was KILLED at its 900000ms deadline, so its
        // recorded 900794ms is a censored measurement, not a completed one. Seeding it
        // would pin it WithinBasis below ~1.8M ms under the 2x comparator and enshrine a
        // deadline ceiling as normal cost. It must stay BasisAbsent — the honest third
        // state — until ClaimOutcome::TimedOut can distinguish killed from completed.
        for line in text.lines() {
            if line.trim_start().starts_with('#') {
                continue;
            }
            assert!(
                !line.contains("resolution_divergence_silent_pick_gate_keystone_holds"),
                "censored (deadline-killed) row must not be seeded as a basis: {line}"
            );
        }
    }

    #[test]
    fn parse_witness_row_cost_basis_line_requires_srv_fleet_arm64() {
        // RED control for review 43284: wrong/missing host_class must refuse, never load.
        let ok = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64")
            .expect("parse")
            .expect("row");
        assert_eq!(ok.0, ("e.dag".to_string(), "f".to_string()));
        assert_eq!(ok.1.eval_ms_basis, 10);
        assert_eq!(ok.1.run_ref, "run-1");

        assert!(parse_witness_row_cost_basis_line("# comment")
            .unwrap()
            .is_none());
        assert!(parse_witness_row_cost_basis_line("").unwrap().is_none());

        let wrong = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tlocal_x86")
            .expect_err("wrong host_class must refuse");
        assert!(
            wrong.contains("host_class") && wrong.contains("srv_fleet_arm64"),
            "expected host_class refusal, got: {wrong}"
        );

        let short =
            parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1").expect_err("need 5 cols");
        assert!(short.contains("need 5 cols"), "got: {short}");

        let zero = parse_witness_row_cost_basis_line("e.dag\tf\t0\trun-1\tsrv_fleet_arm64")
            .expect_err("zero eval must refuse");
        assert!(zero.contains("zero eval_ms_basis"), "got: {zero}");
    }

    /// Discovery is THE falsifier path — `resolution_divergence_silent_pick_gate_keystone_holds`
    /// is a discovery row — so a budget kill there must classify structurally like any other.
    ///
    /// RED control for review 45220, which caught this as a live regression: a discovery batch
    /// flattens N witness outcomes into one `ok`/`detail`, and this result previously hardcoded
    /// `budget_refusal: None`. Combined with the new detail wording (which deliberately contains
    /// none of `falsifier_failure_mode`'s substrings), a budget kill on the primary path fell
    /// through to `WitnessRed` — the exact misclassification this change exists to remove, in
    /// new prose. The assertion below on the string classifier keeps the control non-vacuous.
    #[test]
    fn discovery_budget_kill_classifies_structurally_on_the_falsifier_path() {
        use v1_compiler::cli_run::{
            ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
            ResolveStageNanos,
        };
        let killed_detail =
            "1 of 1 discovery witness(es) failed: fn=silent_pick killed at its wall budget: \
             900001ms elapsed > 900000ms budget";
        assert_eq!(
            falsifier_failure_mode(&[killed_detail.to_string()]),
            "WitnessRed",
            "control is only meaningful if the string path would misclassify this detail"
        );

        let summary_with = |outcome: ClaimOutcome| DiscoverySummary {
            total: 1,
            passed: 0,
            skipped: 0,
            deferred_rows: Vec::new(),
            predicted_unaffected: Vec::new(),
            divergences: Vec::new(),
            failures: vec![killed_detail.into()],
            witness_outcomes: vec![DiscoveryWitnessOutcome {
                entry: "dag/test/claim/resolution_divergence_silent_pick_gate_witness_test.dag"
                    .into(),
                module_path: "test.claim.resolution_divergence_silent_pick_gate".into(),
                function: "resolution_divergence_silent_pick_gate_keystone_holds".into(),
                outcome,
                execution_leg: "InterpretedLeg".into(),
            }],
            entry_resolve_receipts: Vec::<EntryResolveReceipt>::new(),
            total_resolve_nanos: 0,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: Vec::new(),
            total_measured_nanos: 0,
            roster_closure_nodes: 0,
        };
        let killed = ClaimOutcome::TimedOut {
            elapsed_ms: 900_001,
            budget_ms: 900_000,
            kind: BudgetKind::Wall,
        };

        // Both arms: the receipt projection may succeed or refuse, and a receipt refusal must
        // ADD a cause rather than erase the budget kill already established.
        for projected in [
            Ok(Vec::new()),
            Err("[witness-row-cost] REFUSED: missing measured resolve parent".to_string()),
        ] {
            let result = discovery_claim_result(
                "discovery-corpus".into(),
                false,
                killed_detail.to_string(),
                &summary_with(killed.clone()),
                projected,
            );
            assert!(
                result.budget_refusal.is_some(),
                "discovery batch must lift the budget kill out of witness_outcomes"
            );
            let (mode, _) = batch_failure_mode_and_detail(&batch_record_for_test(vec![result]));
            assert_eq!(mode, "BudgetExceeded");
        }

        // And a discovery batch with an ordinary red must NOT be dragged into BudgetExceeded.
        let result = discovery_claim_result(
            "discovery-corpus".into(),
            false,
            "red".into(),
            &summary_with(ClaimOutcome::Fail),
            Ok(Vec::new()),
        );
        assert!(result.budget_refusal.is_none());
    }

    #[test]
    fn discovery_claim_result_preserves_caller_detail_on_receipt_refuse() {
        // RED control for review 43284: receipt refusal must append, never overwrite
        // the discovery failure diagnostic.
        use v1_compiler::cli_run::{
            ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
            ResolveStageNanos,
        };
        let summary = DiscoverySummary {
            total: 1,
            passed: 0,
            skipped: 0,
            deferred_rows: Vec::new(),
            predicted_unaffected: Vec::new(),
            divergences: Vec::new(),
            failures: vec!["e.dag::f failed".into()],
            witness_outcomes: vec![DiscoveryWitnessOutcome {
                entry: "e.dag".into(),
                module_path: "test.e".into(),
                function: "f".into(),
                outcome: ClaimOutcome::Fail,
                execution_leg: "InterpretedLeg".into(),
            }],
            // Empty entry_resolve_receipts → the authored receipt projection refuses.
            entry_resolve_receipts: Vec::<EntryResolveReceipt>::new(),
            total_resolve_nanos: 0,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: Vec::new(),
            total_measured_nanos: 0,
            roster_closure_nodes: 0,
        };
        let prior = "1 of 1 discovery witness(es) failed: e.dag::f failed";
        let result = discovery_claim_result(
            "probe".into(),
            false,
            prior.to_string(),
            &summary,
            Err("[witness-row-cost] REFUSED: missing measured resolve parent for e.dag".into()),
        );
        assert!(!result.ok);
        assert!(
            result.detail.contains(prior),
            "caller discovery failure must be preserved, got: {}",
            result.detail
        );
        assert!(
            result.detail.contains("witness row-cost receipt refused"),
            "receipt refusal must also be present, got: {}",
            result.detail
        );
    }

    fn latch_result(function: &str) -> ClaimResult {
        ClaimResult {
            function: function.to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            witness_row_costs: Vec::new(),
            budget_refusal: None,
        }
    }

    /// LATCH, not timing. Each member increments a shared counter, then waits on a
    /// condvar until the counter reaches 2 — which can only happen if the OTHER member
    /// is already running. Under a serial executor the first member waits forever for a
    /// peer that has not been started, so the bounded wait expires and `observed_peer`
    /// stays false. The bound is a deadlock detector, never the assertion: the assertion
    /// is that each member SAW the other, which no amount of slowness can fake and no
    /// amount of speed can satisfy serially.
    #[test]
    fn stage_members_actually_overlap() {
        use std::sync::{Arc, Condvar, Mutex};

        let state = Arc::new((Mutex::new(0usize), Condvar::new()));
        let observed_peer = Arc::new(Mutex::new(vec![false, false]));

        let work: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>> = (0..2)
            .map(|i| {
                let state = state.clone();
                let observed = observed_peer.clone();
                let boxed: Box<dyn FnOnce() -> Vec<ClaimResult> + Send> = Box::new(move || {
                    let (lock, cvar) = &*state;
                    let mut count = lock.lock().unwrap();
                    *count += 1;
                    cvar.notify_all();
                    let mut saw_peer = *count == 2;
                    while !saw_peer {
                        let (guard, timeout) = cvar
                            .wait_timeout(count, std::time::Duration::from_secs(10))
                            .unwrap();
                        count = guard;
                        saw_peer = *count == 2;
                        if timeout.timed_out() {
                            break;
                        }
                    }
                    observed.lock().unwrap()[i] = saw_peer;
                    vec![latch_result(&format!("member-{i}"))]
                });
                boxed
            })
            .collect();

        let (results, panicked) = join_units(spawn_units(work));
        assert!(!panicked, "no member should panic");
        assert_eq!(results.len(), 2, "both members must report");
        let observed = observed_peer.lock().unwrap();
        assert!(
            observed[0] && observed[1],
            "each member must observe the other running concurrently; \
             serial execution leaves one or both false: {observed:?}"
        );
    }

    /// The other half of the barrier: the join returns only after EVERY member finished.
    /// The slow member sets a flag last; if `join_units` returned early, the flag would
    /// still be false when the assertion runs, and the result count would be short.
    /// Together with the stage loop's sequential `&mut stage_memo` borrow, this is what
    /// makes "stage N+1 cannot begin before every stage-N member completed" true.
    #[test]
    fn join_waits_for_every_member_before_returning() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let finished = Arc::new(AtomicUsize::new(0));
        let members = 4usize;
        let work: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>> = (0..members)
            .map(|i| {
                let finished = finished.clone();
                let boxed: Box<dyn FnOnce() -> Vec<ClaimResult> + Send> = Box::new(move || {
                    // Staggered so at least one member is still running well after the
                    // others have returned — the case an early join would expose.
                    std::thread::sleep(std::time::Duration::from_millis(20 * (i as u64 + 1)));
                    finished.fetch_add(1, Ordering::SeqCst);
                    vec![latch_result(&format!("member-{i}"))]
                });
                boxed
            })
            .collect();

        let (results, panicked) = join_units(spawn_units(work));
        assert!(!panicked);
        assert_eq!(
            finished.load(Ordering::SeqCst),
            members,
            "join must not return before every member completed"
        );
        assert_eq!(
            results.len(),
            members,
            "every member's results must be collected"
        );
    }

    /// A panicking member is collected as an INFRA fault, not silently dropped and not
    /// propagated: the caller still needs to close its host-effect group and report the
    /// fault distinctly from a claim verdict. The surviving members' results still come
    /// back, so one bad unit does not erase the stage's evidence.
    #[test]
    fn join_reports_a_panicking_member_without_losing_the_others() {
        let work: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>> = vec![
            Box::new(|| vec![latch_result("ok-member")]),
            Box::new(|| panic!("member exploded")),
        ];
        let (results, panicked) = join_units(spawn_units(work));
        assert!(panicked, "a panicking member must be reported");
        assert_eq!(
            results.len(),
            1,
            "the surviving member's results must still be collected"
        );
    }

    // ---- walk-attempt identity ----
    //
    // The refusals below are the point of the feature, so each is asserted as a REFUSAL,
    // not merely as "not equal to the good value". The positive controls exist so the
    // negatives are discriminating: a `compose_walk_attempt_id` that refused everything
    // would pass every Err assertion and fail these.

    #[test]
    fn attempt_identity_composes_from_the_github_triple() {
        assert_eq!(
            compose_walk_attempt_id("", "30654655022", "1", "ci").unwrap(),
            "30654655022-1-ci"
        );
    }

    #[test]
    fn attempt_identity_prefers_the_explicit_value() {
        assert_eq!(
            compose_walk_attempt_id("local-probe-7", "30654655022", "1", "ci").unwrap(),
            "local-probe-7",
            "an explicitly supplied identity must win over the ambient GitHub triple"
        );
    }

    #[test]
    fn attempt_identity_refuses_rather_than_defaulting_when_absent() {
        // THE load-bearing control. The ruling forbids "a silent constant like a bare
        // local, which would make every local run one attempt and the wrong-attempt
        // refusal unreachable off CI". If someone ever adds a default, this reds.
        let refusal = compose_walk_attempt_id("", "", "", "")
            .expect_err("an unidentified walk must refuse, never default");
        assert!(
            refusal.contains("GUNBC_WALK_ATTEMPT_ID"),
            "the refusal must name the input that would satisfy it, got: {refusal}"
        );
    }

    #[test]
    fn attempt_identity_refuses_a_partial_github_triple() {
        // A partial triple is the shape a non-`ci` job or a changed workflow produces.
        // Composing from it would silently collide two jobs of one run onto one identity.
        for (run_id, attempt, job) in [
            ("30654655022", "1", ""),
            ("30654655022", "", "ci"),
            ("", "1", "ci"),
        ] {
            assert!(
                compose_walk_attempt_id("", run_id, attempt, job).is_err(),
                "partial triple ({run_id:?},{attempt:?},{job:?}) must refuse"
            );
        }
    }

    #[test]
    fn attempt_identity_refuses_every_unsafe_path_segment() {
        // Mirrors `std.types.path_segment_is_safe` clause for clause. `..` and `/` are the
        // escapes that would let a receipt be written outside its attempt directory; CR and
        // LF are the ones that would split the line-oriented receipt written under that
        // name, so two lines could be forged from one field.
        for bad in ["", ".", "..", "a/b", "a\\b", "a\nb", "a\rb", "a\0b"] {
            assert!(
                compose_walk_attempt_id(bad, "run", "1", "ci").is_err() || bad.trim().is_empty(),
                "unsafe explicit segment {bad:?} must refuse"
            );
            assert!(
                !walk_attempt_id_segment_is_safe(bad),
                "segment {bad:?} must not be considered safe"
            );
        }
        assert!(
            walk_attempt_id_segment_is_safe("30654655022-1-ci"),
            "a real composed identity must be accepted — else the negatives above prove nothing"
        );
    }
}
