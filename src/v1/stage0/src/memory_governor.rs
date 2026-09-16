//! Host memory budget authority and scheduler-hold observation mirrors.
//!
//! The AIMD admission controller that lived here is deleted — concurrency is now a
//! fixed width derived up front by `derived_realization_schedule` from `std.realize_pack`

// CLIPPY ROSTER -- 12 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::single_element_loop,  // 1
    dead_code,  // 11
)]

use std::path::{Path, PathBuf};

/// Census anchor for observation witnesses — hold-line mirrors only; scheduling moved to
/// `derived_realization_schedule`.
pub const GOVERNOR_CENSUS_MARKER: &str = "[governor]";

fn governor_emoji() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
}

fn mirror_ci_tenths_text(tenths: u64) -> String {
    format!("{}.{}", tenths / 10, tenths % 10)
}

fn mirror_ci_human_bytes(bytes: u64) -> String {
    let tenths = (bytes.saturating_mul(10)) / 1_073_741_824;
    format!("{} GiB", mirror_ci_tenths_text(tenths))
}

pub(crate) fn mirror_ci_human_percent(bp: u64) -> String {
    format!("{}%", mirror_ci_tenths_text(bp / 10))
}

fn render_governor_info_line(text: &str, emoji: bool) -> String {
    let glyph = if emoji { "🕐" } else { "◷" };
    format!("{glyph} {text}")
}

fn render_governor_done_line(text: &str, emoji: bool) -> String {
    // Glyph discipline (operator live-log 2026-07-25): a receipt is *data*, not an
    // outcome — StatusPulse, never the Done/success glyph.
    render_governor_info_line(text, emoji)
}

/// Multiplicative-decrease divisor is 2 (halve), additive increase is +1 — classic AIMD.
/// The remaining thresholds are POLICY (like TCP's), not measurements of any workload:
/// they scale with the budget or are dimensionless, so no per-corpus constant returns.
/// Admission holds when `memory.current` exceeds this fraction of the budget.
const HIGH_WATER_NUM: u64 = 4;
const HIGH_WATER_DEN: u64 = 5;
/// Admission holds when PSI memory `some avg10` exceeds this percentage — the kernel is
/// already stalling tasks on reclaim, i.e. the buffer is absorbing overshoot right now.
const PSI_HOLD_AVG10: f64 = 10.0;
/// Admission holds when swap usage grew at least this much since the previous sample —
/// growth (not absolute level) is the "actively creeping above physical" signal.
const SWAP_GROWTH_HOLD_BYTES: u64 = 8 * 1024 * 1024;
/// Admissions stop once `memory.current` exceeds this fraction of the budget — the
/// maturation reserve (TCP's ssthresh): admitted demand matures MINUTES after admission
/// at ~7× its digest-time footprint (runs 29181858455/29183064852/29183727188 — a share
/// measured at index-build time read 0.48GiB where the mature share was ~3.5GiB), so
/// the other half of the pipe is reserved for in-flight growth the signals cannot see
/// yet. Dimensionless policy, like the high-water fraction — no workload constant.
const ADMIT_CEILING_NUM: u64 = 1;
const ADMIT_CEILING_DEN: u64 = 2;
/// Poll cadence while holding for admission.
const HOLD_POLL: std::time::Duration = std::time::Duration::from_millis(150);

/// One sample of the memory signals. `None` means the file/field was unreadable — the
/// corresponding check is inert for this sample (never a fabricated value).
#[derive(Debug, Clone, Copy, Default)]
pub struct MemorySignals {
    pub current_bytes: Option<u64>,
    pub swap_current_bytes: Option<u64>,
    pub psi_some_avg10: Option<f64>,
    pub events_high: Option<u64>,
    pub events_oom_kill: Option<u64>,
}

/// Why an admission was not granted this poll.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoldReason {
    /// The AIMD window is full — normal arithmetic, not a memory event (uncounted).
    WindowFull { active: usize, target: usize },
    /// `memory.current` is past the high-water fraction of the budget.
    CurrentHighWater { current: u64, high_water: u64 },
    /// PSI memory `some avg10` is past the hold threshold.
    PsiPressure { avg10: f64 },
    /// Swap usage grew since the last sample — overshoot is landing in the buffer.
    SwapGrowth { delta: u64 },
    /// A previously admitted worker has not yet paid its front-loaded admission cost
    /// (its whole-tree index build): admitting more before that demand lands would let
    /// the window outrun the memory signal (the slow-start overshoot that killed CI run
    /// 29180195694 — 16 index builds admitted on skip-speed completions before any of
    /// them allocated). Pacing arithmetic, not a memory event.
    AwaitFirstCost { undigested: usize },
    /// Admitting one more worker of the RUN-MEASURED share would cross the high-water
    /// line. Worker demand matures minutes after admission (run 29183064852: the creep
    /// back-off fired within one poll of high-water and the box still died 2 minutes
    /// later — 10GiB of margin consumed in 33s by already-admitted growth), so reactive
    /// arms alone always act too late; this gate is the predictive complement, priced by
    /// the first worker's own measured cost, never an authored constant.
    InsufficientHeadroom {
        current: u64,
        share: u64,
        high_water: u64,
    },
    /// `memory.current` is past the admission ceiling (half the budget): the maturation
    /// reserve is spoken for. Existing workers run on; new demand waits until in-flight
    /// demand finishes maturing (or drains).
    AdmissionCeiling { current: u64, ceiling: u64 },
}

impl HoldReason {
    fn describe(&self) -> String {
        // Internal diagnostic text retained for tests/debug; log projection goes through
        // `render_governor_hold_line_mirror` (ci_hold_cause_text authority).
        match self {
            HoldReason::WindowFull { active, target } => {
                format!("window full (active={active} target={target})")
            }
            HoldReason::CurrentHighWater {
                current,
                high_water,
            } => format!("memory.current {current} > high-water {high_water}"),
            HoldReason::PsiPressure { avg10 } => {
                format!("psi some avg10={avg10} > {PSI_HOLD_AVG10}")
            }
            HoldReason::SwapGrowth { delta } => {
                format!("swap grew {delta} bytes since last sample")
            }
            HoldReason::AwaitFirstCost { undigested } => {
                format!("pacing: {undigested} admitted worker(s) yet to pay first cost")
            }
            HoldReason::InsufficientHeadroom {
                current,
                share,
                high_water,
            } => format!(
                "headroom: current {current} + measured worker share {share} > high-water {high_water}"
            ),
            HoldReason::AdmissionCeiling { current, ceiling } => format!(
                "admission ceiling: current {current} > {ceiling} (half the budget is the maturation reserve)"
            ),
        }
    }
    fn emit_hold_line(&self, old_target: usize, new_target: usize) -> String {
        let _ = GOVERNOR_CENSUS_MARKER;
        let base = render_governor_hold_line_mirror(self, governor_emoji());
        if old_target != new_target {
            format!("{base} — target_width {old_target}→{new_target}")
        } else {
            base
        }
    }
    fn is_memory_creep(&self) -> bool {
        !matches!(
            self,
            HoldReason::WindowFull { .. }
                | HoldReason::AwaitFirstCost { .. }
                | HoldReason::InsufficientHeadroom { .. }
                | HoldReason::AdmissionCeiling { .. }
        )
    }
}

/// Mirror of `gunbc.observation_ci_render.ci_hold_cause_text` over the seed's HoldReason
/// (lockstep with SchedulerHold). Proven byte-equal to the seed oracle for the two
/// narrated arms (PsiPressure, CurrentHighWater); other variants share the model's
/// generic "blocked on scheduler admission" text.
pub fn mirror_ci_hold_cause_text(hold: &HoldReason) -> String {
    match hold {
        HoldReason::CurrentHighWater {
            current,
            high_water,
        } => format!(
            "blocked on the memory high-water line ({} of {})",
            mirror_ci_human_bytes(*current),
            mirror_ci_human_bytes(*high_water)
        ),
        HoldReason::PsiPressure { avg10 } => {
            // avg10 is percent with one decimal; basis points = percent × 100.
            let bp = (*avg10 * 100.0).round() as u64;
            format!(
                "blocked on memory reclaim (pressure {})",
                mirror_ci_human_percent(bp)
            )
        }
        _ => "blocked on scheduler admission".to_string(),
    }
}

/// Mirror of `gunbc.observation_seed_render.seed_governor_hold_line` —
/// `ci_hold_cause_text ∘ StatusBlocked ∘ ci_render_line`.
pub fn render_governor_hold_line_mirror(hold: &HoldReason, emoji: bool) -> String {
    let glyph = if emoji { "⏳" } else { "◷" };
    format!("{glyph} {}", mirror_ci_hold_cause_text(hold))
}
/// SEED MIRROR of `gunbc.runner_slot_allocation` `gunbc_runner_slot_desired` — the declared
/// per-slot throttle line (field `memory_high`). A mirror, not an independent value: it may only
/// move toward its authority row. Joined by `test.claim.seed_mirror_constant_lens_witness_test`.
///
/// IT IS NOT A BUDGET SOURCE — the distinction `read_host_budget_bytes` erased until 2026-08-30.
/// It declares what THIS fleet configures its own self-hosted runner slots to, true of nothing
/// else. Capping a host-shared MemAvailable reading at it made a claim about an unmeasured third
/// party's executor; on BuildBuddy (whose slot exposes no cgroup limit file) the resulting budget
/// got `main_wet` SIGKILLed at rc=137 with no diagnostic. A declared constant may bound a
/// refusal, never stand in for a reading. Remaining uses: fixtures about the fleet's own slots.
/// Authority: `gunbc.host_budget_source` the `host_budget_declared_slot_is_not_a_reading_note` annotation.
pub const DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES: u64 = 26843545600;

/// SEED MIRROR of `gunbc.runner_slot_allocation` `gunbc_floor_minimum_viable_armed_budget`
/// — SCAFFOLD (§7 seed-retained HAND-RUST; doomed/success witness receipts in that module):
/// arm-time floor refusal when the governor budget is below the measured minimum viable
/// footprint, else a crowded slot with a genuinely low cgroup limit starts a doomed ~30min walk
/// (runs 29834380839, 29845210061). Its witnesses were taken when an uncapped host could still
/// be admitted on a low MemAvailable reading; that arm is gone (such a host refuses outright),
/// so it now guards a small but READABLE bound — a tight cgroup limit or a low operator override.
/// dissolve-on: v2 emit of stage0 host-budget constants from `gunbc.runner_slot_allocation`
/// (self-host frontier row for `memory_governor` cgroup-budget readers); re-measure when
/// bright-seal #6999 fill-deferral cuts mature index residency.
pub const DECLARED_FLOOR_MINIMUM_VIABLE_ARMED_BUDGET_BYTES: u64 = 12884901888;

/// Fail-fast refusal when a floor walk's armed budget is provably below the measured
/// minimum viable footprint.
///
/// - `Some(diagnostic)` — `budget` is known and strictly below the declaration.
/// - `None` — `budget` is `None` (unreadable; refusal does not fire — the governor
///   already logs its budget source on stderr) **or** budget is at/above the threshold.
pub fn floor_budget_below_minimum_footprint(budget: Option<u64>) -> Option<String> {
    let budget = budget?;
    if budget < DECLARED_FLOOR_MINIMUM_VIABLE_ARMED_BUDGET_BYTES {
        Some(format!(
            "FloorBudgetBelowMinimumFootprint: armed budget={budget} bytes < minimum viable {} bytes (gunbc.runner_slot_allocation.gunbc_floor_minimum_viable_armed_budget; doomed witnesses 29834380839, 29845210061) — requeue on a less-crowded runner (fail-fast, not a doomed walk)",
            DECLARED_FLOOR_MINIMUM_VIABLE_ARMED_BUDGET_BYTES
        ))
    } else {
        None
    }
}

/// THE IDENTITY A WHOLE-ROOT COMPILE IS ADMITTED UNDER — the seed realization of
/// `gunbc.whole_corpus_compile_admission` `WholeCorpusCompileRootIdentity`. Repository, primary
/// root and ORDERED dependency pools, compared verbatim: a spelling that differs is a different
/// root, which refuses as unmeasured rather than joining a row it resembles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WholeCorpusCompileRootIdentity {
    pub repository: String,
    pub primary_root: String,
    pub dependency_pools: Vec<String>,
}

impl WholeCorpusCompileRootIdentity {
    fn label(&self) -> String {
        if self.dependency_pools.is_empty() {
            format!(
                "{} primary-root {} (no dependency pools)",
                self.repository, self.primary_root
            )
        } else {
            format!(
                "{} primary-root {} + dependency pools {}",
                self.repository,
                self.primary_root,
                self.dependency_pools.join(", ")
            )
        }
    }
}

/// One projected `MeasuredForRoot` row: the identity key and the peak the admission compares to.
/// The census and receipt travel in the projection for the reader and are not re-typed here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasuredRootDemandRow {
    pub root: WholeCorpusCompileRootIdentity,
    pub peak_bytes: u64,
}

/// Where the rows were read from, as an outcome — the seed realization of
/// `gunbc.whole_corpus_compile_admission` `WholeCorpusCompileDemandsRead`. The admission runs
/// before any source resolves, so the rows cannot be `.dag` values: they arrive as the
/// repository's generated projection (`gunbc.whole_corpus_compile_demand_projection`) from a
/// location the run is handed. There is no default location and no compiled-in row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WholeCorpusCompileDemandsRead {
    NotDeclared,
    Unreadable {
        location: String,
        reason: String,
    },
    Read {
        location: String,
        rows: Vec<MeasuredRootDemandRow>,
    },
}

/// The schema the projection emitter writes (`gunbc.whole_corpus_compile_demand_projection`
/// `whole_corpus_compile_demand_projection_schema`). A projection naming another schema does not
/// read: its rows are not known to mean what this reader would take them to mean.
pub const WHOLE_CORPUS_COMPILE_DEMAND_PROJECTION_SCHEMA: &str =
    "gunbc.whole_corpus_compile_demand_projection/1";

