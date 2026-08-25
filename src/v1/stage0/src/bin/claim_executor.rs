#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[cfg(test)]
use v1_compiler::cli_run::workspace_root;
use v1_compiler::cli_run::{
    active_workset_admit, active_workset_complete, build_floor_discovery_request,
    compute_histogram_data, discover_floor_witness_roster_with_snapshot,
    enable_floor_compile_clean_lazy_install, heartbeat_feed_enter_batch,
    heartbeat_feed_entry_completed, heartbeat_feed_snapshot, install_floor_compile_clean_receipt,
    make_eval_context, project_witness_cost_receipt, resolve_entry_graph,
    resolve_entry_graph_shared, run_claim, run_discovery_corpus_with_options, run_value, set_phase,
    top_n_slowest_witnesses, verify_floor_discovery_terminal_for_coordinator, BudgetKind,
    ClaimOutcome, DiscoveryCorpusOptions, DiscoverySummary, DiscoveryWidthPolicy,
    DiscoveryWitnessOutcome, FloorDiscoveryConsumerRole, FloorPhase, HistogramData, PhaseProfile,
    TimingPercentiles, WitnessRowCost, DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N,
};
use v1_compiler::derived_realization_schedule::{RealizationConcurrency, RealizationSlot};
use v1_compiler::memory_governor::{
    binding_cap_cgroup_dir, binding_high_cgroup_dir, floor_budget_below_minimum_footprint,
    leaf_cgroup_dir, mem_total_bytes, memory_pressure_some_avg10, read_cgroup_raw, read_cgroup_u64,
};
use v1_compiler::v1_interpreter::{
    color_enabled, paint, run_in_context, run_in_context_with_args, sgr, str_value, ExecutionMode,
    InterpContext, Value,
};

/// Per-LANE budgets for the falsifier's rostered batches, each keyed by the lane's own
/// roster so a batch draws exactly the ceiling its lane declares. Self-host wet (green +
/// known-red quarantine) share the 600s wall budget; the Hermetic substrate long lane owns
/// `gunbc_falsifier_substrate_long_lane_witness_eval_budget`. No lane inherits another's
/// ceiling — the substrate long lane must not inherit the 5s per-PR fast-lane eval budget
/// (run 30176416535, 7 of 10 rows killed at ~5001ms).
#[derive(Clone, Default)]
struct FalsifierSelfHostWetBudgets {
    wall_budget_ms: Option<u64>,
    interp_eval_budget_ms: Option<u64>,
    /// Entry paths from `falsifier_self_host_wet_entries` (green wet roster).
    roster_entry_paths: Vec<String>,
    /// Entry paths from `falsifier_self_host_wet_known_red_entries` — the Wet quarantine's
    /// WALL budget only. Its polarity comes from `expected_red_witnesses` like every other
    /// witness's; the path-grain hermetic twin of this field was deleted when function-grain
    /// expectation replaced it, rather than left beside its successor.
    known_red_entry_paths: Vec<String>,
    /// Paths requiring the long eval ceiling (`witness_long_eval_budget_entries`) — the
    /// UNION of the long-lane batch roster and every admission row declaring
    /// `SubstrateLongLaneEvalBudget`.
    ///
    /// It reads the union, not the batch roster, because how much eval time a witness needs
    /// is a property of the WITNESS and batch membership is a property of the schedule.
    /// While this read was the batch roster the two were one fact, so an expensive known-red
    /// row could only obtain its ceiling by ALSO joining the long batch — a second schedule
    /// occurrence, on a batch that expects green, for a witness that is red by design.
    substrate_long_lane_entry_paths: Vec<String>,
    substrate_long_lane_eval_budget_ms: Option<u64>,
    /// The FUNCTION-grain expected-red roster: `(entry, function)` from both known-red
    /// cadences. An empty function is the declared file-grain form.
    expected_red_witnesses: Vec<(String, String)>,
    /// The strict SUBSET of the above declaring `ExpectTypedPreVerdictRefusal` — witnesses whose
    /// agreement is a typed stop BEFORE any assertion runs, so that a corpus resolve refuse is
    /// their expected result rather than an infrastructure fact. Separate from
    /// `expected_red_witnesses` because blessing an arbitrary resolve failure as every
    /// quarantine holding is exactly what one undifferentiated red expectation did.
    pre_verdict_refusal_witnesses: Vec<(String, String)>,
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

/// Where a resolved floor-batch clamp came from — seed projection of the `authority: DeclarationRef`
/// field on `std.realization_schedule` `RunnableBatchClamp`.
///
/// Two independently owned populations feed one aligned clamp list: the positional
/// `gunbc.ci_spec` rows for ordinary batches, and the scoped witness batch's own row in
/// `gunbc.ci_layer_roots`. This executor used to reconstruct the origin from the LIST POSITION and
/// spell it in the refusal's format string, which is how a breach came to report `clamp_ms=360000`
/// while citing `gunbc_ci_floor_batch_clamp_params[0]` — a row whose overhead is 240 seconds. The
/// authority now travels WITH the value; the index is carried beside it because an offset into a
/// list is the one part of the citation no symbol can name.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum FloorBatchClampAuthority {
    PositionalCiSpecClamp {
        module_path: String,
        decl_name: String,
        index: usize,
    },
    ScopedBatchOwnedClamp {
        batch_id: String,
        module_path: String,
        decl_name: String,
    },
}

impl FloorBatchClampAuthority {
    /// The citation as it appears in the refusal. Symbol first (`module decl`), position only where
    /// a position is what is being named — DESIGN §3.
    fn render(&self) -> String {
        match self {
            FloorBatchClampAuthority::PositionalCiSpecClamp {
                module_path,
                decl_name,
                index,
            } => format!("{module_path} {decl_name}[{index}]"),
            FloorBatchClampAuthority::ScopedBatchOwnedClamp {
                module_path,
                decl_name,
                ..
            } => format!("{module_path} {decl_name}"),
        }
    }

    fn batch_id(&self) -> Option<&str> {
        match self {
            FloorBatchClampAuthority::PositionalCiSpecClamp { .. } => None,
            FloorBatchClampAuthority::ScopedBatchOwnedClamp { batch_id, .. } => Some(batch_id),
        }
    }
}

/// A clamp plus the declaration that produced it. Constructing one without an authority is not
/// expressible, which is what keeps the refusal's citation and its number from drifting apart.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ResolvedFloorBatchClamp {
    overhead_ms: u128,
    per_unit_ms: u128,
    authority: FloorBatchClampAuthority,
}

impl ResolvedFloorBatchClamp {
    fn clamp_ms(&self, units: u128) -> u128 {
        self.overhead_ms + units * self.per_unit_ms
    }

    fn units_contribution_ms(&self, units: u128) -> u128 {
        units * self.per_unit_ms
    }
}

/// THE COST WALL (Piece 3 derived clamp — authority `gunbc.ci_spec.gunbc_ci_floor_batch_clamp_params`
/// + `gunbc_ci_floor_batch_clamp_note`): the per-batch clamp is `overhead_seconds*1000 +
/// runtime_unit_count * per_unit_ms`. This reads the index-aligned param lists fail-closed at
/// arm time (the fast-lane-budget pattern); the clamp itself is computed at enforcement, because the
/// affected-set-selected unit count is a runtime datum the schedule does not hold. Clamps are
/// admission/scheduling facts at the walk grain — witness verdicts never carry a wall-clock term
/// (the ruling split reconciled in the carrier note).
///
/// The authority lists are read from the SAME rows the numbers come from
/// (`gunbc_ci_floor_batch_clamp_authority_modules` / `..._decls` project `c.authority`), so a
/// citation cannot stay correct while the value it names moves.
fn read_floor_batch_clamp_params(
    plan_ctx: &InterpContext,
    batch_count: usize,
) -> Result<Vec<ResolvedFloorBatchClamp>, String> {
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
    let authority_modules = read_floor_batch_clamp_authority_list(
        plan_ctx,
        "gunbc_ci_floor_batch_clamp_authority_modules",
    )?;
    let authority_decls = read_floor_batch_clamp_authority_list(
        plan_ctx,
        "gunbc_ci_floor_batch_clamp_authority_decls",
    )?;
    if overheads_ms.len() != batch_count
        || rates_ms.len() != batch_count
        || authority_modules.len() != batch_count
        || authority_decls.len() != batch_count
    {
        return Err(format!(
            "claim_executor: floor batch clamp params (overhead {} row(s), rate {} row(s), authority module {} row(s), authority decl {} row(s)) must each cover the {} scheduled batch(es) exactly (fail-closed; update gunbc.ci_spec beside the schedule change)",
            overheads_ms.len(),
            rates_ms.len(),
            authority_modules.len(),
            authority_decls.len(),
            batch_count
        ));
    }
    Ok(overheads_ms
        .into_iter()
        .zip(rates_ms)
        .zip(authority_modules)
        .zip(authority_decls)
        .enumerate()
        .map(
            |(index, (((overhead_ms, per_unit_ms), module_path), decl_name))| {
                ResolvedFloorBatchClamp {
                    overhead_ms,
                    per_unit_ms,
                    authority: FloorBatchClampAuthority::PositionalCiSpecClamp {
                        module_path,
                        decl_name,
                        index,
                    },
                }
            },
        )
        .collect())
}

/// One of the two authority projections beside the clamp numbers. An empty name is refused rather
/// than rendered: a citation that names nothing is worse than none, because it reads as located.
fn read_floor_batch_clamp_authority_list(
    plan_ctx: &InterpContext,
    function: &str,
) -> Result<Vec<String>, String> {
    let items = match run_value(plan_ctx, function) {
        Ok(Value::List(items)) => items,
        Ok(other) => {
            return Err(format!(
                "claim_executor: {function} must be a List<String>, got {other:?} (fail-closed)"
            ));
        }
        Err(msg) => {
            return Err(format!(
                "claim_executor: floor plan schedules batches but {function} is unavailable (fail-closed): {msg}"
            ));
        }
    };
    let mut out: Vec<String> = Vec::new();
    for item in items.iter() {
        match item {
            Value::Str(s) if !s.is_empty() => out.push(s.to_string()),
            other => {
                return Err(format!(
                    "claim_executor: {function} rows must be non-empty Strings, got {other:?} (fail-closed)"
                ));
            }
        }
    }
    Ok(out)
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
    let mut clock_mismatch_count = 0usize;
    for (kind, subj, observed) in &rows {
        match basis.get(&(kind.clone(), subj.clone())) {
            None => {
                basis_absent_count += 1;
                body.push_str(&format!("{kind}\t{subj}\t{observed}\t\tBasisAbsent\t\n"));
            }
            Some(b) => {
                let verdict = match witness_row_cost_verdict_via_authority(&ctx, *observed, Some(b))
                {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!(
                                "claim_executor: compile-clean drift comparator refused for {kind}::{subj}: {e} — walk fails closed here"
                            );
                        return false;
                    }
                };
                match verdict.as_str() {
                    "DriftExceeded" => drift_count += 1,
                    "BasisClockMismatch" => clock_mismatch_count += 1,
                    _ => {}
                }
                body.push_str(&format!(
                    "{kind}\t{subj}\t{observed}\t{}\t{verdict}\t{}\n",
                    b.eval_ms_basis, b.run_ref
                ));
            }
        }
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
        "[receipt] floor compile-clean cost drift: basis_absent={basis_absent_count} clock_mismatch={clock_mismatch_count} drift_exceeded={drift_count} (TSV: {})",
        path.display()
    );
    true
}

/// Hand-Rust mirror of `gunbc.ci_spec` `RuntimeUnitCount` (variant names must match exactly).
#[derive(Debug, Clone, PartialEq, Eq)]
enum FloorRuntimeUnitCount {
    Observed { units: u128 },
    Unavailable { cause: String },
}

fn single_claim_runtime_unit_count() -> FloorRuntimeUnitCount {
    FloorRuntimeUnitCount::Observed { units: 1 }
}

fn discovery_runtime_unit_count_from_summary(total: usize) -> FloorRuntimeUnitCount {
    FloorRuntimeUnitCount::Observed {
        units: total as u128,
    }
}

fn runtime_unit_count_unavailable(cause: impl Into<String>) -> FloorRuntimeUnitCount {
    FloorRuntimeUnitCount::Unavailable {
        cause: cause.into(),
    }
}

/// Runtime per-batch unit count for the derived clamp: sum Observed rows; any Unavailable
/// refuses the whole batch clamp (authority `gunbc_ci_floor_batch_runtime_unit_count_note`).
fn aggregate_batch_runtime_units(results: &[ClaimResult]) -> FloorRuntimeUnitCount {
    let mut sum = 0u128;
    for result in results {
        match &result.runtime_unit_count {
            FloorRuntimeUnitCount::Observed { units } => sum += *units,
            FloorRuntimeUnitCount::Unavailable { cause } => {
                return FloorRuntimeUnitCount::Unavailable {
                    cause: cause.clone(),
                };
            }
        }
    }
    FloorRuntimeUnitCount::Observed { units: sum }
}

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority:
/// `gunbc.ci_spec.gunbc_ci_floor_batch_stop_policy_claim_executor_seed_note`,
/// type `gunbc.ci_spec.FloorBatchStopPolicy`):
/// seed-side enum mirror + `run_walk` consumer for the event-scoped batch halt;
/// policy mapping and plan roster enrollment are delegated to `.dag` eval
/// (`gunbc_ci_floor_batch_stop_policy_for_github_event`,
/// `gunbc_ci_floor_plan_uses_batch_stop_policy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
            str_value(plan_function.to_string()),
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
        &[(Some("event".to_string()), str_value(event))],
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
enum ParsedProfileProvenance {
    Declared,
    Undeclared,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
        native_bundle_entries: Vec<(String, String)>,
        exclude_substrings: Vec<String>,
        discovery_scope_dirs: Vec<String>,
        execution_mode: ExecutionMode,
        spawns_host_compiler: bool,
    },
    ScopedWitnessBatch {
        batch_id: String,
        source_roots: Vec<String>,
        source_roots_digest: String,
        entries: Vec<ScopedScheduleEntry>,
        scan_dirs: Vec<String>,
        execution_authority: ScopedWitnessExecutionAuthority,
        profile: ParsedRunnableProfile,
        clamp: ResolvedFloorBatchClamp,
        process_isolation: ScopedProcessIsolation,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ScopedScheduleEntry {
    entry: String,
    function: String,
    witness_kind: String,
}

#[derive(Clone)]
struct ScopedReceiptBatch {
    batch_id: String,
    source_roots_digest: String,
    entries: Vec<ScopedScheduleEntry>,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum ScopedProcessIsolation {
    SharedWalkProcess,
    SequentialChildProcess,
    FreshJobProcess,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum ScopedWitnessExecutionAuthority {
    InheritedWalkSourceRoots,
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
            Value::Str(s) => out.push(s.to_string()),
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
        Some(Value::Str(s)) => Ok(s.to_string()),
        Some(other) => Err(format!(
            "{}.{} is {}, not String",
            owner,
            name,
            ctx.format_value(other)
        )),
        None => Err(format!("{} missing field `{}`", owner, name)),
    }
}

fn nonnegative_measure_count(
    value: Option<&Value>,
    owner: &str,
    ctx: &InterpContext,
) -> Result<u128, String> {
    let fields = match value {
        Some(Value::Record { fields, .. }) | Some(Value::Variant { fields, .. }) => fields,
        Some(other) => {
            return Err(format!(
                "{owner} must be a std.measure Measure value, got {}",
                other.type_label_public()
            ))
        }
        None => return Err(format!("{owner} is absent")),
    };
    match ctx.field(fields, "count") {
        Some(Value::Int(n)) if *n >= 0 => Ok(*n as u128),
        _ => Err(format!("{owner}.count must be a nonnegative Int")),
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
            let (explicit_entries, native_bundle_entries) = match ctx
                .field(fields, "explicit_entries")
            {
                Some(v) => {
                    let mut out = Vec::new();
                    let mut native = Vec::new();
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
                        let entry = str_field(efields, "entry", "explicit_entries", ctx)?;
                        let function = str_field(efields, "function", "explicit_entries", ctx)?;
                        match ctx.field(efields, "kind") {
                            Some(Value::Variant { variant_name, .. })
                                if ctx.sym_eq(*variant_name, "CorpusWitnessKind")
                                    || ctx.sym_eq(*variant_name, "ExecutionWitnessKind") =>
                            {
                                out.push((entry, function));
                            }
                            Some(Value::Variant { variant_name, .. })
                                if ctx.sym_eq(*variant_name, "NativeBundleWitnessKind") =>
                            {
                                native.push((entry, function));
                            }
                            Some(Value::Variant { variant_name, .. }) => {
                                return Err(format!(
                                    "RunnableDiscoveryBatch explicit entry {entry}::{function}: \
                                     unhandled WitnessKind `{}` (kind_dispatch_refusal_count=1); \
                                     refusing instead of interpreting",
                                    ctx.resolve(*variant_name)
                                ));
                            }
                            Some(other) => {
                                return Err(format!(
                                    "RunnableDiscoveryBatch explicit entry {entry}::{function}: \
                                     WitnessKind is {}, not a variant \
                                     (kind_dispatch_refusal_count=1)",
                                    other.type_label_public()
                                ));
                            }
                            None => {
                                return Err(format!(
                                    "RunnableDiscoveryBatch explicit entry {entry}::{function}: \
                                     WitnessKind is absent (kind_dispatch_refusal_count=1); \
                                     refusing instead of interpreting"
                                ));
                            }
                        }
                    }
                    (out, native)
                }
                None => (Vec::new(), Vec::new()),
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
            let profile = parsed_runnable_profile_from_field(
                ctx.field(fields, "profile"),
                "RunnableDiscoveryBatch",
                ctx,
            )?;
            Ok(Runnable::DiscoveryBatch {
                source_roots,
                scan_dirs,
                explicit_entries,
                native_bundle_entries,
                exclude_substrings,
                discovery_scope_dirs,
                execution_mode: profile.execution_mode,
                spawns_host_compiler: profile.spawns_host_compiler,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableScopedWitnessBatch") => {
            let batch = match ctx.field(fields, "batch") {
                Some(Value::Record { fields, .. }) | Some(Value::Variant { fields, .. }) => fields,
                Some(other) => {
                    return Err(format!(
                    "RunnableScopedWitnessBatch.batch must be a ScopedWitnessBatch record, got {}",
                    other.type_label_public()
                ))
                }
                None => return Err("RunnableScopedWitnessBatch missing field `batch`".to_string()),
            };
            let batch_id = str_field(batch, "batch_id", "ScopedWitnessBatch", ctx)?;
            let source_roots = match ctx.field(batch, "source_roots") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => return Err("ScopedWitnessBatch missing field `source_roots`".to_string()),
            };
            let source_roots_digest = match run_in_context_with_args(
                ctx,
                "scoped_witness_source_roots_digest_for_wire",
                &[(
                    (Some("source_roots".to_string())),
                    str_list_value(&source_roots),
                )],
                false,
            ) {
                Ok(Value::Str(digest)) if !digest.is_empty() => digest.to_string(),
                Ok(other) => {
                    return Err(format!(
                        "scoped_witness_source_roots_digest_for_wire returned {}, expected String",
                        other.type_label_public()
                    ))
                }
                Err(msg) => {
                    return Err(format!(
                        "scoped_witness_source_roots_digest_for_wire refused: {msg}"
                    ))
                }
            };
            let entries = match ctx.field(batch, "entries") {
                Some(v) => {
                    let mut out = Vec::new();
                    for elem in free_monoid_elems(v, ctx)? {
                        let efields = match elem {
                            Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
                            other => {
                                return Err(format!(
                                    "ScopedWitnessBatch.entries element is {}, not a ScheduleWitnessEntry record",
                                    other.type_label_public()
                                ))
                            }
                        };
                        let witness_kind = match ctx.field(efields, "kind") {
                            Some(Value::Variant { variant_name, .. })
                                if ctx.sym_eq(*variant_name, "CorpusWitnessKind") =>
                            {
                                "corpus"
                            }
                            Some(Value::Variant { variant_name, .. })
                                if ctx.sym_eq(*variant_name, "ExecutionWitnessKind") =>
                            {
                                "execution"
                            }
                            Some(other) => {
                                return Err(format!(
                                    "ScopedWitnessBatch.entries.kind must be WitnessKind, got {}",
                                    other.type_label_public()
                                ))
                            }
                            None => {
                                return Err(
                                    "ScopedWitnessBatch.entries row missing `kind`".to_string()
                                )
                            }
                        };
                        out.push(ScopedScheduleEntry {
                            entry: str_field(efields, "entry", "ScopedWitnessBatch.entries", ctx)?,
                            function: str_field(
                                efields,
                                "function",
                                "ScopedWitnessBatch.entries",
                                ctx,
                            )?,
                            witness_kind: witness_kind.to_string(),
                        });
                    }
                    out
                }
                None => return Err("ScopedWitnessBatch missing field `entries`".to_string()),
            };
            let scan_dirs = match ctx.field(batch, "scan_dirs") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => return Err("ScopedWitnessBatch missing field `scan_dirs`".to_string()),
            };
            let execution_authority = match ctx.field(batch, "execution_authority") {
                Some(Value::Variant { variant_name, .. })
                    if ctx.sym_eq(*variant_name, "InheritedWalkSourceRoots") =>
                {
                    ScopedWitnessExecutionAuthority::InheritedWalkSourceRoots
                }
                Some(other) => {
                    return Err(format!(
                        "ScopedWitnessBatch.execution_authority must be ScopedWitnessExecutionAuthority, got {}",
                        other.type_label_public()
                    ))
                }
                None => {
                    return Err("ScopedWitnessBatch missing field `execution_authority`".to_string())
                }
            };
            let resource = match ctx.field(batch, "resource_profile") {
                Some(Value::Record { fields, .. }) | Some(Value::Variant { fields, .. }) => fields,
                Some(other) => {
                    return Err(format!(
                        "ScopedWitnessBatch.resource_profile must be a record, got {}",
                        other.type_label_public()
                    ))
                }
                None => {
                    return Err("ScopedWitnessBatch missing field `resource_profile`".to_string())
                }
            };
            let profile = parsed_runnable_profile_from_field(
                ctx.field(resource, "runnable"),
                "ScopedWitnessBatch.resource_profile.runnable",
                ctx,
            )?;
            let clamp_fields = match ctx.field(resource, "clamp") {
                Some(Value::Record { fields, .. }) | Some(Value::Variant { fields, .. }) => fields,
                Some(other) => {
                    return Err(format!(
                    "ScopedWitnessBatch.resource_profile.clamp must be RunnableBatchClamp, got {}",
                    other.type_label_public()
                ))
                }
                None => {
                    return Err("ScopedWitnessBatch.resource_profile missing `clamp`".to_string())
                }
            };
            let overhead_seconds = nonnegative_measure_count(
                ctx.field(clamp_fields, "overhead"),
                "ScopedWitnessBatch.resource_profile.clamp.overhead",
                ctx,
            )?;
            if overhead_seconds == 0 {
                return Err("ScopedWitnessBatch clamp overhead must be positive".to_string());
            }
            let per_unit_ms = nonnegative_measure_count(
                ctx.field(clamp_fields, "per_unit"),
                "ScopedWitnessBatch.resource_profile.clamp.per_unit",
                ctx,
            )?;
            // The batch's clamp declares its own home. Read it rather than inferring one from the
            // batch's position in the aligned list — the inference is what printed a ci_spec
            // citation beside a number ci_spec never produced.
            let clamp_authority_fields = match ctx.field(clamp_fields, "authority") {
                Some(Value::Record { fields, .. }) | Some(Value::Variant { fields, .. }) => fields,
                Some(other) => {
                    return Err(format!(
                        "ScopedWitnessBatch.resource_profile.clamp.authority must be DeclarationRef, got {}",
                        other.type_label_public()
                    ))
                }
                None => {
                    return Err(
                        "ScopedWitnessBatch.resource_profile.clamp missing `authority`".to_string()
                    )
                }
            };
            let clamp_authority_module = str_field(
                clamp_authority_fields,
                "module_path",
                "ScopedWitnessBatch.resource_profile.clamp.authority",
                ctx,
            )?;
            let clamp_authority_decl = str_field(
                clamp_authority_fields,
                "decl_name",
                "ScopedWitnessBatch.resource_profile.clamp.authority",
                ctx,
            )?;
            if clamp_authority_module.is_empty() || clamp_authority_decl.is_empty() {
                return Err(
                    "ScopedWitnessBatch.resource_profile.clamp.authority must name a module and a declaration"
                        .to_string(),
                );
            }
            let process_isolation = match ctx.field(resource, "process_isolation") {
                Some(Value::Variant { variant_name, .. })
                    if ctx.sym_eq(*variant_name, "SharedWalkProcess") => ScopedProcessIsolation::SharedWalkProcess,
                Some(Value::Variant { variant_name, .. })
                    if ctx.sym_eq(*variant_name, "SequentialChildProcess") => ScopedProcessIsolation::SequentialChildProcess,
                Some(Value::Variant { variant_name, .. })
                    if ctx.sym_eq(*variant_name, "FreshJobProcess") => ScopedProcessIsolation::FreshJobProcess,
                Some(other) => {
                    return Err(format!(
                        "ScopedWitnessBatch.resource_profile.process_isolation must be ScopedWitnessProcessIsolation, got {}",
                        other.type_label_public()
                    ))
                }
                None => return Err("ScopedWitnessBatch.resource_profile missing `process_isolation`".to_string()),
            };
            Ok(Runnable::ScopedWitnessBatch {
                batch_id: batch_id.clone(),
                source_roots,
                source_roots_digest,
                entries,
                scan_dirs,
                execution_authority,
                profile,
                clamp: ResolvedFloorBatchClamp {
                    overhead_ms: overhead_seconds * 1000,
                    per_unit_ms,
                    authority: FloorBatchClampAuthority::ScopedBatchOwnedClamp {
                        batch_id: batch_id.clone(),
                        module_path: clamp_authority_module,
                        decl_name: clamp_authority_decl,
                    },
                },
                process_isolation,
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

fn scoped_batch_clamp(batch: &[Runnable]) -> Result<Option<ResolvedFloorBatchClamp>, String> {
    let mut owned = None;
    for runnable in batch {
        if let Runnable::ScopedWitnessBatch { clamp, .. } = runnable {
            if batch.len() != 1 {
                return Err("RunnableScopedWitnessBatch must occupy a singleton batch".to_string());
            }
            if owned.replace(clamp.clone()).is_some() {
                return Err("batch carries more than one scoped witness clamp".to_string());
            }
        }
    }
    Ok(owned)
}

/// The parsed form of `std.realization_schedule.WalkPlan` — the ONE plan shape every
/// plan function returns (see `walk_plan_note` on the carrier). Two populations with
/// different ordering laws: `batches` are the ordinary floor under the walk's
/// `FloorBatchStopPolicy`; `on_success_stages` run only after the ordinary floor
/// completed AND its receipts wrote, each stage a barrier, always fail-fast between
/// stages regardless of the ordinary stop policy.
struct ParsedWalkPlan {
    pre_walk_execution: PreWalkExecution,
    batches: Vec<Vec<Runnable>>,
    finalization: Option<FloorFinalization>,
    on_success_stages: Vec<Vec<Runnable>>,
    ordinary_budget_ms: Option<u64>,
    on_success_budget_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PreWalkExecution {
    None,
    TypedClaimSubprocess {
        transport_entry: String,
        transport_function: String,
        source_roots: Vec<String>,
        claim_entry: String,
        claim_function: String,
    },
}

fn pre_walk_execution_from_value(
    value: &Value,
    ctx: &InterpContext,
) -> Result<PreWalkExecution, String> {
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = value
    else {
        return Err(format!(
            "WalkPlan.pre_walk_execution must be NoPreWalkExecution or TypedClaimSubprocess, got {}",
            ctx.format_value(value)
        ));
    };
    if ctx.sym_eq(*variant_name, "NoPreWalkExecution") {
        return Ok(PreWalkExecution::None);
    }
    if !ctx.sym_eq(*variant_name, "TypedClaimSubprocess") {
        return Err(format!(
            "WalkPlan.pre_walk_execution has unknown variant {}",
            ctx.format_value(value)
        ));
    }
    let string_field = |name: &str| -> Result<String, String> {
        match ctx.field(fields, name) {
            Some(Value::Str(s)) if !s.trim().is_empty() => Ok(s.to_string()),
            other => Err(format!(
                "TypedClaimSubprocess.{name} must be a non-empty String, got {other:?}"
            )),
        }
    };
    let roots = ctx
        .field(fields, "source_roots")
        .ok_or_else(|| "TypedClaimSubprocess.source_roots is missing".to_string())?;
    let source_roots = str_list_from_value(roots, ctx)?;
    if source_roots.is_empty() || source_roots.iter().any(|root| root.trim().is_empty()) {
        return Err("TypedClaimSubprocess.source_roots must contain non-empty paths".to_string());
    }
    Ok(PreWalkExecution::TypedClaimSubprocess {
        transport_entry: string_field("transport_entry")?,
        transport_function: string_field("transport_function")?,
        source_roots,
        claim_entry: string_field("claim_entry")?,
        claim_function: string_field("claim_function")?,
    })
}

fn optional_walk_budget_ms(
    value: &Value,
    ctx: &InterpContext,
    field: &str,
) -> Result<Option<u64>, String> {
    match value {
        Value::Null => Ok(None),
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "Present") => match ctx.field(fields, "value") {
            // The budget is a std.measure Millisecond — a Measure record whose `count`
            // carries the magnitude; the unit lives in the type, never in a field name.
            Some(measure @ (Value::Record { .. } | Value::Variant { .. })) => {
                let count =
                    nonnegative_measure_count(Some(measure), &format!("WalkPlan.{field}"), ctx)?;
                if count == 0 {
                    return Err(format!(
                        "WalkPlan.{field} must be a positive Millisecond, got count 0"
                    ));
                }
                u64::try_from(count)
                    .map(Some)
                    .map_err(|_| format!("WalkPlan.{field} exceeds the executor's u64 range"))
            }
            other => Err(format!(
                "WalkPlan.{field} Present.value must be a std.measure Millisecond, got {other:?}"
            )),
        },
        other => Err(format!(
            "WalkPlan.{field} must be Present {{ value: Millisecond }} or Absent, got {}",
            ctx.format_value(other)
        )),
    }
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
/// Transported semantic resolve obligation — population decoded from
/// `WalkPlan.finalization.expected_resolve_obligations`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TransportedObligation {
    identity: String,
    entry: String,
    function: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FloorFinalization {
    expected_obligations: Vec<TransportedObligation>,
}

type ObligationSubjectKey = (String, String);
type ObligationSubjectSet = std::collections::HashSet<ObligationSubjectKey>;

fn obligation_subject_set(fin: Option<&FloorFinalization>) -> Option<ObligationSubjectSet> {
    fin.map(|fin| {
        fin.expected_obligations
            .iter()
            .map(|obl| (obl.entry.clone(), obl.function.clone()))
            .collect()
    })
}

fn is_rostered_obligation_subject(
    subjects: &ObligationSubjectSet,
    entry: &str,
    function: &str,
) -> bool {
    subjects.contains(&(entry.to_string(), function.to_string()))
}

/// Attach the group's resolve-realization observation to the first rostered obligation
/// subject in the resolve group — not the first arbitrary co-resident claim.
fn take_group_observation_for_claim(
    obligation_subjects: Option<&ObligationSubjectSet>,
    entry: &str,
    function: &str,
    group_observation: &Option<ResolveRealizationObservation>,
    group_observation_attached: &mut bool,
) -> Option<ResolveRealizationObservation> {
    if *group_observation_attached {
        return None;
    }
    let rostered = match obligation_subjects {
        None => true,
        Some(subjects) => is_rostered_obligation_subject(subjects, entry, function),
    };
    if !rostered {
        return None;
    }
    *group_observation_attached = true;
    group_observation.clone()
}

impl FloorFinalization {
    /// Derived roster size — never a stored count literal (DESIGN §5).
    #[allow(dead_code)]
    fn declared_resolve_count(&self) -> i64 {
        self.expected_obligations.len() as i64
    }
}

fn string_field_from_record(
    fields: &[(v1_compiler::v1_interpreter::Symbol, Value)],
    ctx: &InterpContext,
    field: &str,
) -> Result<String, String> {
    match ctx.field(fields, field) {
        Some(Value::Str(s)) => Ok(s.to_string()),
        other => Err(format!("expected {field}: String, got {other:?}")),
    }
}

fn parse_resolve_obligation_identity(v: &Value, ctx: &InterpContext) -> Result<String, String> {
    match v {
        Value::Variant { variant_name, .. } => Ok(ctx.resolve(*variant_name)),
        other => Err(format!(
            "ResolveObligationIdentity must be a variant, got {}",
            ctx.format_value(other)
        )),
    }
}

fn parse_resolve_obligation_from_value(
    v: &Value,
    ctx: &InterpContext,
) -> Result<TransportedObligation, String> {
    let fields = match v {
        Value::Record { fields, .. } => fields,
        other => {
            return Err(format!(
                "ResolveObligation must be a record, got {}",
                ctx.format_value(other)
            ))
        }
    };
    let identity = parse_resolve_obligation_identity(
        ctx.field(fields, "identity")
            .ok_or_else(|| "ResolveObligation missing field `identity`".to_string())?,
        ctx,
    )?;
    let subject_fields = match ctx.field(fields, "subject") {
        Some(Value::Record { fields, .. }) => fields,
        other => {
            return Err(format!(
                "ResolveObligation.subject must be a record, got {other:?}"
            ))
        }
    };
    Ok(TransportedObligation {
        identity,
        entry: string_field_from_record(subject_fields, ctx, "entry")?,
        function: string_field_from_record(subject_fields, ctx, "function")?,
    })
}

fn parse_expected_obligations_from_fields(
    fields: &[(v1_compiler::v1_interpreter::Symbol, Value)],
    ctx: &InterpContext,
) -> Result<Vec<TransportedObligation>, String> {
    let list_val = ctx
        .field(fields, "expected_resolve_obligations")
        .ok_or_else(|| {
            "FloorFinalization missing field `expected_resolve_obligations`".to_string()
        })?;
    free_monoid_elems(list_val, ctx)?
        .iter()
        .map(|elem| parse_resolve_obligation_from_value(elem, ctx))
        .collect()
}

fn finalization_from_value(
    v: &Value,
    ctx: &InterpContext,
) -> Result<Option<FloorFinalization>, String> {
    // The two inhabitants have DIFFERENT runtime shapes, and that is a consequence of
    // the carrier split rather than an accident to paper over: FloorFinalization is a
    // standalone record in gunbc.ci_materialization (Value::Record), while
    // NoFinalizationDeclared is the nullary variant of std's NoWalkFinalization sum
    // (Value::Variant). Both are matched by TYPE NAME — never by "has a field called
    // expected_resolve_obligations", which would admit any record that happened to carry one.
    let floor_from_fields =
        |fields: &[(v1_compiler::v1_interpreter::Symbol, Value)]| -> Result<FloorFinalization, String> {
            Ok(FloorFinalization {
                expected_obligations: parse_expected_obligations_from_fields(fields, ctx)?,
            })
        };
    match v {
        Value::Record {
            type_name, fields, ..
        } if ctx.sym_eq(*type_name, "FloorFinalization") => Ok(Some(floor_from_fields(fields)?)),
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if ctx.sym_eq(*variant_name, "NoFinalizationDeclared") {
                Ok(None)
            } else if ctx.sym_eq(*variant_name, "FloorFinalization") {
                Ok(Some(floor_from_fields(fields)?))
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
    let pre_walk_execution_val = ctx.field(fields, "pre_walk_execution").ok_or_else(|| {
        "WalkPlan missing field `pre_walk_execution` — a plan with no pre-walk effect declares NoPreWalkExecution; the parser never invents absence".to_string()
    })?;
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
    let ordinary_budget_val = ctx.field(fields, "ordinary_budget").ok_or_else(|| {
        "WalkPlan missing field `ordinary_budget` — unbounded plans declare Absent; the parser never invents a budget".to_string()
    })?;
    let on_success_budget_val = ctx.field(fields, "on_success_budget").ok_or_else(|| {
        "WalkPlan missing field `on_success_budget` — plans without bounded postconditions declare Absent; the parser never invents a budget".to_string()
    })?;
    Ok(ParsedWalkPlan {
        pre_walk_execution: pre_walk_execution_from_value(pre_walk_execution_val, ctx)?,
        batches: batches_from_plan(batches_val, ctx)
            .map_err(|msg| format!("WalkPlan.batches: {msg}"))?,
        finalization: finalization_from_value(finalization_val, ctx)
            .map_err(|msg| format!("WalkPlan.finalization: {msg}"))?,
        on_success_stages: batches_from_plan(stages_val, ctx)
            .map_err(|msg| format!("WalkPlan.on_success_stages: {msg}"))?,
        ordinary_budget_ms: optional_walk_budget_ms(ordinary_budget_val, ctx, "ordinary_budget")?,
        on_success_budget_ms: optional_walk_budget_ms(
            on_success_budget_val,
            ctx,
            "on_success_budget",
        )?,
    })
}

/// A discovery claim's subject is a scanned corpus, not one declaration. Naming that
/// explicitly keeps a reader from parsing a blank `entry` as "unknown" when the truth is
/// "not one entry" — the state-space conflation this field exists to avoid.
const DISCOVERY_AGGREGATE_ENTRY: &str = "<discovery corpus — many entries>";

/// PAIRED LITERALS (review 47596): the .dag single authorities are
/// `tools.merge_admission_capture` `merge_admission_capture_refusal_wire_relpath` and
/// `tools.merge_admission_walk` `merge_admission_refresh_refusal_wire_relpath`; the seed
/// cannot import a .dag datum, so the pairing is these consts plus the sentence on each
/// datum naming this file — the floor-population-budget-refusal.txt precedent, under the
/// same walk_plan_run_stage_claim_executor_seed_deferral. A rename lands on both sides
/// or the wire read degrades to the loud wire-absent arm, never a silent success.
const MERGE_ADMISSION_CAPTURE_REFUSAL_WIRE: &str = "target/merge-admission-capture-refusal.txt";
const MERGE_ADMISSION_REFRESH_REFUSAL_WIRE: &str = "target/merge-admission-refresh-refusal.txt";

/// The .dag writers anchor both wires at `git.Inspect.Toplevel()` while these reads run
/// from the executor's cwd; anchoring the read on the same toplevel keeps a non-root cwd
/// from turning a written typed cause into a false "wire absent" (review 47663). A failed
/// toplevel resolution falls back to the bare relpath — the pre-anchor behavior — because
/// the wire read is itself a diagnostic path: degrading its precision is acceptable,
/// swallowing the stage failure it decorates is not.
fn merge_admission_wire_read(relpath: &str) -> std::io::Result<String> {
    let toplevel = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    match toplevel {
        Some(root) => fs::read_to_string(Path::new(&root).join(relpath)),
        None => fs::read_to_string(relpath),
    }
}

struct ClaimResult {
    function: String,
    /// The entry file this claim was declared in. Carried rather than looked up: a
    /// function name is NOT a declaration identity (review 2026-07-31), and two fixture or
    /// production modules may lawfully spell a function the same way. Recovering it by
    /// searching a stage's runnables for a matching function name would reproduce exactly
    /// the ambiguity that makes the name insufficient in the first place.
    ///
    /// Sites with no single declaring entry say so explicitly rather than passing an empty
    /// string — a discovery aggregate spans a corpus, and a blank field would read as
    /// "unknown" when the truth is "not one entry".
    entry: String,
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
    /// Unit count for the derived batch clamp — explicit availability, never inferred from
    /// `corpus_witnesses == 0` (that conflates discovery aggregate loss with gate rows).
    runtime_unit_count: FloorRuntimeUnitCount,
    /// Per-witness eval+resolve identity preserved from discovery (empty for gate/single-claim rows).
    witness_row_costs: Vec<WitnessRowCost>,
    /// Set only when this row was killed at a budget, carrying the pair that explains it.
    ///
    /// `ok`/`detail` are a lossy flattening of `ClaimOutcome`, so without this the batch's
    /// failure mode has to be recovered by substring-matching `detail` — which is what
    /// `falsifier_failure_mode` did, and its fallback arm is `WitnessRed`, so a reworded
    /// message silently demoted a budget refusal to a witness failure. This keeps the fact
    /// as data on the path that needs it. It is a projection of the two budget arms of
    /// `ClaimOutcome`, not a second authority: nothing sets it except those arms below.
    budget_refusal: Option<BudgetRefusal>,
    /// Set when a wet witness refused because a host CLI dependency was absent on PATH
    /// before execution. Parsed from the failure-receipt wire
    /// `HostDependencyAbsent{tool=...,hint=...}`. Dissolve-on:
    /// `gunbc.witness_row_cost` `host_dependency_refusal_seed_deferral_note`.
    host_dependency_refusal: Option<HostDependencyRefusal>,
    /// Set only when this batch contained a witness declared expected-RED that ran GREEN,
    /// carrying the identities that must be un-quarantined.
    ///
    /// Same reason `budget_refusal` exists, one class over: without it the mode has to be
    /// recovered from prose, and the fallback arm is `WitnessRed` — so an un-quarantine
    /// would be indistinguishable from a genuine regression in the alert signature, with a
    /// completely different remedy (delete the admission row vs. fix the code). It is
    /// derived from ROSTER MEMBERSHIP joined against per-witness outcomes, never from a
    /// message, so no rewording can move the class.
    expectation_refusal: Option<ExpectationRefusal>,
    /// Discovery batch only: finalized selection-degradation facts for floor receipts.
    /// Recorded at the reuse decision site for rostered obligation subjects only.
    /// Disposition is NEVER inferred from `resolve_nanos` alone — timing is cost evidence.
    resolve_realization: Option<ResolveRealizationObservation>,
}

/// Hand-Rust mirror of `gunbc.ci_materialization` `ResolveRealization` (variant names
/// must match exactly; dissolve-on: witness-realization P4 executor cutover decodes
/// modeled `.dag` observations at reuse sites instead of re-authoring them).
#[derive(Debug, Clone, PartialEq, Eq)]
enum ResolveRealizationObservation {
    ColdResolvePerformed {
        resolve_nanos: u128,
    },
    SatisfiedFromSharedPool {
        computation_identity: String,
        provider_id: String,
    },
}

/// Receipt disposition tags — must match `ResolveRealization` variant names exactly.
const RESOLVE_REALIZATION_DISPOSITION_COLD: &str = "ColdResolvePerformed";
const RESOLVE_REALIZATION_DISPOSITION_WARM: &str = "SatisfiedFromSharedPool";

/// Per-entry walk memo provider — grain distinct from index-build `process_shared_index`.
/// Authority: `gunbc.floor_materialization` `floor_entry_walk_memo_provider_id`;
/// drift gate: `floor_entry_walk_memo_provider_id_matches_dag_authority`.
const FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID: &str = "walk_memo";

/// A refusal about the EXPECTATION machinery itself, as distinct from a witness verdict.
///
/// Every arm means the same class of thing — the known-red roster does not describe reality as
/// observed — and NONE is an ordinary `WitnessRed`, because their remedies are edits to the
/// admission authority or to the seam, never to a witness. They are separate arms because the
/// edits differ: a stale quarantine means DELETE the row, an absent observation means make the
/// row RUN, and an unverified pre-verdict declaration means carry the typed cause across the
/// execution boundary.
///
/// This enum is also the ONLY structural route to a typed receipt mode. `batch_failure_mode_and_detail`
/// reads it off the VALUE and otherwise falls through to `falsifier_failure_mode`, whose fallback
/// arm is `"WitnessRed"` — so a mode carried only in prose (in `function`/`detail`) is not carried
/// at all. That is exactly how the pre-verdict arm shipped half-wired: the `.dag` vocabulary and
/// the non-green result existed while the receipt still said `WitnessRed`, which is the
/// one-fact-two-representations defect this PR exists to remove, committed by the PR itself.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectationRefusal {
    /// Declared expected-red, ran GREEN. The quarantine is stale.
    StaleKnownRed { witnesses: Vec<(String, String)> },
    /// Declared expected-red and rostered onto this batch, but NO observation came back.
    ///
    /// This is a COVERAGE refusal, not a semantic one: it asserts nothing about whether the
    /// witness would be red, only that nobody looked. It is a refusal rather than a log line
    /// because the alternative — which this bin shipped in its first shape — is a batch passing
    /// while an admitted known-red row silently stopped executing, which is precisely the
    /// coverage-by-illusion tier: the roster claims a red control exists, the run produces no
    /// evidence either way, and the green report is read as though it did.
    ExpectedRedEvidenceAbsent { witnesses: Vec<(String, String)> },
    /// Every entry DECLARED a typed pre-verdict refusal and the batch stopped before any
    /// verdict — so the stop is real, but the observed phase/cause is unavailable at this seam
    /// and the declaration cannot be checked against it. Neither agreement nor a witness red.
    PreVerdictUnverified {
        declared_entries: usize,
        cause: String,
    },
}

impl ExpectationRefusal {
    fn mode(&self) -> &'static str {
        match self {
            Self::StaleKnownRed { .. } => STALE_KNOWN_RED_MODE,
            Self::ExpectedRedEvidenceAbsent { .. } => EXPECTED_RED_EVIDENCE_ABSENT_MODE,
            Self::PreVerdictUnverified { .. } => EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE,
        }
    }

    fn detail(&self) -> String {
        match self {
            Self::StaleKnownRed { witnesses } => format!(
                "{STALE_KNOWN_RED_MODE}: {} witness(es) declared expected-red ran GREEN: {} — un-quarantine (delete the row from gunbc.explicit_witness_admission explicit_witness_admissions, the single authority both known-red cadences project from) or restore the discriminating red",
                witnesses.len(),
                render_witness_ids(witnesses)
            ),
            Self::ExpectedRedEvidenceAbsent { witnesses } => format!(
                "{EXPECTED_RED_EVIDENCE_ABSENT_MODE}: {} witness(es) declared expected-red produced NO observation on this batch: {} — the red control did not execute, so neither agreement nor failure was established; restore its execution or delete the admission row",
                witnesses.len(),
                render_witness_ids(witnesses)
            ),
            Self::PreVerdictUnverified {
                declared_entries,
                cause,
            } => format!(
                "{EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE}: {declared_entries} entry(ies) declare a typed pre-verdict refusal and the batch stopped before any verdict, but the observed phase/cause is unavailable at this seam so the declaration cannot be verified — non-green by construction until EXPECTED-RED-CAUSE-1 lands: {cause}"
            ),
        }
    }
}

/// The failure-mode tag for an expected-red row that produced no observation. Must match the tag
/// `gunbc.floor_component_receipt` `floor_component_failure_mode_of` parses.
const EXPECTED_RED_EVIDENCE_ABSENT_MODE: &str = "ExpectedRedEvidenceAbsent";

/// A batch whose entries all DECLARE a typed pre-verdict refusal, stopped before any verdict.
/// Non-green: the declaration classifies the stop, it does not verify it. See the Err arm.
const EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE: &str = "ExpectedRedPreVerdictUnverified";

/// The Err-path result for a batch whose entries ALL declare a typed pre-verdict refusal.
///
/// Extracted from the arm so `ok == false` is reachable by a test. Inline, the non-green
/// property was asserted only by a doc comment: nothing executed `run_discovery_batch_node`,
/// so flipping `ok` back to `true` left every assertion green — a stated regression control
/// that did not exist, the same defect class as a stated identity join that was a length
/// agreement. The arm has exactly one construction site and it is this function.
fn pre_verdict_unverified_claim_result(
    label: &str,
    declared_entries: usize,
    msg: &str,
) -> ClaimResult {
    // The mode must reach the receipt off the VALUE. Carried only in `function`/`detail` it is
    // not carried at all: `batch_failure_mode_and_detail` reads `expectation_refusal`
    // structurally and otherwise falls through to `falsifier_failure_mode`, whose fallback arm
    // is "WitnessRed" — so these batches were reported as ordinary witness failures while the
    // `.dag` vocabulary said `Refused`. Found by cursor review 50221.
    let refusal = ExpectationRefusal::PreVerdictUnverified {
        declared_entries,
        cause: msg.to_string(),
    };
    ClaimResult {
        function: format!("{label} ({EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE})"),
        entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
        // NON-GREEN. A declaration classifies the stop; it does not verify it.
        ok: false,
        detail: refusal.detail(),
        wall_nanos: 0,
        resolve_nanos: 0,
        corpus_resolve_nanos: 0,
        corpus_eval_nanos: 0,
        corpus_witnesses: 0,
        // The seam that ate the typed cause also ate the unit count: nothing ran, so this is
        // UNAVAILABLE rather than zero. Zero would assert a measurement nobody took.
        runtime_unit_count: runtime_unit_count_unavailable(msg),
        witness_row_costs: Vec::new(),
        expectation_refusal: Some(refusal),
        budget_refusal: None,
        host_dependency_refusal: None,
        resolve_realization: None,
    }
}

const STALE_KNOWN_RED_MODE: &str = "StaleKnownRed";

/// Which verdict is AGREEMENT for one witness — `std.witness_admission`
/// `WitnessExpectedVerdict`, matched at FUNCTION grain.
///
/// This replaces a batch-wide boolean derived from entry PATHS. That shape decided polarity
/// from the batch a witness happened to sit in, so a mixed batch silently reverted to
/// ordinary polarity (`.all()`), a green sibling in a quarantined file inherited the
/// quarantine, and an expensive known-red row that joined a second batch to obtain its eval
/// budget became expected-green there — which is how a witness that is red BY DESIGN reached
/// the falsifier alert as a component failure (gunbc#7737). Expectation is a property of the
/// witness; nothing about which batch runs it may change what counts as agreement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WitnessExpectedVerdict {
    ExpectWitnessHolds,
    ExpectWitnessRed,
}

/// `(entry, function)` from the admission authority. An empty `function` is
/// `ScheduleWitnessEntry`'s declared file-grain form and covers every witness in the entry.
fn expected_verdict_for(
    expected_red: &[(String, String)],
    entry: &str,
    function: &str,
) -> WitnessExpectedVerdict {
    let matched = expected_red
        .iter()
        .any(|(e, f)| e == entry && (f == function || f.is_empty()));
    if matched {
        WitnessExpectedVerdict::ExpectWitnessRed
    } else {
        WitnessExpectedVerdict::ExpectWitnessHolds
    }
}

/// What actually happened to ONE witness declared expected-red.
///
/// AGREEMENT MEANS THE ASSERTION RAN AND WAS FALSE — not merely that it was not `Pass`.
/// The first shape of this classifier computed `green = (outcome == Pass)` and routed every
/// other outcome into the agreement arm, which blessed a budget kill, an interpreter error, a
/// non-Bool return and an unobserved row as though the business assertion had executed and
/// returned false. That is the ⊥-as-ignorance conflation in the one place that decides whether
/// a red is real: "the assertion was false" and "no verdict was produced" are different states
/// with different remedies, and only the first is a quarantine holding. It also contradicted a
/// recorded ruling outright — `BudgetExceeded` is an interruption plus a measured lower bound on
/// cost, NEVER a semantic verdict on a witness — so a known-red witness that timed out reported
/// its quarantine as intact while its real behaviour was unobserved.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedRedDisposition {
    /// The assertion ran and returned false. THE ONLY AGREEMENT.
    AgreementAssertionReturnedFalse,
    /// The assertion ran and returned true — the quarantine is stale and its row must go.
    StaleQuarantineAssertionReturnedTrue,
    /// Killed at a declared budget: an interruption plus a lower bound on cost, no verdict.
    BudgetFailure,
    /// The interpreter raised, or the witness returned a non-Bool. No verdict was produced.
    ///
    /// `RuntimeError` and `NotBool` share this arm because the seam cannot tell them apart from
    /// a REFUSAL BEFORE EVALUATION today: `run_claim` formats the typed `InterpError` into a
    /// `String` at the point it builds `ClaimOutcome::RuntimeError`, so a pre-evaluation refusal
    /// and a mid-evaluation fault arrive indistinguishable. They are not split here by sniffing
    /// that message — recovering a class from prose the seed just formatted is the exact
    /// mechanism `budget_refusal` exists to avoid, and it is the shape that let a reworded error
    /// silently demote a budget refusal to a witness failure. Collapsing two states that cannot
    /// be observed here, and saying so, is the honest rung; fabricating the distinction would be
    /// rung inflation in a table whose whole purpose is rung honesty.
    ///
    /// Dissolve-on: NOT a separate trigger — the same one `gunbc.witness_row_cost`
    /// `witness_cost_timed_out_seed_deferral_note` already carries for this seam, because it is
    /// the same fact: the typed error does not survive the marshalling of `ClaimOutcome` across
    /// the interpreter boundary. When witness execution is realized from `.dag` rather than
    /// interpreted in the seed bin (the witness-realization lane), the typed error is in hand at
    /// the point of judgement and this arm splits in two. Pointed at the existing note rather
    /// than minting a second ledger entry for one cause (DESIGN §6, one fact one home).
    InfrastructureOrReferentFailure,
}

impl ExpectedRedDisposition {
    fn of(outcome: &ClaimOutcome) -> Self {
        match outcome {
            ClaimOutcome::Fail => Self::AgreementAssertionReturnedFalse,
            ClaimOutcome::Pass => Self::StaleQuarantineAssertionReturnedTrue,
            // SPLIT ON completion, because the two arms differ in whether a verdict exists.
            // An INTERRUPTED row produced none — budget failure is the honest reading. A
            // CompletedOverBudget row RAN TO COMPLETION AND PASSED (both converters match only
            // `ClaimOutcome::Pass`), so reporting "no verdict" about it is the same
            // state-space conflation this PR removes on the required-floor path, left standing
            // one function away.
            //
            // It maps to the stale-quarantine arm rather than a new one deliberately. The
            // verdict-bearing half — the enrolled claim is passing and the roster row is stale
            // — is exactly what that arm means, and it is the half a consumer of this tally
            // acts on. The cost half is NOT represented here: this surface has one channel per
            // row, where `run_required_floor` reports such a row to both `stale_quarantine`
            // and `completed_over_cost_requirement`. Minting a `StaleAndOverBudget` arm to carry it would be
            // growth on a surface the floor cut is retiring — `--required-floor` returns
            // before any caller of `run_batch_unit`, so CI does not execute this path at all.
            // Losing a cost signal on a retiring surface is the right trade; reporting a
            // passing row as undecided is not.
            ClaimOutcome::BudgetInterrupted { .. } => Self::BudgetFailure,
            ClaimOutcome::CompletedOverBudget { .. } => Self::StaleQuarantineAssertionReturnedTrue,
            // A ROUTE GAP JOINS THE INFRASTRUCTURE ARM, not the assertion arm, and for the
            // same reason `HostToolUnresolved` already sits there: enrollment asserts an
            // expected VERDICT, and a claim whose route has no arm for a host effect never
            // reached its subject to produce one. Mapping it to
            // `AgreementAssertionReturnedFalse` would report an unrun witness as agreement.
            ClaimOutcome::HostEffectRefused { .. }
            | ClaimOutcome::HostToolUnresolved { .. }
            | ClaimOutcome::RuntimeError { .. }
            | ClaimOutcome::NotBool { .. } => Self::InfrastructureOrReferentFailure,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::AgreementAssertionReturnedFalse => "agreement",
            Self::StaleQuarantineAssertionReturnedTrue => "stale-quarantine",
            Self::BudgetFailure => "budget-failure",
            Self::InfrastructureOrReferentFailure => "infrastructure-or-referent-failure",
        }
    }
}

/// The result of matching each executed witness against its own declared expectation.
///
/// Every cell is a distinct fact with a distinct remedy — collapsing any two is what the
/// batch-wide boolean, and then the `!= Pass` shortcut, each did in turn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct WitnessExpectationTally {
    agreements: Vec<(String, String)>,
    stale_known_red: Vec<(String, String)>,
    /// Expected-red witnesses that produced NO verdict: budget kills and infra/referent faults.
    /// Real failures, previously blessed as agreement.
    expected_red_without_verdict: Vec<(String, String, ExpectedRedDisposition)>,
    /// Rostered expected-red witnesses with no observation at all. Counted, never a verdict.
    evidence_absent: Vec<(String, String)>,
    /// Witnesses expected to hold that did not pass. Ordinary reds.
    unexpected_failures: Vec<(String, String)>,
    /// Every non-`Pass` outcome identity this classifier actually saw, whatever its expectation.
    /// The population the admission join must exactly reproduce — identities, never a count.
    classified_non_pass: Vec<(String, String)>,
}

impl WitnessExpectationTally {
    /// Stale quarantine first: it is a statement about an executed witness, and it names an
    /// edit that also removes the row from the absent-evidence population. Absent evidence is
    /// reported when nothing ran green, so one batch reports the sharper of the two.
    fn refusal(&self) -> Option<ExpectationRefusal> {
        if !self.stale_known_red.is_empty() {
            Some(ExpectationRefusal::StaleKnownRed {
                witnesses: self.stale_known_red.clone(),
            })
        } else if !self.evidence_absent.is_empty() {
            Some(ExpectationRefusal::ExpectedRedEvidenceAbsent {
                witnesses: self.evidence_absent.clone(),
            })
        } else {
            None
        }
    }

    /// Failures that must red the batch: an expected-red witness that produced no verdict, and
    /// an expected-to-hold witness that did not pass.
    ///
    /// Absent evidence is deliberately NOT counted here, and that is not the hole it looks like:
    /// it refuses through `refusal()` above, on its own mode, so it stops the line without being
    /// mislabelled as a semantic red. Counting it as a witness failure would put a coverage fact
    /// into the verdict population and corrupt the exact failure accounting below, which joins
    /// against outcomes that exist.
    fn hard_failures(&self) -> usize {
        self.expected_red_without_verdict.len() + self.unexpected_failures.len()
    }

    /// The identities this classifier actually placed into a non-`Pass` bucket.
    ///
    /// `stale_known_red` is deliberately absent: a stale quarantine is a witness that PASSED,
    /// so it never enters the non-pass population — it refuses on its own arm instead.
    fn accounted_non_pass(&self) -> Vec<(String, String)> {
        let mut ids = self.agreements.clone();
        ids.extend(
            self.expected_red_without_verdict
                .iter()
                .map(|(e, f, _)| (e.clone(), f.clone())),
        );
        ids.extend(self.unexpected_failures.iter().cloned());
        ids.sort();
        ids
    }

    /// Completeness is an IDENTITY JOIN, not a count equality (operator oracle ruling,
    /// 2026-08-01): every non-`Pass` outcome must appear in exactly one bucket, matched by
    /// witness identity. A count agreeing while the identities differ is the same absorption
    /// `<=` allowed, one layer in.
    fn non_pass_join_is_complete(&self) -> bool {
        let mut seen = self.classified_non_pass.clone();
        seen.sort();
        let accounted = self.accounted_non_pass();
        seen == accounted
    }
}

#[cfg(test)]
fn classify_witness_expectations(
    outcomes: &[DiscoveryWitnessOutcome],
    expected_red: &[(String, String)],
) -> WitnessExpectationTally {
    classify_witness_expectations_in(outcomes, expected_red, &[])
}

/// `rostered` is the batch's explicit entries, used only to find expected-red rows that produced
/// no observation at all. Empty means the absent-evidence axis is not being checked.
fn classify_witness_expectations_in(
    outcomes: &[DiscoveryWitnessOutcome],
    expected_red: &[(String, String)],
    rostered: &[(String, String)],
) -> WitnessExpectationTally {
    let mut tally = WitnessExpectationTally::default();
    for row in outcomes {
        let id = (row.entry.clone(), row.function.clone());
        if !matches!(row.outcome, ClaimOutcome::Pass) {
            tally.classified_non_pass.push(id.clone());
        }
        match expected_verdict_for(expected_red, &row.entry, &row.function) {
            WitnessExpectedVerdict::ExpectWitnessRed => {
                match ExpectedRedDisposition::of(&row.outcome) {
                    ExpectedRedDisposition::AgreementAssertionReturnedFalse => {
                        tally.agreements.push(id)
                    }
                    ExpectedRedDisposition::StaleQuarantineAssertionReturnedTrue => {
                        tally.stale_known_red.push(id)
                    }
                    other => tally.expected_red_without_verdict.push((id.0, id.1, other)),
                }
            }
            WitnessExpectedVerdict::ExpectWitnessHolds => {
                if !matches!(row.outcome, ClaimOutcome::Pass) {
                    tally.unexpected_failures.push(id);
                }
            }
        }
    }
    // An expected-red row this batch was asked to run, for which no observation came back.
    for (entry, function) in rostered {
        let expected = matches!(
            expected_verdict_for(expected_red, entry, function),
            WitnessExpectedVerdict::ExpectWitnessRed
        );
        let observed = outcomes
            .iter()
            .any(|o| &o.entry == entry && &o.function == function);
        if expected && !observed {
            tally
                .evidence_absent
                .push((entry.clone(), function.clone()));
        }
    }
    tally
}

/// The still-red pass arm, as a predicate so the accounting is testable rather than inline.
///
/// TWO independent conditions, because there are two populations and one cannot vouch for the
/// other:
///
///   1. `non_pass_join_is_complete` — the ADMISSION authority. Every non-`Pass` witness outcome
///      is matched, BY IDENTITY, to exactly one bucket (operator oracle ruling, 2026-08-01:
///      completeness is an identity join, never a count equality).
///   2. `summary_failure_count` — a residual guard over `summary.failures`, which is a `Vec` of
///      RENDERED DIAGNOSTIC STRINGS carrying no witness identity. It is deliberately NOT the
///      authority; it exists only so a failure that arrives with no per-witness outcome at all
///      refuses rather than being read as "nothing failed" — the empty-observation narrow
///      applied to the batch's own accounting, which the former `<=` admitted outright.
///
/// Dissolve-on for (2): `failures` carrying witness identities, at which point it folds into the
/// join in (1) and stops being a separate count at all.
fn still_red_batch_passes(tally: &WitnessExpectationTally, summary_failure_count: usize) -> bool {
    tally.hard_failures() == 0
        && tally.non_pass_join_is_complete()
        && summary_failure_count == tally.classified_non_pass.len()
}

/// Absent evidence is counted and named on every path. This is the LOG beside the refusal, not
/// instead of it: `WitnessExpectationTally::refusal` turns the same population into a typed
/// `ExpectedRedEvidenceAbsent` batch failure. The first shape of this bin only logged, so a batch
/// whose admitted known-red row stopped executing still reported green — the deficit's frequency
/// was observable in stderr and nowhere the line could stop.
fn report_absent_expected_red_evidence(tally: &WitnessExpectationTally) {
    if !tally.evidence_absent.is_empty() {
        eprintln!(
            "[expect-red] EVIDENCE ABSENT for {} expected-red witness(es) — not agreement, not failure, no verdict: {}",
            tally.evidence_absent.len(),
            render_witness_ids(&tally.evidence_absent)
        );
    }
}

fn render_disposition_ids(ids: &[(String, String, ExpectedRedDisposition)]) -> String {
    ids.iter()
        .map(|(e, f, d)| format!("{e}::{f} [{}]", d.label()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_witness_ids(ids: &[(String, String)]) -> String {
    ids.iter()
        .map(|(e, f)| format!("{e}::{f}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The pair explaining a budget kill, kept alongside the flattened `ok`/`detail`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BudgetRefusal {
    elapsed_ms: u64,
    budget_ms: u64,
    kind: BudgetKind,
    /// Carried, not dropped. Without it the erasure the classifier just stopped doing simply
    /// moves one layer down: this carrier feeds the durable component and alert path, so a
    /// completed-and-passed row would arrive there as an indistinguishable "budget refusal"
    /// and its exact elapsed would be read as a ceiling.
    completion: v1_compiler::cli_run::BudgetCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostDependencyRefusal {
    tool: String,
    hint: String,
}

impl HostDependencyRefusal {
    fn mode(&self) -> &'static str {
        HOST_DEPENDENCY_ABSENT_MODE
    }

    fn detail(&self) -> String {
        format!(
            "HostDependencyAbsent{{tool={},hint={}}}",
            self.tool, self.hint
        )
    }
}

const HOST_DEPENDENCY_ABSENT_MODE: &str = "HostDependencyAbsent";

fn host_dependency_refusal_from_detail(detail: &str) -> Option<HostDependencyRefusal> {
    const PREFIX: &str = "HostDependencyAbsent{tool=";
    // Anchor on the last wire token so an earlier accidental substring cannot win.
    let start = detail.rfind(PREFIX)?;
    let rest = &detail[start + PREFIX.len()..];
    let comma = rest.find(",hint=")?;
    let tool = rest[..comma].to_string();
    if tool.is_empty() || tool.contains('{') || tool.contains('}') {
        return None;
    }
    let hint_start = comma + ",hint=".len();
    let hint_rest = &rest[hint_start..];
    if hint_rest.contains('{') {
        return None;
    }
    let hint_end = hint_rest.rfind('}')?;
    if hint_end != hint_rest.len() - 1 {
        return None;
    }
    let hint = hint_rest[..hint_end].to_string();
    if hint.is_empty() {
        return None;
    }
    Some(HostDependencyRefusal { tool, hint })
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
    NativeBundle {
        entry: String,
        selector_function: String,
        execution_mode: ExecutionMode,
    },
    Discovery {
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        explicit_entries: Vec<(String, String)>,
        exclude_substrings: Vec<String>,
        discovery_scope_dirs: Vec<String>,
        execution_mode: ExecutionMode,
        spawns_host_compiler: bool,
    },
    ScopedDiscovery {
        batch_id: String,
        source_roots_digest: String,
        entries_with_kind: Vec<ScopedScheduleEntry>,
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        execution_authority: ScopedWitnessExecutionAuthority,
        execution_mode: ExecutionMode,
        spawns_host_compiler: bool,
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
                native_bundle_entries,
                exclude_substrings,
                discovery_scope_dirs,
                execution_mode,
                spawns_host_compiler,
            } => {
                if !scan_dirs.is_empty() || !explicit_entries.is_empty() {
                    units.push(BatchUnit::Discovery {
                        source_roots: source_roots.clone(),
                        scan_dirs: scan_dirs.clone(),
                        explicit_entries: explicit_entries.clone(),
                        exclude_substrings: exclude_substrings.clone(),
                        discovery_scope_dirs: discovery_scope_dirs.clone(),
                        execution_mode: *execution_mode,
                        spawns_host_compiler: *spawns_host_compiler,
                    });
                }
                for (entry, selector_function) in native_bundle_entries {
                    units.push(BatchUnit::NativeBundle {
                        entry: entry.clone(),
                        selector_function: selector_function.clone(),
                        execution_mode: *execution_mode,
                    });
                }
            }
            Runnable::ScopedWitnessBatch {
                batch_id,
                source_roots,
                source_roots_digest,
                entries,
                scan_dirs,
                execution_authority,
                profile,
                ..
            } => units.push(BatchUnit::ScopedDiscovery {
                batch_id: batch_id.clone(),
                source_roots_digest: source_roots_digest.clone(),
                entries_with_kind: entries.clone(),
                source_roots: source_roots.clone(),
                scan_dirs: scan_dirs.clone(),
                execution_authority: *execution_authority,
                execution_mode: profile.execution_mode,
                spawns_host_compiler: profile.spawns_host_compiler,
            }),
        }
    }
    units
}

/// `ctx` exists solely to reach the failure-receipt companion on a red witness.
///
/// `cli_run`'s summary path has projected companion receipts since the Lane B agreement work,
/// but THIS binary — the one the CI floor actually runs — did not, so the same witness was
/// loud locally and mute in CI. That divergence is what let ten consecutive
/// `extdeps_scope_placement_gate_passes` reds on main report only `returned Bool(false)`,
/// naming neither the refusing arm nor the offending path. Both surfaces now read the one
/// derivation (`cli_run::failure_receipt_companion`) and the one runner
/// (`cli_run::run_claim_failure_receipt`); a witness with no companion is unchanged.
///
/// HAND-RUST DISPOSITION (DESIGN §7 seed-shrinks-toward-zero; review 47022 asked for this
/// receipt explicitly). This edit is DEFERRED seed retention, not a new scaffold and not a
/// census movement: it adds zero tracked-Rust paths (both touched files are already tracked
/// and neither carries a `rust_source_lifecycle_residue_rows` row), so the path-grain
/// population `gunbc.stage0_rust_honest_frontier_projection` measures is unchanged by it.
/// The line-grain axis that would price it, `HandAuthoredLOC`, is one of the three axes that
/// projection's own note declares explicitly deferred, so there is no line census to shrink
/// here and claiming one would be a fabricated receipt.
///
/// Lane: **v1 exit**, whose fourth finish line is zero hand-maintained Rust.
/// ROADMAP row: "Get hand-written Rust in this repository down to zero"
/// (authority `dag/gunbc/v1_deletion_plan.dag`).
/// Dissolution trigger: this projection is floor-runner plumbing and has no independent
/// lifetime — it is deleted WITH `claim_executor` when witness execution leaves the
/// hand-maintained seed runner, not migrated ahead of it. Writing the receipt in `.dag`
/// first is not available: the failure-receipt channel is the seed's own witness-reporting
/// surface, and `.dag` has no print primitive to surface a reason through.
///
/// The alternative was to leave the reason discarded, which is the DESIGN §5 trap this whole
/// PR exists to close — a refusal that cannot be located is not a refusal anyone can act on.
///
/// `resolve_realization` is acquisition evidence from the resolve seam; every outcome arm
/// preserves it when present so a semantic witness failure cannot fabricate a missing-
/// realization refusal at finalization.
fn claim_result_for_outcome(
    ctx: &InterpContext,
    function: String,
    entry: String,
    outcome: ClaimOutcome,
    wall_nanos: u128,
    resolve_nanos: u128,
    resolve_realization: Option<ResolveRealizationObservation>,
) -> ClaimResult {
    match outcome {
        ClaimOutcome::Pass => ClaimResult {
            function,
            entry: entry.clone(),
            ok: true,
            detail: String::new(),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization,
        },
        ClaimOutcome::Fail => {
            let mut detail = "returned Bool(false)".to_string();
            v1_compiler::cli_run::append_failure_receipt_companion_loudness(
                &mut detail,
                ctx,
                &function,
            );
            v1_compiler::cli_run::append_witness_verdict_diagnostic_loudness(
                &mut detail,
                ctx,
                &function,
            );
            let host_dependency_refusal = host_dependency_refusal_from_detail(&detail);
            ClaimResult {
                detail,
                function,
                entry: entry.clone(),
                ok: false,
                wall_nanos,
                resolve_nanos,
                corpus_resolve_nanos: 0,
                corpus_eval_nanos: 0,
                corpus_witnesses: 0,
                runtime_unit_count: single_claim_runtime_unit_count(),
                witness_row_costs: Vec::new(),
                expectation_refusal: None,
                budget_refusal: None,
                host_dependency_refusal,
                resolve_realization,
            }
        }
        ClaimOutcome::NotBool { got } => ClaimResult {
            function,
            entry: entry.clone(),
            ok: false,
            detail: format!("returned `{}`, not Bool", got),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization,
        },
        ClaimOutcome::RuntimeError { cause, message } => ClaimResult {
            function,
            entry: entry.clone(),
            ok: false,
            // The cause token leads the detail on this path too. `claim_batch` is the OTHER
            // reader of a throw, and leaving it prose-only would rebuild the same asymmetry the
            // floor just lost — one consumer able to partition, one not.
            detail: format!("runtime error [{}]: {}", cause.token(), message),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization,
        },
        ClaimOutcome::HostToolUnresolved { name, probed } => {
            let hint = if probed.is_empty() {
                "no candidates probed".to_string()
            } else {
                probed.join(", ")
            };
            let host_dependency_refusal = Some(HostDependencyRefusal {
                tool: name.clone(),
                hint,
            });
            ClaimResult {
                function,
                entry: entry.clone(),
                ok: false,
                detail: format!(
                    "host tool unresolved: {:?} (probed: {})",
                    name,
                    probed.join(", ")
                ),
                wall_nanos,
                resolve_nanos,
                corpus_resolve_nanos: 0,
                corpus_eval_nanos: 0,
                corpus_witnesses: 0,
                runtime_unit_count: single_claim_runtime_unit_count(),
                witness_row_costs: Vec::new(),
                expectation_refusal: None,
                budget_refusal: None,
                host_dependency_refusal,
                resolve_realization,
            }
        }
        // NOT ok, and NOT a host_dependency_refusal either. The row did not answer, so `ok`
        // is false; but the remedy is a route for an effect this execution mode cannot
        // realize, not a missing binary to provision, so it does not borrow
        // `HostDependencyRefusal`'s channel and misreport itself as a tool-chain gap.
        ClaimOutcome::HostEffectRefused { operation, ground } => ClaimResult {
            function,
            entry: entry.clone(),
            ok: false,
            detail: format!(
                "hermetic route has no arm for {}: {} — the claim never reached its subject",
                operation,
                v1_compiler::cli_run::hermetic_effect_ground_label(&ground)
            ),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization,
        },
        // ONE ARM PER READING. The prose used to be selected by matching a field this arm
        // could have wildcarded away; it is now selected by which variant was constructed, so
        // a receipt cannot say "killed ... elapsed is a ceiling" about a row that completed.
        ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms,
            budget_ms,
            kind,
        } => ClaimResult {
            function,
            entry: entry.clone(),
            ok: false,
            // The detail names the budget in prose for the human reading a log; `budget_refusal`
            // beside it is what classification reads, so the mode does not depend on this
            // wording. What the wording MUST get right is whether the number is a bound or a
            // measurement, and it used to say "killed ... elapsed is a ceiling" unconditionally.
            // That is true of an interrupted row and FALSE of a completed one: a
            // `CompletedOverBudget` row ran to completion, passed, and has an exact elapsed, so
            // asserting it was killed and censored is a fabricated fact in a durable receipt.
            // The old comment defended the wording ("'ceiling' is deliberate"), which is how it
            // survived — a considered-looking justification for a claim that only held on one
            // arm.
            detail: format!(
                "killed at its {} budget: at least {}ms elapsed against a {}ms budget \
                 (interrupted, so elapsed bounds the cost and does not measure it)",
                kind.label(),
                elapsed_at_least_ms,
                budget_ms
            ),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: Some(BudgetRefusal {
                elapsed_ms: elapsed_at_least_ms,
                budget_ms,
                kind,
                completion: v1_compiler::cli_run::BudgetCompletion::Interrupted,
            }),
            host_dependency_refusal: None,
            resolve_realization,
        },
        ClaimOutcome::CompletedOverBudget {
            elapsed_ms,
            budget_ms,
            kind,
        } => ClaimResult {
            function,
            entry: entry.clone(),
            ok: false,
            // The detail names the budget in prose for the human reading a log; `budget_refusal`
            // beside it is what classification reads, so the mode does not depend on this
            // wording. What the wording MUST get right is whether the number is a bound or a
            // measurement, and it used to say "killed ... elapsed is a ceiling" unconditionally.
            // That is true of an interrupted row and FALSE of a completed one: a
            // `CompletedOverBudget` row ran to completion, passed, and has an exact elapsed, so
            // asserting it was killed and censored is a fabricated fact in a durable receipt.
            // The old comment defended the wording ("'ceiling' is deliberate"), which is how it
            // survived — a considered-looking justification for a claim that only held on one
            // arm.
            detail: format!(
                "completed over its {} budget: exactly {}ms elapsed against a {}ms budget \
                 (ran to completion and passed, then was reclassified on cost)",
                kind.label(),
                elapsed_ms,
                budget_ms
            ),
            wall_nanos,
            resolve_nanos,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: Some(BudgetRefusal {
                elapsed_ms,
                budget_ms,
                kind,
                completion: v1_compiler::cli_run::BudgetCompletion::CompletedOverBudget,
            }),
            host_dependency_refusal: None,
            resolve_realization,
        },
    }
}

#[derive(Clone)]
struct NativeBundleProcessSpec {
    workspace_dir: String,
    bundle_identity: String,
    selected_count: u64,
    bundle_count: u64,
    shard_count: u64,
    files: Vec<(String, String)>,
    build: Vec<Vec<String>>,
    run: Vec<String>,
    expected_stdout: Vec<u8>,
}

struct NativeTransportObservation {
    success: bool,
    compile_skipped: bool,
    stdout: Vec<u8>,
    /// Which leg of the transport produced this observation — `build`, `run`, or
    /// `run_cached`. Without it a refusal cannot say whether the bundle failed to
    /// COMPILE or failed to RUN, which are different defects with different owners.
    phase: String,
    /// How the process ended. Carried rather than collapsed into `success` so a
    /// signalled process (a runner OOM-kill is the live candidate) stays separable
    /// from a process that reported a nonzero exit.
    termination: ProcessTermination,
    /// What the process said on stderr. This is the only channel that names the
    /// actual failure — a missing toolchain, a compile error, a panic — and dropping
    /// it is why the fleet's counted fallback has been undiagnosable.
    stderr: Vec<u8>,
    artifact_lookup_nanos: u128,
    cold_compile_nanos: u128,
    native_execution_nanos: u128,
}

/// Last `MAX` bytes of a process stream, rendered for a one-line diagnostic. The tail
/// rather than the head: a cargo build writes its error last, and a panic writes its
/// message last. Bounded so one refusal cannot flood the floor's result stream.
fn stream_excerpt(bytes: &[u8]) -> String {
    const MAX: usize = 1200;
    if bytes.is_empty() {
        return "<empty>".to_string();
    }
    let start = bytes.len().saturating_sub(MAX);
    // Trim BEFORE flattening: a stream almost always ends in a newline, and replacing
    // first leaves a dangling line marker with nothing after it.
    let tail = String::from_utf8_lossy(&bytes[start..])
        .trim()
        .replace(['\n', '\r'], " ⏎ ");
    if tail.is_empty() {
        return "<whitespace only>".to_string();
    }
    if start > 0 {
        format!("…(last {MAX} of {} bytes) {tail}", bytes.len())
    } else {
        tail
    }
}

fn native_bundle_u64_field(
    fields: &[(v1_compiler::v1_interpreter::Symbol, Value)],
    name: &str,
    ctx: &InterpContext,
) -> Result<u64, String> {
    match ctx.field(fields, name) {
        Some(Value::Int(n)) if *n >= 0 => Ok(*n as u64),
        Some(other) => Err(format!(
            "native bundle spec field `{name}` must be a non-negative Int, got {}",
            other.type_label_public()
        )),
        None => Err(format!("native bundle spec missing field `{name}`")),
    }
}

fn native_bundle_string_list(value: &Value, ctx: &InterpContext) -> Result<Vec<String>, String> {
    free_monoid_elems(value, ctx)?
        .into_iter()
        .map(|item| match item {
            Value::Str(s) => Ok(s.to_string()),
            other => Err(format!(
                "native bundle argv element must be String, got {}",
                other.type_label_public()
            )),
        })
        .collect()
}

fn native_bundle_spec_from_value(
    value: &Value,
    ctx: &InterpContext,
) -> Result<NativeBundleProcessSpec, String> {
    let outcome_fields = match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "Accepted") => fields,
        Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Rejected") => {
            return Err("native bundle selector returned typed Rejected".to_string())
        }
        other => {
            return Err(format!(
            "native bundle selector must return Outcome<NativeSelectedBundleProcessSpec>, got {}",
            other.type_label_public()
        ))
        }
    };
    let spec = ctx
        .field(outcome_fields, "value")
        .ok_or_else(|| "native bundle Accepted outcome missing `value`".to_string())?;
    let fields = match spec {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
        other => {
            return Err(format!(
                "native bundle selector value must be a record, got {}",
                other.type_label_public()
            ))
        }
    };
    let workspace_dir = str_field(fields, "workspace_dir", "native bundle spec", ctx)?;
    let bundle_identity = str_field(fields, "bundle_identity", "native bundle spec", ctx)?;
    if bundle_identity.is_empty() || !workspace_dir.contains(&bundle_identity) {
        return Err(
            "native bundle artifact identity is absent from its workspace path".to_string(),
        );
    }
    let files_value = ctx
        .field(fields, "files")
        .ok_or_else(|| "native bundle spec missing `files`".to_string())?;
    let mut files = Vec::new();
    for file in free_monoid_elems(files_value, ctx)? {
        let ff = match file {
            Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
            other => {
                return Err(format!(
                    "native bundle file must be a record, got {}",
                    other.type_label_public()
                ))
            }
        };
        files.push((
            str_field(ff, "path", "native bundle file", ctx)?,
            str_field(ff, "text", "native bundle file", ctx)?,
        ));
    }
    let build_value = ctx
        .field(fields, "build")
        .ok_or_else(|| "native bundle spec missing `build`".to_string())?;
    let build = free_monoid_elems(build_value, ctx)?
        .into_iter()
        .map(|argv| native_bundle_string_list(argv, ctx))
        .collect::<Result<Vec<_>, _>>()?;
    let run = native_bundle_string_list(
        ctx.field(fields, "run")
            .ok_or_else(|| "native bundle spec missing `run`".to_string())?,
        ctx,
    )?;
    let expected_stdout = free_monoid_elems(
        ctx.field(fields, "expected_stdout_octets")
            .ok_or_else(|| "native bundle spec missing `expected_stdout_octets`".to_string())?,
        ctx,
    )?
    .into_iter()
    .map(|octet| match octet {
        Value::Int(n) if (0..=255).contains(n) => Ok(*n as u8),
        _ => Err("native bundle expected stdout contains a non-octet".to_string()),
    })
    .collect::<Result<Vec<_>, _>>()?;
    let spec = NativeBundleProcessSpec {
        workspace_dir,
        bundle_identity,
        selected_count: native_bundle_u64_field(fields, "selected_count", ctx)?,
        bundle_count: native_bundle_u64_field(fields, "bundle_count", ctx)?,
        shard_count: native_bundle_u64_field(fields, "shard_count", ctx)?,
        files,
        build,
        run,
        expected_stdout,
    };
    if spec.selected_count != 3 || spec.bundle_count != 1 || spec.shard_count != 1 {
        return Err(format!(
            "native bundle bounded-population refusal: selected={} bundle={} shard={} (required 3/1/1)",
            spec.selected_count, spec.bundle_count, spec.shard_count
        ));
    }
    if spec.files.is_empty() || spec.build.is_empty() || spec.run.is_empty() {
        return Err("native bundle process spec has empty files/build/run".to_string());
    }
    Ok(spec)
}

fn native_transport_observation(
    value: &Value,
    ctx: &InterpContext,
) -> Result<NativeTransportObservation, String> {
    let fields = match value {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
        other => {
            return Err(format!(
                "native transport result must be a record, got {}",
                other.type_label_public()
            ))
        }
    };
    let boolean = |name: &str| match ctx.field(fields, name) {
        Some(Value::Bool(v)) => Ok(*v),
        _ => Err(format!("native transport result missing Bool `{name}`")),
    };
    let nanos = |name: &str| native_bundle_u64_field(fields, name, ctx).map(u128::from);
    let octets = |name: &str| -> Result<Vec<u8>, String> {
        free_monoid_elems(
            ctx.field(fields, name)
                .ok_or_else(|| format!("native transport result missing {name}"))?,
            ctx,
        )?
        .into_iter()
        .map(|v| match v {
            Value::Int(n) if (0..=255).contains(n) => Ok(*n as u8),
            _ => Err(format!("native transport {name} contains a non-octet")),
        })
        .collect::<Result<Vec<_>, _>>()
    };
    // The transport's phase vocabulary is closed; an unknown phase is malformed wire,
    // not a new kind of leg the receipt should silently carry.
    let phase = match ctx.field(fields, "phase") {
        Some(Value::Str(s)) if matches!(s.as_ref(), "build" | "run" | "run_cached") => {
            s.to_string()
        }
        Some(Value::Str(s)) => {
            return Err(format!(
                "native transport phase `{s}` is outside build|run|run_cached"
            ))
        }
        _ => return Err("native transport result missing String `phase`".to_string()),
    };
    let termination =
        transport_termination(ctx.field(fields, "termination"), ctx).map_err(|refusal| {
            format!(
                "native transport termination wire refused: {}",
                refusal.located
            )
        })?;
    Ok(NativeTransportObservation {
        // DERIVED, never a second field. The transport used to carry `success`
        // alongside the exit code, which is one fact written twice; a receipt could
        // then claim success beside a nonzero termination and nothing would notice.
        success: matches!(termination, ProcessTermination::Exited(0)),
        compile_skipped: boolean("compile_skipped")?,
        stdout: octets("stdout_octets")?,
        phase,
        termination,
        stderr: octets("stderr_octets")?,
        artifact_lookup_nanos: nanos("artifact_lookup_nanos")?,
        cold_compile_nanos: nanos("cold_compile_nanos")?,
        native_execution_nanos: nanos("native_execution_nanos")?,
    })
}

fn run_native_transport(
    spec: &NativeBundleProcessSpec,
    ctx: &InterpContext,
) -> Result<NativeTransportObservation, String> {
    let value = v1_compiler::v1_interpreter::run_native_bundle_process_cached(
        ctx,
        spec.workspace_dir.clone(),
        &spec.files,
        &spec.build,
        &spec.run,
    )
    .map_err(|e| e.to_string())?;
    native_transport_observation(&value, ctx)
}

fn write_native_transition_receipt(body: &str) -> Result<(), String> {
    let path = Path::new("target/native-selected-witness-transition-receipt.tsv");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("native transition receipt mkdir: {e}"))?;
    }
    fs::write(path, body).map_err(|e| format!("native transition receipt write: {e}"))
}

/// Native execution is AUTHORITATIVE for this population (CI-0 cutover,
/// operator direction 2026-08-08): the counted interpreter fallback is DELETED,
/// not gated. The arm it occupied was the §5 absorbing fallback in its purest
/// form — from the first enrolled main run (30774223741, 2026-08-03) through
/// the cutover every floor run cold-failed natively and reported green through
/// interpretation, and the "srv4-03 outage" that justified the arm (run
/// 30764923923, review 47508) was the same deterministic failure the arm then
/// zeroed the frequency of for four days. Receipt:
/// docs/plans/ci2-native-cutover-reobservation.md.
///
/// ACCEPTED requires the full native bar: native ran green twice (cold + warm,
/// warm compile-skipped), the interpreter oracle agrees (the retained
/// equivalence evidence, §4b dissolution-on-climb — the evidence stays, the
/// production fallback machinery does not), and the planted RED built and
/// reproduced its wrong output natively. Anything else — outage, divergence,
/// oracle red, planted-red non-equivalence — is a typed, located refusal
/// carrying the transport causes (build log + stderr tail). Interpreted and
/// fallback counts are 0 by construction: no code path can produce another
/// value.
fn native_transition_accepted(
    native_ok: bool,
    oracle_green: bool,
    planted_red_equivalent: bool,
) -> bool {
    native_ok && oracle_green && planted_red_equivalent
}

/// A member that RAN natively but produced the wrong output is DIVERGED, not
/// unavailable: the native realization exists and executed, so counting it in
/// the outage column over-claims "unavailable" and hides the scarier defect
/// (wrong answer) inside the milder one (no answer). The two columns carry
/// different remedies — an outage is fixed in the transport/build environment,
/// a divergence is a semantics defect in the emitted code.
fn native_transition_population_counts(
    selected: u64,
    native_ok: bool,
    native_diverged: bool,
) -> (u64, u64, u64, u64, u64) {
    let native_count = if native_ok { selected } else { 0 };
    let diverged_count = if !native_ok && native_diverged {
        selected
    } else {
        0
    };
    let unavailable_count = if native_ok || native_diverged {
        0
    } else {
        selected
    };
    // interpreted_count and fallback_count: the interpreter is not a production
    // execution route for this population; both are structurally zero.
    (native_count, 0, unavailable_count, diverged_count, 0)
}

fn run_native_bundle_unit(
    source_roots: &[String],
    entry: String,
    selector_function: String,
    execution_mode: ExecutionMode,
) -> ClaimResult {
    let started = Instant::now();
    let fail_before_resolve = |detail: String| ClaimResult {
        function: selector_function.clone(),
        entry: entry.clone(),
        ok: false,
        detail,
        wall_nanos: started.elapsed().as_nanos(),
        resolve_nanos: 0,
        corpus_resolve_nanos: 0,
        corpus_eval_nanos: 0,
        corpus_witnesses: 3,
        runtime_unit_count: FloorRuntimeUnitCount::Observed { units: 3 },
        witness_row_costs: Vec::new(),
        expectation_refusal: None,
        budget_refusal: None,
        host_dependency_refusal: None,
        resolve_realization: None,
    };
    if execution_mode != ExecutionMode::Wet {
        return fail_before_resolve(
            "NativeBundle handler requires Wet execution_mode (typed envelope refusal)".to_string(),
        );
    }
    let resolve_started = Instant::now();
    let (graph, indices) = match resolve_entry_graph(source_roots, &entry) {
        Ok(v) => v,
        Err(e) => {
            return fail_before_resolve(format!("native bundle selector resolve refusal: {e}"))
        }
    };
    let resolve_nanos = resolve_started.elapsed().as_nanos();
    let resolve_observation =
        || Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos });
    let fail_after_resolve = |detail: String| ClaimResult {
        function: selector_function.clone(),
        entry: entry.clone(),
        ok: false,
        detail,
        wall_nanos: started.elapsed().as_nanos(),
        resolve_nanos,
        corpus_resolve_nanos: 0,
        corpus_eval_nanos: 0,
        corpus_witnesses: 3,
        runtime_unit_count: FloorRuntimeUnitCount::Observed { units: 3 },
        witness_row_costs: Vec::new(),
        expectation_refusal: None,
        budget_refusal: None,
        host_dependency_refusal: None,
        resolve_realization: resolve_observation(),
    };
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
    let primary = match run_in_context(&ctx, &selector_function, false)
        .map_err(|e| e.to_string())
        .and_then(|v| native_bundle_spec_from_value(&v, &ctx))
    {
        Ok(spec) => spec,
        Err(e) => return fail_after_resolve(format!("native bundle selector refusal: {e}")),
    };
    let planted = run_in_context(&ctx, "native_selected_logic_planted_red_spec", false)
        .map_err(|e| e.to_string())
        .and_then(|v| native_bundle_spec_from_value(&v, &ctx));

    let cold = run_native_transport(&primary, &ctx);
    let warm = match &cold {
        Ok(obs) if obs.success => run_native_transport(&primary, &ctx),
        _ => Err("primary cold artifact unavailable".to_string()),
    };
    let planted_native = planted
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|spec| run_native_transport(spec, &ctx));

    let oracle_started = Instant::now();
    let oracle_green = matches!(
        run_claim(&ctx, "native_selected_logic_interpreter_oracle_holds"),
        ClaimOutcome::Pass
    );
    let planted_oracle = matches!(
        run_claim(&ctx, "native_selected_logic_planted_red_oracle_holds"),
        ClaimOutcome::Pass
    );
    let interpreter_oracle_wall_nanos = oracle_started.elapsed().as_nanos();

    let native_ok = matches!((&cold, &warm), (Ok(c), Ok(w))
        if c.success && w.success && w.compile_skipped
            && c.stdout == primary.expected_stdout && w.stdout == primary.expected_stdout);
    // Divergence = the native realization RAN (process success) and produced output the
    // spec did not declare. Distinct from an outage (transport Err / process failure);
    // both refuse — the distinction survives only in the verdict label.
    let leg_diverged = |leg: &Result<NativeTransportObservation, String>| matches!(leg, Ok(obs) if obs.success && obs.stdout != primary.expected_stdout);
    let native_diverged = leg_diverged(&cold) || leg_diverged(&warm);
    let planted_red_equivalent = planted
        .as_ref()
        .ok()
        .zip(planted_native.as_ref().ok())
        .map(|(spec, obs)| obs.success && obs.stdout == spec.expected_stdout && planted_oracle)
        .unwrap_or(false);
    let accepted = native_transition_accepted(native_ok, oracle_green, planted_red_equivalent);
    // The transport causes are the located half of any non-accepted verdict; dropping
    // them made CI's outage red opaque (run 30764923923 refused with no cause on the
    // wire). Rendered into the FAIL/fallback detail and stderr, never into the TSV
    // receipt (its shape is a parsed contract).
    //
    // A failed leg names its PHASE, its TERMINATION, and its STDERR. The three
    // together are what makes a fleet refusal actionable: the phase says whether the
    // bundle failed to compile or failed to run, the termination separates a
    // signalled process (an OOM-kill) from one that reported a nonzero exit, and
    // stderr carries the message the tool actually produced. Before this, every one
    // of these collapsed into the four words "process failed", which is why a counted
    // fallback could sit on the fleet indefinitely without ranking for a fix.
    let process_failure = |name: &str, obs: &NativeTransportObservation| {
        format!(
            "{name}: process failed in phase `{}` ({}); stderr: {}",
            obs.phase,
            obs.termination.located(),
            stream_excerpt(&obs.stderr)
        )
    };
    let leg_cause = |name: &str, leg: &Result<NativeTransportObservation, String>| match leg {
        Ok(obs) if obs.success && obs.stdout == primary.expected_stdout => None,
        Ok(obs) if obs.success => Some(format!(
            "{name}: ran but diverged (stdout {} bytes != expected {} bytes); stdout: {}",
            obs.stdout.len(),
            primary.expected_stdout.len(),
            stream_excerpt(&obs.stdout)
        )),
        Ok(obs) => Some(process_failure(name, obs)),
        Err(e) => Some(format!(
            "{name}: transport refused before a process ran: {e}"
        )),
    };
    let transport_causes: Vec<String> = [
        leg_cause("cold", &cold),
        leg_cause("warm", &warm),
        match &planted_native {
            Ok(obs) if obs.success => None,
            Ok(obs) => Some(process_failure("planted-native", obs)),
            Err(e) => Some(format!(
                "planted-native: transport refused before a process ran: {e}"
            )),
        },
    ]
    .into_iter()
    .flatten()
    .collect();
    if !transport_causes.is_empty() {
        eprintln!(
            "[native-selected-bundle] transport causes: {}",
            transport_causes.join(" | ")
        );
    }
    let selected = primary.selected_count;
    let (native_count, interpreted_count, unavailable_count, diverged_count, fallback_count) =
        native_transition_population_counts(selected, native_ok, native_diverged);
    let cold_compile_wall = cold
        .as_ref()
        .ok()
        .map(|o| o.cold_compile_nanos)
        .unwrap_or(0);
    let warm_artifact_hit_wall = warm
        .as_ref()
        .ok()
        .map(|o| o.artifact_lookup_nanos)
        .unwrap_or(0);
    let native_execution_wall = warm
        .as_ref()
        .ok()
        .map(|o| o.native_execution_nanos)
        .unwrap_or(0);
    let rss_peak = peak_rss_bytes().unwrap_or(0);
    let cgroup_peak = cgroup_job_measurement().map(|m| m.leaf_peak).unwrap_or(0);
    let verdict = if accepted {
        "accepted"
    } else if native_diverged {
        "refused:native_divergence"
    } else if !native_ok {
        "refused:native_realization_unavailable"
    } else {
        "refused:equivalence_or_planted_red"
    };
    // The located causes belong IN the persisted receipt, not only on stderr. The
    // coordinator's own note records that the Actions stream drops worker stderr, so a
    // cause that lives only there is a cause the fleet cannot be asked about after the
    // fact. One row per failed leg, so the count of causes is readable too; a green run
    // writes no such row rather than an empty one.
    let cause_rows: String = transport_causes
        .iter()
        .map(|c| format!("transport_cause\t{}\n", c.replace(['\t', '\n'], " ")))
        .collect();
    let receipt = format!(
        "selected_witness_count\t{selected}\nnative_count\t{native_count}\ninterpreted_count\t{interpreted_count}\nunavailable_count\t{unavailable_count}\nbundle_count\t{}\nshard_count\t{}\ncold_compile_wall_nanos\t{cold_compile_wall}\nwarm_artifact_hit_wall_nanos\t{warm_artifact_hit_wall}\nnative_execution_wall_nanos\t{native_execution_wall}\ninterpreter_oracle_wall_nanos\t{interpreter_oracle_wall_nanos}\ndiverged_count\t{diverged_count}\nfallback_count\t{fallback_count}\nrss_peak_bytes\t{rss_peak}\ncgroup_peak_bytes\t{cgroup_peak}\nverdict\t{verdict}\nplanted_red_equivalent\t{planted_red_equivalent}\nbundle_identity\t{}\n{cause_rows}",
        primary.bundle_count, primary.shard_count, primary.bundle_identity
    );
    if let Err(e) = write_native_transition_receipt(&receipt) {
        return fail_after_resolve(e);
    }
    eprintln!("[native-selected-bundle] {}", receipt.replace('\n', " "));
    ClaimResult {
        function: selector_function,
        entry,
        ok: accepted,
        detail: if accepted {
            receipt
        } else {
            format!(
                "native transition refused ({}); {receipt}",
                transport_causes.join(" | ")
            )
        },
        wall_nanos: started.elapsed().as_nanos(),
        resolve_nanos,
        corpus_resolve_nanos: 0,
        corpus_eval_nanos: interpreter_oracle_wall_nanos,
        corpus_witnesses: selected as usize,
        runtime_unit_count: FloorRuntimeUnitCount::Observed {
            units: selected as u128,
        },
        witness_row_costs: Vec::new(),
        expectation_refusal: None,
        budget_refusal: None,
        host_dependency_refusal: None,
        resolve_realization: Some(ResolveRealizationObservation::ColdResolvePerformed {
            resolve_nanos,
        }),
    }
}

fn scoped_execution_authority_source_roots(
    authority: ScopedWitnessExecutionAuthority,
    walk_source_roots: &[String],
) -> Vec<String> {
    match authority {
        ScopedWitnessExecutionAuthority::InheritedWalkSourceRoots => walk_source_roots.to_vec(),
    }
}

fn run_batch_unit(
    source_roots: Vec<String>,
    unit: BatchUnit,
    governor: Arc<RealizationConcurrency>,
    fast_lane_eval_budget_ms: Option<u64>,
    falsifier_self_host_wet_budgets: FalsifierSelfHostWetBudgets,
    obligation_subjects: Option<&ObligationSubjectSet>,
) -> Vec<ClaimResult> {
    match unit {
        BatchUnit::UnrunnableSentinel { function } => vec![ClaimResult {
            function,
            // An unmapped node never bound to a declaration, so there is no entry to
            // name. Said explicitly rather than blanked.
            entry: "<unmapped node — no declaring entry>".to_string(),
            ok: false,
            detail: "unrunnable sentinel (unmapped node or non-complete plan) — failing closed"
                .to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        }],
        BatchUnit::NativeBundle {
            entry,
            selector_function,
            execution_mode,
        } => {
            let slot =
                RealizationSlot::acquire_blocking(&governor, &format!("native-bundle {entry}"));
            let result =
                run_native_bundle_unit(&source_roots, entry, selector_function, execution_mode);
            slot.note_unit_complete();
            vec![result]
        }
        BatchUnit::Discovery {
            source_roots: roots,
            scan_dirs,
            explicit_entries,
            exclude_substrings,
            discovery_scope_dirs,
            execution_mode,
            spawns_host_compiler,
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
            let expected_red = falsifier_self_host_wet_budgets
                .expected_red_witnesses
                .clone();
            let execution_authority_source_roots = roots.clone();
            vec![run_discovery_batch_node(
                roots,
                execution_authority_source_roots,
                scan_dirs,
                explicit_entries,
                exclude_substrings,
                discovery_scope_dirs,
                governor,
                execution_mode,
                spawns_host_compiler,
                effective_fast_lane,
                wet_wall_budget_ms,
                wet_interp_budget_ms,
                expected_red,
                falsifier_self_host_wet_budgets
                    .pre_verdict_refusal_witnesses
                    .clone(),
                None,
            )]
        }
        BatchUnit::ScopedDiscovery {
            batch_id,
            source_roots_digest,
            entries_with_kind,
            source_roots: roots,
            scan_dirs,
            execution_authority,
            execution_mode,
            spawns_host_compiler,
        } => {
            let explicit_entries: Vec<(String, String)> = entries_with_kind
                .iter()
                .map(|row| (row.entry.clone(), row.function.clone()))
                .collect();
            let execution_authority_source_roots =
                scoped_execution_authority_source_roots(execution_authority, &source_roots);
            vec![run_discovery_batch_node(
                roots,
                execution_authority_source_roots,
                scan_dirs,
                explicit_entries,
                Vec::new(),
                Vec::new(),
                governor,
                execution_mode,
                spawns_host_compiler,
                fast_lane_eval_budget_ms,
                None,
                None,
                // Scoped batches never computed a polarity at all — the batch-wide flag was
                // hardcoded false here, so a known-red witness reached by this route reddened
                // its component. Function-grain matching makes the same roster correct on
                // every route, which is the point of moving the fact onto the witness.
                falsifier_self_host_wet_budgets
                    .expected_red_witnesses
                    .clone(),
                falsifier_self_host_wet_budgets
                    .pre_verdict_refusal_witnesses
                    .clone(),
                Some(ScopedReceiptBatch {
                    batch_id,
                    source_roots_digest,
                    entries: entries_with_kind,
                }),
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
            let slot = RealizationSlot::acquire_blocking(&governor, &format!("gate-unit {entry}"));
            let results = run_shared_entry_claims(
                &source_roots,
                &entry,
                &functions,
                execution_mode,
                obligation_subjects,
            );
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
    obligation_subjects: Option<&ObligationSubjectSet>,
) -> Vec<ClaimResult> {
    let resolve_start = Instant::now();
    let (graph, source_indices) = match resolve_entry_graph(source_roots, entry) {
        Ok(pair) => pair,
        Err(msg) => {
            return functions
                .iter()
                .map(|function| ClaimResult {
                    function: function.clone(),
                    entry: entry.to_string(),
                    ok: false,
                    detail: format!("resolve failed for {}: {}", entry, msg),
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: None,
                })
                .collect();
        }
    };
    let resolve_nanos = resolve_start.elapsed().as_nanos();
    let group_resolve_observation =
        Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos });
    let ctx = make_eval_context(&graph, source_indices, execution_mode);
    let mut first_physical = true;
    let mut group_observation_attached = false;
    functions
        .iter()
        .map(|function| {
            set_phase(FloorPhase::Gate, &format!("{entry}::{function}"));
            let claim_start = Instant::now();
            active_workset_admit(entry, function);
            let outcome = run_claim(&ctx, function);
            active_workset_complete(entry, function);
            // Witness frame exit: the memo must not retain values across
            // witnesses sharing this ctx (byte-unbounded, 20GiB-class kills).
            v1_compiler::v1_interpreter::eval_call_memo_frame_exit(&ctx);
            let wall_nanos = claim_start.elapsed().as_nanos();
            let (rn, observation) = {
                let rn = if first_physical {
                    first_physical = false;
                    resolve_nanos
                } else {
                    0
                };
                let observation = take_group_observation_for_claim(
                    obligation_subjects,
                    entry,
                    function,
                    &group_resolve_observation,
                    &mut group_observation_attached,
                );
                (rn, observation)
            };
            claim_result_for_outcome(
                &ctx,
                function.clone(),
                entry.to_string(),
                outcome,
                wall_nanos,
                rn,
                observation,
            )
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
    obligation_subjects: Option<&ObligationSubjectSet>,
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
                        entry: entry.to_string(),
                        ok: false,
                        detail: format!("resolve failed for {}: {}", entry, msg),
                        wall_nanos: 0,
                        resolve_nanos: 0,
                        corpus_resolve_nanos: 0,
                        corpus_eval_nanos: 0,
                        corpus_witnesses: 0,
                        runtime_unit_count: single_claim_runtime_unit_count(),
                        witness_row_costs: Vec::new(),
                        expectation_refusal: None,
                        budget_refusal: None,
                        host_dependency_refusal: None,
                        resolve_realization: None,
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
    let group_resolve_observation = if fresh_resolve {
        Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos })
    } else {
        Some(ResolveRealizationObservation::SatisfiedFromSharedPool {
            computation_identity: format!("entry-closure:{entry}:{execution_mode:?}"),
            provider_id: FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID.to_string(),
        })
    };
    let ctx = memo.get(&memo_key).expect("memo populated above");
    let mut first_physical = true;
    let mut group_observation_attached = false;
    functions
        .iter()
        .map(|function| {
            set_phase(FloorPhase::Gate, &format!("{entry}::{function}"));
            let claim_start = Instant::now();
            active_workset_admit(entry, function);
            let outcome = run_claim(ctx, function);
            active_workset_complete(entry, function);
            // Witness frame exit — this memoized ctx outlives whole entry
            // groups, so per-witness release matters here most of all.
            v1_compiler::v1_interpreter::eval_call_memo_frame_exit(ctx);
            let wall_nanos = claim_start.elapsed().as_nanos();
            let (rn, observation) = {
                let rn = if first_physical {
                    first_physical = false;
                    resolve_nanos
                } else {
                    0
                };
                let observation = take_group_observation_for_claim(
                    obligation_subjects,
                    entry,
                    function,
                    &group_resolve_observation,
                    &mut group_observation_attached,
                );
                (rn, observation)
            };
            claim_result_for_outcome(
                ctx,
                function.clone(),
                entry.to_string(),
                outcome,
                wall_nanos,
                rn,
                observation,
            )
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
                (Some("title".to_string()), str_value(title.to_string())),
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
            Value::Str(s) => Ok(s.to_string()),
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
            .map(str_value)
            .collect::<Vec<_>>()
            .into(),
    ))
}

/// Render the top-N slowest witnesses through `dag/gunbc/ci_render.dag`.
fn render_slowest_witnesses(
    source_roots: &[String],
    rows: &[WitnessRowCost],
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
                    str_value(row.function.clone()),
                ),
                (Some("entry".to_string()), str_value(row.entry.clone())),
                (
                    Some("eval_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.eval_wall_nanos)),
                ),
                (
                    Some("resolve_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.resolve_nanos)),
                ),
                (
                    Some("total_ns".to_string()),
                    Value::Int(clamp_nanos_to_i64(row.warm_nanos)),
                ),
            ],
            false,
        )
        .map_err(|e| format!("slowest_witness_row eval failed: {e}"))?;
        match line {
            Value::Str(s) => body_lines.push(s.to_string()),
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
        Value::Str(s) => Ok(s.to_string()),
        other => Err(format!(
            "render_slowest_witnesses_box returned non-string: {other}"
        )),
    }
}

fn emit_slowest_witness_attribution(source_roots: &[String], rows: &[WitnessRowCost]) {
    let n = slowest_witness_attribution_n().min(rows.len());
    if n == 0 {
        return;
    }
    let top = top_n_slowest_witnesses(rows, n);
    match render_slowest_witnesses(source_roots, &top) {
        Ok(boxed) => {
            eprintln!("{boxed}");
            let tail_eval_ms: u128 =
                top.iter().map(|r| r.eval_wall_nanos).sum::<u128>() / 1_000_000;
            let total_eval_ms = rows.iter().map(|r| r.eval_wall_nanos).sum::<u128>() / 1_000_000;
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

/// Whether EVERY entry in this batch declared `ExpectTypedPreVerdictRefusal`.
///
/// THE NARROW ROSTER, NOT THE KNOWN-RED ROSTER. This gates the one arm where no per-witness
/// outcome exists — a corpus resolve refuse — and reporting that arm as agreement is a real
/// claim: it says the run stopped exactly where the admission row said it would. The first
/// shape of this function asked whether every entry was expected-red at all, so ANY resolve or
/// evaluation failure anywhere in a known-red batch reported every quarantine as holding, with
/// `ok: true`. That contradicts the rule the same change established one screen up — agreement
/// means the assertion RAN and returned false — and it is the widest possible reading of a
/// narrow legacy fact: three witnesses that genuinely cannot resolve, because the resolver does
/// not bind imported bare variant constructors in expression position.
///
/// So only rows that DECLARED a pre-verdict refusal, naming its phase and cause, can reach the
/// inversion (`std.witness_admission` `ExpectTypedPreVerdictRefusal`, projected by
/// `gunbc.explicit_witness_admission` `known_red_pre_verdict_refusal_roster`). An ordinary
/// known-red row now keeps the refusal loud, because for it a resolve failure is an
/// infrastructure fact about the run and proves neither the red nor the quarantine.
///
/// BOUNDED COMPATIBILITY ARM, stated rather than claimed: the declaration GATES the inversion
/// but the OBSERVED cause is not yet compared against the declared one, because `run_claim`
/// formats the typed `InterpError` into a `String` before any outcome exists and the only way
/// to recover the class here would be to parse that prose — the mechanism this whole lane
/// exists to remove. Dissolve-on is the same seam trigger `gunbc.witness_row_cost`
/// `witness_cost_timed_out_seed_deferral_note` already carries: when the typed error survives
/// into the judgement, `PreVerdictPhase` and `PreVerdictCause` are matched exactly and a
/// mismatch refuses. The `all` form is on purpose — one entry outside the roster keeps the
/// refusal loud — and an empty batch is never agreement.
fn batch_entries_all_expect_pre_verdict_refusal(
    explicit_entries: &[(String, String)],
    pre_verdict_refusal: &[(String, String)],
) -> bool {
    batch_entries_all_in(explicit_entries, pre_verdict_refusal)
}

/// Every entry of a nonempty batch appears on `roster`, at the roster's own grain (an empty
/// `function` is `ScheduleWitnessEntry`'s file-grain form). An empty batch is never "all".
fn batch_entries_all_in(
    explicit_entries: &[(String, String)],
    roster: &[(String, String)],
) -> bool {
    if explicit_entries.is_empty() {
        return false;
    }
    explicit_entries.iter().all(|(entry, function)| {
        roster
            .iter()
            .any(|(e, f)| e == entry && (f == function || f.is_empty()))
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
/// The fast-lane eval budget (operator ruling 2026-08-17, superseding the 5s rule of 2026-07-12;
/// the live ceiling is `v2.workflow.required_floor` `required_floor_claim_cpu_safety_limit_ms` /
/// `required_floor_claim_wall_safety_limit_ms` — two independent
/// safety deadlines per the 2026-08-19 budget policy cut's superseding correction, never
/// transcribed here) governs the per-PR discovery corpus and its
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

/// `(entry, function)` identities from a `List<ScheduleWitnessEntry>` plan function.
///
/// The function slot is what makes expectation a per-WITNESS fact. `read_schedule_witness_entry_paths`
/// drops it, and every consumer that reads polarity through the path-only form inherits file
/// grain: a green sibling in a quarantined file gets quarantined with it.
fn read_schedule_witness_entry_pairs(
    plan_ctx: &InterpContext,
    function: &str,
) -> Result<Vec<(String, String)>, String> {
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
                out.push((
                    str_field(&fields, "entry", function, plan_ctx)?,
                    str_field(&fields, "function", function, plan_ctx)?,
                ));
            }
            Ok(out)
        }
        Err(msg) => Err(format!(
            "claim_executor: {function} is unavailable (fail-closed): {msg}"
        )),
    }
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
/// budget arms actually live (`resolution_divergence_silent_pick_gate_keystone_holds` is a
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
        // Both budget arms lift, through the projection rather than a wildcard: this carrier
        // keeps the axis as data, so it wants the pair, not one of the two arms.
        .find_map(|w| {
            w.outcome.budget_event().map(|event| BudgetRefusal {
                elapsed_ms: event.elapsed_ms,
                budget_ms: event.budget_ms,
                kind: event.kind,
                completion: event.completion,
            })
        })
}

/// Lift the first host-dependency refusal among this discovery batch's failure lines.
///
/// Parallel to `discovery_budget_refusal`: falsifier batch 5 runs Codex materialization
/// witnesses as a `RunnableDiscoveryBatch`, flattening N reds into one `ClaimResult`.
/// The failure-receipt wire is appended in `cli_run` on discovery reds, but
/// `batch_failure_mode_and_detail` reads `host_dependency_refusal` off the value — not
/// substring-matching `detail` — so this lift is required for `HostDependencyAbsent` on
/// the primary falsifier path (run 31685755058 component 6). Dissolve-on:
/// `gunbc.witness_row_cost` `host_dependency_refusal_seed_deferral_note`.
fn discovery_host_dependency_refusal(summary: &DiscoverySummary) -> Option<HostDependencyRefusal> {
    summary
        .failures
        .iter()
        .find_map(|failure| host_dependency_refusal_from_detail(failure))
}

const SCOPED_WITNESS_RECEIPT_PATH: &str = "target/scoped-witness-execution-receipt.tsv";
const SCOPED_WITNESS_RECEIPT_HEADER: &str =
    "head_sha\tbatch_id\tsource_roots_digest\tentry\tfunction\twitness_kind\toutcome\tdetail";

fn scoped_witness_head_sha() -> Result<String, String> {
    let head = std::env::var("GITHUB_SHA")
        .map_err(|_| "scoped witness execution requires explicit GITHUB_SHA".to_string())?;
    if head.len() == 40
        && head
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(head)
    } else {
        Err("scoped witness execution requires lowercase 40-hex GITHUB_SHA".to_string())
    }
}

fn initialize_scoped_witness_receipt() -> Result<(), String> {
    let path = Path::new(SCOPED_WITNESS_RECEIPT_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    std::fs::write(path, SCOPED_WITNESS_RECEIPT_HEADER)
        .map_err(|e| format!("write {}: {e}", path.display()))
}

fn scoped_wire_text(text: &str) -> String {
    text.replace(['\t', '\r', '\n'], " ")
}

fn scoped_witness_summary_outcome(
    summary: &DiscoverySummary,
    entry: &str,
    function: &str,
) -> Option<(&'static str, String)> {
    if let Some(found) = summary
        .witness_outcomes
        .iter()
        .find(|row| row.entry == entry && row.function == function)
    {
        return Some(match &found.outcome {
            ClaimOutcome::Pass => ("executed", "true".to_string()),
            ClaimOutcome::Fail
            | ClaimOutcome::NotBool { .. }
            | ClaimOutcome::RuntimeError { .. }
            | ClaimOutcome::HostToolUnresolved { .. } => ("executed", "false".to_string()),
            // ITS OWN TAG, never "executed". The tag is what downstream reads, and this row
            // did NOT execute its subject — reporting it as an execution that answered false
            // is the conflation the typed outcome exists to remove.
            ClaimOutcome::HostEffectRefused { operation, ground } => (
                "route-gap",
                format!(
                    "operation={} ground={}",
                    operation,
                    v1_compiler::cli_run::hermetic_effect_ground_label(ground)
                ),
            ),
            // "budget-killed" was unconditional and is only true of an interrupted row; a
            // completed one passed and was reclassified on an exact cost. The tag is what
            // downstream reads, so a wrong tag is worse than wrong prose — and it is now the ARM
            // that selects the tag, so the two cannot be given one name by a wildcard.
            ClaimOutcome::BudgetInterrupted {
                elapsed_at_least_ms,
                budget_ms,
                kind,
            } => (
                "budget-killed",
                format!(
                    "{} elapsed_ms={} budget_ms={} elapsed_is=at least",
                    kind.label(),
                    elapsed_at_least_ms,
                    budget_ms
                ),
            ),
            ClaimOutcome::CompletedOverBudget {
                elapsed_ms,
                budget_ms,
                kind,
            } => (
                "budget-exceeded-completed",
                format!(
                    "{} elapsed_ms={} budget_ms={} elapsed_is=exactly",
                    kind.label(),
                    elapsed_ms,
                    budget_ms
                ),
            ),
        });
    }
    None
}

fn append_scoped_witness_receipt_rows(
    batch_id: &str,
    source_roots_digest: &str,
    entries: &[ScopedScheduleEntry],
    summary: Option<&DiscoverySummary>,
    scheduling_detail: Option<&str>,
) -> Result<(), String> {
    let head = scoped_witness_head_sha()?;
    let pairs: Vec<(String, String)> = entries
        .iter()
        .map(|row| (row.entry.clone(), row.function.clone()))
        .collect();
    let expanded = v1_compiler::cli_run::expand_explicit_witness_entries(&pairs)?;
    let mut body = String::new();
    for (entry, function) in expanded {
        let kind = entries
            .iter()
            .find(|row| row.entry == entry && (row.function.is_empty() || row.function == function))
            .map(|row| row.witness_kind.as_str())
            .ok_or_else(|| {
                format!("expanded scoped witness row {entry}::{function} has no schedule kind")
            })?;
        let (outcome, detail) = if let Some(reason) = scheduling_detail {
            ("scheduling-refused", reason.to_string())
        } else if let Some(outcome) =
            summary.and_then(|s| scoped_witness_summary_outcome(s, &entry, &function))
        {
            outcome
        } else {
            (
                "scheduling-refused",
                "enrolled row produced no execution or selection fact".to_string(),
            )
        };
        body.push_str(&format!(
            "\n{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            head,
            scoped_wire_text(batch_id),
            scoped_wire_text(source_roots_digest),
            scoped_wire_text(&entry),
            scoped_wire_text(&function),
            kind,
            outcome,
            scoped_wire_text(&detail),
        ));
    }
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(SCOPED_WITNESS_RECEIPT_PATH)
        .and_then(|mut file| file.write_all(body.as_bytes()))
        .map_err(|e| format!("append {SCOPED_WITNESS_RECEIPT_PATH}: {e}"))
}

fn discovery_claim_result(
    function: String,
    ok: bool,
    detail: String,
    summary: &DiscoverySummary,
    projected: Result<Vec<WitnessRowCost>, String>,
    expectation_refusal: Option<ExpectationRefusal>,
) -> ClaimResult {
    // Per-row identity is load-bearing for the receipt spine: a compute failure OR an
    // incomplete row set must refuse the discovery claim (typed/located), never silently
    // emit a partial receipt as complete (§5 / review 43261 + review 43274).
    match projected {
        Ok(witness_row_costs) => ClaimResult {
            function,
            entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
            ok,
            detail,
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: summary.total_resolve_nanos,
            corpus_eval_nanos: summary.total_measured_nanos,
            corpus_witnesses: summary.total,
            runtime_unit_count: discovery_runtime_unit_count_from_summary(summary.total),
            witness_row_costs,
            expectation_refusal,
            budget_refusal: discovery_budget_refusal(summary),
            host_dependency_refusal: discovery_host_dependency_refusal(summary),
            resolve_realization: None,
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
                entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
                ok: false,
                detail,
                wall_nanos: 0,
                resolve_nanos: 0,
                corpus_resolve_nanos: summary.total_resolve_nanos,
                corpus_eval_nanos: summary.total_measured_nanos,
                corpus_witnesses: summary.total,
                runtime_unit_count: discovery_runtime_unit_count_from_summary(summary.total),
                witness_row_costs: Vec::new(),
                // Same rule as the detail above: a receipt refusal ADDS a cause, it does not
                // erase the one already established. A batch that blew its budget and then
                // failed to project its receipt is still a budget kill, and dropping that
                // here would hand it back to the substring classifier.
                expectation_refusal,
                budget_refusal: discovery_budget_refusal(summary),
                host_dependency_refusal: discovery_host_dependency_refusal(summary),
                resolve_realization: None,
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// Log label for a discovery batch, derived from the plan's modeled profile axes
/// (`execution_mode`, `spawns_host_compiler`) — the same axes `ci_floor_plan` uses
/// to split execution vs bin-witness corpora — not from explicit-entry count.
fn discovery_corpus_kind_label(
    scan_dirs: &[String],
    execution_mode: ExecutionMode,
    spawns_host_compiler: bool,
) -> &'static str {
    if !scan_dirs.is_empty() {
        "discovery-corpus"
    } else if execution_mode == ExecutionMode::Wet && spawns_host_compiler {
        "bin-witness-corpus"
    } else if execution_mode == ExecutionMode::Wet {
        "execution-corpus"
    } else {
        "explicit-corpus"
    }
}

fn run_discovery_batch_node(
    source_roots: Vec<String>,
    execution_authority_source_roots: Vec<String>,
    scan_dirs: Vec<String>,
    explicit_entries: Vec<(String, String)>,
    exclude_substrings: Vec<String>,
    discovery_scope_dirs: Vec<String>,
    _governor: Arc<RealizationConcurrency>,
    execution_mode: ExecutionMode,
    spawns_host_compiler: bool,
    fast_lane_eval_budget_ms: Option<u64>,
    wet_receipt_wall_budget_ms: Option<u64>,
    wet_receipt_interp_eval_budget_ms: Option<u64>,
    // The FUNCTION-grain expected-red roster (`gunbc.explicit_witness_admission`), not a
    // batch-wide polarity flag. Every executed witness is matched against its own row.
    expected_red: Vec<(String, String)>,
    // The strict subset of `expected_red` declaring `ExpectTypedPreVerdictRefusal`. Only these
    // may turn a corpus-level refuse — which carries no per-witness outcome — into agreement.
    pre_verdict_refusal: Vec<(String, String)>,
    scoped_receipt: Option<ScopedReceiptBatch>,
) -> ClaimResult {
    let corpus_kind = discovery_corpus_kind_label(&scan_dirs, execution_mode, spawns_host_compiler);
    set_phase(FloorPhase::Discovery, corpus_kind);
    // Post-discovery projections are executor machinery too.  Scoped batches keep
    // their witness subjects under the narrow `source_roots`, while the authored
    // timing projector and its renderers live in the enclosing walk universe.  Keep
    // that authority available after the options value moves into discovery; using
    // subject roots here makes a fully-green scoped roster refuse while resolving
    // `gunbc.witness_row_cost` and tempts callers to widen the subject envelope.
    let execution_projection_source_roots = execution_authority_source_roots.clone();
    let label = format!(
        "{corpus_kind}[{} root(s)+{} explicit, derived schedule width{}]",
        source_roots.len(),
        explicit_entries.len(),
        if batch_entries_all_in(&explicit_entries, &expected_red) {
            ", expect_red"
        } else {
            ""
        },
    );
    match run_discovery_corpus_with_options(
        &source_roots,
        &scan_dirs,
        &explicit_entries,
        execution_mode,
        DiscoveryWidthPolicy::DerivedSchedule,
        DiscoveryCorpusOptions {
            execution_authority_source_roots,
            explicit_roster_only: false,
            exclude_substrings,
            discovery_scope_dirs,
            fast_lane_eval_budget_ms,
            wet_receipt_wall_budget_ms,
            wet_receipt_interp_eval_budget_ms,
        },
    ) {
        Ok(summary) if summary.failures.is_empty() => {
            if let Some(scoped) = &scoped_receipt {
                if let Err(msg) = append_scoped_witness_receipt_rows(
                    &scoped.batch_id,
                    &scoped.source_roots_digest,
                    &scoped.entries,
                    Some(&summary),
                    None,
                ) {
                    return ClaimResult {
                        function: label,
                        entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
                        ok: false,
                        detail: format!("scoped witness receipt refused: {msg}"),
                        wall_nanos: 0,
                        resolve_nanos: 0,
                        corpus_resolve_nanos: summary.total_resolve_nanos,
                        corpus_eval_nanos: summary.total_measured_nanos,
                        corpus_witnesses: summary.total,
                        runtime_unit_count: discovery_runtime_unit_count_from_summary(
                            summary.total,
                        ),
                        witness_row_costs: Vec::new(),
                        expectation_refusal: None,
                        budget_refusal: discovery_budget_refusal(&summary),
                        host_dependency_refusal: discovery_host_dependency_refusal(&summary),
                        resolve_realization: None,
                    };
                }
            }
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
                "[assembly-split] schedule={:.1}ms probe={:.1}ms graph={:.1}ms symbol_index={:.1}ms pool_fill={:.1}ms symbol_index_merge={:.1}ms variant_base={:.1}ms root_symbol_index={:.1}ms root_variant_base={:.1}ms environment={:.1}ms diagnostics={:.1}ms registry={:.1}ms services={:.1}ms rewire_type_env={:.1}ms rewire_import_str={:.1}ms rewire_func_env={:.1}ms emit_info={:.1}ms other={:.1}ms rewire_total_observation={:.1}ms",
                ms(st.assembly_schedule),
                ms(st.assembly_probe),
                ms(st.assembly_graph),
                ms(st.assembly_symbol_index),
                ms(st.assembly_pool_fill),
                ms(st.assembly_symbol_index_merge),
                ms(st.assembly_variant_base),
                ms(st.assembly_root_symbol_index),
                ms(st.assembly_root_variant_base),
                ms(st.assembly_environment),
                ms(st.assembly_diagnostics),
                ms(st.assembly_registry),
                ms(st.assembly_services),
                ms(st.assembly_rewire_type_env),
                ms(st.assembly_rewire_import_str),
                ms(st.assembly_rewire_func_env),
                ms(st.assembly_emit_info),
                ms(st.reconcile_assembly),
                ms(st.assembly_rewire),
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
            let projected =
                project_witness_cost_receipt(&execution_projection_source_roots, &summary);
            match &projected {
                Ok(rows) => {
                    match render_timing_histogram(
                        &execution_projection_source_roots,
                        &compute_histogram_data(rows),
                    ) {
                        Ok(histogram) => eprintln!("{histogram}"),
                        Err(e) => eprintln!("[histogram] render failed (timings unaffected): {e}"),
                    }
                }
                Err(msg) => eprintln!("{msg}"),
            }
            if let Ok(rows) = &projected {
                emit_slowest_witness_attribution(&execution_projection_source_roots, rows);
            }
            // Every witness green means every expected-RED witness here is a STALE
            // quarantine — its dissolve-on fired. Named at function grain rather than
            // counted, because the remedy is deleting those specific admission rows.
            let tally = classify_witness_expectations_in(
                &summary.witness_outcomes,
                &expected_red,
                &explicit_entries,
            );
            report_absent_expected_red_evidence(&tally);
            match tally.refusal() {
                Some(refusal) => discovery_claim_result(
                    label,
                    false,
                    refusal.detail(),
                    &summary,
                    projected,
                    Some(refusal),
                ),
                None => discovery_claim_result(
                    format!("{label} ({} witnesses)", summary.total),
                    true,
                    String::new(),
                    &summary,
                    projected,
                    None,
                ),
            }
        }
        Ok(summary) => {
            if let Some(scoped) = &scoped_receipt {
                if let Err(msg) = append_scoped_witness_receipt_rows(
                    &scoped.batch_id,
                    &scoped.source_roots_digest,
                    &scoped.entries,
                    Some(&summary),
                    None,
                ) {
                    return ClaimResult {
                        function: label,
                        entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
                        ok: false,
                        detail: format!("scoped witness receipt refused: {msg}"),
                        wall_nanos: 0,
                        resolve_nanos: 0,
                        corpus_resolve_nanos: summary.total_resolve_nanos,
                        corpus_eval_nanos: summary.total_measured_nanos,
                        corpus_witnesses: summary.total,
                        runtime_unit_count: discovery_runtime_unit_count_from_summary(
                            summary.total,
                        ),
                        witness_row_costs: Vec::new(),
                        expectation_refusal: None,
                        budget_refusal: discovery_budget_refusal(&summary),
                        host_dependency_refusal: discovery_host_dependency_refusal(&summary),
                        resolve_realization: None,
                    };
                }
            }
            let projected =
                project_witness_cost_receipt(&execution_projection_source_roots, &summary);
            // Per-witness, never per-batch: a red witness that declared ExpectWitnessRed is
            // AGREEMENT and a green one that declared it is a stale-quarantine refusal, in
            // the same batch, in either order. A mixed batch is ordinary rather than a trap.
            let tally = classify_witness_expectations_in(
                &summary.witness_outcomes,
                &expected_red,
                &explicit_entries,
            );
            report_absent_expected_red_evidence(&tally);
            if !tally.agreements.is_empty() {
                eprintln!(
                    "[expect-red] {} known-red witness(es) still red (agreement — quarantine holds): {}",
                    tally.agreements.len(),
                    render_witness_ids(&tally.agreements)
                );
            }
            match tally.refusal() {
                Some(refusal) => discovery_claim_result(
                    label,
                    false,
                    refusal.detail(),
                    &summary,
                    projected,
                    Some(refusal),
                ),
                // ACCOUNTING, not just emptiness: the batch passes only when every failure
                // in the summary is matched by an agreement. `witness_outcomes` and
                // `failures` are pushed in lockstep today, so the counts agree — but if a
                // failure ever arrives without a per-witness identity, this arm must refuse
                // rather than read "no unexpected failures" as "nothing failed". That would
                // be the empty-observation narrow: an unmatchable failure rendered as the
                // verdict "nothing is wrong".
                None if still_red_batch_passes(&tally, summary.failures.len()) => {
                    discovery_claim_result(
                        format!("{label} (expect_red still-red OK)"),
                        true,
                        String::new(),
                        &summary,
                        projected,
                        None,
                    )
                }
                None => discovery_claim_result(
                    label,
                    false,
                    format!(
                        "{} of {} discovery witness(es) failed ({} agreement(s) held; {} expected-red produced NO verdict{}): {}",
                        summary.failures.len(),
                        summary.total,
                        tally.agreements.len(),
                        tally.expected_red_without_verdict.len(),
                        if tally.expected_red_without_verdict.is_empty() {
                            String::new()
                        } else {
                            format!(" — {}", render_disposition_ids(&tally.expected_red_without_verdict))
                        },
                        summary.failures.join("; ")
                    ),
                    &summary,
                    projected,
                    None,
                ),
            }
        }
        Err(msg) => {
            let scoped_write_error = scoped_receipt.as_ref().and_then(|scoped| {
                append_scoped_witness_receipt_rows(
                    &scoped.batch_id,
                    &scoped.source_roots_digest,
                    &scoped.entries,
                    None,
                    Some(&msg),
                )
                .err()
            });
            // No witness outcomes exist on this path, so per-witness matching has nothing
            // to match. The batch-wide condition survives HERE ONLY, and deliberately in its
            // narrow `all` form: a resolve refuse is agreement only when every entry in the
            // batch declared expected-red, so one ordinary entry keeps the refusal loud.
            if batch_entries_all_expect_pre_verdict_refusal(&explicit_entries, &pre_verdict_refusal)
            {
                // NON-GREEN, and the declaration only classifies WHY (operator ruling,
                // 2026-08-07). This arm previously returned ok:true whenever every entry
                // DECLARED a typed pre-verdict refusal. That made an unobserved fact
                // verdict-bearing: no `ClaimOutcome` exists on this path, `run_claim` has
                // already formatted the typed `InterpError` into a String, so the observed
                // phase and cause cannot be matched against the declared ones — a row
                // declaring `PreVerdictResolve`/`UnboundImportedVariantConstructor` was
                // admitted when some entirely different pre-verdict fault occurred. A
                // declaration may be expressible before typed observation survives the
                // execution boundary; it may not flip the result green until it can be
                // structurally matched. Dissolve-on: EXPECTED-RED-CAUSE-1 — `ClaimOutcome`
                // preserves typed phase and cause, at which point this arm becomes
                // agreement exactly when observed == declared, and refuses on either
                // mismatch.
                eprintln!(
                    "[expect-red] REFUSED: every entry declares a typed pre-verdict refusal, but no typed observation survives this path, so the declared phase/cause CANNOT be matched — declaration classifies, it does not verify: {msg}"
                );
                pre_verdict_unverified_claim_result(&label, explicit_entries.len(), &msg)
            } else {
                let detail = match scoped_write_error {
                    Some(receipt) => format!(
                        "discovery corpus failed: {msg}; scoped witness receipt refused: {receipt}"
                    ),
                    None => format!("discovery corpus failed: {msg}"),
                };
                let runtime_unit_count = runtime_unit_count_unavailable(&detail);
                ClaimResult {
                    function: label,
                    entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
                    ok: false,
                    detail,
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count,
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: None,
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

fn run_pre_walk_execution(
    executor_source_roots: &[String],
    plan_site: &str,
    execution: &PreWalkExecution,
) -> Result<(), String> {
    let PreWalkExecution::TypedClaimSubprocess {
        transport_entry,
        transport_function,
        source_roots,
        claim_entry,
        claim_function,
    } = execution
    else {
        return Ok(());
    };
    let started = Instant::now();
    let memory_before = stage_memory_snapshot();
    eprintln!(
        "claim_executor: pre-walk typed claim subprocess — transport={transport_entry}::{transport_function} claim={claim_entry}::{claim_function}"
    );
    let (graph, indices) = resolve_entry_graph(executor_source_roots, transport_entry).map_err(
        |msg| {
            format!(
                "PRE-WALK-REFUSED plan_site={plan_site} transport={transport_entry}::{transport_function} claim={claim_entry}::{claim_function}: transport resolve failed: {msg}"
            )
        },
    )?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
    let result = run_in_context_with_args(
        &ctx,
        transport_function,
        &[
            (
                Some("source_roots".to_string()),
                str_list_value(source_roots),
            ),
            (
                Some("claim_entry".to_string()),
                str_value(claim_entry.clone()),
            ),
            (
                Some("claim_function".to_string()),
                str_value(claim_function.clone()),
            ),
        ],
        false,
    );
    drop(ctx);
    drop(graph);
    let wall_ms = started.elapsed().as_millis();
    let memory_after = stage_memory_snapshot();
    eprintln!(
        "claim_executor: pre-walk typed claim subprocess receipt — wall_ms={wall_ms} memory_current_before={} memory_current_after={} memory_peak_after={} swap_after={} high_events_after={}",
        receipt_optional_u64(memory_before.current_bytes),
        receipt_optional_u64(memory_after.current_bytes),
        receipt_optional_u64(memory_after.peak_bytes),
        receipt_optional_u64(memory_after.swap_bytes),
        receipt_optional_u64(memory_after.high_events),
    );
    match result {
        Ok(Value::Bool(true)) => Ok(()),
        Ok(Value::Bool(false)) => {
            // The claim channel is a Bool, so the typed cause crosses as the durable
            // refusal wire the capture writes before returning false (the
            // floor-population-budget-refusal.txt pattern; merge_admission_capture
            // merge_admission_capture_refusal_wire_note). Wire-absent is its own
            // reported state, never folded into a generic failure.
            let cause = match merge_admission_wire_read(MERGE_ADMISSION_CAPTURE_REFUSAL_WIRE) {
                Ok(wire) => format!("capture refusal wire: {}", wire.trim()),
                Err(_) => "typed child returned false with no capture-refusal wire (child died before writing its cause, or the wire write itself refused — indistinguishable from here: the Bool claim is the only surviving channel)".to_string(),
            };
            Err(format!(
                "PRE-WALK-REFUSED plan_site={plan_site} transport={transport_entry}::{transport_function} claim={claim_entry}::{claim_function}: {cause}"
            ))
        }
        Ok(other) => Err(format!(
            "PRE-WALK-REFUSED plan_site={plan_site} transport={transport_entry}::{transport_function} claim={claim_entry}::{claim_function}: transport returned {other:?}, expected Bool"
        )),
        Err(msg) => Err(format!(
            "PRE-WALK-REFUSED plan_site={plan_site} transport={transport_entry}::{transport_function} claim={claim_entry}::{claim_function}: transport evaluation failed: {msg}"
        )),
    }
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
            (Some("phase".to_string()), str_value(phase.to_string())),
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
        Value::Str(s) => Some(s.to_string()),
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
    if batch.iter().any(|r| {
        matches!(
            r,
            Runnable::DiscoveryBatch { .. } | Runnable::ScopedWitnessBatch { .. }
        )
    }) {
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
    /// The runtime unit count the clamp used when observed.
    unit_count: u128,
    /// Full availability for the batch clamp receipt (`ClampRefused` when unavailable).
    runtime_units: FloorRuntimeUnitCount,
    /// Flattened results from all units in this batch (order: unit by unit).
    results: Vec<ClaimResult>,
    /// Heartbeat label for this batch — carried so the component receipt names the
    /// component the same way the progress lines did.
    label: String,
    /// The batch's node-frontier selection role, the STRUCTURAL identity the component
    /// receipt keys on (`gunbc.floor_component_receipt` role note): the affected-set
    /// cold control is the `predict_only` component, never "batch 1" — indices shift
    /// when a `gunbc_falsifier_batches` enrollment flag flips.
    /// Wet-profiled batches (bin_witness wet corpus, falsifier wet follow-on, …).
    is_wet: bool,
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
    // Same rule, one class earlier: a stale quarantine is read off the VALUE. It must not
    // fall through to `falsifier_failure_mode`, whose fallback arm is "WitnessRed" — that
    // would make an un-quarantine indistinguishable from a genuine regression both in the
    // component receipt and in the alert's class signature, while the two have opposite
    // remedies (delete an admission row vs. fix the code). It is checked BEFORE the budget
    // refusal because a batch cannot be both: a budget kill is a red, and this arm fires
    // only where an expected-red witness actually returned green.
    if let Some(refusal) = rec
        .results
        .iter()
        .find_map(|r| r.expectation_refusal.as_ref())
    {
        return (
            refusal.mode(),
            refusal.detail().chars().take(600).collect::<String>(),
        );
    }
    if rec.results.iter().any(|r| r.budget_refusal.is_some()) {
        return (
            "BudgetExceeded",
            joined.chars().take(600).collect::<String>(),
        );
    }
    if let Some(refusal) = rec
        .results
        .iter()
        .find_map(|r| r.host_dependency_refusal.as_ref())
    {
        return (
            refusal.mode(),
            refusal.detail().chars().take(600).collect::<String>(),
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
/// Why a planned component carries no verdict in the receipt being written.
///
/// This is the WRITE's own state, not a per-component judgement, which is why it is a
/// parameter of the writer rather than a field on `BatchRecord`: a record exists only for
/// a batch that ran, and the question here is what to say about the ones that did not.
/// `gunbc.floor_component_receipt` `floor_component_run_incomplete_note` owns the
/// distinction; the seed only reports which of the two situations it is in, and it knows
/// that from the call site rather than by inspecting anything.
#[derive(Clone, Copy, PartialEq, Eq)]
enum UnreachedCause {
    /// The plan concluded. A component with no record was deliberately not run because an
    /// earlier one stopped the line under the stop policy.
    StopPolicy,
    /// The run was still in progress when these bytes were written. A component with no
    /// record has an UNKNOWN fate — it may be running right now — so the receipt must not
    /// claim the plan decided anything about it.
    RunIncomplete,
    /// The walk stopped admitting work at its internal soft deadline
    /// (`gunbc.falsifier_workflow` `gunbc_falsifier_soft_deadline_minutes`). A component
    /// with no record was never STARTED because the RUN ran out of time — a deliberate,
    /// located stop at a declared ceiling, which is why this one is Refused rather than
    /// Skipped on the `.dag` side.
    DeadlineReached,
}

impl UnreachedCause {
    /// The `.dag` failure-mode tag. Kept as a method rather than inlined at the two call
    /// sites so the seed spells each tag exactly once; an unknown tag is refused by
    /// `floor_component_failure_mode_of`, so a typo here stops the line rather than
    /// dropping a component.
    fn failure_mode(self) -> &'static str {
        match self {
            UnreachedCause::StopPolicy => "not_reached",
            UnreachedCause::RunIncomplete => "run_incomplete",
            UnreachedCause::DeadlineReached => "deadline_reached",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            UnreachedCause::StopPolicy => {
                "batch not reached — an earlier batch failed under the stop policy"
            }
            UnreachedCause::RunIncomplete => {
                "batch not concluded — checkpoint written while the run was still in \
                 progress; this component's fate is unknown at write time"
            }
            UnreachedCause::DeadlineReached => {
                "batch not admitted — the walk reached its internal soft deadline before \
                 starting this component; the corpus did not fit the lane's time budget"
            }
        }
    }
}

fn write_floor_component_receipt_at(
    base: &std::path::Path,
    source_roots: &[String],
    batch_records: &[BatchRecord],
    batches: &[Vec<Runnable>],
    unreached: UnreachedCause,
) -> bool {
    let total_batches = batches.len();
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
    // The receipt's SUBJECT. It binds the document to the run and tree that produced
    // it so a consumer can refuse a receipt that is not about the run it is reacting
    // to (`gunbc.floor_component_receipt_document` floor_component_receipt_subject_note).
    // The fallbacks mirror run_id's: off a GitHub runner these are not "unknown", they
    // are a LOCAL run, and a local receipt is not addressed to any workflow run — the
    // decoder's subject match then refuses it against any event, which is correct.
    let workflow_name = std::env::var("GITHUB_WORKFLOW").unwrap_or_else(|_| "local".to_string());
    let head_sha = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());

    let mut rows: Vec<Value> = Vec::new();
    for rec in batch_records {
        let (failure_mode, detail) = batch_failure_mode_and_detail(rec);
        match floor_component_row_value(
            &ctx,
            &run_id,
            rec.batch_index as i64 + 1,
            &rec.label,
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
    //
    // They carry their PLANNED identity, not a placeholder. `batch_heartbeat_label` is a pure
    // function of `batches[bi]`, so the roster identity of an unreached component is fully
    // available here; an earlier version discarded it and wrote a literal "not reached". That
    // made the receipt complete by COUNT and anonymous by IDENTITY for exactly the components
    // that did not run -- the shape DESIGN section 5 rules out ("Completeness is an identity
    // join, not a count equality").
    //
    // A second erased field, the per-component selection tag, used to be the load-bearing half
    // of this argument: padding the affected-set cold control's `predict_only` tag as "off"
    // deleted that control from its own receipt. Both the tag and the control are gone with
    // affected-set selection (2026-08-15), so the label is now the whole identity -- which
    // makes carrying it accurately the only thing standing between an unreached component and
    // anonymity.
    for bi in batch_records.len()..total_batches {
        let label = batch_heartbeat_label(&batches[bi]);
        let row = if unreached == UnreachedCause::RunIncomplete {
            floor_component_row_not_concluded_value(
                &ctx,
                &run_id,
                bi as i64 + 1,
                &label,
                unreached.failure_mode(),
                unreached.detail(),
            )
        } else {
            floor_component_row_value(
                &ctx,
                &run_id,
                bi as i64 + 1,
                &label,
                0,
                unreached.failure_mode(),
                unreached.detail(),
                0,
            )
        };
        match row {
            Some(v) => rows.push(v),
            None => return false,
        }
    }

    let Some(doc) = write_floor_component_receipt_document(
        &ctx,
        &workflow_name,
        &run_id,
        &head_sha,
        rows,
        unreached,
        batch_records,
    ) else {
        return false;
    };

    let path = base.join("floor-component-receipt.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // WRITE-THEN-RENAME, because this path is now written repeatedly WHILE the run is
    // subject to a foreign SIGKILL. `std::fs::write` truncates and then fills, so a kill
    // landing between those two leaves a truncated or empty file at the exact path the
    // alert reads — and a half-written receipt is strictly worse than an absent one: the
    // absent case is already reported as a named unknown, while a torn one decodes as a
    // malformed receipt or, worse, as a shorter component list that reads like a real
    // observation. The rename is atomic within the directory, so a reader at any instant
    // sees either the previous complete checkpoint or the new complete one, never a
    // partial write. The temp file is per-process so two executors cannot interleave.
    let tmp = base.join(format!(
        "floor-component-receipt.json.tmp.{}",
        std::process::id()
    ));
    if let Err(e) = std::fs::write(&tmp, doc) {
        eprintln!(
            "claim_executor: floor component receipt REFUSED — write {}: {e}",
            tmp.display()
        );
        return false;
    }
    match std::fs::rename(&tmp, &path) {
        Ok(()) => {
            eprintln!(
                "[receipt] floor component receipt: {} component(s) ({})",
                total_batches,
                path.display()
            );
            true
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            eprintln!(
                "claim_executor: floor component receipt REFUSED — rename {} -> {}: {e}",
                tmp.display(),
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
        (Some("run_id".to_string()), str_value(run_id.to_string())),
        (Some("index".to_string()), Value::Int(index)),
        (Some("label".to_string()), str_value(label.to_string())),
        (Some("witnesses".to_string()), Value::Int(witnesses)),
        (
            Some("failure_mode".to_string()),
            str_value(failure_mode.to_string()),
        ),
        (Some("detail".to_string()), str_value(detail.to_string())),
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
                 absent for batch {index}: failure_mode={failure_mode:?} are outside the .dag vocabulary"
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

fn floor_component_row_not_concluded_value(
    ctx: &InterpContext,
    run_id: &str,
    index: i64,
    label: &str,
    failure_mode: &str,
    detail: &str,
) -> Option<Value> {
    let constructor = "floor_component_row_not_concluded";
    let args: Vec<(Option<String>, Value)> = vec![
        (Some("run_id".to_string()), str_value(run_id.to_string())),
        (Some("index".to_string()), Value::Int(index)),
        (Some("label".to_string()), str_value(label.to_string())),
        (
            Some("failure_mode".to_string()),
            str_value(failure_mode.to_string()),
        ),
        (Some("detail".to_string()), str_value(detail.to_string())),
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
                 absent for batch {index}: failure_mode={failure_mode:?} are outside the .dag vocabulary"
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
            eprintln!("claim_executor: floor component receipt REFUSED — {constructor} eval: {e}");
            None
        }
    }
}

fn write_floor_component_receipt_document(
    ctx: &InterpContext,
    workflow_name: &str,
    run_id: &str,
    head_sha: &str,
    rows: Vec<Value>,
    unreached: UnreachedCause,
    batch_records: &[BatchRecord],
) -> Option<String> {
    let rows_list = Value::List(Rc::new(rows.into()));
    if unreached == UnreachedCause::RunIncomplete {
        let concluded_count = batch_records.len() as i64;
        let pending_from_index = (batch_records.len() + 1) as i64;
        let args: Vec<(Option<String>, Value)> = vec![
            (
                Some("workflow_name".to_string()),
                str_value(workflow_name.to_string()),
            ),
            (Some("run_id".to_string()), str_value(run_id.to_string())),
            (
                Some("head_sha".to_string()),
                str_value(head_sha.to_string()),
            ),
            (Some("rows".to_string()), rows_list),
            (
                Some("run_terminal_cause".to_string()),
                str_value(unreached.failure_mode().to_string()),
            ),
            (
                Some("concluded_count".to_string()),
                Value::Int(concluded_count),
            ),
            (
                Some("pending_from_index".to_string()),
                Value::Int(pending_from_index),
            ),
        ];
        let constructor = "floor_component_receipt_document_incomplete";
        match run_in_context_with_args(ctx, constructor, &args, false) {
            Ok(Value::Str(s)) => Some(s.to_string()),
            Ok(other) => {
                eprintln!(
                    "claim_executor: floor component receipt REFUSED — \
                     {constructor} returned {other:?}, not Str"
                );
                None
            }
            Err(e) => {
                eprintln!(
                    "claim_executor: floor component receipt REFUSED — {constructor} eval: {e}"
                );
                None
            }
        }
    } else {
        match run_in_context_with_args(
            ctx,
            "floor_component_receipt_document",
            &[
                (
                    Some("workflow_name".to_string()),
                    str_value(workflow_name.to_string()),
                ),
                (Some("run_id".to_string()), str_value(run_id.to_string())),
                (
                    Some("head_sha".to_string()),
                    str_value(head_sha.to_string()),
                ),
                (Some("rows".to_string()), rows_list),
            ],
            false,
        ) {
            Ok(Value::Str(s)) => Some(s.to_string()),
            Ok(other) => {
                eprintln!(
                    "claim_executor: floor component receipt REFUSED — \
                     floor_component_receipt_document returned {other:?}, not Str"
                );
                None
            }
            Err(e) => {
                eprintln!(
                    "claim_executor: floor component receipt REFUSED — \
                     floor_component_receipt_document eval: {e}"
                );
                None
            }
        }
    }
}

fn write_resolve_receipt(
    source_roots: &[String],
    batch_records: &[BatchRecord],
    floor_finalization: Option<&FloorFinalization>,
) -> bool {
    write_resolve_receipt_at(
        std::path::Path::new("target"),
        source_roots,
        batch_records,
        floor_finalization,
    )
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
        match (&rec.runtime_units, rec.clamp_ms) {
            (FloorRuntimeUnitCount::Unavailable { .. }, _) => {
                body.push_str(&format!("batch_{n}_units=unavailable\n"));
                body.push_str(&format!("batch_{n}_verdict=ClampRefused\n"));
            }
            (FloorRuntimeUnitCount::Observed { .. }, Some(clamp_ms)) => {
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
            (FloorRuntimeUnitCount::Observed { .. }, None) => {
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

fn write_gate_warm_cost_receipt(batch_records: &[BatchRecord]) -> bool {
    write_gate_warm_cost_receipt_at(std::path::Path::new("target"), batch_records)
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
fn write_gate_warm_cost_receipt_at(base: &std::path::Path, batch_records: &[BatchRecord]) -> bool {
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

fn batch_is_wet(batch: &[Runnable]) -> bool {
    batch.iter().any(|runnable| match runnable {
        Runnable::DiscoveryBatch { execution_mode, .. } => *execution_mode == ExecutionMode::Wet,
        Runnable::SingleClaim { profile, .. } => profile.execution_mode == ExecutionMode::Wet,
        Runnable::ScopedWitnessBatch { profile, .. } => {
            profile.execution_mode == ExecutionMode::Wet
        }
    })
}

fn wet_witness_row_outcome_label(outcome_variant: &str) -> &'static str {
    match outcome_variant {
        "Done" => "passed",
        "selection-skipped" => "selection-skipped",
        _ => "failed",
    }
}

fn write_floor_wet_witness_row_outcome_receipt(batch_records: &[BatchRecord]) -> bool {
    write_floor_wet_witness_row_outcome_receipt_at(Path::new("target"), batch_records)
}

fn write_floor_wet_witness_row_outcome_receipt_at(
    base: &Path,
    batch_records: &[BatchRecord],
) -> bool {
    let mut body = String::from("batch\tentry\tfunction\toutcome\tdetail\n");
    let mut row_count = 0usize;
    for rec in batch_records {
        if !rec.is_wet {
            continue;
        }
        let batch = rec.batch_index + 1;
        for result in &rec.results {
            for row in &result.witness_row_costs {
                let outcome = wet_witness_row_outcome_label(&row.outcome);
                let detail = scoped_wire_text(&row.detail);
                body.push_str(&format!(
                    "{}\n",
                    [
                        batch.to_string(),
                        row.entry.clone(),
                        row.function.clone(),
                        outcome.to_string(),
                        detail,
                    ]
                    .join("\t")
                ));
                row_count += 1;
            }
        }
    }
    let path = base.join("floor-wet-witness-row-outcome-receipt.tsv");
    if let Err(e) = std::fs::create_dir_all(base).and_then(|_| std::fs::write(&path, &body)) {
        eprintln!(
            "claim_executor: failed to write wet witness row-outcome receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] floor wet witness row-outcome: {row_count} row(s) (TSV receipt: {})",
        path.display()
    );
    trace_floor_phase(
        "wet-witness-row-outcome-receipt",
        "completed",
        &format!("row_count={row_count} path={}", path.display()),
    );
    true
}

/// Required `host_class` on every signed basis row (`witness_row_cost_basis_host_class_note`).
const WITNESS_ROW_COST_BASIS_HOST_CLASS: &str = "srv_fleet_arm64";

/// The `.dag` constructor for each `std.observation.ClockBasis` arm, keyed by the spelling the
/// basis file uses. The seed names no clock of its own: it maps the cell to a constructor and
/// calls it, so a clock that this repository does not model has no path into a comparison.
fn clock_basis_constructor_for(cell: &str) -> Option<&'static str> {
    match cell {
        "wall" => Some("clock_basis_wall"),
        "cpu" => Some("clock_basis_cpu"),
        _ => None,
    }
}

#[derive(Debug)]
struct WitnessRowCostBasisRow {
    eval_ms_basis: u128,
    run_ref: String,
    /// Which clock `eval_ms_basis` was read from, as the `.dag` constructor name for it.
    ///
    /// Carried per row rather than assumed for the file, because the file's rows are seeded
    /// over time from whatever the producer of the day recorded — and a basis whose clock is
    /// assumed is exactly the state the 2026-08-05 ruling ends. A comparison against an
    /// observation on a different clock refuses (`BasisClockMismatch`) rather than answering.
    clock_constructor: &'static str,
}

/// Parse one TSV body line from `witness_row_cost_basis.tsv`.
/// Returns `Ok(None)` for blank/comment lines; `Err` for malformed, wrong-host-class, or
/// unknown-clock rows (caller must not insert them — a wrong host class would poison the 2×
/// comparator, and an unmodelled clock cannot be compared at all).
fn parse_witness_row_cost_basis_line(
    line: &str,
) -> Result<Option<((String, String), WitnessRowCostBasisRow)>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 6 {
        return Err(format!(
            "malformed witness-row-cost basis line (need 6 cols: entry function eval_ms_basis run_ref host_class clock): {line}"
        ));
    }
    let host_class = parts[4];
    if host_class != WITNESS_ROW_COST_BASIS_HOST_CLASS {
        return Err(format!(
            "witness-row-cost basis row host_class={host_class:?} refused (required {WITNESS_ROW_COST_BASIS_HOST_CLASS}; wrong host class poisons the 2× comparator): {line}"
        ));
    }
    // A row that does not say which clock its figure came from is REFUSED, not defaulted.
    // Defaulting to wall would be right for every row in the file today and wrong the first
    // time someone seeds one from a CPU receipt — and it would be wrong silently, which is
    // the whole failure this column exists to prevent.
    let Some(clock_constructor) = clock_basis_constructor_for(parts[5]) else {
        return Err(format!(
            "witness-row-cost basis row clock={:?} refused (known clocks: wall, cpu; a basis whose clock is unknown cannot be compared): {line}",
            parts[5]
        ));
    };
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
            clock_constructor,
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
/// The clock the drift wire's observed figure is read from.
///
/// `witness_row_costs.2` is the row's `eval_wall_ms` — wall, and named so since #7820 — which
/// is also the clock the witness threshold is stated on (operator ruling 2026-08-05). It is
/// passed to the comparator rather than left implicit, which is the entire point: the
/// comparison now asserts that both sides are the same clock instead of both happening to be.
const WITNESS_ROW_COST_OBSERVED_CLOCK: &str = "clock_basis_wall";

/// Ask the authored comparator for a verdict and return the ARM IT NAMED.
///
/// The seed used to call `witness_row_cost_exceeds_basis` — the bare ratio predicate — and
/// then rebuild `DriftExceeded` / `WithinBasis` / `BasisAbsent` as Rust string literals. That
/// was a second representation of `WitnessRowCostVerdict`, and it meant the cross-clock wall
/// the carrier grew could not reach the cadence receipt at all: the clock never crossed the
/// seam, so a CPU figure against a wall basis would still have answered confidently.
///
/// Now the arm's own name is what gets rendered, so a new arm reaches the receipt without a
/// Rust edit and `BasisClockMismatch` fires here exactly as it does in the witness.
fn witness_row_cost_verdict_via_authority(
    ctx: &InterpContext,
    observed_ms: u128,
    basis: Option<&WitnessRowCostBasisRow>,
) -> Result<String, String> {
    let observed = millisecond_value(ctx, observed_ms)?;
    let observed_clock = run_in_context_with_args(ctx, WITNESS_ROW_COST_OBSERVED_CLOCK, &[], false)
        .map_err(|e| format!("{WITNESS_ROW_COST_OBSERVED_CLOCK}: {e}"))?;
    let (function, args) = match basis {
        None => (
            "witness_row_cost_seed_verdict_undated",
            vec![
                (Some("observed".to_string()), observed),
                (Some("observed_clock".to_string()), observed_clock),
            ],
        ),
        Some(b) => {
            let basis_clock = run_in_context_with_args(ctx, b.clock_constructor, &[], false)
                .map_err(|e| format!("{}: {e}", b.clock_constructor))?;
            let basis_eval = millisecond_value(ctx, b.eval_ms_basis)?;
            (
                "witness_row_cost_seed_verdict_dated",
                vec![
                    (Some("observed".to_string()), observed),
                    (Some("observed_clock".to_string()), observed_clock),
                    (Some("basis_clock".to_string()), basis_clock),
                    (Some("basis_eval".to_string()), basis_eval),
                    (Some("run_ref".to_string()), str_value(b.run_ref.clone())),
                ],
            )
        }
    };
    match run_in_context_with_args(ctx, function, &args, false) {
        Ok(Value::Variant { variant_name, .. }) => Ok(ctx.resolve(variant_name).to_string()),
        Ok(other) => Err(format!(
            "{function} returned {other}, expected a WitnessRowCostVerdict (fail-closed)"
        )),
        Err(e) => Err(format!("{function}: {e}")),
    }
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

/// Coordinator-spawned floor workers inherit `GUNBC_FLOOR_WALK_ATTEMPT_ID` so the scoped
/// consumer reads the ordinary producer's attempt-scoped snapshot directory.
fn floor_walk_attempt_id() -> Result<String, String> {
    if let Ok(id) = std::env::var("GUNBC_FLOOR_WALK_ATTEMPT_ID") {
        if !id.trim().is_empty() {
            return if walk_attempt_id_segment_is_safe(&id) {
                Ok(id)
            } else {
                Err(format!(
                    "GUNBC_FLOOR_WALK_ATTEMPT_ID={id:?} is not a safe path segment (std.types path_segment_is_safe: non-empty, not `.`/`..`, no `/` `\\` CR LF NUL)"
                ))
            };
        }
    }
    observe_walk_attempt_id()
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
    plan_site: &str,
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
    // The plan site completes the subject location: attempt says WHICH RUN, plan site says
    // WHICH PLAN within it, entry+function on each row say WHICH DECLARATION.
    body.push_str(&format!("plan_site={plan_site}\n"));
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
    body.push_str(&format!(
        "memory_current_before_bytes={}\n",
        receipt_optional_u64(run.memory_before.current_bytes)
    ));
    body.push_str(&format!(
        "memory_current_after_bytes={}\n",
        receipt_optional_u64(run.memory_after.current_bytes)
    ));
    body.push_str(&format!(
        "memory_peak_before_bytes={}\n",
        receipt_optional_u64(run.memory_before.peak_bytes)
    ));
    body.push_str(&format!(
        "memory_peak_after_bytes={}\n",
        receipt_optional_u64(run.memory_after.peak_bytes)
    ));
    body.push_str(&format!(
        "memory_swap_before_bytes={}\n",
        receipt_optional_u64(run.memory_before.swap_bytes)
    ));
    body.push_str(&format!(
        "memory_swap_after_bytes={}\n",
        receipt_optional_u64(run.memory_after.swap_bytes)
    ));
    body.push_str(&format!(
        "memory_high_events_before={}\n",
        receipt_optional_u64(run.memory_before.high_events)
    ));
    body.push_str(&format!(
        "memory_high_events_after={}\n",
        receipt_optional_u64(run.memory_after.high_events)
    ));
    // Recorded rather than omitted so a future DECLARED stage clamp shows up as a value
    // change here instead of a new field a reader has to notice.
    body.push_str(&format!(
        "clamp_ms={}\n",
        match run.clamp_ms {
            Some(ms) => ms.to_string(),
            None => "none".to_string(),
        }
    ));
    // ENTRY BEFORE FUNCTION: the pair is the declaration identity, and the entry is the
    // discriminating half. A row keyed on function alone cannot tell two modules that
    // lawfully spell a claim the same way apart (review 2026-07-31).
    for r in &run.results {
        body.push_str(&format!(
            "claim\t{}\t{}\t{}\t{}\n",
            r.entry,
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
    attempt_id: &str,
    plan_site: &str,
) -> bool {
    let mut body = String::new();
    // SAME IDENTITY DISCIPLINE AS THE PER-STAGE RECEIPTS, and it was missing here while
    // the PR describing this work claimed receipt identity was closed (review
    // 2026-07-31). The aggregate is part of the same evidence population — it answers
    // "how much of the declared sequence ran", and that answer is worthless unless it is
    // attributable to an attempt. A reused worktree would otherwise retain a prior
    // attempt's aggregate when the current floor fails before stages ever start.
    body.push_str(&format!("attempt_id={attempt_id}\n"));
    body.push_str(&format!("plan_site={plan_site}\n"));
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
    let base = std::path::Path::new("target").join(format!("floor-attempt-{attempt_id}"));
    let base = base.as_path();
    let path = base.join("on-success-receipt.txt");
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

/// Receipt line derived from transported obligations joined to observed realizations.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolveObligationLine {
    identity: String,
    disposition: &'static str,
    resolve_nanos: u128,
    provider_id: Option<String>,
    computation_identity: Option<String>,
    entry: String,
    function: String,
}

fn find_claim_result<'a>(
    batch_records: &'a [BatchRecord],
    entry: &str,
    function: &str,
) -> Option<&'a ClaimResult> {
    for rec in batch_records {
        for result in &rec.results {
            if result.entry == entry && result.function == function {
                return Some(result);
            }
        }
    }
    None
}

fn observation_to_obligation_line(
    obl: &TransportedObligation,
    result: &ClaimResult,
) -> Result<ResolveObligationLine, String> {
    let observation = result.resolve_realization.as_ref().ok_or_else(|| {
        format!(
            "floor resolve obligation missing realization observation: {} ({}::{})",
            obl.identity, obl.entry, obl.function
        )
    })?;
    match observation {
        ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos } => {
            if *resolve_nanos == 0 {
                return Err(format!(
                    "floor resolve cold disposition without resolve receipt: {}",
                    obl.identity
                ));
            }
            Ok(ResolveObligationLine {
                identity: obl.identity.clone(),
                disposition: RESOLVE_REALIZATION_DISPOSITION_COLD,
                resolve_nanos: *resolve_nanos,
                provider_id: None,
                computation_identity: None,
                entry: obl.entry.clone(),
                function: obl.function.clone(),
            })
        }
        ResolveRealizationObservation::SatisfiedFromSharedPool {
            computation_identity,
            provider_id,
        } => {
            if provider_id.is_empty() {
                return Err(format!(
                    "floor resolve warm disposition without provider id: {}",
                    obl.identity
                ));
            }
            Ok(ResolveObligationLine {
                identity: obl.identity.clone(),
                disposition: RESOLVE_REALIZATION_DISPOSITION_WARM,
                resolve_nanos: result.resolve_nanos,
                provider_id: Some(provider_id.clone()),
                computation_identity: Some(computation_identity.clone()),
                entry: obl.entry.clone(),
                function: obl.function.clone(),
            })
        }
    }
}

fn derive_resolve_obligation_receipts(
    fin: &FloorFinalization,
    batch_records: &[BatchRecord],
) -> Result<Vec<ResolveObligationLine>, String> {
    fin.expected_obligations
        .iter()
        .map(|obl| {
            let result =
                find_claim_result(batch_records, &obl.entry, &obl.function).ok_or_else(|| {
                    format!(
                        "floor resolve obligation missing: {} ({}::{})",
                        obl.identity, obl.entry, obl.function
                    )
                })?;
            observation_to_obligation_line(obl, result)
        })
        .collect()
}

fn obligation_entries_with_realization(
    fin: &FloorFinalization,
    batch_records: &[BatchRecord],
) -> std::collections::HashSet<String> {
    let subjects = obligation_subject_set(Some(fin)).expect("fin provided");
    let mut entries = std::collections::HashSet::new();
    for rec in batch_records {
        for result in &rec.results {
            if result.resolve_realization.is_none() {
                continue;
            }
            if is_rostered_obligation_subject(&subjects, &result.entry, &result.function) {
                entries.insert(result.entry.clone());
            }
        }
    }
    entries
}

fn unattributed_physical_resolve_subjects(
    fin: &FloorFinalization,
    batch_records: &[BatchRecord],
) -> Vec<(String, String)> {
    let subjects = obligation_subject_set(Some(fin)).expect("fin provided");
    let satisfied_entries = obligation_entries_with_realization(fin, batch_records);
    let mut seen = std::collections::HashSet::new();
    let mut surplus = Vec::new();
    for rec in batch_records {
        for result in &rec.results {
            if result.resolve_nanos == 0 {
                continue;
            }
            if is_rostered_obligation_subject(&subjects, &result.entry, &result.function) {
                continue;
            }
            if satisfied_entries.contains(&result.entry) {
                continue;
            }
            let key = format!("{}::{}", result.entry, result.function);
            if seen.insert(key) {
                surplus.push((result.entry.clone(), result.function.clone()));
            }
        }
    }
    surplus
}

fn count_unattributed_physical_resolves(
    fin: &FloorFinalization,
    batch_records: &[BatchRecord],
) -> u64 {
    unattributed_physical_resolve_subjects(fin, batch_records).len() as u64
}

fn obligation_entry_duplicate_cold_resolves(
    fin: &FloorFinalization,
    batch_records: &[BatchRecord],
) -> Vec<(String, u64)> {
    let obligation_entries: std::collections::HashSet<String> = fin
        .expected_obligations
        .iter()
        .map(|o| o.entry.clone())
        .collect();
    let mut cold_per_entry: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    for rec in batch_records {
        for result in &rec.results {
            if result.resolve_nanos == 0 {
                continue;
            }
            if !obligation_entries.contains(&result.entry) {
                continue;
            }
            *cold_per_entry.entry(result.entry.clone()).or_insert(0) += 1;
        }
    }
    obligation_entries
        .into_iter()
        .filter_map(|entry| {
            let count = cold_per_entry.get(&entry).copied().unwrap_or(0);
            if count > 1 {
                Some((entry, count))
            } else {
                None
            }
        })
        .collect()
}

fn append_resolve_obligation_receipt_body(
    body: &mut String,
    fin: &FloorFinalization,
    batch_records: &[BatchRecord],
) -> Result<(), String> {
    let obligations = derive_resolve_obligation_receipts(fin, batch_records)?;
    let cold_resolves_total = obligations
        .iter()
        .filter(|o| o.disposition == RESOLVE_REALIZATION_DISPOSITION_COLD)
        .count();
    let unattributed_physical_resolves = count_unattributed_physical_resolves(fin, batch_records);
    body.push_str(&format!("obligations_total={}\n", obligations.len()));
    body.push_str(&format!("cold_resolves_total={cold_resolves_total}\n"));
    body.push_str(&format!(
        "unattributed_physical_resolves={unattributed_physical_resolves}\n"
    ));
    for line in &obligations {
        let provider = line.provider_id.as_deref().unwrap_or("-");
        body.push_str(&format!(
            "obligation={} disposition={} resolve_nanos={} provider_id={} entry={} function={}\n",
            line.identity,
            line.disposition,
            line.resolve_nanos,
            provider,
            line.entry,
            line.function
        ));
    }
    Ok(())
}

fn write_resolve_receipt_at(
    base: &std::path::Path,
    _source_roots: &[String],
    batch_records: &[BatchRecord],
    floor_finalization: Option<&FloorFinalization>,
) -> bool {
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
    let mut body = format!(
        "resolves_total={resolves_total}\nresolve_ms_total={resolve_ms_total}\ndiscovery_corpus_resolve_ms={discovery_corpus_resolve_ms}\ndiscovery_corpus_eval_ms={discovery_corpus_eval_ms}\n{discovery_phases}"
    );
    if let Some(fin) = floor_finalization {
        if let Err(msg) = append_resolve_obligation_receipt_body(&mut body, fin, batch_records) {
            eprintln!("claim_executor: resolve obligation receipt refused: {msg}");
            return false;
        }
    }
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

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority:
/// `gunbc.ci_materialization.ci_floor_on_success_materialization_receipt_claim_executor_seed_note`):
/// harvests the on-success-stage eval population into
/// `target/floor-attempt-<attempt_id>/floor-on-success-materialization-receipt.txt`
/// after `stage_memo` drops; ordinary-floor harvest stays in `write_materialization_receipt`. Receipt shape and
/// path are modeled in `.dag`; the fs write + process-accumulator boundary are
/// executor realization until claim_executor self-emits.
fn materialization_receipt_body(
    t: &v1_compiler::v1_interpreter::EvalRecomputeTotals,
    attempt_id: Option<&str>,
    plan_site: Option<&str>,
) -> String {
    let mut body = String::new();
    if let Some(attempt_id) = attempt_id {
        body.push_str(&format!("attempt_id={attempt_id}\n"));
    }
    if let Some(plan_site) = plan_site {
        body.push_str(&format!("plan_site={plan_site}\n"));
    }
    body.push_str(&format!(
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
    ));
    body
}

/// The materialization demand receipt at the eval-frame grain: process-wide
/// ledger totals accumulated by every InterpContext on Drop (threads included).
/// Determinism, as measured (2026-07-10, 5 receipts): unkeyed_calls is
/// corpus-deterministic (identical across schedules, machines, and debug/release);
/// keyed/distinct/duplicated jitter a few counts because they sum PER-CTX numbers
/// and witness→ctx grouping is a thread-pool accident — so counts disclose, they
/// do not pin, until the frame grain is structural. wasted_ms lines are
/// observational and must never gate. The derived ci.yml gate fails closed on a
/// missing/malformed file or zeroed keyed_calls (a floor that evaluated nothing
/// is a lie, so disabling the trace cannot silently green the gate). Returns
/// false on a write error — the walk fails closed here, not only at the
/// downstream missing-file gate.
fn write_materialization_receipt_at(
    path: &std::path::Path,
    population: &str,
    attempt_id: Option<&str>,
    plan_site: Option<&str>,
) -> bool {
    let t = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
    let body = materialization_receipt_body(&t, attempt_id, plan_site);
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    if let Err(e) = std::fs::create_dir_all(parent).and_then(|_| std::fs::write(path, &body)) {
        eprintln!(
            "claim_executor: failed to write {population} materialization receipt {}: {e} — walk fails closed here",
            path.display()
        );
        return false;
    }
    eprintln!(
        "[receipt] {population} materialization: keyed_calls={} unkeyed_calls={} duplicated_keys={} (single_site={} multi_site={}) wasted_ms={} memo_hits={} memo_misses={} (receipt: {})",
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

/// Ordinary-floor materialization disclosure (gates via `FloorFinalization`).
fn write_materialization_receipt() -> bool {
    write_materialization_receipt_at(
        std::path::Path::new("target/floor-materialization-receipt.txt"),
        "floor",
        None,
        None,
    )
}

/// On-success-stage materialization disclosure — a SEPARATE population from the
/// ordinary floor receipt, harvested after `stage_memo` drops. Attempt-scoped
/// like the other on-success receipts so a reused worktree cannot retain a
/// prior attempt's success population when the current floor fails before stages.
fn write_on_success_materialization_receipt(attempt_id: &str, plan_site: &str) -> bool {
    let path = std::path::Path::new("target")
        .join(format!("floor-attempt-{attempt_id}"))
        .join("floor-on-success-materialization-receipt.txt");
    write_materialization_receipt_at(&path, "on-success floor", Some(attempt_id), Some(plan_site))
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
        BatchUnit::Discovery { .. } | BatchUnit::ScopedDiscovery { .. } => UnitLane::MainThread,
        _ => UnitLane::Spawned,
    }
}

/// Population-semantic refinement of the shared lane decision. Green-only claims that
/// would spawn instead stay on the executor's main thread, where the process-shared
/// module index is already warm. The claim still runs through `run_batch_unit`, so this
/// changes placement only — not governor admission, grouping, or verdict semantics.
fn population_unit_lane(
    population: StagePopulation,
    unit: &BatchUnit,
    memo: &std::collections::HashMap<(String, ExecutionMode), InterpContext>,
    memo_path_entries: &std::collections::HashSet<(String, ExecutionMode)>,
) -> UnitLane {
    match (population, batch_unit_lane(unit, memo, memo_path_entries)) {
        (StagePopulation::OnSuccessStage, UnitLane::Spawned) => UnitLane::MainThread,
        (_, lane) => lane,
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
                Runnable::ScopedWitnessBatch { .. } => refusals.push(format!(
                    "on-success stage {} contains a RunnableScopedWitnessBatch — scoped batches belong to the ordinary execution population",
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
/// Obligation subjects never executed on a completed walk.
fn unexecuted_transport_obligations<'a>(
    fin: &'a FloorFinalization,
    batch_records: &[BatchRecord],
) -> Vec<&'a TransportedObligation> {
    fin.expected_obligations
        .iter()
        .filter(|obl| find_claim_result(batch_records, &obl.entry, &obl.function).is_none())
        .collect()
}

fn validate_floor_finalization(
    fin: &FloorFinalization,
    _plan_site: &str,
    batch_records: &[BatchRecord],
    walk_truncated: bool,
) -> Vec<String> {
    let mut refusals = Vec::new();
    // Law 1 — semantic resolve OBLIGATIONS (gunbc.ci_materialization / 0B):
    // transported obligation subjects == observed realization observations.
    // Warm shared-pool satisfaction requires provider + receipt recorded at the reuse
    // site; unattributed physical cold resolves refuse. resolve_nanos is cost evidence.
    let unattributed = unattributed_physical_resolve_subjects(fin, batch_records);
    if !unattributed.is_empty() {
        let listing = unattributed
            .iter()
            .map(|(entry, function)| format!("{entry}::{function}"))
            .collect::<Vec<_>>()
            .join("; ");
        refusals.push(format!(
            "floor resolve unattributed physical resolve(s): {} subject(s) with \
             resolve_nanos > 0 outside transported expected_resolve_obligations — {listing}",
            unattributed.len(),
        ));
    }
    for (entry, count) in obligation_entry_duplicate_cold_resolves(fin, batch_records) {
        refusals.push(format!(
            "floor resolve duplicate cold on obligation entry {entry}: {count} physical \
             resolve(s) with resolve_nanos > 0 (expected at most 1 per rostered entry)"
        ));
    }
    let unexecuted = unexecuted_transport_obligations(fin, batch_records);
    if !unexecuted.is_empty() {
        let listing = unexecuted
            .iter()
            .map(|obl| format!("{} ({}::{})", obl.identity, obl.entry, obl.function))
            .collect::<Vec<_>>()
            .join("; ");
        let cause = if walk_truncated {
            "walk stopped before dependent batches"
        } else {
            "obligation subject(s) never executed on a completed walk"
        };
        refusals.push(format!(
            "floor resolve obligations not fully scheduled: {} obligation subject(s) never ran \
             ({cause}); count law unevaluable — {listing}",
            unexecuted.len(),
        ));
    } else {
        match derive_resolve_obligation_receipts(fin, batch_records) {
            Ok(obligations) => {
                let mut seen = std::collections::HashSet::new();
                for line in &obligations {
                    if !seen.insert(line.identity.clone()) {
                        refusals.push(format!(
                            "floor resolve obligation duplicate identity: {}",
                            line.identity
                        ));
                    }
                }
            }
            Err(msg) => refusals.push(msg),
        }
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
/// SCAFFOLD (§7 seed-retained HAND-RUST — authority: `std.realization_schedule`
/// `walk_plan_run_stage_claim_executor_seed_deferral`). This function, `join_units`,
/// `run_stage`, `batch_unit_lane`, the stage receipt writers, and the walk-attempt
/// observation are all covered by that row: the executor is the seed that runs before any
/// `.dag` walk exists, so the code deciding how a walk executes cannot itself be a walk.
/// dissolve-on: executor scheduling expressed as a `.dag` walk over `WalkPlan` (lane
/// selection, admission, receipt emission as modeled effects), gated behind the
/// witness-realization lane.
///
/// THE CONTRACT, stated at the strength `walk_plan_note` actually carries: DISTINCT
/// SPAWNED resolve groups MAY overlap, subject to governor admission. Same-entry groups,
/// memo-lane units, main-thread units, and width-constrained spawned units may execute
/// serially, and NO SIBLING ORDER IS GUARANTEED. An earlier version of this comment said
/// members "run concurrently" — the stronger promise the carrier retracted (review
/// 2026-07-31), and one the executor would violate every time grouping, memo placement,
/// or a width-1 governor legitimately withdrew overlap.
///
/// The absence of a guaranteed order is what stage occupants actually depend on, and it
/// is why anything sequential must be one claim whose body sequences its steps, or two
/// singleton stages. While the spawn and the join were inlined in the batch loop, even
/// that weaker property was checkable only by reading the code, and the first attempt at
/// success stages promised the stronger one while executing serially. Split out, the two
/// halves each get a latch control: spawned peers really can overlap, and the join really
/// does wait for all of them.
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

/// Which of the walk's two populations a stage belongs to. Besides labels, this selects
/// the one population-semantic placement difference: on-success units that would be
/// spawned execute through `run_batch_unit` on the main thread so they consume its warm
/// thread-local index. Ordinary spawned units stay unchanged. Ordering and failure
/// policy continue to live in the callers.
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
    /// Aggregated runtime unit observation for the clamp (authority
    /// `gunbc_ci_floor_batch_runtime_unit_count_note`).
    runtime_units: FloorRuntimeUnitCount,
    /// Observed unit sum when available; zero when unavailable (receipt uses `runtime_units`).
    unit_count: u128,
    /// The derived clamp actually computed, or None when the plan declares no clamp
    /// params for this population.
    clamp_ms: Option<u128>,
    /// A worker thread panicked — an infra fault, distinct from a claim verdict.
    thread_panicked: bool,
    /// The stage exceeded its derived clamp. Already printed as a typed refusal.
    over_budget: bool,
    memory_before: StageMemorySnapshot,
    memory_after: StageMemorySnapshot,
}

#[derive(Clone, Copy, Default)]
struct StageMemorySnapshot {
    current_bytes: Option<u64>,
    peak_bytes: Option<u64>,
    swap_bytes: Option<u64>,
    high_events: Option<u64>,
}

fn stage_memory_snapshot() -> StageMemorySnapshot {
    let Some(dir) = binding_high_cgroup_dir()
        .or_else(binding_cap_cgroup_dir)
        .or_else(leaf_cgroup_dir)
    else {
        return StageMemorySnapshot::default();
    };
    let high_events = read_cgroup_raw(&dir, "memory.events").and_then(|events| {
        events.lines().find_map(|line| {
            let mut fields = line.split_whitespace();
            match (fields.next(), fields.next()) {
                (Some("high"), Some(value)) => value.parse::<u64>().ok(),
                _ => None,
            }
        })
    });
    StageMemorySnapshot {
        current_bytes: read_cgroup_u64(&dir, "memory.current"),
        peak_bytes: read_cgroup_u64(&dir, "memory.peak"),
        swap_bytes: read_cgroup_u64(&dir, "memory.swap.current"),
        high_events,
    }
}

fn receipt_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "unreadable".to_string(), |n| n.to_string())
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
    governor: &Arc<RealizationConcurrency>,
    fast_lane_eval_budget_ms: Option<u64>,
    falsifier_self_host_wet_budgets: &FalsifierSelfHostWetBudgets,
    clamp_params: Option<ResolvedFloorBatchClamp>,
    budget_tighten_ms: Option<u128>,
    obligation_subjects: Option<&ObligationSubjectSet>,
) -> StageRun {
    let units = group_batch_units(stage);
    // Arm the observation heartbeat feed at stage-enter: discovery leaves entry_total
    // pending (filled when the roster's entry-group count is known); SingleClaim arms
    // immediately with the claim count. Never a fabricated 0-of-0.
    let label = batch_heartbeat_label(stage);
    let entry_total = if stage.iter().any(|r| {
        matches!(
            r,
            Runnable::DiscoveryBatch { .. } | Runnable::ScopedWitnessBatch { .. }
        )
    }) {
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
    let memory_before = stage_memory_snapshot();
    let stage_start = Instant::now();
    // Partition units into lanes (decision table: `batch_unit_lane`). Ordinary spawned
    // units remain worker-thread units. On-success units that would otherwise spawn run
    // through the SAME run_batch_unit path on the main thread: they still acquire an
    // AdmittedSlot, but now consume this thread's already-warm process_shared_index
    // instead of rebuilding a cold index in a fresh thread-local. The WalkPlan contract
    // deliberately permits withdrawing sibling overlap, and the live roster's stages are
    // singletons. Memo and Discovery placement is unchanged.
    let mut memo_units: Vec<BatchUnit> = Vec::new();
    let mut main_thread_units: Vec<BatchUnit> = Vec::new();
    let mut thread_units: Vec<BatchUnit> = Vec::new();
    for unit in units {
        match population_unit_lane(population, &unit, memo, memo_path_entries) {
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
            let obligation_subjects_owned = obligation_subjects.cloned();
            let boxed: Box<dyn FnOnce() -> Vec<ClaimResult> + Send> = Box::new(move || {
                run_batch_unit(
                    roots,
                    unit,
                    unit_governor,
                    fast_lane_eval_budget_ms,
                    wet_budgets,
                    obligation_subjects_owned.as_ref(),
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
            let results = run_memo_shared_claims(
                source_roots,
                &entry,
                &functions,
                execution_mode,
                memo,
                obligation_subjects,
            );
            memo_results.extend(results);
        }
    }
    // Main-thread units include Discovery pumps and on-success spawned-eligible claims.
    // Both intentionally consume this thread's process_shared_index — the one the eager
    // compile-clean receipt install warmed. run_batch_unit itself preserves governor-slot
    // acquisition for the claim case; Discovery keeps its existing admission behavior.
    for unit in main_thread_units {
        memo_results.extend(run_batch_unit(
            source_roots.to_vec(),
            unit,
            governor.clone(),
            fast_lane_eval_budget_ms,
            falsifier_self_host_wet_budgets.clone(),
            obligation_subjects,
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
    if !stage.iter().any(|r| {
        matches!(
            r,
            Runnable::DiscoveryBatch { .. } | Runnable::ScopedWitnessBatch { .. }
        )
    }) {
        for _ in &results {
            heartbeat_feed_entry_completed();
        }
    }
    let wall_nanos = stage_start.elapsed().as_nanos();
    let memory_after = stage_memory_snapshot();
    // THE COST WALL (Piece 3 derived clamp): the per-stage clamp is overhead + runtime
    // unit count * rate, computed HERE where the affected-set-selected count is known
    // (the schedule holds one opaque discovery runnable; the count is runtime).
    // Over-clamp is a typed, located refusal that reds the walk; it never widens (no
    // rerun, no scope change, no cap raise). Witness verdicts inside the stage stand as
    // evaluated — the clamp is an admission/scheduling fact, not a verdict term. A
    // population whose plan declares no clamp params gets None and is not clamped;
    // nothing is fabricated for it.
    let runtime_units = aggregate_batch_runtime_units(&results);
    let mut over_budget = false;
    let (unit_count, clamp_ms) = match (&runtime_units, clamp_params) {
        (FloorRuntimeUnitCount::Unavailable { cause }, Some(_)) => {
            println!(
                "{}",
                paint(
                    &format!(
                        "✗ FLOOR-BATCH-CLAMP-REFUSED {}={} units unavailable ({cause})                                  (clamp comparison refused; authority gunbc.ci_spec                                  gunbc_ci_floor_batch_runtime_unit_count_note — not FLOOR-BATCH-OVER-BUDGET)",
                        population.phase_slug(),
                        index + 1,
                    ),
                    sgr::ERROR
                )
            );
            (0u128, None)
        }
        (FloorRuntimeUnitCount::Observed { units }, Some(resolved)) => {
            let mut clamp = resolved.clamp_ms(*units);
            if let Some(t) = budget_tighten_ms {
                clamp = clamp.min(t);
            }
            let wall_ms = wall_nanos / 1_000_000;
            if wall_ms > clamp {
                over_budget = true;
                // Every term the reader needs to decide whether the units axis is even live:
                // per_unit_ms and units_contribution_ms are printed beside the count, because a
                // bare `units=27` invites trimming the roster on a row whose rate is zero — a
                // remedy that cannot move the threshold by construction, and whose split form
                // would hand the same work a second overhead allowance.
                let batch_id_field = match resolved.authority.batch_id() {
                    Some(id) => format!(" batch_id={id}"),
                    None => String::new(),
                };
                println!(
                    "{}",
                    paint(
                        &format!(
                            "✗ FLOOR-BATCH-OVER-BUDGET {}={}{} wall_ms={} clamp_ms={} overhead_ms={} units={} per_unit_ms={} units_contribution_ms={}                                  (clamp = overhead + units*rate; authority {}; raising an overhead or rate requires                                  an operator-signed line per gunbc_ci_floor_batch_clamp_note — a refusal,                                  never a widen)",
                            population.phase_slug(),
                            index + 1,
                            batch_id_field,
                            wall_ms,
                            clamp,
                            resolved.overhead_ms,
                            units,
                            resolved.per_unit_ms,
                            resolved.units_contribution_ms(*units),
                            resolved.authority.render(),
                        ),
                        sgr::ERROR
                    )
                );
            }
            (*units, Some(clamp))
        }
        (FloorRuntimeUnitCount::Observed { units }, None) => (*units, None),
        (FloorRuntimeUnitCount::Unavailable { .. }, None) => (0u128, None),
    };
    StageRun {
        results,
        label,
        wall_nanos,
        runtime_units,
        unit_count,
        clamp_ms,
        thread_panicked,
        over_budget,
        memory_before,
        memory_after,
    }
}

fn arm_population_budget_watchdog(
    population: &'static str,
    plan_site: &str,
    budget_ms: Option<u64>,
    progress: PopulationBudgetProgress,
) -> Arc<std::sync::atomic::AtomicBool> {
    let armed = Arc::new(std::sync::atomic::AtomicBool::new(budget_ms.is_some()));
    let Some(budget_ms) = budget_ms else {
        return armed;
    };
    let watchdog_armed = armed.clone();
    let site = plan_site.to_string();
    let started = Instant::now();
    let _ = std::thread::Builder::new()
        .name(format!("{population}-budget-watchdog"))
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(budget_ms));
            if watchdog_armed.load(std::sync::atomic::Ordering::Acquire) {
                let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
                let locus = progress.snapshot();
                let label = population.to_ascii_uppercase().replace('_', "-");
                let detail = population_budget_over_budget_detail(
                    &label, &site, &locus, elapsed_ms, budget_ms,
                );

                // THE REFUSAL IS ANNOUNCED BEFORE ANY WORK THAT CAN BLOCK OR FAIL, because the
                // incident this ordering fixes is a budget exhaustion that killed the floor and
                // said NOTHING. Run 31474198106 (PR #8132) ran the floor step for exactly 55m01s
                // against gunbc_ci_ordinary_floor_budget_minutes = 55, exited 1, and emitted no
                // OVER-BUDGET line anywhere in the attempt log — so the failure was
                // indistinguishable from an unexplained death, and readers reached for whatever
                // number was nearby (a cgroup peak RSS reading, in that case) and diagnosed a
                // memory kill that never happened. A fail-closed wall whose diagnostic is missing
                // is strictly worse than a loud one: the refusal is correct, the silence teaches
                // everyone the wrong cause.
                //
                // TWO CANDIDATE LOSS MECHANISMS, and this ordering closes both without needing to
                // decide between them. The receipt write ran FIRST and its path was interpolated
                // into the message, so a slow or stalled write under a loaded runner delayed the
                // only announcement past process death. And the message then took an explicit
                // stderr lock guard held across the write — a lock the main thread holds while it
                // streams its own receipt phases, which is exactly what the floor is doing in the
                // window this fired in. A watchdog thread must never wait on a resource the thread
                // it is policing can hold.
                //
                // The annotation goes to stdout as `::error::` so the cause reaches the run summary
                // rather than only line ~7000 of a step log; the same located detail is replayed
                // into the worker terminal receipt after these lines flush. eprintln!/println!
                // take the lock per call rather than holding a guard across several operations,
                // and the durable receipt writes move after both, where a failure to write can
                // no longer suppress the diagnosis.
                println!("::error::claim_executor: {detail}");
                eprintln!("claim_executor: {detail}");
                use std::io::Write as _;
                let _ = std::io::stdout().flush();
                let _ = std::io::stderr().flush();

                let receipt = PopulationBudgetRefusal {
                    population,
                    plan_site: &site,
                    population_index: locus.population_index,
                    active_unit: &locus.active_unit,
                    elapsed_ms,
                    budget_ms,
                };
                let receipt_path =
                    write_population_budget_refusal_at(Path::new("target"), &receipt)
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|error| format!("UNWRITABLE ({error})"));
                eprintln!("claim_executor: {label}-OVER-BUDGET receipt={receipt_path}");
                let _ = std::io::stderr().flush();
                population_budget_watchdog_exit(&detail);
            }
        });
    armed
}

#[derive(Clone, Debug)]
struct PopulationBudgetLocus {
    population_index: usize,
    active_unit: String,
}

#[derive(Clone)]
struct PopulationBudgetProgress(Arc<std::sync::Mutex<PopulationBudgetLocus>>);

impl PopulationBudgetProgress {
    fn before_first_unit() -> Self {
        Self(Arc::new(std::sync::Mutex::new(PopulationBudgetLocus {
            population_index: 0,
            active_unit: "<before first unit>".to_string(),
        })))
    }

    fn enter(&self, population_index: usize, active_unit: String) {
        let mut locus = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *locus = PopulationBudgetLocus {
            population_index,
            active_unit,
        };
    }

    fn snapshot(&self) -> PopulationBudgetLocus {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

struct PopulationBudgetRefusal<'a> {
    population: &'a str,
    plan_site: &'a str,
    population_index: usize,
    active_unit: &'a str,
    elapsed_ms: u64,
    budget_ms: u64,
}

fn population_budget_over_budget_detail(
    label: &str,
    plan_site: &str,
    locus: &PopulationBudgetLocus,
    elapsed_ms: u64,
    budget_ms: u64,
) -> String {
    format!(
        "{label}-OVER-BUDGET plan_site={plan_site} population_index={} active_unit={} elapsed_ms={elapsed_ms} budget_ms={budget_ms}; executor refusing inside the population boundary",
        locus.population_index, locus.active_unit,
    )
}

fn population_budget_refusal_body(receipt: &PopulationBudgetRefusal<'_>) -> String {
    format!(
        "population={}\nplan_site={}\npopulation_index={}\nactive_unit={}\nelapsed_ms={}\nbudget_ms={}\noutcome=refused\n",
        receipt.population,
        receipt.plan_site,
        receipt.population_index,
        receipt.active_unit,
        receipt.elapsed_ms,
        receipt.budget_ms,
    )
}

fn write_population_budget_refusal_at(
    base: &Path,
    receipt: &PopulationBudgetRefusal<'_>,
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(base)?;
    let path = base.join("floor-population-budget-refusal.txt");
    fs::write(&path, population_budget_refusal_body(receipt))?;
    Ok(path)
}

fn budget_unit_label(stage: &[Runnable]) -> String {
    stage
        .iter()
        .map(|runnable| match runnable {
            Runnable::SingleClaim {
                entry, function, ..
            } => format!("{entry}::{function}"),
            Runnable::DiscoveryBatch { .. } => "<discovery batch>".to_string(),
            Runnable::ScopedWitnessBatch { batch_id, .. } => {
                format!("<scoped witness batch {batch_id}>")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

// Why `floor_finalization` is absent this attempt — a typed enum rather than a bare
// bool or string so the scoped-by-construction arm and the incidental-absence arm
// cannot later drift into one bucket (a caller must name which it means). There is
// deliberately NO way to name "no reason" without naming it: `Undeclared` is a real
// variant a caller must pass explicitly, not a default a caller can fall through to —
// a caller that hasn't decided which cause applies still produces a counted line
// naming that fact, rather than silently producing nothing (review 2026-08-07,
// smart-badger-549: an earlier revision let an absent `Option` collapse to no
// disposition at all, reintroducing case B under a name that read as handled).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FloorFinalizationAbsenceReason {
    /// The scoped floor-worker branch sets `floor_finalization = None` deliberately:
    /// one batch cannot evaluate a whole-floor resolve-obligation identity join. This
    /// is correct behavior, not a defect — the line exists only to make the skip
    /// countable, never to make it refuse.
    ScopedWorkerByConstruction,
    /// The resolved plan itself declared `NoFinalizationDeclared {}` — the regen,
    /// plan-artifact, falsifier, and native-cache-cold plans never carry a
    /// `FloorFinalization` contract at all (`ci_floor_plan.dag`), so `walk_plan.finalization`
    /// parses to `None` before any worker-role branch runs. This is expected and
    /// by-construction, exactly like the scoped case, but for a different reason — a
    /// plan fact rather than a role fact — so it must not share `IncidentalAbsence`'s
    /// bucket (review 49917, cursor/composer-2.5: collapsing both into one label dilutes
    /// the incidental bucket with the four plans that will always land there, masking a
    /// genuine incidental absence's frequency).
    PlanDeclaresNoFinalization,
    /// The walk carried no finalization contract and no known construction reason
    /// explains why — the walk simply never reached the call, and the plan itself
    /// declared a real `FloorFinalization` contract. Distinct from both the scoped and
    /// plan-declared cases precisely because it is NOT expected, and its frequency is
    /// the signal this whole mechanism exists to make visible.
    IncidentalAbsence,
    /// The caller has no opinion on why finalization is absent here (e.g. a
    /// diagnostic-only re-walk that never applies the whole-floor contract at all).
    /// Distinct from both named causes so it can never be mistaken for either.
    Undeclared,
}

impl FloorFinalizationAbsenceReason {
    fn label(self) -> &'static str {
        match self {
            FloorFinalizationAbsenceReason::ScopedWorkerByConstruction => {
                "scoped-worker-by-construction"
            }
            FloorFinalizationAbsenceReason::PlanDeclaresNoFinalization => {
                "plan-declares-no-finalization"
            }
            FloorFinalizationAbsenceReason::IncidentalAbsence => "incidental-absence",
            FloorFinalizationAbsenceReason::Undeclared => "undeclared",
        }
    }
}

/// The floor-finalization verdict for one walk attempt, computed as a pure function of
/// its inputs so the disposition — and therefore the exact lines a reader sees — is
/// directly unit-testable without capturing process stderr. `run_walk` does nothing but
/// print `floor_finalization_disposition_lines(&disposition)` and act on
/// `Refused`; every other decision lives here.
#[derive(Clone, Debug, PartialEq, Eq)]
enum FloorFinalizationDisposition {
    /// Laws evaluated and held. Carries no ordinary-floor-outcome dependency — this
    /// verdict is unconditional: a held contract on an otherwise-failing floor is
    /// still information a reader needs (case C, still-bee-788/smart-badger-549).
    Held,
    /// Laws evaluated and at least one refused; the floor MUST fail.
    Refused(Vec<String>),
    /// Laws did not evaluate this attempt, for the named reason. Never fails the
    /// floor — visibility is the whole fix, per DESIGN §5's "don't make the scoped
    /// path refuse."
    Absent(FloorFinalizationAbsenceReason),
}

fn floor_finalization_disposition(
    floor_finalization: Option<&FloorFinalization>,
    absence_reason: FloorFinalizationAbsenceReason,
    plan_site: &str,
    batch_records: &[BatchRecord],
    walk_truncated: bool,
) -> FloorFinalizationDisposition {
    match floor_finalization {
        Some(fin) => {
            let refusals =
                validate_floor_finalization(fin, plan_site, batch_records, walk_truncated);
            floor_finalization_disposition_from_refusals(refusals)
        }
        None => FloorFinalizationDisposition::Absent(absence_reason),
    }
}

/// Split out so the Held/Refused mapping is testable without depending on
/// `validate_floor_finalization`'s on-disk materialization receipt read — and so the
/// mapping is visibly a pure function of `refusals` alone, with no `ordinary_failed`
/// (or any other floor-outcome) input anywhere in its signature. That absence is the
/// fix for case C: a held verdict cannot be suppressed by floor outcome because nothing
/// in this call chain ever receives it.
fn floor_finalization_disposition_from_refusals(
    refusals: Vec<String>,
) -> FloorFinalizationDisposition {
    if refusals.is_empty() {
        FloorFinalizationDisposition::Held
    } else {
        FloorFinalizationDisposition::Refused(refusals)
    }
}

// Scaffold: the sink `emit_floor_finalization_disposition` writes to is worker stderr in
// production, a stream the Actions harness drops — the same silent stream that let cases
// A/B/C go unnoticed in the first place. Making the verdict unconditional and typed (this
// PR) closes the emission side; it does not yet close the observation side. The durable
// fix is routing the disposition into the same persisted floor-materialization-receipt
// (or an equivalent counted receipt) the other floor observations already use, so a
// reader can count the verdict without depending on stderr reaching the log at all —
// and because emission is already behind a `Write` seam (below), that fix becomes a
// change of sink rather than a rewrite (review 2026-08-07, smart-badger-549).
// Dissolve-on: `FloorFinalizationDisposition` (or its rendered lines) is written to a
// persisted receipt alongside `target/floor-materialization-receipt.txt`, and this stderr
// emission becomes a redundant mirror of it rather than the sole carrier.
fn floor_finalization_disposition_lines(disposition: &FloorFinalizationDisposition) -> Vec<String> {
    match disposition {
        FloorFinalizationDisposition::Held => vec![
            "floor contract finalized — resolve obligation identity join holds and \
             materialization disclosure holds"
                .to_string(),
        ],
        FloorFinalizationDisposition::Refused(refusals) => refusals
            .iter()
            .map(|msg| format!("FLOOR-FINALIZATION-REFUSED: {msg}"))
            .collect(),
        FloorFinalizationDisposition::Absent(reason) => vec![format!(
            "FLOOR-FINALIZATION-ABSENT[{}]: floor finalization laws did not evaluate this attempt",
            reason.label()
        )],
    }
}

/// The one place the verdict actually leaves the process. Takes the sink as a
/// parameter rather than hardcoding stderr so emission is an observable seam: a test
/// can pass a `Vec<u8>` and assert the exact bytes a reader would see, so deleting or
/// bypassing this call in `run_walk` fails that test instead of leaving all-pure-function
/// tests green while the floor goes silent (review 2026-08-07, smart-badger-549 —
/// testing `floor_finalization_disposition_lines` alone proves the verdict is computed,
/// never that it is seen).
fn emit_floor_finalization_disposition(
    sink: &mut dyn std::io::Write,
    disposition: &FloorFinalizationDisposition,
) {
    for line in floor_finalization_disposition_lines(disposition) {
        // A write failure on the diagnostic sink is not itself a floor-finalization
        // verdict and must not be promoted into one; best-effort emission matches the
        // `eprintln!` this replaces, which likewise cannot make a broken stderr refuse
        // the floor.
        let _ = writeln!(sink, "claim_executor: {line}");
    }
}

fn run_walk(
    source_roots: &[String],
    plan_site: &str,
    batches: &[Vec<Runnable>],
    on_success_stages: &[Vec<Runnable>],
    floor_finalization: Option<&FloorFinalization>,
    finalization_absence_reason: FloorFinalizationAbsenceReason,
    // The finalization verdict's sink. Production callers pass `&mut std::io::stderr()`;
    // a test passes a `Vec<u8>` so emission itself — not just the disposition it prints —
    // is under test (see `emit_floor_finalization_disposition`).
    finalization_sink: &mut dyn std::io::Write,
    ordinary_budget_ms: Option<u64>,
    on_success_budget_ms: Option<u64>,
    // The walk's own deadline, derived from the foreign step timeout the runner enforces
    // (`gunbc.falsifier_workflow` `gunbc_falsifier_soft_deadline_minutes`). Armed only for
    // the falsifier, which is the plan carrying no `ordinary_budget` and running under a
    // SIGKILL it cannot conclude through. `None` leaves the walk's admission unbounded,
    // exactly as before.
    soft_deadline_ms: Option<u64>,
    governor: &Arc<RealizationConcurrency>,
    fast_lane_eval_budget_ms: Option<u64>,
    falsifier_self_host_wet_budgets: FalsifierSelfHostWetBudgets,
    stop_policy: FloorBatchStopPolicy,
    batch_clamp_params: Option<&[Option<ResolvedFloorBatchClamp>]>,
    budget_tighten_ms: Option<u128>,
    emit_ordinary_floor_receipts: bool,
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
    let ordinary_start = Instant::now();
    let ordinary_budget_progress = PopulationBudgetProgress::before_first_unit();
    let ordinary_budget_armed = arm_population_budget_watchdog(
        "ordinary_floor",
        plan_site,
        ordinary_budget_ms,
        ordinary_budget_progress.clone(),
    );
    // WHY the walk stopped admitting components, which is what the final receipt reports
    // about every component with no record. It starts at StopPolicy — the pre-existing
    // meaning, "the plan concluded and an earlier batch stopped the line" — and each break
    // that is NOT that sets its own cause. Carrying it as one variable rather than deciding
    // at the write site is what keeps the receipt's story and the loop's actual exit from
    // drifting apart.
    let mut walk_stop_cause = UnreachedCause::StopPolicy;
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
    let obligation_subjects = obligation_subject_set(floor_finalization);
    for (bi, batch) in batches.iter().enumerate() {
        if ordinary_budget_ms
            .is_some_and(|budget| ordinary_start.elapsed().as_millis() >= u128::from(budget))
        {
            let budget = ordinary_budget_ms.expect("checked Some above");
            any_failed = true;
            failure_details.push(format!(
                "ordinary floor exceeded its population budget before batch {}: budget_ms={budget}",
                bi + 1
            ));
            eprintln!(
                "claim_executor: ORDINARY-FLOOR-OVER-BUDGET before batch {} — budget_ms={budget}; admission postcondition allowance remains reserved",
                bi + 1
            );
            // This break is a deadline stop, not a stop-policy skip: the components below
            // were never started because the RUN ran out of time, not because an earlier
            // one failed. It previously reported the stop-policy cause, which named the
            // wrong reason on the ordinary floor for the same reason it would have on the
            // falsifier.
            walk_stop_cause = UnreachedCause::DeadlineReached;
            break;
        }
        // THE WALK'S OWN DEADLINE. Distinct from the population budget above: that one
        // bounds the ordinary floor's cost, this one exists because the falsifier runs
        // under a foreign SIGKILL (a GitHub Actions step timeout) that the process cannot
        // catch, log, or conclude through — crossing it destroyed the run's evidence
        // entirely. Stopping admission a flush allowance early lets the walk conclude
        // ITSELF, with a typed disposition on every unadmitted component and the receipt
        // written on the ordinary path. It refuses; it never widens or absorbs: the run is
        // red, every skipped component is counted, and reaching it is a measurement of a
        // cost problem rather than a resolution of one.
        if soft_deadline_ms
            .is_some_and(|deadline| walk_start.elapsed().as_millis() >= u128::from(deadline))
        {
            let deadline = soft_deadline_ms.expect("checked Some above");
            let elapsed_ms = walk_start.elapsed().as_millis();
            any_failed = true;
            failure_details.push(format!(
                "walk reached its soft deadline before batch {}: deadline_ms={deadline} elapsed_ms={elapsed_ms}",
                bi + 1
            ));
            eprintln!(
                "claim_executor: WALK-SOFT-DEADLINE before batch {} — deadline_ms={deadline} elapsed_ms={elapsed_ms}; \
                 admitting no further components so the receipt concludes before the step's hard kill",
                bi + 1
            );
            walk_stop_cause = UnreachedCause::DeadlineReached;
            break;
        }
        batches_run = bi + 1;
        ordinary_budget_progress.enter(bi + 1, budget_unit_label(batch));
        let StageRun {
            results: batch_results,
            label,
            wall_nanos: batch_wall_nanos,
            runtime_units: batch_runtime_units,
            unit_count: batch_unit_count,
            clamp_ms: batch_clamp_ms,
            thread_panicked,
            over_budget,
            memory_before: _,
            memory_after: _,
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
            batch_clamp_params
                .and_then(|p| p.get(bi))
                .cloned()
                .flatten(),
            budget_tighten_ms,
            obligation_subjects.as_ref(),
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
            runtime_units: batch_runtime_units,
            results: batch_results,
            label: label.clone(),
            is_wet: batch_is_wet(batch),
        });
        // CHECKPOINT. The receipt used to be written once, after every batch, so the one
        // failure mode that most needs evidence destroyed all of it: the falsifier's floor
        // step carries a foreign 170-minute SIGKILL and a run that crossed it produced no
        // file at all, leaving the alert to report ENOENT while every completed component's
        // real state — including green ones — was lost. Receipt: the two longest failing
        // runs of the 2026-08-03 window both landed there.
        //
        // Written after the RECORD lands, so a checkpoint always includes the batch that
        // just finished; the tail carries RunIncomplete, which is a distinct mode from the
        // stop-policy not_reached the final write uses (floor_component_run_incomplete_note
        // — the two route to different owners, so collapsing them would misattribute a kill
        // as a deliberate skip).
        //
        // COST: the writer resolves `gunbc/floor_component_receipt.dag` through
        // `resolve_entry_graph_shared`, which memoizes on (source_roots, entry) in
        // PROCESS_RESOLVE_STORE — so the first checkpoint pays the resolve and every later
        // one hits the memo. What repeats is the document eval, which is proportional to the
        // batch COUNT (single digits), not to the witness population. Checkpointing at
        // batch grain rather than per witness is what keeps that true.
        //
        // A failed checkpoint does NOT stop the line: the batch it describes has already
        // run, the final write is still ahead, and refusing the whole floor because an
        // intermediate snapshot could not be written would turn a diagnostic aid into a new
        // failure mode. It is loud (the writer prints its own refusal) and the final write
        // remains fail-closed, so a persistent write fault still reds the run there.
        if emit_ordinary_floor_receipts {
            let _ = write_floor_component_receipt_at(
                std::path::Path::new("target"),
                source_roots,
                &batch_records,
                &batches,
                UnreachedCause::RunIncomplete,
            );
        }
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
    trace_floor_phase("resolve-receipt", "started", "");
    let resolve_receipt_ok = !emit_ordinary_floor_receipts
        || write_resolve_receipt(source_roots, &batch_records, floor_finalization);
    trace_floor_phase("resolve-receipt", "completed", "");
    trace_floor_phase("batch-wall-receipt", "started", "");
    let batch_wall_receipt_ok =
        !emit_ordinary_floor_receipts || write_batch_wall_receipt(&batch_records);
    trace_floor_phase("batch-wall-receipt", "completed", "");
    trace_floor_phase("gate-warm-cost-receipt", "started", "");
    let gate_warm_cost_receipt_ok =
        !emit_ordinary_floor_receipts || write_gate_warm_cost_receipt(&batch_records);
    trace_floor_phase("gate-warm-cost-receipt", "completed", "");
    trace_floor_phase("wet-witness-row-outcome-receipt", "started", "");
    let wet_witness_row_outcome_receipt_ok = !emit_ordinary_floor_receipts
        || write_floor_wet_witness_row_outcome_receipt(&batch_records);
    trace_floor_phase("wet-witness-row-outcome-receipt", "completed", "");
    // Written on EVERY exit path, red included — a red run is precisely when the
    // alert needs to know which component failed (gunbc.floor_component_receipt).
    trace_floor_phase("floor-component-receipt", "started", "");
    let floor_component_receipt_ok = !emit_ordinary_floor_receipts
        || write_floor_component_receipt_at(
            std::path::Path::new("target"),
            source_roots,
            &batch_records,
            &batches,
            walk_stop_cause,
        );
    trace_floor_phase("floor-component-receipt", "completed", "");
    // Memo contexts absorb their ledger totals into the process accumulator on
    // Drop, so they must die before the materialization receipt is written.
    trace_floor_phase(
        "materialization-ledger-harvest",
        "started",
        &format!("contexts={}", walk_memo.len()),
    );
    drop(walk_memo);
    trace_floor_phase("materialization-ledger-harvest", "completed", "");
    let materialization_receipt_ok =
        !emit_ordinary_floor_receipts || write_materialization_receipt();
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
        || !wet_witness_row_outcome_receipt_ok
        || !floor_component_receipt_ok
        || !materialization_receipt_ok;
    push_ordinary_receipt_write_refusals(
        &mut failure_details,
        resolve_receipt_ok,
        batch_wall_receipt_ok,
        gate_warm_cost_receipt_ok,
        wet_witness_row_outcome_receipt_ok,
        floor_component_receipt_ok,
        materialization_receipt_ok,
    );
    // Floor finalization laws (the in-executor form of the deleted resolve/
    // materialization gate steps): validated AFTER the receipts wrote and BEFORE the
    // on-success stages, so a violation blocks admission instead of post-dating it.
    // The verdict below is UNCONDITIONAL — printed whether or not the ordinary floor
    // already failed for some other reason. A held contract on a failing floor is
    // still information a reader needs; suppressing it made "laws ran and held" and
    // "laws never ran" indistinguishable on exactly the reads that matter most
    // (case C, still-bee-788/smart-badger-549 — DESIGN §5's specification-without-
    // execution: a mechanism whose own verdict can vanish is not a verdict).
    let walk_truncated = batch_records.len() < batches.len();
    let disposition = floor_finalization_disposition(
        floor_finalization,
        finalization_absence_reason,
        plan_site,
        &batch_records,
        walk_truncated,
    );
    emit_floor_finalization_disposition(finalization_sink, &disposition);
    if let FloorFinalizationDisposition::Refused(refusals) = disposition {
        ordinary_failed = true;
        for msg in refusals {
            failure_details.push(format!("floor finalization refused: {msg}"));
        }
    }
    ordinary_budget_armed.store(false, std::sync::atomic::Ordering::Release);
    // On-success stages: run only on a fully-green ordinary floor; each stage is a
    // barrier and stage-to-stage execution is ALWAYS fail-fast — FloorBatchStopPolicy
    // is an ordinary-floor policy and never applies between stages. Members run through
    // `run_stage`, the SAME executor the ordinary batches use — same unit grouping, same
    // lane decision, same CLAMP MECHANISM, same per-lane admission rule. On-success
    // spawned-eligible units have one population-semantic placement refinement: they run
    // on this main thread to consume its warm thread-local index. Three of those
    // are deliberately weaker than an earlier draft's wording (review 2026-07-31). NOT
    // "same governor admission": admission is PER-LANE. Ordinary spawned units and
    // on-success main-thread claim units both reach run_batch_unit and acquire a slot;
    // memo and Discovery lanes do not. NOT "same derived clamp": ordinary batches SUPPLY clamp parameters, stages
    // deliberately pass None and stay inside a narrow admissible profile instead — the
    // mechanism is shared, the parameters are not. And NOT "concurrent exactly as the
    // contract says": the contract never guarantees sibling overlap, and on-success
    // placement deliberately withdraws it to preserve index warmth. The two populations
    // differ where their semantics require it: ordering/failure policy and warm-index
    // placement for postconditions. Their memo contexts drop AFTER
    // write_materialization_receipt above, so stage materialization is structurally NOT
    // folded into the floor receipt. A second attempt-scoped receipt harvests the stage
    // population after stage_memo drops.
    let mut on_success_failed = false;
    if !on_success_stages.is_empty() {
        if ordinary_failed {
            eprintln!(
                "claim_executor: ordinary floor failed — {} on-success stage(s) NOT run \
                 (green-only by construction)",
                on_success_stages.len()
            );
            // The skip receipt is still attempt-scoped: "the ordinary floor failed so no
            // stage ran" is an answer about THIS attempt, and an unattributable copy of it
            // in a reused worktree is indistinguishable from a prior attempt's.
            match walk_attempt_id {
                Some(attempt) => {
                    let _ = write_on_success_receipt(
                        &[],
                        0,
                        true,
                        on_success_stages.len(),
                        attempt,
                        plan_site,
                    );
                }
                None => eprintln!(
                    "claim_executor: on-success skip receipt NOT written — no walk-attempt \
                     identity (arm-time observation should have refused this walk)"
                ),
            }
        } else {
            let on_success_start = Instant::now();
            let on_success_budget_progress = PopulationBudgetProgress::before_first_unit();
            let on_success_budget_armed = arm_population_budget_watchdog(
                "on_success",
                plan_site,
                on_success_budget_ms,
                on_success_budget_progress.clone(),
            );
            let mut stage_rows: Vec<(usize, bool)> = Vec::new();
            let mut stage_resolves: u64 = 0;
            let mut stage_memo: std::collections::HashMap<(String, ExecutionMode), InterpContext> =
                std::collections::HashMap::new();
            // The stage population's own memo-path set. The stage memo is SEPARATE from
            // `walk_memo` by lifecycle, not by accident: stage contexts must drop after
            // `write_materialization_receipt` above so stage materialization is
            // structurally not folded into the floor receipt. Only heavy claims enroll in
            // the memo lane, and those refuse stage admission today; non-heavy claims in
            // separate stages therefore resolve independently. The map remains because it
            // is the common run_stage interface, not as a cross-stage reuse promise.
            let stage_memo_path_entries = memo_path_entry_keys(on_success_stages);
            for (si, stage) in on_success_stages.iter().enumerate() {
                if on_success_budget_ms.is_some_and(|budget| {
                    on_success_start.elapsed().as_millis() >= u128::from(budget)
                }) {
                    let budget = on_success_budget_ms.expect("checked Some above");
                    on_success_failed = true;
                    failure_details.push(format!(
                        "on-success population exceeded its budget before stage {}: budget_ms={budget}",
                        si + 1
                    ));
                    eprintln!(
                        "claim_executor: ON-SUCCESS-OVER-BUDGET before stage {} — budget_ms={budget}",
                        si + 1
                    );
                    break;
                }
                on_success_budget_progress.enter(si + 1, budget_unit_label(stage));
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
                    obligation_subjects.as_ref(),
                );
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
                        // A stage claim's channel is a Bool, so its typed cause crosses
                        // as a durable refusal wire (merge_admission_walk
                        // merge_admission_refresh_refusal_wire_note — the same pattern
                        // as the pre-walk capture). Read fresh per failure; absent wire
                        // is its own reported state.
                        let stage_cause =
                            match merge_admission_wire_read(MERGE_ADMISSION_REFRESH_REFUSAL_WIRE) {
                                Ok(wire) => format!("; refusal wire: {}", wire.trim()),
                                Err(_) => String::new(),
                            };
                        println!(
                            "{}",
                            paint(
                                &format!(
                                    "✗ FAIL [on-success stage {}] {} ({}{})",
                                    si + 1,
                                    r.function,
                                    r.detail,
                                    stage_cause
                                ),
                                sgr::ERROR
                            )
                        );
                        failure_details.push(format!(
                            "on-success stage {} fn={} detail={}{}",
                            si + 1,
                            r.function,
                            r.detail,
                            stage_cause
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
                        plan_site,
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
            drop(stage_memo);
            let materialization_written = match walk_attempt_id {
                Some(attempt) => write_on_success_materialization_receipt(attempt, plan_site),
                None => {
                    eprintln!(
                        "claim_executor: on-success materialization receipt has no walk-attempt \
                         identity — refusing to write an unidentifiable receipt"
                    );
                    false
                }
            };
            if !materialization_written {
                on_success_failed = true;
                failure_details.push("on-success materialization receipt write failed".to_string());
            }
            let aggregate_written = match walk_attempt_id {
                Some(attempt) => write_on_success_receipt(
                    &stage_rows,
                    stage_resolves,
                    false,
                    on_success_stages.len(),
                    attempt,
                    plan_site,
                ),
                None => {
                    eprintln!(
                        "claim_executor: on-success aggregate receipt has no walk-attempt \
                         identity — refusing to write an unattributable receipt"
                    );
                    false
                }
            };
            if !aggregate_written {
                on_success_failed = true;
            }
            on_success_budget_armed.store(false, std::sync::atomic::Ordering::Release);
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
                        native_bundle_entries,
                        exclude_substrings,
                        discovery_scope_dirs,
                        execution_mode,
                        spawns_host_compiler,
                    } => Runnable::DiscoveryBatch {
                        source_roots: roots.iter().map(|r| remap_root(r)).collect(),
                        scan_dirs: scan_dirs.iter().map(|d| remap_root(d)).collect(),
                        explicit_entries: explicit_entries.clone(),
                        native_bundle_entries: native_bundle_entries.clone(),
                        exclude_substrings: exclude_substrings.clone(),
                        discovery_scope_dirs: discovery_scope_dirs.clone(),
                        execution_mode: *execution_mode,
                        spawns_host_compiler: *spawns_host_compiler,
                    },
                    Runnable::ScopedWitnessBatch {
                        batch_id,
                        source_roots,
                        source_roots_digest,
                        entries,
                        scan_dirs,
                        execution_authority,
                        profile,
                        clamp,
                        process_isolation,
                    } => Runnable::ScopedWitnessBatch {
                        batch_id: batch_id.clone(),
                        source_roots: source_roots.iter().map(|r| remap_root(r)).collect(),
                        source_roots_digest: source_roots_digest.clone(),
                        entries: entries
                            .iter()
                            .map(|row| ScopedScheduleEntry {
                                entry: remap_entry_for_temp(primary, &temp_src, &row.entry)
                                    .to_string_lossy()
                                    .into_owned(),
                                function: row.function.clone(),
                                witness_kind: row.witness_kind.clone(),
                            })
                            .collect(),
                        scan_dirs: scan_dirs.iter().map(|d| remap_root(d)).collect(),
                        execution_authority: *execution_authority,
                        profile: *profile,
                        clamp: clamp.clone(),
                        process_isolation: *process_isolation,
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
        // Diagnostic-only re-walk of a perturbed single witness — the floor's whole-
        // contract finalization laws never apply here, so absence is Undeclared rather
        // than one of the two named causes, but is still visible.
        FloorFinalizationAbsenceReason::Undeclared,
        &mut std::io::stderr(),
        None,
        None,
        None,
        &RealizationConcurrency::for_walk(1).expect("test schedule"),
        None,
        FalsifierSelfHostWetBudgets::default(),
        FloorBatchStopPolicy::StopBeforeDependents,
        None,
        None,
        true,
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

const FLOOR_WORKER_TERMINAL_ENV: &str = "GUNBC_FLOOR_WORKER_TERMINAL_RECEIPT";
const FLOOR_PHASE_JOURNAL_ENV: &str = "GUNBC_FLOOR_PHASE_JOURNAL";
const FLOOR_WORKER_OBSERVATION_RECEIPT_PATH: &str = "target/floor-worker-observation-receipt.tsv";
const FLOOR_WET_WITNESS_ROW_OUTCOME_RECEIPT_PATH: &str =
    "target/floor-wet-witness-row-outcome-receipt.tsv";

fn wet_witness_row_outcome_replay_line(
    batch: &str,
    entry: &str,
    function: &str,
    outcome: &str,
    detail: &str,
) -> String {
    format!(
        "[wet-witness-row-outcome] batch={batch} entry={entry} function={function} outcome={outcome} detail={detail}"
    )
}

fn collect_wet_witness_row_outcome_replay_lines(path: &Path) -> Result<Vec<String>, String> {
    let body = fs::read_to_string(path).map_err(|e| {
        format!(
            "read wet witness row-outcome receipt {}: {e}",
            path.display()
        )
    })?;
    let mut lines = Vec::new();
    for line in body.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        let (batch, entry, function, outcome, detail) = match parts.len() {
            4 => (parts[0], parts[1], parts[2], parts[3], ""),
            5 => (parts[0], parts[1], parts[2], parts[3], parts[4]),
            _ => {
                return Err(format!(
                    "malformed wet witness row-outcome line (need 4-5 cols): {line}"
                ));
            }
        };
        lines.push(wet_witness_row_outcome_replay_line(
            batch, entry, function, outcome, detail,
        ));
    }
    Ok(lines)
}

fn replay_floor_wet_witness_row_outcomes_from_receipt(path: &Path) -> Result<usize, String> {
    let lines = collect_wet_witness_row_outcome_replay_lines(path)?;
    for line in &lines {
        eprintln!("{line}");
    }
    Ok(lines.len())
}

fn replay_ordinary_floor_wet_witness_row_outcomes() {
    let path = Path::new(FLOOR_WET_WITNESS_ROW_OUTCOME_RECEIPT_PATH);
    match replay_floor_wet_witness_row_outcomes_from_receipt(path) {
        Ok(count) => {
            eprintln!(
                "[wet-witness-row-outcome] coordinator replayed {count} row(s) from {}",
                path.display()
            );
            append_floor_phase_journal(
                "wet-witness-row-outcome-replay",
                "completed",
                &format!("row_count={count} path={}", path.display()),
            );
            if let Ok(lines) = collect_wet_witness_row_outcome_replay_lines(path) {
                for line in lines {
                    append_floor_phase_journal("wet-witness-row-outcome-replay", "row", &line);
                }
            }
        }
        Err(msg) if path.exists() => {
            eprintln!("claim_executor: wet witness row-outcome coordinator replay refused: {msg}");
            append_floor_phase_journal("wet-witness-row-outcome-replay", "refused", &msg);
        }
        Err(_) => {
            eprintln!(
                "claim_executor: wet witness row-outcome receipt absent at {} — per-row wet batch outcomes unobservable",
                path.display()
            );
            append_floor_phase_journal(
                "wet-witness-row-outcome-replay",
                "absent",
                &format!("path={}", path.display()),
            );
        }
    }
}

fn append_floor_phase_journal(phase: &str, state: &str, detail: &str) {
    let Some(path) = std::env::var_os(FLOOR_PHASE_JOURNAL_ENV) else {
        return;
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "claim_executor: create floor phase journal directory {}: {e}",
                parent.display()
            );
            return;
        }
    }
    let mut file = match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "claim_executor: open floor phase journal {}: {e}",
                path.display()
            );
            return;
        }
    };
    let unix_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let clean_detail = detail.replace(['\t', '\r', '\n'], " ");
    let row = format!(
        "{unix_millis}\t{}\t{phase}\t{state}\t{clean_detail}\n",
        std::process::id()
    );
    use std::io::Write as _;
    if let Err(e) = file
        .write_all(row.as_bytes())
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_data())
    {
        eprintln!(
            "claim_executor: persist floor phase journal {}: {e}",
            path.display()
        );
    }
}

fn trace_floor_phase(phase: &str, state: &str, detail: &str) {
    // Persist first: stdout is the channel under diagnosis and may block before
    // the runner can surface the marker.  The workflow reads this synced journal
    // from an always-running post-step after the floor process group is gone.
    append_floor_phase_journal(phase, state, detail);
    if detail.is_empty() {
        eprintln!("[floor-phase] phase={phase} state={state}");
    } else {
        eprintln!("[floor-phase] phase={phase} state={state} {detail}");
    }
}

#[derive(Clone, PartialEq, Eq)]
enum FloorWorkerRole {
    Ordinary,
    Scoped { batch_id: String },
}

struct ObservedFloorWorker {
    worker: String,
    termination: ProcessTermination,
    terminal_receipt: FloorWorkerTerminalReceipt,
}

/// Seed projection of `std.process_termination` `ProcessTermination` — how an observed process
/// ended. One carrier for both places the executor observes a child: the floor-worker
/// coordinator (which spawns workers directly and reads an `ExitStatus`) and the native
/// bundle transport (which reads the termination the interpreter transport carries).
/// Named for the concept rather than for either subject so the second consumer did not
/// mint a second spelling of it.
#[derive(Debug, PartialEq, Eq, Clone)]
enum ProcessTermination {
    Exited(i32),
    Signaled(i32),
    Unobserved,
}

impl ProcessTermination {
    /// Located rendering for a diagnostic. A signalled process says so; it never
    /// borrows an exit code it does not have.
    fn located(&self) -> String {
        match self {
            ProcessTermination::Exited(code) => format!("exited {code}"),
            ProcessTermination::Signaled(signal) => format!("killed by signal {signal}"),
            ProcessTermination::Unobserved => "termination unobserved".to_string(),
        }
    }
}

/// The transport violated its own modeled wire: the `termination` value could not be
/// decoded as a `std.process_termination` `ProcessTermination`. Deliberately a DIFFERENT state
/// from `Unobserved` — Unobserved is the transport honestly reporting that the spawn
/// never produced a status (an observation about the child), while this refusal means
/// the transport or the interning under it is broken (a defect in the wire). The two
/// have different owners and different fixes, so the decoder refuses instead of
/// absorbing malformed wire into the legitimate arm.
#[derive(Debug)]
struct ProcessTerminationDecodeRefusal {
    located: String,
}

/// Read the `std.process_termination` `ProcessTermination` the transport carries. ONLY the
/// explicit modeled `ProcessTerminationUnobserved` arm decodes to `Unobserved`;
/// absent, mistyped, unknown-variant, and missing/non-integer-field wire all carry a
/// located decode refusal, never a fabricated exit code and never a borrowed
/// legitimate arm.
fn transport_termination(
    value: Option<&Value>,
    ctx: &InterpContext,
) -> Result<ProcessTermination, ProcessTerminationDecodeRefusal> {
    let refuse = |located: String| Err(ProcessTerminationDecodeRefusal { located });
    let (variant_name, fields) = match value {
        Some(Value::Variant {
            variant_name,
            fields,
            ..
        }) => (variant_name, fields),
        Some(other) => {
            return refuse(format!(
                "termination is a {} where a ProcessTermination variant was modeled",
                other.type_label_public()
            ))
        }
        None => return refuse("termination field is absent from the transport record".to_string()),
    };
    let int_field = |name: &str, variant: &str| match ctx.field(fields, name) {
        Some(Value::Int(n)) => i32::try_from(*n).map_err(|_| ProcessTerminationDecodeRefusal {
            located: format!("{variant}.{name} {n} does not fit an i32"),
        }),
        Some(other) => Err(ProcessTerminationDecodeRefusal {
            located: format!(
                "{variant}.{name} is a {} where Int was modeled",
                other.type_label_public()
            ),
        }),
        None => Err(ProcessTerminationDecodeRefusal {
            located: format!("{variant} is missing its `{name}` field"),
        }),
    };
    if ctx.sym_eq(*variant_name, "ProcessExited") {
        return Ok(ProcessTermination::Exited(int_field(
            "code",
            "ProcessExited",
        )?));
    }
    if ctx.sym_eq(*variant_name, "ProcessSignaled") {
        return Ok(ProcessTermination::Signaled(int_field(
            "signal",
            "ProcessSignaled",
        )?));
    }
    if ctx.sym_eq(*variant_name, "ProcessTerminationUnobserved") {
        return Ok(ProcessTermination::Unobserved);
    }
    refuse(format!(
        "unknown ProcessTermination variant `{}`",
        ctx.resolve(*variant_name)
    ))
}

#[derive(Debug, PartialEq, Eq)]
enum FloorWorkerTerminalReceipt {
    Observed(FloorWorkerTerminalReport),
    Missing,
}

#[derive(Debug, PartialEq, Eq)]
enum FloorWorkerTerminalReport {
    Completed(String),
    Refused(String),
    Failed(String),
    Malformed(String),
}

struct DerivedFloorWorkerOutcome {
    label: &'static str,
    detail: String,
}
fn exit_status_termination(status: &ExitStatus) -> ProcessTermination {
    if let Some(code) = status.code() {
        return ProcessTermination::Exited(code);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;
        if let Some(signal) = status.signal() {
            return ProcessTermination::Signaled(signal);
        }
    }
    ProcessTermination::Unobserved
}

fn floor_worker_termination_label(termination: &ProcessTermination) -> String {
    match termination {
        ProcessTermination::Exited(code) => format!("exited:{code}"),
        ProcessTermination::Signaled(signal) => format!("signaled:{signal}"),
        ProcessTermination::Unobserved => "termination-unobserved".to_string(),
    }
}

fn floor_worker_terminal_report_label(report: &FloorWorkerTerminalReport) -> &'static str {
    match report {
        FloorWorkerTerminalReport::Completed(_) => "completed",
        FloorWorkerTerminalReport::Refused(_) => "refused",
        FloorWorkerTerminalReport::Failed(_) => "failed",
        FloorWorkerTerminalReport::Malformed(_) => "malformed",
    }
}

fn floor_worker_observation_outcome(row: &ObservedFloorWorker) -> DerivedFloorWorkerOutcome {
    let termination = floor_worker_termination_label(&row.termination);
    match &row.terminal_receipt {
        FloorWorkerTerminalReceipt::Missing => DerivedFloorWorkerOutcome {
            label: "died-without-terminal-receipt",
            detail: format!(
                "worker `{}` terminated as {termination}; no terminal receipt was observed",
                row.worker
            ),
        },
        FloorWorkerTerminalReceipt::Observed(report) => match (&row.termination, report) {
            (ProcessTermination::Exited(0), FloorWorkerTerminalReport::Completed(detail)) => {
                DerivedFloorWorkerOutcome {
                    label: "completed",
                    detail: detail.clone(),
                }
            }
            (ProcessTermination::Exited(code), FloorWorkerTerminalReport::Refused(detail))
                if *code != 0 =>
            {
                DerivedFloorWorkerOutcome {
                    label: "refused",
                    detail: detail.clone(),
                }
            }
            (_, FloorWorkerTerminalReport::Failed(detail))
            | (_, FloorWorkerTerminalReport::Malformed(detail)) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: detail.clone(),
            },
            (ProcessTermination::Signaled(signal), _) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: format!(
                    "worker `{}` reported {} but died from signal {signal}",
                    row.worker,
                    floor_worker_terminal_report_label(report)
                ),
            },
            (ProcessTermination::Unobserved, _) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: format!(
                    "worker `{}` reported {} but process termination was unobserved",
                    row.worker,
                    floor_worker_terminal_report_label(report)
                ),
            },
            (ProcessTermination::Exited(code), _) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: format!(
                    "worker `{}` report {} contradicted exit code {code}",
                    row.worker,
                    floor_worker_terminal_report_label(report)
                ),
            },
        },
    }
}

fn observe_floor_worker(
    worker: &str,
    status: ExitStatus,
    terminal_path: &Path,
) -> ObservedFloorWorker {
    let termination = exit_status_termination(&status);
    let terminal = fs::read_to_string(terminal_path).ok();
    let terminal_receipt = match terminal {
        None => FloorWorkerTerminalReceipt::Missing,
        Some(body) => {
            let line = body.trim_end_matches(['\r', '\n']);
            let (label, terminal_detail) = line.split_once('\t').unwrap_or((line, ""));
            let report = match label {
                "completed" => FloorWorkerTerminalReport::Completed(terminal_detail.to_string()),
                "refused" => FloorWorkerTerminalReport::Refused(terminal_detail.to_string()),
                "failed" => FloorWorkerTerminalReport::Failed(terminal_detail.to_string()),
                _ => FloorWorkerTerminalReport::Malformed(format!(
                    "unknown worker terminal report `{label}`: {terminal_detail}"
                )),
            };
            FloorWorkerTerminalReceipt::Observed(report)
        }
    };
    ObservedFloorWorker {
        worker: worker.to_string(),
        termination,
        terminal_receipt,
    }
}

fn append_floor_worker_observation(row: &ObservedFloorWorker) -> Result<(), String> {
    let path = Path::new(FLOOR_WORKER_OBSERVATION_RECEIPT_PATH);
    let needs_header = !path.exists();
    let mut options = fs::OpenOptions::new();
    options.create(true).append(true);
    let mut file = options
        .open(path)
        .map_err(|e| format!("open floor worker observation {}: {e}", path.display()))?;
    use std::io::Write as _;
    if needs_header {
        writeln!(
            file,
            "worker\ttermination\tterminal_receipt\tterminal_report\toutcome\tdetail"
        )
        .map_err(|e| format!("write floor worker observation header: {e}"))?;
    }
    let outcome = floor_worker_observation_outcome(row);
    let (receipt_label, report_label) = match &row.terminal_receipt {
        FloorWorkerTerminalReceipt::Missing => ("missing", "absent"),
        FloorWorkerTerminalReceipt::Observed(report) => {
            ("observed", floor_worker_terminal_report_label(report))
        }
    };
    let clean_detail = outcome.detail.replace(['\t', '\r', '\n'], " ");
    writeln!(
        file,
        "{}\t{}\t{}\t{}\t{}\t{}",
        row.worker,
        floor_worker_termination_label(&row.termination),
        receipt_label,
        report_label,
        outcome.label,
        clean_detail
    )
    .map_err(|e| format!("write floor worker observation row: {e}"))
}

fn floor_worker_succeeded(row: &ObservedFloorWorker) -> bool {
    floor_worker_observation_outcome(row).label == "completed"
}

fn journal_floor_worker_observation(row: &ObservedFloorWorker) {
    let outcome = floor_worker_observation_outcome(row);
    append_floor_phase_journal(
        "coordinator-observation",
        outcome.label,
        &format!(
            "worker={} termination={} detail={}",
            row.worker,
            floor_worker_termination_label(&row.termination),
            outcome.detail.replace(['\t', '\r', '\n'], " ")
        ),
    );
}

fn source_roots_from_executor_args(args: &[String]) -> Result<Vec<String>, String> {
    let mut roots = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--source-root" {
            i += 1;
            let Some(root) = args.get(i) else {
                return Err("claim_executor: --source-root requires a value".to_string());
            };
            roots.push(root.clone());
        }
        i += 1;
    }
    if roots.is_empty() {
        return Err("claim_executor: --source-root is required".to_string());
    }
    Ok(roots)
}

fn spawn_floor_worker(
    base_args: &[String],
    role: &str,
    batch_id: Option<&str>,
    ordinal: usize,
    walk_attempt_id: &str,
) -> Result<ObservedFloorWorker, String> {
    let worker = match batch_id {
        Some(id) => format!("scoped:{id}"),
        None => "ordinary".to_string(),
    };
    let terminal_path = PathBuf::from(format!(
        "target/floor-worker-terminal-{ordinal}-{}.tsv",
        worker.replace(':', "-")
    ));
    let _ = fs::remove_file(&terminal_path);
    let exe = std::env::current_exe().map_err(|e| format!("locate claim_executor: {e}"))?;
    let mut command = Command::new(exe);
    command.args(base_args).arg("--floor-worker-role").arg(role);
    if let Some(id) = batch_id {
        command.arg("--scoped-batch-id").arg(id);
    }
    command.env(FLOOR_WORKER_TERMINAL_ENV, &terminal_path);
    command.env("GUNBC_FLOOR_WALK_ATTEMPT_ID", walk_attempt_id);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        // When the thin coordinator is killed by the foreign step timeout, workers must not
        // keep running as orphaned claim_executors (signature 2). SIGTERM on parent death
        // gives the worker a chance to flush its terminal receipt before the runner reaps it.
        unsafe {
            command.pre_exec(|| {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    // The worker's post-walk ledger harvest can spend minutes dropping its memo
    // contexts after the final discovery line.  A blocking `Command::status`
    // made that whole interval silent: the worker's own heartbeat shares its
    // allocator and can stop making progress during the drop, while the thin
    // coordinator had no chance to say that the child was still alive.  Keep
    // the execution and receipt ordering unchanged, but let the coordinator
    // observe the wait and emit a pulse from its independent process.
    let mut child = command
        .spawn()
        .map_err(|e| format!("spawn floor worker `{worker}`: {e}"))?;
    let wait_started = Instant::now();
    // Per-worker, beside `wait_started`, NOT a `static`: a process-global counter is shared by
    // every worker's wait loop, so with N workers each one prints roughly every Nth tick and a
    // given worker can skip all of its own heartbeats. Liveness per worker is the entire fact this
    // line carries, so the counter has to have the same scope as the thing it reports on.
    let mut wait_ticks: u64 = 0;
    let status = loop {
        match child
            .try_wait()
            .map_err(|e| format!("observe floor worker `{worker}`: {e}"))?
        {
            Some(status) => {
                append_floor_phase_journal(
                    "coordinator-wait",
                    "completed",
                    &format!(
                        "worker={worker} elapsed_seconds={}",
                        wait_started.elapsed().as_secs()
                    ),
                );
                break status;
            }
            None => {
                append_floor_phase_journal(
                    "coordinator-wait",
                    "running",
                    &format!(
                        "worker={worker} elapsed_seconds={}",
                        wait_started.elapsed().as_secs()
                    ),
                );
                // CADENCE DIVERGENCE, marked here because this diff creates it: the journal append
                // above runs EVERY iteration, this print runs every tenth. They were one cadence
                // before, so anything that reads these two sites as interchangeable is now wrong.
                // Attach no state write to this site — a snapshot written here is up to five
                // minutes stale, and a consumer polling it for liveness (a stuck-worker detector is
                // the live proposal) would inherit that staleness as its detection floor. The
                // per-iteration journal site is where a progress fact belongs.
                //
                // The wait tick is journaled above on every iteration; the console keeps one line
                // per worker per five minutes rather than one per thirty seconds. Not journal-only, and the
                // difference is deliberate: the journal is an artifact a reader reaches AFTER the
                // run, so a floor that hangs would print nothing for ninety minutes and then a
                // timeout, which reads as a dead process rather than a waiting one. Liveness is
                // the one fact a heartbeat exists to carry, so it stays on the console at the
                // coarsest cadence that still carries it.
                {
                    let n = wait_ticks;
                    wait_ticks += 1;
                    if n % 10 == 0 {
                        eprintln!(
                            "[floor-worker-wait] worker={worker} elapsed_seconds={} state=running",
                            wait_started.elapsed().as_secs()
                        );
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }
    };
    let observed = observe_floor_worker(&worker, status, &terminal_path);
    // Replay before observation persistence: worker stderr may be dropped on Actions after
    // discovery, and a failed worker is the case where the log otherwise says nothing.
    if worker == "ordinary" {
        replay_ordinary_floor_wet_witness_row_outcomes();
    }
    append_floor_worker_observation(&observed)?;
    let outcome = floor_worker_observation_outcome(&observed);
    // The Actions log transport can drop the worker's inherited stderr after a
    // large discovery run. Persist the derived verdict before reporting it on
    // that channel so the existing always() post-step still surfaces the exact
    // failure arm and detail.
    journal_floor_worker_observation(&observed);
    eprintln!(
        "[floor-worker-observation] worker={} termination={} terminal_receipt={:?} outcome={} detail={}",
        observed.worker,
        floor_worker_termination_label(&observed.termination),
        observed.terminal_receipt,
        outcome.label,
        outcome.detail
    );
    Ok(observed)
}

fn floor_plan_function_arg(args: &[String]) -> Option<&str> {
    args.windows(2)
        .find(|pair| pair[0] == "--plan-function")
        .map(|pair| pair[1].as_str())
}

fn coordinator_terminal_refusal(detail: &str) -> ExitCode {
    append_floor_phase_journal("coordinator-terminal", "refused", detail);
    eprintln!("claim_executor: floor coordinator refusal: {detail}");
    ExitCode::from(1)
}

fn coordinator_report_worker_failure(worker: &ObservedFloorWorker, context: &str) -> ExitCode {
    let outcome = floor_worker_observation_outcome(worker);
    let detail = format!(
        "{context}: worker={} termination={} outcome={} detail={}",
        worker.worker,
        floor_worker_termination_label(&worker.termination),
        outcome.label,
        outcome.detail
    );
    coordinator_terminal_refusal(&detail)
}

fn maybe_run_floor_coordinator(args: &[String]) -> Option<ExitCode> {
    if args.iter().any(|arg| arg == "--floor-worker-role")
        || floor_plan_function_arg(args) != Some("gunbc_ci_floor_plan")
    {
        return None;
    }
    if let Some(parent) = Path::new(FLOOR_WORKER_OBSERVATION_RECEIPT_PATH).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator receipt directory refusal: {e}"
            )));
        }
    }
    let _ = fs::remove_file(FLOOR_WORKER_OBSERVATION_RECEIPT_PATH);
    let _ = fs::remove_file(SCOPED_EXECUTION_REQUESTS_PATH);
    if let Some(path) = std::env::var_os(FLOOR_PHASE_JOURNAL_ENV) {
        let _ = fs::remove_file(path);
    }
    if let Err(msg) = initialize_scoped_witness_receipt() {
        return Some(coordinator_terminal_refusal(&format!(
            "floor coordinator receipt arm refusal: {msg}"
        )));
    }
    let walk_attempt_id = match observe_walk_attempt_id() {
        Ok(id) => id,
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator walk-attempt refusal: {msg}"
            )));
        }
    };
    let ordinary = match spawn_floor_worker(args, "ordinary", None, 0, &walk_attempt_id) {
        Ok(observed) => observed,
        Err(msg) => {
            replay_ordinary_floor_wet_witness_row_outcomes();
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator ordinary-worker refusal: {msg}"
            )));
        }
    };
    if !floor_worker_succeeded(&ordinary) {
        return Some(coordinator_report_worker_failure(
            &ordinary,
            "stopping before scoped workers because ordinary worker did not complete",
        ));
    }
    let source_roots = match source_roots_from_executor_args(args) {
        Ok(roots) => roots,
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator source-roots refusal: {msg}"
            )));
        }
    };
    let excludes = v1_compiler::cli_run::witness_exclusion_substrings();
    let pre_plan_request = match build_floor_discovery_request(
        &source_roots,
        &[],
        &excludes,
        &[],
        "Hermetic",
        &source_roots,
    ) {
        Ok(request) => request,
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator pre-plan request refusal: {msg}"
            )));
        }
    };
    if let Err(msg) =
        verify_floor_discovery_terminal_for_coordinator(&walk_attempt_id, &pre_plan_request)
    {
        return Some(coordinator_terminal_refusal(&format!(
            "floor coordinator snapshot terminal refusal: {msg}"
        )));
    }
    let batch_ids = match read_scoped_execution_requests() {
        Ok(requests) => requests
            .into_iter()
            .map(|request| request.batch_id)
            .collect::<Vec<String>>(),
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator scoped request refusal: {msg}"
            )));
        }
    };
    for (index, batch_id) in batch_ids.iter().enumerate() {
        let scoped = match spawn_floor_worker(
            args,
            "scoped",
            Some(batch_id),
            index.saturating_add(1),
            &walk_attempt_id,
        ) {
            Ok(observed) => observed,
            Err(msg) => {
                return Some(coordinator_terminal_refusal(&format!(
                    "floor coordinator scoped-worker `{batch_id}` refusal: {msg}"
                )));
            }
        };
        if !floor_worker_succeeded(&scoped) {
            return Some(coordinator_report_worker_failure(
                &scoped,
                &format!("stopping after scoped worker `{batch_id}` did not complete"),
            ));
        }
    }
    append_floor_phase_journal(
        "coordinator-terminal",
        "completed",
        &format!(
            "scoped_workers={} walk_attempt_id={walk_attempt_id}",
            batch_ids.len()
        ),
    );
    eprintln!(
        "[floor-coordinator] outcome=completed scoped_workers={} walk_attempt_id={walk_attempt_id}",
        batch_ids.len()
    );
    Some(ExitCode::SUCCESS)
}

fn write_floor_worker_terminal(outcome: &str, detail: &str) -> Result<(), String> {
    let Some(path) = std::env::var_os(FLOOR_WORKER_TERMINAL_ENV) else {
        return Ok(());
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create worker terminal directory {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&tmp, format!("{outcome}\t{detail}\n"))
        .map_err(|e| format!("write worker terminal {}: {e}", tmp.display()))?;
    fs::rename(&tmp, &path).map_err(|e| format!("publish worker terminal {}: {e}", path.display()))
}

const SCOPED_EXECUTION_REQUESTS_PATH: &str = "target/floor-attempts/scoped-execution-requests.json";

/// The exact work a scoped child is asked to do, published by the ordinary worker before the
/// child is spawned.
///
/// A scoped child used to receive the ordinary worker's whole CLI and rebuild its way back to one
/// answer: it re-resolved the plan entry, re-evaluated the plan, and scanned the resulting batches
/// for the single `ScopedWitnessBatch` whose id matched its `--scoped-batch-id`. Every other value
/// that evaluation produced was discarded behind a `!Scoped` guard. This carrier hands the child
/// that one answer directly, with the plan-derived budgets it genuinely reads, so the second
/// resolve and evaluation have nothing left to compute.
///
/// The subject fields are not decoration: a child must refuse a request frozen against a different
/// commit, tree, or tool rather than executing it against whatever it finds on disk.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ScopedExecutionRequest {
    tested_commit: String,
    tested_tree: String,
    tool_identity: String,
    batch_id: String,
    source_roots: Vec<String>,
    source_roots_digest: String,
    entries: Vec<ScopedScheduleEntry>,
    scan_dirs: Vec<String>,
    execution_authority: ScopedWitnessExecutionAuthority,
    profile: ParsedRunnableProfile,
    clamp: ResolvedFloorBatchClamp,
    process_isolation: ScopedProcessIsolation,
    /// Plan-derived and read by the child while executing rows. Carried here because the child no
    /// longer evaluates the plan that produced them.
    fast_lane_eval_budget_ms: Option<u64>,
    ordinary_budget_ms: Option<u64>,
    /// Also plan-derived, and the reason the first two CI runs of this change died: the stop
    /// policy is resolved UNCONDITIONALLY for every worker, so a child with no plan context
    /// refused there after the ordinary walk had already succeeded. It is the parent's decision
    /// in exactly the sense the rest of this carrier is — freeze it, hand it over.
    batch_stop_policy: FloorBatchStopPolicy,
}

impl ScopedExecutionRequest {
    fn to_runnable(&self) -> Runnable {
        Runnable::ScopedWitnessBatch {
            batch_id: self.batch_id.clone(),
            source_roots: self.source_roots.clone(),
            source_roots_digest: self.source_roots_digest.clone(),
            entries: self.entries.clone(),
            scan_dirs: self.scan_dirs.clone(),
            execution_authority: self.execution_authority,
            profile: self.profile,
            clamp: self.clamp.clone(),
            process_isolation: self.process_isolation,
        }
    }
}

fn scoped_execution_requests_from_rows(
    rows: &[Runnable],
    fast_lane_eval_budget_ms: Option<u64>,
    ordinary_budget_ms: Option<u64>,
    batch_stop_policy: FloorBatchStopPolicy,
) -> Result<Vec<ScopedExecutionRequest>, String> {
    let (tested_commit, tested_tree) = v1_compiler::cli_run::floor_tested_commit_and_tree()?;
    let tool_identity = v1_compiler::cli_run::floor_tool_identity()?;
    let mut out = Vec::new();
    for runnable in rows {
        if let Runnable::ScopedWitnessBatch {
            batch_id,
            source_roots,
            source_roots_digest,
            entries,
            scan_dirs,
            execution_authority,
            profile,
            clamp,
            process_isolation:
                isolation @ (ScopedProcessIsolation::SequentialChildProcess
                | ScopedProcessIsolation::FreshJobProcess),
        } = runnable
        {
            out.push(ScopedExecutionRequest {
                tested_commit: tested_commit.clone(),
                tested_tree: tested_tree.clone(),
                tool_identity: tool_identity.clone(),
                batch_id: batch_id.clone(),
                source_roots: source_roots.clone(),
                source_roots_digest: source_roots_digest.clone(),
                entries: entries.clone(),
                scan_dirs: scan_dirs.clone(),
                execution_authority: *execution_authority,
                profile: *profile,
                clamp: clamp.clone(),
                process_isolation: *isolation,
                fast_lane_eval_budget_ms,
                ordinary_budget_ms,
                batch_stop_policy,
            });
        }
    }
    Ok(out)
}

fn write_scoped_execution_requests(requests: &[ScopedExecutionRequest]) -> Result<(), String> {
    let path = Path::new(SCOPED_EXECUTION_REQUESTS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create scoped request directory {}: {e}", parent.display()))?;
    }
    for request in requests {
        if request.batch_id.is_empty()
            || !request
                .batch_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(format!(
                "ScopedWitnessBatchId `{}` is not transport-safe for the worker request",
                request.batch_id
            ));
        }
    }
    let body = serde_json::to_string_pretty(requests)
        .map_err(|e| format!("serialize scoped execution requests: {e}"))?;
    fs::write(path, body).map_err(|e| format!("write scoped requests {}: {e}", path.display()))
}

fn read_scoped_execution_requests() -> Result<Vec<ScopedExecutionRequest>, String> {
    let path = Path::new(SCOPED_EXECUTION_REQUESTS_PATH);
    let body = fs::read_to_string(path)
        .map_err(|e| format!("read scoped requests {}: {e}", path.display()))?;
    let requests: Vec<ScopedExecutionRequest> =
        serde_json::from_str(&body).map_err(|e| format!("parse scoped requests: {e}"))?;
    refuse_duplicate_scoped_batch_ids(&requests, &path.display().to_string())?;
    Ok(requests)
}

/// A batch id addresses exactly one frozen population, so a repeated id is an ambiguity, not an
/// ordering question. It is refused at the READ rather than at either consumer, because the
/// coordinator spawns one child per row and would otherwise turn one contradiction into N parallel
/// workers racing on one batch's outputs. The manifest this carrier replaces carried the same
/// refusal; dropping it on the way through would have been a silent widen (DESIGN §5).
fn refuse_duplicate_scoped_batch_ids(
    requests: &[ScopedExecutionRequest],
    located: &str,
) -> Result<(), String> {
    for (index, request) in requests.iter().enumerate() {
        if requests[..index]
            .iter()
            .any(|earlier| earlier.batch_id == request.batch_id)
        {
            return Err(format!(
                "scoped requests {located} contain duplicate batch id `{}` — refused",
                request.batch_id
            ));
        }
    }
    Ok(())
}

/// Select and VERIFY one published request. Every refusal is typed and located: a child that
/// cannot prove it was handed the work it is about to do must stop, never fall back to
/// reconstructing the plan for itself — that fallback is the boundary this carrier deletes, and it
/// would sit exactly where it is least observable.
fn scoped_execution_request_for(batch_id: &str) -> Result<ScopedExecutionRequest, String> {
    let requests = read_scoped_execution_requests()?;
    let matching: Vec<&ScopedExecutionRequest> =
        requests.iter().filter(|r| r.batch_id == batch_id).collect();
    if matching.len() != 1 {
        return Err(format!(
            "scoped execution request for batch `{batch_id}` resolved to {} rows; expected exactly one",
            matching.len()
        ));
    }
    let request = matching[0].clone();
    let (tested_commit, tested_tree) = v1_compiler::cli_run::floor_tested_commit_and_tree()?;
    let tool_identity = v1_compiler::cli_run::floor_tool_identity()?;
    refuse_subject_mismatch(&request, &tested_commit, &tested_tree, &tool_identity)?;
    Ok(request)
}

/// The subject comparison, separated from the observation that supplies it so a control can drive
/// the production decision rather than restate its field comparisons (review 51445). Observing the
/// live commit, tree and tool is git and filesystem work; deciding whether they match a frozen
/// request is not, and only the second is what refuses.
fn refuse_subject_mismatch(
    request: &ScopedExecutionRequest,
    tested_commit: &str,
    tested_tree: &str,
    tool_identity: &str,
) -> Result<(), String> {
    if request.tested_commit != tested_commit
        || request.tested_tree != tested_tree
        || request.tool_identity != tool_identity
    {
        return Err(format!(
            "scoped execution request for batch `{}` was frozen against a different subject \
             (request {}/{}/{}, observed {tested_commit}/{tested_tree}/{tool_identity}) — \
             execution refused",
            request.batch_id, request.tested_commit, request.tested_tree, request.tool_identity,
        ));
    }
    Ok(())
}

/// Whether a worker executes witness rows, and whether it must derive a witness roster.
///
/// These are two questions and were one flag until the scoped-child boundary deletion. A scoped
/// child EXECUTES rows — so it needs the per-witness eval budget — but must NOT walk a roster,
/// because its entries were frozen by the ordinary worker and re-deriving them is the duplicate
/// selection the boundary removes. Collapsing them drops the child's eval deadline while looking
/// like a pure scope narrowing, which is exactly the mistake `witness_walk_flags_split_the_two_questions`
/// pins.
struct WitnessWalkFlags {
    executes_witness_rows: bool,
    schedules_discovery: bool,
}

fn witness_walk_flags(carries_witness_rows: bool, is_scoped_child: bool) -> WitnessWalkFlags {
    WitnessWalkFlags {
        executes_witness_rows: carries_witness_rows,
        schedules_discovery: carries_witness_rows && !is_scoped_child,
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
    let mut floor_worker_role: Option<FloorWorkerRole> = None;
    let mut scoped_batch_id: Option<String> = None;
    let mut required_floor_mode = false;
    let mut required_ci_mode = false;
    let mut required_v2_emission_mode = false;
    let mut required_v2_emission_selftest_mode = false;
    let mut corpus_type_judgment_mode = false;
    let mut required_regen_mode = false;
    let mut required_regen_fixed_point_mode = false;
    let mut heads_reading_differential_mode = false;
    let mut behavioral_receipt_plan_mode = false;
    let mut behavioral_receipt_selftest_mode = false;
    let mut behavioral_receipt_census_mode = false;
    let mut regen_candidate_dir = "target/stage0-regen-candidate".to_string();
    let mut regen_receipt_path = "target/stage0-regen-receipt.json".to_string();

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
            "--required-floor" => {
                required_floor_mode = true;
            }
            "--required-ci" => {
                required_ci_mode = true;
            }
            "--required-v2-emission" => {
                required_v2_emission_mode = true;
            }
            "--required-v2-emission-selftest" => {
                required_v2_emission_selftest_mode = true;
            }
            "--corpus-type-judgment" => {
                corpus_type_judgment_mode = true;
            }
            "--required-regen" => {
                required_regen_mode = true;
            }
            "--heads-reading-differential" => {
                heads_reading_differential_mode = true;
            }
            "--required-regen-fixed-point" => {
                required_regen_fixed_point_mode = true;
            }
            "--behavioral-receipt-plan" => {
                behavioral_receipt_plan_mode = true;
            }
            "--behavioral-receipt-selftest" => {
                behavioral_receipt_selftest_mode = true;
            }
            "--behavioral-receipt-census" => {
                behavioral_receipt_census_mode = true;
            }
            "--regen-candidate-dir" => {
                i += 1;
                regen_candidate_dir = require_value(&args, i, "--regen-candidate-dir")?;
            }
            "--regen-receipt" => {
                i += 1;
                regen_receipt_path = require_value(&args, i, "--regen-receipt")?;
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
            "--floor-worker-role" => {
                i += 1;
                let role = require_value(&args, i, "--floor-worker-role")?;
                floor_worker_role = match role.as_str() {
                    "ordinary" => Some(FloorWorkerRole::Ordinary),
                    "scoped" => Some(FloorWorkerRole::Scoped {
                        batch_id: String::new(),
                    }),
                    _ => {
                        eprintln!("claim_executor: invalid --floor-worker-role `{role}`");
                        return Err(ExitCode::from(2));
                    }
                };
            }
            "--scoped-batch-id" => {
                i += 1;
                scoped_batch_id = Some(require_value(&args, i, "--scoped-batch-id")?);
            }
            other => {
                eprintln!("claim_executor: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. })) {
        let Some(batch_id) = scoped_batch_id else {
            eprintln!("claim_executor: scoped floor worker requires --scoped-batch-id");
            return Err(ExitCode::from(2));
        };
        floor_worker_role = Some(FloorWorkerRole::Scoped { batch_id });
    } else if scoped_batch_id.is_some() {
        eprintln!("claim_executor: --scoped-batch-id requires --floor-worker-role scoped");
        return Err(ExitCode::from(2));
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

    // ── V2 EMISSION: TWO STANDALONE ENTRY POINTS BESIDE THE REQUIRED PHASE ──────
    //
    // The v2-emission transaction IS ENROLLED in `--required-ci` as phase 3 (see that
    // mode below), on an operator ruling relayed through the requesting session
    // 2026-08-23, at a measured +135s against the floor's ~30-40 minutes. These two
    // flags are not an opt-in alternative to that phase and must not be read as one:
    // they exist because running the emission alone, or running only its red/green
    // evidence, are real local actions -- the same reason the `src/v1` .dag parse sweep
    // keeps its own bin beside its required phase.
    //
    // This comment said "DELIBERATELY NOT ENROLLED" in the revision that ADDED the
    // enrolment, which is the premise contamination DESIGN warns about rather than a
    // stale comment: a reader grepping here would conclude the phase is opt-in when it
    // is required. It is rewritten rather than annotated, because two accounts of one
    // fact is what produced the contradiction.
    //
    // Both flags run the SAME producer the required phase runs
    // (`cli_run::compile_entry_emission`), so a green here and a green there cannot be
    // different facts.
    if required_v2_emission_selftest_mode {
        let failures = v1_compiler::cli_run::run_required_v2_emission_selftest();
        for failure in &failures {
            eprintln!("required-v2-emission-selftest: FAIL {failure}");
        }
        return if failures.is_empty() {
            eprintln!(
                "required-v2-emission-selftest: OK red fixture refused on the annotation cause, green fixture emitted"
            );
            Ok(ExitCode::SUCCESS)
        } else {
            Err(ExitCode::from(1))
        };
    }

    // ORDERED AHEAD OF THE SOURCE-ROOT REQUIREMENT, and that ordering is load-bearing rather
    // than cosmetic: this mode takes NO roots, so the generic "provide at least one --source-root"
    // guard below would refuse the only invocation it has. Measured, not reasoned: the emitted
    // workflow's exact command line exited 2 with that message before this branch was moved here,
    // which is a mode that could never have run once. It short-circuits for the same reason
    // `--verify-build-artifacts` and `--measure-cgroup-peak` do -- a mode with no subject argument
    // cannot be gated on a subject argument.
    // THE CORPUS TYPE-JUDGMENT MEASUREMENT — the required run's preparation, stopped before it
    // judges.
    //
    // It reaches for `corpus_type_judgment_census`, which assembles THE SAME subject
    // `--required-ci`'s floor phase prepares, over the SAME source roots, and runs the SAME
    // whole-corpus compile — then reports the located judgment population instead of collapsing it
    // to pass/fail. Nothing is evaluated: no claim scope, no interpreter context, no witness.
    //
    // THIS MODE TAKES NO SUBJECT ARGUMENT, and that is the design rather than an omission. The
    // roots come from the `gunbc.ci_layer_roots` authority, so `--source-root` is REFUSED here
    // instead of honoured: every instrument this lane has watched fail measured some closure and
    // reported it as the corpus, and a scoping flag only moves that error onto whoever forgets it.
    // There is no spelling of "measure part of it", so there is nothing to forget.
    //
    // WHY IT EXISTS. The strict gate asks that compile exactly one question — is anything blocking
    // — and discards the rest. So the advisory residue (the method-existence frontier and its
    // siblings) is computed on every required run and counted by nothing, which is the state
    // DESIGN §4b names: a frontier whose deficit frequency is unobservable never ranks for
    // climbing.
    //
    // WHAT ITS EXIT CODE MEANS, because a measurement and a gate must not be read as each other:
    // 0 means MEASURED, whatever the corpus turned out to say; nonzero means COULD NOT MEASURE.
    // A blocking count above zero is a reported figure, not this mode's failure — the gate that
    // stops the line on it is `--required-ci`, and duplicating that verdict here would be a second
    // authority for the same fact.
    if corpus_type_judgment_mode {
        if !source_roots.is_empty() {
            eprintln!(
                "corpus-type-judgment: CORPUS-TYPE-JUDGMENT REFUSAL cause=SubjectSupplied — this \
                 mode measures the corpus the required run compiles and derives its roots from \
                 gunbc.ci_layer_roots; --source-root is refused rather than honoured, because a \
                 scopeable corpus measurement is how a closure gets reported as the corpus"
            );
            return Err(ExitCode::from(2));
        }
        let census = match v1_compiler::cli_run::corpus_type_judgment_census() {
            Ok(c) => c,
            Err(e) => {
                // A REFUSAL PRINTS NO COUNT. The one thing this mode must never do is answer a
                // question it could not measure with the number that reads as good news.
                eprintln!("corpus-type-judgment: {e}");
                return Err(ExitCode::from(1));
            }
        };
        // The plant prints FIRST and unconditionally: it is what makes every number below it
        // readable, so it must not be something a reader has to scroll past a zero to find.
        for site in &census.planted_sites {
            println!(
                "CORPUS_JUDGMENT_PLANT\t{}\t{}\t{}\t{}",
                site.identity(),
                site.severity.as_str(),
                site.line,
                site.col
            );
        }
        println!("CORPUS_JUDGMENT_SUBJECT\t{}", census.subject_statement());
        println!(
            "CORPUS_JUDGMENT_PARTITION\tblocking={}\tadvisory={}\tunclassified={}\tsites={}",
            census.blocking_total,
            census.advisory_total,
            census.unclassified_total,
            census.sites.len()
        );
        // THE PARTITIONS DO NOT CARRY THE SAME EVIDENCE, and saying so is not a caveat — it is the
        // rung each one occupies (DESIGN §4b, rung honesty at a declared grain). The blocking
        // partition has a second observer: required-ci prints its own located diagnostics when
        // strict preparation refuses, so these identities can be joined against it on the same
        // head. The advisory partition has NO independent observer anywhere in the repository,
        // because the gate that computes it discards it. Reporting both as one number would carry
        // the joined partition's standing over to the unjoined one.
        println!(
            "CORPUS_JUDGMENT_EVIDENCE\tblocking=joinable-against-required-ci\t\
             advisory=no-independent-observer\tnext-trigger=a-second-observer-of-the-advisory-partition"
        );
        for site in &census.sites {
            println!(
                "CORPUS_JUDGMENT_SITE\t{}\t{}\t{}\t{}\t{}",
                site.identity(),
                site.severity.as_str(),
                site.line,
                site.col,
                site.module
            );
        }
        // `unclassified` is not a rounding bucket. A class counted by neither severity predicate is
        // a frontier that renders to a terminal while every gate reports zero of it, so it is named
        // on its own line rather than left to a reader to notice in a column.
        if census.unclassified_total > 0 {
            eprintln!(
                "corpus-type-judgment: {} judgment(s) are admitted by NEITHER the blocking \
                 predicate nor the advisory allowlist — each is a judgment the corpus produced \
                 that no severity policy counts",
                census.unclassified_total
            );
        }
        // TERMINAL COMPLETION, printed only here. A run that refused above never reaches this
        // line, so the presence of the line is what distinguishes a whole population from a
        // truncated one — a consumer that reads rows without finding it is reading a partial run.
        println!(
            "CORPUS_JUDGMENT_TERMINAL\tcomplete\tplanted_observed={}\t{}",
            census.planted_sites.len(),
            census.subject_statement()
        );
        eprintln!("corpus-type-judgment: measured, no witness evaluated");
        return Ok(ExitCode::SUCCESS);
    }

    if source_roots.is_empty() {
        eprintln!("claim_executor: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let _phase_profile = PhaseProfile::install_from_env();

    // THE REQUIRED WITNESS FLOOR: one repository preparation, one immutable scope per distinct
    // claim scope, one cheap mutable frame per claim, one linear fold over every witness in the
    // tree. It takes no plan, so it short-circuits before the plan-arg requirement — there is no
    // schedule to resolve, no batch to assign, no worker to spawn and no selection to compute,
    // and the absence of those flags is the point rather than an omission. `run_required_floor`
    // refuses when planned, executed and terminal identity counts disagree, so a silently short
    // roster cannot report as a pass.
    if behavioral_receipt_census_mode {
        return run_behavioral_receipt_census(&source_roots);
    }

    if behavioral_receipt_selftest_mode {
        return run_behavioral_receipt_selftest(&source_roots);
    }

    if behavioral_receipt_plan_mode {
        return run_behavioral_receipt_plan(&source_roots);
    }

    // THE COMPOSED CI RUN — one process, three phases: the src/v1 .dag parse sweep, the regen
    // first-generation comparison, and the witness floor.
    //
    // WHAT IT IS AND IS NOT. Sequencing a program's phases is the program's job (DESIGN §3: the
    // workflow is a realization of the intent, not the place the intent lives), so the order
    // lives here rather than in a YAML step list whose preconditions read each other's
    // `outcome`. What is NOT here is a judgement about which checks CI ought to run: the roster
    // is an operator decision, and the 2026-08-21 ruling set it to exactly these three.
    //
    // WHAT WAS DELETED AND WHY IT IS NOT LEFT AS A SKIPPED PHASE (operator ruling, 2026-08-21).
    // Five phases previously ran inside this fold: merge-admission-capture, regen-determinism
    // (the fixed point), receipt-selftest, receipt-vs-changed-authorities, and
    // merge-admission-stamp. They are GONE from this mode — not disabled behind a flag, not
    // reported as SKIPPED — because a phase that always reports the same non-verdict is the
    // absorbing fallback wearing a phase's clothes (DESIGN §5): its deficit frequency is zero by
    // construction and it reads as coverage on the ledger. The capabilities themselves survive
    // where they had their own entry points and consumers: `--required-regen-fixed-point`,
    // `--behavioral-receipt-selftest`, `--behavioral-receipt-plan` and
    // `--behavioral-receipt-census` are unchanged and still run standalone. What ends is their
    // enrolment in the required run, and that is the whole of the change.
    //
    // WHAT THAT COSTS, named rather than left to be rediscovered. No CI run now measures regen
    // DETERMINISM (that the emitter reproduces its own output), behavioural equivalence of a
    // changed authority against its committed mirror, or the receipt machinery's own
    // discriminating arms; and no run mints a merge-admission receipt. The merge-admission
    // consumers refuse on a missing receipt rather than admitting on one (DESIGN, the CI
    // paragraph's re-add queue), so nothing is admitted by the absence — but the three
    // measurements above are simply not taken, which is a declared rung drop, not a silent one.
    //
    // WHAT THE ORDER IS, AND WHY EACH PHASE RUNS ANYWAY. The four phases are independent —
    // the one real data dependency, the fixed point's need for regen's pass-1 digest, went with
    // the phase that consumed it — so every phase RUNS EVEN AFTER AN EARLIER FAILURE and the run
    // reports the complete ledger instead of letting the first defect hide the rest. The line
    // still stops (a nonzero exit on any failed phase); it stops with every deficit named. This
    // is the stopped-line AUDIT DESIGN §5 sanctions: it reports, it never greens.
    if required_ci_mode {
        let mut phase_failures: Vec<String> = Vec::new();
        let mut ran: Vec<&'static str> = Vec::new();

        // PHASE 1 — the .dag parse sweep, over every authored root (src/v1, dag, src/v2).
        // Independent of everything below it. The roster is
        // `cli_run::DAG_PARSE_SWEEP_ROOTS`, shared with the standalone bin so the cheapest
        // local check and this phase cover the same files.
        eprintln!(
            "required-ci: phase parse (.dag: {})",
            v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS.join(", ")
        );
        match v1_compiler::cli_run::run_dag_parse_sweep(
            &v1_compiler::cli_run::workspace_root(),
            &v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS,
        ) {
            Ok(sweep) => {
                eprintln!(
                    "required-ci: parse OK {} file(s) parse-clean",
                    sweep.parse_clean
                );
                // THE DECLARATION INTEGRITY CHECKS RIDE THE PARSE THAT JUST RAN.
                //
                // They are reported inside this phase rather than as a phase of their own
                // because they are not a second pass over anything: the index was built by
                // insertion from the sweep above, so there is no walk to order, nothing to
                // schedule, and no second acquisition of the corpus. DESIGN §6 and §3's
                // cited-symbol row both name exactly this — one module's facts from one
                // module's source, at ingestion, instead of a corpus-wide job per question.
                let population =
                    v1_compiler::cli_run::declaration_index::index_population(&sweep.index);
                let findings =
                    v1_compiler::cli_run::declaration_index::index_findings(&sweep.index);
                // A GREEN NAMES ITS DENOMINATORS. `checked=0` and `all clean` are different
                // states with different remedies, and an instrument that renders them
                // identically is the failure DESIGN §5 names, not a tidy report.
                eprintln!(
                    "required-ci: declarations modules={} declared={} import_members={} \
                     citations={} debt={} in_fixtures={} outside_index={} lens_modules={}",
                    population.modules,
                    population.declarations,
                    population.import_members,
                    population.citations,
                    population.citations_pre_existing_debt,
                    population.citations_in_fixtures,
                    population.citations_outside_index,
                    population.lens_modules,
                );
                for finding in &findings {
                    eprintln!(
                        "required-ci: declarations FAIL {}",
                        v1_compiler::cli_run::declaration_index::render_finding(
                            &v1_compiler::cli_run::workspace_root(),
                            finding
                        )
                    );
                }
                if !findings.is_empty() {
                    phase_failures.push(format!("declarations ({} finding(s))", findings.len()));
                }
            }
            Err(errors) => {
                for e in &errors {
                    eprintln!("required-ci: parse FAIL {e}");
                }
                phase_failures.push(format!("parse ({} error(s))", errors.len()));
            }
        }
        ran.push("parse");

        // PHASE 2 — regen first generation: the emitted mirrors against what is committed.
        eprintln!("required-ci: phase regen (first generation vs committed)");
        match v1_compiler::cli_run::run_required_regen(&regen_candidate_dir, &regen_receipt_path) {
            Ok(outcome) => {
                // Read through accessors, and print `unmeasured` rather than a plausible
                // default when the pass built the wrong variant (#8650's shape).
                let fge = outcome
                    .receipt
                    .first_generation_equal()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unmeasured".to_string());
                let candidate = outcome.receipt.candidate_artifact().unwrap_or("unmeasured");
                eprintln!("required-ci: regen first_generation_equal={fge} candidate={candidate}");
                for failure in &outcome.failures {
                    eprintln!("required-ci: regen FAIL {failure}");
                }
                if !outcome.failures.is_empty() {
                    phase_failures.push(format!("regen ({} failure(s))", outcome.failures.len()));
                }
            }
            Err(e) => {
                // A REFUSAL IS NOT A MISMATCH: nothing was emitted, so there is no comparison
                // to report, and the refusal is named rather than folded into a drift verdict.
                eprintln!("required-ci: regen refused: {e}");
                phase_failures.push(format!("regen refused: {e}"));
            }
        }
        ran.push("regen");

        // PHASE 3 — v2 emission. ENROLLED 2026-08-23 on an operator ruling relayed through
        // the requesting session, after the 2026-08-23 break reached main and stayed for
        // hours with every required phase green: the required run parses src/v1 .dag,
        // compares the regen mirrors and folds the floor, and NONE OF THE THREE COMPILES A
        // v2 ENTRY. Measured cost +135s against the floor's ~30-40 minutes.
        //
        // ORDERED AHEAD OF THE FLOOR, and the ordering is a report-order fact only: the
        // phases stay independent and every one runs even after an earlier failure, so
        // this does not stop the floor starting on an unemittable tree. Making it an early
        // PREREQUISITE (an exit before the floor) is a scheduling decision that belongs to
        // the operator with the number attached, and it would change this mode's
        // stopped-line audit design, so it is not taken here.
        //
        // The subject is the SAME PRODUCER the cargo board runs
        // (cli_run::compile_entry_emission, which `gunbc compile --entry` also calls), so
        // a green here and an emitting board are one fact rather than two.
        eprintln!("required-ci: phase v2-emission (one entry, the board's producer)");
        match v1_compiler::cli_run::run_required_v2_emission(&source_roots) {
            Ok(runs) => {
                let mut not_completed = 0usize;
                for run in &runs {
                    eprintln!("{}", run.measurement_line("required-ci: v2-emission"));
                    match &run.disposition {
                        v1_compiler::cli_run::EntryEmissionDisposition::Completed { .. } => {}
                        v1_compiler::cli_run::EntryEmissionDisposition::Refused {
                            phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-ci: v2-emission EmissionRefused entry={} phase={phase} cause={cause}",
                                run.entry
                            );
                        }
                        v1_compiler::cli_run::EntryEmissionDisposition::NotExecuted {
                            earlier_phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-ci: v2-emission EmissionNotExecuted entry={} earlier_phase={earlier_phase} cause={cause}",
                                run.entry
                            );
                        }
                    }
                }
                if not_completed > 0 {
                    phase_failures.push(format!("v2-emission ({not_completed} not completed)"));
                }
            }
            Err(e) => {
                eprintln!(
                    "required-ci: v2-emission EmissionNotExecuted earlier_phase=roster cause={e}"
                );
                phase_failures.push(format!("v2-emission roster: {e}"));
            }
        }
        ran.push("v2-emission");

        // PHASE 4 — the witness floor. Independent; runs whatever happened above.
        eprintln!("required-ci: phase floor (one prepared subject, one fold)");
        let commit = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());
        match v1_compiler::cli_run::run_required_floor(
            &source_roots,
            &commit,
            v1_compiler::cli_run::ShardStyle::single_shard(),
        ) {
            Ok(outcome) => {
                report_required_floor_outcome(&outcome);
                if !required_floor_outcome_is_clean(&outcome) {
                    phase_failures.push("floor".to_string());
                }
            }
            Err(e) => {
                eprintln!("required-ci: floor refused: {e}");
                phase_failures.push(format!("floor refused: {e}"));
            }
        }
        ran.push("floor");

        eprintln!(
            "required-ci: phases_run={} failed={}",
            ran.len(),
            phase_failures.len()
        );
        for failure in &phase_failures {
            eprintln!("required-ci: FAILED PHASE {failure}");
        }
        return if phase_failures.is_empty() {
            Ok(ExitCode::SUCCESS)
        } else {
            Err(ExitCode::from(1))
        };
    }

    if required_v2_emission_mode {
        let roots = if source_roots.is_empty() {
            v1_compiler::cli_run::witness_layer_roots()
        } else {
            source_roots.clone()
        };
        return match v1_compiler::cli_run::run_required_v2_emission(&roots) {
            Ok(runs) => {
                let mut not_completed = 0usize;
                for run in &runs {
                    // ONE SELF-DESCRIBING LINE. The disposition is ON the line, so a run
                    // that never reached the compiler cannot be read as a clean emission
                    // of nothing by anything that reads one line at a time.
                    eprintln!("{}", run.measurement_line("required-v2-emission"));
                    match &run.disposition {
                        v1_compiler::cli_run::EntryEmissionDisposition::Completed { .. } => {}
                        v1_compiler::cli_run::EntryEmissionDisposition::Refused {
                            phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionRefused entry={} phase={phase} cause={cause}",
                                run.entry
                            );
                        }
                        v1_compiler::cli_run::EntryEmissionDisposition::NotExecuted {
                            earlier_phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionNotExecuted entry={} earlier_phase={earlier_phase} cause={cause}",
                                run.entry
                            );
                        }
                    }
                }
                eprintln!(
                    "required-v2-emission: entries={} not_completed={}",
                    runs.len(),
                    not_completed
                );
                if not_completed == 0 {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            // The roster itself being unreadable is reported as itself: no entry ran, so
            // there is no per-entry disposition to render.
            Err(e) => {
                eprintln!(
                    "required-v2-emission: EmissionNotExecuted earlier_phase=roster cause={e}"
                );
                Err(ExitCode::from(1))
            }
        };
    }

    if required_regen_fixed_point_mode {
        return match v1_compiler::cli_run::run_required_regen_fixed_point(&regen_receipt_path, None)
        {
            Ok(outcome) => {
                // The provenance is printed, not just carried. This line previously read
                // `first_generation_equal={}` off the receipt as though the fixed-point pass had
                // measured it; it never does. Labelling it `referenced_` and naming the commit it
                // came from means the log itself distinguishes measured from quoted -- and since
                // the host refuses a cross-tree reference, `referenced_at` equals HEAD on every
                // line that is allowed to print.
                let (referenced_fge, referenced_at) = match outcome.receipt.prior() {
                    Some(prior) => (
                        prior.first_generation_equal.to_string(),
                        prior.commit_sha.clone(),
                    ),
                    None => ("unavailable".to_string(), "unavailable".to_string()),
                };
                eprintln!(
                    "required-regen-fixed-point: fixed_point_equal={} referenced_first_generation_equal={} referenced_at={}",
                    outcome
                        .receipt
                        .fixed_point_equal()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unmeasured".to_string()),
                    referenced_fge,
                    referenced_at
                );
                for failure in &outcome.failures {
                    eprintln!("required-regen-fixed-point: FAIL {failure}");
                }
                if outcome.failures.is_empty() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("required-regen-fixed-point: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    // THE CITED-SYMBOL CENSUS IS GONE FROM HERE, AND ITS SUBJECT MOVED RATHER THAN LAPSING.
    //
    // `--required-cited-symbol` ran `v2.lens.cited_symbol_resolution` over the corpus-wide
    // `decl_facts` + `module_declaration_facts` walks, answering each authored reference by a
    // LINEAR SCAN of a flat list of every declaration in the repository. DESIGN §3's rung-drop
    // row names exactly that shape as the thing to stop doing: the wall belongs "checked at
    // ingestion, on the module whose source carries the citation, from that module's own text,
    // rather than reconstructed corpus-wide by a second job".
    //
    // It is now the `parse` phase above, over the per-module declaration index the sweep builds
    // from the parse it was already performing. The mode is deleted rather than left standing as
    // a second route to one question (§3, single authority), and the replacement is STRICTLY
    // WIDER, which is the test §3's replacement doctrine sets: it enrolls every authored
    // `DeclarationRef` in the corpus rather than the five carriers the lens's population named,
    // and it indexes test modules, which `decl_facts` deliberately did not — so the
    // outside-index disposition those exclusions forced is not needed at all.

    // The heads reading's own instrument. It is not enrolled in the required run and this
    // clause does not pretend otherwise: reading 3875 modules TWICE is precisely the cost
    // the heads reading exists to remove, so paying it on every push would spend more than
    // the repair saves. It is a re-runnable receipt — the differential this change was
    // landed on can be re-taken by anyone, on any tree, rather than being a number quoted
    // from a run nobody can repeat.
    if heads_reading_differential_mode {
        let roots = if source_roots.is_empty() {
            vec!["dag".to_string(), "src/v2".to_string()]
        } else {
            source_roots.clone()
        };
        let d = v1_compiler::cli_run::heads_reading_differential(&roots);
        eprintln!(
            "heads-reading-differential: compared={} divergent={} narrowed={} regressed={} both_refused={}",
            d.modules_compared,
            d.divergent.len(),
            d.narrowed.len(),
            d.regressed.len(),
            d.both_refused.len(),
        );
        // The two readings' summed parse wall, from the same process over the same
        // modules. This is the PARSE term only — not the whole `pool_parse` row, which
        // also pays tokenize, newline indexing and per-file setup that neither reading
        // changes.
        eprintln!(
            "heads-reading-differential: full_reading_parse_ms={} heads_reading_parse_ms={}",
            d.full_reading_nanos / 1_000_000,
            d.heads_reading_nanos / 1_000_000,
        );
        for path in d.divergent.iter().take(20) {
            eprintln!("heads-reading-differential: DIVERGENT {path}");
        }
        for path in d.regressed.iter().take(20) {
            eprintln!("heads-reading-differential: REGRESSED {path}");
        }
        for path in d.narrowed.iter().take(20) {
            eprintln!("heads-reading-differential: narrowed (declared scope) {path}");
        }
        return if d.holds() {
            Ok(ExitCode::SUCCESS)
        } else {
            Err(ExitCode::from(1))
        };
    }

    if required_regen_mode {
        return match v1_compiler::cli_run::run_required_regen(
            &regen_candidate_dir,
            &regen_receipt_path,
        ) {
            Ok(outcome) => {
                // Both values here ARE measured by this pass, so they print unqualified. Read
                // through accessors rather than by matching the variant: the
                // `required_regen_host` module is private to `cli_run`, so the type is usable here
                // but not nameable. The accessors return Option because the sibling variant does
                // not measure these fields; a `None` on this path would mean the first pass built
                // the wrong variant, so it prints `unmeasured` rather than defaulting to a
                // plausible-looking value.
                let fge = outcome
                    .receipt
                    .first_generation_equal()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unmeasured".to_string());
                let candidate = outcome
                    .receipt
                    .candidate_artifact()
                    .unwrap_or("unmeasured")
                    .to_string();
                eprintln!("required-regen: first_generation_equal={fge} candidate={candidate}");
                for failure in &outcome.failures {
                    eprintln!("required-regen: FAIL {failure}");
                }
                if outcome.failures.is_empty() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("required-regen: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    if required_floor_mode {
        let commit = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());
        return match v1_compiler::cli_run::run_required_floor(
            &source_roots,
            &commit,
            v1_compiler::cli_run::ShardStyle::single_shard(),
        ) {
            Ok(outcome) => {
                report_required_floor_outcome(&outcome);
                if required_floor_outcome_is_clean(&outcome) {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("required-floor: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }
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
    // naming-hygiene walk and every subsequent corpus read — so `[file] read` /
    // `[rest]` / `[hermetic:mock]` etc. are funnelled per `gunbc.output_policy`
    // (Instrumentation is Suppressed at Normal, the CI default) instead of flooding the
    // floor log. Installing AFTER the walk (the prior order) left the walk's whole-tree
    // read at the `Full` default — ~2.3k `[file] read` lines, the firehose the
    // observation-emit census (`gunbc.observation_emit_census`) targets. That ordering
    // requirement is unchanged by #8140: the walk moved AFTER plan evaluation (it is now
    // demand-directed on `schedules_discovery`), so this install precedes it by even
    // more, and the walk's whole-tree read is still policy-funnelled.
    if let Err(why) = v1_compiler::cli_run::install_output_policy(&source_roots) {
        eprintln!("claim_executor: {why}");
        return Err(ExitCode::from(1));
    }
    // Install the per-target group-marker syntax (GitHub Actions `::group::` vs a
    // plain-terminal header) from the .dag authority, so the parallel walk folds each
    // batch's host-effect traces into a collapsible group.
    v1_compiler::cli_run::install_group_syntax(&source_roots);
    phase_mark("output-policy + group-syntax install");

    // The walk-attempt id is a tracing coordinate every later phase stamps, so it is
    // minted unconditionally here. The corpus WALK it used to gate is not: see the
    // demand-directed hygiene walk after the plan's batches settle.
    let floor_walk_attempt_id_value = match floor_walk_attempt_id() {
        Ok(id) => id,
        Err(msg) => {
            eprintln!("claim_executor: witness naming hygiene walk-attempt refusal: {msg}");
            return Err(ExitCode::from(1));
        }
    };
    if std::env::var("GUNBC_FLOOR_WALK_ATTEMPT_ID")
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        std::env::set_var("GUNBC_FLOOR_WALK_ATTEMPT_ID", &floor_walk_attempt_id_value);
    }

    if perturb_check {
        return run_perturb_check(&source_roots, &plan_entry, &plan_function);
    }

    // A SCOPED CHILD DOES NOT RESOLVE OR EVALUATE THE ORDINARY PLAN. Its work arrives as a
    // published, subject-verified `ScopedExecutionRequest` — the batch the ordinary worker already
    // froze, plus the plan-derived budgets the child reads. Re-resolving the plan entry and
    // re-evaluating it per child, only to scan the result for one batch id and discard every other
    // value behind a `!Scoped` guard, is the duplicate prelude this boundary deletes.
    //
    // There is no fallback arm. A child that cannot load or verify its request refuses; falling
    // back to reconstructing the plan would restore the boundary in the failure case, where it is
    // least observable.
    let scoped_request: Option<ScopedExecutionRequest> = match &floor_worker_role {
        Some(FloorWorkerRole::Scoped { batch_id }) => {
            match scoped_execution_request_for(batch_id) {
                Ok(request) => Some(request),
                Err(msg) => {
                    eprintln!("claim_executor: scoped floor worker request refusal: {msg}");
                    let _ = write_floor_worker_terminal("refused", &msg);
                    return Err(ExitCode::from(1));
                }
            }
        }
        _ => None,
    };

    let (plan_ctx, walk_plan) = if let Some(request) = &scoped_request {
        phase_mark("scoped request verified");
        (
            None,
            ParsedWalkPlan {
                pre_walk_execution: PreWalkExecution::None,
                batches: vec![vec![request.to_runnable()]],
                finalization: None,
                on_success_stages: Vec::new(),
                ordinary_budget_ms: request.ordinary_budget_ms,
                on_success_budget_ms: None,
            },
        )
    } else {
        // Resolve the plan entry ONCE and evaluate both the batches (hermetic) and the
        // spawn width (wet) from the same resolved graph — this resolve was previously
        // paid twice back-to-back (the §2 double-paid-compute trap, at minutes each).
        let (plan_graph, plan_indices) =
            match resolve_entry_graph_shared(&source_roots, &plan_entry) {
                Ok(resolved) => resolved,
                Err(msg) => {
                    eprintln!("claim_executor: resolve failed for plan {plan_entry}:\n{msg}");
                    return Err(ExitCode::from(1));
                }
            };
        phase_mark("plan resolve");

        let plan_ctx =
            make_eval_context(&plan_graph, plan_indices.clone(), ExecutionMode::Hermetic);
        let walk_plan = match eval_plan_in_ctx(&plan_ctx, &plan_entry, &plan_function) {
            Ok(b) => b,
            Err(msg) => {
                eprintln!("claim_executor: {msg}");
                return Err(ExitCode::from(1));
            }
        };
        (Some(plan_ctx), walk_plan)
    };

    // Every remaining plan read belongs to a path a scoped child never takes. Rather than unwrap
    // an Option ten times, ask once and refuse in the child's voice if that ever stops being true:
    // a missing plan context is a routing error, not a value to substitute a default for.
    macro_rules! plan_ctx_or_refuse {
        ($what:expr) => {
            match plan_ctx.as_ref() {
                Some(ctx) => ctx,
                None => {
                    // WRITE THE TERMINAL. Without it the coordinator observes only `exited:1` and
                    // reports "worker returned before producing a walk terminal receipt" — a
                    // located refusal rendered as an unlocated absence, which is what hid this
                    // exact defect through two full CI floors.
                    let msg = format!(
                        "{} requires the ordinary plan context, which a scoped child does not have (fail-closed)",
                        $what
                    );
                    eprintln!("claim_executor: {msg}");
                    let _ = write_floor_worker_terminal("refused", &msg);
                    return Err(ExitCode::from(1));
                }
            }
        };
    }

    let pre_walk_execution = walk_plan.pre_walk_execution;
    let ordinary_budget_ms = walk_plan.ordinary_budget_ms;
    let on_success_budget_ms = walk_plan.on_success_budget_ms;
    let mut batches = walk_plan.batches;
    let mut on_success_stages = walk_plan.on_success_stages;
    let mut floor_finalization: Option<FloorFinalization> = walk_plan.finalization;
    // Captured BEFORE the scoped-worker override below can overwrite `floor_finalization`
    // to `None` for its own, different reason — this is the plan's own fact, not the
    // role's, and the two must not be read back through one collapsed `Option` later
    // (review 49917, cursor/composer-2.5).
    let plan_declared_no_finalization = floor_finalization.is_none();
    let mut published_scoped_rows: Vec<Runnable> = Vec::new();
    match floor_worker_role.as_ref() {
        Some(FloorWorkerRole::Ordinary) => {
            // The child rows leave `batches` on the retain below, so capture their exact payload
            // first. Publication waits until the plan-derived budgets are known — a request
            // missing them would force the child to evaluate the plan for the very values this
            // carrier exists to hand it.
            // Clone the SCOPED rows only. The publishable population is a small handful of
            // batches; cloning every runnable in the plan and filtering afterwards copies the
            // whole corpus roster to keep a few rows, which is the cost shape §6 forbids
            // independently of how large the plan happens to be today.
            published_scoped_rows = batches
                .iter()
                .flatten()
                .filter(|runnable| {
                    matches!(
                        runnable,
                        Runnable::ScopedWitnessBatch {
                            process_isolation: ScopedProcessIsolation::SequentialChildProcess
                                | ScopedProcessIsolation::FreshJobProcess,
                            ..
                        }
                    )
                })
                .cloned()
                .collect();
            batches.retain(|batch| {
                !batch.iter().any(|runnable| {
                    matches!(
                        runnable,
                        Runnable::ScopedWitnessBatch {
                            process_isolation: ScopedProcessIsolation::SequentialChildProcess
                                | ScopedProcessIsolation::FreshJobProcess,
                            ..
                        }
                    )
                })
            });
        }
        Some(FloorWorkerRole::Scoped { batch_id }) => {
            // The batch comes from the VERIFIED request, not from a re-evaluated plan. There is no
            // fallback arm: a child that cannot prove it was handed this work refuses.
            //
            // The request is READ here, never re-loaded. A second `scoped_execution_request_for`
            // would repeat the disk read, the duplicate refusal and the subject verification that
            // already ran above — double-paid compute inside the very boundary this change exists
            // to delete (review 51433). The refusal below is not the verification: it is the
            // structural fact that this arm is reachable only when the earlier one produced a
            // request, refused rather than unwrapped so a future edit that breaks the pairing
            // stops the line instead of panicking.
            // The pairing check stays even though this arm now consumes nothing off the request:
            // reaching execution as a scoped worker with no verified request in hand is the state
            // this whole change exists to make impossible, and it must stop the line where it is
            // observed rather than surface later as a missing value.
            match &scoped_request {
                Some(_) => {}
                None => {
                    let msg = format!(
                        "scoped floor worker for batch `{batch_id}` reached execution with no \
                         verified request in hand — refused"
                    );
                    eprintln!("claim_executor: {msg}");
                    let _ = write_floor_worker_terminal("refused", &msg);
                    return Err(ExitCode::from(1));
                }
            }
            // `batches` already IS this request's single row — it came from the walk plan built
            // above out of the same verified request, so rebuilding it here would be a second
            // construction of one decision (review 51445). What this arm still owes is the
            // surrounding shape: a scoped child runs its batch and nothing else.
            on_success_stages.clear();
            floor_finalization = None;
        }
        None => {}
    }
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
    // Fast-lane 5s rule (operator 2026-07-12): a plan that schedules a discovery batch
    // must declare the per-witness eval budget; a missing/mistyped row refuses the run
    // (fail-closed), while discovery-free plans (regen, plan-artifact) never read it.
    // A SCOPED CHILD NEVER SCHEDULES DISCOVERY. Its batch arrives with `entries` already
    // frozen by the ordinary worker — exact (entry, function) identities — so a roster walk
    // here can only re-derive a selection its parent already made, and re-deriving it is the
    // duplicate work this boundary exists to delete. Naming hygiene is a property of the
    // corpus, not of one child's slice: the ordinary worker pays it once per run, and every
    // PR runs a plan that does. Deleting the child's walk also removes the last caller of
    // `FloorDiscoveryConsumerRole::CoordinatedConsumer`, and with it the whole-graph snapshot
    // transport that existed solely to make that walk's facts acquisition a cache hit.
    let is_scoped_child = matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. }));
    // TWO QUESTIONS, deliberately not one flag. *Does this worker execute witness rows?* decides
    // the per-witness eval budget, and a scoped child does execute them. *Must this worker derive
    // a witness roster?* decides the naming-hygiene walk, and a scoped child must not — its
    // entries are already frozen. They were one predicate until this change, so narrowing the walk
    // would have silently dropped the child's eval deadline with it.
    let carries_witness_rows = batches.iter().flatten().any(|r| {
        matches!(
            r,
            Runnable::DiscoveryBatch { .. } | Runnable::ScopedWitnessBatch { .. }
        )
    });
    let WitnessWalkFlags {
        executes_witness_rows,
        schedules_discovery,
    } = witness_walk_flags(carries_witness_rows, is_scoped_child);
    // Witness naming hygiene (`test fn` outside `*_test.dag`; the `__`-basename rule and
    // the orphan-helper census were deleted in gunbc#8155) is a
    // property of the witness ROSTER, so it is paid by the plans that have one. It ran
    // unconditionally before plan evaluation until this change, on the stated ground
    // that a naming violation should be "the cheapest possible failure"; measured, the
    // walk is the most expensive phase in the process (5.9 min of a 56.5-min floor,
    // ~6 min of a ~15-min regen), because the roster producer it calls builds
    // module-graph facts, a second strict reference-resolution pass, and (until
    // gunbc#8141 deleted them) inert-lens reachability plus the
    // construction-justification census. A two-node regen plan
    // paid all of it to discover a roster it never reads. The roster is memoized by
    // request digest (IN_PROCESS_ROSTER_BY_REQUEST), so plans that DO schedule
    // discovery pay exactly what they paid before — the corpus batch hits the memo
    // this call fills. Plans that do not schedule discovery now pay nothing, and
    // cannot: they have no roster to be unhygienic about.
    if schedules_discovery {
        let excludes = v1_compiler::cli_run::witness_exclusion_substrings();
        // Only a non-scoped worker reaches here (see `schedules_discovery`), so the roster is
        // always produced, never consumed from a transported snapshot.
        let discovery_consumer = FloorDiscoveryConsumerRole::Producer;
        if let Err(msg) = discover_floor_witness_roster_with_snapshot(
            &source_roots,
            &[],
            &excludes,
            &[],
            &floor_walk_attempt_id_value,
            discovery_consumer,
            "Hermetic",
            &source_roots,
        ) {
            eprintln!("claim_executor: witness naming hygiene (roster walk): {msg}");
            return Err(ExitCode::from(1));
        }
    }
    phase_mark("naming-hygiene walk");
    let fast_lane_eval_budget_ms: Option<u64> = if let Some(request) = &scoped_request {
        // Carried by the verified request: the child reads this budget but no longer evaluates
        // the plan that declares it. Read off the request itself rather than a second tuple
        // copied out of it — one value, one home.
        request.fast_lane_eval_budget_ms
    } else if executes_witness_rows {
        match run_value(
            plan_ctx_or_refuse!("fast-lane eval budget"),
            "gunbc_ci_fast_lane_eval_budget_ms",
        ) {
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
                plan_ctx_or_refuse!("falsifier budget"),
                "gunbc_falsifier_self_host_wet_receipt_wall_budget_ms",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            interp_eval_budget_ms: match read_positive_budget_ms(
                plan_ctx_or_refuse!("falsifier budget"),
                "gunbc_falsifier_self_host_wet_interp_eval_budget_ms",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            roster_entry_paths: match read_schedule_witness_entry_paths(
                plan_ctx_or_refuse!("plan read"),
                "falsifier_self_host_wet_entries",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            known_red_entry_paths: match read_schedule_witness_entry_paths(
                plan_ctx_or_refuse!("plan read"),
                "falsifier_self_host_wet_known_red_roster",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            substrate_long_lane_entry_paths: match read_schedule_witness_entry_paths(
                plan_ctx_or_refuse!("plan read"),
                "witness_long_eval_budget_entries",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            expected_red_witnesses: {
                let mut pairs = match read_schedule_witness_entry_pairs(
                    plan_ctx_or_refuse!("known-red probe roster"),
                    "known_red_probe_roster",
                ) {
                    Ok(v) => v,
                    Err(msg) => {
                        eprintln!("{msg}");
                        return Err(ExitCode::from(1));
                    }
                };
                match read_schedule_witness_entry_pairs(
                    plan_ctx_or_refuse!("plan read"),
                    "falsifier_self_host_wet_known_red_roster",
                ) {
                    Ok(v) => pairs.extend(v),
                    Err(msg) => {
                        eprintln!("{msg}");
                        return Err(ExitCode::from(1));
                    }
                }
                pairs
            },
            pre_verdict_refusal_witnesses: match read_schedule_witness_entry_pairs(
                plan_ctx_or_refuse!("plan read"),
                "known_red_pre_verdict_refusal_roster",
            ) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return Err(ExitCode::from(1));
                }
            },
            substrate_long_lane_eval_budget_ms: match read_positive_budget_ms(
                plan_ctx_or_refuse!("falsifier budget"),
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
    // THE WALK'S OWN DEADLINE, armed for the falsifier only. It is the plan with no
    // `ordinary_budget` (so nothing bounds admission today) AND the one running under a
    // foreign step timeout the process cannot conclude through, which is the exact pair
    // that made a crossing destroy the run's evidence. Read fail-closed through the same
    // seam as every other budget: an unavailable or non-positive value stops the line
    // rather than silently leaving admission unbounded, because a deadline that quietly
    // failed to arm is indistinguishable from the state it was added to fix.
    let falsifier_soft_deadline_ms = if plan_function == "gunbc_falsifier_plan" {
        match read_positive_budget_ms(
            plan_ctx_or_refuse!("falsifier budget"),
            "gunbc_falsifier_soft_deadline_ms",
        ) {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("{msg}");
                return Err(ExitCode::from(1));
            }
        }
    } else {
        None
    };
    // A scoped child does not re-derive this: the parent resolved it against the plan and froze
    // it into the request. Reading it from the plan here is what made the child refuse after the
    // ordinary walk had already passed — this site is unconditional for every worker.
    let batch_stop_policy = match &scoped_request {
        Some(request) => request.batch_stop_policy,
        None => resolve_floor_batch_stop_policy(
            plan_ctx_or_refuse!("batch stop policy"),
            &plan_function,
        ),
    };
    // PUBLISH THE EXACT SCOPED WORK, and publish it HERE: everything a child needs must already
    // be known, which includes the stop policy resolved just above. Publishing earlier is what
    // shipped a request missing a plan-derived value the child then had no way to obtain, so it
    // refused mid-floor with the ordinary walk already paid for.
    if matches!(floor_worker_role, Some(FloorWorkerRole::Ordinary)) {
        if let Err(msg) = scoped_execution_requests_from_rows(
            &published_scoped_rows,
            fast_lane_eval_budget_ms,
            ordinary_budget_ms,
            batch_stop_policy,
        )
        .and_then(|requests| write_scoped_execution_requests(&requests))
        {
            eprintln!("claim_executor: ordinary floor worker scoped request refusal: {msg}");
            return Err(ExitCode::from(1));
        }
    }
    // THE COST WALL (Piece 3 derived clamp): the floor plan's per-batch clamp params, read
    // fail-closed at arm time (the fast-lane-budget pattern). Scoped to the full floor plan only:
    // the plan-artifact shortcut runs a single batch of the same schedule and the falsifier
    // carries its own receipt budgets, so neither reads these lists.
    let scoped_clamps: Vec<Option<ResolvedFloorBatchClamp>> = match batches
        .iter()
        .map(|batch| scoped_batch_clamp(batch))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("claim_executor: scoped witness batch refusal: {msg}");
            return Err(ExitCode::from(1));
        }
    };
    let positional_count = scoped_clamps.iter().filter(|clamp| clamp.is_none()).count();
    let positional_clamps = if plan_function == "gunbc_ci_floor_plan" && positional_count > 0 {
        match read_floor_batch_clamp_params(
            plan_ctx_or_refuse!("positional batch clamps"),
            positional_count,
        ) {
            Ok(v) => Some(v),
            Err(msg) => {
                eprintln!("{msg}");
                return Err(ExitCode::from(1));
            }
        }
    } else {
        None
    };
    let batch_clamp_params: Option<Vec<Option<ResolvedFloorBatchClamp>>> =
        if positional_clamps.is_some() || scoped_clamps.iter().any(|clamp| clamp.is_some()) {
            let mut positional_index = 0usize;
            let mut aligned = Vec::with_capacity(batches.len());
            for owned in scoped_clamps {
                if let Some(clamp) = owned {
                    aligned.push(Some(clamp));
                } else if let Some(rows) = &positional_clamps {
                    aligned.push(rows.get(positional_index).cloned());
                    positional_index += 1;
                } else {
                    aligned.push(None);
                }
            }
            Some(aligned)
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
    let compile_clean_clamp: Option<(u128, u128)> =
        if !matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. }))
            && (plan_function == "gunbc_ci_floor_plan"
                || plan_function == "gunbc_ci_plan_artifact_plan")
        {
            match read_compile_clean_clamp(plan_ctx_or_refuse!("compile-clean clamp")) {
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
    let scoped_rows: Vec<&Runnable> = batches
        .iter()
        .flat_map(|batch| batch.iter())
        .filter(|runnable| matches!(runnable, Runnable::ScopedWitnessBatch { .. }))
        .collect();
    if !scoped_rows.is_empty() {
        let receipt_arm = scoped_witness_head_sha().and_then(|_| {
            if floor_worker_role.is_none()
                || matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. }))
            {
                initialize_scoped_witness_receipt()
            } else {
                Ok(())
            }
        });
        if let Err(msg) = receipt_arm {
            eprintln!("claim_executor: scoped witness receipt arm refusal: {msg}");
            return Err(ExitCode::from(1));
        }
        let isolation_refusal = scoped_rows.iter().find_map(|runnable| match runnable {
            Runnable::ScopedWitnessBatch {
                process_isolation: ScopedProcessIsolation::FreshJobProcess,
                ..
            } => Some("FreshJobProcess scoped witness execution has no workflow realization; it refuses rather than degrading to SequentialChildProcess"),
            Runnable::ScopedWitnessBatch {
                process_isolation: ScopedProcessIsolation::SequentialChildProcess,
                ..
            } if !matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. })) => Some(
                "SequentialChildProcess scoped witness execution requires the thin floor coordinator; the shared walk refuses rather than retaining both closures",
            ),
            Runnable::ScopedWitnessBatch {
                process_isolation: ScopedProcessIsolation::SharedWalkProcess,
                ..
            } if matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. })) => Some(
                "SharedWalkProcess scoped witness execution was routed to a child worker; the mismatched realization refuses",
            ),
            _ => None,
        });
        if let Some(detail) = isolation_refusal {
            for runnable in scoped_rows {
                if let Runnable::ScopedWitnessBatch {
                    batch_id,
                    source_roots_digest,
                    entries,
                    ..
                } = runnable
                {
                    if let Err(msg) = append_scoped_witness_receipt_rows(
                        batch_id,
                        source_roots_digest,
                        entries,
                        None,
                        Some(detail),
                    ) {
                        eprintln!(
                            "claim_executor: scoped witness scheduling receipt refusal: {msg}"
                        );
                        return Err(ExitCode::from(1));
                    }
                }
            }
            eprintln!("claim_executor: scoped witness scheduling refusal: {detail}");
            if matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. })) {
                let _ = write_floor_worker_terminal("refused", detail);
            }
            return Err(ExitCode::from(1));
        }
    }
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

    // Pre-walk execution belongs to the whole-floor walk, not to a scoped child: a
    // Scoped worker executes exactly its selected batch, and re-running the capture
    // effect per child would duplicate an effect the ordinary worker already owns.
    if !matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. })) {
        if let Err(msg) = run_pre_walk_execution(
            &source_roots,
            &format!("{plan_entry}::{plan_function}"),
            &pre_walk_execution,
        ) {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
        phase_mark("pre-walk execution");
    }
    // Derived schedule width: no plan-evaluated spawn width — realize_pack chooses
    // concurrency up front from host budget and derived space bounds (P4).
    let hardware_max = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let governor = match RealizationConcurrency::for_walk(hardware_max) {
        Ok(g) => g,
        Err(msg) => {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    };
    if plan_requires_floor_arm_time_budget_refusal(&plan_function) {
        if let Some(msg) = floor_budget_below_minimum_footprint(governor.budget_bytes()) {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    }
    phase_mark("realization-schedule arm");
    spawn_floor_memory_heartbeat();

    // Plans whose schedule carries the compile-clean gate node: the gate only CONSUMES the
    // in-run whole-tree compile receipt, so these plans must arm the lazy install.
    // `gunbc_ci_plan_artifact_plan` is batch 1 of the floor schedule (the docs-only
    // shortcut) — same gate node, same receipt dependency.
    if !matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. }))
        && (plan_function == "gunbc_ci_floor_plan"
            || plan_function == "gunbc_ci_plan_artifact_plan")
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
        &format!("{plan_entry}::{plan_function}"),
        &batches,
        &on_success_stages,
        floor_finalization.as_ref(),
        if matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. })) {
            FloorFinalizationAbsenceReason::ScopedWorkerByConstruction
        } else if plan_declared_no_finalization {
            FloorFinalizationAbsenceReason::PlanDeclaresNoFinalization
        } else {
            FloorFinalizationAbsenceReason::IncidentalAbsence
        },
        &mut std::io::stderr(),
        ordinary_budget_ms,
        on_success_budget_ms,
        falsifier_soft_deadline_ms,
        &governor,
        fast_lane_eval_budget_ms,
        falsifier_self_host_wet_budgets,
        batch_stop_policy,
        batch_clamp_params.as_deref(),
        budget_tighten_ms,
        !matches!(floor_worker_role, Some(FloorWorkerRole::Scoped { .. })),
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
                    "floor peak RSS (derived schedule width)",
                    Some(bytes),
                    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true"),
                )
            );
        }
        None => eprintln!(
            "{}",
            v1_compiler::cli_run::render_peak_rss_line_mirror(
                "floor peak RSS (derived schedule width)",
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
    emit_cgroup_measurement("floor derived-schedule width");
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
    let mut terminal_failure_details = outcome.failure_details.clone();
    if compile_clean_over_budget {
        terminal_failure_details.push(
            "post-walk compile_clean leg exceeded its declared clamp (FLOOR-COMPILE-CLEAN-OVER-BUDGET)"
                .to_string(),
        );
    }
    if !compile_clean_drift_receipt_ok {
        terminal_failure_details
            .push("post-walk compile_clean cost drift receipt refused or unwritable".to_string());
    }
    let any_failed =
        outcome.any_failed || compile_clean_over_budget || !compile_clean_drift_receipt_ok;
    if any_failed {
        emit_falsifier_failure_class(&terminal_failure_details, &outcome.infra_faults);
    }
    let terminal_detail = walk_terminal_detail(
        any_failed,
        &terminal_failure_details,
        &outcome.infra_faults,
        compile_clean_over_budget,
        compile_clean_drift_receipt_ok,
    );
    floor_terminal_fast_exit(walk_exit_code(any_failed), &terminal_detail)
}

/// Typed terminal failure class for the falsifier/floor walk (brief Step 2, 2026-07-25):
/// names BudgetExceeded{wall,budget} vs WitnessRed{claims} vs Infra{spawn/toolchain/eviction}
/// so "falsifier dark" is one of three modes, never an undifferentiated exit 1.
// STALE SIGNAL, NOT MERELY DEAD CODE. These substrings matched a budget refusal back when a
// raised one reached the claim seam as a `RuntimeError` carrying the refusal as prose. It now
// arrives as a `ClaimOutcome` budget arm with the pair typed, so nothing renders this text on the
// required-floor path any more. That matters more for the RESTORE than for today: this function
// sits past the required-floor early return, on the falsifier lane whose workflows the floor cut
// deleted, and a dead `.contains` does not fail loudly when a lane comes back — it matches
// nothing and reports NO budget findings, which says "there are none" where it should say "I
// cannot tell". Whoever re-adds this lane reads the variant, not the prose; the classification
// is deletion population otherwise.
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

/// Located refusal lines for receipt-write failures that redden the ordinary floor without
/// a witness batch failure. Without these, the walk can exit 1 with every batch green and
/// an empty `failure_details` — the silent exit-1 class #8058's terminal receipt fix did
/// not cover because the refusal lives only in the writer's eprintln.
fn push_ordinary_receipt_write_refusals(
    failure_details: &mut Vec<String>,
    resolve_receipt_ok: bool,
    batch_wall_receipt_ok: bool,
    gate_warm_cost_receipt_ok: bool,
    wet_witness_row_outcome_receipt_ok: bool,
    floor_component_receipt_ok: bool,
    materialization_receipt_ok: bool,
) {
    let mut push = |ok: bool, detail: &'static str| {
        if !ok {
            failure_details.push(detail.to_string());
        }
    };
    push(
        resolve_receipt_ok,
        "ordinary-floor resolve realization receipt write refused",
    );
    push(
        batch_wall_receipt_ok,
        "ordinary-floor batch wall receipt write refused",
    );
    push(
        gate_warm_cost_receipt_ok,
        "ordinary-floor gate warm-cost receipt write refused",
    );
    push(
        wet_witness_row_outcome_receipt_ok,
        "ordinary-floor wet witness row-outcome receipt write refused",
    );
    push(
        floor_component_receipt_ok,
        "ordinary-floor floor component receipt write refused",
    );
    push(
        materialization_receipt_ok,
        "ordinary-floor materialization demand receipt write refused",
    );
}

/// Located refusal carried in the worker terminal receipt for the coordinator to replay.
///
/// `floor_terminal_fast_exit` used to write only `walk terminal exit code N`, which the
/// thin coordinator then surfaced as the sole failure detail — a refusal that names
/// nothing is worse than none (DESIGN §5; receipt: run 31263508328 printed
/// `detail=walk terminal exit code 1` while the worker had already emitted the real
/// witness-red batch line on stderr that Actions dropped).
/// Whether a walk `failure_details` row names a located witness/content failure rather
/// than a walk-level cost interrupt or receipt fault. Batch rows whose `detail=` prose
/// merely mentions `eval budget exceeded` inside a multi-failure summary still count —
/// substring sniffing on the whole walk must not collapse those into BudgetExceeded
/// (receipt: cold-corpus walk-terminal printed mode=BudgetExceeded while detail= carried
/// ~10 typed witness findings plus one budget fact).
fn walk_detail_is_semantic_batch_finding(d: &str) -> bool {
    if !d.starts_with("batch=") || !d.contains("fn=") {
        return false;
    }
    if d.contains("BudgetExceeded{wall_ms") || d.contains("walk reached its soft deadline") {
        return false;
    }
    if d.contains("receipt write refused") || d.contains("floor finalization refused") {
        return false;
    }
    let body = d.split_once("detail=").map(|(_, b)| b).unwrap_or(d);
    if body.contains("eval budget exceeded")
        || body.contains("witness receipt wall budget exceeded")
        || body.contains("wet self-host receipt wall budget exceeded")
    {
        // A lone budget interrupt on one witness is not a semantic finding; a batch
        // summary that also names resolve/cast/expectation failures is.
        return body.contains("failed:")
            || body.contains("resolve")
            || body.contains("StaleKnownRed")
            || body.contains("ExpectedRedPreVerdictUnverified")
            || body.contains("cannot cast")
            || body.contains("is not a published mock case")
            || body.contains("returned Bool(false)");
    }
    true
}

/// Strictly per-row derived: a walk carries semantic findings exactly when some row IS
/// one. An earlier revision also short-circuited on `batch_rows > 1`, which classified two
/// pure budget interrupts as WitnessRed — this fix's own defect run backwards, a cost
/// interrupt reported as a semantic verdict. A count cannot answer which rows are
/// semantic; `walk_detail_is_semantic_batch_finding` is that property, and where the
/// property already decides, the count adds only false positives.
fn walk_has_semantic_witness_findings(details: &[String]) -> bool {
    details
        .iter()
        .any(|d| walk_detail_is_semantic_batch_finding(d))
}

/// Walk-terminal mode: semantic witness findings outrank cost interrupts in the
/// emitted mode. BudgetExceeded remains in the detail string as interruption evidence;
/// it must not become the sole terminal class when located findings already exist.
fn walk_terminal_failure_mode(details: &[String], infra_faults: &[InfraFault]) -> &'static str {
    if !infra_faults.is_empty() {
        return "Infra";
    }
    if walk_has_semantic_witness_findings(details) {
        return "WitnessRed";
    }
    if details.iter().any(|d| {
        d.contains("walk reached its soft deadline") || d.contains("ORDINARY-FLOOR-OVER-BUDGET")
    }) {
        return "BudgetExceeded";
    }
    falsifier_failure_mode(details)
}

fn walk_terminal_detail(
    any_failed: bool,
    failure_details: &[String],
    infra_faults: &[InfraFault],
    compile_clean_over_budget: bool,
    compile_clean_drift_receipt_ok: bool,
) -> String {
    if !any_failed {
        return "walk terminal exit code 0".to_string();
    }
    let mode = walk_terminal_failure_mode(failure_details, infra_faults);
    let mut parts = vec![format!(
        "walk terminal exit code 1 mode={mode} ci_failure_class_arm={}",
        ci_failure_class_arm(mode)
    )];
    if !failure_details.is_empty() {
        parts.push(failure_details.join(" | "));
    }
    if compile_clean_over_budget {
        parts.push("compile_clean_over_budget".to_string());
    }
    if !compile_clean_drift_receipt_ok {
        parts.push("compile_clean_cost_drift_receipt_refused".to_string());
    }
    scoped_wire_text(&parts.join(" "))
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
///
/// #8058 carried the located refusal in the worker terminal receipt file only; Actions
/// can drop inherited stderr after a large discovery run, and the phase journal had no
/// terminal row — so a post-walk compile_clean refusal or a receipt-write failure looked
/// like a silent exit-1 with every batch green. Emit the same detail on stderr and the
/// durable journal before `process::exit`.
fn population_budget_watchdog_exit(detail: &str) -> ! {
    emit_floor_terminal_outcome("failed", detail);
    if let Err(msg) = write_floor_worker_terminal("refused", detail) {
        eprintln!("claim_executor: population budget watchdog terminal receipt refusal: {msg}");
    }
    use std::io::Write as _;
    let _ = std::io::stderr().flush();
    std::process::exit(1);
}

fn floor_terminal_fast_exit(code: i32, detail: &str) -> ! {
    use std::io::Write as _;
    let terminal_outcome = if code == 0 { "completed" } else { "failed" };
    emit_floor_terminal_outcome(terminal_outcome, detail);
    let final_code = match write_floor_worker_terminal(terminal_outcome, detail) {
        Ok(()) => code,
        Err(msg) => {
            eprintln!("claim_executor: floor worker terminal receipt refusal: {msg}");
            1
        }
    };
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    std::process::exit(final_code)
}

fn emit_floor_terminal_outcome(outcome: &str, detail: &str) {
    append_floor_phase_journal("walk-terminal", outcome, detail);
    if outcome == "completed" {
        eprintln!("[floor-terminal] outcome=completed detail={detail}");
    } else {
        eprintln!("[floor-terminal-refusal] outcome=failed detail={detail}");
    }
}

/// Pre-walk worker failures return through `main()` instead of `floor_terminal_fast_exit`.
/// Mirror the walk-terminal journal/stderr emission so those refusals survive Actions log
/// truncation the same way post-walk failures do.
fn emit_worker_terminal_before_return(code: ExitCode) -> ExitCode {
    let Some(path) = std::env::var_os(FLOOR_WORKER_TERMINAL_ENV) else {
        return code;
    };
    if code == ExitCode::SUCCESS {
        return code;
    }
    let path = PathBuf::from(path);
    let (outcome, detail) = if path.exists() {
        match fs::read_to_string(&path) {
            Ok(body) => {
                let line = body.trim_end_matches(['\r', '\n']);
                let (label, detail) = line.split_once('\t').unwrap_or((line, ""));
                let outcome = if label == "completed" {
                    "completed"
                } else {
                    "failed"
                };
                (outcome, detail.to_string())
            }
            Err(e) => ("failed", format!("worker terminal receipt unreadable: {e}")),
        }
    } else {
        let detail = "worker returned before producing a walk terminal receipt".to_string();
        let _ = write_floor_worker_terminal("failed", &detail);
        ("failed", detail)
    };
    emit_floor_terminal_outcome(outcome, &detail);
    code
}

/// Print the floor's complete ledger. ONE implementation, called by `--required-floor` and by
/// the composed `--required-ci` run, so the two modes cannot drift into reporting the same
/// outcome differently (DESIGN §3).
fn report_required_floor_outcome(outcome: &v1_compiler::cli_run::RequiredFloorOutcome) {
    eprintln!(
        "required-floor: subject={} modules_resolved={} modules_excluded={}",
        outcome.subject_digest, outcome.modules_resolved, outcome.modules_excluded
    );
    // THE SUBJECT THE ROSTER WAS PROJECTED FROM, STATED BEFORE THE ROSTER.
    // `planned` is the population that SURVIVED site projection; printing it
    // without `offered` and `declined_long` made the receipt unable to say what it
    // dropped, which is how a roster that narrowed read exactly like one that did
    // not. The three are printed together so the subtraction is visible rather
    // than inferable.
    eprintln!(
        "required-floor: offered={} routed={} declined_long={} declined_fixture={} \
         declined_live={} — every discovered site is exactly one of these",
        outcome.sites_offered,
        outcome.claims_planned,
        outcome.declined_long_module,
        outcome.declined_fixture_member,
        outcome.declined_live_tree
    );
    // WHY route_gap IS NOW SPELLED route_gap_unenrolled, AND WHY route_gap_held JOINS IT HERE.
    // The old field printed `outcome.route_gap.len()` under the bare name `route_gap` — the
    // UNENROLLED gaps alone. Measured on main run 32407436149 that printed `route_gap=0` while
    // 101 enrolled identities were held as route-gapped on a `[floor-route-gap]` line ~300 lines
    // away, and the headline's own categories did not close: executed 9810 − passed 9502 −
    // known_red_held 207 − failed 0 left exactly those 101 unaccounted. A reader did not see an
    // understated count they might interrogate; they saw a ZERO, which closes the question rather
    // than opening it. The correcting evidence existed and was disjoint from the surface anyone
    // reads, so it lands on THIS line — shipping it as yet another separate line would reproduce
    // the defect it repairs.
    //
    // THESE COUNTERS ARE OVERLAPPING AXES, NOT A PARTITION, and the receipt must not be read as
    // one. A single claim that is enrolled expected-red AND route-gapped increments
    // `known_red_host_effect_refused` and `route_gap_held` both (see the ExpectedRedArm::
    // HostEffectRefused arm in cli_run: the two rosters are different axes and the row sits on
    // both). So `passed + known_red_held + … == executed` is NOT an invariant and asserting it
    // would red on a legitimate state. The tree already has the vocabulary for this distinction —
    // `InclusiveCostRow` carries "a counter deliberately INCLUSIVE of another … so the receipt can
    // print it without it ever entering the exclusive sum" — but the floor's ledger has no such
    // typed exclusive/inclusive split, which is why the sum silently failed to close instead of
    // refusing. Modelling that split is a RequiredFloorDisposition question in
    // src/v2/workflow/required_floor.dag, not something to decide inside an eprintln!.
    eprintln!(
        "required-floor: planned={} executed={} terminal={} passed={} \
         known_red_held={} failed={} stale_quarantine={} \
         interrupted_before_verdict={} completed_over_cost_requirement={} \
         host_tool_unresolved={} route_gap_unenrolled={} route_gap_held={} \
         stale_route_gap={} known_red_now_passing={} known_red_budget_refused={} \
         known_red_passed_over_budget={} known_red_host_tool_unresolved={} \
         known_red_host_effect_refused={} known_red_runtime_errored={} \
         known_red_observation_unreadable={} over_cost_line_diagnostic={}",
        outcome.claims_planned,
        outcome.claims_executed,
        outcome.receipt_identities,
        outcome.passed,
        outcome.known_red_held,
        outcome.failures.len(),
        outcome.stale_quarantine.len(),
        outcome.interrupted_before_verdict.len(),
        outcome.completed_over_cost_requirement.len(),
        outcome.host_tool_unresolved.len(),
        outcome.route_gap.len(),
        outcome.route_gap_held,
        outcome.stale_route_gap.len(),
        outcome.known_red_now_passing,
        outcome.known_red_budget_refused,
        outcome.known_red_passed_over_budget,
        outcome.known_red_host_tool_unresolved_held,
        outcome.known_red_host_effect_refused,
        outcome.known_red_runtime_errored.len(),
        outcome.known_red_observation_unreadable.len(),
        outcome.over_cost_line_diagnostic
    );
    // ONE receipt, both numbers (#8642). This replaced a per-miss trace line that had no hit
    // counterpart, so the ratio it is really about was never readable.
    //
    // EXACTLY ONE OF THESE MAY EXIST, and a duplicate is not cosmetic: two lines reporting one
    // pair is the second-representation shape the receipt was introduced to remove, so
    // duplicating it degrades the property it asserts. There WAS a second copy here briefly —
    // this function is re-derived from main's inline block on every merge that touches it, and
    // a note reading "each merge has to graft it back deliberately" instructed the re-add
    // without saying to check whether main's block already carried it. It did. Caught in
    // review 54101. The instruction is deleted with the duplicate: re-derivation copies main's
    // block wholesale, so this line arrives WITH it and needs no grafting.
    let (memo_hits, memo_misses) = v1_compiler::cli_run::compile_dag_rust_emit_check_memo_counts();
    eprintln!(
        "required-floor: compile_dag_rust_emit_check_memo hits={memo_hits} \
         misses={memo_misses}"
    );
    for failure in &outcome.failures {
        eprintln!("required-floor: FAIL {failure}");
    }
    // SEVEN CAUSES, SEVEN COUNTS, ONE STOPPED LINE. All seven refuse the run, and
    // they are reported apart because their remedies differ: a FAIL is a defect to
    // fix, a STALE-QUARANTINE is a fix that already landed and a roster row to
    // delete, an INTERRUPTED-BEFORE-VERDICT is an undecided claim whose real cost
    // is unmeasured (operator ruling 2026-08-19, BUDGET POLICY CUT), a
    // COMPLETED-OVER-COST-REQUIREMENT is a claim that reached a verdict and then
    // was found to cost too much (an exact measurement, not a bound), and a
    // HOST-TOOL-UNRESOLVED is an infra gap to provision (never a witness-cost
    // chase), and a ROUTE-GAP is a claim that never reached its subject because
    // its execution route has no arm for a host effect it reached for — remedied by
    // supplying a route, never by editing the witness, and a STALE-ROUTE-GAP is
    // that same roster's other direction — a route that WAS supplied, whose
    // enrollment must now be deleted. Summing them into `failed` would make an un-quarantine
    // indistinguishable from a regression in the alert signature, which is the
    // conflation `std.witness_admission` rules out. Splitting the former
    // `budget_refused` collection in two makes it visible whether a stopped run
    // is a cost debt on a claim that actually finished, or a claim the safety
    // deadline preempted before it could answer at all — but both still stop the
    // line: an interruption is NotEvaluated, and NotEvaluated is never green.
    for stale in &outcome.stale_quarantine {
        eprintln!("required-floor: STALE-QUARANTINE {stale}");
    }
    for refused in &outcome.interrupted_before_verdict {
        eprintln!("required-floor: INTERRUPTED-BEFORE-VERDICT {refused}");
    }
    for over_cost in &outcome.completed_over_cost_requirement {
        eprintln!("required-floor: COMPLETED-OVER-COST-REQUIREMENT {over_cost}");
    }
    for unresolved in &outcome.host_tool_unresolved {
        eprintln!("required-floor: HOST-TOOL-UNRESOLVED {unresolved}");
    }
    for errored in &outcome.known_red_runtime_errored {
        eprintln!("required-floor: KNOWN-RED-RUNTIME-ERRORED {errored}");
    }
    for unreadable in &outcome.known_red_observation_unreadable {
        eprintln!("required-floor: KNOWN-RED-OBSERVATION-UNREADABLE {unreadable}");
    }
    for unenrolled in &outcome.non_verdict_unenrolled {
        eprintln!("required-floor: NON-VERDICT-UNENROLLED {unenrolled}");
    }
    for stale in &outcome.stale_non_verdict {
        eprintln!("required-floor: STALE-NON-VERDICT {stale}");
    }
    for gap in &outcome.route_gap {
        eprintln!("required-floor: ROUTE-GAP {gap}");
    }
    for stale in &outcome.stale_route_gap {
        eprintln!("required-floor: STALE-ROUTE-GAP {stale}");
    }
    // THE VERDICT LINE, AND WHY A ZERO NEEDED A SENTENCE BESIDE IT.
    //
    // `failed=0` is a sentence a reader can finish alone, and finishes wrongly: it says nothing
    // about whether the population that was supposed to answer actually answered. This exact
    // misreading dispatched a session against a regression that did not exist — a run was
    // compared to a baseline carrying a fix, `planned` matched on both sides, and `failed=0`
    // supplied the confidence that the rest was equivalent. Separating the two questions removes
    // the ability to finish that sentence: `unexpected_failures` is how many claims answered
    // WRONG, `verdict_incomplete` is how many never answered AT ALL, and a run can be admitted
    // while the second is large.
    //
    // ADMITTED IS NOT CLEAN, and the word carries that. `FloorAdmittedWithNonVerdictDebt` gates
    // nothing by itself — the conjunct that gates is growth in the non-verdict population — but
    // it refuses to let a run with 142 unanswered assertions render identically to one with
    // none.
    let verdict_incomplete =
        outcome.known_red_runtime_errored.len() + outcome.known_red_observation_unreadable.len();
    let verdict = if !required_floor_outcome_is_clean(outcome) {
        "FloorRefused"
    } else if verdict_incomplete > 0 {
        "FloorAdmittedWithNonVerdictDebt"
    } else {
        "FloorClean"
    };
    eprintln!(
        "required-floor: verdict={verdict} unexpected_failures={} verdict_incomplete={} \
         non_verdict_unenrolled={} stale_non_verdict={}",
        outcome.failures.len(),
        verdict_incomplete,
        outcome.non_verdict_unenrolled.len(),
        outcome.stale_non_verdict.len()
    );
}

/// Whether the floor outcome permits a green run.
///
/// NINE CAUSES, ONE STOPPED LINE — and the conjunction is written once here rather than at each
/// caller, because a mode that forgot one of them would green a run the other refused. (The
/// count is stated because a reader checks it; it was five before main added `route_gap` and
/// `stale_route_gap`, and the sentence went on saying five through the merge that added them.
/// It briefly said nine while `known_red_runtime_errored` and `known_red_observation_unreadable`
/// were wired in here directly; that was reverted and the count returned to seven.)
///
/// THE EIGHTH IS `non_verdict_unenrolled`, AND IT IS NOT THOSE TWO ARMS MADE GATING. The
/// distinction is the whole design. Those arms are HONEST OBSERVATIONS — they say correctly that
/// an enrolled claim produced no verdict — and gating on them directly would red every lane
/// holding a row of a population nobody has repaired. What was below floor is the COMPOSITION:
/// this function returned CLEAN while an enrolled expected-red assertion had ceased to assert
/// anything, so a true diagnostic sat beside a false conclusion drawn from it. The conjunct
/// therefore gates on GROWTH at identity grain — an identity producing no verdict that
/// `v2.workflow.floor_non_verdict` does not carry — which admits 142 → 0 in any order and
/// refuses 142 → 143, and refuses a swap that leaves the count untouched.
///
/// THE NINTH IS `stale_non_verdict`, AND IT GATES FOR THE REASON THE EIGHTH DOES. A row whose
/// identity has been repaired is a LIVE EXEMPTION until it is deleted: the witness is fixed
/// today and, should it stop producing a verdict again, it is already rostered and the eighth
/// conjunct admits it. Repayment and deletion are therefore one act, which is what
/// `stale_route_gap` and the expected-red staleness join already require. This shipped as
/// report-only for one commit under the argument that refusing "punishes the fix"; it does not
/// — it requires the fix to be complete, and the diagnostic names every row to delete.
fn required_floor_outcome_is_clean(outcome: &v1_compiler::cli_run::RequiredFloorOutcome) -> bool {
    outcome.failures.is_empty()
        && outcome.non_verdict_unenrolled.is_empty()
        && outcome.stale_non_verdict.is_empty()
        && outcome.stale_quarantine.is_empty()
        && outcome.interrupted_before_verdict.is_empty()
        && outcome.completed_over_cost_requirement.is_empty()
        && outcome.host_tool_unresolved.is_empty()
        && outcome.route_gap.is_empty()
        && outcome.stale_route_gap.is_empty()
}

fn main() -> ExitCode {
    let coordinator_args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = maybe_run_floor_coordinator(&coordinator_args) {
        return code;
    }
    // The materialization demand receipt is mandatory on the floor: enable the
    // interpreter's recompute-trace ledger unless the environment already set
    // it. An explicit =0 zeroes the receipt, and the derived ci.yml gate fails
    // closed on keyed_calls=0 — disabling is loud, never silent.
    if std::env::var_os("GUNBC_RECOMPUTE_TRACE").is_none() {
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
    }
    let code = match run() {
        Ok(code) => code,
        Err(code) => code,
    };
    emit_worker_terminal_before_return(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_budget_watchdog_refuses_and_writes_located_receipt() {
        const CHILD_MARKER: &str = "GUNBC_POPULATION_BUDGET_WATCHDOG_CHILD";
        if std::env::var(CHILD_MARKER).as_deref() == Ok("1") {
            let progress = PopulationBudgetProgress::before_first_unit();
            progress.enter(3, "dag/tools/fixture.dag::bounded_stage_claim".to_string());
            let terminal = PathBuf::from("target/population-budget-watchdog-terminal.tsv");
            let _ = fs::remove_file(&terminal);
            std::env::set_var(FLOOR_WORKER_TERMINAL_ENV, &terminal);
            let _armed = arm_population_budget_watchdog(
                "ordinary_floor",
                "fixture::bounded_plan",
                Some(50),
                progress,
            );
            std::thread::sleep(std::time::Duration::from_secs(2));
            panic!("watchdog did not terminate the child");
        }

        let dir = std::env::temp_dir().join(format!(
            "gunbc-population-budget-watchdog-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create watchdog fixture directory");
        let terminal = dir.join("target/population-budget-watchdog-terminal.tsv");
        let output = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .arg("--exact")
            .arg("tests::population_budget_watchdog_refuses_and_writes_located_receipt")
            .arg("--nocapture")
            .env(CHILD_MARKER, "1")
            .current_dir(&dir)
            .output()
            .expect("run watchdog child");
        assert!(!output.status.success(), "budget child must refuse");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("ORDINARY-FLOOR-OVER-BUDGET")
                && stderr.contains("fixture::bounded_plan")
                && stderr.contains("population_index=3")
                && stderr.contains("dag/tools/fixture.dag::bounded_stage_claim")
                && stderr.contains("elapsed_ms=")
                && stderr.contains("budget_ms=50"),
            "refusal must be typed and located, got: {stderr}"
        );
        let receipt = fs::read_to_string(dir.join("target/floor-population-budget-refusal.txt"))
            .expect("watchdog refusal receipt");
        assert!(
            receipt.contains("population=ordinary_floor")
                && receipt.contains("plan_site=fixture::bounded_plan")
                && receipt.contains("population_index=3")
                && receipt.contains("active_unit=dag/tools/fixture.dag::bounded_stage_claim")
                && receipt.contains("elapsed_ms=")
                && receipt.contains("budget_ms=50")
                && receipt.contains("outcome=refused"),
            "receipt must carry the refused population and its subject: {receipt}"
        );
        let worker_terminal = fs::read_to_string(&terminal).expect("worker terminal receipt");
        assert!(
            worker_terminal.starts_with("refused\t")
                && worker_terminal.contains("ORDINARY-FLOOR-OVER-BUDGET")
                && worker_terminal.contains("fixture::bounded_plan")
                && worker_terminal.contains("population_index=3")
                && worker_terminal.contains("dag/tools/fixture.dag::bounded_stage_claim")
                && worker_terminal.contains("budget_ms=50"),
            "progress announcement and worker terminal receipt must carry the same located refusal: {worker_terminal:?}"
        );
        fs::remove_dir_all(&dir).expect("remove watchdog fixture directory");
    }

    // Process-wide eval-recompute totals are shared across every test in this
    // binary. Tests that drain or assert on the accumulator must serialize so
    // one cannot steal another's totals (DESIGN §5 hermetic discriminating tests).
    static PROCESS_EVAL_RECOMPUTE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_process_eval_recompute_test_lock<F: FnOnce()>(f: F) {
        let _guard = PROCESS_EVAL_RECOMPUTE_TEST_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let prior_trace = std::env::var_os("GUNBC_RECOMPUTE_TRACE");
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
        v1_compiler::v1_interpreter::refresh_eval_recompute_trace_enabled_cache_for_tests();
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        match prior_trace {
            Some(value) => std::env::set_var("GUNBC_RECOMPUTE_TRACE", value),
            None => std::env::remove_var("GUNBC_RECOMPUTE_TRACE"),
        }
        v1_compiler::v1_interpreter::refresh_eval_recompute_trace_enabled_cache_for_tests();
        match run {
            Ok(()) => {}
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    fn with_workspace_root_current_dir<F: FnOnce()>(root: &std::path::Path, f: F) {
        let prior_cwd = std::env::current_dir().ok();
        std::env::set_current_dir(root).expect("chdir to workspace root for receipt paths");
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        if let Some(cwd) = prior_cwd {
            let _ = std::env::set_current_dir(cwd);
        }
        match run {
            Ok(()) => {}
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    fn finalization_record(resolve_counts: &[u64]) -> BatchRecord {
        BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "finalization-fixture".to_string(),
            is_wet: false,
            results: resolve_counts
                .iter()
                .map(|n| ClaimResult {
                    function: "fixture".to_string(),
                    entry: "fixture.dag".to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: *n as u128,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: None,
                })
                .collect(),
        }
    }

    fn test_floor_finalization() -> FloorFinalization {
        FloorFinalization {
            expected_obligations: vec![
                TransportedObligation {
                    identity: "CompileAnchorWholeTree".to_string(),
                    entry: TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY.to_string(),
                    function: TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION.to_string(),
                },
                TransportedObligation {
                    identity: "NativeBundleEscapingEntry".to_string(),
                    entry: TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY.to_string(),
                    function: TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION.to_string(),
                },
            ],
        }
    }

    fn repo_root_from_manifest() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root from CARGO_MANIFEST_DIR")
    }

    fn dag_source_from_repo(rel: &str) -> String {
        let path = repo_root_from_manifest().join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn dag_string_data_literal(source: &str, data_name: &str) -> String {
        let marker = format!("data {data_name}: String = \"");
        let start = source
            .find(&marker)
            .unwrap_or_else(|| panic!("string data row {data_name} not found in authority source"))
            + marker.len();
        let rest = &source[start..];
        let end = rest
            .find('"')
            .expect("unterminated string literal in authority source");
        rest[..end].to_string()
    }

    fn dag_record_string_field(source: &str, data_name: &str, field: &str) -> String {
        let marker = format!("data {data_name}:");
        let start = source
            .find(&marker)
            .unwrap_or_else(|| panic!("data row {data_name} not found in authority source"));
        let slice = &source[start..];
        let field_marker = format!("{field}: \"");
        let fstart = slice
            .find(&field_marker)
            .unwrap_or_else(|| panic!("field {field} not found on data row {data_name}"))
            + field_marker.len();
        let rest = &slice[fstart..];
        let end = rest
            .find('"')
            .expect("unterminated string field in authority source");
        rest[..end].to_string()
    }

    // Test-only mirror of gunbc.ci_materialization obligation subject rows. Production
    // parses transported obligations from WalkPlan.finalization; this module is the
    // checkable drift receipt (review 48261 / 48570).
    const TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY: &str = "dag/tools/floor_effect_gate_witness.dag";
    const TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION: &str = "dag_compile_clean_gate_passes";
    const TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY: &str =
        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag";
    const TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION: &str = "native_selected_logic_production_spec";

    /// Seed-retained walk-memo provider id must track the `.dag` authority row.
    #[test]
    fn floor_entry_walk_memo_provider_id_matches_dag_authority() {
        let floor_materialization = dag_source_from_repo("dag/gunbc/floor_materialization.dag");
        assert_eq!(
            dag_string_data_literal(&floor_materialization, "floor_entry_walk_memo_provider_id"),
            FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID,
        );
    }

    /// Test-only obligation-subject literals must track gunbc.ci_materialization authority rows.
    #[test]
    fn floor_resolve_obligation_seed_constants_match_dag_authority() {
        let ci_materialization = dag_source_from_repo("dag/gunbc/ci_materialization.dag");
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "compile_anchor_obligation_subject",
                "entry"
            ),
            TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "compile_anchor_obligation_subject",
                "function"
            ),
            TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "native_bundle_obligation_subject",
                "entry"
            ),
            TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "native_bundle_obligation_subject",
                "function"
            ),
            TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION,
        );
    }

    /// Resolve realization is acquisition evidence: a semantic witness failure must not
    /// discard the observation recorded when resolve succeeded.
    #[test]
    fn claim_result_for_outcome_preserves_resolve_realization_on_semantic_failure() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/tools/floor_effect_gate_witness.dag")
            .to_string_lossy()
            .into_owned();
        let (graph, indices) =
            resolve_entry_graph(&source_roots, &entry).expect("resolve compile-anchor entry");
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let observation =
            Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 42 });
        let result = claim_result_for_outcome(
            &ctx,
            "dag_compile_clean_gate_passes".to_string(),
            entry,
            ClaimOutcome::NotBool {
                got: "unit".to_string(),
            },
            1,
            42,
            observation.clone(),
        );
        assert!(!result.ok);
        assert_eq!(result.resolve_realization, observation);
    }

    #[test]
    fn floor_finalization_preserves_resolve_evidence_when_business_claim_fails() {
        let fin = test_floor_finalization();
        let mut records = obligation_finalization_records(5, 3);
        records[0].results[0].ok = false;
        records[0].results[0].detail = "returned Bool(false)".to_string();
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals.iter().any(|m| {
                m.contains("missing realization observation") || m.contains("count law unevaluable")
            }),
            "semantic witness failure must not fabricate missing resolve evidence: {refusals:?}"
        );
    }

    fn resolve_observation_for_nanos(nanos: u64) -> Option<ResolveRealizationObservation> {
        if nanos > 0 {
            Some(ResolveRealizationObservation::ColdResolvePerformed {
                resolve_nanos: nanos as u128,
            })
        } else {
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool {
                computation_identity: "entry-closure:fixture".to_string(),
                provider_id: FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID.to_string(),
            })
        }
    }

    fn obligation_finalization_records(anchor_nanos: u64, native_nanos: u64) -> Vec<BatchRecord> {
        vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "obligation-fixture".to_string(),
            is_wet: false,
            results: vec![
                ClaimResult {
                    function: "dag_compile_clean_gate_passes".to_string(),
                    entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: anchor_nanos as u128,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: resolve_observation_for_nanos(anchor_nanos),
                },
                ClaimResult {
                    function: "native_selected_logic_production_spec".to_string(),
                    entry:
                        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag"
                            .to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: native_nanos as u128,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: resolve_observation_for_nanos(native_nanos),
                },
            ],
        }]
    }

    /// The `<entry>::<function>` a refusal locates itself at — the same shape the
    /// production caller passes.
    const TEST_PLAN_SITE: &str = "src/v2/workflow/ci_floor_plan.dag::gunbc_ci_floor_plan";

    #[test]
    fn floor_finalization_refuses_unattributed_physical_resolve() {
        let fin = test_floor_finalization();
        let mut records = obligation_finalization_records(3, 0);
        records[0].results.push(ClaimResult {
            function: "extra_witness".to_string(),
            entry: "dag/test/claim/extra.dag".to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 42,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        });
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("unattributed physical resolve")),
            "extra cold resolve must refuse: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_refuses_unattributed_physical_resolve_at_non_rostered_subject() {
        let fin = test_floor_finalization();
        let anchor_entry = TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY;
        let records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "emit-only".to_string(),
            is_wet: false,
            results: vec![ClaimResult {
                function: "emit_host_gate_passes".to_string(),
                entry: anchor_entry.to_string(),
                ok: true,
                detail: String::new(),
                wall_nanos: 0,
                resolve_nanos: 7,
                corpus_resolve_nanos: 0,
                corpus_eval_nanos: 0,
                corpus_witnesses: 0,
                runtime_unit_count: single_claim_runtime_unit_count(),
                witness_row_costs: Vec::new(),
                expectation_refusal: None,
                budget_refusal: None,
                host_dependency_refusal: None,
                resolve_realization: None,
            }],
        }];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals.iter().any(|m| {
                m.contains("unattributed physical resolve")
                    && m.contains(&format!("{anchor_entry}::emit_host_gate_passes"))
            }),
            "non-rostered subject with cold resolve_nanos must be named: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_names_unattributed_subject_when_roster_subject_also_missing() {
        let fin = test_floor_finalization();
        let anchor_entry = TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY;
        let mut records = obligation_finalization_records(5, 0);
        records[0]
            .results
            .retain(|r| r.function == "dag_compile_clean_gate_passes");
        records[0].results[0].resolve_realization = None;
        records[0].results.push(ClaimResult {
            function: "emit_host_gate_passes".to_string(),
            entry: anchor_entry.to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 7,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        });
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals.iter().any(|m| {
                m.contains("unattributed physical resolve")
                    && m.contains(&format!("{anchor_entry}::emit_host_gate_passes"))
            }),
            "unattributed subject must be named even when a roster subject is also missing: {refusals:?}"
        );
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("native_selected_logic_production_spec")),
            "missing roster subject must still be named: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_cheap_gate_cold_on_shared_obligation_entry_is_not_unattributed() {
        let fin = test_floor_finalization();
        let anchor_entry = "dag/tools/floor_effect_gate_witness.dag";
        let records = vec![
            BatchRecord {
                batch_index: 0,
                wall_nanos: 0,
                clamp_ms: None,
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                label: "cheap-gates".to_string(),
                is_wet: false,
                results: vec![ClaimResult {
                    function: "cheap_claim_pool_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 9,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 9 },
                    ),
                }],
            },
            BatchRecord {
                batch_index: 1,
                wall_nanos: 0,
                clamp_ms: None,
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                label: "compile-anchor".to_string(),
                is_wet: false,
                results: vec![ClaimResult {
                    function: "dag_compile_clean_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::SatisfiedFromSharedPool {
                            computation_identity: format!("entry-closure:{anchor_entry}:Hermetic"),
                            provider_id: FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID.to_string(),
                        },
                    ),
                }],
            },
            BatchRecord {
                batch_index: 2,
                wall_nanos: 0,
                clamp_ms: None,
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                label: "native-bundle".to_string(),
                is_wet: false,
                results: vec![ClaimResult {
                    function: "native_selected_logic_production_spec".to_string(),
                    entry:
                        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag"
                            .to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 3,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: resolve_observation_for_nanos(3),
                }],
            },
        ];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("unattributed physical resolve")),
            "cheap-gate cold on shared obligation entry must not refuse: {refusals:?}"
        );
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("floor resolve obligation missing")),
            "warm compile-clean must satisfy anchor obligation: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_refuses_duplicate_cold_on_obligation_entry() {
        let fin = test_floor_finalization();
        let anchor_entry = "dag/tools/floor_effect_gate_witness.dag";
        let records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "memo-regression".to_string(),
            is_wet: false,
            results: vec![
                ClaimResult {
                    function: "cheap_claim_pool_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 9,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 9 },
                    ),
                },
                ClaimResult {
                    function: "dag_compile_clean_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 7,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 7 },
                    ),
                },
            ],
        }];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("duplicate cold on obligation entry")),
            "second cold on rostered entry must refuse: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_warm_native_includes_provider_receipt() {
        let fin = test_floor_finalization();
        let records = obligation_finalization_records(3, 0);
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("warm disposition without provider")),
            "warm native with provider must not refuse: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_refuses_warm_without_provider_observation() {
        let fin = test_floor_finalization();
        let mut records = obligation_finalization_records(3, 0);
        records[0].results[1].resolve_realization =
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool {
                computation_identity: "entry-closure:fixture".to_string(),
                provider_id: String::new(),
            });
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("warm disposition without provider id")),
            "fabricated warm without provider must refuse: {refusals:?}"
        );
    }

    // Floor finalization law 1 (in-executor form of the deleted resolve-receipt gate
    // step): transported obligation population must match observed realizations.
    #[test]
    fn floor_finalization_refuses_on_obligation_mismatch_both_directions() {
        let fin = test_floor_finalization();
        let mut partial = obligation_finalization_records(5, 0);
        partial[0]
            .results
            .retain(|r| r.function == "dag_compile_clean_gate_passes");
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &partial, false);
        assert!(
            refusals.iter().any(|m| {
                m.contains("count law unevaluable")
                    && m.contains("never executed on a completed walk")
                    && m.contains("native_selected_logic_production_spec")
            }),
            "missing transported obligation on complete walk must refuse unevaluable: {refusals:?}"
        );
        let under = [finalization_record(&[])];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &under, false);
        assert!(
            refusals.iter().any(|m| m.contains("count law unevaluable")),
            "missing obligations on complete walk must refuse unevaluable: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_truncated_walk_reports_unevaluable_not_debt_change() {
        let fin = test_floor_finalization();
        let mut partial = obligation_finalization_records(5, 0);
        partial[0]
            .results
            .retain(|r| r.function == "dag_compile_clean_gate_passes");
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &partial, true);
        assert!(
            refusals.iter().any(|m| {
                m.contains("obligations not fully scheduled")
                    && m.contains("native_selected_logic_production_spec")
            }),
            "truncated walk must name never-ran obligations: {refusals:?}"
        );
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("differs from transported")),
            "truncated walk must not diagnose roster debt change: {refusals:?}"
        );
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("floor resolve obligation missing")),
            "truncated walk must not use missing-obligation debt wording: {refusals:?}"
        );
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("walk stopped before dependent batches")),
            "truncated walk must name truncation cause: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_warm_native_satisfies_obligations_with_one_cold() {
        let fin = test_floor_finalization();
        let records = obligation_finalization_records(3, 0);
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("floor resolve obligation")),
            "warm native + cold anchor must satisfy obligations: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_matching_obligations_leaves_only_the_materialization_arm() {
        let fin = test_floor_finalization();
        let records = obligation_finalization_records(3, 9);
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("differs from transported")),
            "obligation law must be satisfied: {refusals:?}"
        );
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("floor materialization receipt missing")),
            "absent receipt must refuse, never pass: {refusals:?}"
        );
    }

    // --- floor-finalization DISPOSITION visibility (the fix this PR ships) -----------
    //
    // Three previously-silent cases, each with its own discriminating control so the
    // three cannot collapse back into one indistinguishable "nothing printed" bucket:
    //   A — finalization absent by construction (scoped floor worker)
    //   B — finalization absent incidentally (walk never reached the call)
    //   C — finalization HELD on an otherwise-failed floor (verdict must not vanish)

    #[test]
    fn floor_finalization_disposition_case_c_held_is_unconditional_on_refusals_alone() {
        // RED CONTROL for case C: `floor_finalization_disposition_from_refusals` takes
        // no `ordinary_failed` (or any other floor-outcome) parameter — so a held
        // verdict cannot be suppressed by floor outcome, because nothing in this call
        // chain can ever see it. If a future edit reintroduces suppression by inlining
        // an outcome check ahead of this call, this function's signature is what would
        // have to change to make it possible, which is the load-bearing fact this test
        // pins.
        let disposition = floor_finalization_disposition_from_refusals(Vec::new());
        assert_eq!(disposition, FloorFinalizationDisposition::Held);
        let lines = floor_finalization_disposition_lines(&disposition);
        assert_eq!(
            lines.len(),
            1,
            "held verdict must emit exactly one line: {lines:?}"
        );
        assert!(
            lines[0].contains("floor contract finalized"),
            "held line missing its verdict text: {lines:?}"
        );
    }

    #[test]
    fn floor_finalization_disposition_case_a_scoped_worker_is_absent_never_refused() {
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction
            )
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert_eq!(
            lines.len(),
            1,
            "absence must emit exactly one counted line: {lines:?}"
        );
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[scoped-worker-by-construction]"),
            "scoped absence must name its reason, not just say nothing: {lines:?}"
        );
        assert!(
            !lines[0].contains("REFUSED"),
            "scoped-by-construction absence must never read as a refusal: {lines:?}"
        );
    }

    #[test]
    fn floor_finalization_disposition_case_b_incidental_absence_is_distinct_from_scoped() {
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::IncidentalAbsence,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(FloorFinalizationAbsenceReason::IncidentalAbsence)
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[incidental-absence]"),
            "incidental absence must carry its own distinct marker: {lines:?}"
        );
        // The discriminating half of the control: A and B must never render identically,
        // or a reader is back to being unable to tell which one happened.
        let scoped_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            ));
        assert_ne!(
            lines, scoped_lines,
            "scoped-by-construction and incidental absence must render as distinguishable lines"
        );
    }

    #[test]
    fn floor_finalization_disposition_plan_declared_absence_is_distinct_from_incidental() {
        // Review 49917 (cursor/composer-2.5): the regen/plan-artifact/falsifier/
        // native-cache-cold plans always declare `NoFinalizationDeclared {}` — that is a
        // PLAN fact, expected on every one of their runs, and must not render as
        // `incidental-absence`, which exists to flag runs that were NOT supposed to skip.
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::PlanDeclaresNoFinalization,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::PlanDeclaresNoFinalization
            )
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[plan-declares-no-finalization]"),
            "plan-declared absence must carry its own distinct marker: {lines:?}"
        );
        let incidental_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::IncidentalAbsence,
            ));
        let scoped_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            ));
        assert_ne!(
            lines, incidental_lines,
            "a plan that always declares no finalization must not dilute the incidental bucket"
        );
        assert_ne!(lines, scoped_lines);
    }

    #[test]
    fn floor_finalization_disposition_undeclared_absence_still_emits_a_counted_line() {
        // Inverse of the defect an earlier revision of this PR reintroduced (review
        // 2026-08-07, smart-badger-549): a caller with no opinion on WHY finalization is
        // absent still cannot construct a silent outcome, because `absence_reason` is a
        // required `FloorFinalizationAbsenceReason`, not an `Option` a caller could pass
        // `None` for. `Undeclared` is that caller's only honest choice, and it renders
        // its own distinct, non-empty line — never nothing, and never mistaken for A or B.
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::Undeclared,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(FloorFinalizationAbsenceReason::Undeclared)
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert_eq!(
            lines.len(),
            1,
            "undeclared absence must still emit exactly one counted line: {lines:?}"
        );
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[undeclared]"),
            "undeclared absence must name itself, not read as A or B: {lines:?}"
        );
        let scoped_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            ));
        let incidental_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::IncidentalAbsence,
            ));
        assert_ne!(lines, scoped_lines);
        assert_ne!(lines, incidental_lines);
    }

    #[test]
    fn run_walk_emits_the_finalization_disposition_through_the_injected_sink() {
        // Discriminating control for "nothing proves the verdict is emitted" (review
        // 2026-08-07, smart-badger-549): this calls the REAL run_walk, not a pure
        // disposition function, and inspects the REAL bytes written through the sink
        // production passes as stderr. Deleting or bypassing the
        // `emit_floor_finalization_disposition` call inside `run_walk` reds this test;
        // testing `floor_finalization_disposition_lines` alone cannot.
        let mut sink: Vec<u8> = Vec::new();
        let _outcome = run_walk(
            &[],
            TEST_PLAN_SITE,
            &[],
            &[],
            None,
            FloorFinalizationAbsenceReason::IncidentalAbsence,
            &mut sink,
            None,
            None,
            None,
            &RealizationConcurrency::for_walk(1).expect("test schedule"),
            None,
            FalsifierSelfHostWetBudgets::default(),
            FloorBatchStopPolicy::StopBeforeDependents,
            None,
            None,
            false,
            None,
        );
        let emitted = String::from_utf8(sink).expect("emitted bytes must be valid utf-8");
        assert!(
            emitted.contains("FLOOR-FINALIZATION-ABSENT[incidental-absence]"),
            "run_walk must actually write the disposition line through its sink, not just compute it: {emitted:?}"
        );
    }

    #[test]
    fn floor_finalization_disposition_refused_still_carries_the_law_message() {
        // Sanity: wiring the disposition wrapper through validate_floor_finalization
        // preserves the existing REFUSED law text — this PR changes visibility, never
        // the laws' semantics (constraint from the brief).
        let fin = test_floor_finalization();
        let disposition = floor_finalization_disposition(
            Some(&fin),
            FloorFinalizationAbsenceReason::Undeclared,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        match &disposition {
            FloorFinalizationDisposition::Refused(refusals) => {
                assert!(
                    !refusals.is_empty(),
                    "empty batch records must refuse unexecuted obligations"
                );
            }
            other => panic!("expected Refused with no batch records, got {other:?}"),
        }
        let lines = floor_finalization_disposition_lines(&disposition);
        assert!(
            lines
                .iter()
                .all(|l| l.starts_with("FLOOR-FINALIZATION-REFUSED: ")),
            "every refusal line must carry the located, typed marker: {lines:?}"
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
    fn walk_terminal_detail_carries_located_refusal_not_bare_exit_code() {
        let detail = walk_terminal_detail(
            true,
            &[
                "batch=3 fn=discovery-corpus detail=25 of 8158 discovery witness(es) failed"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("batch=3 fn=discovery-corpus"),
            "the terminal receipt must carry the located batch refusal, got: {detail}"
        );
        assert!(
            detail.contains("mode=WitnessRed"),
            "the terminal receipt must carry the typed mode, got: {detail}"
        );
        assert!(
            !detail.trim().eq("walk terminal exit code 1"),
            "a bare exit-code string is not a located refusal: {detail}"
        );
        assert_eq!(
            walk_terminal_detail(false, &[], &[], false, true),
            "walk terminal exit code 0"
        );
    }

    #[test]
    fn walk_terminal_detail_surfaces_post_walk_compile_clean_refusals() {
        let detail = walk_terminal_detail(true, &[], &[], true, false);
        assert!(detail.contains("compile_clean_over_budget"));
        assert!(detail.contains("compile_clean_cost_drift_receipt_refused"));
    }

    /// Receipted cold-corpus shape: ten typed witness findings plus one eval-budget fact
    /// and a walk soft-deadline interrupt must emit WitnessRed, not BudgetExceeded-only mode.
    #[test]
    fn walk_terminal_mode_witness_findings_outrank_budget_interrupt() {
        let detail = walk_terminal_detail(
            true,
            &[
                "batch=2 fn=discovery-corpus detail=10 of 8158 discovery witness(es) failed: \
                 english_emit_add_prose_holds StaleKnownRed; \
                 realization_vocab_live_corpus_receipt_holds returned Bool(false)"
                    .to_string(),
                "batch=2 fn=discovery-corpus detail=3 of 8158 discovery witness(es) failed: \
                 cannot cast List to List"
                    .to_string(),
                "walk reached its soft deadline before batch 9: deadline_ms=9000000 \
                 elapsed_ms=9000123"
                    .to_string(),
                "batch=7 fn=explicit-corpus detail=wave1_gate1_d eval budget exceeded: \
                 180002ms elapsed > 180000ms substrate long lane budget"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("mode=WitnessRed"),
            "semantic findings must set terminal mode, got: {detail}"
        );
        assert!(
            !detail.contains("mode=BudgetExceeded"),
            "budget interrupt must not collapse terminal mode: {detail}"
        );
        assert!(
            detail
                .contains("ci_failure_class_arm=FloorFailed{class:Structural{reason:WitnessRed}}"),
            "arm must carry WitnessRed reason: {detail}"
        );
        assert!(
            detail.contains("soft deadline"),
            "interrupt evidence must remain in detail: {detail}"
        );
        assert!(
            detail.contains("eval budget exceeded"),
            "budget lower bound must remain in detail: {detail}"
        );
    }

    #[test]
    fn walk_terminal_mode_budget_only_interrupt_stays_budget_exceeded() {
        let detail = walk_terminal_detail(
            true,
            &[
                "walk reached its soft deadline before batch 3: deadline_ms=9000000 \
                 elapsed_ms=9000123"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("mode=BudgetExceeded"),
            "a lone deadline interrupt with no witness findings stays BudgetExceeded: {detail}"
        );
    }

    /// The discriminating control for dropping the `batch_rows > 1` short-circuit. Two
    /// batch rows that are BOTH pure budget interrupts must stay BudgetExceeded: the count
    /// arm classified them WitnessRed, reporting a cost interrupt as a semantic verdict —
    /// the mirror of the defect this classifier exists to fix. The sibling budget-only test
    /// above carries zero batch rows, so it cannot reach this arm; without this case the
    /// removal is indistinguishable from never having had the bug.
    #[test]
    fn walk_terminal_mode_two_budget_only_batch_rows_stay_budget_exceeded() {
        let detail = walk_terminal_detail(
            true,
            &[
                "batch=7 fn=explicit-corpus detail=wave1_gate1_d eval budget exceeded: \
                 180002ms elapsed > 180000ms substrate long lane budget"
                    .to_string(),
                "batch=8 fn=explicit-corpus detail=wave2_gate3_a eval budget exceeded: \
                 180011ms elapsed > 180000ms substrate long lane budget"
                    .to_string(),
                "walk reached its soft deadline before batch 9: deadline_ms=9000000 \
                 elapsed_ms=9000123"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("mode=BudgetExceeded"),
            "two budget-only batch rows carry no semantic finding: {detail}"
        );
        assert!(
            !detail.contains("mode=WitnessRed"),
            "row count must not stand in for the semantic property: {detail}"
        );
    }

    #[test]
    fn push_ordinary_receipt_write_refusals_locates_each_failed_writer() {
        let mut details = Vec::new();
        push_ordinary_receipt_write_refusals(&mut details, true, false, true, true, false, true);
        assert_eq!(details.len(), 2);
        assert!(details
            .iter()
            .any(|d| d.contains("batch wall receipt write refused")));
        assert!(details
            .iter()
            .any(|d| d.contains("floor component receipt write refused")));
    }

    #[test]
    fn emit_worker_terminal_before_return_journals_pre_walk_refusal() {
        let dir = std::env::temp_dir().join(format!(
            "claim-executor-worker-terminal-before-return-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&dir);
        let terminal = dir.join("worker-terminal.tsv");
        let journal = dir.join("floor-phase-journal.tsv");
        let _ = fs::remove_file(&terminal);
        let _ = fs::remove_file(&journal);
        fs::write(
            &terminal,
            "refused\tscoped witness scheduling refusal: fixture detail\n",
        )
        .expect("write worker terminal fixture");
        std::env::set_var(FLOOR_WORKER_TERMINAL_ENV, &terminal);
        std::env::set_var(FLOOR_PHASE_JOURNAL_ENV, &journal);
        let code = emit_worker_terminal_before_return(ExitCode::from(1));
        std::env::remove_var(FLOOR_WORKER_TERMINAL_ENV);
        std::env::remove_var(FLOOR_PHASE_JOURNAL_ENV);
        assert_eq!(code, ExitCode::from(1));
        let persisted = fs::read_to_string(&journal).expect("walk-terminal journal row");
        let _ = fs::remove_dir_all(&dir);
        assert!(
            persisted.contains("\twalk-terminal\tfailed\tscoped witness scheduling refusal"),
            "pre-walk worker refusal must reach the durable journal: {persisted:?}"
        );
    }

    fn outcome(entry: &str, function: &str, o: ClaimOutcome) -> DiscoveryWitnessOutcome {
        DiscoveryWitnessOutcome {
            entry: entry.to_string(),
            module_path: "test.m".to_string(),
            function: function.to_string(),
            outcome: o,
            execution_leg: "InterpretedLeg".to_string(),
        }
    }

    fn stale_known_red_result(refusal: Option<ExpectationRefusal>) -> ClaimResult {
        ClaimResult {
            function: "discovery-corpus".to_string(),
            entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
            ok: refusal.is_none(),
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: runtime_unit_count_unavailable("test fixture"),
            witness_row_costs: Vec::new(),
            expectation_refusal: refusal,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        }
    }

    /// THE DISCRIMINATING PAIR for the expectation axis. Either half alone is satisfiable by
    /// a broken mechanism: a classifier that never refuses passes the still-red half, and one
    /// that always refuses passes the unexpected-green half. Both must hold at once.
    #[test]
    fn expected_red_still_red_is_agreement_and_unexpected_green_refuses() {
        let expected_red = vec![(
            "src/v2/test/claim/long/direct_rust_door_production_group_test.dag".to_string(),
            "direct_rust_door_production_group_closing_expectation_holds".to_string(),
        )];

        // HALF ONE — still red is AGREEMENT: not a failure, no refusal, and the component
        // receipt reports no failure mode at all, so the alert has nothing to notify on.
        let still_red = classify_witness_expectations(
            &[outcome(
                "src/v2/test/claim/long/direct_rust_door_production_group_test.dag",
                "direct_rust_door_production_group_closing_expectation_holds",
                ClaimOutcome::Fail,
            )],
            &expected_red,
        );
        assert_eq!(still_red.agreements.len(), 1);
        assert!(still_red.unexpected_failures.is_empty());
        assert_eq!(still_red.refusal(), None);
        let (mode, _) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![stale_known_red_result(
                still_red.refusal(),
            )]));
        assert_eq!(
            mode, "none",
            "a known-red witness holding its quarantine must not reach the alert as a failing component"
        );

        // HALF TWO — unexpected green REFUSES, typed and naming the row to delete. It must
        // not be `WitnessRed`: an un-quarantine and a regression have opposite remedies and
        // must not share a class signature.
        let went_green = classify_witness_expectations(
            &[outcome(
                "src/v2/test/claim/long/direct_rust_door_production_group_test.dag",
                "direct_rust_door_production_group_closing_expectation_holds",
                ClaimOutcome::Pass,
            )],
            &expected_red,
        );
        assert!(went_green.agreements.is_empty());
        let refusal = went_green
            .refusal()
            .expect("an expected-red witness that ran green must refuse, never pass quietly");
        assert!(
            matches!(&refusal, ExpectationRefusal::StaleKnownRed { witnesses } if witnesses.len() == 1),
            "the stale arm, not the coverage arm: this witness WAS observed, running green"
        );
        let (mode, detail) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![stale_known_red_result(
                Some(refusal),
            )]));
        assert_eq!(mode, STALE_KNOWN_RED_MODE);
        assert_ne!(mode, "WitnessRed");
        assert_ne!(mode, "none");
        assert!(
            detail.contains("direct_rust_door_production_group_closing_expectation_holds"),
            "the refusal must name the identity to un-quarantine, got: {detail}"
        );
    }

    /// ONE ROW PER DISPOSITION of the required result table, each discriminating.
    ///
    /// The table exists because the first classifier computed `green = (outcome == Pass)` and
    /// swept every other outcome into the agreement arm. Each row below is an outcome that
    /// arm would have blessed as "the quarantine held" while the assertion produced no verdict
    /// at all.
    #[test]
    fn expected_red_disposition_table_distinguishes_every_outcome() {
        let entry = "src/v2/test/claim/long/direct_rust_door_production_group_test.dag";
        let f = "direct_rust_door_production_group_closing_expectation_holds";
        let expected_red = vec![(entry.to_string(), f.to_string())];

        let row = |o: ClaimOutcome| {
            classify_witness_expectations_in(
                &[outcome(entry, f, o)],
                &expected_red,
                &[(entry.to_string(), f.to_string())],
            )
        };

        // ROW 1 — assertion executed, returned false: the ONLY agreement.
        let t = row(ClaimOutcome::Fail);
        assert_eq!(t.agreements.len(), 1);
        assert_eq!(t.hard_failures(), 0);
        assert!(t.evidence_absent.is_empty());

        // ROW 2 — assertion executed, returned true: stale quarantine, a typed refusal.
        let t = row(ClaimOutcome::Pass);
        assert!(t.agreements.is_empty());
        assert!(t.refusal().is_some());

        // ROW 3 — timed out: a BUDGET failure. An interruption plus a lower bound on cost is
        // never a semantic verdict, so it must not read as a quarantine holding.
        let t = row(ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms: 5001,
            budget_ms: 5000,
            // The VARIANT now says which case this is, so 5001 is a lower bound by
            // construction: the completed row is a different arm the fixture cannot reach by
            // omitting a field.
            kind: BudgetKind::Cpu,
        });
        assert!(
            t.agreements.is_empty(),
            "a budget kill must never count as agreement — the assertion did not produce a verdict"
        );
        assert_eq!(
            t.refusal(),
            None,
            "a budget kill is not a stale quarantine either"
        );
        assert_eq!(
            t.expected_red_without_verdict,
            vec![(
                entry.to_string(),
                f.to_string(),
                ExpectedRedDisposition::BudgetFailure
            )]
        );
        assert_eq!(t.hard_failures(), 1, "it must red the batch");

        // ROW 4 — interpreter fault, and ROW 4b — non-Bool referent: no verdict produced.
        // Row 5 of the table (refused BEFORE evaluation) shares this arm today because
        // `run_claim` formats the typed error to a String; that is declared on the variant,
        // not papered over by sniffing the message.
        for o in [
            ClaimOutcome::RuntimeError {
                cause: v1_compiler::cli_run::WitnessRuntimeCause::NoSuchVariable,
                message: "undefined variable: X".into(),
            },
            ClaimOutcome::NotBool {
                got: "Record".into(),
            },
        ] {
            let t = row(o);
            assert!(t.agreements.is_empty());
            assert_eq!(
                t.expected_red_without_verdict
                    .iter()
                    .map(|(_, _, d)| d.clone())
                    .collect::<Vec<_>>(),
                vec![ExpectedRedDisposition::InfrastructureOrReferentFailure]
            );
            assert_eq!(t.hard_failures(), 1);
        }

        // ROW 6 — no observation at all: ABSENT EVIDENCE, which is not a verdict in either
        // direction. Not agreement and not a witness failure — but it REFUSES, on its own
        // coverage mode. This row previously asserted `refusal() == None`, which is exactly the
        // hole the coverage arm closes: absence is not a verdict, and it is also not silence.
        let t = classify_witness_expectations_in(
            &[],
            &expected_red,
            &[(entry.to_string(), f.to_string())],
        );
        assert!(t.agreements.is_empty());
        assert_eq!(t.hard_failures(), 0, "the remedy is coverage, not code");
        assert_eq!(t.evidence_absent, vec![(entry.to_string(), f.to_string())]);
        assert!(
            matches!(
                t.refusal(),
                Some(ExpectationRefusal::ExpectedRedEvidenceAbsent { .. })
            ),
            "the coverage arm, distinct from the stale arm rows 2 and 5 exercise"
        );
    }

    /// The regression this table was written for, stated as one assertion: under the `!= Pass`
    /// shortcut EVERY one of these blessed the quarantine. None may.
    #[test]
    fn no_verdict_outcome_is_ever_agreement_for_an_expected_red_witness() {
        let entry = "e.dag";
        let f = "witness_holds";
        let expected_red = vec![(entry.to_string(), f.to_string())];
        for o in [
            ClaimOutcome::BudgetInterrupted {
                elapsed_at_least_ms: 1,
                budget_ms: 1,
                kind: BudgetKind::Wall,
            },
            ClaimOutcome::RuntimeError {
                cause: v1_compiler::cli_run::WitnessRuntimeCause::TypeError,
                message: "boom".into(),
            },
            ClaimOutcome::NotBool { got: "Int".into() },
        ] {
            let t = classify_witness_expectations(&[outcome(entry, f, o.clone())], &expected_red);
            assert!(
                t.agreements.is_empty() && t.hard_failures() == 1,
                "{o:?} must produce no agreement and must red the batch"
            );
        }
        // …while the one that genuinely ran and was false still does agree, so the fix is not
        // simply "nothing agrees any more".
        let t =
            classify_witness_expectations(&[outcome(entry, f, ClaimOutcome::Fail)], &expected_red);
        assert_eq!(t.agreements.len(), 1);
        assert_eq!(t.hard_failures(), 0);
    }

    /// Expectation is a property of the WITNESS, so a quarantined function and a green
    /// sibling in the SAME FILE are classified independently — and a mixed batch stays
    /// ordinary instead of reverting to one polarity for everything in it.
    #[test]
    fn expected_red_is_function_grain_not_file_or_batch_grain() {
        let entry = "src/v2/test/claim/manual/english_emit_add_test.dag";
        let expected_red = vec![(
            entry.to_string(),
            "logic_complement_truth_table".to_string(),
        )];

        let tally = classify_witness_expectations(
            &[
                outcome(entry, "logic_complement_truth_table", ClaimOutcome::Fail),
                outcome(entry, "logic_sibling_that_must_hold", ClaimOutcome::Pass),
                outcome(
                    "other/witness_test.dag",
                    "unrelated_holds",
                    ClaimOutcome::Fail,
                ),
            ],
            &expected_red,
        );
        assert_eq!(tally.agreements.len(), 1, "the quarantined fn is agreement");
        assert_eq!(
            tally.refusal(),
            None,
            "its green sibling is not a stale quarantine"
        );
        assert_eq!(
            tally.unexpected_failures.len(),
            1,
            "an ordinary red in a batch containing a known-red row is still an ordinary red — \
             the batch-wide flag this replaced would have inverted it to a pass"
        );

        // The green SIBLING going red is an ordinary failure, not agreement: file placement
        // never confers polarity.
        let sibling_red = classify_witness_expectations(
            &[outcome(
                entry,
                "logic_sibling_that_must_hold",
                ClaimOutcome::Fail,
            )],
            &expected_red,
        );
        assert!(sibling_red.agreements.is_empty());
        assert_eq!(sibling_red.unexpected_failures.len(), 1);
    }

    /// The one surviving batch-wide consult, kept narrow twice over: a corpus resolve refuse
    /// produces no per-witness outcomes, so agreement requires that every entry declared
    /// `ExpectTypedPreVerdictRefusal` — an ordinary known-red row is NOT enough.
    ///
    /// The discriminating half is the second block. A batch of ordinary known-red rows and a
    /// batch of declared pre-verdict rows are both "all expected-red"; only the second may
    /// invert a refuse. Without that block this test would pass against the defect it exists
    /// to hold closed, because the defect's population is a superset of this one's.
    /// The pre-verdict arm is NON-GREEN, and this OBSERVES the result rather than restating
    /// the constant.
    ///
    /// The first shape of this test compared string constants to each other and to a literal,
    /// while its doc comment claimed "if someone restores the green arm, the first assertion
    /// fails". It did not: nothing executed the arm, so flipping `ok` back to `true` left every
    /// assertion green — a stated regression control that did not exist. That is the same
    /// defect class as a stated identity join that was really a length agreement, and it was
    /// caught the same way, by someone checking the claim against what the code drives. The
    /// arm's construction is now extracted to one function and this asserts its `ok`.
    #[test]
    fn declared_pre_verdict_refusal_is_non_green_and_carries_its_own_mode() {
        let r = pre_verdict_unverified_claim_result("known-red probe", 3, "resolve refused");

        // THE REGRESSION CONTROL, and it now observes the value: restoring the green arm
        // flips this and the test fails.
        assert!(
            !r.ok,
            "a declaration classifies a stop; it does not verify it"
        );

        // THE MODE MUST REACH THE RECEIPT OFF THE VALUE, not off the prose. Carried only in
        // `function`/`detail`, `batch_failure_mode_and_detail` fell through to
        // `falsifier_failure_mode` and reported these batches as ordinary "WitnessRed" while
        // the .dag vocabulary said Refused — one mode in two representations, the second
        // guessed back from a string, which is the defect this PR exists to remove.
        // cursor review 50221 found it; this assertion is what would have caught it.
        let (mode, receipt_detail) = batch_failure_mode_and_detail(&batch_record_for_test(vec![
            pre_verdict_unverified_claim_result("known-red probe", 3, "resolve refused"),
        ]));
        assert_eq!(
            mode, EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE,
            "the receipt must classify structurally, never as WitnessRed"
        );
        assert_ne!(mode, "WitnessRed");
        assert!(receipt_detail.contains(EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE));
        assert!(matches!(
            &r.expectation_refusal,
            Some(ExpectationRefusal::PreVerdictUnverified { .. })
        ));
        assert!(
            r.detail.contains(EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE),
            "the refusal must be typed at the mode, not left as prose"
        );
        assert!(
            r.detail.contains("resolve refused"),
            "the located cause is carried"
        );
        assert!(
            r.detail.contains('3'),
            "the declared entry count is carried"
        );
        assert!(r
            .function
            .contains(EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE));
        // It is a refusal, not a verdict about any witness: no per-witness populations.
        // (The typed refusal itself IS carried — asserted above; an earlier
        // `expectation_refusal == None` here contradicted that assertion and made the whole
        // test unexecutable, so it never observed anything it claimed to.)
        assert!(r.witness_row_costs.is_empty());

        // Distinct from BOTH siblings on the same axis — different remedies, different modes.
        assert_ne!(
            EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE,
            EXPECTED_RED_EVIDENCE_ABSENT_MODE
        );
        assert_ne!(
            EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE,
            ExpectationRefusal::StaleKnownRed {
                witnesses: Vec::new()
            }
            .mode()
        );
    }

    #[test]
    fn resolve_refuse_agreement_requires_every_entry_expect_a_pre_verdict_refusal() {
        let pre_verdict = (
            "src/v2/test/claim/manual/english_emit_add_test.dag".to_string(),
            "logic_complement_truth_table".to_string(),
        );
        let ordinary_known_red = (
            "dag/test/claim/observation_raw_print_retirement_acceptance_test.dag".to_string(),
            "observation_emit_frontier_is_zero".to_string(),
        );
        let ordinary = (
            "dag/test/claim/design_register_lift_parity_witness_test.dag".to_string(),
            "design_register_lift_parity_holds".to_string(),
        );
        let pre_verdict_roster = vec![pre_verdict.clone()];

        assert!(batch_entries_all_expect_pre_verdict_refusal(
            &[pre_verdict.clone()],
            &pre_verdict_roster
        ));
        assert!(
            !batch_entries_all_expect_pre_verdict_refusal(
                &[pre_verdict.clone(), ordinary.clone()],
                &pre_verdict_roster
            ),
            "one ordinary entry must keep a resolve refuse loud"
        );
        assert!(!batch_entries_all_expect_pre_verdict_refusal(
            &[ordinary.clone()],
            &pre_verdict_roster
        ));
        assert!(!batch_entries_all_expect_pre_verdict_refusal(
            &[],
            &pre_verdict_roster
        ));

        // THE DISCRIMINATION. Both rows below are known-red; only one declared that its
        // agreement is a stop before any verdict. An arbitrary resolve failure in a batch of
        // the other must NOT report every quarantine as holding.
        let expected_red = vec![pre_verdict.clone(), ordinary_known_red.clone()];
        assert!(
            batch_entries_all_in(&[ordinary_known_red.clone()], &expected_red),
            "the ordinary row IS expected-red — this is the population the defect consulted"
        );
        assert!(
            !batch_entries_all_expect_pre_verdict_refusal(
                &[ordinary_known_red],
                &pre_verdict_roster
            ),
            "but it did NOT declare a pre-verdict refusal, so a resolve failure in its batch is \
             an infrastructure fact about the run and proves neither the red nor the quarantine"
        );
    }

    /// THE BUDGET AXIS, separated. A witness obtains its long eval ceiling from its OWN
    /// declared budget, without joining the long-lane BATCH roster — which is what forced an
    /// expensive known-red root to occur twice in the schedule, the second time on a batch
    /// that expects green.
    #[test]
    fn declared_long_budget_arms_ceiling_without_joining_the_long_lane_batch() {
        let known_red_long = "src/v2/test/claim/long/direct_rust_door_production_group_test.dag";
        // The union `witness_long_eval_budget_entries` produces: this path is present because
        // its ADMISSION row declares SubstrateLongLaneEvalBudget, not because it is on the
        // long-lane batch roster.
        let budgets = FalsifierSelfHostWetBudgets {
            substrate_long_lane_entry_paths: vec![known_red_long.to_string()],
            substrate_long_lane_eval_budget_ms: Some(180_000),
            expected_red_witnesses: vec![(
                known_red_long.to_string(),
                "direct_rust_door_production_group_closing_expectation_holds".to_string(),
            )],
            ..Default::default()
        };
        let batch = vec![(
            known_red_long.to_string(),
            "direct_rust_door_production_group_closing_expectation_holds".to_string(),
        )];

        // Both axes hold at once on one witness in one batch: the long ceiling AND
        // expected-red polarity. Under the fused shape this pair was inexpressible.
        assert_eq!(
            select_discovery_batch_budgets(
                ExecutionMode::Hermetic,
                &batch,
                Some(5_000),
                &budgets
            )
            .eval_budget_ms,
            Some(180_000),
            "the declared budget must arm the ceiling, or a known-red row reds at 5s for the wrong reason"
        );
        assert!(batch_entries_all_in(
            &batch,
            &budgets.expected_red_witnesses
        ));
    }

    /// P0-1. ABSENT EVIDENCE STOPS THE LINE. A rostered expected-red witness that produced no
    /// observation is neither agreement nor failure — and the arm that matters is that it is
    /// also not SILENCE. The first shape of this bin only wrote it to stderr, so a batch whose
    /// admitted red control had quietly stopped executing still reported green: the roster
    /// claimed a discriminating red existed, the run produced no evidence either way, and the
    /// green report was read as though it had.
    #[test]
    fn absent_expected_red_evidence_refuses_as_coverage_never_as_a_verdict() {
        let entry = "src/v2/test/claim/long/direct_rust_door_production_group_test.dag";
        let f = "direct_rust_door_production_group_closing_expectation_holds";
        let expected_red = vec![(entry.to_string(), f.to_string())];
        let rostered = expected_red.clone();

        // Nothing came back for the rostered row.
        let tally = classify_witness_expectations_in(&[], &expected_red, &rostered);
        assert_eq!(tally.evidence_absent.len(), 1);
        assert!(
            tally.agreements.is_empty() && tally.unexpected_failures.is_empty(),
            "absence is not a verdict in either direction"
        );
        assert_eq!(
            tally.hard_failures(),
            0,
            "and it must not be counted as a witness failure — the remedy is coverage, not code"
        );

        let refusal = tally
            .refusal()
            .expect("absence must refuse; logging it and passing is coverage by illusion");
        assert!(matches!(
            refusal,
            ExpectationRefusal::ExpectedRedEvidenceAbsent { .. }
        ));
        let (mode, detail) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![stale_known_red_result(
                Some(refusal),
            )]));
        assert_eq!(mode, EXPECTED_RED_EVIDENCE_ABSENT_MODE);
        assert_ne!(mode, STALE_KNOWN_RED_MODE, "nobody observed it passing");
        assert_ne!(mode, "none");
        assert!(detail.contains(f), "the refusal must name what did not run");

        // POSITIVE CONTROL: the same roster, observed. No coverage refusal.
        let observed = classify_witness_expectations_in(
            &[outcome(entry, f, ClaimOutcome::Fail)],
            &expected_red,
            &rostered,
        );
        assert!(observed.evidence_absent.is_empty());
        assert_eq!(observed.refusal(), None);
        assert_eq!(observed.agreements.len(), 1);
    }

    /// P1, second half. The admission authority is an IDENTITY JOIN over witness outcomes, so a
    /// bucket population that agrees in COUNT while disagreeing in identity must still refuse.
    /// A count equality alone cannot see this; that is the whole reason the join replaced it.
    #[test]
    fn non_pass_join_matches_identities_not_merely_counts() {
        let entry = "src/v2/test/claim/manual/english_emit_add_test.dag";
        let expected_red = vec![(
            entry.to_string(),
            "logic_complement_truth_table".to_string(),
        )];
        let mut tally = classify_witness_expectations(
            &[outcome(
                entry,
                "logic_complement_truth_table",
                ClaimOutcome::Fail,
            )],
            &expected_red,
        );
        assert!(tally.non_pass_join_is_complete());
        assert!(still_red_batch_passes(&tally, 1));

        // Same COUNT on both sides — one seen non-pass, one agreement — but the agreement
        // names a different witness. The count equality is satisfied; the join is not.
        tally.agreements = vec![(entry.to_string(), "some_other_witness".to_string())];
        assert_eq!(
            tally.classified_non_pass.len(),
            tally.accounted_non_pass().len()
        );
        assert!(
            !tally.non_pass_join_is_complete(),
            "equal counts over different identities must refuse"
        );
        assert!(
            !still_red_batch_passes(&tally, 1),
            "the join, not the count, is the admission authority"
        );
    }

    /// P1. The still-red pass arm joins on an EXACT count, not `<=`. A failure that arrives
    /// without a per-witness identity must refuse rather than be read as "nothing failed" —
    /// the empty-observation narrow, applied to the batch's own accounting.
    #[test]
    fn still_red_pass_arm_requires_exact_failure_accounting() {
        let entry = "src/v2/test/claim/manual/english_emit_add_test.dag";
        let f = "logic_complement_truth_table";
        let expected_red = vec![(entry.to_string(), f.to_string())];
        let tally =
            classify_witness_expectations(&[outcome(entry, f, ClaimOutcome::Fail)], &expected_red);

        assert_eq!(tally.hard_failures(), 0);
        assert_eq!(tally.classified_non_pass.len(), 1);

        // Lockstep: one classified non-pass, one summary failure. Passes.
        assert!(still_red_batch_passes(&tally, 1));
        // An extra summary failure with no matching per-witness outcome must REFUSE. Under
        // `<=` this returned true, so unjoinable failures were absorbed into a green batch.
        assert!(
            !still_red_batch_passes(&tally, 2),
            "an unjoinable failure must refuse, never be absorbed"
        );
        // And fewer, which is the same accounting break in the other direction.
        assert!(!still_red_batch_passes(&tally, 0));

        // A real unexpected failure still reds regardless of the accounting.
        let with_red = classify_witness_expectations(
            &[outcome(entry, "an_ordinary_sibling", ClaimOutcome::Fail)],
            &expected_red,
        );
        assert_eq!(with_red.hard_failures(), 1);
        assert!(!still_red_batch_passes(&with_red, 1));
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
        // A non-self-host wet witness must not inherit the 600s whole-receipt ceiling.
        assert!(!discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/codex_package_delivery_wet_witness_test.dag".into(),
                "materialize_codex_runtime_bundle_produces_native_executable_holds".into()
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

    fn batch_record_for_test(results: Vec<ClaimResult>) -> BatchRecord {
        let unit_count = results.len() as u128;
        BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count,
            runtime_units: FloorRuntimeUnitCount::Observed { units: unit_count },
            results,
            label: "batch-under-test".to_string(),
            is_wet: false,
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
            entry: "some_witness_test.dag".to_string(),
            ok: false,
            detail: "wording that mentions no budget phrase at all".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: Some(BudgetRefusal {
                elapsed_ms: 900_001,
                budget_ms: 900_000,
                kind: BudgetKind::Wall,
                completion: v1_compiler::cli_run::BudgetCompletion::Interrupted,
            }),
            host_dependency_refusal: None,
            resolve_realization: None,
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
            entry: "other_witness_test.dag".to_string(),
            ok: false,
            detail: "returned Bool(false)".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        };
        let (mode, _) = batch_failure_mode_and_detail(&batch_record_for_test(vec![plain]));
        assert_eq!(mode, "WitnessRed");
    }

    /// A host dependency refusal must classify as HostDependencyAbsent from the VALUE,
    /// not WitnessRed — same structural rule as budget_kill_classifies_structurally_not_by_message_text.
    #[test]
    fn host_dependency_absent_classifies_structurally_not_as_witness_red() {
        let npm_absent = ClaimResult {
            function: "materialize_codex_runtime_bundle_produces_native_executable_holds"
                .to_string(),
            entry: "dag/test/claim/codex_package_delivery_wet_witness_test.dag".to_string(),
            ok: false,
            detail: "returned Bool(false) | HostDependencyAbsent{tool=npm,hint=apt install npm}"
                .to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: Some(HostDependencyRefusal {
                tool: "npm".to_string(),
                hint: "apt install npm".to_string(),
            }),
            resolve_realization: None,
        };
        assert_eq!(
            falsifier_failure_mode(&[npm_absent.detail.clone()]),
            "WitnessRed",
            "control: string classifier alone would misclassify"
        );
        let (mode, detail) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![npm_absent]));
        assert_eq!(mode, HOST_DEPENDENCY_ABSENT_MODE);
        assert!(detail.contains("HostDependencyAbsent{tool=npm"));
    }

    #[test]
    fn host_dependency_refusal_from_detail_parses_wire() {
        let parsed = host_dependency_refusal_from_detail(
            "returned Bool(false) | HostDependencyAbsent{tool=npm,hint=apt install npm}",
        )
        .expect("wire must parse");
        assert_eq!(parsed.tool, "npm");
        assert_eq!(parsed.hint, "apt install npm");
    }

    #[test]
    fn host_dependency_refusal_from_detail_anchors_last_wire_token() {
        let parsed = host_dependency_refusal_from_detail(
            "HostDependencyAbsent{tool=decoy,hint=x} | returned Bool(false) | HostDependencyAbsent{tool=npm,hint=apt install npm}",
        )
        .expect("last wire token must win");
        assert_eq!(parsed.tool, "npm");
        assert_eq!(parsed.hint, "apt install npm");
    }

    #[test]
    fn host_dependency_refusal_from_detail_refuses_brace_in_hint() {
        assert!(host_dependency_refusal_from_detail(
            "HostDependencyAbsent{tool=npm,hint=install {pkg} on runner}"
        )
        .is_none());
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
                "batch=3 fn=expensive_wet_witness detail=witness receipt wall budget exceeded: 707687ms elapsed > 600000ms whole-receipt budget".into()
            ]),
            "BudgetExceeded"
        );
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=3 fn=expensive_wet_witness detail=wet self-host receipt wall budget exceeded: 707687ms elapsed > 600000ms whole-receipt budget".into()
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
        assert!(!write_resolve_receipt_at(&base, &[], &[], None));
        assert!(!write_batch_wall_receipt_at(&base, &[]));
        let _ = fs::remove_file(&base);
    }

    /// TOTALITY, BOTH DIRECTIONS: a component the stop policy never reached appears in the
    /// receipt under its OWN planned identity, not a placeholder.
    ///
    /// This is the discriminating pair for the identity join. The plan below is the shape
    /// that matters: batch 0 is an ordinary claim that fails, batch 1 is the affected-set
    /// cold control (`predict_only`), and the stop policy means batch 1 never runs — so it
    /// exists only as a padded row. Before this fix that row was written with a literal
    /// "not reached" label and an "off" selection tag, which deleted the one property that
    /// IDENTIFIES the cold control (`gunbc.floor_component_receipt` role note keys on
    /// `predict_only`). `floor_affected_set_control_state` then answered `ControlAbsent`,
    /// conflating "the control was never enrolled" with "the control was enrolled and the
    /// line stopped before it ran" — different states with different remedies.
    ///
    /// The negative half is the point: asserting only that `predict_only` is present would
    /// also pass on a receipt that tagged EVERY padded row predict_only. So the batch-0
    /// side is asserted too — a real `off` component stays `off` — and the vanished
    /// placeholder literal is asserted absent.
    #[test]
    fn unreached_components_carry_planned_identity_not_placeholder() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let base = std::env::temp_dir().join(format!(
            "claim-executor-totality-{}-{}",
            std::process::id(),
            "unreached"
        ));
        let _ = fs::remove_dir_all(&base);

        // The PLAN: two components. Batch 1 is the cold control and is never reached.
        let batches: Vec<Vec<Runnable>> = vec![
            vec![Runnable::SingleClaim {
                entry: "dag/test/claim/some_gate_test.dag".to_string(),
                function: "some_gate_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }],
            vec![Runnable::DiscoveryBatch {
                source_roots: source_roots.clone(),
                scan_dirs: vec!["dag/test/claim".to_string()],
                explicit_entries: Vec::new(),
                native_bundle_entries: Vec::new(),
                exclude_substrings: Vec::new(),
                discovery_scope_dirs: Vec::new(),
                execution_mode: ExecutionMode::Hermetic,
                spawns_host_compiler: false,
            }],
        ];

        // The RUN: only batch 0 produced a record — the line stopped there.
        let batch_records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 1_000_000_000,
            clamp_ms: None,
            unit_count: 1,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 1 },
            results: Vec::new(),
            label: batch_heartbeat_label(&batches[0]),
            is_wet: false,
        }];

        assert!(write_floor_component_receipt_at(
            &base,
            &source_roots,
            &batch_records,
            &batches,
            UnreachedCause::StopPolicy,
        ));
        let body = fs::read_to_string(base.join("floor-component-receipt.json")).unwrap();

        // The unreached cold control is present AS the cold control.
        assert!(
            body.contains("\"selection\": \"predict_only\""),
            "the unreached cold control must keep the predict_only tag that identifies it: {body}"
        );
        assert!(
            body.contains("some_gate_holds"),
            "the reached component keeps its own label: {body}"
        );
        // Its outcome is honestly Skipped with the stop-policy cause — not fabricated Done.
        assert!(
            body.contains("\"outcome\": \"skipped\""),
            "an unreached component concludes Skipped: {body}"
        );
        // The placeholder identity is gone.
        assert!(
            !body.contains("\"label\": \"not reached\""),
            "the placeholder label must not survive: {body}"
        );
        // NEGATIVE HALF: a genuinely `off` component is not relabelled predict_only.
        assert!(
            body.contains("\"selection\": \"off\""),
            "batch 0 is an ordinary off component and must stay off: {body}"
        );

        let _ = fs::remove_dir_all(&base);
    }

    // Hard discovery failure must refuse the batch clamp — never mis-render as
    // FLOOR-BATCH-OVER-BUDGET via the legacy zero→one unit mapping.
    #[test]
    fn hard_discovery_error_refuses_batch_clamp_not_over_budget() {
        let discovery_fail = ClaimResult {
            function: "discovery-corpus".to_string(),
            entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
            ok: false,
            detail: "discovery corpus failed: resolve refused".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: runtime_unit_count_unavailable("resolve refused"),
            witness_row_costs: Vec::new(),
            budget_refusal: None,
            host_dependency_refusal: None,
            expectation_refusal: None,
            resolve_realization: None,
        };
        let units = aggregate_batch_runtime_units(&[discovery_fail]);
        assert!(
            matches!(units, FloorRuntimeUnitCount::Unavailable { .. }),
            "hard corpus error must mark units unavailable, got {units:?}"
        );
        // Positive control: a single gate row still contributes one observed unit.
        let gate = ClaimResult {
            function: "some_gate_holds".to_string(),
            entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            budget_refusal: None,
            host_dependency_refusal: None,
            expectation_refusal: None,
            resolve_realization: None,
        };
        assert_eq!(
            aggregate_batch_runtime_units(&[gate]),
            FloorRuntimeUnitCount::Observed { units: 1 }
        );
        // RED control: the old zero→one mapping would fabricate OverBudget on a slow wall
        // against overhead-only clamp when the corpus actually failed.
        let overhead_ms = 60_000u128;
        let rate_ms = 1_000u128;
        let wall_ms = 120_000u128;
        if let FloorRuntimeUnitCount::Observed { units: 1 } = units {
            let clamp = overhead_ms + 1 * rate_ms;
            assert!(
                wall_ms > clamp,
                "control: units=1 would have breached overhead-only clamp"
            );
        }
    }

    #[test]
    fn hard_discovery_batch_wall_receipt_is_clamp_refused_not_over_budget() {
        let base = std::env::temp_dir().join(format!(
            "claim-executor-clamp-refused-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        let records = vec![BatchRecord {
            batch_index: 2,
            wall_nanos: 600_000_000_000,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: runtime_unit_count_unavailable("discovery corpus failed: test"),
            results: Vec::new(),
            label: "discovery-corpus".to_string(),
            is_wet: false,
        }];
        assert!(write_batch_wall_receipt_at(&base, &records));
        let body = fs::read_to_string(base.join("floor-batch-wall-receipt.txt")).unwrap();
        assert!(body.contains("batch_3_units=unavailable"));
        assert!(body.contains("batch_3_verdict=ClampRefused"));
        assert!(!body.contains("OverBudget"));
        assert!(body.contains("over_budget_batches=0"));

        let _ = fs::remove_dir_all(&base);
    }

    /// A CHECKPOINT is not a CONCLUSION, and the receipt must say which one it is.
    ///
    /// Both causes collapse to the same `skipped` outcome by design, so asserting the
    /// outcome proves nothing about this distinction — the whole claim lives in the
    /// failure_mode, and it is asserted in BOTH directions on the same plan and the same
    /// records. Without the negative halves this test would pass on a writer that emitted
    /// one constant mode for every unreached row, which is precisely the pre-existing
    /// behaviour being corrected.
    ///
    /// Why it matters: a 170-minute SIGKILL leaves the last checkpoint on disk as the
    /// alert's only evidence. If its tail read `not_reached`, the alert would report a
    /// killed run as a plan that deliberately skipped those components under the stop
    /// policy — a wrong causal story about the exact incident the checkpoint exists to
    /// explain, and one that routes to the wrong owner.
    #[test]
    fn checkpoint_tail_is_run_incomplete_and_final_tail_is_not_reached() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let base = std::env::temp_dir().join(format!(
            "claim-executor-checkpoint-{}-{}",
            std::process::id(),
            "cause"
        ));
        let _ = fs::remove_dir_all(&base);

        let batches: Vec<Vec<Runnable>> = vec![
            vec![Runnable::SingleClaim {
                entry: "dag/test/claim/some_gate_test.dag".to_string(),
                function: "some_gate_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }],
            vec![Runnable::SingleClaim {
                entry: "dag/test/claim/other_gate_test.dag".to_string(),
                function: "other_gate_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }],
        ];
        let batch_records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 1_000_000_000,
            clamp_ms: None,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 1 },
            unit_count: 1,
            results: Vec::new(),
            label: batch_heartbeat_label(&batches[0]),
            is_wet: false,
        }];

        // MID-RUN: batch 1 has not concluded, and its fate is unknown.
        assert!(write_floor_component_receipt_at(
            &base,
            &source_roots,
            &batch_records,
            &batches,
            UnreachedCause::RunIncomplete,
        ));
        let checkpoint = fs::read_to_string(base.join("floor-component-receipt.json")).unwrap();
        assert!(
            checkpoint.contains("\"failure_mode\": \"run_incomplete\""),
            "a checkpoint's unreached tail carries run_incomplete: {checkpoint}"
        );
        assert!(
            checkpoint.contains("\"outcome\": \"pending\""),
            "a checkpoint's unreached tail is pending, not a fabricated skip: {checkpoint}"
        );
        assert!(
            !checkpoint.contains("\"outcome\": \"skipped\""),
            "a checkpoint must not manufacture skipped verdicts for unreached batches: {checkpoint}"
        );
        assert!(
            checkpoint.contains("\"disposition\": \"incomplete\""),
            "a checkpoint names the run-level incomplete fact: {checkpoint}"
        );

        // CONCLUDED: the same records, but the plan is over — the tail was genuinely skipped.
        assert!(write_floor_component_receipt_at(
            &base,
            &source_roots,
            &batch_records,
            &batches,
            UnreachedCause::StopPolicy,
        ));
        let concluded = fs::read_to_string(base.join("floor-component-receipt.json")).unwrap();
        assert!(
            concluded.contains("\"failure_mode\": \"not_reached\""),
            "a concluded walk's unreached tail carries not_reached: {concluded}"
        );
        assert!(
            !concluded.contains("\"failure_mode\": \"run_incomplete\""),
            "a concluded walk must not report itself as still in progress: {concluded}"
        );

        // The rename left no temp file behind for the artifact upload to trip over.
        let strays: Vec<_> = fs::read_dir(&base)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".tmp."))
            .collect();
        assert!(
            strays.is_empty(),
            "temp receipts must not survive: {strays:?}"
        );

        let _ = fs::remove_dir_all(&base);
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
                runtime_units: FloorRuntimeUnitCount::Observed { units: 3 },
                results: Vec::new(),
                label: "batch-0".to_string(),
                is_wet: false,
            },
            BatchRecord {
                batch_index: 1,
                wall_nanos: 1_000_000_000, // 1s
                clamp_ms: Some(2_000),     // 2s
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                results: Vec::new(),
                label: "batch-1".to_string(),
                is_wet: false,
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
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            results: Vec::new(),
            label: "batch-0".to_string(),
            is_wet: false,
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

    #[test]
    fn on_success_spawned_claims_move_to_warm_main_thread_only() {
        let runnable = Runnable::SingleClaim {
            entry: "dag/tools/merge_admission_walk.dag".to_string(),
            function: "stamp_tested_floor".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        };
        let units = group_batch_units(&[runnable]);
        let memo = std::collections::HashMap::new();
        let memo_entries = std::collections::HashSet::new();
        assert_eq!(
            batch_unit_lane(&units[0], &memo, &memo_entries),
            UnitLane::Spawned,
            "the shared lane authority remains unchanged"
        );
        assert_eq!(
            population_unit_lane(
                StagePopulation::OrdinaryBatch,
                &units[0],
                &memo,
                &memo_entries,
            ),
            UnitLane::Spawned,
            "ordinary claim placement must remain unchanged"
        );
        assert_eq!(
            population_unit_lane(
                StagePopulation::OnSuccessStage,
                &units[0],
                &memo,
                &memo_entries,
            ),
            UnitLane::MainThread,
            "green-only claim must consume the warm main-thread index"
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
                (Some("phase".to_string()), str_value(phase.to_string())),
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
            Value::Str(s) => Some(s.to_string()),
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
            Value::Str(s) => Some(s.to_string()),
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
            Value::Str(s) => Some(s.to_string()),
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
                    str_value(module_path.to_string()),
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
            Value::Str(s) => Some(s.to_string()),
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

    fn run_seed_witness_claim_result_text(
        source_roots: &[String],
        subject: &str,
        function: &str,
        variant_name: &str,
        cause: Option<&str>,
        wall_nanos: u128,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_ci_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let wall = run_in_context_with_args(
            &ctx,
            "nanosecond",
            &[(Some("count".to_string()), Value::Int(wall_nanos as i64))],
            false,
        )
        .ok()?;
        let out = run_in_context_with_args(
            &ctx,
            "ci_witness_claim_result_text",
            &[
                (Some("subject".to_string()), str_value(subject.to_string())),
                (
                    Some("function".to_string()),
                    str_value(function.to_string()),
                ),
                (
                    Some("verdict".to_string()),
                    Value::Variant {
                        type_name: ctx.sym("CiWitnessVerdict"),
                        variant_name: ctx.sym(variant_name),
                        fields: std::rc::Rc::new(match cause {
                            None => Vec::new(),
                            Some(cause) => vec![(
                                ctx.sym("cause"),
                                Value::Variant {
                                    type_name: ctx.sym("CiWitnessRuntimeCause"),
                                    variant_name: ctx.sym(cause),
                                    fields: std::rc::Rc::new(Vec::new()),
                                },
                            )],
                        }),
                    },
                ),
                (Some("wall".to_string()), wall),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    // ENROLLMENT MAY ONLY HOLD A CLAIM THAT WAS DECIDED. Found on gunbc#8642 by the side
    // thread: the first version of `from_outcome` matched `_ if enrolled => KnownRed` above
    // every failing arm, so an enrolled row that was INTERRUPTED at a budget, or whose host
    // tool could not be resolved, printed KNOWN-RED — owned semantic debt, for two states that
    // carry no semantic verdict at all.
    //
    // RED: restoring the blanket `_ if enrolled` arm turns the two interruption cases into
    // KnownRed and fails this test. The `Fail` case is the positive control — without it, a
    // `from_outcome` that never returned KnownRed at all would also pass.
    #[test]
    fn enrollment_holds_only_semantic_failures_not_undecided_claims() {
        use v1_compiler::cli_run::{BudgetKind, CiWitnessVerdict, ClaimOutcome};

        let interrupted = ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms: 5_000,
            budget_ms: 5_000,
            kind: BudgetKind::Cpu,
        };
        let missing_tool = ClaimOutcome::HostToolUnresolved {
            name: "git".to_string(),
            probed: vec!["/usr/bin/git".to_string()],
        };

        assert_eq!(
            CiWitnessVerdict::from_outcome(&ClaimOutcome::Fail, true),
            CiWitnessVerdict::KnownRed,
            "positive control: an enrolled SEMANTIC failure is the one thing enrollment holds"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&interrupted, true),
            CiWitnessVerdict::BudgetRefused,
            "an interruption is a lower bound on cost, not a verdict enrollment can hold"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&missing_tool, true),
            CiWitnessVerdict::HostToolUnresolved,
            "an unresolved host tool is an infra gap, not owned semantic debt"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&interrupted, false),
            CiWitnessVerdict::from_outcome(&interrupted, true),
            "enrollment must not change the token for an undecided claim, either direction"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&ClaimOutcome::Pass, true),
            CiWitnessVerdict::Passed,
            "an enrolled row that passes still prints PASSED; the roster staleness is the \
             ledger's to report, not a token the console invents"
        );
    }

    #[test]
    fn render_witness_claim_result_text_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let oracle = run_seed_witness_claim_result_text(
            &roots,
            "test.claim.observation_ci_render_witness_test",
            "w_witness_claim_line_from_module_path_holds",
            "WitnessPassed",
            None,
            230_000_000,
        )
        .expect("ci_witness_claim_result_text must resolve and render");
        let mirror = v1_compiler::cli_run::render_witness_claim_result_text_mirror(
            "test.claim.observation_ci_render_witness_test",
            "w_witness_claim_line_from_module_path_holds",
            230_000_000,
            v1_compiler::cli_run::CiWitnessVerdict::Passed,
        );
        assert_eq!(
            oracle, mirror,
            "witness claim-result mirror must be byte-equal to the .dag oracle"
        );
        assert!(
            mirror.contains("PASSED in 230ms") && !mirror.contains("PASS in "),
            "drift control: legacy PASS token must not return: {mirror:?}"
        );
        assert!(
            mirror.starts_with("//test/claim/observation_ci_render_witness_test:"),
            "drift control: Bazel label prefix required: {mirror:?}"
        );

        let under_boundary = run_seed_witness_claim_result_text(
            &roots,
            "test.claim.foo",
            "w_bar",
            "WitnessPassed",
            None,
            89_000_000_000,
        )
        .expect("89s boundary oracle");
        let at_boundary = run_seed_witness_claim_result_text(
            &roots,
            "test.claim.foo",
            "w_bar",
            "WitnessPassed",
            None,
            90_000_000_000,
        )
        .expect("90s boundary oracle");
        assert_eq!(
            v1_compiler::cli_run::render_witness_claim_result_text_mirror(
                "test.claim.foo",
                "w_bar",
                89_000_000_000,
                v1_compiler::cli_run::CiWitnessVerdict::Passed,
            ),
            under_boundary,
            "89s minute-switch boundary must match oracle"
        );
        assert_eq!(
            v1_compiler::cli_run::render_witness_claim_result_text_mirror(
                "test.claim.foo",
                "w_bar",
                90_000_000_000,
                v1_compiler::cli_run::CiWitnessVerdict::Passed,
            ),
            at_boundary,
            "90s minute-switch boundary must match oracle"
        );
        assert!(
            under_boundary.contains("89 seconds") && at_boundary.contains("1 minutes"),
            "minute-switch drift control: {under_boundary:?} vs {at_boundary:?}"
        );

        // Every arm of the verdict coproduct is rendered by the mirror byte-equal to the
        // `.dag` oracle, and every arm produces a DISTINCT token. The bool this parameter
        // used to be could only witness two of these eight; five typed outcomes were
        // flattened into FAILED at the call site before the renderer was ever reached.
        // RED: collapsing any two arms to one token fails the distinctness assert;
        // drifting one arm's spelling in either representation fails byte-equality.
        let arms: [(&str, v1_compiler::cli_run::CiWitnessVerdict); 8] = [
            (
                "WitnessPassed",
                v1_compiler::cli_run::CiWitnessVerdict::Passed,
            ),
            (
                "WitnessFailed",
                v1_compiler::cli_run::CiWitnessVerdict::Failed,
            ),
            (
                "WitnessNotBool",
                v1_compiler::cli_run::CiWitnessVerdict::NotBool,
            ),
            (
                "WitnessRuntimeError",
                v1_compiler::cli_run::CiWitnessVerdict::RuntimeError(
                    v1_compiler::cli_run::WitnessRuntimeCause::NoSuchVariable,
                ),
            ),
            (
                "WitnessBudgetRefused",
                v1_compiler::cli_run::CiWitnessVerdict::BudgetRefused,
            ),
            (
                "WitnessHostToolUnresolved",
                v1_compiler::cli_run::CiWitnessVerdict::HostToolUnresolved,
            ),
            (
                "WitnessKnownRed",
                v1_compiler::cli_run::CiWitnessVerdict::KnownRed,
            ),
            (
                "WitnessRouteGap",
                v1_compiler::cli_run::CiWitnessVerdict::RouteGap,
            ),
        ];
        let arm_count = arms.len();
        let mut rendered: Vec<String> = Vec::new();
        for (variant, verdict) in arms {
            // The cause rides inside the arm on BOTH sides, so the oracle call carries it too:
            // a fieldless `WitnessRuntimeError` is not constructible in the `.dag` type any more.
            let cause = match verdict {
                v1_compiler::cli_run::CiWitnessVerdict::RuntimeError(_) => {
                    Some("WitnessCauseNoSuchVariable")
                }
                _ => None,
            };
            let oracle_arm = run_seed_witness_claim_result_text(
                &roots,
                "test.claim.foo",
                "w_bar",
                variant,
                cause,
                1_000_000,
            )
            .unwrap_or_else(|| panic!("oracle must render {variant}"));
            let mirror_arm = v1_compiler::cli_run::render_witness_claim_result_text_mirror(
                "test.claim.foo",
                "w_bar",
                1_000_000,
                verdict,
            );
            assert_eq!(
                oracle_arm, mirror_arm,
                "{variant}: mirror must be byte-equal to the .dag oracle"
            );
            rendered.push(mirror_arm);
        }
        let mut distinct = rendered.clone();
        distinct.sort();
        distinct.dedup();
        // WAS THE LITERAL `7` AGAINST AN EIGHT-ARM TABLE, and it did not start failing with this
        // change -- it was already red on `main`. The Rust suite has not run in CI since
        // 2026-07-11 (operator ruling, recorded at `gunbc.commit_workflow`
        // `commit_gate_rust_suite_removed_disposition`), so a stale literal here is invisible
        // until someone runs the bin's tests by hand. Executed receipt: this assertion reports
        // `left: 8 right: 7`, and the eight lines it prints are pairwise distinct with or
        // without the cause field, which the `.dag` side asserts independently in
        // `w_every_verdict_arm_renders_a_distinct_token`.
        //
        // Pinned to `arm_count` rather than to a new literal `8`, so adding a ninth arm cannot
        // reproduce this: the fixture is the table, and a count copied out of it by hand is the
        // second representation that went stale in the first place.
        assert_eq!(
            distinct.len(),
            arm_count,
            "every verdict arm must render a distinct line: {rendered:?}"
        );
        assert!(
            !rendered[6].contains("FAILED"),
            "an enrolled known-red must not be spelled as FAILED: {:?}",
            rendered[6]
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
                (Some("label".to_string()), str_value(label.to_string())),
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
            Value::Str(s) => Some(s.to_string()),
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
                (Some("intent".to_string()), str_value(intent.to_string())),
                (
                    Some("argv_collapsed".to_string()),
                    str_value(argv_collapsed.to_string()),
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
            Value::Str(s) => Some(s.to_string()),
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
                    str_value(batch_label.to_string()),
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
            Value::Str(s) => Some(s.to_string()),
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
    // totals into the process accumulator. Serialized via
    // PROCESS_EVAL_RECOMPUTE_TEST_LOCK so no sibling test can drain or pollute
    // the accumulator mid-assertion.
    #[test]
    fn materialization_receipt_totals_absorb_on_ctx_drop() {
        with_process_eval_recompute_test_lock(|| {
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
        });
    }

    /// The deadline must FIRE, not merely be derivable.
    ///
    /// `falsifier_soft_deadline_is_derived_and_strictly_inside_the_hard_cap` proves the
    /// arithmetic; a number that no code ever compares against would satisfy it completely.
    /// This runs a real walk with the deadline already elapsed (0ms) and asserts the three
    /// things the mechanism actually owes: admission stops, the run is RED rather than
    /// silently short, and the unadmitted tail is reported as `deadline_reached` — not as
    /// the stop-policy skip it would have been misattributed as before.
    ///
    /// The `None` control is the discriminating half: the SAME two batches with no deadline
    /// run to completion. Without it this test would pass on a walk that was broken in some
    /// unrelated way and stopped early for a different reason.
    #[test]
    fn soft_deadline_stops_admission_and_reports_deadline_reached() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("src/v2/test/fixture/floor_skip/falsifier_divergence_control_test.dag")
            .to_string_lossy()
            .into_owned();
        let batch = || {
            vec![Runnable::SingleClaim {
                entry: entry.clone(),
                function: "falsifier_green_control_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }]
        };
        let batches = vec![batch(), batch()];

        let walk = |deadline: Option<u64>| {
            run_walk(
                &roots,
                "test::soft_deadline_fixture",
                &batches,
                &[],
                None,
                FloorFinalizationAbsenceReason::Undeclared,
                &mut std::io::stderr(),
                None,
                None,
                deadline,
                &RealizationConcurrency::for_walk(1).expect("test schedule"),
                None,
                FalsifierSelfHostWetBudgets::default(),
                FloorBatchStopPolicy::StopBeforeDependents,
                None,
                None,
                false,
                None,
            )
        };

        // CONTROL: no deadline — both components are admitted and the walk is green.
        let unbounded = walk(None);
        assert!(
            !unbounded.any_failed,
            "control walk must be green: {:?}",
            unbounded.failure_details
        );

        // ARMED: a deadline already in the past stops admission at the FIRST batch.
        let bounded = walk(Some(0));
        assert!(
            bounded.any_failed,
            "a walk that could not admit its components must be RED, never silently short"
        );
        assert!(
            bounded
                .failure_details
                .iter()
                .any(|d| d.contains("soft deadline")),
            "the refusal must name the deadline as the cause: {:?}",
            bounded.failure_details
        );
        // NEGATIVE: it must not be reported as the population budget, which is a different
        // ceiling with a different remedy.
        assert!(
            !bounded
                .failure_details
                .iter()
                .any(|d| d.contains("population budget")),
            "a deadline stop must not masquerade as the ordinary population budget: {:?}",
            bounded.failure_details
        );
    }

    fn parse_materialization_receipt_field(body: &str, key: &str) -> Option<u64> {
        body.lines()
            .find_map(|line| line.strip_prefix(key))
            .and_then(|v| v.trim().parse::<u64>().ok())
    }

    fn run_walk_materialization_fixture(
        roots: &[String],
        on_success_stages: &[Vec<Runnable>],
    ) -> WalkOutcome {
        let root = workspace_root();
        let ordinary_entry = root
            .join("src/v2/test/fixture/floor_skip/falsifier_divergence_control_test.dag")
            .to_string_lossy()
            .into_owned();
        let ordinary_batch = vec![vec![Runnable::SingleClaim {
            entry: ordinary_entry,
            function: "falsifier_green_control_holds".to_string(),
            profile: ParsedRunnableProfile::undeclared(),
        }]];
        const TEST_ATTEMPT_ID: &str = "materialization-receipt-test";
        let walk_attempt_id = if on_success_stages.is_empty() {
            None
        } else {
            Some(TEST_ATTEMPT_ID)
        };
        run_walk(
            roots,
            "test::on_success_materialization_fixture",
            &ordinary_batch,
            on_success_stages,
            None,
            FloorFinalizationAbsenceReason::Undeclared,
            &mut std::io::stderr(),
            None,
            None,
            None,
            &RealizationConcurrency::for_walk(1).expect("test schedule"),
            None,
            FalsifierSelfHostWetBudgets::default(),
            FloorBatchStopPolicy::StopBeforeDependents,
            None,
            None,
            true,
            walk_attempt_id,
        )
    }

    // Discriminating control (DESIGN §5): post-floor pure demand must appear in the
    // on-success materialization receipt and NOT in the ordinary-floor receipt, which
    // is harvested before stages run. Population separation is asserted within ONE
    // staged walk (ordinary receipt is written before on_success_stages in the same
    // run_walk — no cross-run keyed_calls equality that parallel siblings could
    // perturb, review 45766). PROCESS_EVAL_RECOMPUTE_TEST_LOCK serializes the
    // trace-enabled receipt tests and restores trace env/cache afterward (review
    // 45737, review 45756).
    #[test]
    fn on_success_materialization_receipt_separates_from_ordinary_floor() {
        with_process_eval_recompute_test_lock(|| {
            let root = workspace_root();
            with_workspace_root_current_dir(&root, || {
                let roots = vec![
                    root.join("src/v2").to_string_lossy().into_owned(),
                    root.join("dag").to_string_lossy().into_owned(),
                ];
                const TEST_ATTEMPT_ID: &str = "materialization-receipt-test";
                let success_receipt = root
                    .join("target")
                    .join(format!("floor-attempt-{TEST_ATTEMPT_ID}"))
                    .join("floor-on-success-materialization-receipt.txt");
                let ordinary_receipt = root.join("target/floor-materialization-receipt.txt");
                let _ = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();

                let run_fixture = |on_success_stages: &[Vec<Runnable>]| {
                    let _ = std::fs::remove_file(&success_receipt);
                    let _ = std::fs::remove_file(&ordinary_receipt);
                    let outcome = run_walk_materialization_fixture(&roots, on_success_stages);
                    assert!(
                        !outcome.any_failed,
                        "walk must pass: {:?}",
                        outcome.failure_details
                    );
                    outcome
                };

                // RED: no on-success stages => no success-stage materialization receipt.
                run_fixture(&[]);
                assert!(
                    !success_receipt.exists(),
                    "empty on_success_stages must not write a success materialization receipt"
                );

                // GREEN: one walk — ordinary receipt harvested before stages; success
                // receipt harvested after stage_memo drops in the same run_walk.
                let stage_entry = root
                    .join("dag/test/claim/materialization_ladder_witness_test.dag")
                    .to_string_lossy()
                    .into_owned();
                let success_stage = vec![vec![Runnable::SingleClaim {
                    entry: stage_entry,
                    function: "single_pure_demand_is_accepted_recompute".to_string(),
                    profile: ParsedRunnableProfile {
                        provenance: ParsedProfileProvenance::Declared,
                        heavy_whole_tree_resolve: false,
                        spawns_host_compiler: false,
                        memory: ParsedMemoryClass::Negligible,
                        execution_mode: ExecutionMode::Hermetic,
                    },
                }]];
                run_fixture(&success_stage);

                let ordinary_body =
                    std::fs::read_to_string(&ordinary_receipt).expect("ordinary receipt");
                let ordinary_keyed =
                    parse_materialization_receipt_field(&ordinary_body, "keyed_calls=")
                        .expect("keyed_calls");
                let success_body = std::fs::read_to_string(&success_receipt)
                    .expect("on-success materialization receipt");
                let success_keyed =
                    parse_materialization_receipt_field(&success_body, "keyed_calls=").unwrap();

                assert!(
                    success_body.starts_with(&format!(
                        "attempt_id={TEST_ATTEMPT_ID}\nplan_site=test::on_success_materialization_fixture\n"
                    )),
                    "success receipt must carry attempt and plan identity in the payload"
                );
                assert!(
                    ordinary_keyed > 0,
                    "ordinary-floor walk must record pure demand in the ordinary receipt"
                );
                assert!(
                    success_keyed > 0,
                    "on-success stage pure demand must be counted in the success receipt"
                );
                assert!(
                    parse_materialization_receipt_field(&success_body, "memo_hits=").is_some()
                        && parse_materialization_receipt_field(&success_body, "memo_misses=")
                            .is_some()
                        && parse_materialization_receipt_field(&success_body, "duplicated_keys=")
                            .is_some(),
                    "success receipt must carry the same field set as the ordinary receipt"
                );
            });
        });
    }

    // The eval-frame memo by execution: the same pure claim evaluated twice on
    // one ctx must (a) produce identical values — the memo-vs-recompute
    // equivalence oracle at the value grain — and (b) record verified hits, so
    // "the cache worked" is a counted fact, never an assumption. Assertions
    // are per-ctx (eval_call_memo_counters), immune to test-process sharing.
    // Serialized on PROCESS_EVAL_RECOMPUTE_TEST_LOCK: ctx Drop absorbs ledger
    // totals into the process accumulator when trace is on, so this test must
    // not overlap materialization-receipt walks (review 45737).
    #[test]
    fn eval_call_memo_serves_verified_hits_with_identical_values() {
        with_process_eval_recompute_test_lock(|| {
            let _ = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
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
            let (_, misses_after_first, _) =
                v1_compiler::v1_interpreter::eval_call_memo_counters(&ctx);
            let second = run_value(
                &ctx,
                "cross_frame_duplicate_discharged_by_covering_provider",
            )
            .expect("second evaluation");
            assert!(
                first == second,
                "memo-served evaluation must equal the recomputed one"
            );
            let (hits, misses, overflow) =
                v1_compiler::v1_interpreter::eval_call_memo_counters(&ctx);
            assert!(
                hits > 0,
                "second identical evaluation must serve verified hits from the eval memo"
            );
            assert!(
                misses >= misses_after_first,
                "miss counter is monotone (counted, never reset)"
            );
            assert_eq!(overflow, 0, "tiny workload must not hit the entry cap");
        });
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

    /// A discovery batch that actually discovers something. `scan_dirs` is non-empty
    /// deliberately: `group_batch_units` emits a `BatchUnit::Discovery` only when the batch
    /// has scan dirs or explicit entries, because empty-and-empty is defined to mean zero
    /// witness-corpus nodes (the regen spec's shape, DESIGN "Building & checks"). A fixture
    /// with both empty is therefore not a discovery batch at all, and asserting that it
    /// produces a discovery unit asserts against the spec.
    fn discovery() -> Runnable {
        Runnable::DiscoveryBatch {
            source_roots: vec!["src/v2".to_string()],
            scan_dirs: vec!["dag/test/claim".to_string()],
            explicit_entries: vec![],
            native_bundle_entries: vec![],
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: ExecutionMode::Hermetic,
            spawns_host_compiler: false,
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
                BatchUnit::Discovery { .. }
                | BatchUnit::ScopedDiscovery { .. }
                | BatchUnit::NativeBundle { .. } => {}
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
    fn native_bundle_kind_becomes_only_a_native_unit() {
        let batch = vec![Runnable::DiscoveryBatch {
            source_roots: vec!["src/v2".to_string(), "dag".to_string()],
            scan_dirs: vec![],
            explicit_entries: vec![],
            native_bundle_entries: vec![("bundle.dag".to_string(), "bundle_spec".to_string())],
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: ExecutionMode::Wet,
            spawns_host_compiler: true,
        }];
        let units = group_batch_units(&batch);
        assert_eq!(
            units.len(),
            1,
            "native kind must not create an interpreter discovery unit"
        );
        assert!(matches!(
            &units[0],
            BatchUnit::NativeBundle { entry, selector_function, execution_mode }
                if entry == "bundle.dag"
                    && selector_function == "bundle_spec"
                    && *execution_mode == ExecutionMode::Wet
        ));
    }

    #[test]
    fn native_bundle_refuses_without_fallback() {
        // CI-0 cutover: there is no fallback arm to test — the discriminating
        // evidence is that NOTHING except the full native bar reports ok.
        // Outage: refused (this exact input was the counted-fallback arm before
        // the cutover; it is the permanent regression control for the deleted arm).
        assert!(!native_transition_accepted(false, true, false));
        // Native green but oracle red: refused (equivalence evidence retained).
        assert!(!native_transition_accepted(true, false, true));
        // Native green, oracle green, planted RED not natively reproduced: refused.
        assert!(!native_transition_accepted(true, true, false));
        // Full native bar: accepted.
        assert!(native_transition_accepted(true, true, true));
        // Population counts: interpreted and fallback are structurally zero,
        // and a divergence (ran, wrong output) is its own column — never
        // laundered into "unavailable" (review 50560-class finding).
        assert_eq!(
            native_transition_population_counts(3, true, false),
            (3, 0, 0, 0, 0)
        );
        assert_eq!(
            native_transition_population_counts(3, false, false),
            (0, 0, 3, 0, 0)
        );
        assert_eq!(
            native_transition_population_counts(3, false, true),
            (0, 0, 0, 3, 0)
        );
    }

    /// A signalled process must not borrow an exit code it does not have. The seed
    /// rendered `.code().unwrap_or(-1)` for every leg before this, which made a runner
    /// OOM-kill read exactly like a process that chose to exit -1.
    #[test]
    fn native_leg_signal_death_is_not_rendered_as_an_exit_code() {
        assert_eq!(
            ProcessTermination::Signaled(9).located(),
            "killed by signal 9"
        );
        assert_eq!(ProcessTermination::Exited(101).located(), "exited 101");
        assert_eq!(
            ProcessTermination::Unobserved.located(),
            "termination unobserved"
        );
        // The discriminating half: no termination renders as the old fabricated -1,
        // and the signal arm shares no rendering with any exit arm.
        for termination in [
            ProcessTermination::Signaled(9),
            ProcessTermination::Signaled(11),
            ProcessTermination::Unobserved,
        ] {
            assert!(
                !termination.located().contains("exited"),
                "{termination:?} rendered as an exit"
            );
        }
    }

    /// Malformed transport wire must not borrow the legitimate `Unobserved` arm. "The
    /// process never produced a status" and "the transport violated its own modeled
    /// wire" have different owners and different fixes; collapsing them is the same
    /// state-space conflation this change removes at the `ExitStatus` boundary, one
    /// layer down. ONLY the explicit modeled `ProcessTerminationUnobserved` variant
    /// decodes to `Unobserved` — every other shape carries a located refusal.
    #[test]
    fn malformed_termination_wire_refuses_instead_of_reading_as_unobserved() {
        let graph = v1_compiler::v1_compiler_infer_items::ResolvedGraph {
            modules: Rc::new(Default::default()),
            item_registry: Rc::new(Default::default()),
            diagnostics: Rc::new(Default::default()),
            emit_graph_info: v1_compiler::v1_compiler_infer_emit_info::empty_emit_graph_info(),
        };
        let ctx = InterpContext::new(&graph, Rc::new(Default::default()), ExecutionMode::Hermetic);
        let variant = |name: &str, fields: Vec<(&str, Value)>| Value::Variant {
            type_name: ctx.sym("ProcessTermination"),
            variant_name: ctx.sym(name),
            fields: Rc::new(v1_compiler::v1_interpreter::sorted_fields(
                fields.into_iter().map(|(k, v)| (ctx.sym(k), v)).collect(),
            )),
        };
        // The modeled arms decode — including the ONE legitimate route to Unobserved.
        assert_eq!(
            transport_termination(Some(&variant("ProcessTerminationUnobserved", vec![])), &ctx)
                .unwrap(),
            ProcessTermination::Unobserved
        );
        assert_eq!(
            transport_termination(
                Some(&variant("ProcessExited", vec![("code", Value::Int(101))])),
                &ctx
            )
            .unwrap(),
            ProcessTermination::Exited(101)
        );
        assert_eq!(
            transport_termination(
                Some(&variant("ProcessSignaled", vec![("signal", Value::Int(9))])),
                &ctx
            )
            .unwrap(),
            ProcessTermination::Signaled(9)
        );
        // Every malformed shape refuses with a located cause, never Unobserved.
        let malformed: Vec<(&str, Option<Value>)> = vec![
            ("absent field", None),
            ("non-variant value", Some(Value::Int(0))),
            ("unknown variant", Some(variant("ProcessVanished", vec![]))),
            ("missing code", Some(variant("ProcessExited", vec![]))),
            (
                "non-integer signal",
                Some(variant(
                    "ProcessSignaled",
                    vec![("signal", str_value("9".to_string()))],
                )),
            ),
        ];
        for (label, value) in malformed {
            let refusal = transport_termination(value.as_ref(), &ctx)
                .err()
                .unwrap_or_else(|| panic!("{label}: malformed wire decoded as legitimate"));
            assert!(
                !refusal.located.is_empty(),
                "{label}: refusal carries no location"
            );
        }
    }

    /// The excerpt is the TAIL: cargo writes its error last and a panic writes its
    /// message last, so a head-anchored excerpt would reliably carry the least useful
    /// bytes. It is bounded so one refusal cannot flood the floor's result stream.
    #[test]
    fn stream_excerpt_keeps_the_tail_bounded_and_marks_truncation() {
        assert_eq!(stream_excerpt(b""), "<empty>");
        // An absent stream and a stream that said nothing are different observations
        // and neither may read as a message.
        assert_eq!(stream_excerpt(b"\n \n"), "<whitespace only>");
        // A stream almost always ends in a newline; the trailing marker must not
        // survive as a line the process never wrote.
        assert_eq!(
            stream_excerpt(b"error: no such command\n"),
            "error: no such command"
        );
        assert_eq!(
            stream_excerpt(b"first line\nsecond line\n"),
            "first line ⏎ second line"
        );
        let long: Vec<u8> = std::iter::repeat(b'x')
            .take(4000)
            .chain(b"THE ACTUAL ERROR".iter().copied())
            .collect();
        let excerpt = stream_excerpt(&long);
        assert!(excerpt.contains("THE ACTUAL ERROR"), "tail was dropped");
        assert!(excerpt.starts_with('…'), "truncation was not marked");
        assert!(
            excerpt.len() < 1400,
            "excerpt is unbounded: {}",
            excerpt.len()
        );
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
            RealizationConcurrency::for_walk(1).expect("test schedule"),
            None,
            FalsifierSelfHostWetBudgets::default(),
            None,
        );
        assert_eq!(results.len(), 1);
        assert!(!results[0].ok, "unmapped sentinel must fail closed");
    }

    #[test]
    fn discovery_corpus_kind_label_follows_profile_not_roster_size() {
        assert_eq!(
            discovery_corpus_kind_label(&[], ExecutionMode::Wet, true),
            "bin-witness-corpus"
        );
        assert_eq!(
            discovery_corpus_kind_label(&[], ExecutionMode::Wet, false),
            "execution-corpus"
        );
        assert_eq!(
            discovery_corpus_kind_label(
                &["dag/test/claim".to_string()],
                ExecutionMode::Hermetic,
                false,
            ),
            "discovery-corpus"
        );
        assert_eq!(
            discovery_corpus_kind_label(&[], ExecutionMode::Hermetic, false),
            "explicit-corpus"
        );
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
                    None,
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
                    None,
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
            None,
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
            None,
        );
        assert_eq!(
            second[0].resolve_nanos, 0,
            "second call must cache-hit — resolve_entry_graph must NOT fire again"
        );
    }

    /// Warm memo reuse must attach SatisfiedFromSharedPool to the first claim in the
    /// group — finalization reads resolve_realization from find_claim_result, not from
    /// resolve_nanos alone. Goes RED if `first` is gated on `fresh_resolve`.
    #[test]
    fn memo_warm_attaches_shared_pool_observation_on_first_claim() {
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
        let _cold = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_negligible_profile_is_not_heavy".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
            None,
        );
        let warm = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_substantial_memory_forbids_corpus_co_residence".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
            None,
        );
        assert_eq!(warm[0].resolve_nanos, 0);
        match warm[0].resolve_realization.as_ref() {
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool { provider_id, .. }) => {
                assert_eq!(provider_id, FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID);
            }
            other => panic!(
                "warm memo first claim must carry SatisfiedFromSharedPool observation, got {other:?}"
            ),
        }
    }

    #[test]
    fn take_group_observation_attaches_only_to_rostered_subject() {
        let subjects: ObligationSubjectSet =
            [("fixture.dag".to_string(), "rostered_fn".to_string())]
                .into_iter()
                .collect();
        let observation =
            Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 42 });
        let mut attached = false;
        assert!(take_group_observation_for_claim(
            Some(&subjects),
            "fixture.dag",
            "other_fn",
            &observation,
            &mut attached,
        )
        .is_none());
        assert!(!attached);
        assert!(matches!(
            take_group_observation_for_claim(
                Some(&subjects),
                "fixture.dag",
                "rostered_fn",
                &observation,
                &mut attached,
            ),
            Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 42 })
        ));
    }

    #[test]
    fn obligation_observation_skips_non_rostered_co_residents() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let rostered_fn = "witness_substantial_memory_forbids_corpus_co_residence".to_string();
        let non_rostered_fn = "witness_negligible_profile_is_not_heavy".to_string();
        let subjects: ObligationSubjectSet =
            [(entry.clone(), rostered_fn.clone())].into_iter().collect();
        let mode = ExecutionMode::Hermetic;

        let cheap_only = run_shared_entry_claims(
            &source_roots,
            &entry,
            &[non_rostered_fn.clone()],
            mode,
            Some(&subjects),
        );
        assert!(cheap_only[0].resolve_realization.is_none());

        let mut memo = std::collections::HashMap::new();
        let rostered_only = run_memo_shared_claims(
            &source_roots,
            &entry,
            &[rostered_fn.clone()],
            mode,
            &mut memo,
            Some(&subjects),
        );
        assert!(matches!(
            rostered_only[0].resolve_realization,
            Some(ResolveRealizationObservation::ColdResolvePerformed { .. })
        ));

        let co_resident = run_memo_shared_claims(
            &source_roots,
            &entry,
            &[non_rostered_fn, rostered_fn],
            mode,
            &mut memo,
            Some(&subjects),
        );
        assert!(co_resident[0].resolve_realization.is_none());
        assert!(matches!(
            co_resident[1].resolve_realization,
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool { .. })
        ));
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
        // state — until ClaimOutcome distinguishes killed from completed, which it now does.
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

    fn drift_authority_source_roots() -> Vec<String> {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        vec![
            workspace.join("dag").to_string_lossy().into_owned(),
            workspace.join("src/v2").to_string_lossy().into_owned(),
        ]
    }

    fn drift_basis_fixture(clock_constructor: &'static str) -> WitnessRowCostBasisRow {
        WitnessRowCostBasisRow {
            eval_ms_basis: 10,
            run_ref: "synthetic-run".to_string(),
            clock_constructor,
        }
    }

    /// The END-TO-END proof that the drift seam carries the clock, which neither the `.dag`
    /// witnesses nor the parser tests can give: the witnesses never cross this Rust boundary,
    /// and the parser only shows the cell is read.
    ///
    /// Two claims. First, every verdict string in the receipt is a name the AUTHORITY
    /// produced — the seed no longer decides that exceeding a basis means `DriftExceeded`, so
    /// an arm added to `WitnessRowCostVerdict` reaches the cadence receipt with no Rust edit.
    /// Second, a cross-clock pair REFUSES, asserted on BOTH sides of the 2× ratio: 21ms and
    /// 20ms against a 10ms basis land on opposite sides of the threshold, so a single case
    /// would be satisfiable by a comparator refusing for the wrong reason. The pair proves
    /// the refusal is decided by the CLOCK, not the magnitude.
    #[test]
    fn drift_verdicts_come_from_the_authority_and_a_cross_clock_basis_refuses() {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        let roots = drift_authority_source_roots();
        let entry = workspace.join("dag/gunbc/witness_row_cost.dag");
        let (graph, indices) =
            resolve_entry_graph(&roots, &entry.to_string_lossy()).expect("resolve");
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);

        let wall = drift_basis_fixture("clock_basis_wall");
        let cpu = drift_basis_fixture("clock_basis_cpu");
        let verdict = |observed, basis| {
            witness_row_cost_verdict_via_authority(&ctx, observed, basis).expect("verdict")
        };

        // Same clock on both sides: the ratio decides, in both directions.
        assert_eq!(verdict(21, Some(&wall)), "DriftExceeded");
        assert_eq!(verdict(20, Some(&wall)), "WithinBasis");

        // No dated basis: the honest third state, and it too comes from the authority.
        assert_eq!(verdict(999, None), "BasisAbsent");

        // Different clocks: refused on BOTH sides of the ratio. Before the clock crossed
        // this seam these two answered `DriftExceeded` and `WithinBasis` — confident
        // verdicts about two different quantities.
        assert_eq!(verdict(21, Some(&cpu)), "BasisClockMismatch");
        assert_eq!(verdict(20, Some(&cpu)), "BasisClockMismatch");
    }

    #[test]
    fn parse_witness_row_cost_basis_line_requires_srv_fleet_arm64() {
        // RED control for review 43284: wrong/missing host_class must refuse, never load.
        let ok = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64\twall")
            .expect("parse")
            .expect("row");
        assert_eq!(ok.0, ("e.dag".to_string(), "f".to_string()));
        assert_eq!(ok.1.eval_ms_basis, 10);
        assert_eq!(ok.1.run_ref, "run-1");
        assert_eq!(ok.1.clock_constructor, "clock_basis_wall");

        // The clock is READ, not assumed: a cpu-clocked basis row loads as cpu, which is
        // what lets the comparator refuse it against a wall observation instead of
        // answering. If this cell were ignored the whole column would be decoration.
        let cpu = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64\tcpu")
            .expect("parse")
            .expect("row");
        assert_eq!(cpu.1.clock_constructor, "clock_basis_cpu");

        assert!(parse_witness_row_cost_basis_line("# comment")
            .unwrap()
            .is_none());
        assert!(parse_witness_row_cost_basis_line("").unwrap().is_none());

        let wrong = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tlocal_x86\twall")
            .expect_err("wrong host_class must refuse");
        assert!(
            wrong.contains("host_class") && wrong.contains("srv_fleet_arm64"),
            "expected host_class refusal, got: {wrong}"
        );

        // A row with no clock cell REFUSES rather than defaulting to wall. Defaulting
        // would be right for every row in the file today and silently wrong the first
        // time one is seeded from a CPU receipt — the exact failure the column exists to
        // prevent, so the pre-clock 5-column shape must not still parse.
        let short = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64")
            .expect_err("a row without a clock column must refuse");
        assert!(short.contains("need 6 cols"), "got: {short}");

        // An unmodelled clock is not a clock. It cannot map to a ClockBasis constructor,
        // so it cannot enter a comparison at all.
        let unknown =
            parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64\tmonotonic")
                .expect_err("unknown clock must refuse");
        assert!(
            unknown.contains("clock") && unknown.contains("monotonic"),
            "got: {unknown}"
        );

        let zero = parse_witness_row_cost_basis_line("e.dag\tf\t0\trun-1\tsrv_fleet_arm64\twall")
            .expect_err("zero eval must refuse");
        assert!(zero.contains("zero eval_ms_basis"), "got: {zero}");
    }

    #[test]
    /// Discriminating replay control: planted passed, failed, and selection-skipped rows
    /// must surface as three distinct coordinator replay outcomes (not collapsed).
    /// Evidence: `claim_executor` unit tests — the binary that gates CI floor merge.
    fn wet_witness_row_outcome_replay_discriminates_passed_failed_and_selection_skipped() {
        let base =
            std::env::temp_dir().join(format!("claim-executor-wet-outcome-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let records = vec![BatchRecord {
            batch_index: 3,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 3,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 3 },
            label: "bin-witness-corpus".to_string(),
            is_wet: true,
            results: vec![ClaimResult {
                function: "bin-witness-corpus (3 witnesses)".to_string(),
                entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
                ok: false,
                detail: "1 of 3 discovery witness(es) failed".to_string(),
                wall_nanos: 0,
                resolve_nanos: 0,
                corpus_resolve_nanos: 0,
                corpus_eval_nanos: 0,
                corpus_witnesses: 3,
                runtime_unit_count: discovery_runtime_unit_count_from_summary(3),
                witness_row_costs: vec![
                    WitnessRowCost {
                        entry: "dag/test/claim/stage0_rust_host_observation_live_witness_test.dag"
                            .to_string(),
                        function: "planted_pass_wet_row".to_string(),
                        eval_wall_nanos: 1_000_000,
                        eval_cpu_nanos: Some(800_000),
                        resolve_nanos: 0,
                        warm_nanos: 0,
                        outcome: "Done".to_string(),
                        detail: String::new(),
                    },
                    WitnessRowCost {
                        entry: "dag/test/claim/planted_fail_wet_row_test.dag".to_string(),
                        function: "planted_fail_wet_row".to_string(),
                        eval_wall_nanos: 500_000,
                        eval_cpu_nanos: Some(400_000),
                        resolve_nanos: 0,
                        warm_nanos: 0,
                        outcome: "Failed".to_string(),
                        detail: "returned Bool(false)".to_string(),
                    },
                    WitnessRowCost {
                        entry: "dag/other.dag".to_string(),
                        function: "planted_selection_skip_wet_row".to_string(),
                        eval_wall_nanos: 0,
                        eval_cpu_nanos: None,
                        resolve_nanos: 0,
                        warm_nanos: 0,
                        outcome: "selection-skipped".to_string(),
                        detail: "affected-set".to_string(),
                    },
                ],
                expectation_refusal: None,
                budget_refusal: None,
                host_dependency_refusal: None,
                resolve_realization: None,
            }],
        }];
        assert!(write_floor_wet_witness_row_outcome_receipt_at(
            &base, &records
        ));
        let path = base.join("floor-wet-witness-row-outcome-receipt.tsv");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("planted_pass_wet_row"));
        assert!(body.contains("planted_pass_wet_row\tpassed"));
        assert!(body.contains("planted_fail_wet_row"));
        assert!(body.contains("planted_fail_wet_row\tfailed\t"));
        assert!(body.contains("planted_selection_skip_wet_row"));
        assert!(body.contains("selection-skipped"));
        let lines = collect_wet_witness_row_outcome_replay_lines(&path).expect("replay lines");
        assert_eq!(lines.len(), 3);
        let passed_line = lines
            .iter()
            .find(|line| line.contains("planted_pass_wet_row"))
            .expect("passed replay line");
        let failed_line = lines
            .iter()
            .find(|line| line.contains("planted_fail_wet_row"))
            .expect("failed replay line");
        let skipped_line = lines
            .iter()
            .find(|line| line.contains("planted_selection_skip_wet_row"))
            .expect("selection-skipped replay line");
        assert!(passed_line.contains("outcome=passed"));
        assert!(failed_line.contains("outcome=failed"));
        assert!(skipped_line.contains("outcome=selection-skipped"));
        assert_ne!(passed_line, failed_line);
        assert_ne!(passed_line, skipped_line);
        assert_ne!(failed_line, skipped_line);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn wet_witness_row_outcome_label_maps_three_distinct_values() {
        assert_eq!(wet_witness_row_outcome_label("Done"), "passed");
        assert_eq!(
            wet_witness_row_outcome_label("selection-skipped"),
            "selection-skipped"
        );
        assert_eq!(wet_witness_row_outcome_label("Failed"), "failed");
        assert_eq!(wet_witness_row_outcome_label("Refused"), "failed");
    }

    /// Discovery budget kills must classify structurally like any other batch.
    #[test]
    fn discovery_budget_kill_classifies_structurally_on_the_falsifier_path() {
        use v1_compiler::cli_run::{
            ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
            ResolveStageNanos,
        };
        let killed_detail =
            "1 of 1 discovery witness(es) failed: fn=expensive_witness killed at its wall budget: \
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
            divergences: Vec::new(),
            failures: vec![killed_detail.into()],
            witness_outcomes: vec![DiscoveryWitnessOutcome {
                entry: "dag/test/claim/expensive_wet_witness_test.dag".into(),
                module_path: "test.claim.expensive_wet_witness".into(),
                function: "expensive_witness_keystone_holds".into(),
                outcome,
                execution_leg: "InterpretedLeg".into(),
            }],
            entry_resolve_receipts: Vec::<EntryResolveReceipt>::new(),
            total_resolve_nanos: 0,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: Vec::new(),
            total_measured_nanos: 0,
            roster_closure_nodes: 0,
            total_entry_groups: 0,
            selected_entry_groups: 0,
        };
        let killed = ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms: 900_001,
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
                None,
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
            None,
        );
        assert!(result.budget_refusal.is_none());
    }

    /// RED control for review 51796: discovery batch must lift host-dependency refusals out of
    /// `summary.failures`, parallel to `discovery_budget_kill_classifies_structurally_on_the_falsifier_path`.
    #[test]
    fn discovery_host_dependency_absent_classifies_structurally_on_the_falsifier_path() {
        use v1_compiler::cli_run::{
            ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
            ResolveStageNanos,
        };
        let wire = "HostDependencyAbsent{tool=npm,hint=apt install npm}";
        let failure = format!(
            "materialize_codex_runtime_bundle_produces_native_executable_holds \
             (dag/test/claim/codex_package_delivery_wet_witness_test.dag) returned Bool(false) | \
             {wire}"
        );
        let aggregate_detail = format!("1 of 1 discovery witness(es) failed: {failure}");
        assert_eq!(
            falsifier_failure_mode(&[aggregate_detail.clone()]),
            "WitnessRed",
            "control: string classifier alone would misclassify"
        );

        let summary_with = |outcome: ClaimOutcome, failures: Vec<String>| DiscoverySummary {
            total: 1,
            passed: 0,
            skipped: 0,
            deferred_rows: Vec::new(),
            divergences: Vec::new(),
            failures,
            witness_outcomes: vec![DiscoveryWitnessOutcome {
                entry: "dag/test/claim/codex_package_delivery_wet_witness_test.dag".into(),
                module_path: "test.claim.codex_package_delivery_wet_witness_test".into(),
                function: "materialize_codex_runtime_bundle_produces_native_executable_holds"
                    .into(),
                outcome,
                execution_leg: "InterpretedLeg".into(),
            }],
            entry_resolve_receipts: Vec::<EntryResolveReceipt>::new(),
            total_resolve_nanos: 0,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: Vec::new(),
            total_measured_nanos: 0,
            roster_closure_nodes: 0,
            total_entry_groups: 0,
            selected_entry_groups: 0,
        };

        for projected in [
            Ok(Vec::new()),
            Err("[witness-row-cost] REFUSED: missing measured resolve parent".to_string()),
        ] {
            let result = discovery_claim_result(
                "discovery-corpus".into(),
                false,
                aggregate_detail.clone(),
                &summary_with(ClaimOutcome::Fail, vec![failure.clone()]),
                projected,
                None,
            );
            assert!(
                result.host_dependency_refusal.is_some(),
                "discovery batch must lift the host dependency refusal out of summary.failures"
            );
            let (mode, _) = batch_failure_mode_and_detail(&batch_record_for_test(vec![result]));
            assert_eq!(mode, HOST_DEPENDENCY_ABSENT_MODE);
        }

        let result = discovery_claim_result(
            "discovery-corpus".into(),
            false,
            "red".into(),
            &summary_with(ClaimOutcome::Fail, vec!["returned Bool(false)".into()]),
            Ok(Vec::new()),
            None,
        );
        assert!(result.host_dependency_refusal.is_none());
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
            total_entry_groups: 0,
            selected_entry_groups: 0,
        };
        let prior = "1 of 1 discovery witness(es) failed: e.dag::f failed";
        let result = discovery_claim_result(
            "probe".into(),
            false,
            prior.to_string(),
            &summary,
            Err("[witness-row-cost] REFUSED: missing measured resolve parent for e.dag".into()),
            None,
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
            entry: "latch_fixture.dag".to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
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

    #[test]
    fn floor_worker_missing_terminal_receipt_is_an_observed_death() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-missing-terminal-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&terminal);
        let status = Command::new("sh").arg("-c").arg("exit 0").status().unwrap();
        let observed = observe_floor_worker("ordinary", status, &terminal);
        let outcome = floor_worker_observation_outcome(&observed);
        assert_eq!(
            observed.terminal_receipt,
            FloorWorkerTerminalReceipt::Missing
        );
        assert_eq!(outcome.label, "died-without-terminal-receipt");
        assert!(outcome.detail.contains("ordinary"));
        assert!(outcome.detail.contains("no terminal receipt"));
        assert!(
            !floor_worker_succeeded(&observed),
            "an OS-success exit cannot turn receipt absence into floor success"
        );
    }

    #[test]
    fn floor_phase_journal_persists_a_completed_path() {
        let journal = std::env::temp_dir().join(format!(
            "claim-executor-floor-phase-positive-control-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&journal);
        std::env::set_var(FLOOR_PHASE_JOURNAL_ENV, &journal);
        append_floor_phase_journal("positive-control", "completed", "known-green-path");
        std::env::remove_var(FLOOR_PHASE_JOURNAL_ENV);

        let persisted = fs::read_to_string(&journal)
            .expect("the synced out-of-band journal must be readable after a completed path");
        let _ = fs::remove_file(&journal);
        assert!(
            persisted.contains("\tpositive-control\tcompleted\tknown-green-path\n"),
            "the persisted row must retain the phase, state, and detail: {persisted:?}"
        );
    }

    #[test]
    fn floor_worker_verdict_reaches_the_durable_journal() {
        let journal = std::env::temp_dir().join(format!(
            "claim-executor-floor-worker-verdict-journal-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&journal);
        std::env::set_var(FLOOR_PHASE_JOURNAL_ENV, &journal);
        journal_floor_worker_observation(&ObservedFloorWorker {
            worker: "scoped:batch-a".to_string(),
            termination: ProcessTermination::Exited(1),
            terminal_receipt: FloorWorkerTerminalReceipt::Observed(
                FloorWorkerTerminalReport::Failed("witness row was red".to_string()),
            ),
        });
        std::env::remove_var(FLOOR_PHASE_JOURNAL_ENV);

        let persisted = fs::read_to_string(&journal)
            .expect("the synced worker verdict must survive on the out-of-band journal");
        let _ = fs::remove_file(&journal);
        assert!(
            persisted.contains(
                "\tcoordinator-observation\tfailed\tworker=scoped:batch-a termination=exited:1 detail=witness row was red\n"
            ),
            "the journal must retain the derived verdict rather than only process completion: {persisted:?}"
        );
    }

    #[test]
    fn scoped_execution_authority_inherits_walk_roots_without_widening_subject_roots() {
        let walk_roots = vec!["dag".to_string(), "src/v2".to_string()];
        let subject_roots = vec!["dag".to_string(), "src/v1".to_string()];
        let authority_roots = scoped_execution_authority_source_roots(
            ScopedWitnessExecutionAuthority::InheritedWalkSourceRoots,
            &walk_roots,
        );

        assert_eq!(authority_roots, walk_roots);
        assert_eq!(subject_roots, ["dag", "src/v1"]);
        assert_ne!(
            authority_roots, subject_roots,
            "the selector authority must not be fused into the scoped witness subject envelope"
        );
    }

    #[test]
    fn floor_worker_terminal_crosses_receipt_with_exit_status() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-terminal-cross-{}",
            std::process::id()
        ));
        fs::write(&terminal, "completed\twalk said complete\n").unwrap();
        let status = Command::new("sh").arg("-c").arg("exit 7").status().unwrap();
        let observed = observe_floor_worker("scoped:batch-a", status, &terminal);
        let _ = fs::remove_file(&terminal);
        assert_eq!(observed.termination, ProcessTermination::Exited(7));
        assert!(matches!(
            observed.terminal_receipt,
            FloorWorkerTerminalReceipt::Observed(FloorWorkerTerminalReport::Completed(_))
        ));
        assert_eq!(floor_worker_observation_outcome(&observed).label, "failed");
        assert!(
            !floor_worker_succeeded(&observed),
            "a completed receipt cannot normalize a failing process exit"
        );
    }

    #[test]
    fn floor_worker_refusal_remains_distinct_from_failure() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-refused-{}",
            std::process::id()
        ));
        fs::write(&terminal, "refused\tFreshJobProcess has no realization\n").unwrap();
        let status = Command::new("sh").arg("-c").arg("exit 1").status().unwrap();
        let observed = observe_floor_worker("scoped:batch-a", status, &terminal);
        let _ = fs::remove_file(&terminal);
        let outcome = floor_worker_observation_outcome(&observed);
        assert_eq!(outcome.label, "refused");
        assert!(outcome.detail.contains("FreshJobProcess"));
        assert!(!floor_worker_succeeded(&observed));
    }

    #[cfg(unix)]
    #[test]
    fn floor_worker_signal_death_is_not_flattened_to_an_exit_code() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-signaled-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&terminal);
        let status = Command::new("sh")
            .arg("-c")
            .arg("kill -TERM $$")
            .status()
            .unwrap();
        let observed = observe_floor_worker("scoped:batch-a", status, &terminal);
        assert!(matches!(
            observed.termination,
            ProcessTermination::Signaled(_)
        ));
        assert_eq!(
            floor_worker_observation_outcome(&observed).label,
            "died-without-terminal-receipt"
        );
        assert!(!floor_worker_succeeded(&observed));
    }

    #[test]
    fn floor_worker_signal_overrules_a_completed_terminal_report() {
        let observed = ObservedFloorWorker {
            worker: "scoped:batch-a".to_string(),
            termination: ProcessTermination::Signaled(9),
            terminal_receipt: FloorWorkerTerminalReceipt::Observed(
                FloorWorkerTerminalReport::Completed("walk said complete".to_string()),
            ),
        };
        let outcome = floor_worker_observation_outcome(&observed);
        assert_eq!(outcome.label, "failed");
        assert!(outcome.detail.contains("signal 9"));
        assert!(!floor_worker_succeeded(&observed));
    }
}

#[cfg(test)]
mod witness_walk_flags_tests {
    use super::witness_walk_flags;

    /// The scoped-child boundary deletion narrowed the roster walk. This pins that it narrowed
    /// ONLY the walk: a scoped child still executes rows, so it must still be handed the
    /// per-witness eval budget. RED if the two questions are ever collapsed back into one flag —
    /// the collapse is silent at the type level and would weaken a budget wall while reading as a
    /// pure scope narrowing.
    #[test]
    fn witness_walk_flags_split_the_two_questions() {
        let ordinary = witness_walk_flags(true, false);
        assert!(ordinary.executes_witness_rows);
        assert!(
            ordinary.schedules_discovery,
            "an ordinary worker carrying witness rows must still derive the roster"
        );

        let scoped = witness_walk_flags(true, true);
        assert!(
            scoped.executes_witness_rows,
            "a scoped child executes its frozen rows, so it must keep the eval budget"
        );
        assert!(
            !scoped.schedules_discovery,
            "a scoped child must never re-derive a roster its parent already froze"
        );

        // No rows at all: neither question is yes, for either role.
        for is_scoped in [false, true] {
            let empty = witness_walk_flags(false, is_scoped);
            assert!(!empty.executes_witness_rows);
            assert!(!empty.schedules_discovery);
        }
    }
}

#[cfg(test)]
mod scoped_execution_request_tests {
    use super::*;

    fn entry(name: &str) -> ScopedScheduleEntry {
        ScopedScheduleEntry {
            entry: format!("dag/test/claim/{name}_test.dag"),
            function: format!("{name}_holds"),
            witness_kind: "CorpusWitnessKind".to_string(),
        }
    }

    fn request(batch_id: &str, entries: Vec<ScopedScheduleEntry>) -> ScopedExecutionRequest {
        ScopedExecutionRequest {
            tested_commit: "a".repeat(40),
            tested_tree: "b".repeat(40),
            tool_identity: "tool-identity".to_string(),
            batch_id: batch_id.to_string(),
            source_roots: vec!["dag".to_string(), "src/v1".to_string()],
            source_roots_digest: "digest".to_string(),
            entries,
            scan_dirs: Vec::new(),
            execution_authority: ScopedWitnessExecutionAuthority::InheritedWalkSourceRoots,
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Hermetic,
            },
            clamp: ResolvedFloorBatchClamp {
                overhead_ms: 1,
                per_unit_ms: 2,
                authority: FloorBatchClampAuthority::ScopedBatchOwnedClamp {
                    batch_id: batch_id.to_string(),
                    module_path: "gunbc.ci_layer_roots".to_string(),
                    decl_name: "scoped_witness_batches".to_string(),
                },
            },
            process_isolation: ScopedProcessIsolation::SequentialChildProcess,
            fast_lane_eval_budget_ms: Some(5_000),
            ordinary_budget_ms: Some(60_000),
            batch_stop_policy: FloorBatchStopPolicy::StopBeforeDependents,
        }
    }

    /// EXACT IDENTITY TRANSPORT. The child executes what the parent froze — not a re-derivation of
    /// it — so the round trip must preserve the identity set exactly. RED if a field is dropped
    /// from the carrier or reordered into a different batch.
    #[test]
    fn round_trip_preserves_the_frozen_identity_set() {
        let original = request("v1_claim_scoped", vec![entry("alpha"), entry("beta")]);
        let wire = serde_json::to_string(&[original.clone()]).expect("serialize");
        let decoded: Vec<ScopedExecutionRequest> = serde_json::from_str(&wire).expect("decode");
        assert_eq!(decoded.len(), 1);

        let before: Vec<(String, String)> = original
            .entries
            .iter()
            .map(|e| (e.entry.clone(), e.function.clone()))
            .collect();
        let after: Vec<(String, String)> = decoded[0]
            .entries
            .iter()
            .map(|e| (e.entry.clone(), e.function.clone()))
            .collect();
        assert_eq!(
            before, after,
            "the frozen identity set must survive transport exactly"
        );
        assert_eq!(decoded[0].fast_lane_eval_budget_ms, Some(5_000));
        assert_eq!(decoded[0].ordinary_budget_ms, Some(60_000));

        // The runnable the child executes carries the same identities, in order.
        match decoded[0].to_runnable() {
            Runnable::ScopedWitnessBatch {
                entries, batch_id, ..
            } => {
                assert_eq!(batch_id, "v1_claim_scoped");
                let rebuilt: Vec<(String, String)> = entries
                    .iter()
                    .map(|e| (e.entry.clone(), e.function.clone()))
                    .collect();
                assert_eq!(rebuilt, before);
            }
            _ => panic!("expected a scoped witness batch"),
        }
    }

    /// PLANTED OMITTED IDENTITY. A request that lost one of its frozen rows must not silently
    /// execute the smaller set: the identity sets differ, and this control is what notices.
    #[test]
    fn planted_omitted_identity_is_visible_in_the_request() {
        let full = request("v1_claim_scoped", vec![entry("alpha"), entry("beta")]);
        let truncated = request("v1_claim_scoped", vec![entry("alpha")]);
        let full_ids: Vec<String> = full.entries.iter().map(|e| e.function.clone()).collect();
        let truncated_ids: Vec<String> = truncated
            .entries
            .iter()
            .map(|e| e.function.clone())
            .collect();
        assert_ne!(
            full_ids, truncated_ids,
            "an omitted frozen identity must be a visible difference, never an equal set"
        );
        assert_eq!(truncated_ids.len(), full_ids.len() - 1);
    }

    /// PLANTED WRONG SUBJECT. Selection alone is not verification: a request frozen against another
    /// commit, tree, or tool must refuse rather than execute against whatever is on disk. This
    /// drives the PRODUCTION refusal on each axis — an earlier revision asserted field inequality
    /// on hand-authored data, which restates the comparison instead of exercising it and would
    /// have stayed green if the loader stopped consulting one axis (review 51445). Observing the
    /// live subject is git work and stays with the floor; deciding on it is what refuses here.
    #[test]
    fn planted_wrong_subject_differs_on_every_axis() {
        let frozen = request("v1_claim_scoped", vec![entry("alpha")]);
        for (commit, tree, tool) in [
            (
                "c".repeat(40),
                frozen.tested_tree.clone(),
                frozen.tool_identity.clone(),
            ),
            (
                frozen.tested_commit.clone(),
                "d".repeat(40),
                frozen.tool_identity.clone(),
            ),
            (
                frozen.tested_commit.clone(),
                frozen.tested_tree.clone(),
                "other-tool".to_string(),
            ),
        ] {
            let refusal = refuse_subject_mismatch(&frozen, &commit, &tree, &tool)
                .expect_err("a subject perturbed on any axis must refuse");
            assert!(
                refusal.contains("frozen against a different subject")
                    && refusal.contains(&frozen.batch_id),
                "the refusal must say what it refused and for which batch: {refusal}"
            );
        }

        // The unperturbed subject must be ACCEPTED — otherwise the refusals above would pass for a
        // reason unrelated to what they claim (a check that refuses everything is not a check).
        refuse_subject_mismatch(
            &frozen,
            &frozen.tested_commit,
            &frozen.tested_tree,
            &frozen.tool_identity,
        )
        .expect("the subject it was frozen against must be accepted");
    }

    /// THE DEFECT TWO FULL CI FLOORS PAID FOR. `resolve_floor_batch_stop_policy` is read
    /// unconditionally by every worker, so a scoped child — which has no plan context by
    /// construction after this change — refused there AFTER the ordinary walk had already
    /// succeeded, and the coordinator reported it as "worker returned before producing a walk
    /// terminal receipt" because the refusal path exited without writing one.
    ///
    /// The wall is that every plan-derived value a child reads must travel ON the request. This
    /// pins the population: adding a plan read to the child's path without adding its field here
    /// fails, rather than being discovered by a 70-minute floor.
    #[test]
    fn every_plan_derived_value_the_child_reads_travels_on_the_request() {
        let frozen = request("v1_claim_scoped", vec![entry("alpha")]);

        // Each of these is read by a scoped child at execution time and is derived from the plan
        // the child no longer evaluates. Absence is not "default it" — it is unrepresentable.
        assert_eq!(frozen.fast_lane_eval_budget_ms, Some(5_000));
        assert_eq!(frozen.ordinary_budget_ms, Some(60_000));
        assert_eq!(
            frozen.batch_stop_policy,
            FloorBatchStopPolicy::StopBeforeDependents
        );

        // And they must survive the transport — a field the parent freezes but the JSON drops
        // puts the child right back where it was, reading a plan it does not have.
        let encoded = serde_json::to_string(&vec![frozen.clone()]).expect("serializes");
        let decoded: Vec<ScopedExecutionRequest> =
            serde_json::from_str(&encoded).expect("round-trips");
        assert_eq!(
            decoded[0].fast_lane_eval_budget_ms,
            frozen.fast_lane_eval_budget_ms
        );
        assert_eq!(decoded[0].ordinary_budget_ms, frozen.ordinary_budget_ms);
        assert_eq!(decoded[0].batch_stop_policy, frozen.batch_stop_policy);
    }

    /// cursor review 51430 found this: the manifest this carrier replaces refused a duplicate
    /// batch id on read, and the replacement only JSON-parsed. The coordinator spawns one child
    /// per published row, so a repeated id spawned parallel workers for one batch instead of
    /// stopping the line. This drives the production refusal, not a restatement of it.
    #[test]
    fn a_repeated_batch_id_refuses_before_anything_spawns() {
        let unique = vec![
            request("v1_claim_scoped", vec![entry("alpha")]),
            request("v1_claim_scoped_two", vec![entry("beta")]),
        ];
        assert!(
            refuse_duplicate_scoped_batch_ids(&unique, "fixture").is_ok(),
            "distinct batch ids must pass — otherwise the refusal below proves nothing"
        );

        // Same id, DIFFERENT populations: the contradiction a first-wins read would resolve by
        // silently picking one.
        let duplicated = vec![
            request("v1_claim_scoped", vec![entry("alpha")]),
            request("v1_claim_scoped", vec![entry("beta")]),
        ];
        let refusal = refuse_duplicate_scoped_batch_ids(&duplicated, "fixture")
            .expect_err("a repeated batch id must refuse");
        assert!(
            refusal.contains("duplicate batch id") && refusal.contains("v1_claim_scoped"),
            "the refusal must name what it refused and where: {refusal}"
        );
    }
}

/// Run git in the workspace and return trimmed stdout, or a refusal naming the command.
///
/// Kept when the mirror-drift gate was removed: the behavioral receipt resolves its own baseline
/// through it, so this is a shared helper rather than that gate's private one.
fn git_stdout(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .env("GIT_PAGER", "cat")
        .output()
        .map_err(|e| format!("spawn git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// BEHAVIORAL RECEIPT — selection and corpus-domain derivation
// ---------------------------------------------------------------------------
//
// WHAT THIS ANSWERS THAT NOTHING ELSE DOES. `--required-regen` proves the committed mirrors
// equal what the authority emits, and `--required-regen-fixed-point` proves the emit repeats.
// Neither ever COMPILES the emitted candidate, let alone runs it: the regen host spawns exactly
// rustfmt, rustfmt and git. So the whole promotion story rests on bytes — and DESIGN §7 says in
// as many words that a byte-identical fixed point is NOT the goal, because matching bytes forces
// the emitter to reproduce the seed's warts. The evidence it asks for instead is execution:
// the emitted module compiles and is behaviorally equivalent to the seed on a discriminating
// corpus.
//
// WHY THE NAIVE FORM OF THAT IS VACUOUS TODAY, and this is the whole reason the mode is shaped
// the way it is. After the mirror convergence every emitted candidate is byte-identical to its
// committed mirror (measured: 0 drifted of 129). Emitting both sides and diffing therefore
// compares a program against itself — a receipt that cannot fail is not a weak receipt, it is
// not a receipt. The question with content is the one every future authority edit raises: THE
// AUTHORITY CHANGED AND THE EMISSION CHANGED WITH IT — is the new module still behaviorally the
// same? That is exactly what byte-equality cannot answer, and it has content precisely when
// candidate and seed differ.
//
// SELECTION IS DEMAND-DIRECTED AND DERIVED. Two compiler builds per module across 129 modules is
// not a per-PR gate, it is a budget denominated in the corpus rather than in the change — the
// cost shape DESIGN §5 names, where the bill grows with the repository until it breaks. So the
// subject is the modules whose `.dag` AUTHORITY moved in this diff, and the mapping from an
// authority to its emitted mirror is read off the mirror's own authority header rather
// than from any authored roster. Nothing here can be forged by editing a list.
//
// AND IF THE SELECTION CANNOT BE COMPUTED IT REFUSES. It does not widen to the whole population.
// "I could not determine what changed" and "everything changed" are different states, and
// rendering the first as the second is the absorbing fallback — nothing is missed, so it looks
// safe, while the deficit's frequency drops to zero by construction and the cost lands on every
// future run.

/// Why a changed authority produced no receipt. Every arm is COUNTED and NAMED in the report:
/// a module that silently produces nothing is indistinguishable from a module that passed, and
/// that conflation is what makes an unexecuted receipt read as a green one.
///
/// EVERY ARM HAS A PRODUCER. An earlier revision declared three, adding `NoEmissionChange`
/// (authority moved but emission did not) and `CorpusNotDerivable`. Neither was ever constructed:
/// the first needs the emission this fragment does not yet compute, and the second was a second
/// name for `ModuleCorpusPlan::refused`, which already carries every refusal with the function
/// and type that caused it. A declared-but-unconstructed variant is vocabulary claiming a
/// distinction nothing draws, so both were deleted rather than carried until a producer appeared.
/// `NoEmissionChange` returns when the two-build differential lands and can actually observe it.
///
/// `NoFunctionHasACorpus` is the one that came back, WITH a producer and at a different grain --
/// see its own comment.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptExclusion {
    /// No emitted mirror in the generated population names this authority, under EITHER header
    /// convention the corpus uses (`MirrorIndex`). This is a fact rather than a lookup failure:
    /// the index refuses outright if any self-declared generated file is unindexable, so it
    /// cannot reach this arm while blind. Legitimate whenever an authority emits something that
    /// is not a Rust module mirror -- a workflow YAML, a fixture -- and named rather than
    /// skipped, because a changed authority that maps to nothing must be visible.
    NoEmittedMirror { module_path: String },

    /// The authority declares NO functions at all, so it has no behaviour that could diverge.
    ///
    /// SEPARATE FROM `NoFunctionHasACorpus` BECAUSE THEY ARE DIFFERENT STATES WITH DIFFERENT
    /// REMEDIES, and the arm that used to carry both said something false about this one. A
    /// module with twenty functions none of whose corpora this fragment can derive is a DEFICIT
    /// in the derivation -- it ranks, it names the types responsible, and closing them makes the
    /// module checkable. A module with zero functions is not a deficit at all: there is nothing
    /// to derive and never will be, so reporting it as `none of its 0 declared functions yields
    /// a call` sends a reader to fix a derivation that has no subject, and inflates the
    /// non-derivability population with rows no work can remove.
    NoFunctionDeclared { module_path: String },

    /// NOT ONE FUNCTION in this authority yields a call, so there is no corpus to compare -- and
    /// the arm carries EVERY declared function with the cause that stopped it, because the
    /// deficit lives at the function and a module-level "nothing derived" cannot be acted on.
    ///
    /// WHY THIS IS AN EXCLUSION AND NOT A REFUSAL, which is the whole of the change that
    /// introduced it. The verdict used to be decided by a FILE-LEVEL count crossing zero: a
    /// module where 1 of 20 functions derived ran and reported EQUIVALENT, and a module where 0
    /// of 20 derived hard-failed required CI. One deficit -- functions whose corpus this fragment
    /// cannot derive -- and two opposite verdicts, separated by nothing but where the count
    /// happened to land. Worse, the red had no closing move: an author cannot make
    /// `List<T>` enumerable to get their diff through, so the only way past the gate was to stop
    /// touching the authority. A gate whose sole closing move does not exist does not enforce,
    /// it launders (DESIGN.md, the fixed-point repair).
    ///
    /// So non-derivability is reported at the grain it occurs at -- per function, typed, located,
    /// counted -- and a module with no covered function is EXCLUDED from a differential it can
    /// never take, exactly as `NoEmittedMirror` is. It is not silence: the count of uncovered
    /// functions is printed on every run beside the module count, so the deficit stays rankable
    /// rather than having its frequency zeroed.
    NoFunctionHasACorpus {
        module_path: String,
        /// `(function, why it yields no call)`, one row per declared function.
        uncovered: Vec<(String, RefusalCause)>,
    },
}

/// The domain a corpus actually enumerated, reported as a DERIVED FACT rather than a label.
///
/// Both arms are COVERAGE CLAIMS. There is deliberately no "bounded sample" arm: an earlier
/// revision enumerated `Int` over [-2,2] and `List` to length 3 and reported the result beside
/// genuine exhaustive coverage. That arm is DELETED rather than widened, because a window is not
/// a weaker proof of the same thing — it is a receipt that USUALLY cannot fail, and a
/// usually-passing receipt reports as done while a refusal is counted and ranks for work. The
/// window was also measurably absurd in place: it enumerated a lower-hex-digit predicate over
/// five values containing no hex digit at all.
#[derive(Debug, Clone, PartialEq)]
enum EnumeratedDomain {
    /// Finite closed domain, fully covered: closed nullary enums, Bool, and records over them.
    /// Enumeration IS the domain.
    Exhaustive { cardinality: usize },
    /// Infinite domain, fully covered ANYWAY, because the function cannot distinguish the values
    /// inside a class.
    ///
    /// The argument, in full, because the claim is only as good as it: if a parameter's every
    /// occurrence in the body is an operand of a comparison against an integer literal, then the
    /// literals cut the integers into finitely many classes -- the points themselves and the open
    /// gaps between them -- and every comparison in the function yields the same answer for any
    /// two values drawn from one class. The function's behaviour is therefore constant within a
    /// class, so one representative per class covers the type. This is exhaustive in the same
    /// sense as a closed enum, NOT an approximation of it.
    ///
    /// The premise is what makes it sound, and it is checked rather than assumed: the moment the
    /// parameter is returned, embedded in a record, passed to another function, or arithmetically
    /// combined, its VALUE reaches the output and two members of one class stop agreeing --
    /// `fn shard_count_positive(n: Int) -> Int { if n <= 0 { 1 } else { n } }` returns 5 for 5 and
    /// 7 for 7, both in the class `n > 0`. So any occurrence that is not a literal comparison
    /// REFUSES the whole parameter. The check is conservative in the safe direction: an
    /// occurrence it cannot classify refuses rather than being assumed harmless.
    ExhaustiveOverDerivedPartition {
        cardinality: usize,
        partition: String,
    },
}

/// One parameter's domain as the ACTUAL VALUES, rendered as Rust expressions against the emitted
/// mirror, plus how that domain was established.
///
/// The count is `values.len()`. It is not carried separately, because a cardinality computed
/// beside an enumeration is a second producer of one fact: the two can disagree, and the one that
/// gets reported is the one that never ran. An earlier revision derived only the count, which is
/// why the corpus could be described in the report but never executed.
#[derive(Debug, Clone)]
struct ParameterDomain {
    values: Vec<String>,
    partition: Option<String>,
}

impl EnumeratedDomain {
    fn report(&self) -> String {
        match self {
            EnumeratedDomain::Exhaustive { cardinality } => {
                format!("exhaustive(|domain|={cardinality})")
            }
            EnumeratedDomain::ExhaustiveOverDerivedPartition {
                cardinality,
                partition,
            } => format!("exhaustive-over-partition(|reps|={cardinality}, {partition})"),
        }
    }
}

/// A type declared by the authority, in the only two shapes the fragment can enumerate.
#[derive(Debug, Clone)]
enum DagTypeDecl {
    /// `type AxisGoal = HigherIsBetter | LowerIsBetter` — a closed coproduct of NULLARY variants.
    /// A variant carrying a payload is deliberately NOT this: it would need its payload's domain
    /// enumerated too, and admitting it here without doing that would silently under-cover.
    ClosedNullaryEnum { variants: Vec<String> },
    /// `type DominanceTally { saw_better: Bool, saw_worse: Bool }` — a record over named fields.
    Record { fields: Vec<(String, String)> },
    /// A closed coproduct whose variants carry payloads. NOT enumerable here — each payload needs
    /// its own domain — but recorded as its own kind rather than left unparsed, because the two
    /// produce the same refusal COUNT and completely different refusal MEANINGS. Reporting "not a
    /// closed type declared by this authority" about a type that is declared and IS closed sends
    /// the reader to close something already closed.
    PayloadCoproduct { variant_count: usize },
    /// DECLARED, but carrying no constructor set this fragment can enumerate: an opaque type, an
    /// alias, a form the reader does not model, or a record whose field types it could not read.
    ///
    /// This arm exists so those declarations are REGISTERED rather than dropped. Dropped, they
    /// were reported as "no module in the corpus declares this type" -- a positive claim about the
    /// corpus produced by the reader's own silence, which is the empty-observation narrow with a
    /// wildcard for a cause. Registered, they refuse honestly and rank at zero work.
    DeclaredNotEnumerable { form: String },
}

/// Parse one `.dag` source through the GRAMMAR-OWNED parser and return its module node.
///
/// This replaces a hand-rolled line reader, and the replacement is the point rather than a
/// tidying. The line reader recognised only declarations that fit on one line: measured across
/// the corpus that was 989 of 8796, so it was blind to 88% of the type declarations in the
/// repository, and every one of them was then refused as "not a closed type declared by this
/// authority" -- false, since they are declared and many are closed. Nothing unsound was claimed,
/// because the refusal did stop the line; but a refusal's job beyond stopping is to RANK, and
/// that one sent the reader to close types that were already closed.
///
/// The deeper defect is that a hand-rolled reader is a SECOND PARSER for `.dag` standing beside
/// the real one, which is the single-authority violation in its plainest form: it will be wrong
/// again whenever the grammar moves, and wrong SILENTLY, because a line reader cannot distinguish
/// "did not match" from "is not there". `parse_with_table` can: it returns an error arm, so a
/// source it cannot read REFUSES instead of yielding zero declarations.
fn parse_dag_module_node(
    file: &str,
    source: &str,
) -> Result<Rc<v1_compiler::v1_std_core::Node>, String> {
    use v1_compiler::v1_compiler_parse::parse_with_table;
    use v1_compiler::v1_compiler_tokenize::tokenize;
    use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table, NewlineIndex};

    // The map is built with the runtime's own constructors rather than a `std::HashMap`: the
    // parser's source-index map is an `im::HashMap`, and reaching for the concrete type here
    // would be this file asserting a representation the parser owns.
    let index = build_newline_index(file.to_string(), source.to_string());
    let indices = v1_compiler::v1_rt::rc_map_insert(
        v1_compiler::v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
        file.to_string(),
        index,
    );
    let parsed = parse_with_table(
        tokenize(source.to_string(), file.to_string()),
        indices,
        empty_intern_table(),
    );
    if let Some(err) = parsed.result.error.clone() {
        return Err(format!(
            "{file}: the grammar refused this source: {:?}",
            err.diagnostic
        ));
    }
    parsed
        .result
        .module
        .clone()
        .ok_or_else(|| format!("{file}: the parse produced neither a module nor an error"))
}

/// Render a type-annotation node back to the name the corpus writes, generics included.
fn type_text(node: &v1_compiler::v1_std_core::Node) -> String {
    // Generic ARGUMENTS are children of the type node. Reading `params` instead rendered
    // `List<AxisComparison>` as bare `List`, which then failed the `List<` prefix test and fell
    // through to "not a closed type declared by this authority" — a refusal naming the wrong
    // cause for an entire population, and the reason the first corpus histogram had no List row
    // at all despite lists being the second-largest blocker.
    let args: Vec<String> = node.children.iter().map(|c| type_text(c)).collect();
    if args.is_empty() {
        return node.name.clone();
    }
    format!("{}<{}>", node.name, args.join(", "))
}

/// Read every type declaration off a parsed module node.
///
/// A declaration's SHAPE is read from the substrate's own connective rather than from
/// punctuation: `Disj` is a coproduct and `Conj` is a record. That is why this reader does not
/// care whether the author wrote the body on one line or ten -- the distinction the line reader
/// tripped on does not exist at this layer, which is the strongest evidence that the layer is the
/// right one.
fn type_decls_from_module(
    module: &v1_compiler::v1_std_core::Node,
) -> std::collections::HashMap<String, DagTypeDecl> {
    use v1_compiler::v1_std_core::Connective;
    let mut out = std::collections::HashMap::new();
    for decl in module.children.iter() {
        // WHICH CHILDREN ARE TYPE DECLARATIONS AT ALL -- measured, not named from memory. The
        // grouped shape census over one module's children is unambiguous:
        //
        //   is_type=true   conn=Conj/Disj    body=false  params=0  inferred=false
        //   is_type=false  conn=NoConnective body=true   (compare_int, tally_verdict, no_names)
        //
        // A function or a data row carries a BODY; a type declaration does not. Without this the
        // arm has no notion of its own subject, which is the single cause of BOTH failures here:
        // filtering implicitly on two connectives was too narrow (70% of declarations dropped),
        // and filtering on nothing was too wide (every function registered as a type, 6509 read
        // against 957 authored). Neither was a bug in the filter; both were its absence.
        if decl.body.is_some() {
            continue;
        }
        match decl.connective {
            Connective::Disj => {
                let variants: Vec<String> = decl.children.iter().map(|v| v.name.clone()).collect();
                if variants.is_empty() {
                    continue;
                }
                // A variant with no fields of its own is nullary. A variant carrying a payload
                // needs that payload's domain enumerated too, so the whole coproduct is recorded
                // as payload-carrying rather than being enumerated one-value-per-variant, which
                // would under-cover silently.
                let all_nullary = decl.children.iter().all(|v| v.children.is_empty());
                if all_nullary {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::ClosedNullaryEnum { variants },
                    );
                } else {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::PayloadCoproduct {
                            variant_count: variants.len(),
                        },
                    );
                }
            }
            Connective::Conj => {
                // A field's TYPE is its child, exactly as a parameter's is. Reading
                // `type_annotation` here yielded no fields for every record, so each one was
                // dropped from the type environment and later refused as "not a closed type
                // declared by this authority" — false, and pointing at the wrong repair.
                // A RECORD FIELD'S TYPE IS IN `inferred`, NOT IN `children`. Measured, not
                // assumed -- the fourth node-shape fact this reader needed and the third one an
                // assumption got wrong. A PARAMETER's type is its child, which is true and stays
                // true; reading a field the same way returned nothing for every field of every
                // record, so `filter_map` emptied the list and the guard below dropped the whole
                // declaration. std.pareto read 6 of 13 types and ZERO of its 7 records.
                let mut fields: Vec<(String, String)> = Vec::new();
                let mut unreadable: Vec<String> = Vec::new();
                for f in decl.children.iter() {
                    match f.inferred.as_ref().map(|i| i.as_ref()) {
                        Some(v1_compiler::v1_std_core::InferredNode::Resolved { node }) => {
                            fields.push((f.name.clone(), type_text(node)))
                        }
                        _ => unreadable.push(f.name.clone()),
                    }
                }
                // A PARTIALLY READ RECORD IS NOT A RECORD. Enumerating the fields that happened
                // to resolve would produce a Cartesian product over a SUBSET of the record's
                // fields -- a constructor expression missing fields, which does not compile, and
                // a domain claim that is simply false. So the whole declaration refuses, naming
                // the fields responsible, rather than silently narrowing itself.
                if unreadable.is_empty() && !fields.is_empty() {
                    out.insert(decl.name.clone(), DagTypeDecl::Record { fields });
                } else {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::DeclaredNotEnumerable {
                            form: format!(
                                "record with {} field(s) whose declared type the reader could not \
                                 read: {}",
                                unreadable.len(),
                                unreadable.join(", ")
                            ),
                        },
                    );
                }
            }
            // NO WILDCARD. The declaration vocabulary is CLOSED, and a catch-all here discards
            // exactly the guarantee that closure exists to provide -- silently, in the seed, where
            // nothing enforces the exhaustiveness the substrate would. Every remaining form is
            // registered as declared-but-not-enumerable so it refuses by name instead of becoming
            // a false claim that the corpus does not declare it. Opaque types and aliases are the
            // bulk of this arm and are genuinely not enumerable; that answer is now SAID rather
            // than inferred from an absence.
            other => {
                out.insert(
                    decl.name.clone(),
                    DagTypeDecl::DeclaredNotEnumerable {
                        form: format!(
                            "declaration form {other:?} carries no enumerable constructor set"
                        ),
                    },
                );
            }
        }
    }
    out
}

/// The classes an `Int` parameter's own comparisons cut the integers into, with one
/// representative per class.
#[derive(Debug, Clone, PartialEq)]
struct IntPartition {
    /// The integer literals this parameter is compared against, sorted and deduplicated.
    literals: Vec<i64>,
    /// One value per equivalence class. For literals L the classes are each point `l` and each
    /// open gap between consecutive points, plus the two unbounded ends; the representative set
    /// below hits every non-empty one of them.
    representatives: Vec<i64>,
}

impl IntPartition {
    /// Representatives are `{l-1, l, l+1}` over every literal `l`.
    ///
    /// That this covers every class is a three-case check, not an approximation: the point class
    /// `{l}` is hit by `l`; the gap `(l_i, l_{i+1})` is hit by `l_i + 1` unless the gap is empty
    /// (`l_{i+1} == l_i + 1`); and the two unbounded ends are hit by `min-1` and `max+1`. The
    /// `±1` values are also exactly where an off-by-one divergence lives, which is why they are
    /// taken rather than an arbitrary interior point.
    fn from_literals(mut literals: Vec<i64>) -> IntPartition {
        literals.sort_unstable();
        literals.dedup();
        let mut reps: Vec<i64> = Vec::new();
        for l in &literals {
            for r in [l.saturating_sub(1), *l, l.saturating_add(1)] {
                if !reps.contains(&r) {
                    reps.push(r);
                }
            }
        }
        // No literals means the parameter is never compared against one. Reached only when it is
        // never mentioned at all -- any other use refuses upstream -- so the function ignores it
        // and a single arbitrary value covers the whole type.
        if reps.is_empty() {
            reps.push(0);
        }
        reps.sort_unstable();
        IntPartition {
            literals,
            representatives: reps,
        }
    }

    fn describe(&self) -> String {
        let lits = if self.literals.is_empty() {
            "unused by the body".to_string()
        } else {
            format!(
                "literals {{{}}}",
                self.literals
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        format!(
            "{lits}, reps {{{}}}",
            self.representatives
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/// Derive the partition for one `Int` parameter from the BODY NODE that uses it, or REFUSE
/// naming the occurrence that defeated it.
///
/// This is the whole soundness argument in code. Every occurrence of the parameter must be an
/// operand of a comparison whose other operand is an integer literal. Everything else -- being
/// returned, passed as an argument, added, having a field read off it -- lets the parameter's
/// VALUE reach the output, at which point two members of one class stop agreeing and the
/// partition claim is false. `fn shard_count_positive(n: Int) -> Int { if n <= 0 { 1 } else { n } }`
/// compares `n` against a literal AND returns it, so it returns 5 for 5 and 7 for 7 while both
/// sit in the class `n > 0`. That function refuses here.
///
/// IT WALKS THE PARSED TREE, NOT TEXT. An earlier revision lexed the body itself, and carried the
/// apparatus that implies: a hand tokenizer, a whitelist of what may abut a literal operand so
/// that `a + 1 < n` refused instead of reading `1`, comment stripping so an annotation naming the
/// parameter was not read as a use of it, and a lambda-parameter scan because it could not model
/// scope. Every one of those was a workaround for not having the parse, and every one is deleted:
/// `ExprBinOp` already carries its operator, `ExprLiteral` already holds a typed `LitInt`, and a
/// rebound name is simply a different node. The tell that the old version was in the wrong layer
/// is that its bug -- misreading `a - 1 < n` -- cannot be expressed in this one.
/// WHY a parameter's domain could not be derived -- as a typed cause, not a sentence.
///
/// The census ranks refusals to decide what to ground next, and ranking needs the SUBJECT: the
/// type, or the class of obstacle. The first revision carried only a formatted message and ranked
/// on it, which put `x (used outside a literal comparison...)` at the top with 1500 -- every
/// parameter in the corpus that happens to be named `x`, collapsed into one row that names no
/// type and no work. A parameter name is not a unit of work; the string was doing double duty as
/// an identity and as prose, and it was wrong at the identity job.
///
/// `describe()` is derived from this, so the sentence and the ranking key cannot disagree.
#[derive(Clone, PartialEq, Eq, Hash)]
enum RefusalCause {
    UnboundedString {
        ty: String,
    },
    UnboundedSequence {
        ty: String,
    },
    PayloadCoproduct {
        ty: String,
        variants: usize,
    },
    DeclaredNotEnumerable {
        ty: String,
        form: String,
    },
    /// The type is declared SOMEWHERE in the corpus but is not visible from the module under
    /// plan -- an import-closure gap in the reader, not a property of the type.
    TypeNotVisibleHere {
        ty: String,
    },
    /// No module in the corpus declares this type. Genuinely outside what the authority carries.
    TypeNotDeclaredAnywhere {
        ty: String,
    },
    ProductTooLarge {
        ty: String,
    },
    NestedTooDeep {
        ty: String,
    },
    /// The Int class. Keyed WITHOUT the parameter name -- the name is in the message for locating
    /// it, never in the identity, or one class fragments into as many rows as there are spellings.
    IntValueEscapesComparison {
        param: String,
    },
    IntComparedToNonLiteral {
        param: String,
    },
    IntThroughContainer,
    IntNoBody {
        param: String,
    },
    TupleBudgetExceeded {
        param: String,
    },
    /// Every parameter's domain derived, and the product of them contains NO tuple -- so the
    /// function yields no call while looking, in a count of derivable functions, exactly like one
    /// that yields a thousand.
    ///
    /// COUNTED SEPARATELY FROM THE NEVER-DERIVABLE ONES ON PURPOSE: the remedies differ. A
    /// function refused for `List<T>` needs a length partition; a function with an empty product
    /// needs the enumerator that produced an empty value set fixed. Summing them would name one
    /// piece of work where there are two.
    ///
    /// REACHABILITY, STATED RATHER THAN IMPLIED: no enumerator in this fragment returns an empty
    /// value set today (a zero-variant coproduct is dropped before it becomes a `DagTypeDecl`, and
    /// a zero-field record enumerates to the one empty literal), so there is no live specimen in
    /// the corpus. The producer is the classification in `function_grain_coverage`, which is
    /// executed against this state by `empty_derived_domain_is_uncovered_not_covered`. It exists
    /// because a zero that reads as success is the exact failure this whole arm closes: a
    /// function counted as covered while contributing nothing to the transcript.
    EmptyDerivedDomain,
    /// A refusal reached through a record field, carrying the field path for locating and the
    /// UNDERLYING cause for ranking -- grounding the inner type unlocks the outer record too.
    ViaField {
        ty: String,
        field: String,
        inner: Box<RefusalCause>,
    },
}

impl RefusalCause {
    /// The subject the work would be done to. Ranking key: a `ViaField` ranks as its inner cause,
    /// because the fix is the inner type and counting the wrapper separately would split one
    /// piece of work across as many rows as there are records that embed it.
    fn subject(&self) -> String {
        match self {
            RefusalCause::UnboundedString { ty }
            | RefusalCause::UnboundedSequence { ty }
            | RefusalCause::PayloadCoproduct { ty, .. }
            | RefusalCause::DeclaredNotEnumerable { ty, .. }
            | RefusalCause::ProductTooLarge { ty }
            | RefusalCause::NestedTooDeep { ty } => ty.clone(),
            // THE KIND IS PART OF THE KEY for these two, because the two name different work and
            // the first revision of the split keyed on the bare type name -- so the ranked list
            // printed exactly what it printed before the split, and the whole correction was
            // invisible in its own output. `declared_anywhere` is corpus-global, so a given type
            // falls entirely into one bucket; the tag is therefore stable per type, not a source
            // of fragmentation.
            RefusalCause::TypeNotVisibleHere { ty } => {
                format!("{ty} [declared in corpus, NOT VISIBLE to the reader]")
            }
            RefusalCause::TypeNotDeclaredAnywhere { ty } => {
                format!("{ty} [undeclared anywhere in corpus]")
            }
            RefusalCause::IntValueEscapesComparison { .. } => {
                "Int (value escapes literal comparison)".to_string()
            }
            RefusalCause::IntComparedToNonLiteral { .. } => {
                "Int (compared against a non-literal)".to_string()
            }
            RefusalCause::IntThroughContainer => "Int (reached through a container)".to_string(),
            RefusalCause::IntNoBody { .. } => "Int (no attached body node)".to_string(),
            RefusalCause::TupleBudgetExceeded { .. } => {
                format!("(combination exceeds {MAX_TUPLES_PER_FUNCTION} tuples)")
            }
            RefusalCause::EmptyDerivedDomain => "(derived domain contains no values)".to_string(),
            RefusalCause::ViaField { inner, .. } => inner.subject(),
        }
    }

    fn describe(&self) -> String {
        match self {
            RefusalCause::UnboundedString { ty } => format!("{ty} (unbounded String domain)"),
            RefusalCause::UnboundedSequence { ty } => {
                format!("{ty} (unbounded sequence length; a length partition is not derived)")
            }
            RefusalCause::PayloadCoproduct { ty, variants } => format!(
                "{ty} (closed coproduct, but {variants} variants carry payloads whose domains are \
                 not enumerated)"
            ),
            // THESE TWO WERE ONE ARM, AND THE ONE ARM MISDIRECTED. It read "not a closed type
            // declared by this authority", which parses as "declared, but not closed" -- while
            // the branch is reached ONLY when the type is not in the type environment at all.
            // Node topped the corpus ranking at 798 under that label and was read, by me, as the
            // one big groundable item. It is nothing of the sort: it is a 20-field record with an
            // unbounded String, a recursive List<Node>, and self-reference. Splitting the arm is
            // what makes the difference between "my reader cannot see it" and "the corpus does
            // not have it" reportable, and only the first is work anyone can do.
            RefusalCause::DeclaredNotEnumerable { ty, form } => {
                format!("{ty} (declared, but not enumerable: {form})")
            }
            RefusalCause::TypeNotVisibleHere { ty } => format!(
                "{ty} (declared elsewhere in the corpus but not visible from this module -- an \
                 import-closure gap in the reader, not a property of the type)"
            ),
            RefusalCause::TypeNotDeclaredAnywhere { ty } => {
                format!("{ty} (no module in the corpus declares this type)")
            }
            RefusalCause::ProductTooLarge { ty } => format!(
                "{ty} (record product exceeds {MAX_TUPLES_PER_FUNCTION} values; refusing rather \
                 than sampling)"
            ),
            RefusalCause::NestedTooDeep { ty } => {
                format!("{ty} (nesting deeper than the fragment enumerates)")
            }
            RefusalCause::IntValueEscapesComparison { param } => format!(
                "{param} (used outside a literal comparison, so its value reaches the result and \
                 members of one class need not agree)"
            ),
            RefusalCause::IntComparedToNonLiteral { param } => format!(
                "{param} (compared against a non-literal; the partition is derived from literals, \
                 and a comparison against another parameter or a call would need a joint partition \
                 this fragment does not derive)"
            ),
            RefusalCause::IntThroughContainer => {
                "Int (reached through a container; a partition is \
                 derived from a parameter's own comparisons and does not follow into a field or \
                 element)"
                    .to_string()
            }
            RefusalCause::IntNoBody { param } => {
                format!("{param} (Int parameter on a function with no attached body node)")
            }
            RefusalCause::TupleBudgetExceeded { param } => format!(
                "{param} (corpus exceeds {MAX_TUPLES_PER_FUNCTION} tuples; refusing rather than \
                 sampling)"
            ),
            RefusalCause::EmptyDerivedDomain => {
                "every parameter derived, but their product contains no tuple, so this function \
                 yields no call at all"
                    .to_string()
            }
            RefusalCause::ViaField { ty, field, inner } => {
                format!("{ty}.{field}: {}", inner.describe())
            }
        }
    }
}

/// `Debug` IS `describe()`. A derived `Debug` would print the variant name, which is the one
/// rendering of a refusal that cannot be acted on -- and a test asserting on it would pin the
/// spelling of a Rust identifier rather than the fact.
impl std::fmt::Debug for RefusalCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.describe())
    }
}

fn derive_int_partition(
    param: &str,
    body: &v1_compiler::v1_std_core::Node,
) -> Result<IntPartition, RefusalCause> {
    let mut literals = Vec::new();
    visit_int_param_occurrences(param, body, None, &mut literals)?;
    Ok(IntPartition::from_literals(literals))
}

/// Recurse the body, carrying the enclosing comparison so an occurrence can be judged in context.
///
/// `enclosing` is `Some((op_is_comparison, sibling))` when this node is a direct operand of a
/// binary operation. An occurrence of the parameter is admitted ONLY when that context is a
/// comparison and the sibling is an integer literal.
fn visit_int_param_occurrences(
    param: &str,
    node: &v1_compiler::v1_std_core::Node,
    enclosing: Option<(bool, &v1_compiler::v1_std_core::Node)>,
    literals: &mut Vec<i64>,
) -> Result<(), RefusalCause> {
    use v1_compiler::std_syntax::BinOp;
    use v1_compiler::v1_std_core::ExprData;

    if matches!(node.expr_data.as_ref(), ExprData::ExprVar { .. }) && node.name == param {
        return match enclosing {
            Some((true, sibling)) => match int_literal_of(sibling) {
                Some(v) => {
                    literals.push(v);
                    Ok(())
                }
                None => Err(RefusalCause::IntComparedToNonLiteral {
                    param: param.to_string(),
                }),
            },
            _ => Err(RefusalCause::IntValueEscapesComparison {
                param: param.to_string(),
            }),
        };
    }

    let comparison = match node.expr_data.as_ref() {
        ExprData::ExprBinOp { op, .. } => Some(matches!(
            op,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge
        )),
        _ => None,
    };
    // A binary operation's two operands are each other's sibling; every other node's children
    // are visited with NO enclosing comparison, which is what makes the default a refusal.
    if let (Some(is_cmp), 2) = (comparison, node.children.len()) {
        let lhs = &node.children[0];
        let rhs = &node.children[1];
        visit_int_param_occurrences(param, lhs, Some((is_cmp, rhs)), literals)?;
        visit_int_param_occurrences(param, rhs, Some((is_cmp, lhs)), literals)?;
        return Ok(());
    }
    for child in node.children.iter() {
        visit_int_param_occurrences(param, child, None, literals)?;
    }
    for p in node.params.iter() {
        visit_int_param_occurrences(param, p, None, literals)?;
    }
    if let Some(b) = node.body.as_ref() {
        visit_int_param_occurrences(param, b, None, literals)?;
    }
    Ok(())
}

fn int_literal_of(node: &v1_compiler::v1_std_core::Node) -> Option<i64> {
    use v1_compiler::std_syntax::LiteralValue;
    use v1_compiler::v1_std_core::ExprData;
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitInt { value } => Some(*value),
            _ => None,
        },
        _ => None,
    }
}

/// The cap on one function's corpus. Exceeding it REFUSES rather than sampling: a receipt that
/// ran a subset while reporting the whole is the fabricated-plausible-output failure, and the
/// cheapest way to get one is a Cartesian product nobody bounded.
const MAX_TUPLES_PER_FUNCTION: usize = 4096;

/// Enumerate a parameter's domain as Rust expressions, or REFUSE naming the type that defeated it.
///
/// `module_alias` is the emitted mirror's module, so every constructor is written against the
/// artifact under test rather than against a guess about where a type lives.
fn enumerate_parameter_values(
    ty: &str,
    types: &std::collections::HashMap<String, DagTypeDecl>,
    declared_anywhere: &std::collections::HashSet<String>,
    depth: usize,
    int_partition: Option<&IntPartition>,
    module_alias: &str,
) -> Result<ParameterDomain, RefusalCause> {
    if depth > 4 {
        return Err(RefusalCause::NestedTooDeep { ty: ty.to_string() });
    }
    let ty = ty.trim();
    if ty == "Bool" {
        return Ok(ParameterDomain {
            values: vec!["false".to_string(), "true".to_string()],
            partition: None,
        });
    }
    if ty == "Int" {
        return match int_partition {
            Some(p) => Ok(ParameterDomain {
                values: p
                    .representatives
                    .iter()
                    .map(|r| format!("{r}i64"))
                    .collect(),
                partition: Some(p.describe()),
            }),
            None => Err(RefusalCause::IntThroughContainer),
        };
    }
    if ty.starts_with("List<") {
        return Err(RefusalCause::UnboundedSequence { ty: ty.to_string() });
    }
    match types.get(ty) {
        Some(DagTypeDecl::ClosedNullaryEnum { variants }) => Ok(ParameterDomain {
            values: variants
                .iter()
                .map(|v| format!("{module_alias}::{ty}::{v}"))
                .collect(),
            partition: None,
        }),
        Some(DagTypeDecl::PayloadCoproduct { variant_count }) => {
            Err(RefusalCause::PayloadCoproduct {
                ty: ty.to_string(),
                variants: *variant_count,
            })
        }
        Some(DagTypeDecl::DeclaredNotEnumerable { form }) => {
            Err(RefusalCause::DeclaredNotEnumerable {
                ty: ty.to_string(),
                form: form.clone(),
            })
        }
        Some(DagTypeDecl::Record { fields }) => {
            // The record's own domain is the Cartesian product of its fields'. Built as literal
            // constructor expressions so the driver names every field, which is also what makes a
            // field added upstream a COMPILE error in the driver rather than a silent default.
            let mut acc: Vec<Vec<(String, String)>> = vec![Vec::new()];
            let mut partitioned = Vec::new();
            for (fname, fty) in fields {
                let d = enumerate_parameter_values(
                    fty,
                    types,
                    declared_anywhere,
                    depth + 1,
                    None,
                    module_alias,
                )
                .map_err(|e| RefusalCause::ViaField {
                    ty: ty.to_string(),
                    field: fname.clone(),
                    inner: Box::new(e),
                })?;
                if let Some(pt) = d.partition.clone() {
                    partitioned.push(format!("{fname}: {pt}"));
                }
                let mut next = Vec::new();
                for prefix in &acc {
                    for v in &d.values {
                        if next.len() > MAX_TUPLES_PER_FUNCTION {
                            return Err(RefusalCause::ProductTooLarge { ty: ty.to_string() });
                        }
                        let mut row = prefix.clone();
                        row.push((fname.clone(), v.clone()));
                        next.push(row);
                    }
                }
                acc = next;
            }
            Ok(ParameterDomain {
                values: acc
                    .into_iter()
                    .map(|row| {
                        let inner: Vec<String> =
                            row.into_iter().map(|(f, v)| format!("{f}: {v}")).collect();
                        format!("{module_alias}::{ty} {{ {} }}", inner.join(", "))
                    })
                    .collect(),
                partition: if partitioned.is_empty() {
                    None
                } else {
                    Some(partitioned.join("; "))
                },
            })
        }
        None => Err(if ty == "String" || ty == "NonEmptyStr" {
            RefusalCause::UnboundedString { ty: ty.to_string() }
        } else if declared_anywhere.contains(ty) {
            RefusalCause::TypeNotVisibleHere { ty: ty.to_string() }
        } else {
            RefusalCause::TypeNotDeclaredAnywhere { ty: ty.to_string() }
        }),
    }
}

/// One function as the authority declares it: its parameters and the body node that uses them.
///
/// The body is carried because an `Int` parameter's domain is a fact about HOW THIS FUNCTION USES
/// IT, not about the type: `Int` has no finite domain, but the comparisons a body performs cut it
/// into finitely many classes. A signature-only planner cannot ask that question at all.
#[derive(Debug, Clone)]
struct DagFnSignature {
    name: String,
    params: Vec<(String, String)>,
    body: Option<Rc<v1_compiler::v1_std_core::Node>>,
}

/// Read every function off a parsed module node.
///
/// The third and last hand-rolled reader to go. Its predecessor matched `fn ` at the start of a
/// line and then took the parameter list up to the first `)` on that same line, so a signature
/// spanning lines produced no entry at all -- 14 of `v1.compiler.emit_rust`'s 631 went missing
/// that way, and only a declared-versus-parsed counter made the gap visible rather than reading
/// as a module with a smaller surface. The parser has already separated a declaration's
/// parameters from its body, so none of that arises here.
///
/// That same counter then caught THIS function selecting on the wrong discriminator, which is why
/// it is kept rather than retired once the parser owned the read: two independent readers of one
/// fact disagreeing is the only cheap signal that one of them is wrong.
fn fn_signatures_from_module(module: &v1_compiler::v1_std_core::Node) -> Vec<DagFnSignature> {
    let mut out = Vec::new();
    for decl in module.children.iter() {
        // A declaration is a FUNCTION exactly when it carries a body. Measured against the live
        // tree rather than assumed: `std.pareto`'s items report `compare_int` and friends as
        // `conn=NoConnective children=0 params=N body=true`, while its types report `Disj` with
        // variants or `Conj` with fields and no body. `Connective::Arrow` — which an earlier
        // revision selected on — marks a `Callable` TYPE EXPRESSION, not a declaration, so that
        // filter matched nothing and every module reported `parsed=0`.
        // A FUNCTION carries a body AND a resolved return type in `inferred`. A `data` row also
        // carries a body, which is why selecting on the body alone over-counted: `std.pareto`
        // reported 36 parsed against 33 authored `fn` lines, its three `data` rows swept in.
        // Measured discriminator — `data no_names: List<NonEmptyStr> = []` reports
        // `ta=Some("List") inf=none`, while every function reports `ta=None inf=Resolved{..}`:
        // the declared type of a constant lives in `type_annotation`, a function's return type in
        // `inferred`.
        let (Some(_), Some(_)) = (decl.body.as_ref(), decl.inferred.as_ref()) else {
            continue;
        };
        // A parameter's TYPE is its single CHILD, not a `type_annotation`. Measured: every
        // parameter in `std.pareto` reports `ta=None children=1`. An earlier revision read
        // `type_annotation`, got `None` for every parameter, and derived the empty string as the
        // type name — so all 514 corpus refusals named the SAME empty type and the blocker
        // histogram collapsed to one meaningless row. A refusal that names nothing ranks nothing,
        // which is the defect this whole fragment exists to remove, reproduced in its purest form.
        let params: Vec<(String, String)> = decl
            .params
            .iter()
            .map(|p| {
                let ty = p
                    .children
                    .iter()
                    .next()
                    .map(|t| type_text(t))
                    .unwrap_or_else(|| "<parameter with no type node>".to_string());
                (p.name.clone(), ty)
            })
            .collect();
        out.push(DagFnSignature {
            name: decl.name.clone(),
            params,
            body: decl.body.clone(),
        });
    }
    out
}

/// What the fragment can say about one module's surface.
struct ModuleCorpusPlan {
    module_path: String,
    /// Functions whose every parameter domain derived, with the combined domain per function AND
    /// the argument tuples that domain actually consists of. The tuples are what the driver runs;
    /// carrying only the domain description is how an earlier revision could report a corpus it
    /// had never executed.
    derivable: Vec<(String, EnumeratedDomain, Vec<Vec<String>>)>,
    /// Functions that defeated derivation, each naming the type responsible.
    refused: Vec<(String, RefusalCause)>,
    /// `fn` lines the authority declares vs signatures actually parsed. These must agree; a gap
    /// means the parser missed a form, and reporting the pair is what stops a silent miss from
    /// reading as a module with a small surface.
    declared_fn_lines: usize,
    parsed_signatures: usize,
}

/// One module's surface partitioned AT FUNCTION GRAIN: which declared functions actually yield a
/// call, and which yield none and why.
///
/// This is the fact the receipt is denominated in. `ModuleCorpusPlan::derivable` is close to it
/// but not equal to it: a function whose parameter domains all derived while their product is
/// empty sits in `derivable` and contributes nothing, so counting that vector counts a coverage
/// claim rather than coverage.
#[derive(Debug)]
struct FunctionGrainCoverage {
    /// Functions that yield at least one call, with how many.
    covered: Vec<(String, usize)>,
    /// Functions that yield none, each with the cause. `RefusalCause::EmptyDerivedDomain`
    /// distinguishes "derived, but to nothing" from "did not derive".
    uncovered: Vec<(String, RefusalCause)>,
}

impl FunctionGrainCoverage {
    fn calls(&self) -> usize {
        self.covered.iter().map(|(_, n)| n).sum()
    }
}

/// The partition itself. Pure over the plan, which is what lets both arms of it be executed
/// against hand-built states rather than only against whatever the live corpus happens to hold.
fn function_grain_coverage(plan: &ModuleCorpusPlan) -> FunctionGrainCoverage {
    let mut covered = Vec::new();
    let mut uncovered = Vec::new();
    for (name, _domain, tuples) in &plan.derivable {
        if tuples.is_empty() {
            uncovered.push((name.clone(), RefusalCause::EmptyDerivedDomain));
        } else {
            covered.push((name.clone(), tuples.len()));
        }
    }
    for (name, cause) in &plan.refused {
        uncovered.push((name.clone(), cause.clone()));
    }
    FunctionGrainCoverage { covered, uncovered }
}

/// THE PLAN-GRAIN HALF OF THE ONE SELECTION CRITERION: is there a subject to compare?
///
/// `Ok(None)` selects. `Ok(Some(_))` excludes, typed and counted. `Err(_)` refuses, because the
/// third state is the reader's own blindness and it is not a property of the authority.
///
/// The three answers used to be two, and the missing distinction is the whole point. `zero
/// functions covered` was one arm covering three different states -- an authority with no
/// functions at all, an authority whose functions all defeat derivation, and a parse this
/// fragment failed to read -- with one message that was false about two of them. That is the
/// state-space conflation DESIGN.md names: one value standing for several states whose remedies
/// differ. Splitting it costs nothing and makes each population rankable on its own terms.
fn plan_grain_selection(
    plan: &ModuleCorpusPlan,
    coverage: FunctionGrainCoverage,
) -> Result<Option<ReceiptExclusion>, String> {
    // THE READER'S OWN BLINDNESS IS NOT A PROPERTY OF THE AUTHORITY. The `fn ` line count and the
    // parsed signature count are two readers of one fact, and the pair is carried precisely so a
    // disagreement is visible. When the parser sees none of the functions the source declares,
    // `zero covered` is IGNORANCE rather than the fact that nothing is derivable, and excluding
    // on it would publish a reader deficit as a fact about the module.
    if plan.parsed_signatures == 0 && plan.declared_fn_lines > 0 {
        return Err(format!(
            "behavioral-receipt: {} declares {} `fn ` line(s) and the parser produced no \
             signature at all. The two readers disagree, so this selection cannot say whether \
             the module has no derivable function or whether this fragment simply failed to read \
             it -- and those have opposite remedies. Refusing rather than excluding: an exclusion \
             here would publish the reader's blindness as a fact about the authority",
            plan.module_path, plan.declared_fn_lines
        ));
    }
    if plan.parsed_signatures == 0 {
        return Ok(Some(ReceiptExclusion::NoFunctionDeclared {
            module_path: plan.module_path.clone(),
        }));
    }
    if coverage.covered.is_empty() {
        return Ok(Some(ReceiptExclusion::NoFunctionHasACorpus {
            module_path: plan.module_path.clone(),
            uncovered: coverage.uncovered,
        }));
    }
    Ok(None)
}

/// Does a plan-grain exclusion SURVIVE what the generated-artifact population says about the
/// mirror? `None` means it does not -- the module is selected after all.
///
/// PURE, AND SEPARATED FROM THE ASK FOR ONE REASON: the divergence it decides has an EMPTY
/// POPULATION today, so nothing in the live corpus executes the branch that matters. An empty
/// population is not a safe place to leave a behaviour undecided -- the first real member would
/// settle it by accident, and whoever met it would be debugging a difference nobody had written
/// down. Pulling the decision out of the loop lets a planted control state the intent and execute
/// it permanently (DESIGN.md 4b(4): the evidence stays enrolled, it does not retire when the
/// production population catches up).
fn exclusion_survives_generated_artifact_population(
    exclusion: ReceiptExclusion,
    body: &GeneratedArtifactPathBody,
) -> Option<ReceiptExclusion> {
    match body {
        // A POSITIVE ANSWER: this path is a module mirror, not a generated artifact. Only here is
        // "no function yields a call" the whole story, so only here is the exclusion a fact.
        GeneratedArtifactPathBody::NotGenerated => Some(exclusion),
        // Produced OR Refused: either way the path belongs to the generated-artifact population,
        // whose subject is BYTES rather than behaviour -- and for that subject a declared function
        // is not required, because the identity check calls nothing. The differential loop is
        // where that population's answer is reported (identity, drift, or a generator refusal);
        // deciding it here would be a second adjudicator of one question.
        GeneratedArtifactPathBody::Produced(_) | GeneratedArtifactPathBody::Refused(_) => None,
    }
}

/// A plan CARRYING AT LEAST ONE CALL. There is no other way to build one.
///
/// The differential used to check this itself -- `if total == 0 { Refused }` -- which is
/// validation of a state the caller was free to construct (DESIGN.md section 5: prefer making the
/// bad state unwritable to flagging it afterwards). Selection now decides admission at function
/// grain, and hands the differential a value that cannot represent an empty corpus, so the
/// comparison-over-nothing the old guard existed to catch has no spelling. The guard is deleted
/// rather than kept beside the new arm: two answers to one question is the dual authority
/// DESIGN.md section 3 forbids, and the surviving one would be the one that never runs.
struct AdmittedPlan<'a> {
    plan: &'a ModuleCorpusPlan,
    coverage: FunctionGrainCoverage,
}

impl<'a> AdmittedPlan<'a> {
    /// `Err` carries the function-grain partition of a module no function of which yields a call
    /// -- the rows the exclusion is built from, so the refusal and the report read the same fact.
    fn of(plan: &'a ModuleCorpusPlan) -> Result<AdmittedPlan<'a>, FunctionGrainCoverage> {
        let coverage = function_grain_coverage(plan);
        if coverage.covered.is_empty() {
            Err(coverage)
        } else {
            Ok(AdmittedPlan { plan, coverage })
        }
    }
}

/// Every `.dag` module reachable from the source roots, keyed by its declared module path.
fn collect_dag_module_sources(
    source_roots: &[String],
) -> Result<std::collections::HashMap<String, String>, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    let mut out = std::collections::HashMap::new();
    let mut stack: Vec<PathBuf> = source_roots.iter().map(|r| workspace.join(r)).collect();
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(mp) = v1_compiler::cli_run::extract_module_path_public(&content) {
                        out.insert(mp, content);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// The types VISIBLE to a module: its own declarations plus those of its transitive imports.
///
/// THIS RESOLVES, IT DOES NOT WIDEN, and the distinction is the whole risk of this function.
/// Reaching further to find a declaration feels like covering more, and the refusal count can
/// quietly drop for the wrong reason. So what is found is put through `derive_parameter_domain`
/// UNCHANGED: a record with a `String` field still refuses, a `String where` refinement still
/// refuses, a coproduct with payload variants still refuses. The only thing that changes is
/// WHERE a declaration may be found — `Ordering` is `Less | Equal | Greater` whether it is
/// declared in this module or in `std.algebra`, and refusing it for its address rather than its
/// shape was an artifact of the implementation, not a property of the fragment.
///
/// The control for that claim is external and specific: `std.content_hash` refuses 26 of its 27
/// functions before that change, and must still refuse 26 of 27 after it. If that number moves,
/// this resolved nothing and widened something.
///
/// BOTH READS GO THROUGH THE GRAMMAR. An earlier revision found declarations with a line-based
/// reader and found imports with `line.strip_prefix("import ")`. Each was a second implementation
/// of something the parser already owns, and the first was measurably wrong -- blind to 88% of
/// the corpus's type declarations. They are gone rather than kept working: a hand reader that
/// currently agrees with the grammar has exactly the standing the line reader had until it was
/// measured. Imports now come from the module node's `params` and declarations from its
/// `children`, which is where the parser puts them.
///
/// A module that will not parse REFUSES the whole plan rather than contributing nothing. Silently
/// skipping it would make an unreadable import indistinguishable from an import that declares no
/// types, and the second is a fine reason to derive nothing while the first is not.
fn visible_type_decls(
    module_path: &str,
    source: &str,
    modules: &std::collections::HashMap<String, String>,
) -> Result<std::collections::HashMap<String, DagTypeDecl>, String> {
    let mut merged = std::collections::HashMap::new();
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![(module_path.to_string(), source.to_string())];
    let mut depth_guard = 0usize;
    while let Some((mp, src)) = queue.pop() {
        depth_guard += 1;
        if depth_guard > 4096 {
            return Err(format!(
                "{module_path}: import closure exceeded 4096 modules; refusing rather than \
                 reporting a partial type environment"
            ));
        }
        if !seen.insert(mp.clone()) {
            continue;
        }
        let node = parse_dag_module_node(&format!("{mp}.dag"), &src)?;
        // A module's OWN declarations win: a local name shadows an imported one, and taking the
        // import would answer with a different type than the module compiles against.
        for (name, decl) in type_decls_from_module(&node) {
            merged.entry(name).or_insert(decl);
        }
        for imp in node.params.iter() {
            if let Some(next_src) = modules.get(&imp.name) {
                queue.push((imp.name.clone(), next_src.clone()));
            }
        }
    }
    Ok(merged)
}

/// The one entry point. An earlier revision also had a module-local planner that consulted only
/// the types declared in the module under plan; it is DELETED rather than kept beside this one.
/// Two resolvers over one question would answer differently for any imported type — which is the
/// exact defect this change fixes — and keeping the narrower one available is how a caller
/// silently gets the old answer back.
/// Every type name any module in the corpus declares.
///
/// Exists to separate two states a single refusal arm used to conflate: a type the reader could
/// not SEE from this module, and a type the corpus does not HAVE. Only the first is work.
fn declared_type_names(
    modules: &std::collections::HashMap<String, String>,
) -> Result<std::collections::HashSet<String>, String> {
    let mut out = std::collections::HashSet::new();
    for (mp, src) in modules {
        let node = parse_dag_module_node(&format!("{mp}.dag"), src)?;
        for (name, _decl) in type_decls_from_module(&node) {
            out.insert(name);
        }
    }
    Ok(out)
}

fn plan_module_corpus(
    module_path: &str,
    source: &str,
    module: &v1_compiler::v1_std_core::Node,
    types: &std::collections::HashMap<String, DagTypeDecl>,
    declared_anywhere: &std::collections::HashSet<String>,
    module_alias: &str,
) -> ModuleCorpusPlan {
    let sigs = fn_signatures_from_module(module);
    // Kept as a CROSS-CHECK, not as the source of truth. The authored `fn ` line count and the
    // parsed function count come from two different readers, so a disagreement means one of them
    // is wrong -- which is exactly how the line reader's 14 missing signatures were found. Now
    // that the parser owns the read they should agree, and the pair is reported so that a future
    // divergence is visible rather than silently halving a module's surface.
    let declared_fn_lines = source
        .lines()
        .filter(|l| l.trim_start().starts_with("fn "))
        .count();
    let mut derivable = Vec::new();
    let mut refused = Vec::new();
    for sig in &sigs {
        let mut partitioned = false;
        let mut partitions = Vec::new();
        let mut failure: Option<RefusalCause> = None;
        // Tuples are accumulated as a Cartesian product across parameters. A zero-parameter
        // function has exactly one tuple -- the empty one -- which is a real call, not an absence.
        let mut tuples: Vec<Vec<String>> = vec![Vec::new()];
        for (pname, pty) in &sig.params {
            // An `Int` parameter is asked about its own occurrences first. The partition IS its
            // domain, so a refusal here refuses the function -- there is no fallback window to
            // drop to, by design.
            let int_partition = if pty.trim() == "Int" {
                // No body means no occurrences to justify a partition. That REFUSES rather than
                // defaulting to "unused, so one value covers it": a declaration whose body the
                // parser did not attach is an unknown, and treating an unknown as an empty set of
                // uses is the narrow that turns "I could not see" into "there was nothing there".
                let Some(body) = sig.body.as_ref() else {
                    failure = Some(RefusalCause::IntNoBody {
                        param: pname.clone(),
                    });
                    break;
                };
                match derive_int_partition(pname, body) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        failure = Some(e);
                        break;
                    }
                }
            } else {
                None
            };
            match enumerate_parameter_values(
                pty,
                types,
                declared_anywhere,
                0,
                int_partition.as_ref(),
                module_alias,
            ) {
                Ok(d) => {
                    if let Some(pt) = d.partition.clone() {
                        partitioned = true;
                        partitions.push(format!("{pname}: {pt}"));
                    }
                    let mut next: Vec<Vec<String>> = Vec::new();
                    for prefix in &tuples {
                        for v in &d.values {
                            if next.len() >= MAX_TUPLES_PER_FUNCTION {
                                break;
                            }
                            let mut row = prefix.clone();
                            row.push(v.clone());
                            next.push(row);
                        }
                    }
                    if next.len() >= MAX_TUPLES_PER_FUNCTION {
                        failure = Some(RefusalCause::TupleBudgetExceeded {
                            param: pname.clone(),
                        });
                        break;
                    }
                    tuples = next;
                }
                Err(offending) => {
                    failure = Some(offending);
                    break;
                }
            }
        }
        match failure {
            Some(offending) => refused.push((sig.name.clone(), offending)),
            None => {
                // The reported cardinality IS the number of tuples that will run. Deriving it
                // separately would let the report and the execution disagree.
                let cardinality = tuples.len();
                let domain = if partitioned {
                    EnumeratedDomain::ExhaustiveOverDerivedPartition {
                        cardinality,
                        partition: partitions.join("; "),
                    }
                } else {
                    EnumeratedDomain::Exhaustive { cardinality }
                };
                derivable.push((sig.name.clone(), domain, tuples));
            }
        }
    }
    ModuleCorpusPlan {
        module_path: module_path.to_string(),
        derivable,
        refused,
        declared_fn_lines,
        parsed_signatures: sigs.len(),
    }
}

/// The verdict for one candidate module. Three arms, and the third is not a soft pass.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptVerdict {
    /// Both builds ran the derived corpus and every call THAT COULD BE COMPARED agreed.
    ///
    /// `nondeterministic_calls` is carried on this arm rather than printed beside it because a
    /// green with an excluded population is a DIFFERENT claim from a green over everything, and
    /// separating the two would let the weaker one be read as the stronger. Zero is the ordinary
    /// case and reads as the full claim.
    Equivalent {
        calls: usize,
        nondeterministic_calls: usize,
        nondeterministic_functions: Vec<String>,
    },
    /// Both builds ran and at least one call disagreed. The first difference is carried because
    /// a count alone cannot be acted on.
    Divergent {
        calls: usize,
        first_difference: String,
    },
    /// The comparison could NOT be taken. Never reported as equivalence: an emit that failed, or
    /// a driver that would not compile, is ignorance, and rendering ignorance as the clean verdict
    /// is the empty-observation narrow. A corpus with nothing in it is NOT among these any more --
    /// it never reaches the differential, because `AdmittedPlan` cannot carry it.
    Refused { reason: String },
    /// EVERY derived call in this module renders unstably, so no comparison exists to take.
    ///
    /// Not `Equivalent` (nothing was compared) and not `Divergent` (nothing disagreed about the
    /// program). Not `Refused` either: a refusal in this fragment means the measurement could not
    /// be attempted, whereas this one WAS attempted and produced a well-defined result -- the
    /// subject is unaskable, and the fix lives in emission rather than in this gate or in the
    /// diff under test.
    NondeterministicRendering {
        unstable_calls: usize,
        functions: Vec<String>,
    },
}

/// Generate the driver: one `println!` per call in the derived corpus.
///
/// The transcript line carries the function name and the argument expressions as authored, so a
/// divergence names the exact call rather than an index into a product nobody can reconstruct.
/// Output goes through `{:?}`, which is why the fragment admits only types the emitted mirror
/// derives `Debug` on.
fn generate_receipt_driver(module_alias: &str, plan: &ModuleCorpusPlan) -> String {
    let mut out = String::new();
    out.push_str("// GENERATED by claim_executor --behavioral-receipt. Do not edit.\n");
    // FULLY QUALIFIED, with no `use ... as m` alias. An earlier revision aliased the module for
    // calls while the enumerated constructor values were rendered against the bare module name,
    // so the driver referred to the same module two ways and only one of them resolved. One
    // spelling, produced in one place, cannot drift from itself.
    out.push_str("fn main() {\n");
    for (name, _domain, tuples) in &plan.derivable {
        for args in tuples {
            let call = format!("{module_alias}::{name}({})", args.join(", "));
            let shown = args.join(", ").replace('"', "\\\"");
            out.push_str(&format!(
                "    println!(\"{name}({shown}) = {{:?}}\", {call});\n"
            ));
        }
    }
    out.push_str("}\n");
    out
}

/// One driver's output, plus WHICH of its lines are not a function of the program alone.
///
/// A line is `unstable` when two executions of the SAME binary printed different text for it.
/// That is proof, not inference: the code, the inputs and the build are identical across the two
/// runs, so anything that differs came from somewhere other than the program's meaning. In this
/// corpus the somewhere is `HashMap`/`HashSet` iteration order reaching `{:?}`, whose seed is
/// randomized per process -- measured at 20 distinct transcripts over 20 executions of one
/// unchanged binary for `std.algebra::kernel_algebra_profile_value`.
///
/// WHY THIS IS MEASURED RATHER THAN DECIDED FROM THE TYPE. Order-dependent rendering is a
/// property of the value's TRANSITIVE shape: a record CONTAINING a map renders nondeterministically
/// while its own return type says `Record`. Any check keying on the outermost constructor
/// under-refuses by construction, and it under-refuses SILENTLY -- the missed call lands in
/// `Divergent`, indistinguishable from a real divergence. Running the binary twice keys on the
/// property itself, so there is no type walk to keep in sync with the corpus.
///
/// THE RESIDUE, STATED: two randomized renderings can coincide, so a subject that happened to
/// agree twice is not caught. That makes every count derived from this a FLOOR, and the floor is
/// printed as such rather than left in this comment. It is a residue that SHRINKS with more
/// executions, unlike a structural blind spot, but this is not an argument for adding runs
/// speculatively -- one extra run is what the evidence to date justifies.
#[derive(Debug, Clone, PartialEq)]
struct DriverTranscript {
    lines: Vec<String>,
    /// Indices into `lines` that differed between the two executions.
    unstable: std::collections::BTreeSet<usize>,
}

impl DriverTranscript {
    fn of(first: Vec<String>, second: Vec<String>) -> Self {
        // A length difference between two runs of one binary is itself instability, and it is
        // not attributable to any single index -- so every line of the longer run is marked
        // rather than none. Silently comparing the common prefix would hide it.
        let unstable = if first.len() != second.len() {
            (0..first.len().max(second.len())).collect()
        } else {
            first
                .iter()
                .zip(second.iter())
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, _)| i)
                .collect()
        };
        Self {
            lines: first,
            unstable,
        }
    }
}

/// Build the crate as it currently stands, compile the driver against it, and return the
/// transcript. Both halves of the differential go through THIS function, so the two transcripts
/// cannot differ because of how they were produced.
/// WHICH crate the driver is built against and linked to.
///
/// These three were literals inside `run_receipt_driver`. A hardwired `-p v1-compiler` is policy
/// standing inside a mechanism (DESIGN §3: an argv carrying a literal it should receive as a
/// parameter), and it is exactly what made the mode impossible to exercise against anything but
/// the production mirror -- so its own arms could only ever be run by a human with a script.
/// Parameterised, the control and the production path are ONE code path with one argument
/// different, rather than two machineries that could drift apart while both looked green.
struct ReceiptCrate {
    package: String,
    extern_name: String,
    rlib: String,
}

impl ReceiptCrate {
    fn v1_compiler() -> Self {
        Self {
            package: "v1-compiler".to_string(),
            extern_name: "v1_compiler".to_string(),
            rlib: "libv1_compiler.rlib".to_string(),
        }
    }

    fn receipt_fixture() -> Self {
        Self {
            package: "v1-receipt-fixture".to_string(),
            extern_name: "v1_receipt_fixture".to_string(),
            rlib: "libv1_receipt_fixture.rlib".to_string(),
        }
    }
}

fn run_receipt_driver(
    workspace: &std::path::Path,
    krate: &ReceiptCrate,
    driver_src: &str,
    label: &str,
) -> Result<DriverTranscript, String> {
    let drv_dir = workspace.join("target/behavioral-receipt");
    fs::create_dir_all(&drv_dir).map_err(|e| format!("create {}: {e}", drv_dir.display()))?;
    let src_path = drv_dir.join(format!("driver_{label}.rs"));
    fs::write(&src_path, driver_src).map_err(|e| format!("write driver: {e}"))?;

    let lib = Command::new("cargo")
        .args(["build", "--release", "-p", &krate.package, "--lib"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn cargo ({label}): {e}"))?;
    if !lib.status.success() {
        return Err(format!(
            "{label}: the crate did not build; the candidate is not admissible without a compile. \
             {}",
            String::from_utf8_lossy(&lib.stderr)
                .lines()
                .filter(|l| l.starts_with("error"))
                .take(4)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    let bin_path = drv_dir.join(format!("driver_{label}"));
    let rustc = Command::new("rustc")
        .args(["--edition", "2021", "-O"])
        .arg(&src_path)
        .arg("--extern")
        .arg(format!(
            "{}={}",
            krate.extern_name,
            workspace.join("target/release").join(&krate.rlib).display()
        ))
        .arg("-L")
        .arg(workspace.join("target/release/deps"))
        .arg("-o")
        .arg(&bin_path)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn rustc ({label}): {e}"))?;
    if !rustc.status.success() {
        return Err(format!(
            "{label}: the driver did not compile against the mirror: {}",
            String::from_utf8_lossy(&rustc.stderr)
                .lines()
                .filter(|l| l.starts_with("error"))
                .take(4)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    // TWICE, AND THE SECOND RUN IS THE POINT. The two are executions of the SAME BINARY -- the
    // cargo build and the rustc compile above are already paid, so this costs milliseconds and
    // not a rebuild. Two executions of one unchanged binary that disagree PROVE the disagreeing
    // call renders nondeterministically, which is the only way to learn that fact: it is a
    // property of the value's rendering, not of its type, so nothing before the run can know it.
    let first = run_driver_binary(workspace, &bin_path, label)?;
    let second = run_driver_binary(workspace, &bin_path, label)?;
    Ok(DriverTranscript::of(first, second))
}

fn run_driver_binary(
    workspace: &std::path::Path,
    bin_path: &std::path::Path,
    label: &str,
) -> Result<Vec<String>, String> {
    let run = Command::new(bin_path)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn driver ({label}): {e}"))?;
    if !run.status.success() {
        return Err(format!(
            "{label}: the driver ran and exited {}; a corpus call panicking is a behavioural fact, \
             but it is not one this comparison can attribute, so it refuses",
            run.status
        ));
    }
    Ok(String::from_utf8_lossy(&run.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// The two-build differential for ONE candidate module.
///
/// Seed transcript first, from the tree as committed. Then the emitted candidate is written over
/// its mirror, the crate is rebuilt, and the SAME driver runs again. The mirror is restored on
/// every path, including failure -- leaving a candidate in the tree would make the next reader's
/// measurement a lie.
///
/// This is what the whole fragment is for. CI proves the committed mirrors equal what the
/// authority emits and that the emit repeats; it never compiles the candidate, let alone runs it.
/// A byte comparison cannot distinguish a rename from a semantic change, and DESIGN §7 says a
/// byte-identical fixed point is explicitly NOT the goal -- behavioural equivalence on a
/// discriminating corpus is.
fn behavioral_differential(
    workspace: &std::path::Path,
    krate: &ReceiptCrate,
    mirror_path: &std::path::Path,
    candidate_source: &str,
    admitted: &AdmittedPlan<'_>,
    module_alias: &str,
) -> ReceiptVerdict {
    // No emptiness check: `AdmittedPlan` cannot be built from a plan with no call in it, so the
    // comparison-over-nothing this function used to guard against is unrepresentable here rather
    // than rejected here.
    let plan = admitted.plan;
    let driver = generate_receipt_driver(module_alias, plan);
    let shown = mirror_path.display().to_string();

    // THE SEED MUST ACTUALLY BE THE SEED (review 54094). This function installs a candidate over
    // the committed mirror and restores it on every path -- but a process killed between the two
    // leaves the candidate in the tree, and the NEXT run would then read it as the committed
    // bytes and compare the candidate against itself. That comparison answers EQUIVALENT, which
    // is the worst available wrong answer: a green that means nothing, produced by a mechanism
    // whose entire job is to be trusted.
    //
    // The residue cannot be prevented -- no arrangement of writes survives SIGKILL -- so it is
    // made LOUD instead. A dirty path is a refusal, not a warning, and it names the recovery.
    match git_stdout(
        workspace,
        &[
            "status",
            "--porcelain",
            "--",
            &mirror_path.to_string_lossy(),
        ],
    ) {
        Ok(out) if !out.trim().is_empty() => {
            return ReceiptVerdict::Refused {
                reason: format!(
                    "{shown} is modified in the working tree, so the bytes this would read as the \
                     SEED are not the committed bytes. Most likely a previous run was killed \
                     between installing a candidate and restoring it -- in which case this run \
                     would compare that candidate against itself and answer EQUIVALENT. Restore \
                     it (`git checkout -- {shown}`) before measuring: {}",
                    out.trim()
                ),
            }
        }
        Ok(_) => {}
        Err(e) => {
            return ReceiptVerdict::Refused {
                reason: format!(
                    "cannot determine whether {shown} is clean ({e}), so cannot establish that the \
                     seed transcript comes from committed bytes"
                ),
            }
        }
    }
    let committed = match fs::read_to_string(mirror_path) {
        Ok(c) => c,
        Err(e) => {
            return ReceiptVerdict::Refused {
                reason: format!("read {shown}: {e}"),
            }
        }
    };

    let seed = match run_receipt_driver(workspace, krate, &driver, "seed") {
        Ok(t) => t,
        Err(e) => return ReceiptVerdict::Refused { reason: e },
    };

    if let Err(e) = fs::write(mirror_path, candidate_source) {
        return ReceiptVerdict::Refused {
            reason: format!("install candidate {shown}: {e}"),
        };
    }
    let candidate = run_receipt_driver(workspace, krate, &driver, "candidate");
    // Restore BEFORE interpreting the result, so no early return can leave the candidate in place.
    if let Err(e) = fs::write(mirror_path, &committed) {
        return ReceiptVerdict::Refused {
            reason: format!(
                "restore {shown} after the candidate build: {e}. The tree may still hold \
                 the candidate; do not trust a later measurement without checking"
            ),
        };
    }
    let candidate = match candidate {
        Ok(t) => t,
        Err(e) => return ReceiptVerdict::Refused { reason: e },
    };

    if seed.lines.len() != candidate.lines.len() {
        return ReceiptVerdict::Divergent {
            calls: seed.lines.len(),
            first_difference: format!(
                "transcript lengths differ: seed {} lines, candidate {} lines",
                seed.lines.len(),
                candidate.lines.len()
            ),
        };
    }
    // THE UNION, NOT THE SEED'S SET ALONE. Each side is its own binary and each was measured
    // independently, so a call can render unstably in one and (by coincidence, on that pair of
    // runs) stably in the other. Comparing a line either side proved unstable would score a
    // coin flip as a behavioural difference, which is the fabricated-difference failure this
    // whole change exists to remove.
    let unstable: std::collections::BTreeSet<usize> =
        seed.unstable.union(&candidate.unstable).copied().collect();
    let excluded = nondeterministic_call_functions(admitted, &unstable);

    let compared: Vec<(usize, (&String, &String))> = seed
        .lines
        .iter()
        .zip(candidate.lines.iter())
        .enumerate()
        .filter(|(i, _)| !unstable.contains(i))
        .collect();

    // Nothing left to compare is NOT equivalence, and it is not a divergence either: it is a
    // module whose every derived call renders unstably, so this fragment cannot ask it anything
    // honestly. Reported as its own verdict rather than folded into either, because the action
    // it calls for -- make emission deterministic -- is neither "fix the diff" nor "nothing to do".
    if compared.is_empty() {
        return ReceiptVerdict::NondeterministicRendering {
            unstable_calls: unstable.len(),
            functions: excluded,
        };
    }
    for (_, (a, b)) in &compared {
        if a != b {
            return ReceiptVerdict::Divergent {
                calls: compared.len(),
                first_difference: format!("seed: {a}  |  candidate: {b}"),
            };
        }
    }
    ReceiptVerdict::Equivalent {
        calls: compared.len(),
        nondeterministic_calls: unstable.len(),
        nondeterministic_functions: excluded,
    }
}

/// Which declared functions own the calls at `unstable`.
///
/// The driver prints one line per call in exactly the order `AdmittedPlan` enumerates them, so
/// the mapping is positional and derived from the same iteration that produced the transcript --
/// not a second traversal that could disagree with it. Names, because a COUNT of excluded calls
/// cannot be acted on and a name can: it is the function whose return value to make deterministic.
fn nondeterministic_call_functions(
    admitted: &AdmittedPlan<'_>,
    unstable: &std::collections::BTreeSet<usize>,
) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut index = 0usize;
    for (name, _domain, tuples) in &admitted.plan.derivable {
        for _ in tuples {
            if unstable.contains(&index) && !names.iter().any(|n| n == name) {
                names.push(name.clone());
            }
            index += 1;
        }
    }
    names
}

/// What the generated-artifact population says about one repo-relative path.
///
/// Three states because the honest answers are three. `NotGenerated` is a POSITIVE answer -- this
/// path is not in the generated-artifact population at all -- and is what routes a caller to the
/// mirror-emit population. Folding it into `Refused` would tell a caller that generation FAILED
/// for an ordinary mirror, which is false and differently actionable.
enum GeneratedArtifactPathBody {
    Produced(String),
    Refused(String),
    NotGenerated,
}

/// Ask the already-resolved generated-artifact authority for the body it generates at a path.
///
/// THIS IS NOT A SECOND PRODUCER. The `.dag` side is a projection over the same three authorities
/// `main_wet` uses -- the committed-artifact roster, `artifact_path`, and the single
/// `artifact_generate` dispatch -- asked by path instead of by artifact. Reaching past it to a
/// per-artifact emitter would have been the forked dispatch DESIGN §3 forbids.
///
/// COST SHAPE, and it is why this takes a CONTEXT rather than `source_roots`. The first draft
/// resolved `generated_artifact_emit`'s whole closure inside the per-module loop, making the unit
/// of computation the corpus while the unit of fact was one path -- DESIGN §6's cost-shape defect,
/// where the rule is that a proven one is fixed regardless of the realized n. The caller resolves
/// once; each path is then one interpreter call against that context.
/// The generated-artifact authority's evaluation context, resolved AT MOST ONCE per run and
/// shared by every caller that needs it.
///
/// One cell rather than one resolve per asking site: selection asks it for a module that yields
/// no call, and the differential loop asks it for every selected module. Two resolves of one
/// closure would be two producers of the same context and would pay the corpus-sized cost twice.
fn generated_artifact_ctx<'a>(
    source_roots: &[String],
    cell: &'a mut Option<v1_compiler::v1_interpreter::InterpContext>,
) -> Result<&'a v1_compiler::v1_interpreter::InterpContext, String> {
    if cell.is_none() {
        let entry = "dag/gunbc/generated_artifact_emit.dag";
        let (graph, indices) =
            v1_compiler::cli_run::resolve_entry_graph_shared(source_roots, entry)
                .map_err(|e| format!("resolve {entry}: {e}"))?;
        *cell = Some(v1_compiler::cli_run::make_eval_context(
            &graph,
            indices,
            // HERMETIC, not Wet. The projection is pure -- it folds a roster and returns a String
            // -- so a host effect reached during it would mean a generator is doing something
            // this gate must not perform on its behalf. Hermetic refuses there instead of
            // carrying it out.
            v1_compiler::v1_interpreter::ExecutionMode::Hermetic,
        ));
    }
    Ok(cell.as_ref().expect("the context was just installed"))
}

fn generated_artifact_body_for_path(
    ctx: &v1_compiler::v1_interpreter::InterpContext,
    repo_rel_path: &str,
) -> Result<GeneratedArtifactPathBody, String> {
    use v1_compiler::v1_interpreter::Value;
    let out = v1_compiler::v1_interpreter::run_in_context_with_args(
        ctx,
        "generated_artifact_body_for_path",
        &[(
            Some("path".to_string()),
            Value::Str(repo_rel_path.to_string().into()),
        )],
        false,
    )
    .map_err(|e| format!("generated_artifact_body_for_path({repo_rel_path}): {e:?}"))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &out
    else {
        // No default arm: a shape this code does not understand is ignorance, and guessing
        // NotGenerated here would silently route a real generated artifact to the mirror emit
        // and refuse it there for the wrong reason.
        return Err(format!(
            "generated_artifact_body_for_path({repo_rel_path}) returned a non-variant value"
        ));
    };
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathNotGenerated") {
        return Ok(GeneratedArtifactPathBody::NotGenerated);
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyProduced") {
        return match ctx.field(fields, "content") {
            Some(Value::Str(c)) => Ok(GeneratedArtifactPathBody::Produced(c.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyProduced for {repo_rel_path} carried no String content"
            )),
        };
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyRefused") {
        return match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Ok(GeneratedArtifactPathBody::Refused(r.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyRefused for {repo_rel_path} carried no String reason"
            )),
        };
    }
    Err(format!(
        "generated_artifact_body_for_path({repo_rel_path}) returned an unknown variant"
    ))
}

/// Map an authority module path to the emitted mirror that declares it as its authority.
///
/// DERIVED, never authored: each generated file names its authority in its own header, written
/// by the emitter, so the mapping is a property of the artifact rather than of a roster someone
/// maintains, and it cannot be forged by editing a list.
///
/// TWO HEADER KEYS, BECAUSE THE CORPUS HAS TWO EMITTERS. The v1 compiler writes
/// `// Source module: <mod>`; `gunbc`'s own artifact emitters write `// Authority: <mod> ...`.
/// Measured over `src/v1/stage0/src`: 130 files declare themselves generated -- 126 by the first
/// key, 2 by the second (`bootstrap_stage0_crate_layout_generated.rs`,
/// `v1_interpreter_dispatch_generated.rs`), and `lib.rs`/`main.rs`, which are crate roots rather
/// than module mirrors. An earlier revision of this function read ONLY the first key, so those
/// two real mirrors of two real authorities were invisible to it: a change to either would have
/// been reported as "no emitted mirror declares it" -- FALSE, and false in the direction that
/// silently skips the check.
///
/// THAT WAS THE SAME TWO-ZEROS CONFLATION THIS MODE FIXES ONE LEVEL UP: "nothing mirrors this
/// authority" and "I could not find what mirrors it under the key I searched" are different
/// states with different remedies, and they printed the same exclusion line. So the index does
/// not merely learn the second key -- learning keys one incident at a time is how the blind spot
/// recurs. It ASSERTS ITS OWN KEY-SPACE COMPLETENESS: any file that declares itself generated
/// and yet carries neither key is a THIRD convention, which means this index has just gone blind
/// again, and the whole selection REFUSES rather than excluding authorities it cannot see.
///
/// The two crate roots are exempt by KIND, not by roster: `lib.rs` and `main.rs` are cargo's
/// crate-root names (an external, versioned authority), they aggregate the module SET rather
/// than mirroring any one authority, and no authority module path can ever select them.
struct MirrorIndex {
    by_module: std::collections::HashMap<String, String>,
}

/// The header keys, as a closed set read in one place. A key added here is a decision recorded
/// once; a key MISSING here is caught by the completeness refusal below rather than by silence.
const MIRROR_AUTHORITY_HEADER_KEYS: [&str; 2] = ["// Source module: ", "// Authority: "];

fn mirror_authority_of_header(content: &str) -> Option<String> {
    for line in content.lines().take(6) {
        let line = line.trim();
        for key in MIRROR_AUTHORITY_HEADER_KEYS {
            if let Some(rest) = line.strip_prefix(key) {
                // The `Authority:` form carries a symbol and a regen recipe after the module
                // path, separated by whitespace or `;`. Take the module path and nothing else --
                // comparing the whole remainder against a module path matches nothing, which is
                // the same silent miss one layer down.
                let module = rest
                    .trim()
                    .split([' ', ';', '\t'])
                    .next()
                    .unwrap_or_default()
                    .trim();
                if !module.is_empty() {
                    return Some(module.to_string());
                }
            }
        }
    }
    None
}

fn build_mirror_index(stage0_src: &std::path::Path) -> Result<MirrorIndex, String> {
    let entries =
        fs::read_dir(stage0_src).map_err(|e| format!("read_dir {}: {e}", stage0_src.display()))?;
    let mut by_module: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut unindexable: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let base = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        match mirror_authority_of_header(&content) {
            Some(module) => {
                by_module.insert(module, base);
            }
            None => {
                let declares_generated = content
                    .lines()
                    .next()
                    .map(|l| l.trim_start().starts_with("// Generated by "))
                    .unwrap_or(false);
                let is_crate_root = matches!(base.as_str(), "lib.rs" | "main.rs");
                if declares_generated && !is_crate_root {
                    unindexable.push(base);
                }
            }
        }
    }
    if !unindexable.is_empty() {
        unindexable.sort();
        return Err(format!(
            "the mirror index cannot see {} generated file(s) that declare themselves generated \
             but carry neither known authority header ({}): {}. A third header convention means \
             every \"no emitted mirror\" answer below is IGNORANCE rather than a fact, so the \
             selection refuses instead of excluding authorities it cannot see. Either the new \
             convention joins MIRROR_AUTHORITY_HEADER_KEYS, or the emitter writes an existing one",
            unindexable.len(),
            MIRROR_AUTHORITY_HEADER_KEYS.join("| "),
            unindexable.join(", ")
        ));
    }
    Ok(MirrorIndex { by_module })
}

/// THE POPULATION, AND WHAT DEFEATS IT -- a census, not a gate.
///
/// The differential answers ONE candidate. This answers the prior question: across every module
/// the seed actually carries, how much of each one's surface can be covered at all, and what
/// stands in the way of the rest. It runs no build and installs no candidate; it is the ranking
/// input for how far to extend a mechanism that is now proven on one module, and it deliberately
/// exits SUCCESS on any population -- a census that refused would be a gate, and nothing here
/// establishes what the right coverage is.
///
/// The population is DERIVED, never authored: every emitted mirror names its authority in its
/// own header, so the roster is a property of the artifacts. A module whose authority source is
/// missing is REPORTED, not skipped -- a census that silently drops what it cannot read reports a
/// smaller corpus as a cleaner one.
fn behavioral_receipt_census(source_roots: &[String]) -> Result<bool, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");
    let modules = collect_dag_module_sources(source_roots)?;
    let declared_anywhere = declared_type_names(&modules)?;

    let mut roster: Vec<(String, String)> = Vec::new();
    let entries =
        fs::read_dir(&stage0_src).map_err(|e| format!("read_dir {}: {e}", stage0_src.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("dir entry: {e}"))?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        // Same reader as the plan mode's index, so the census and the gate cannot disagree
        // about which files mirror an authority -- including the two that name it with the
        // second header convention.
        if let Some(declared) = mirror_authority_of_header(&content) {
            roster.push((
                declared,
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string(),
            ));
        }
    }
    roster.sort();

    let mut fns_total = 0usize;
    let mut fns_derivable = 0usize;
    let mut fns_refused = 0usize;
    let mut calls_total = 0usize;
    let mut modules_planned = 0usize;
    // NOT "no authority": the census cannot distinguish an authority that does not exist from one
    // that exists outside the roots it was given, so it reports what it actually knows and names
    // the roots. The first revision of this line said NO AUTHORITY SOURCE, and 55 of 127 modules
    // hit it -- every one of them a v1.compiler module whose .dag lives under src/v1, which is
    // simply not a scanned root. A refusal that names the wrong cause ranks the wrong work.
    let mut out_of_scope: Vec<String> = Vec::new();
    // Keyed on the TYPE that defeated derivation, because that is the unit of work: grounding one
    // type unlocks every function whose only obstacle was that type. Counting refusals instead
    // would rank the same fix once per site.
    let mut by_blocker: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut types_declared_total = 0usize;
    let mut types_read_total = 0usize;
    let mut type_reader_gaps: Vec<(String, usize, usize, Vec<String>)> = Vec::new();

    for (module_path, mirror) in &roster {
        let Some(source) = modules.get(module_path) else {
            out_of_scope.push(module_path.clone());
            continue;
        };
        let alias = format!("v1_compiler::{}", mirror.trim_end_matches(".rs"));
        let node = parse_dag_module_node(&format!("{module_path}.dag"), source)?;
        let types = visible_type_decls(module_path, source, &modules)?;
        let plan = plan_module_corpus(
            module_path,
            source,
            &node,
            &types,
            &declared_anywhere,
            &alias,
        );
        modules_planned += 1;
        fns_total += plan.parsed_signatures;
        fns_derivable += plan.derivable.len();
        fns_refused += plan.refused.len();
        let calls: usize = plan.derivable.iter().map(|(_, _, t)| t.len()).sum();
        calls_total += calls;
        for (_f, why) in &plan.refused {
            *by_blocker.entry(why.subject()).or_insert(0) += 1;
        }
        // THE SAME CROSS-CHECK THE FUNCTION READER CARRIES, for types. Two readers of one fact --
        // an authored `type ` line count and the count the parser actually produced -- so a
        // disagreement means the type reader missed a form. That is not hypothetical: Node ranked
        // 798 as "undeclared anywhere in corpus" while src/v1/00_core.dag declares it plainly, and
        // nothing in the output said the reader had skipped it.
        // The authored names, not just a count: the count says a form was missed, the NAMES say
        // which declarations, and only the second is actionable without guessing.
        let authored: Vec<String> = source
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix("type "))
            .filter_map(|rest| {
                // CUT AT `<` TOO. Without it the authored name of a generic declaration came out
                // as `Magma<T>` or, worse, `Map<key,` -- while the reader registers the bare name
                // -- so the comparison reported 11 modules as having gaps whose type_lines and
                // types_read were EQUAL. The falsifier was manufacturing its own false positives,
                // which is the one failure mode a cross-check cannot be allowed to have: it spends
                // exactly the attention it exists to direct.
                rest.split(|c: char| c.is_whitespace() || c == '{' || c == '=' || c == '<')
                    .find(|t| !t.is_empty())
                    .map(str::to_string)
            })
            .collect();
        let read = type_decls_from_module(&node);
        let type_lines = authored.len();
        let types_read = read.len();
        types_declared_total += type_lines;
        types_read_total += types_read;
        let missed: Vec<String> = authored
            .iter()
            .filter(|n| !read.contains_key(*n))
            .cloned()
            .collect();
        if !missed.is_empty() {
            type_reader_gaps.push((module_path.clone(), type_lines, types_read, missed));
        }
        eprintln!(
            "receipt-census: {module_path} parsed={} derivable={} calls={} refused={} type_lines={type_lines} types_read={types_read}",
            plan.parsed_signatures,
            plan.derivable.len(),
            calls,
            plan.refused.len()
        );
    }

    let mut ranked: Vec<(String, usize)> = by_blocker.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    eprintln!(
        "receipt-census: modules_with_mirror={} planned={} authority_outside_scanned_roots={}",
        roster.len(),
        modules_planned,
        out_of_scope.len()
    );
    // A COUNT IS NOT A SAFE REPORT HERE. Read as a small number, out-of-scope looks like a rounding
    // error; but the same shape at a wrong root set is a hole centred on whatever nobody scanned,
    // and this arm has already been that hole once -- it swallowed 55 of 127 as "no authority"
    // because src/v1 was not a root. So the fraction is stated against the population, and any
    // out-of-scope module at all makes the run say what its coverage actually is rather than
    // leading with the modules it managed to plan.
    if !out_of_scope.is_empty() {
        eprintln!(
            "receipt-census: COVERAGE {}/{} modules planned; {} of the corpus was NOT measured \
             because no scanned root holds its authority. Roots given: {}. A module below is not \
             evidence that its authority is missing -- it is evidence that this run could not see \
             it, and if this fraction is large the root set is the finding, not the corpus",
            modules_planned,
            roster.len(),
            out_of_scope.len(),
            source_roots.join(", ")
        );
    }
    for m in &out_of_scope {
        eprintln!("receipt-census: OUT OF SCOPE {m} — counted, not skipped");
    }
    eprintln!(
        "receipt-census: functions parsed={fns_total} derivable={fns_derivable} refused={fns_refused} calls={calls_total}"
    );
    eprintln!(
        "receipt-census: TYPE READER type_lines={types_declared_total} types_read={types_read_total} modules_with_a_gap={}",
        type_reader_gaps.len()
    );
    if !type_reader_gaps.is_empty() {
        eprintln!(
            "receipt-census: a gap means the reader did not produce a declaration the source \
             authors, so every refusal naming those types is measuring THIS READER, not the corpus"
        );
        let mut worst = type_reader_gaps.clone();
        worst.sort_by(|a, b| b.3.len().cmp(&a.3.len()));
        for (m, lines, read, missed) in worst.iter() {
            eprintln!(
                "receipt-census:   GAP {m} type_lines={lines} types_read={read} missed={}",
                missed.join(", ")
            );
        }
        // WHAT SHAPE the missed declarations actually have, for one module, printed rather than
        // assumed. Three times now a node-shape assumption has been wrong and each time the cost
        // was a whole measurement built on it; the shape is cheap to report and there is no reason
        // for the next reader of this output to have to re-derive it.
        // WHAT DISTINGUISHES A TYPE DECLARATION FROM A FUNCTION OR A DATA ROW, printed for one
        // module rather than assumed. The reader must filter on a property of the node; every
        // previous attempt to name one from memory has been wrong.
        if let Some(src) = modules.get("std.pareto") {
            if let Ok(node) = parse_dag_module_node("std.pareto.dag", src) {
                let authored: std::collections::HashSet<String> = src
                    .lines()
                    .filter_map(|l| l.trim_start().strip_prefix("type "))
                    .filter_map(|r| {
                        r.split(|c: char| c.is_whitespace() || c == '{' || c == '=' || c == '<')
                            .find(|t| !t.is_empty())
                            .map(str::to_string)
                    })
                    .collect();
                let mut shapes: std::collections::BTreeMap<String, (usize, Vec<String>)> =
                    std::collections::BTreeMap::new();
                for c in node.children.iter() {
                    let key = format!(
                        "is_type={} conn={:?} body={} params={} inferred={} children={}",
                        authored.contains(&c.name),
                        c.connective,
                        c.body.is_some(),
                        c.params.len(),
                        c.inferred.is_some(),
                        c.children.len()
                    );
                    let e = shapes.entry(key).or_insert((0, Vec::new()));
                    e.0 += 1;
                    if e.1.len() < 3 {
                        e.1.push(c.name.clone());
                    }
                }
                for (k, (n, ex)) in &shapes {
                    eprintln!(
                        "receipt-census:   CHILDSHAPE {n:4}  {k}  e.g. {}",
                        ex.join(", ")
                    );
                }
            }
        }
        if let Some((m, _, _, missed)) = worst.iter().find(|(m, _, _, _)| m == "std.pareto") {
            if let Some(src) = modules.get(m) {
                if let Ok(node) = parse_dag_module_node(&format!("{m}.dag"), src) {
                    for name in missed {
                        match node.children.iter().find(|c| &c.name == name) {
                            Some(c) => {
                                let f = c.children.iter().next();
                                eprintln!(
                                    "receipt-census:   SHAPE {m}::{name} connective={:?} \
                                     children={} | field name={:?} children={} conn={:?} \
                                     type_annotation={:?} inferred={}",
                                    c.connective,
                                    c.children.len(),
                                    f.map(|f| f.name.clone()),
                                    f.map(|f| f.children.len()).unwrap_or(0),
                                    f.map(|f| f.connective.clone()),
                                    f.and_then(|f| f.type_annotation.as_ref())
                                        .map(|t| t.name.clone()),
                                    f.map(|f| f.inferred.is_some()).unwrap_or(false)
                                )
                            }
                            None => eprintln!(
                                "receipt-census:   SHAPE {m}::{name} — NO module child carries \
                                 this name; the declaration is not where the reader looks"
                            ),
                        }
                    }
                }
            }
        }
    }
    eprintln!("receipt-census: refusals ranked by the type responsible");
    for (why, n) in ranked.iter().take(40) {
        eprintln!("receipt-census:   {n:5}  {why}");
    }
    if ranked.len() > 40 {
        let tail: usize = ranked.iter().skip(40).map(|(_, n)| n).sum();
        eprintln!(
            "receipt-census:   {tail:5}  [{} further distinct causes, not shown]",
            ranked.len() - 40
        );
    }
    Ok(true)
}

fn run_behavioral_receipt_census(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_census(source_roots) {
        Ok(_) => Ok(ExitCode::SUCCESS),
        Err(e) => {
            eprintln!("receipt-census: REFUSED — {e}");
            Err(ExitCode::from(1))
        }
    }
}

fn behavioral_receipt_selftest(source_roots: &[String]) -> Result<bool, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    // NOT under src/v1: regen seeds every .dag there into the stage0 compile closure, and this
    // authority must never be emitted. Measured, not assumed -- placing it there made the emit
    // produce receipt_fixture.rs with no committed mirror, and required-regen refused the whole
    // surface as a population mismatch.
    let fixture = workspace.join("fixtures/receipt_fixture");
    let module_path = "receipt.fixture";
    let alias = "v1_receipt_fixture";

    let authority = fixture.join("authority.dag");
    let source =
        fs::read_to_string(&authority).map_err(|e| format!("read {}: {e}", authority.display()))?;
    let modules = collect_dag_module_sources(source_roots)?;
    let node = parse_dag_module_node(&format!("{module_path}.dag"), &source)?;
    let types = visible_type_decls(module_path, &source, &modules)?;
    let declared_anywhere = declared_type_names(&modules)?;
    let plan = plan_module_corpus(
        module_path,
        &source,
        &node,
        &types,
        &declared_anywhere,
        alias,
    );

    eprintln!(
        "receipt-selftest: fixture parsed={} derivable={} refused={}",
        plan.parsed_signatures,
        plan.derivable.len(),
        plan.refused.len()
    );
    for (f, d, tuples) in &plan.derivable {
        eprintln!(
            "receipt-selftest:   derivable {f} {} calls={}",
            d.report(),
            tuples.len()
        );
    }
    for (f, why) in &plan.refused {
        eprintln!("receipt-selftest:   REFUSED {f} — {}", why.describe());
    }

    // THE PRECONDITION THE ARMS DEPEND ON, checked before the arms rather than assumed by them.
    //
    // Arm 2 changes `band_of` at exactly one input in all of i64: level = 100. If the boundary
    // enumeration ever regresses to sampling, or to a window that excludes 100, arm 2 goes GREEN
    // and this whole control dies without a sound. So the tuple set is required to CONTAIN that
    // input, and the requirement is stated over the enumerated corpus -- the thing that will
    // actually run -- not over the domain description of it.
    let band_of = plan
        .derivable
        .iter()
        .find(|(f, _, _)| f == "band_of")
        .ok_or_else(|| {
            format!(
                "the fixture's band_of did not derive, so arm 2 could not discriminate even if it \
                 ran. Refusing rather than reporting a control that cannot fail. Refusals: {}",
                plan.refused
                    .iter()
                    .map(|(f, w)| format!("{f}: {}", w.describe()))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })?;
    if !matches!(
        band_of.1,
        EnumeratedDomain::ExhaustiveOverDerivedPartition { .. }
    ) {
        return Err(format!(
            "band_of derived as {}, not over a derived partition. The Int partition is the thing \
             arm 2 exercises; covering it some other way would leave that arm untested",
            band_of.1.report()
        ));
    }
    if !band_of.2.iter().any(|t| t == &vec!["100i64".to_string()]) {
        return Err(format!(
            "the enumerated corpus for band_of does not contain the boundary input 100i64, so arm \
             2's single behavioural difference is outside what the receipt would run. Enumerated: \
             {:?}",
            band_of.2
        ));
    }

    let seed_path = fixture.join("src/lib.rs");
    let mut ok = true;
    let krate = ReceiptCrate::receipt_fixture();
    // The fixture must ADMIT, and it says so here rather than deep inside an arm: a fixture that
    // stopped yielding calls would otherwise turn both arms into an exclusion and this control
    // into a control that cannot fail.
    let admitted = AdmittedPlan::of(&plan).map_err(|c| {
        format!(
            "the fixture yields no call at all, so neither arm could discriminate. Uncovered: {}",
            c.uncovered
                .iter()
                .map(|(f, w)| format!("{f}: {}", w.describe()))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;

    for (arm, candidate_file, expect_equivalent) in [
        ("preserving", "behaviour_preserving.rs", true),
        ("changing", "behaviour_changing.rs", false),
    ] {
        let cand_path = fixture.join("candidates").join(candidate_file);
        let candidate = fs::read_to_string(&cand_path)
            .map_err(|e| format!("read {}: {e}", cand_path.display()))?;
        // An arm whose candidate bytes equal the seed's proves nothing in EITHER direction: the
        // preserving arm would report equivalence for the trivial reason, and the changing arm
        // would report equivalence and be read as a regression. Checked, not trusted.
        let seed = fs::read_to_string(&seed_path)
            .map_err(|e| format!("read {}: {e}", seed_path.display()))?;
        if seed == candidate {
            return Err(format!(
                "arm {arm}: {candidate_file} is byte-identical to the fixture seed, so this arm \
                 compares a file with itself"
            ));
        }

        let verdict =
            behavioral_differential(&workspace, &krate, &seed_path, &candidate, &admitted, alias);
        match (&verdict, expect_equivalent) {
            (
                ReceiptVerdict::Equivalent {
                    calls,
                    nondeterministic_calls,
                    ..
                },
                true,
            ) => {
                // FALSE-POSITIVE CONTROL for the two-run instability probe, and it is the reason
                // this arm now reads a second field. The fixture's corpus is deterministic, so
                // the probe must find NOTHING unstable in it. Without this, a probe that marked
                // every line unstable would still print EQUIVALENT here -- over an empty compared
                // set it would not even reach this arm, but over a partially-marked one it would,
                // and the arm would pass while the gate had quietly stopped comparing anything.
                if *nondeterministic_calls != 0 {
                    eprintln!(
                        "receipt-selftest: arm {arm} EQUIVALENT but the instability probe marked {nondeterministic_calls} call(s) unstable in a DETERMINISTIC fixture — the probe is producing false positives, so its exclusions cannot be trusted"
                    );
                    ok = false;
                } else {
                    eprintln!("receipt-selftest: arm {arm} EQUIVALENT over {calls} derived calls — as required");
                }
            }
            (
                ReceiptVerdict::Divergent {
                    calls,
                    first_difference,
                },
                false,
            ) => {
                // Not merely THAT it diverged: divergence at the wrong call would mean the arm is
                // catching something other than the difference it was authored to catch, and a
                // control that passes for the wrong reason is not a control.
                if !(first_difference.contains("band_of") && first_difference.contains("100i64")) {
                    eprintln!(
                        "receipt-selftest: arm {arm} DIVERGENT over {calls} calls but at the WRONG \
                         call — expected band_of(100i64): {first_difference}"
                    );
                    ok = false;
                } else {
                    eprintln!(
                        "receipt-selftest: arm {arm} DIVERGENT over {calls} derived calls at the \
                         authored difference — {first_difference}"
                    );
                }
            }
            (v, _) => {
                eprintln!(
                    "receipt-selftest: arm {arm} expected {} but got {v:?}",
                    if expect_equivalent {
                        "EQUIVALENT"
                    } else {
                        "DIVERGENT"
                    }
                );
                ok = false;
            }
        }
    }
    Ok(ok)
}

fn run_behavioral_receipt_selftest(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_selftest(source_roots) {
        Ok(true) => Ok(ExitCode::SUCCESS),
        Ok(false) => {
            eprintln!(
                "receipt-selftest: REFUSED — the behavioral receipt's own arms no longer \
                 discriminate. Until this is green, no verdict the mode reports is evidence"
            );
            Err(ExitCode::from(1))
        }
        Err(e) => {
            eprintln!("receipt-selftest: REFUSED — {e}");
            Err(ExitCode::from(1))
        }
    }
}

/// STANDALONE INVOCATION, where an absent subject is a MISUSE rather than a state to report.
///
/// Naming this mode on the command line asserts there is a pull request to check. If the merge
/// base is the head there is not, so the invocation is answered rather than silently succeeding.
/// This mode has no other caller: the 2026-08-21 operator ruling cut the receipt out of
/// `--required-ci`, so nothing invokes it automatically and its only route is someone typing it.
///
/// A DIFF-SUBJECT GATE CANNOT BE EXERCISED BY THE BRANCH IT PROTECTS, and this paragraph is
/// carried here because the property is a fact about THIS gate's subject rather than about the
/// phase that used to run it. It was learned expensively and it survives its enrolment.
///
/// The subject is a diff against the merge base. On main the merge base IS the head, so there is
/// no pull request to check and never was -- which means NO main run, green or otherwise, is
/// evidence about this gate: comparing a PR's red against main's green compares a run against a
/// SKIP, not against a pass. Coverage is entirely PR-side by construction. The consequence that
/// makes it worth writing down: a defect that reds every PR touching one class of authority can
/// sit indefinitely while main stays green, because the only runs that could have seen it are
/// the ones a person reads as "my branch is broken". That is not hypothetical -- it is how the
/// wet-actuator selection defect (gunbc#8704, excluded at selection since) survived.
///
/// SO IT BINDS ANY FUTURE PROPOSAL, not just this one. If a diff-subject gate is ever enrolled
/// in CI again -- this receipt or another -- read this first: it needs a real subject on main
/// (the PUSH RANGE is one, and is a DIFFERENT subject rather than a stand-in for a PR diff), or
/// it is accepting PR-only coverage knowingly. What must NOT move is the absent-subject arm:
/// making it return an answer gives it a deficit frequency of zero by construction, which is the
/// absorbing fallback wearing the fix's clothes (DESIGN section 5).
fn run_behavioral_receipt_plan(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_plan(source_roots) {
        Ok(ReceiptPlanOutcome::Ran { agreed: true }) => Ok(ExitCode::SUCCESS),
        Ok(ReceiptPlanOutcome::Ran { agreed: false }) => Err(ExitCode::from(1)),
        Ok(ReceiptPlanOutcome::NoSubject { head }) => {
            eprintln!(
                "behavioral-receipt: NO SUBJECT — the merge base resolves to HEAD ({head}), so \
                 the diff compares this commit against itself and cannot observe what changed. \
                 This is not an empty selection and is not reported as a pass: `nothing changed` \
                 and `I could not see what changed` are different states. This mode's subject is \
                 a pull request against main; invoke it there"
            );
            Err(ExitCode::from(1))
        }
        Err(refusal) => {
            eprintln!("behavioral-receipt: refused: {refusal}");
            Err(ExitCode::from(1))
        }
    }
}

/// What the per-PR receipt run found, as a state rather than a bool.
///
/// `NoSubject` exists because "this run has nothing to check" and "this run checked and agreed"
/// are the two zeros this mode was corrected for once already, one level down. Collapsing them
/// into `Ok(true)` is precisely how the vacuous pass on `push: main` was written.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptPlanOutcome {
    /// The merge base resolves to HEAD, so no diff exists to read. Not a pass, not a failure --
    /// an absent subject.
    NoSubject { head: String },
    /// A selection was computed and every selected module reached a verdict.
    Ran { agreed: bool },
}

fn behavioral_receipt_plan(source_roots: &[String]) -> Result<ReceiptPlanOutcome, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");

    // BASELINE FIRST, asserted and printed, exactly as the mirror-drift gate does and for the
    // same reason: a selection computed against an unresolvable baseline is ignorance, and the
    // tempting fallback -- treat everything as changed -- is the absorbing arm that turns a
    // per-change gate into a per-corpus one.
    let head = git_stdout(&workspace, &["rev-parse", "HEAD"])?;
    let base = git_stdout(&workspace, &["merge-base", "origin/main", "HEAD"]).map_err(|e| {
        format!(
            "cannot resolve the merge base against origin/main ({e}). The selection is NOT \
             widened to the whole population in this case: `I could not determine what changed` \
             and `everything changed` are different states, and two compiler builds per module \
             across the corpus is a budget breach denominated in the repository rather than in \
             the change. Fetch the base first: \
             `git fetch --depth=200 origin main:refs/remotes/origin/main`"
        )
    })?;
    eprintln!("behavioral-receipt: merge_base={base} head={head}");

    // A BASELINE THAT IS THE HEAD IS NOT AN EMPTY SELECTION -- IT IS NO OBSERVATION AT ALL.
    //
    // On a push to main, `git merge-base origin/main HEAD` resolves to HEAD itself, so the diff
    // compares the commit against itself and yields zero changed authorities. The empty-selection
    // arm below would then report a real pass over a corpus that was never looked at. That is the
    // empty-observation narrow DESIGN names by its live specimen -- a push whose baseline ref IS
    // the pushed ref -- and it is the mirror of the absorbing fallback: a widen is merely
    // expensive, a narrow is silently uncovered.
    //
    // `nothing changed` and `I could not see what changed` are different states with different
    // remedies, so they get different answers. This one refuses, and it names the invocation that
    // makes sense instead of guessing at a substitute baseline: the subject of this gate is a
    // pull request, and a push to a branch that IS the baseline has no such subject.
    if base == head {
        return Ok(ReceiptPlanOutcome::NoSubject { head });
    }

    let changed = git_stdout(
        &workspace,
        &["diff", "--name-only", &base, &head, "--", "*.dag"],
    )?;
    let changed: Vec<String> = changed
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    // Loaded once for the whole run: the type a module compiles against may be declared in any
    // module it transitively imports.
    let modules = collect_dag_module_sources(source_roots)?;
    let declared_anywhere = declared_type_names(&modules)?;

    // Built ONCE, before any authority is classified, and it refuses if it cannot see the whole
    // generated population -- so an exclusion below is a fact about the corpus rather than a fact
    // about what this reader happened to recognise.
    let mirror_index = build_mirror_index(&stage0_src)?;

    let mut exclusions: Vec<ReceiptExclusion> = Vec::new();
    let mut plans: Vec<(String, ModuleCorpusPlan, String)> = Vec::new();
    // BUILT ON FIRST DEMAND, not before the loop. Only a module that yields no call needs to ask
    // the generated-artifact population during selection, which is a small population and often
    // an empty one; resolving its closure unconditionally would put a corpus-sized resolve on
    // every run of a per-change gate -- the cost shape DESIGN §6 names, where the unit of
    // computation is the corpus and the unit of fact is one path.
    let mut generated_ctx_cell: Option<v1_compiler::v1_interpreter::InterpContext> = None;
    // BOTH GRAINS, ACCUMULATED ACROSS EVERY CHANGED AUTHORITY -- including the excluded ones,
    // which is the point: a function that yields no call is uncovered whether or not its module
    // had a sibling that saved it from exclusion.
    let mut declared_functions = 0usize;
    let mut covered_functions = 0usize;

    for rel in &changed {
        let abs = workspace.join(rel);
        let Ok(source) = fs::read_to_string(&abs) else {
            continue;
        };
        let Some(module_path) = v1_compiler::cli_run::extract_module_path_public(&source) else {
            continue;
        };
        match mirror_index.by_module.get(&module_path).cloned() {
            None => exclusions.push(ReceiptExclusion::NoEmittedMirror { module_path }),
            Some(mirror) => {
                // The Rust module path is the mirror's basename without its extension — derived
                // from the artifact, like the mapping that found it, never spelled out here.
                // The full crate path to the emitted mirror, derived from the artifact's own
                // basename. Both the enumerated constructor values and the generated calls are
                // written against THIS string, so there is one spelling of the module under test.
                let alias = format!("v1_compiler::{}", mirror.trim_end_matches(".rs"));
                let node = parse_dag_module_node(&format!("{module_path}.dag"), &source)?;
                let types = visible_type_decls(&module_path, &source, &modules)?;
                let plan = plan_module_corpus(
                    &module_path,
                    &source,
                    &node,
                    &types,
                    &declared_anywhere,
                    &alias,
                );
                // ADMISSION IS DECIDED HERE, AT FUNCTION GRAIN, and it is decided from a fact the
                // source already carries -- no build, no emit, no differential is spent on a
                // module that cannot produce a call. The decision itself is a PURE function, so
                // both of its exclusion arms and its refusal can be executed against hand-built
                // states rather than only against whatever the live corpus happens to hold.
                let coverage = function_grain_coverage(&plan);
                declared_functions += coverage.covered.len() + coverage.uncovered.len();
                covered_functions += coverage.covered.len();
                match plan_grain_selection(&plan, coverage)? {
                    None => plans.push((mirror, plan, alias)),
                    Some(exclusion) => {
                        // A ZERO-CALL MODULE IS NOT AUTOMATICALLY A SUBJECTLESS ONE, and getting
                        // this wrong is how an exclusion becomes the thing it was meant to
                        // prevent. #8753 established the fact by measurement: for a mirror in the
                        // GENERATED-ARTIFACT population the subject is the artifact's BYTES, not
                        // its behaviour, and that population's only drift observer anywhere is the
                        // identity check inside the differential loop. Excluding here on `no
                        // function yields a call` would delete that observer for exactly the
                        // artifacts that have no functions to call -- silently, and while printing
                        // a line that says the module has nothing to compare.
                        //
                        // So the population is ASKED, and it is asked with the same projection
                        // the loop asks (`generated_artifact_body_for_path`), not with a second
                        // roster. `NotGenerated` is a positive answer -- this path is a module
                        // mirror -- and only then is the exclusion a fact.
                        let repo_rel = format!("src/v1/stage0/src/{mirror}");
                        let ctx = generated_artifact_ctx(source_roots, &mut generated_ctx_cell)?;
                        let body = generated_artifact_body_for_path(ctx, &repo_rel)?;
                        match exclusion_survives_generated_artifact_population(exclusion, &body) {
                            Some(exclusion) => exclusions.push(exclusion),
                            None => plans.push((mirror, plan, alias)),
                        }
                    }
                }
            }
        }
    }

    // BOTH GRAINS, SIDE BY SIDE, ON EVERY RUN -- even when they agree (operator directive,
    // 2026-08-21). A module count alone reads as coverage ("3 modules, one pass"); a function
    // count alone reads as what happened but hides how much of the corpus was in scope at all.
    // The gap between the two sentences is the thing worth seeing, and a reader given one number
    // infers the other one wrongly.
    //
    // EXCLUSIONS ARE COUNTED PER ARM, not as one total. A total says how many changed authorities
    // reached no verdict; only the split says WHY, and the arms rank differently: a
    // non-derivability row is work someone can do, an outside-the-population row is a fact no
    // work removes. Collapsing them is how a population that cannot shrink gets read as debt that
    // simply has not been paid.
    let mut excluded_no_mirror = 0usize;
    let mut excluded_no_function_declared = 0usize;
    let mut excluded_no_corpus = 0usize;
    for e in &exclusions {
        match e {
            ReceiptExclusion::NoEmittedMirror { .. } => excluded_no_mirror += 1,
            ReceiptExclusion::NoFunctionDeclared { .. } => excluded_no_function_declared += 1,
            ReceiptExclusion::NoFunctionHasACorpus { .. } => excluded_no_corpus += 1,
        }
    }
    eprintln!(
        "behavioral-receipt: GRAIN module: changed_authorities={} selected={} excluded={} \
         (no-emitted-mirror={excluded_no_mirror} \
         no-function-declared={excluded_no_function_declared} \
         no-function-has-a-corpus={excluded_no_corpus}) | \
         function: declared={declared_functions} covered={covered_functions} uncovered={}",
        changed.len(),
        plans.len(),
        exclusions.len(),
        declared_functions - covered_functions
    );
    for e in &exclusions {
        match e {
            ReceiptExclusion::NoEmittedMirror { module_path } => eprintln!(
                "behavioral-receipt: excluded {module_path} — no emitted mirror in the \
                 generated population names it as its authority, under either header \
                 convention (the index refuses outright if any generated file is \
                 unindexable, so this is a fact about the corpus and not a lookup miss)"
            ),
            ReceiptExclusion::NoFunctionDeclared { module_path } => eprintln!(
                "behavioral-receipt: excluded {module_path} — the authority declares no \
                 functions, so it carries no behaviour that could diverge and there is nothing \
                 for a differential to compare. NOT a derivation deficit: it contributes no \
                 uncovered function to the FUNCTION-GRAIN line, because there is no function to \
                 cover"
            ),
            ReceiptExclusion::NoFunctionHasACorpus {
                module_path,
                uncovered,
            } => {
                eprintln!(
                    "behavioral-receipt: excluded {module_path} — none of its {} declared \
                     functions yields a call, so there is no corpus to compare. NOT a failure of \
                     this diff: the same non-derivability in a module with one derivable sibling \
                     runs and passes, so refusing here would be a verdict decided by where a \
                     file-level count landed. Every uncovered function is named below and counted \
                     in the FUNCTION-GRAIN line",
                    uncovered.len()
                );
                for (f, why) in uncovered {
                    eprintln!(
                        "behavioral-receipt:   uncovered {module_path}::{f} — {}",
                        why.describe()
                    );
                }
            }
        }
    }
    for (_mirror, p, _alias) in &plans {
        // Both counts are coverage claims, so they are reported as the two ways coverage was
        // ESTABLISHED -- closed type versus derived partition -- and not as strong-versus-weak.
        // There is no third number here any more; the bounded column it replaced counted
        // functions that had been sampled, not covered.
        let closed = p
            .derivable
            .iter()
            .filter(|(_, d, _)| matches!(d, EnumeratedDomain::Exhaustive { .. }))
            .count();
        eprintln!(
            "behavioral-receipt: {} fn_lines={} parsed={} derivable={} (closed-type={} derived-partition={}) refused={}",
            p.module_path,
            p.declared_fn_lines,
            p.parsed_signatures,
            p.derivable.len(),
            closed,
            p.derivable.len() - closed,
            p.refused.len()
        );
        for (f, d, _tuples) in &p.derivable {
            eprintln!(
                "behavioral-receipt:   derivable {}::{f} {}",
                p.module_path,
                d.report()
            );
        }
        for (f, why) in &p.refused {
            eprintln!(
                "behavioral-receipt:   REFUSED {}::{f} — corpus not derivable: {}",
                p.module_path,
                why.describe()
            );
        }
    }

    // THE DIFFERENTIAL. Everything above decides WHAT to run; this runs it.
    //
    // The emit happens once for the whole selection rather than once per module: it is the
    // expensive step, and asking for it per candidate would make a two-module change cost twice
    // what a one-module change costs for no additional information.
    if plans.is_empty() {
        eprintln!(
            "behavioral-receipt: no changed authority module reached the differential — every one \
             was excluded above, each under one of the three typed arms counted on the \
             module-grain line. Nothing to compare, so this run costs nothing. That \
             is a real pass over an EMPTY selection, stated rather than printed as a bare PASS, \
             and the FUNCTION-GRAIN counts above say how much surface that silence covers"
        );
        return Ok(ReceiptPlanOutcome::Ran { agreed: true });
    }

    // A DECLARED CAP, REFUSED ABOVE RATHER THAN SAMPLED (operator ruling, 2026-08-20).
    //
    // The differential costs one crate build per selected module plus one for the seed. The
    // tempting arm when a PR touches many authorities is to check the first few and report a pass
    // -- the absorbing fallback exactly: the deficit's frequency goes to zero by construction and
    // nobody learns the gate stopped covering things. So an over-cap selection REFUSES, typed and
    // counted, and an over-cap PR is visible rather than quietly under-checked.
    //
    // WHAT KIND OF NUMBER THIS IS, because a bare literal in a merge-blocking check is exactly
    // what DESIGN §5 tells reviewers to distrust (review 54096 asked, correctly). It is a POLICY
    // BUDGET -- one of the four sanctioned grounds -- and the resource it caps is CI wall clock:
    // each selected module costs a full v1-compiler release build, so four modules is roughly
    // forty minutes on a job that already runs thirty. It is NOT a measurement copied from the
    // tree, and automating its update would not collapse this check to `measure() == measure()`.
    //
    // DISSOLUTION, so it is a policy rather than a bounded scaffold wearing one's clothes: this
    // cap exists only because the differential rebuilds the whole crate per candidate. It is
    // RAISED, not removed, when that stops being true -- the seed transcript is captured once for
    // a whole selection today, but each candidate install still forces a full rebuild, and a
    // per-module compilation unit would make the cost linear in the changed surface rather than
    // in the crate. Until then a four-authority PR is genuinely more than this gate can check,
    // and saying so is the honest answer rather than checking three of four and printing a pass.
    // A PR that legitimately needs more splits, or the operator raises the number here.
    const MAX_SELECTED_MODULES_PER_RUN: usize = 3;
    if plans.len() > MAX_SELECTED_MODULES_PER_RUN {
        eprintln!(
            "behavioral-receipt: REFUSED — {} authority modules selected, above the declared cap \
             of {MAX_SELECTED_MODULES_PER_RUN}. Not sampling the first {MAX_SELECTED_MODULES_PER_RUN}: \
             a partial check reported as a pass is how a gate stops covering things without anyone \
             finding out. Selected: {}",
            plans.len(),
            plans
                .iter()
                .map(|(_, p, _)| p.module_path.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return Ok(ReceiptPlanOutcome::Ran { agreed: false });
    }

    let emitted = v1_compiler::cli_run::emitted_generated_sources()?;
    let mut all_equivalent = true;
    // Accumulated as each module is admitted and printed after every verdict, so the denominator
    // is the SAME partition the admission was decided from. Recomputing it from the plan
    // afterwards would be a second producer of one fact, and the one that gets reported is the
    // one that never gated anything.
    // ONE RESOLVE for the whole run -- see `generated_artifact_body_for_path`'s cost-shape note.
    // The cell may already hold it: selection asks the same authority when a module yields no
    // call, and asking it twice would resolve one closure twice.
    let generated_ctx = generated_artifact_ctx(source_roots, &mut generated_ctx_cell)?;
    let mut denominators: Vec<String> = Vec::new();
    let mut nondeterministic_calls_total = 0usize;
    let mut nondeterministic_modules = 0usize;
    for (mirror, plan, alias) in &plans {
        // WHICH POPULATION OWNS THIS MIRROR, decided before anything is fetched.
        //
        // Two generators write into `src/v1/stage0/src`: the v1 compiler emits module mirrors,
        // and gunbc's artifact emitters write a handful of generated files whose headers say so.
        // This fragment used to ask only the first and refuse when it came back empty, which made
        // the SECOND population permanently unaskable -- any change to the interpreter dispatch
        // roster or the stage0 crate layout redded this run with "a missing candidate is
        // ignorance", correct as written and with no reachable green.
        //
        // The population is asked FIRST and answers positively. This is deliberately not a
        // fallback from the mirror-emit miss: a fallback would make absence in one producer mean
        // presence in the other, so a genuinely unknown path would silently be regenerated from
        // nothing instead of refused. Here each population answers for what it owns, and a path
        // in neither still refuses.
        let repo_rel = format!("src/v1/stage0/src/{mirror}");
        let generated = match generated_artifact_body_for_path(&generated_ctx, &repo_rel) {
            Ok(g) => g,
            Err(e) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — could not ask the generated-artifact \
                     population about {repo_rel}: {e}. Not equivalence: an unanswered question \
                     is ignorance",
                    plan.module_path
                );
                all_equivalent = false;
                continue;
            }
        };
        let owned_candidate: Option<String> = match generated {
            GeneratedArtifactPathBody::Produced(content) => Some(content),
            GeneratedArtifactPathBody::Refused(reason) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — {repo_rel} is a generated artifact and its \
                     generator refused: {reason}",
                    plan.module_path
                );
                all_equivalent = false;
                continue;
            }
            GeneratedArtifactPathBody::NotGenerated => None,
        };
        // A GENERATED ARTIFACT IS CHECKED BY IDENTITY, NOT BY CALLING IT -- and this is the
        // correct check for it, not a weaker stand-in for the differential.
        //
        // MEASURED, after the first draft of this change got it wrong. Producing the candidate
        // was necessary and not sufficient: with the candidate in hand the differential went one
        // step further and refused again, because it compiles a driver that CALLS the authority's
        // declared functions against the mirror --
        //
        //   the driver did not compile against the mirror: error[E0425]: cannot find function
        //   `v1_interpreter_arm_shape_derivability` in module v1_compiler::v1_interpreter_dispatch_generated
        //
        // -- and `v1_interpreter_dispatch_generated.rs` exposes enums and `lookup_*` fns. It is a
        // file DERIVED FROM the authority's data, not a Rust projection of the authority's
        // functions, so those functions are not there and never will be. The differential's
        // precondition (the mirror answers the same calls the authority declares) simply does not
        // hold for this population.
        //
        // For a generated artifact the whole content IS the product, so byte identity between a
        // freshly generated candidate and the committed file is the complete statement of
        // correctness -- which is exactly the drift check that has had no owner since the
        // generated-artifact drift gates were dropped in the floor cut.
        let candidate_source: &String = match owned_candidate.as_ref() {
            Some(candidate) => {
                let committed_path = workspace.join(&repo_rel);
                match fs::read_to_string(&committed_path) {
                    Err(e) => {
                        eprintln!(
                            "behavioral-receipt: {} REFUSED — {repo_rel} is a generated artifact \
                             but its committed bytes could not be read ({e}), so identity cannot \
                             be established",
                            plan.module_path
                        );
                        all_equivalent = false;
                    }
                    Ok(committed_bytes) if committed_bytes == *candidate => {
                        eprintln!(
                            "behavioral-receipt: {} ARTIFACT-IDENTICAL — {repo_rel} regenerates \
                             byte-for-byte from its authority. This is identity, not behavioural \
                             equivalence: the artifact exposes no function this fragment could \
                             call, so its bytes are the whole claim",
                            plan.module_path
                        );
                    }
                    Ok(_) => {
                        eprintln!(
                            "behavioral-receipt: {} ARTIFACT-DRIFT — {repo_rel} does not match \
                             what its authority generates. Regenerate it (main_wet on \
                             dag/tools/generated_artifact_gate.dag) and commit the result",
                            plan.module_path
                        );
                        all_equivalent = false;
                    }
                }
                continue;
            }
            None => match emitted.get(mirror) {
                Some(c) => c,
                None => {
                    eprintln!(
                        "behavioral-receipt: {} REFUSED — {mirror} is in neither population: the \
                         v1 emit produced no mirror for it and it is not a committed generated \
                         artifact. Not equivalence: a missing candidate is ignorance",
                        plan.module_path
                    );
                    all_equivalent = false;
                    continue;
                }
            },
        };
        // Infallible in fact -- selection only pushed plans that admitted -- but derived here
        // rather than asserted, so the differential's precondition is carried by the value it
        // receives instead of by a comment about an earlier loop.
        let admitted = match AdmittedPlan::of(plan) {
            Ok(a) => a,
            Err(coverage) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — selected but yields no call ({} uncovered \
                     functions). Selection and admission disagree, which is a defect in this \
                     fragment, not in the authority",
                    plan.module_path,
                    coverage.uncovered.len()
                );
                all_equivalent = false;
                continue;
            }
        };
        denominators.push(format!(
            "behavioral-receipt: DENOMINATOR {} — {} derived calls over {} of {} declared \
             functions; the other {} yield no call and are NOT covered by this verdict",
            plan.module_path,
            admitted.coverage.calls(),
            admitted.coverage.covered.len(),
            plan.parsed_signatures,
            admitted.coverage.uncovered.len()
        ));
        match behavioral_differential(
            &workspace,
            &ReceiptCrate::v1_compiler(),
            &workspace.join("src/v1/stage0/src").join(mirror),
            candidate_source,
            &admitted,
            alias,
        ) {
            ReceiptVerdict::Equivalent {
                calls,
                nondeterministic_calls,
                nondeterministic_functions,
            } => {
                if nondeterministic_calls == 0 {
                    eprintln!(
                        "behavioral-receipt: {} EQUIVALENT over {calls} derived calls",
                        plan.module_path
                    );
                } else {
                    eprintln!(
                        "behavioral-receipt: {} EQUIVALENT over {calls} derived calls, with \
                         {nondeterministic_calls} EXCLUDED as nondeterministically rendered — \
                         {}. Those calls were NOT compared, so this green does not cover them",
                        plan.module_path,
                        nondeterministic_functions.join(", ")
                    );
                    nondeterministic_calls_total += nondeterministic_calls;
                    nondeterministic_modules += 1;
                }
            }
            ReceiptVerdict::NondeterministicRendering {
                unstable_calls,
                functions,
            } => {
                eprintln!(
                    "behavioral-receipt: {} NONDETERMINISTIC-RENDERING — all {unstable_calls} \
                     derived call(s) render unstably, so nothing could be compared: {}. This is \
                     a property of what the mirror RETURNS, not a defect in this diff, and it is \
                     not scored as a divergence",
                    plan.module_path,
                    functions.join(", ")
                );
                nondeterministic_calls_total += unstable_calls;
                nondeterministic_modules += 1;
            }
            ReceiptVerdict::Divergent {
                calls,
                first_difference,
            } => {
                eprintln!(
                    "behavioral-receipt: {} DIVERGENT over {calls} derived calls — {first_difference}",
                    plan.module_path
                );
                all_equivalent = false;
            }
            ReceiptVerdict::Refused { reason } => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — {reason}",
                    plan.module_path
                );
                all_equivalent = false;
            }
        }
    }
    // THE DENOMINATOR, EVERY RUN (operator ruling, 2026-08-20). A green here means the DERIVED
    // CALLS in the selected modules agreed -- never that a module is behaviourally equivalent.
    // Printing a bare PASS is how, inside a week, someone reads this as promotion evidence.
    for line in &denominators {
        eprintln!("{line}");
    }
    // EVERY RUN, INCLUDING ZERO -- a counter that appears only when nonzero teaches a reader that
    // its absence means "not measured", and the two then look alike in a log tail.
    //
    // THE FLOOR IS IN THE LINE, NOT IN A NOTE BESIDE IT. The line outlives the note: someone will
    // trend this number, watch it sit at N, and conclude the class is nearly closed. It is a floor
    // because the probe proves instability by DISAGREEMENT between two runs, and two randomized
    // renderings can coincide -- an uncaught call is scored as an ordinary comparison and, if it
    // then differs across the seed/candidate pair, inflates the DIVERGENT count instead.
    //
    // DISSOLUTION: this goes to zero when emission is deterministic (a `BTreeMap` container
    // template rather than `HashMap`), NOT when the probe gets better at spotting the residue.
    // A shrinking count from a sharper probe would be the metric improving while the defect stays.
    eprintln!(
        "behavioral-receipt: nondeterministic_rendering={nondeterministic_calls_total} call(s) \
         across {nondeterministic_modules} module(s) — FLOOR, not a total: instability is proved \
         by two runs disagreeing, so a call whose randomized rendering happened to agree twice is \
         not counted here and is compared as if it were deterministic. Not failures; each is a \
         subject this fragment cannot ask about until emission is deterministic"
    );
    Ok(ReceiptPlanOutcome::Ran {
        agreed: all_equivalent,
    })
}

#[cfg(test)]
mod driver_transcript_tests {
    use super::*;

    fn lines(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    /// The property the exclusion rests on: two runs of ONE binary that disagree at an index
    /// prove that index is not a function of the program alone.
    #[test]
    fn a_line_that_differs_between_two_runs_is_unstable() {
        let t = DriverTranscript::of(
            lines(&["stable() = 1", "m() = {a, b}", "also_stable() = 2"]),
            lines(&["stable() = 1", "m() = {b, a}", "also_stable() = 2"]),
        );
        assert_eq!(t.unstable, [1].into_iter().collect());
        // The transcript itself is the FIRST run, unchanged -- the probe observes, it does not
        // rewrite what gets compared.
        assert_eq!(
            t.lines,
            lines(&["stable() = 1", "m() = {a, b}", "also_stable() = 2"])
        );
    }

    /// THE FALSE-POSITIVE CONTROL, at unit grain. A deterministic corpus must yield NOTHING
    /// unstable, or every module would silently stop being compared while still printing a green.
    #[test]
    fn two_identical_runs_mark_nothing_unstable() {
        let t = DriverTranscript::of(
            lines(&["a() = 1", "b() = 2"]),
            lines(&["a() = 1", "b() = 2"]),
        );
        assert!(t.unstable.is_empty());
    }

    /// A length difference is instability that belongs to no single index. Marking every line is
    /// the fail-closed reading; comparing the common prefix would silently compare a shifted pair.
    #[test]
    fn a_length_difference_marks_every_line() {
        let t = DriverTranscript::of(lines(&["a() = 1"]), lines(&["a() = 1", "b() = 2"]));
        assert_eq!(t.unstable, [0, 1].into_iter().collect());
    }

    /// THE RESIDUE, ASSERTED SO IT IS NOT MISTAKEN FOR A CLOSED CLASS. Two randomized renderings
    /// can coincide; when they do, the probe cannot see it and the call is compared as if it were
    /// deterministic. This test PINS that limitation rather than hiding it -- if someone later
    /// makes the probe complete, this test fails and forces the FLOOR wording to be revisited.
    #[test]
    fn a_nondeterministic_call_that_agreed_twice_is_not_caught() {
        let t = DriverTranscript::of(lines(&["m() = {a, b}"]), lines(&["m() = {a, b}"]));
        assert!(t.unstable.is_empty());
    }
}

#[cfg(test)]
mod function_grain_admission_tests {
    use super::*;

    fn plan(derivable: Vec<(&str, usize)>, refused: Vec<(&str, RefusalCause)>) -> ModuleCorpusPlan {
        let declared = derivable.len() + refused.len();
        ModuleCorpusPlan {
            module_path: "test.module".to_string(),
            derivable: derivable
                .into_iter()
                .map(|(name, calls)| {
                    (
                        name.to_string(),
                        EnumeratedDomain::Exhaustive { cardinality: calls },
                        (0..calls).map(|i| vec![format!("{i}i64")]).collect(),
                    )
                })
                .collect(),
            refused: refused
                .into_iter()
                .map(|(name, cause)| (name.to_string(), cause))
                .collect(),
            declared_fn_lines: declared,
            parsed_signatures: declared,
        }
    }

    fn unbounded() -> RefusalCause {
        RefusalCause::UnboundedString {
            ty: "String".to_string(),
        }
    }

    /// BOTH ARMS OF THE ONE DEFECT, IN ONE TEST, and it goes green only when BOTH are closed.
    ///
    /// The specimen is gunbc#8704, which tripped both in a single run:
    /// `v2.compiler.self_host.stage0_crate_layout` (a wet-actuator mirror the emit never writes)
    /// and `gunbc.stage0_crate_layout_generated` (zero declared functions). A fix closing one
    /// half leaves a live red reachable by the very change that motivated it, so the assertions
    /// are stated together rather than in two tests that could be satisfied separately.
    #[test]
    fn a_zero_function_authority_is_excluded_as_such_and_not_as_a_derivation_deficit() {
        let mut p = plan(vec![], vec![]);
        p.declared_fn_lines = 0;
        p.parsed_signatures = 0;
        let coverage = function_grain_coverage(&p);
        match plan_grain_selection(&p, coverage).expect("a module with no functions is a fact") {
            Some(ReceiptExclusion::NoFunctionDeclared { module_path }) => {
                assert_eq!(module_path, "test.module")
            }
            other => panic!(
                "an authority declaring no function has no behaviour that could diverge, so it \
                 must be excluded AS SUCH -- reporting it as `none of its 0 declared functions \
                 yields a call` sends a reader to close a derivation with no subject: {other:?}"
            ),
        }
    }

    /// THE PLANTED CONTROL FOR AN EMPTY POPULATION, and it is planted precisely BECAUSE the
    /// population is empty. No authority in the corpus today declares zero functions AND owns a
    /// generated artifact, so nothing live exercises this branch -- which means main and this
    /// branch disagree about it with no measurement in either direction, and the first real member
    /// would decide the verdict by accident. This test is the statement of intent, executing:
    ///
    ///   such a module is SELECTED, not excluded, because its subject is the artifact's BYTES.
    ///
    /// The identity check in the differential loop calls no function, so "declares no function"
    /// says nothing about whether it can be checked -- and that check is the only drift observer
    /// those artifacts have had since the floor cut removed the generated-artifact drift gates
    /// (DESIGN.md names them first in what the cut left unguarded). Excluding here would delete
    /// it. Per DESIGN.md 4b(4) this control stays enrolled once the population is non-empty; it is
    /// the evidence that the higher rung is real, not scaffolding for its absence.
    #[test]
    fn a_zero_function_authority_owning_a_generated_artifact_is_selected_not_excluded() {
        let excluded = ReceiptExclusion::NoFunctionDeclared {
            module_path: "test.module".to_string(),
        };
        let survives = exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::Produced("generated bytes".to_string()),
        );
        assert!(
            survives.is_none(),
            "an artifact-owning authority has a subject -- its bytes -- and excluding it would \
             delete the only drift observer those artifacts have"
        );
    }

    /// The same for a generator that REFUSED: the path still belongs to the artifact population,
    /// and the loop is where that refusal is reported. Swallowing it here would turn a generator
    /// failure into a quiet exclusion -- a refusal downgraded to a skip.
    #[test]
    fn a_generator_refusal_does_not_become_an_exclusion() {
        let excluded = ReceiptExclusion::NoFunctionHasACorpus {
            module_path: "test.module".to_string(),
            uncovered: vec![("alpha".to_string(), unbounded())],
        };
        assert!(exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::Refused("generator said no".to_string()),
        )
        .is_none());
    }

    /// THE OTHER SIDE OF THE SAME CONTROL, without which the two above could be satisfied by never
    /// excluding anything: an ordinary module mirror answers `NotGenerated`, and there the
    /// exclusion IS the fact and must survive.
    #[test]
    fn an_ordinary_mirror_keeps_its_exclusion() {
        let excluded = ReceiptExclusion::NoFunctionDeclared {
            module_path: "test.module".to_string(),
        };
        match exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::NotGenerated,
        ) {
            Some(ReceiptExclusion::NoFunctionDeclared { module_path }) => {
                assert_eq!(module_path, "test.module")
            }
            other => panic!("a module mirror with no function has no subject at all: {other:?}"),
        }
    }

    /// THE FALSE-POSITIVE CONTROL for the arm above, read beside it: a module that DOES declare
    /// functions, none of them derivable, must still land on the derivation-deficit arm. Without
    /// this, a fix could satisfy the test above by sending every uncovered module to the new arm
    /// and the deficit population would silently go to zero.
    #[test]
    fn a_module_whose_functions_all_refuse_stays_a_derivation_deficit() {
        let p = plan(vec![], vec![("alpha", unbounded())]);
        let coverage = function_grain_coverage(&p);
        match plan_grain_selection(&p, coverage).expect("non-derivability is a fact, not ignorance")
        {
            Some(ReceiptExclusion::NoFunctionHasACorpus { uncovered, .. }) => {
                assert_eq!(uncovered.len(), 1)
            }
            other => panic!("a declared-but-underivable function is a rankable deficit: {other:?}"),
        }
    }

    /// A module with one derivable function is SELECTED -- the arm that must not be swallowed by
    /// either exclusion above.
    #[test]
    fn a_module_with_a_derivable_function_is_selected() {
        let p = plan(vec![("alpha", 2)], vec![("beta", unbounded())]);
        let coverage = function_grain_coverage(&p);
        assert!(plan_grain_selection(&p, coverage)
            .expect("a derivable function is a subject")
            .is_none());
    }

    /// THE READER-BLINDNESS ARM. `fn ` lines declared and no signature parsed is a disagreement
    /// between two readers of one fact; answering it with an exclusion would publish this
    /// fragment's own failure as a property of the authority, so it refuses.
    #[test]
    fn a_parse_that_sees_none_of_the_declared_functions_refuses_rather_than_excluding() {
        let mut p = plan(vec![], vec![]);
        p.declared_fn_lines = 7;
        p.parsed_signatures = 0;
        let coverage = function_grain_coverage(&p);
        let refusal = plan_grain_selection(&p, coverage)
            .expect_err("two readers disagreeing is ignorance, not a fact about the module");
        assert!(
            refusal.contains("readers disagree") && refusal.contains("test.module"),
            "the refusal must name what it refused and why: {refusal}"
        );
    }

    /// THE DISCRIMINATING RED for the whole change. A module whose every declared function refuses
    /// derivation used to reach the differential and come back `Refused`, which made the diff that
    /// touched it hard-fail required CI. It must now fail admission instead -- and name every
    /// function, because a module-level "nothing derived" is not something an author can act on.
    #[test]
    fn a_module_with_no_derivable_function_is_excluded_not_refused() {
        let p = plan(vec![], vec![("alpha", unbounded()), ("beta", unbounded())]);
        let coverage = AdmittedPlan::of(&p).err().expect(
            "a module yielding no call must not be admissible to a differential that would \
             compare a program against itself over an empty transcript",
        );
        let named: Vec<&str> = coverage.uncovered.iter().map(|(f, _)| f.as_str()).collect();
        assert_eq!(named, vec!["alpha", "beta"]);
        assert_eq!(coverage.calls(), 0);
    }

    /// THE POSITIVE CONTROL, and the one that makes the red above load-bearing rather than a
    /// mechanism that refuses everything. One derivable function is enough to admit, and the
    /// refused sibling stays counted as uncovered rather than disappearing into the pass.
    #[test]
    fn one_function_with_a_call_admits_and_the_rest_stay_counted() {
        let p = plan(vec![("alpha", 3)], vec![("beta", unbounded())]);
        let admitted = AdmittedPlan::of(&p).expect("one function with calls must admit");
        assert_eq!(admitted.coverage.covered, vec![("alpha".to_string(), 3)]);
        assert_eq!(admitted.coverage.calls(), 3);
        assert_eq!(
            admitted
                .coverage
                .uncovered
                .iter()
                .map(|(f, _)| f.as_str())
                .collect::<Vec<_>>(),
            vec!["beta"],
            "a function nothing ran must remain visible in the denominator of a passing module"
        );
    }

    /// THE SUBTLER HALF: a function that derived a domain containing nothing sits in `derivable`
    /// and contributes no call. Counting it as covered is a zero that reads as success. It is
    /// uncovered, with its OWN cause -- separate from the never-derivable ones, whose remedy is a
    /// different piece of work.
    #[test]
    fn empty_derived_domain_is_uncovered_not_covered() {
        let p = plan(vec![("alpha", 0)], vec![("beta", unbounded())]);
        assert_eq!(
            p.derivable.len(),
            1,
            "precondition: the plan does record this function as derivable, which is exactly why \
             counting that vector would over-report coverage"
        );
        let coverage = AdmittedPlan::of(&p)
            .err()
            .expect("a derivable function with an empty domain yields no call, so nothing admits");
        assert_eq!(
            coverage.uncovered,
            vec![
                ("alpha".to_string(), RefusalCause::EmptyDerivedDomain),
                ("beta".to_string(), unbounded()),
            ],
            "the two causes must stay distinguishable: one needs an enumerator fixed, the other \
             needs a type grounded"
        );
    }

    /// A module carrying an empty-domain function BESIDE a real one still admits, and the empty
    /// one is not silently promoted into the covered count by the module having passed.
    #[test]
    fn an_empty_domain_function_does_not_ride_a_covered_sibling() {
        let p = plan(vec![("alpha", 2), ("empty", 0)], vec![]);
        let admitted = AdmittedPlan::of(&p).expect("alpha yields calls, so the module admits");
        assert_eq!(admitted.coverage.covered, vec![("alpha".to_string(), 2)]);
        assert_eq!(
            admitted.coverage.uncovered,
            vec![("empty".to_string(), RefusalCause::EmptyDerivedDomain)]
        );
    }
}