/// Parse projection bytes already in hand. Pure, so the discriminating cases are unit-testable
/// without a file; `read_whole_corpus_compile_demands` is the effect around it. Any shape it cannot
/// decode is `Unreadable` with the reason — never a row with a guessed field.
pub fn parse_whole_corpus_compile_demand_projection(
    location: &str,
    text: &str,
) -> WholeCorpusCompileDemandsRead {
    let unreadable = |reason: String| WholeCorpusCompileDemandsRead::Unreadable {
        location: location.to_string(),
        reason,
    };
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => return unreadable(format!("not JSON: {e}")),
    };
    match value.get("schema").and_then(|v| v.as_str()) {
        Some(WHOLE_CORPUS_COMPILE_DEMAND_PROJECTION_SCHEMA) => {}
        other => {
            return unreadable(format!(
                "schema is {other:?}, expected {WHOLE_CORPUS_COMPILE_DEMAND_PROJECTION_SCHEMA:?}"
            ))
        }
    }
    let Some(raw_rows) = value.get("rows").and_then(|v| v.as_array()) else {
        return unreadable("no rows array".to_string());
    };
    let mut rows = Vec::with_capacity(raw_rows.len());
    for (index, row) in raw_rows.iter().enumerate() {
        let text_field = |key: &str| row.get(key).and_then(|v| v.as_str()).map(str::to_string);
        let (Some(repository), Some(primary_root)) =
            (text_field("repository"), text_field("primary_root"))
        else {
            return unreadable(format!("row {index} lacks repository or primary_root"));
        };
        let Some(pools) = row.get("dependency_pools").and_then(|v| v.as_array()) else {
            return unreadable(format!("row {index} lacks dependency_pools"));
        };
        let mut dependency_pools = Vec::with_capacity(pools.len());
        for pool in pools {
            match pool.as_str() {
                Some(p) => dependency_pools.push(p.to_string()),
                None => return unreadable(format!("row {index} has a non-string pool")),
            }
        }
        let Some(peak_bytes) = row.get("peak_bytes").and_then(|v| v.as_u64()) else {
            return unreadable(format!("row {index} lacks an unsigned peak_bytes"));
        };
        rows.push(MeasuredRootDemandRow {
            root: WholeCorpusCompileRootIdentity {
                repository,
                primary_root,
                dependency_pools,
            },
            peak_bytes,
        });
    }
    WholeCorpusCompileDemandsRead::Read {
        location: location.to_string(),
        rows,
    }
}

/// The effect: read the projection at the location the run was handed. `None` is `NotDeclared`;
/// a location that does not read is `Unreadable` naming it.
pub fn read_whole_corpus_compile_demands(location: Option<&str>) -> WholeCorpusCompileDemandsRead {
    let Some(location) = location else {
        return WholeCorpusCompileDemandsRead::NotDeclared;
    };
    match std::fs::read_to_string(location) {
        Ok(text) => parse_whole_corpus_compile_demand_projection(location, &text),
        Err(e) => WholeCorpusCompileDemandsRead::Unreadable {
            location: location.to_string(),
            reason: e.to_string(),
        },
    }
}

/// Why a root is unmeasured — the seed realization of `WholeCorpusCompileUnmeasuredCause`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WholeCorpusCompileUnmeasuredCause {
    NoDemandsProjectionDeclared,
    DemandsProjectionNotRead { location: String, reason: String },
    NoRowForRoot { location: String },
}

/// Arm-time admission for a WHOLE-ROOT compile — the seed mirror of
/// `gunbc.whole_corpus_compile_admission` `whole_corpus_compile_admission`, arm for arm.
///
/// Exists because the budget was read and printed but joined to nothing: a whole-tree run started
/// a resolve it could not hold and was SIGKILLed on the BuildBuddy runner (invocations
/// a39713da-8cfb-415d-a8f6-1e0ef150d075 and 13cf8d2e-173a-42d2-9a56-101bb3332740), exit 137 with no
/// diagnostic. The demand it compares against is the ROOT's own measured peak, joined on the root's
/// identity; this seed carries no demand figure of its own.
///
/// What it does NOT claim: an admitted budget is not certified sufficient. A row is one tree's
/// peak, so admission means "not provably doomed at the tree that was measured" (mitigatable,
/// §4b), never "will fit".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WholeCorpusCompileAdmission {
    Admitted {
        budget_bytes: u64,
        required_bytes: u64,
    },
    RefusedBudgetBelowMeasuredDemand {
        budget_bytes: u64,
        required_bytes: u64,
        source: String,
    },
    RefusedBudgetUnreadable {
        source: String,
    },
    RefusedUnmeasuredRoot {
        root: WholeCorpusCompileRootIdentity,
        cause: WholeCorpusCompileUnmeasuredCause,
    },
    RefusedDemandsForAnotherRepository {
        root: WholeCorpusCompileRootIdentity,
        location: String,
        projected_repositories: Vec<String>,
    },
}

/// The root is resolved against the projection BEFORE the budget is read, as in the model: a
/// missing row is the deficit no larger host fixes, so it refuses even where the budget is
/// unreadable. No arm falls back to another root's row.
pub fn whole_corpus_compile_admission(
    budget: &HostBudgetResolution,
    root: &WholeCorpusCompileRootIdentity,
    read: &WholeCorpusCompileDemandsRead,
) -> WholeCorpusCompileAdmission {
    let unmeasured = |cause| WholeCorpusCompileAdmission::RefusedUnmeasuredRoot {
        root: root.clone(),
        cause,
    };
    let (location, rows) = match read {
        WholeCorpusCompileDemandsRead::NotDeclared => {
            return unmeasured(WholeCorpusCompileUnmeasuredCause::NoDemandsProjectionDeclared)
        }
        WholeCorpusCompileDemandsRead::Unreadable { location, reason } => {
            return unmeasured(
                WholeCorpusCompileUnmeasuredCause::DemandsProjectionNotRead {
                    location: location.clone(),
                    reason: reason.clone(),
                },
            )
        }
        WholeCorpusCompileDemandsRead::Read { location, rows } => (location, rows),
    };
    if !rows.is_empty() && !rows.iter().any(|r| r.root.repository == root.repository) {
        return WholeCorpusCompileAdmission::RefusedDemandsForAnotherRepository {
            root: root.clone(),
            location: location.clone(),
            projected_repositories: rows.iter().map(|r| r.root.repository.clone()).collect(),
        };
    }
    let Some(row) = rows.iter().find(|r| &r.root == root) else {
        return unmeasured(WholeCorpusCompileUnmeasuredCause::NoRowForRoot {
            location: location.clone(),
        });
    };
    let required_bytes = row.peak_bytes;
    // A DECLARED budget refuses exactly as an unreadable one does, as in the model
    // (`whole_corpus_compile_admission` maps HostBudgetDeclaredUnverified to the unreadable
    // refusal). This arm once took `(Option<u64>, label)`, which admitted on a declared
    // GUNBC_MEMORY_BUDGET_BYTES the model refuses.
    let (budget_bytes, source) = match budget {
        HostBudgetResolution::Resolved {
            effective_bytes, ..
        } => (*effective_bytes, budget.label()),
        HostBudgetResolution::DeclaredUnverified { .. }
        | HostBudgetResolution::Unreadable { .. } => {
            return WholeCorpusCompileAdmission::RefusedBudgetUnreadable {
                source: budget.label(),
            };
        }
    };
    let source = source.as_str();
    if budget_bytes < required_bytes {
        WholeCorpusCompileAdmission::RefusedBudgetBelowMeasuredDemand {
            budget_bytes,
            required_bytes,
            source: source.to_string(),
        }
    } else {
        WholeCorpusCompileAdmission::Admitted {
            budget_bytes,
            required_bytes,
        }
    }
}

/// A ROOT DEMAND MEASUREMENT THAT HAS BEEN ADMITTED — the only way to construct the measurement
/// compile subject. Its field is private, so `CompileSubject::RootDemandMeasurement` cannot be built
/// without passing `root_demand_measurement_admission`: the measurement is its own arm, not the
/// whole-root compile with its refusal skipped (seed realization of `gunbc.root_demand_measurement`,
/// direction ruling on gunbc#11265 review 65495).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedRootDemandMeasurement {
    root: WholeCorpusCompileRootIdentity,
    limit_bytes: u64,
    limit_source: String,
}

impl AdmittedRootDemandMeasurement {
    pub fn root(&self) -> &WholeCorpusCompileRootIdentity {
        &self.root
    }
    pub fn limit_bytes(&self) -> u64 {
        self.limit_bytes
    }
    pub fn limit_source(&self) -> &str {
        &self.limit_source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootDemandMeasurementAdmission {
    Admitted(AdmittedRootDemandMeasurement),
    RefusedNoEnforceableLimit {
        root: WholeCorpusCompileRootIdentity,
        reason: String,
    },
}

/// What bounds a measurement run, READ AS memory.max directly — never the budget resolution, which
/// reports the lower of memory.high and memory.max and so names memory.high whenever both are set.
/// Seed mirror of `gunbc.root_demand_measurement` `RootDemandMeasurementLimitReading`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootDemandMeasurementLimitReading {
    MemoryMaxBindsProcess { cgroup_dir: String, bytes: u64 },
    MemoryHighOnly { cgroup_dir: String, high_bytes: u64 },
    NoCgroupMemoryLimit,
}

/// The effect: walk the cgroup tree for the tightest numeric memory.max; if none binds, report a
/// memory.high that is set so the refusal can name it.
pub fn read_root_demand_measurement_limit() -> RootDemandMeasurementLimitReading {
    if let Some(dir) = binding_cap_cgroup_dir() {
        if let Some(bytes) = read_cgroup_u64(&dir, "memory.max") {
            return RootDemandMeasurementLimitReading::MemoryMaxBindsProcess {
                cgroup_dir: dir.display().to_string(),
                bytes,
            };
        }
    }
    if let Some(dir) = binding_high_cgroup_dir() {
        if let Some(high_bytes) = read_cgroup_u64(&dir, "memory.high") {
            return RootDemandMeasurementLimitReading::MemoryHighOnly {
                cgroup_dir: dir.display().to_string(),
                high_bytes,
            };
        }
    }
    RootDemandMeasurementLimitReading::NoCgroupMemoryLimit
}

/// Admitted ONLY under an observed memory.max that bounds this process (direction ruling on
/// gunbc#11265 review 65682): memory.high throttles and never kills, so under it the Exceeded receipt
/// could not fire and the parent could wait on a thrashing child indefinitely. The seed mirror of
/// `gunbc.root_demand_measurement` `root_demand_measurement_admission`, arm for arm; the same fact
/// `HostBudgetSource::bounds_this_process` carries.
pub fn root_demand_measurement_admission(
    limit: &RootDemandMeasurementLimitReading,
    root: &WholeCorpusCompileRootIdentity,
) -> RootDemandMeasurementAdmission {
    match limit {
        RootDemandMeasurementLimitReading::MemoryMaxBindsProcess { cgroup_dir, bytes } => {
            RootDemandMeasurementAdmission::Admitted(AdmittedRootDemandMeasurement {
                root: root.clone(),
                limit_bytes: *bytes,
                limit_source: HostBudgetSource::CgroupMemoryMax {
                    cgroup_dir: cgroup_dir.clone(),
                }
                .label(),
            })
        }
        RootDemandMeasurementLimitReading::MemoryHighOnly { cgroup_dir, high_bytes } => {
            RootDemandMeasurementAdmission::RefusedNoEnforceableLimit {
                root: root.clone(),
                reason: format!(
                    "memory.high={high_bytes} is set at {cgroup_dir} but memory.max is not: memory.high \
                     throttles and never kills, so it does not bound the process; the measurement \
                     needs memory.max"
                ),
            }
        }
        RootDemandMeasurementLimitReading::NoCgroupMemoryLimit => {
            RootDemandMeasurementAdmission::RefusedNoEnforceableLimit {
                root: root.clone(),
                reason: "no cgroup memory.max binds this process; a declared or physical-memory \
                         budget bounds nothing, and the measurement needs memory.max"
                    .to_string(),
            }
        }
    }
}

pub fn root_demand_measurement_refusal_diagnostic(
    admission: &RootDemandMeasurementAdmission,
) -> Option<String> {
    match admission {
        RootDemandMeasurementAdmission::Admitted(_) => None,
        RootDemandMeasurementAdmission::RefusedNoEnforceableLimit { root, reason } => Some(format!(
            "RootDemandMeasurementRefusedNoEnforceableLimit: measuring {} needs an enforceable cgroup \
             memory.max binding this process, because a root with no row has no demand figure to be \
             refused against and only the host can bound the run — {reason}. Remedy: run the \
             measurement where a cgroup memory.max binds the process.",
            root.label()
        )),
    }
}

/// How the measured child ended, as the parent observed it through `wait4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootDemandMeasurementWait {
    Exited(i32),
    Signaled(i32),
}

/// What the measured child reported resolving, when it lived to report it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootDemandMeasurementCensus {
    pub source_count: u64,
    pub source_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootDemandMeasurementRun {
    pub root: WholeCorpusCompileRootIdentity,
    pub limit_bytes: u64,
    pub limit_source: String,
    pub measured_on_host: String,
    pub instrument_run: String,
    pub census: Option<RootDemandMeasurementCensus>,
}

/// The receipt the PARENT records — the seed mirror of `gunbc.root_demand_measurement`
/// `RootDemandMeasurementReceipt`. It carries no artifact and no subject verdict by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootDemandMeasurementReceipt {
    Completed {
        run: RootDemandMeasurementRun,
        exit_status: i32,
        peak_bytes: u64,
    },
    /// Killed at the limit: an honest lower bound (demand > limit), never a peak.
    Exceeded { run: RootDemandMeasurementRun },
    Terminated {
        run: RootDemandMeasurementRun,
        signal: i32,
    },
}

pub const SIGKILL_SIGNAL: i32 = 9;

/// THE PARENT'S EXIT CODE IS THE RECEIPT'S STANDING, NOT THE FACT THAT A RECEIPT WAS WRITTEN.
/// A child killed at the limit or by any other signal, or one that completed with a nonzero
/// status, measured no peak; a caller reading `$?` must see that as a deficit (DESIGN section 5:
/// a wrong answer is a loud error). Only a completed, zero-status child answers 0 (review 66337).
pub fn root_demand_measurement_exit_code(receipt: &RootDemandMeasurementReceipt) -> i32 {
    match receipt {
        RootDemandMeasurementReceipt::Completed { exit_status: 0, .. } => 0,
        RootDemandMeasurementReceipt::Completed { .. }
        | RootDemandMeasurementReceipt::Exceeded { .. }
        | RootDemandMeasurementReceipt::Terminated { .. } => 1,
    }
}

/// Map the parent's observation to the receipt. A SIGKILL under an enforceable limit is the kill
/// at the limit; any other signal measured nothing.
pub fn root_demand_measurement_receipt(
    run: RootDemandMeasurementRun,
    wait: RootDemandMeasurementWait,
    peak_bytes: u64,
) -> RootDemandMeasurementReceipt {
    match wait {
        RootDemandMeasurementWait::Exited(exit_status) => RootDemandMeasurementReceipt::Completed {
            run,
            exit_status,
            peak_bytes,
        },
        RootDemandMeasurementWait::Signaled(SIGKILL_SIGNAL) => {
            RootDemandMeasurementReceipt::Exceeded { run }
        }
        RootDemandMeasurementWait::Signaled(signal) => {
            RootDemandMeasurementReceipt::Terminated { run, signal }
        }
    }
}

pub const ROOT_DEMAND_MEASUREMENT_RECEIPT_SCHEMA: &str = "gunbc.root_demand_measurement/receipt/1";

pub fn root_demand_measurement_receipt_json(receipt: &RootDemandMeasurementReceipt) -> String {
    let (arm, run) = match receipt {
        RootDemandMeasurementReceipt::Completed { run, .. } => ("completed", run),
        RootDemandMeasurementReceipt::Exceeded { run } => ("exceeded", run),
        RootDemandMeasurementReceipt::Terminated { run, .. } => ("terminated", run),
    };
    let mut v = serde_json::json!({
        "schema": ROOT_DEMAND_MEASUREMENT_RECEIPT_SCHEMA,
        "arm": arm,
        "repository": run.root.repository,
        "primary_root": run.root.primary_root,
        "dependency_pools": run.root.dependency_pools,
        "limit_bytes": run.limit_bytes,
        "limit_source": run.limit_source,
        "measured_on_host": run.measured_on_host,
        "instrument_run": run.instrument_run,
        "census": run.census.map(|c| serde_json::json!({"source_count": c.source_count, "source_bytes": c.source_bytes})),
    });
    match receipt {
        RootDemandMeasurementReceipt::Completed {
            exit_status,
            peak_bytes,
            ..
        } => {
            v["exit_status"] = serde_json::json!(exit_status);
            v["peak_bytes"] = serde_json::json!(peak_bytes);
        }
        RootDemandMeasurementReceipt::Exceeded { .. } => {
            v["demand_exceeds_bytes"] = serde_json::json!(run.limit_bytes);
        }
        RootDemandMeasurementReceipt::Terminated { signal, .. } => {
            v["signal"] = serde_json::json!(signal);
        }
    }
    format!("{v}\n")
}

/// The census line the measured child prints and the parent reads; one line, one prefix, so the
/// parent never parses compiler output for it.
pub const ROOT_DEMAND_MEASUREMENT_CENSUS_PREFIX: &str = "root-demand-measurement-census:";

pub fn parse_root_demand_measurement_census(stdout: &str) -> Option<RootDemandMeasurementCensus> {
    stdout.lines().find_map(|line| {
        let rest = line.strip_prefix(ROOT_DEMAND_MEASUREMENT_CENSUS_PREFIX)?;
        let mut parts = rest.split_whitespace();
        Some(RootDemandMeasurementCensus {
            source_count: parts.next()?.parse().ok()?,
            source_bytes: parts.next()?.parse().ok()?,
        })
    })
}

/// The recipe an unmeasured root names — the seed rendering of
/// `gunbc.whole_corpus_compile_admission` `whole_corpus_compile_measurement_recipe`.
fn whole_corpus_compile_measurement_recipe(root: &WholeCorpusCompileRootIdentity) -> String {
    let pools: String = root
        .dependency_pools
        .iter()
        .map(|p| format!(" --source-root {p}"))
        .collect();
    format!(
        "measure once, on a host where a cgroup memory.max binds the process: gunbc \
         measure-root-demand --repository {} --source-root {}{pools} --receipt <receipt.json>; the \
         run's only product is that receipt. Author the MeasuredForRoot row for {} from it with \
         gunbc.root_demand_measurement measured_for_root_from_receipt in that repository's own \
         measured-root demands, regenerate the repository's demand projection, and hand its \
         location to the compile",
        root.repository,
        root.primary_root,
        root.label()
    )
}

/// `Some(diagnostic)` on every refusal arm, `None` when admitted. Each names the disagreeing
/// quantities or the missing fact and a remedy — a refusal proposing none is a stopped line nobody
/// can restart.
///
/// Deliberately a free function, not an inherent method: `std.decl_ref` `DeclField` offers
/// `WholeDeclaration` or `NamedField`, neither naming an impl-block method, so an impl method
/// cannot be cited in the `SeedGrowthJustification` this change owes.
pub fn whole_corpus_compile_refusal_diagnostic(
    admission: &WholeCorpusCompileAdmission,
) -> Option<String> {
    match admission {
        WholeCorpusCompileAdmission::Admitted { .. } => None,
        WholeCorpusCompileAdmission::RefusedBudgetBelowMeasuredDemand {
            budget_bytes,
            required_bytes,
            source,
        } => Some(format!(
            "WholeCorpusCompileBudgetBelowMeasuredDemand: host memory budget={budget_bytes} \
             bytes (source={source}) is below this root's measured whole-root compile demand of \
             {required_bytes} bytes (its MeasuredForRoot row, gunbc.whole_corpus_compile_admission). \
             Refusing to start a run that is provably below measured demand — the previous \
             behaviour was to start it and be SIGKILLed, which reports as a silent exit-137 zero \
             rather than a diagnostic. Remedy: scope the compile with --entry <file.dag>, or run \
             it where a larger budget is readable."
        )),
        WholeCorpusCompileAdmission::RefusedBudgetUnreadable { source } => Some(format!(
            "WholeCorpusCompileBudgetUnreadable: no modeled host memory source answered \
             ({source}), so the bound on a whole-root compile is UNKNOWN. Refusing rather than \
             admitting against the widest cap available — an unbounded resolve on an unbounded \
             host is the OOM-kill this arm exists to prevent. Model this platform's memory source \
             (dag/gunbc/host/host_budget_source.dag)."
        )),
        WholeCorpusCompileAdmission::RefusedUnmeasuredRoot { root, cause } => {
            let why = match cause {
                WholeCorpusCompileUnmeasuredCause::NoDemandsProjectionDeclared => {
                    "no demand projection was declared (--measured-root-demands)".to_string()
                }
                WholeCorpusCompileUnmeasuredCause::DemandsProjectionNotRead { location, reason } => {
                    format!("the demand projection at {location} did not read: {reason}")
                }
                WholeCorpusCompileUnmeasuredCause::NoRowForRoot { location } => {
                    format!("the demand projection at {location} holds no row for this root")
                }
            };
            Some(format!(
                "WholeCorpusCompileUnmeasuredRoot: {} has no measured whole-root compile demand — \
                 {why}. Refusing rather than admitting on another root's number or extrapolating \
                 one. Remedy: {}.",
                root.label(),
                whole_corpus_compile_measurement_recipe(root)
            ))
        }
        WholeCorpusCompileAdmission::RefusedDemandsForAnotherRepository {
            root,
            location,
            projected_repositories,
        } => Some(format!(
            "WholeCorpusCompileDemandsForAnotherRepository: the demand projection at {location} \
             carries rows for {} and none for repository {}, so it is another repository's \
             projection, not this one's. Refusing: admitting {} on it would decide this root on a \
             population it does not describe. Remedy: hand the compile this repository's own \
             projection.",
            projected_repositories.join(", "),
            root.repository,
            root.label()
        )),
    }
}

/// SEED MIRROR of `gunbc.memory_stall_refusal` `memory_stall_major_fault_rate_per_minute_threshold`.
/// POLICY, like the AIMD thresholds above: two orders of magnitude over healthy background
/// (a warm resolve sustains ~0 majflt/min), an order under the measured treadmill (a
/// thrashing swapless VM refaults thousands per second — the 2026-08-30 default-VM
/// specimens, `memory_stall_observed_receipt_note`).
pub const MEMORY_STALL_MAJOR_FAULT_RATE_PER_MINUTE_THRESHOLD: u64 = 6000;

/// SEED MIRROR of `gunbc.memory_stall_refusal` `memory_stall_verdict_window_minimum_wall_ms`.
/// What makes the rate a rate rather than a spike detector: no verdict below this much wall,
/// so a cold-start refault burst can never refuse a run that then progresses.
pub const MEMORY_STALL_VERDICT_WINDOW_MINIMUM_WALL_MS: u64 = 30000;

/// SEED MIRROR of `gunbc.memory_stall_refusal` `memory_stall_progress_cpu_share_floor`.
/// The progress half of the refusal's conjunction, forced by this verdict's own first CI
/// execution (`memory_stall_admitted_under_pressure_receipt_note`): the floor runner
/// sustained 12431 majflt/min through a 13-minute CPU-bound typecheck under a memory.high
/// reclaim throttle — over the rate line and demonstrably completing, so rate alone
/// over-refuses. A window is a treadmill only if the process's own USER CPU is also below
/// this share of the wall — utime alone, never stime, because the measured treadmill
/// (invocation dd090164-9f2b-45fe-93bc-789cdd4ef9c4: 178795 majflt/min, utime 0.8% of
/// wall, stime 34%) shows the kernel billing its reclaim labour to the faulting process
/// as system time, so a utime+stime share reads a pure treadmill as one-third computing.
/// Declared in BASIS POINTS (the `std.measure` `BasisPoint` carrier the authority row
/// uses): 2000 bp = 20%.
pub const MEMORY_STALL_PROGRESS_CPU_SHARE_FLOOR_BASIS_POINTS: u64 = 2000;

/// Seed mirror of `gunbc.memory_stall_refusal` `MemoryStallObservation`: one windowed
/// sample of the process's own eviction/readmission behaviour — its major-fault counter
/// (the kernel's record of this process re-reading its own evicted pages) and its own CPU
/// time for the same wall, beside the typed cache's own eviction and readmission counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryStallObservation {
    pub window_wall_ms: u64,
    pub major_faults_in_window: u64,
    pub self_user_cpu_ms_in_window: u64,
    pub cache_evictions_in_window: u64,
    pub cache_readmissions_in_window: u64,
}

/// Seed mirror of `gunbc.memory_stall_refusal` `MemoryStallVerdict`. Three states, none
/// conflated: an open window is not evidence in either direction, a computable window with
/// the fault counter flat is progress however slow the arithmetic (time enters only as the
/// rate's denominator — never a refusal condition), and a computable window over the line
/// is the refusal, carrying the whole observation so the diagnostic names what was seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStallVerdict {
    StallWindowOpen {
        window_wall_ms: u64,
        minimum_wall_ms: u64,
    },
    ProgressUnderMemoryAdmissible {
        major_faults_per_minute: u64,
        self_cpu_share_basis_points: u64,
    },
    StallRefusedPageThrash {
        major_faults_per_minute: u64,
        threshold_per_minute: u64,
        self_cpu_share_basis_points: u64,
        cpu_share_floor_basis_points: u64,
        observation: MemoryStallObservation,
    },
}

/// Mirror of `memory_stall_major_faults_per_minute`. Callers reach it through
/// `memory_stall_verdict`, which guards the zero-wall window behind `StallWindowOpen`.
pub fn memory_stall_major_faults_per_minute(o: &MemoryStallObservation) -> u64 {
    o.major_faults_in_window.saturating_mul(60000) / o.window_wall_ms.max(1)
}

/// Mirror of `memory_stall_self_cpu_share` — the progress half of the conjunction, in
/// basis points of the window's wall (the `BasisPoint` grain of the authority row).
pub fn memory_stall_self_cpu_share_basis_points(o: &MemoryStallObservation) -> u64 {
    o.self_user_cpu_ms_in_window.saturating_mul(10_000) / o.window_wall_ms.max(1)
}

/// Mirror of `gunbc.memory_stall_refusal` `memory_stall_verdict`: the refusal is a
/// CONJUNCTION — fault rate above its line AND the process's own CPU share of the window
/// below its floor. Rate alone over-refuses (the CI floor runner sustains 12431 majflt/min
/// through a CPU-bound typecheck under a memory.high reclaim throttle and completes —
/// `memory_stall_admitted_under_pressure_receipt_note`); CPU share is what the "not
/// computing" claim in the refusal actually measures.
pub fn memory_stall_verdict(o: MemoryStallObservation) -> MemoryStallVerdict {
    if o.window_wall_ms < MEMORY_STALL_VERDICT_WINDOW_MINIMUM_WALL_MS {
        return MemoryStallVerdict::StallWindowOpen {
            window_wall_ms: o.window_wall_ms,
            minimum_wall_ms: MEMORY_STALL_VERDICT_WINDOW_MINIMUM_WALL_MS,
        };
    }
    let rate = memory_stall_major_faults_per_minute(&o);
    let cpu_share_bp = memory_stall_self_cpu_share_basis_points(&o);
    if rate > MEMORY_STALL_MAJOR_FAULT_RATE_PER_MINUTE_THRESHOLD
        && cpu_share_bp < MEMORY_STALL_PROGRESS_CPU_SHARE_FLOOR_BASIS_POINTS
    {
        MemoryStallVerdict::StallRefusedPageThrash {
            major_faults_per_minute: rate,
            threshold_per_minute: MEMORY_STALL_MAJOR_FAULT_RATE_PER_MINUTE_THRESHOLD,
            self_cpu_share_basis_points: cpu_share_bp,
            cpu_share_floor_basis_points: MEMORY_STALL_PROGRESS_CPU_SHARE_FLOOR_BASIS_POINTS,
            observation: o,
        }
    } else {
        MemoryStallVerdict::ProgressUnderMemoryAdmissible {
            major_faults_per_minute: rate,
            self_cpu_share_basis_points: cpu_share_bp,
        }
    }
}

/// Mirror of `memory_stall_refusal_pressure_text`: the pressure clause of the refusal —
/// what was observed, against what line, from which counter, and the remedy. The LOCATION
/// half (which module, which budget and source) is composed by the consumer that holds
/// those facts. Non-refusing verdicts render nothing: a consumer logging the clause
/// unconditionally must not fabricate pressure prose for a window that admitted.
pub fn memory_stall_refusal_pressure_text(v: &MemoryStallVerdict) -> String {
    match v {
        MemoryStallVerdict::StallWindowOpen { .. }
        | MemoryStallVerdict::ProgressUnderMemoryAdmissible { .. } => String::new(),
        MemoryStallVerdict::StallRefusedPageThrash {
            major_faults_per_minute,
            threshold_per_minute,
            self_cpu_share_basis_points,
            cpu_share_floor_basis_points,
            observation,
        } => format!(
            "MemoryStallRefusedPageThrash: this process refaulted its own evicted pages at \
             {major_faults_per_minute} major faults/minute while computing for only \
             {}% of the wall, over the last {} ms ({} major faults; \
             rate line {threshold_per_minute}/minute, CPU-share floor \
             {}%; source /proc/self/stat majflt and utime; \
             typed-cache evictions in window {}, readmissions {}). The wall clock is being \
             spent re-reading resident pages the kernel evicted, not computing: the machine \
             underneath cannot deliver the admitted budget beside its own page cache, and \
             with swap disabled this treadmill holds indefinitely instead of failing. \
             Refusing rather than holding. Remedy: run where the admitted budget is \
             genuinely available (a larger runner), or scope the compile with --entry to a \
             smaller closure.",
            self_cpu_share_basis_points / 100,
            observation.window_wall_ms,
            observation.major_faults_in_window,
            cpu_share_floor_basis_points / 100,
            observation.cache_evictions_in_window,
            observation.cache_readmissions_in_window,
        ),
    }
}

/// This process's cumulative major-fault count — `/proc/self/stat` majflt, proc.5 field 12
/// (index 9 after the comm field, whose parentheses force the split on the LAST `)` —
/// the same parse discipline as `v1_rt::trace_process_tree_cpu_ms`). `None` where the file
/// is absent or unparsable (Darwin has no procfs): the stall check is inert for that
/// sample, never a fabricated zero — the documented `MemorySignals` discipline above.
pub fn self_major_faults() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let after_comm = stat.rsplit(')').next()?;
    after_comm.split_whitespace().nth(9)?.parse().ok()
}

/// This process's own cumulative USER-CPU milliseconds — utime alone (`/proc/self/stat`
/// field 14, index 11 after comm). Deliberately WITHOUT stime: the measured treadmill
/// runs 34% stime while computing nothing, because the kernel bills reclaim and
/// fault-handling labour to the faulting process as system time — utime is the only
/// component that measures this program's own instructions retiring. And deliberately
/// WITHOUT the reaped-child fields `v1_rt::trace_process_tree_cpu_ms` adds: a child's CPU
/// landing at reap time would spike the share of a window the parent spent waiting. Same
/// `None`-when-unreadable discipline as `self_major_faults`.
pub fn self_user_cpu_ms() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let after_comm = stat.rsplit(')').next()?;
    after_comm
        .split_whitespace()
        .nth(11)?
        .parse::<u64>()
        .ok()
        .map(|t| t * 1000 / 100)
}

/// The typed budget SOURCE — seed mirror of `gunbc.host_budget_source` `HostBudgetSource`.
/// Consumers ask the discriminant rather than scanning the display label:
/// `cli_run::entry_resolve::typed_module_cache_cap_derivation` decided "degraded" with
/// `label.contains("memory.max") || label.contains("memory.high")` — one string doing two jobs,
/// so a reworded diagnostic silently moved the verdict, and the operator's own env override was
/// classified degraded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostBudgetSource {
    CgroupMemoryHigh { cgroup_dir: String },
    CgroupMemoryMax { cgroup_dir: String },
    DarwinPhysicalMemory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostBudgetObservation {
    pub source: HostBudgetSource,
    pub bytes: u64,
}

impl HostBudgetSource {
    /// Mirror of `host_budget_source_label`.
    pub fn label(&self) -> String {
        match self {
            HostBudgetSource::CgroupMemoryHigh { cgroup_dir } => {
                format!("cgroup memory.high ({cgroup_dir})")
            }
            HostBudgetSource::CgroupMemoryMax { cgroup_dir } => {
                format!("cgroup memory.max ({cgroup_dir})")
            }
            HostBudgetSource::DarwinPhysicalMemory => "sysctl hw.memsize".to_string(),
        }
    }

    /// Mirror of `host_budget_source_bounds_this_process` — true when the thing measured is
    /// something THIS process cannot exceed, rather than a fact about the machine.
    pub fn bounds_this_process(&self) -> bool {
        match self {
            HostBudgetSource::CgroupMemoryMax { .. } => true,
            HostBudgetSource::CgroupMemoryHigh { .. } | HostBudgetSource::DarwinPhysicalMemory => {
                false
            }
        }
    }

    /// Mirror of `host_budget_source_is_degraded`.
    pub fn is_degraded(&self) -> bool {
        matches!(self, HostBudgetSource::DarwinPhysicalMemory)
    }
}

/// The seed mirror of `gunbc.host_budget_source` `HostBudgetResolution`. `Unreadable` is NOT
/// a source: it carries a reason and no number, and no consumer may turn it into one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostBudgetResolution {
    Resolved {
        effective_bytes: u64,
        requested_bytes: Option<u64>,
        observation: HostBudgetObservation,
    },
    Unreadable {
        reason: String,
    },
    /// An operator requested a planning ceiling, but no independently observed limit
    /// establishes what this process can actually consume.  The declaration is retained
    /// as a usable planning input for consumers that only derive cache entry count or
    /// concurrency; the distinct variant prevents promotion into an observed/enforced limit.
    DeclaredUnverified {
        requested_bytes: u64,
        reason: String,
    },
}

impl HostBudgetResolution {
    /// Mirror of `host_budget_resolution_bytes`.
    pub fn bytes(&self) -> Option<u64> {
        match self {
            HostBudgetResolution::Resolved {
                effective_bytes, ..
            } => Some(*effective_bytes),
            HostBudgetResolution::DeclaredUnverified {
                requested_bytes, ..
            } => Some(*requested_bytes),
            HostBudgetResolution::Unreadable { .. } => None,
        }
    }

    /// The line a human reads. For `Unreadable` this is the refusal reason, so a caller that
    /// logs the label never fabricates a provenance for a read that did not happen.
    pub fn label(&self) -> String {
        match self {
            HostBudgetResolution::Resolved {
                effective_bytes,
                requested_bytes,
                observation,
            } => match requested_bytes {
                Some(requested) => format!(
                    "effective planning minimum {effective_bytes} bytes (env request {requested}; observed {}={} bytes)",
                    observation.source.label(), observation.bytes
                ),
                None => observation.source.label(),
            },
            HostBudgetResolution::Unreadable { reason } => format!("unreadable: {reason}"),
            HostBudgetResolution::DeclaredUnverified {
                requested_bytes,
                reason,
            } => format!(
                "declared-unverified: env GUNBC_MEMORY_BUDGET_BYTES={requested_bytes}; {reason}"
            ),
        }
    }

    /// Degraded is a question about a SOURCE. An unreadable budget has no source, and the
    /// only honest answer is that there is nothing to grade — callers refuse instead.
    pub fn degraded_source(&self) -> Option<bool> {
        match self {
            HostBudgetResolution::Resolved { observation, .. } => {
                Some(observation.source.is_degraded())
            }
            HostBudgetResolution::Unreadable { .. }
            | HostBudgetResolution::DeclaredUnverified { .. } => None,
        }
    }
}

/// The refusal reason. Mirror of `host_budget_unreadable_on_kernel`: an operator declaration
/// cannot repair a missing observation because it supplies a planning request, not enforcement.
pub fn host_budget_unreadable_reason() -> String {
    format!(
        "no cgroup memory.high or memory.max binds this process and GUNBC_MEMORY_BUDGET_BYTES \
         cannot verify one (target_os={}), so the planning allowance is UNKNOWN. Refusing rather than \
         admitting against the widest signal available: a host-shared reading is a number \
         about the MACHINE, not about this slot, and admitting against one is the rc=137 \
         SIGKILL this arm exists to prevent (BuildBuddy receipt 2026-08-30, \
         gunbc.host_budget_source host_budget_source_seed_mirror_disposition). The executor must \
         expose an enforceable limit; GUNBC_MEMORY_BUDGET_BYTES may only request a lower planning ceiling.",
        std::env::consts::OS
    )
}

/// Resolve the host budget from OBSERVATIONS, so every arm — including the refusal — is
/// reachable from a test on any machine. `read_host_budget_resolution` is this function
/// applied to the real reads; nothing else composes the precedence.
///
/// The effective planning ceiling is the minimum of the operator request and every observed
/// applicable cgroup line. An operator request alone is `DeclaredUnverified`: an integer in an
/// environment variable constrains no allocation and is not evidence of executor provisioning.
/// There is no meminfo arm: MemAvailable and MemTotal describe a MACHINE, and
/// on a kernel that can express a private limit, substituting one for the limit this process
/// failed to read is DESIGN §5's absorbing fallback (answering with a superset). Authority:
/// `gunbc.host_budget_source` `host_budget_source_admissible_as_bound_on_kernel`.
pub fn resolve_host_budget(
    env_override: Option<u64>,
    cgroup_high: Option<(String, u64)>,
    cgroup_max: Option<(String, u64)>,
    darwin_physical: Option<u64>,
) -> HostBudgetResolution {
    let observation = match (cgroup_high, cgroup_max) {
        (Some(high), Some(max)) => Some(if high.1 <= max.1 {
            (
                HostBudgetSource::CgroupMemoryHigh { cgroup_dir: high.0 },
                high.1,
            )
        } else {
            (
                HostBudgetSource::CgroupMemoryMax { cgroup_dir: max.0 },
                max.1,
            )
        }),
        (Some((cgroup_dir, bytes)), None) => {
            Some((HostBudgetSource::CgroupMemoryHigh { cgroup_dir }, bytes))
        }
        (None, Some((cgroup_dir, bytes))) => {
            Some((HostBudgetSource::CgroupMemoryMax { cgroup_dir }, bytes))
        }
        (None, None) => None,
    };
    if let Some((source, observed_bytes)) = observation {
        return HostBudgetResolution::Resolved {
            effective_bytes: env_override
                .map(|requested| requested.min(observed_bytes))
                .unwrap_or(observed_bytes),
            requested_bytes: env_override,
            observation: HostBudgetObservation {
                source,
                bytes: observed_bytes,
            },
        };
    }
    // Darwin only. `darwin_physical_memory_bytes` is `None` on every other target, so this
    // arm cannot be reached on a kernel that has cgroups — which is precisely the wall
    // `host_budget_source_admissible_as_bound_on_kernel` states: a host-shared reading may
    // serve as the budget only where no private-limit mechanism exists.
    if let Some(bytes) = darwin_physical {
        return HostBudgetResolution::Resolved {
            effective_bytes: env_override
                .map(|requested| requested.min(bytes))
                .unwrap_or(bytes),
            requested_bytes: env_override,
            observation: HostBudgetObservation {
                source: HostBudgetSource::DarwinPhysicalMemory,
                bytes,
            },
        };
    }
    if let Some(requested_bytes) = env_override {
        return HostBudgetResolution::DeclaredUnverified {
            requested_bytes,
            reason: "no observed private memory.high or memory.max verifies the executor allowance; the declaration is a planning request, not an enforced process limit".to_string(),
        };
    }
    HostBudgetResolution::Unreadable {
        reason: host_budget_unreadable_reason(),
    }
}

/// The host memory planning ceiling, as a typed resolution. It does not cap RSS. Single authority shared
/// by the MemoryGovernor (which SCHEDULES against it), the typed-module cache cap (which
/// bounds an estimated ENTRY COUNT with it) and the P4 realize advisory (which PREDICTS against it) — no
/// consumer may re-read a partial version of this precedence (§3 single authority).
pub fn read_host_budget_resolution() -> HostBudgetResolution {
    let env_override = std::env::var("GUNBC_MEMORY_BUDGET_BYTES")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    let cgroup_high = binding_high_cgroup_dir().and_then(|dir| {
        read_cgroup_u64(&dir, "memory.high").map(|v| (dir.display().to_string(), v))
    });
    let cgroup_max = binding_cap_cgroup_dir().and_then(|dir| {
        read_cgroup_u64(&dir, "memory.max").map(|v| (dir.display().to_string(), v))
    });
    resolve_host_budget(
        env_override,
        cgroup_high,
        cgroup_max,
        darwin_physical_memory_bytes(),
    )
}

/// `(budget, source label)` view of `read_host_budget_resolution` for the log lines and
/// diagnostics that render the source as text. A consumer deciding anything about the
/// SOURCE must use the resolution, never this label (§3: the label is a rendering, not a
/// second representation of the discriminant).
/// INHERITED DEFECT, DECLARED HERE RATHER THAN LEFT FOR THE NEXT READER TO TRIP OVER
/// (found by review 2026-09-12; NOT introduced by the slot-sizing change that surfaced it).
///
/// THE MISMATCH: `gunbc.whole_corpus_compile_admission`'s `.dag` surface explicitly REFUSES a
/// `HostBudgetDeclaredUnverified` resolution -- an unverified request is not an observation of any
/// host, so it may not license a whole-corpus compile. This function reduces the typed resolution
/// to `(Option<u64>, String)`, and `HostBudgetResolution::bytes()` returns the REQUESTED number for
/// the unverified arm. The Rust admission decision then checks the number and never asks whether it
/// was verified, so a sufficiently large unverified request can admit through the composition that
/// the model refuses.
///
/// WHAT IS AND IS NOT CLAIMED. The mechanism is read from source. It is NOT established that this
/// has occurred on any host, and it is NOT the desired-row substitution it superficially resembles:
/// the runtime path does not pass `DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES`, and the budget resolver
/// limits an environment request by the observed bound where one is readable. Conflating the two
/// would attach this defect to the wrong change and leave it unowned when that change lands.
///
/// NEXT-RUNG TRIGGER: carry the typed `HostBudgetResolution` into the real admission decision so the
/// unverified arm is refused where the model refuses it. Reconstructing verification by parsing the
/// rendered `label()` string is explicitly NOT the repair -- that would make a second authority for
/// a fact the type already holds, which is how this seam lost the distinction in the first place.
///
/// RUNG: *mitigatable*. The `.dag` authority states the correct rule and the Rust path does not
/// enforce it; nothing detects the divergence today.
pub fn read_host_budget_bytes() -> (Option<u64>, String) {
    let resolution = read_host_budget_resolution();
    (resolution.bytes(), resolution.label())
}

/// leaf→root walk — the effective budget the OOM-killer enforces. `None` when unreadable
/// or no ancestor sets a numeric cap.
pub fn binding_cap_cgroup_dir() -> Option<PathBuf> {
    tightest_cgroup_dir_for("memory.max")
}

/// The cgroup directory whose `memory.high` is the TIGHTEST along the same walk — the
/// reclaim-throttle (speed) line, distinct from `memory.max` (the kill line).
pub fn binding_high_cgroup_dir() -> Option<PathBuf> {
    tightest_cgroup_dir_for("memory.high")
}

/// Leaf→root walk shared by the binding-dir readers: the directory carrying the smallest
/// numeric value of `limit_file` (non-numeric `max` = unset).
pub fn tightest_cgroup_dir_for(limit_file: &str) -> Option<PathBuf> {
    let self_cg = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let rel = self_cg
        .lines()
        .find_map(|l| l.strip_prefix("0::"))
        .map(|p| p.trim().trim_start_matches('/').to_string())?;
    let root = Path::new("/sys/fs/cgroup");
    let mut dir = root.join(&rel);
    let mut best: Option<(u64, PathBuf)> = None;
    loop {
        if let Ok(s) = std::fs::read_to_string(dir.join(limit_file)) {
            let s = s.trim();
            if s != "max" {
                if let Ok(v) = s.parse::<u64>() {
                    let take = best.as_ref().map(|(cur, _)| v < *cur).unwrap_or(true);
                    if take {
                        best = Some((v, dir.clone()));
                    }
                }
            }
        }
        if dir == root || !dir.pop() {
            break;
        }
    }
    best.map(|(_, d)| d)
}

/// The process's own deepest (leaf) cgroup from `/proc/self/cgroup`.
pub fn leaf_cgroup_dir() -> Option<PathBuf> {
    let self_cg = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let rel = self_cg
        .lines()
        .find_map(|l| l.strip_prefix("0::"))
        .map(|p| p.trim().trim_start_matches('/').to_string())?;
    Some(Path::new("/sys/fs/cgroup").join(rel))
}

/// `memory.events`-style content: whitespace-separated `key value` lines. Pure over the
/// file content so the parse carries its own unit tests.
pub fn memory_events_field(content: &str, field: &str) -> Option<u64> {
    content.lines().find_map(|l| {
        let mut it = l.split_whitespace();
        match (it.next(), it.next()) {
            (Some(k), Some(v)) if k == field => v.parse().ok(),
            _ => None,
        }
    })
}

/// PSI `memory.pressure` content: extract `avg10` from the `some` line
/// (`some avg10=1.23 avg60=... total=...`).
pub fn memory_pressure_some_avg10(content: &str) -> Option<String> {
    let line = content
        .lines()
        .find(|l| l.split_whitespace().next() == Some("some"))?;
    line.split_whitespace()
        .find_map(|t| t.strip_prefix("avg10="))
        .map(|v| v.to_string())
}

pub fn read_cgroup_u64(dir: &Path, file: &str) -> Option<u64> {
    std::fs::read_to_string(dir.join(file))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

pub fn read_cgroup_raw(dir: &Path, file: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(file))
        .ok()
        .map(|s| s.trim().to_string())
}

/// Total physical RAM in bytes from Darwin's `sysctl` MIB `hw.memsize`.
///
/// Authority: `dag/extdeps/darwin/sysctl.dag` (`HwMemsize`), cited to Apple's sysctl.3. Darwin's
/// answer to `/proc/meminfo` MemTotal, denominated in BYTES where meminfo's fields are kibibytes
/// — the detail a shared parser would get wrong by 1024x. Observed live on macOS 15:
/// 17179869184 (exactly 16 GiB).
///
/// Exists because the governor previously had NO source on Darwin and fell back to the most
/// permissive cap it could name; macOS's memory facts were never asked for.
#[cfg(target_os = "macos")]
pub fn darwin_physical_memory_bytes() -> Option<u64> {
    let name = c"hw.memsize";
    let mut value: u64 = 0;
    let mut len: libc::size_t = std::mem::size_of::<u64>() as libc::size_t;
    // SAFETY: `name` is a NUL-terminated literal, `value`/`len` are live locals sized to
    // match, and `newp`/`newlen` are null/0 for a read-only query per sysctl(3).
    let rc = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&mut value as *mut u64).cast::<libc::c_void>(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if rc == 0 && value > 0 {
        Some(value)
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
pub fn darwin_physical_memory_bytes() -> Option<u64> {
    None
}

/// The bind request an executor may carry, read from `GUNBC_BIND_MEMORY_CGROUP_BYTES`.
///
/// DELIBERATELY NOT `GUNBC_MEMORY_BUDGET_BYTES`, and the separation is the point rather than
/// naming taste. That variable is a PLANNING request: it asks consumers to plan against a
/// smaller number and constrains no allocation (measured inert — halving it moved resident
/// memory by 0.05%), while remaining live as a refusal trigger. This one asks the KERNEL for a
/// limit, and once granted it is enforced against every allocation whether or not any consumer
/// consults it. One is a wish and the other is a wall; reading both from one variable would
/// make a run's boundedness depend on which reader looked.
///
/// Authority: `gunbc.memory_cgroup_binding` `CgroupBindRequest`.
pub const BIND_MEMORY_CGROUP_ENV: &str = "GUNBC_BIND_MEMORY_CGROUP_BYTES";

/// The cgroup this executor creates for itself when it binds. One fixed leaf under the tree
/// root, because a per-run name would leave a growing population of empty groups behind on a
/// reused runner and nothing in this repository would own deleting them.
pub const BIND_MEMORY_CGROUP_LEAF: &str = "gunbc.bound";

/// Authority: `gunbc.memory_cgroup_binding` `CgroupBindRefusalCause`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgroupBindRefusalCause {
    MemoryControllerUnavailable,
    CgroupTreeNotWritable,
    RequestNotBelowMachineMemory { requested: u64, machine: u64 },
    MachineMemoryUnreadable,
    RequestUnreadable { raw: String },
    RequestNotAboveCurrentUsage { requested: u64, current: u64 },
    CurrentUsageUnreadable,
}

/// Authority: `gunbc.memory_cgroup_binding` `CgroupBindRequest`. Absent and unreadable are
/// different answers: silence is the correct response to a question nobody asked and never to one
/// asked badly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgroupBindRequest {
    Requested { bound: u64 },
    Unreadable { raw: String },
}

/// Authority: `gunbc.memory_cgroup_binding` `CgroupBindDecision`. Four outcomes and only one of
/// them writes; `Refused` is the only one that stops the line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgroupBindDecision {
    NotRequested,
    UnnecessaryLimitAlreadyBinds { source: HostBudgetSource },
    Applicable { bound: u64 },
    Refused { cause: CgroupBindRefusalCause },
}

/// The pure decision, mirroring `gunbc.memory_cgroup_binding` `cgroup_bind_decision`. Every
/// input is a parameter so the discriminating cases are reachable from a test on any machine —
/// the same shape `resolve_host_budget` uses, and for the same reason: the states worth walling
/// are ones the CI machine does not have.
pub fn cgroup_bind_decision(
    request: Option<&CgroupBindRequest>,
    existing: Option<&HostBudgetObservation>,
    machine_memory: Option<u64>,
    current_usage: Option<u64>,
    memory_controller_available: bool,
    cgroup_tree_writable: bool,
) -> CgroupBindDecision {
    let Some(request) = request else {
        return CgroupBindDecision::NotRequested;
    };
    let bound = match request {
        CgroupBindRequest::Unreadable { raw } => {
            return CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::RequestUnreadable { raw: raw.clone() },
            }
        }
        CgroupBindRequest::Requested { bound } => *bound,
    };
    // THE NO-OP ARM MUST PROVE THE SUBJECT IT CLAIMS. `bounds_this_process` is the authority
    // already modelling which sources hard-bound this process: memory.max does, memory.high (a
    // reclaim throttle allocations may cross) and Darwin physical memory (a fact about the
    // machine) do not. Treating every observation as "already bound" suppressed a requested
    // KERNEL bind under a memory.high slice and on macOS, and told the operator a limit bound
    // them when nothing hard-bounded them at all.
    if let Some(observed) = existing {
        if observed.source.bounds_this_process() {
            return CgroupBindDecision::UnnecessaryLimitAlreadyBinds {
                source: observed.source.clone(),
            };
        }
    }
    if !memory_controller_available {
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::MemoryControllerUnavailable,
        };
    }
    if !cgroup_tree_writable {
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
        };
    }
    let Some(machine) = machine_memory else {
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::MachineMemoryUnreadable,
        };
    };
    if bound >= machine {
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::RequestNotBelowMachineMemory {
                requested: bound,
                machine,
            },
        };
    }
    let Some(current) = current_usage else {
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::CurrentUsageUnreadable,
        };
    };
    if bound <= current {
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::RequestNotAboveCurrentUsage {
                requested: bound,
                current,
            },
        };
    }
    CgroupBindDecision::Applicable { bound }
}

/// This process's own resident set in bytes, from `/proc/self/status` `VmRSS`.
///
/// Read for one question: whether the requested bound would be fatal the instant it is written.
/// A `memory.max` at or below what the process already holds kills the run before any later
/// admission arm could report the bound as too small, so "an undersized bound refuses loudly
/// later" is false at the bottom of the range unless this is checked first. Like
/// `machine_memory_total_bytes`, it never becomes anybody's budget.
pub fn self_resident_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let rest = line.strip_prefix("VmRSS:")?;
        let kib: u64 = rest.split_whitespace().next()?.parse().ok()?;
        kib.checked_mul(1024)
    })
}

/// Authority: `gunbc.memory_cgroup_binding` `cgroup_bind_refusal_cause_text`.
pub fn cgroup_bind_refusal_cause_text(cause: &CgroupBindRefusalCause) -> String {
    match cause {
        CgroupBindRefusalCause::MemoryControllerUnavailable => {
            "the memory controller is not available in this cgroup2 tree, so no memory.max can \
             be written here; the executor cannot give itself an enforceable limit and must not \
             proceed as though it had one"
                .to_string()
        }
        CgroupBindRefusalCause::CgroupTreeNotWritable => {
            "the cgroup2 tree is not writable by this process, so the requested bound cannot be \
             created; a bound that cannot be written is not a bound and this run has none"
                .to_string()
        }
        CgroupBindRefusalCause::RequestNotBelowMachineMemory { requested, machine } => format!(
            "the requested memory bound is not strictly below this machine's memory (requested \
             {requested} bytes, machine {machine} bytes), so the limit could never be reached \
             and would bound nothing while being cited as a bound; size the executor larger \
             than the bound rather than raising the bound to the executor"
        ),
        CgroupBindRefusalCause::MachineMemoryUnreadable => {
            "this machine's memory is unreadable, so whether the requested bound could bind at \
             all is UNKNOWN; refuse rather than write a limit that may be a decoration"
                .to_string()
        }
        CgroupBindRefusalCause::RequestUnreadable { raw } => format!(
            "a memory bound was REQUESTED and cannot be read as a byte count ({raw}); a request \
             that cannot be understood is refused rather than treated as no request, because \
             proceeding unbounded is precisely what the operator asked not to happen"
        ),
        CgroupBindRefusalCause::RequestNotAboveCurrentUsage { requested, current } => format!(
            "the requested memory bound is not above what this process ALREADY holds (requested \
             {requested} bytes, currently resident {current} bytes), so writing it would kill \
             the run at the moment it takes effect, before any later admission check could \
             report anything; this says nothing about whether the bound is large enough for the \
             work, only that it is not instantly fatal"
        ),
        CgroupBindRefusalCause::CurrentUsageUnreadable => {
            "this process's own resident size is unreadable, so whether the requested bound \
             would be instantly fatal is UNKNOWN; refuse rather than write a limit that may kill \
             the run before it can diagnose anything"
                .to_string()
        }
    }
}

/// The refusal rendering. `None` on every non-refusing arm, so a caller that logs a cause
/// unconditionally cannot fabricate one for a decision that did not refuse.
/// Authority: `gunbc.memory_cgroup_binding` `cgroup_bind_decision_refusal_text`.
pub fn cgroup_bind_refusal_diagnostic(decision: &CgroupBindDecision) -> Option<String> {
    match decision {
        CgroupBindDecision::Refused { cause } => Some(format!(
            "MemoryCgroupBindRefused: {}",
            cgroup_bind_refusal_cause_text(cause)
        )),
        _ => None,
    }
}

/// The announcement, so a log distinguishes a run that bound itself from one that was already
/// bounded from one that asked for neither — three states that all continue and are otherwise
/// indistinguishable downstream.
/// Authority: `gunbc.memory_cgroup_binding` `cgroup_bind_decision_note`.
pub fn cgroup_bind_note(decision: &CgroupBindDecision) -> String {
    match decision {
        CgroupBindDecision::NotRequested => "memory-cgroup-bind: not requested".to_string(),
        CgroupBindDecision::UnnecessaryLimitAlreadyBinds { source } => format!(
            "memory-cgroup-bind: unnecessary, a limit already binds this process: {}",
            source.label()
        ),
        CgroupBindDecision::Applicable { bound } => {
            format!("memory-cgroup-bind: applying memory.max={bound}")
        }
        CgroupBindDecision::Refused { cause } => {
            format!(
                "MemoryCgroupBindRefused: {}",
                cgroup_bind_refusal_cause_text(cause)
            )
        }
    }
}

/// `/proc/meminfo` MemTotal in bytes.
///
/// READ FOR ONE QUESTION ONLY and it is not a budget question: whether a requested bound could
/// bind at all. `resolve_host_budget` deleted its meminfo arms deliberately — a host-shared
/// reading standing in for a process-scoped bound is DESIGN §5's absorbing substitution, and it
/// is what got `main_wet` SIGKILLed with no diagnostic. Nothing here reverses that: this number
/// never becomes anybody's budget, and when it is absent the bind REFUSES rather than assuming
/// the request fits.
pub fn machine_memory_total_bytes() -> Option<u64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    meminfo.lines().find_map(|line| {
        let rest = line.strip_prefix("MemTotal:")?;
        let kib: u64 = rest.split_whitespace().next()?.parse().ok()?;
        kib.checked_mul(1024)
    })
}

/// Whether the cgroup2 tree root lists the `memory` controller.
fn cgroup_memory_controller_available() -> bool {
    std::fs::read_to_string("/sys/fs/cgroup/cgroup.controllers")
        .map(|s| s.split_whitespace().any(|c| c == "memory"))
        .unwrap_or(false)
}

/// Whether this process may create the child group this bind would use, and whether THIS call is
/// what created it. Probed by performing the creation rather than by inspecting permission bits:
/// the question is whether the write succeeds, and anything short of performing it is a
/// prediction.
///
/// AN ALREADY-EXISTING LEAF IS NOT EVIDENCE OF ANYTHING, which is the correction this signature
/// carries (reported by tidy-swift-334 on #10465). The previous form mapped `AlreadyExists` to
/// `true`, so a run that created the leaf and then refused left residue that the NEXT run read
/// back as proof of its own writability — a prediction sourced from an earlier failure instead
/// of from permission bits, and blind to permissions revoked in between. Existence is therefore
/// reported as `created: false` and settles nothing; the only writability evidence this module
/// accepts is a write it performed, and the `memory.max` write that follows is the one that
/// decides.
fn cgroup_leaf_create(dir: &Path) -> Option<bool> {
    match std::fs::create_dir(dir) {
        Ok(()) => Some(true),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Some(false),
        Err(_) => None,
    }
}

/// Undo a leaf THIS call created. Only ever asked about a directory `cgroup_leaf_create` reported
/// creating, so a group that was already there — another run's, or an operator's — is never
/// removed by a refusal.
fn cgroup_leaf_remove_if_created(dir: &Path, created: bool) {
    if created {
        let _ = std::fs::remove_dir(dir);
    }
}

/// Create the bound, move this process into it, and return the executed decision.
///
/// THE ORDER IS THE SAFETY PROPERTY, AND IT IS STRONGER THAN THE FIRST REVISION CLAIMED. The
/// decision is taken first from an observation read through the ordinary budget path, so a
/// machine that already binds this process is never written to; NO ARM THAT REFUSES TOUCHES THE
/// CGROUP TREE AT ALL, and any failure after the first write unwinds the group this call
/// created, so a refusal leaves the tree exactly as it found it; and afterwards every consumer
/// still reads its budget through `read_host_budget_resolution` exactly as before. This
/// function supplies no number to anybody — a successful bind is invisible except as an
/// observation that now exists, which is what keeps `gunbc.host_budget_source` the single
/// authority for what the limit IS.
///
/// AND IT DOES NOT DISARM ANY REFUSAL. The stall verdict still watches this process's own major
/// faults and user-CPU share; the whole-corpus admission still compares the resulting budget
/// against measured demand. A bound that is too small refuses through those arms, loudly, which
/// is the correct ending — this function's job is to make a bound EXIST, never to make one fit.
///
/// Authority: `gunbc.memory_cgroup_binding`.
pub fn apply_memory_cgroup_bind() -> CgroupBindDecision {
    // ABSENT AND UNREADABLE ARE DIFFERENT ANSWERS. The first revision read this through
    // `.ok().and_then(|s| s.trim().parse().ok())`, so an empty, nonnumeric or overflowing value
    // became `None` and `None` meant "not requested" — an operator who asked to be bounded, with
    // a malformed value, proceeded unbounded and was told nothing (review 5117628699).
    let request = match std::env::var(BIND_MEMORY_CGROUP_ENV) {
        Err(_) => None,
        Ok(raw) => Some(match raw.trim().parse::<u64>() {
            Ok(bound) => CgroupBindRequest::Requested { bound },
            Err(_) => CgroupBindRequest::Unreadable { raw },
        }),
    };
    if request.is_none() {
        return CgroupBindDecision::NotRequested;
    }
    let existing = match read_host_budget_resolution() {
        HostBudgetResolution::Resolved { observation, .. } => Some(observation),
        _ => None,
    };
    let controller = cgroup_memory_controller_available();

    // ASK THE DECISION WHAT IT WOULD ANSWER IF THE TREE WERE WRITABLE, BEFORE TOUCHING THE TREE.
    // Writability is the one input that cannot be read without performing a write; every other
    // input can refuse on its own. If the answer is already not `Applicable` it does not depend
    // on writability, so no probe happens and the tree is left untouched.
    let hypothetical = cgroup_bind_decision(
        request.as_ref(),
        existing.as_ref(),
        machine_memory_total_bytes(),
        self_resident_bytes(),
        controller,
        true,
    );
    let CgroupBindDecision::Applicable { bound } = hypothetical else {
        return hypothetical;
    };

    // ONLY NOW IS THE TREE TOUCHED, and every failure from here unwinds what this call created.
    // A REFUSAL LEAVES THE TREE EXACTLY AS IT FOUND IT.
    let leaf = Path::new("/sys/fs/cgroup").join(BIND_MEMORY_CGROUP_LEAF);
    let Some(created) = cgroup_leaf_create(&leaf) else {
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
        };
    };

    // THE `+memory` ERROR IS DISCARDED DELIBERATELY AND THE REASON IS THE WHOLE POINT: this write
    // fails with EBUSY when the controller is ALREADY enabled on the subtree, which is a success
    // condition wearing an error's clothes, and it can fail for permissions, which is not. The
    // two are not distinguishable from the error alone — so nothing is CONCLUDED from this call
    // in either direction. What decides is the read-back below: the controller is enabled if and
    // only if `memory.max` accepts the value and reports it back.
    let _ = std::fs::write("/sys/fs/cgroup/cgroup.subtree_control", "+memory");

    // A FAILED WRITE IS A REFUSAL, NOT A DEGRADED SUCCESS, AND SUCCESS IS PROVEN RATHER THAN
    // ASSUMED (review 5117628699). Every arm below leaves the process exactly as unbounded as it
    // was, so reporting anything but a refusal would be the escape hatch this module closes.
    if std::fs::write(leaf.join("memory.max"), bound.to_string()).is_err() {
        cgroup_leaf_remove_if_created(&leaf, created);
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
        };
    }
    // READ THE LIMIT BACK. A write that returned Ok is not proof the kernel took the value: the
    // claim being published is that this process now runs under `bound`, and only the file says
    // so.
    if read_cgroup_u64(&leaf, "memory.max") != Some(bound) {
        cgroup_leaf_remove_if_created(&leaf, created);
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
        };
    }
    if std::fs::write(leaf.join("cgroup.procs"), std::process::id().to_string()).is_err() {
        cgroup_leaf_remove_if_created(&leaf, created);
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
        };
    }
    // AND PROVE MEMBERSHIP, for the same reason: a limit this process is not IN bounds nothing.
    // `binding_cap_cgroup_dir` walks `/proc/self/cgroup` leaf-to-root, so it answers about this
    // process rather than about the file that was written.
    if binding_cap_cgroup_dir().is_none() {
        cgroup_leaf_remove_if_created(&leaf, created);
        return CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
        };
    }
    CgroupBindDecision::Applicable { bound }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_budget_below_minimum_footprint_refuses_doomed_class() {
        let doomed = floor_budget_below_minimum_footprint(Some(6_987_137_024));
        assert!(doomed.is_some());
        assert!(
            doomed.unwrap().contains("FloorBudgetBelowMinimumFootprint"),
            "typed refusal variant"
        );
        assert!(floor_budget_below_minimum_footprint(Some(
            DECLARED_FLOOR_MINIMUM_VIABLE_ARMED_BUDGET_BYTES
        ))
        .is_none());
        assert!(floor_budget_below_minimum_footprint(Some(
            DECLARED_FLOOR_MINIMUM_VIABLE_ARMED_BUDGET_BYTES - 1
        ))
        .is_some());
        assert!(floor_budget_below_minimum_footprint(None).is_none());
        assert!(
            floor_budget_below_minimum_footprint(Some(DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES))
                .is_none()
        );
    }

    /// THE DISCRIMINATING RED for the 2026-08-30 BuildBuddy SIGKILL: every observation absent
    /// — what that runner presents, `/proc/self/cgroup` reading `0::/` with no `memory.max` or
    /// `memory.high` anywhere under `/sys/fs/cgroup` — must refuse and carry NO number. Before
    /// this change the same observations produced a budget: MemAvailable (a host reading)
    /// capped at the fleet's declared slot line (a declaration about different machines).
    ///
    /// The refusal must name what was unreadable and explain that the env var can only narrow
    /// an observation — since a stop that does not say how to restart cannot be
    /// analyzed.
    #[test]
    #[allow(non_snake_case)]
    fn RED_no_readable_bound_refuses_and_carries_no_number() {
        let refused = resolve_host_budget(None, None, None, None);
        assert!(matches!(refused, HostBudgetResolution::Unreadable { .. }));
        assert_eq!(refused.bytes(), None);
        assert_eq!(refused.degraded_source(), None);
        let label = refused.label();
        assert!(label.contains("memory.high"), "{label}");
        assert!(label.contains("memory.max"), "{label}");
        assert!(label.contains("GUNBC_MEMORY_BUDGET_BYTES"), "{label}");
    }

    /// THE POSITIVE CONTROLS, without which the RED above is satisfied by a resolver refusing
    /// everything. Observed lines are admitted at the tightest applicable value; an operator
    /// request can narrow that value but cannot widen it or establish one by itself.
    #[test]
    fn a_readable_bound_is_admitted_at_its_own_value_and_source() {
        let env = resolve_host_budget(Some(10_737_418_240), None, None, None);
        assert!(matches!(
            env,
            HostBudgetResolution::DeclaredUnverified { .. }
        ));
        // The request remains usable by planning consumers (cache entry count and
        // concurrency); the variant prevents any consumer from calling it observed or enforced.
        assert_eq!(env.bytes(), Some(10_737_418_240));

        let high = resolve_host_budget(
            None,
            Some(("/sys/fs/cgroup/runner.slice".to_string(), 8_589_934_592)),
            Some(("/sys/fs/cgroup/runner.slice".to_string(), 9_663_676_416)),
            None,
        );
        assert_eq!(high.bytes(), Some(8_589_934_592));
        assert_eq!(high.degraded_source(), Some(false));
        assert!(high.label().contains("memory.high"));

        let max = resolve_host_budget(
            None,
            None,
            Some(("/sys/fs/cgroup/runner.slice".to_string(), 9_663_676_416)),
            None,
        );
        assert_eq!(max.bytes(), Some(9_663_676_416));
        assert!(max.label().contains("memory.max"));

        // A declaration cannot widen an observed throttle.
        let both = resolve_host_budget(
            Some(10_737_418_240),
            Some(("/sys/fs/cgroup/runner.slice".to_string(), 8_589_934_592)),
            None,
            None,
        );
        assert_eq!(both.bytes(), Some(8_589_934_592));

        // It can request a narrower planning ceiling once an observed limit verifies the
        // process is actually bounded.
        let narrowed = resolve_host_budget(
            Some(5_368_709_120),
            Some(("/sys/fs/cgroup/runner.slice".to_string(), 8_589_934_592)),
            None,
            None,
        );
        assert_eq!(narrowed.bytes(), Some(5_368_709_120));
    }

    /// Darwin's physical-memory read stays a source and stays DEGRADED, and it is the only
    /// arm that is both. It is reachable only where `darwin_physical_memory_bytes` answers,
    /// i.e. only on a kernel with no cgroups — so a host-shared reading can serve as the
    /// budget exactly where no private bound could have been expressed, and nowhere else.
    #[test]
    fn darwin_physical_memory_is_a_degraded_source_and_the_only_one() {
        let darwin = resolve_host_budget(None, None, None, Some(17_179_869_184));
        assert_eq!(darwin.bytes(), Some(17_179_869_184));
        assert_eq!(darwin.degraded_source(), Some(true));
        assert_eq!(darwin.label(), "sysctl hw.memsize");
        assert!(!HostBudgetSource::DarwinPhysicalMemory.bounds_this_process());
        for bounding in [HostBudgetSource::CgroupMemoryMax {
            cgroup_dir: "/sys/fs/cgroup".to_string(),
        }] {
            assert!(bounding.bounds_this_process(), "{bounding:?}");
            assert!(!bounding.is_degraded(), "{bounding:?}");
        }
        let high = HostBudgetSource::CgroupMemoryHigh {
            cgroup_dir: "/sys/fs/cgroup".to_string(),
        };
        assert!(!high.bounds_this_process());
        assert!(!high.is_degraded());
    }

    /// No arm reports a `/proc` path it did not read. The fabricated-provenance bug this
    /// mirrors (`(None, "/proc/meminfo MemTotal")` on a machine with no `/proc`) is now
    /// unwritable rather than merely absent: the label is a total match on the discriminant,
    /// and no discriminant names meminfo.
    #[test]
    fn no_source_label_names_a_file_the_resolver_did_not_read() {
        for source in [
            HostBudgetSource::CgroupMemoryHigh {
                cgroup_dir: "/sys/fs/cgroup".to_string(),
            },
            HostBudgetSource::CgroupMemoryMax {
                cgroup_dir: "/sys/fs/cgroup".to_string(),
            },
            HostBudgetSource::DarwinPhysicalMemory,
        ] {
            assert!(!source.label().contains("/proc/meminfo"), "{source:?}");
        }
        assert!(!resolve_host_budget(None, None, None, None)
            .label()
            .contains("/proc/meminfo"));
    }

    /// THE COMMITTED PROJECTION, read as the compile reads it. Pinning the tests to the generated
    /// file rather than to a retyped figure is what keeps the seed joined to the row: a re-measure
    /// moves the row, regen moves the file, and these tests follow without an edit.
    const PUBLIC_DEMAND_PROJECTION: &str =
        include_str!("../../../../tools/whole_corpus_compile_measured_root_demands.json");

    fn public_root() -> WholeCorpusCompileRootIdentity {
        WholeCorpusCompileRootIdentity {
            repository: "gunbc".to_string(),
            primary_root: "dag".to_string(),
            dependency_pools: vec!["src/v2".to_string()],
        }
    }

    fn public_read() -> WholeCorpusCompileDemandsRead {
        parse_whole_corpus_compile_demand_projection(
            "tools/whole_corpus_compile_measured_root_demands.json",
            PUBLIC_DEMAND_PROJECTION,
        )
    }

    fn public_peak() -> u64 {
        match public_read() {
            WholeCorpusCompileDemandsRead::Read { rows, .. } => {
                rows.iter()
                    .find(|r| r.root == public_root())
                    .expect("the committed projection carries the public root's row")
                    .peak_bytes
            }
            other => panic!("committed projection must read: {other:?}"),
        }
    }

    fn admit(
        budget: Option<u64>,
        root: &WholeCorpusCompileRootIdentity,
    ) -> WholeCorpusCompileAdmission {
        whole_corpus_compile_admission(&cgroup_budget(budget), root, &public_read())
    }

    /// A budget observed on a cgroup memory.high line, or unreadable when absent.
    fn cgroup_budget(bytes: Option<u64>) -> HostBudgetResolution {
        resolve_host_budget(
            None,
            bytes.map(|b| ("/sys/fs/cgroup/runner.slice".to_string(), b)),
            None,
            None,
        )
    }

    /// The SIGKILLed run's budget and the CI runner slot's `memory.high` are both refused below the
    /// public root's measured peak; twins of the `.dag` witness arms of the same names.
    #[test]
    fn whole_corpus_compile_refuses_the_budget_that_was_sigkilled_and_refuses_the_ci_runner() {
        let doomed = whole_corpus_compile_admission(
            &cgroup_budget(Some(5_269_094_400)),
            &public_root(),
            &public_read(),
        );
        assert!(matches!(
            doomed,
            WholeCorpusCompileAdmission::RefusedBudgetBelowMeasuredDemand { .. }
        ));
        let msg = whole_corpus_compile_refusal_diagnostic(&doomed).expect("refusal must diagnose");
        assert!(msg.contains("WholeCorpusCompileBudgetBelowMeasuredDemand"));
        assert!(msg.contains("cgroup memory.high (/sys/fs/cgroup/runner.slice)"));
        assert!(msg.contains("--entry"));
        // THE OLD EFFECTIVE SLOT STILL REFUSES, AND THIS IS THE CONTROL THAT MUST SURVIVE THE
        // 2026-09-12 SIZING CHANGE. 16106127360 is what every runner slot in the fleet actually
        // carries today; it sits below the measured demand and is refused, which is the property
        // this test was written for and the one a desired-row edit must not quietly retire.
        let old_effective_slot = admit(Some(16_106_127_360), &public_root());
        assert!(matches!(
            old_effective_slot,
            WholeCorpusCompileAdmission::RefusedBudgetBelowMeasuredDemand { .. }
        ));
        assert!(whole_corpus_compile_refusal_diagnostic(&old_effective_slot).is_some());

        // THE DECLARED ROW NOW ADMITS, AND THAT IS A STATEMENT ABOUT A DESIRED CONFIGURATION
        // RATHER THAN ABOUT ANY HOST. `DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES` mirrors
        // `gunbc.runner_slot_allocation`'s desired row, which moved to 25 GiB; 25 GiB clears the
        // 16 GiB measured demand, so the admission flips from refusal to admission BY POLICY.
        //
        // THE THRESHOLD IS NOT TOUCHED. Lowering it to keep the old assertion green would be
        // choosing a rule to preserve an answer, which is the reverse of an oracle.
        //
        // WHAT THIS DOES NOT ESTABLISH: that any host can honour it. No fleet member carries this
        // limit -- the control above is the live figure -- so a run admitted on this basis would
        // still meet the old effective ceiling.
        let declared_slot = admit(Some(DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES), &public_root());
        assert!(whole_corpus_compile_refusal_diagnostic(&declared_slot).is_none());
    }

    /// The discriminating red for review 66337: a killed or failing child must not read as
    /// success to anyone consulting the parent's exit status.
    #[test]
    fn measure_root_demand_parent_exits_nonzero_unless_the_child_completed_cleanly() {
        let run = || RootDemandMeasurementRun {
            root: public_root(),
            limit_bytes: 1,
            limit_source: "test".to_string(),
            measured_on_host: "test".to_string(),
            instrument_run: "test".to_string(),
            census: parse_root_demand_measurement_census(""),
        };
        assert_eq!(
            root_demand_measurement_exit_code(&root_demand_measurement_receipt(
                run(),
                RootDemandMeasurementWait::Exited(0),
                1
            )),
            0
        );
        assert_eq!(
            root_demand_measurement_exit_code(&root_demand_measurement_receipt(
                run(),
                RootDemandMeasurementWait::Exited(1),
                1
            )),
            1
        );
        assert_eq!(
            root_demand_measurement_exit_code(&root_demand_measurement_receipt(
                run(),
                RootDemandMeasurementWait::Signaled(SIGKILL_SIGNAL),
                1
            )),
            1
        );
        assert_eq!(
            root_demand_measurement_exit_code(&root_demand_measurement_receipt(
                run(),
                RootDemandMeasurementWait::Signaled(15),
                1
            )),
            1
        );
    }

    /// Positive control and its one-byte-below red, at the public root's own row.
    #[test]
    fn whole_corpus_compile_admission_is_tight_at_the_measured_demand() {
        let peak = public_peak();
        assert!(
            whole_corpus_compile_refusal_diagnostic(&admit(Some(peak), &public_root())).is_none()
        );
        assert!(matches!(
            admit(Some(peak - 1), &public_root()),
            WholeCorpusCompileAdmission::RefusedBudgetBelowMeasuredDemand { required_bytes, .. }
                if required_bytes == peak
        ));
    }

    /// A root with no row refuses at a budget four times the public peak, naming the recipe —
    /// never admitted on the public root's number. The same budget admits the public root.
    #[test]
    fn whole_corpus_compile_unmeasured_root_refuses_naming_the_recipe() {
        let ample = Some(public_peak() * 4);
        assert!(matches!(
            admit(ample, &public_root()),
            WholeCorpusCompileAdmission::Admitted { .. }
        ));
        for root in [
            WholeCorpusCompileRootIdentity {
                dependency_pools: vec![],
                ..public_root()
            },
            WholeCorpusCompileRootIdentity {
                dependency_pools: vec!["src/v2".to_string(), "extdeps".to_string()],
                ..public_root()
            },
            WholeCorpusCompileRootIdentity {
                primary_root: "src/v2".to_string(),
                dependency_pools: vec!["dag".to_string()],
                ..public_root()
            },
        ] {
            let refused = admit(ample, &root);
            assert!(
                matches!(
                    &refused,
                    WholeCorpusCompileAdmission::RefusedUnmeasuredRoot {
                        cause: WholeCorpusCompileUnmeasuredCause::NoRowForRoot { .. },
                        ..
                    }
                ),
                "{root:?}: {refused:?}"
            );
            let msg =
                whole_corpus_compile_refusal_diagnostic(&refused).expect("refusal must diagnose");
            assert!(msg.contains("WholeCorpusCompileUnmeasuredRoot"));
            assert!(msg.contains(&format!("--source-root {}", root.primary_root)));
            assert!(
                msg.contains("gunbc measure-root-demand --repository"),
                "{msg}"
            );
        }
    }

    /// The public projection handed to another repository's compile of the SAME root spelling is
    /// refused as another repository's projection — the defect re-entered through the location.
    #[test]
    fn whole_corpus_compile_another_repositorys_projection_refuses() {
        let private_spelling = WholeCorpusCompileRootIdentity {
            repository: "gunbc-private".to_string(),
            ..public_root()
        };
        let refused = admit(Some(public_peak() * 4), &private_spelling);
        assert!(matches!(
            &refused,
            WholeCorpusCompileAdmission::RefusedDemandsForAnotherRepository { projected_repositories, .. }
                if projected_repositories == &vec!["gunbc".to_string()]
        ));
        assert!(whole_corpus_compile_refusal_diagnostic(&refused)
            .expect("refusal must diagnose")
            .contains("WholeCorpusCompileDemandsForAnotherRepository"));
    }

    /// No declared location, an unreadable one, and malformed or wrong-schema bytes each refuse as
    /// unmeasured with their own cause, before the budget is consulted.
    #[test]
    fn whole_corpus_compile_undeclared_or_unreadable_projection_refuses() {
        let ample = Some(public_peak() * 4);
        let root = public_root();
        assert!(matches!(
            whole_corpus_compile_admission(
                &cgroup_budget(ample),
                &root,
                &read_whole_corpus_compile_demands(None)
            ),
            WholeCorpusCompileAdmission::RefusedUnmeasuredRoot {
                cause: WholeCorpusCompileUnmeasuredCause::NoDemandsProjectionDeclared,
                ..
            }
        ));
        let missing = read_whole_corpus_compile_demands(Some("/nonexistent/demands.json"));
        assert!(matches!(
            whole_corpus_compile_admission(&cgroup_budget(ample), &root, &missing),
            WholeCorpusCompileAdmission::RefusedUnmeasuredRoot {
                cause: WholeCorpusCompileUnmeasuredCause::DemandsProjectionNotRead { .. },
                ..
            }
        ));
        for bad in [
            "not json",
            r#"{"schema":"other/1","rows":[]}"#,
            r#"{"schema":"gunbc.whole_corpus_compile_demand_projection/1","rows":[{"repository":"gunbc"}]}"#,
        ] {
            assert!(
                matches!(
                    parse_whole_corpus_compile_demand_projection("x", bad),
                    WholeCorpusCompileDemandsRead::Unreadable { .. }
                ),
                "{bad}"
            );
        }
        // A budget that cannot be read does not rescue a root with no row: the row is asked first.
        assert!(matches!(
            whole_corpus_compile_admission(
                &cgroup_budget(None),
                &WholeCorpusCompileRootIdentity {
                    repository: "gunbc".into(),
                    primary_root: "x".into(),
                    dependency_pools: vec![]
                },
                &public_read()
            ),
            WholeCorpusCompileAdmission::RefusedUnmeasuredRoot { .. }
        ));
    }

    /// A DECLARED budget at four times the public peak refuses a measured root exactly as an
    /// unreadable one does. Before this change the seed took `(Option<u64>, label)` and admitted it.
    #[test]
    fn whole_corpus_compile_declared_budget_refuses_as_the_model_does() {
        let declared = resolve_host_budget(Some(public_peak() * 4), None, None, None);
        assert!(matches!(
            declared,
            HostBudgetResolution::DeclaredUnverified { .. }
        ));
        assert!(matches!(
            whole_corpus_compile_admission(&declared, &public_root(), &public_read()),
            WholeCorpusCompileAdmission::RefusedBudgetUnreadable { .. }
        ));
    }

    fn unmeasured_private_root() -> WholeCorpusCompileRootIdentity {
        WholeCorpusCompileRootIdentity {
            repository: "gunbc-private".to_string(),
            primary_root: "strategy".to_string(),
            dependency_pools: vec!["dag".to_string()],
        }
    }

    /// Positive control and its reds: an observed memory.max admits and its value is the receipt's
    /// limit; memory.high alone refuses naming that it never kills; no cgroup limit refuses.
    #[test]
    fn root_demand_measurement_needs_memory_max() {
        let root = unmeasured_private_root();
        match root_demand_measurement_admission(
            &RootDemandMeasurementLimitReading::MemoryMaxBindsProcess {
                cgroup_dir: "/sys/fs/cgroup/session.slice".to_string(),
                bytes: 6_442_450_944,
            },
            &root,
        ) {
            RootDemandMeasurementAdmission::Admitted(a) => {
                assert_eq!(a.limit_bytes(), 6_442_450_944);
                assert!(a.limit_source().contains("memory.max"));
                assert_eq!(a.root(), &root);
            }
            other => panic!("an observed memory.max must admit: {other:?}"),
        }
        let high_only = root_demand_measurement_admission(
            &RootDemandMeasurementLimitReading::MemoryHighOnly {
                cgroup_dir: "/sys/fs/cgroup/runner.slice".to_string(),
                high_bytes: 8_589_934_592,
            },
            &root,
        );
        let msg = root_demand_measurement_refusal_diagnostic(&high_only)
            .expect("memory.high alone must refuse");
        assert!(
            msg.contains("throttles and never kills") && msg.contains("needs memory.max"),
            "{msg}"
        );
        let none = root_demand_measurement_admission(
            &RootDemandMeasurementLimitReading::NoCgroupMemoryLimit,
            &root,
        );
        assert!(root_demand_measurement_refusal_diagnostic(&none)
            .expect("no limit must refuse")
            .contains("RootDemandMeasurementRefusedNoEnforceableLimit"));
    }

    fn fixture_run(census: Option<RootDemandMeasurementCensus>) -> RootDemandMeasurementRun {
        RootDemandMeasurementRun {
            root: unmeasured_private_root(),
            limit_bytes: 6_442_450_944,
            limit_source: "cgroup memory.max (/sys/fs/cgroup/session.slice)".to_string(),
            measured_on_host: "fixture-host".to_string(),
            instrument_run: "fixture".to_string(),
            census,
        }
    }

    /// A kill at the limit is the typed EXCEEDED receipt carrying the limit as a lower bound; any
    /// other signal is TERMINATED; an exit is COMPLETED with its status and peak. The receipt's JSON
    /// carries no artifact and no verdict field.
    #[test]
    fn root_demand_measurement_receipt_types_the_kill() {
        let killed = root_demand_measurement_receipt(
            fixture_run(None),
            RootDemandMeasurementWait::Signaled(SIGKILL_SIGNAL),
            6_400_000_000,
        );
        assert!(matches!(
            killed,
            RootDemandMeasurementReceipt::Exceeded { .. }
        ));
        let json = root_demand_measurement_receipt_json(&killed);
        assert!(
            json.contains("\"arm\":\"exceeded\"")
                && json.contains("\"demand_exceeds_bytes\":6442450944")
        );
        assert!(
            !json.contains("peak_bytes"),
            "an exceeded run has no measured peak: {json}"
        );
        assert!(matches!(
            root_demand_measurement_receipt(
                fixture_run(None),
                RootDemandMeasurementWait::Signaled(15),
                1
            ),
            RootDemandMeasurementReceipt::Terminated { signal: 15, .. }
        ));
        let done = root_demand_measurement_receipt(
            fixture_run(Some(RootDemandMeasurementCensus {
                source_count: 548,
                source_bytes: 9_000_000,
            })),
            RootDemandMeasurementWait::Exited(0),
            4_000_000_000,
        );
        let json = root_demand_measurement_receipt_json(&done);
        assert!(
            json.contains("\"arm\":\"completed\"")
                && json.contains("\"peak_bytes\":4000000000")
                && json.contains("\"source_count\":548")
        );
        for forbidden in ["artifact", "files", "verdict", "emitted"] {
            assert!(
                !json.contains(forbidden),
                "receipt must carry no {forbidden}: {json}"
            );
        }
    }

    #[test]
    fn root_demand_measurement_census_line_round_trips() {
        let out = format!(
            "noise\n{} 548 9000000\nmore\n",
            ROOT_DEMAND_MEASUREMENT_CENSUS_PREFIX
        );
        assert_eq!(
            parse_root_demand_measurement_census(&out),
            Some(RootDemandMeasurementCensus {
                source_count: 548,
                source_bytes: 9_000_000
            })
        );
        assert_eq!(parse_root_demand_measurement_census("no census here"), None);
    }

    /// THE DISCRIMINATING RED for the 2026-08-30 default-VM treadmill: a window shaped like
    /// the measured specimen — two minutes of wall spent almost entirely refaulting — must
    /// refuse with the typed class name, the observed rate against the declared line, the
    /// counter it was read from, and the remedy. Before this change no arm existed: the
    /// budget was readable and admitted, and the run held for hours with no diagnostic.
    #[test]
    #[allow(non_snake_case)]
    fn RED_a_treadmill_window_refuses_and_names_the_pressure() {
        let verdict = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: 120_000,
            major_faults_in_window: 120_000,
            self_user_cpu_ms_in_window: 4_800,
            cache_evictions_in_window: 4_213,
            cache_readmissions_in_window: 388,
        });
        assert!(matches!(
            verdict,
            MemoryStallVerdict::StallRefusedPageThrash {
                major_faults_per_minute: 60_000,
                self_cpu_share_basis_points: 400,
                ..
            }
        ));
        let text = memory_stall_refusal_pressure_text(&verdict);
        assert!(text.contains("MemoryStallRefusedPageThrash"), "{text}");
        assert!(text.contains("/proc/self/stat"), "{text}");
        assert!(text.contains("--entry"), "{text}");
        assert!(text.contains("readmissions 388"), "{text}");
    }

    /// The measured specimen the rate-only form of this verdict wrongly refused on its own
    /// first CI execution (run 33319823294, required-witnesses-floor): 163296 major faults
    /// over 788140 ms — 12431/minute, over the rate line — during one 13-minute CPU-bound
    /// typecheck under srv3's memory.high reclaim throttle, a configuration that completes
    /// green on main daily. It must ADMIT, or the floor's own slow phases red every
    /// crowded runner.
    #[test]
    fn the_ci_runner_progressing_under_pressure_specimen_is_admitted() {
        let verdict = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: 788_140,
            major_faults_in_window: 163_296,
            self_user_cpu_ms_in_window: 552_000,
            cache_evictions_in_window: 0,
            cache_readmissions_in_window: 0,
        });
        assert_eq!(
            verdict,
            MemoryStallVerdict::ProgressUnderMemoryAdmissible {
                major_faults_per_minute: 12_431,
                self_cpu_share_basis_points: 7003,
            }
        );
    }

    /// THE POSITIVE CONTROLS, without which the RED is satisfied by a verdict that refuses
    /// everything — and the acceptance distinction itself: a slow-but-progressing resolve
    /// (an hour of wall, fault counter flat) and a thrashing one are different states, and
    /// no amount of elapsed time alone may refuse. A burst below the minimum window is an
    /// OPEN window, not a verdict in either direction; the line is tight at the declared
    /// rate (at the line admits, one more refuses); non-refusing arms carry no pressure
    /// prose.
    #[test]
    fn a_slow_but_progressing_resolve_is_admitted_however_long_it_runs() {
        let slow = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: 3_600_000,
            major_faults_in_window: 60,
            self_user_cpu_ms_in_window: 3_200_000,
            cache_evictions_in_window: 0,
            cache_readmissions_in_window: 0,
        });
        assert_eq!(
            slow,
            MemoryStallVerdict::ProgressUnderMemoryAdmissible {
                major_faults_per_minute: 1,
                self_cpu_share_basis_points: 8888,
            }
        );
        assert_eq!(memory_stall_refusal_pressure_text(&slow), "");

        let burst = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: MEMORY_STALL_VERDICT_WINDOW_MINIMUM_WALL_MS - 1,
            major_faults_in_window: 1_000_000,
            self_user_cpu_ms_in_window: 0,
            cache_evictions_in_window: 0,
            cache_readmissions_in_window: 0,
        });
        assert!(matches!(burst, MemoryStallVerdict::StallWindowOpen { .. }));
        assert_eq!(memory_stall_refusal_pressure_text(&burst), "");
    }

    /// Tight at BOTH declared lines: with CPU share pinned to zero, a fault count at the
    /// rate line admits and one more refuses; with the rate pinned far over its line, a
    /// CPU share at the floor admits and one point under refuses.
    #[test]
    fn the_refusal_is_tight_at_both_declared_lines() {
        let at_rate_line = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: 60_000,
            major_faults_in_window: MEMORY_STALL_MAJOR_FAULT_RATE_PER_MINUTE_THRESHOLD,
            self_user_cpu_ms_in_window: 0,
            cache_evictions_in_window: 0,
            cache_readmissions_in_window: 0,
        });
        assert!(matches!(
            at_rate_line,
            MemoryStallVerdict::ProgressUnderMemoryAdmissible { .. }
        ));
        let over_rate_line = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: 60_000,
            major_faults_in_window: MEMORY_STALL_MAJOR_FAULT_RATE_PER_MINUTE_THRESHOLD + 1,
            self_user_cpu_ms_in_window: 0,
            cache_evictions_in_window: 0,
            cache_readmissions_in_window: 0,
        });
        assert!(matches!(
            over_rate_line,
            MemoryStallVerdict::StallRefusedPageThrash { .. }
        ));
        let at_cpu_floor = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: 100_000,
            major_faults_in_window: 100_000,
            self_user_cpu_ms_in_window: MEMORY_STALL_PROGRESS_CPU_SHARE_FLOOR_BASIS_POINTS * 10,
            cache_evictions_in_window: 0,
            cache_readmissions_in_window: 0,
        });
        assert!(matches!(
            at_cpu_floor,
            MemoryStallVerdict::ProgressUnderMemoryAdmissible { .. }
        ));
        let under_cpu_floor = memory_stall_verdict(MemoryStallObservation {
            window_wall_ms: 100_000,
            major_faults_in_window: 100_000,
            self_user_cpu_ms_in_window: MEMORY_STALL_PROGRESS_CPU_SHARE_FLOOR_BASIS_POINTS * 10
                - 10,
            cache_evictions_in_window: 0,
            cache_readmissions_in_window: 0,
        });
        assert!(matches!(
            under_cpu_floor,
            MemoryStallVerdict::StallRefusedPageThrash { .. }
        ));
    }

    /// A measured root whose budget is unreadable refuses rather than admitting against the widest
    /// cap available.
    #[test]
    fn whole_corpus_compile_unreadable_budget_refuses_rather_than_widening() {
        let unreadable = admit(None, &public_root());
        assert!(matches!(
            unreadable,
            WholeCorpusCompileAdmission::RefusedBudgetUnreadable { .. }
        ));
        assert!(whole_corpus_compile_refusal_diagnostic(&unreadable)
            .expect("refusal must diagnose")
            .contains("WholeCorpusCompileBudgetUnreadable"));
    }

    fn req(bound: u64) -> CgroupBindRequest {
        CgroupBindRequest::Requested { bound }
    }

    /// A machine large enough that the machine relation is never the thing under test, and a
    /// current usage small enough that the not-instantly-fatal relation is not either.
    const SIZED_MACHINE: u64 = 26_448_039_936;
    const SMALL_RESIDENT: u64 = 1_073_741_824;

    /// The admitting arm: an action holding no observation, on a machine larger than the bound
    /// it asks for, binds. This is the capability the module exists to restore.
    #[test]
    fn an_unbounded_action_on_a_sized_machine_binds() {
        assert_eq!(
            cgroup_bind_decision(
                Some(&req(10_737_418_240)),
                None,
                Some(SIZED_MACHINE),
                Some(SMALL_RESIDENT),
                true,
                true
            ),
            CgroupBindDecision::Applicable {
                bound: 10_737_418_240
            }
        );
    }

    /// RED — the default remote runner, unsized: the bound asked for is larger than the whole
    /// machine, so a written memory.max could never be reached and would bound nothing while
    /// being cited as a bound.
    #[test]
    fn a_bound_not_below_machine_memory_refuses_and_names_both_quantities() {
        let decision = cgroup_bind_decision(
            Some(&req(10_737_418_240)),
            None,
            Some(7_838_253_056),
            Some(SMALL_RESIDENT),
            true,
            true,
        );
        let text = cgroup_bind_refusal_diagnostic(&decision).expect("refusal must diagnose");
        assert!(text.contains("MemoryCgroupBindRefused"));
        assert!(text.contains("10737418240"));
        assert!(text.contains("7838253056"));
        assert!(text.contains("size the executor larger than the bound"));
    }

    /// The machine boundary is STRICT, checked one byte apart from both sides.
    #[test]
    fn the_machine_boundary_is_strict_on_both_sides() {
        assert!(matches!(
            cgroup_bind_decision(
                Some(&req(7_838_253_056)),
                None,
                Some(7_838_253_056),
                Some(SMALL_RESIDENT),
                true,
                true
            ),
            CgroupBindDecision::Refused { .. }
        ));
        assert!(matches!(
            cgroup_bind_decision(
                Some(&req(7_838_253_055)),
                None,
                Some(7_838_253_056),
                Some(SMALL_RESIDENT),
                true,
                true
            ),
            CgroupBindDecision::Applicable { .. }
        ));
    }

    /// RED — a malformed request must NOT read as "no request". An operator who asked to be
    /// bounded, with a value that cannot be parsed, would otherwise proceed unbounded and be
    /// told nothing: silence is the right answer to a question nobody asked and never to one
    /// asked badly. Review 5117628699.
    #[test]
    fn an_unreadable_request_refuses_rather_than_becoming_no_request() {
        let decision = cgroup_bind_decision(
            Some(&CgroupBindRequest::Unreadable {
                raw: "  twelve gigs ".to_string(),
            }),
            None,
            Some(SIZED_MACHINE),
            Some(SMALL_RESIDENT),
            true,
            true,
        );
        let text = cgroup_bind_refusal_diagnostic(&decision).expect("refusal must diagnose");
        assert!(text.contains("twelve gigs"));
        assert!(text.contains("refused rather than treated as no request"));
        assert!(!matches!(decision, CgroupBindDecision::NotRequested));
    }

    /// RED — a bound at or below what the process already holds is fatal the instant it is
    /// written, so the run dies before any later admission arm could report it as too small.
    /// Zero is the extreme case the first revision admitted. Review 5117628699.
    #[test]
    fn a_bound_not_above_current_usage_refuses_including_zero() {
        for bound in [0_u64, SMALL_RESIDENT, SMALL_RESIDENT - 1] {
            let decision = cgroup_bind_decision(
                Some(&req(bound)),
                None,
                Some(SIZED_MACHINE),
                Some(SMALL_RESIDENT),
                true,
                true,
            );
            assert!(
                matches!(
                    decision,
                    CgroupBindDecision::Refused {
                        cause: CgroupBindRefusalCause::RequestNotAboveCurrentUsage { .. }
                    }
                ),
                "bound {bound} must refuse"
            );
        }
        // And one byte above the resident set admits, so the boundary is strict rather than a
        // wide band that would hide a defect on either side.
        assert!(matches!(
            cgroup_bind_decision(
                Some(&req(SMALL_RESIDENT + 1)),
                None,
                Some(SIZED_MACHINE),
                Some(SMALL_RESIDENT),
                true,
                true
            ),
            CgroupBindDecision::Applicable { .. }
        ));
    }

    /// An unreadable machine or an unreadable resident size refuses rather than assuming the
    /// request fits — the arms that would otherwise be the absorbing ones.
    #[test]
    fn an_unreadable_input_refuses_rather_than_assuming_the_request_fits() {
        assert_eq!(
            cgroup_bind_decision(
                Some(&req(1024)),
                None,
                None,
                Some(SMALL_RESIDENT),
                true,
                true
            ),
            CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::MachineMemoryUnreadable
            }
        );
        assert_eq!(
            cgroup_bind_decision(
                Some(&req(1024)),
                None,
                Some(SIZED_MACHINE),
                None,
                true,
                true
            ),
            CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::CurrentUsageUnreadable
            }
        );
    }

    /// Neither mechanism failure may degrade into proceeding unbounded.
    #[test]
    fn an_unwritable_or_uncontrolled_tree_refuses_rather_than_proceeding_unbounded() {
        assert_eq!(
            cgroup_bind_decision(
                Some(&req(10_737_418_240)),
                None,
                Some(SIZED_MACHINE),
                Some(SMALL_RESIDENT),
                false,
                true
            ),
            CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::MemoryControllerUnavailable
            }
        );
        assert_eq!(
            cgroup_bind_decision(
                Some(&req(10_737_418_240)),
                None,
                Some(SIZED_MACHINE),
                Some(SMALL_RESIDENT),
                true,
                false
            ),
            CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::CgroupTreeNotWritable
            }
        );
    }

    /// THE NO-OP ARM, and it is keyed on the source that actually hard-bounds this process. A
    /// memory.max observation suppresses the bind: the process already has what was asked for.
    #[test]
    fn a_process_already_bound_by_memory_max_is_not_written_to() {
        let observation = HostBudgetObservation {
            source: HostBudgetSource::CgroupMemoryMax {
                cgroup_dir: "/sys/fs/cgroup/actions-runner.slice".to_string(),
            },
            bytes: 16_106_127_360,
        };
        let decision = cgroup_bind_decision(
            Some(&req(10_737_418_240)),
            Some(&observation),
            Some(SIZED_MACHINE),
            Some(SMALL_RESIDENT),
            true,
            true,
        );
        assert!(cgroup_bind_refusal_diagnostic(&decision).is_none());
        assert!(matches!(
            decision,
            CgroupBindDecision::UnnecessaryLimitAlreadyBinds { .. }
        ));
        let note = cgroup_bind_note(&decision);
        assert!(note.contains("unnecessary"));
        assert!(note.contains("actions-runner.slice"));
    }

    /// RED — THE NON-BOUNDING-SOURCE CONTROL, which is the discriminating half of the arm above.
    /// `memory.high` is a reclaim throttle allocations may cross, and Darwin physical memory is a
    /// fact about the machine; neither hard-bounds this process, so neither may suppress a
    /// requested kernel bind. The first revision accepted any observation here and reported that
    /// a limit already bound a process that nothing bounded. Review 5117628699.
    #[test]
    fn a_non_bounding_observation_does_not_suppress_the_bind() {
        for source in [
            HostBudgetSource::CgroupMemoryHigh {
                cgroup_dir: "/sys/fs/cgroup/actions-runner.slice".to_string(),
            },
            HostBudgetSource::DarwinPhysicalMemory,
        ] {
            assert!(!source.bounds_this_process());
            let observation = HostBudgetObservation {
                source,
                bytes: 16_106_127_360,
            };
            assert!(
                matches!(
                    cgroup_bind_decision(
                        Some(&req(10_737_418_240)),
                        Some(&observation),
                        Some(SIZED_MACHINE),
                        Some(SMALL_RESIDENT),
                        true,
                        true
                    ),
                    CgroupBindDecision::Applicable { .. }
                ),
                "a source that does not bound this process must not suppress the bind"
            );
        }
    }

    /// Binding is opt-in: with no request nothing is decided against, even on a machine that
    /// would have refused, so no existing caller changes behaviour.
    #[test]
    fn no_request_is_a_no_op_even_where_a_request_would_have_refused() {
        let decision = cgroup_bind_decision(None, None, Some(7_838_253_056), None, false, false);
        assert_eq!(decision, CgroupBindDecision::NotRequested);
        assert!(cgroup_bind_refusal_diagnostic(&decision).is_none());
        assert!(cgroup_bind_note(&decision).contains("not requested"));
    }
}
