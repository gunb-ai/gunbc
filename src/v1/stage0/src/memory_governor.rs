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

fn mirror_ci_human_percent(bp: u64) -> String {
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
pub const DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES: u64 = 16106127360;

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

/// Measured whole-tree compile demand — the threshold a whole-corpus compile is admitted
/// against. SCAFFOLD (§7 seed-retained HAND-RUST; authority
/// `gunbc.whole_corpus_compile_admission` `whole_corpus_compile_measured_peak_demand`), same
/// shape and reason as `DECLARED_FLOOR_MINIMUM_VIABLE_ARMED_BUDGET_BYTES`: the decision runs
/// before any `.dag` value could exist, being the decision whether resolving the corpus may begin.
///
/// Refusing-peak basis: two dated, uncensored whole-corpus peaks ON THE COMPILE ROUTE ITSELF
/// (2026-08-28, session clever-tern-899, srv1), clean tree staged at
/// `91c05c1b344d29f97a363eaff34843177d552a99`, binary BUILT FROM THAT SAME SHA — `--target dag`
/// peaked at 13008052 kB and `--target rust` at 13005964 kB, both EXIT=1 (completed and refused
/// on diagnostics, not killed), 271 GiB still free at exit so neither is a throttle pin. A scoped
/// positive control ran first and returned EXIT=0 with a file emitted, so the harness produces
/// both outcomes.
///
/// Superseded basis: two 2026-07-21 CI receipts (runs 29828873976 / 29834202745, ~6.3 and
/// ~6.2 GiB) on the FLOOR route, declared as a proxy and LOWER bound. That 7 GiB proxy sat 43.6%
/// BELOW the measured 12.41 GiB peak (56.4% of it), so this arm admitted hosts it would then be
/// killed on. The row's own re-measure trigger — a dated uncensored whole-tree peak on the
/// compile route — retired them on 2026-08-28.
///
/// Written as a canonical decimal literal so the mirror lens can join it to its row — an
/// underscored or arithmetic literal is the same value no join can reach.
///
/// SEED MIRROR of `gunbc.whole_corpus_compile_admission` `whole_corpus_compile_measured_peak_demand`
///
/// The marker was withheld for a merge-order reason that is the mechanism: the lens's BACKWARD
/// arm requires marker occurrences per seed file to equal roster rows homed in that file, and the
/// marker is the enrollment act, so a marked constant with no roster row reds main until both
/// land. gunbc#8635 and gunbc#8638 have merged, so marker and row land together here.
///
/// dissolve-on: the emit path that retires this seed's other budget mirrors; re-measure
/// trigger: a dated uncensored whole-corpus peak on the `gunbc compile` route, taken with a
/// binary built from the subject sha, that EXCEEDS this figure.
///
/// THIS VALUE IS NOT DERIVED FROM THE REFUSING PEAKS — that is why the figure is 16 GiB rather
/// than 13. All three of those runs REFUSED on diagnostics, so each peak bounds a run that
/// stopped early; the highest COMPLETING whole-corpus peak on this route is 15871708 kB =
/// 15.14 GiB (`--target dag`, exit 0, attributed to warm-ant-908 and adopted rather than
/// reproduced here). 15.14 GiB rounded UP to whole-gibibyte grain is 16 GiB = 17179869184 —
/// landing on `memory_max` by arithmetic, not by design.
///
/// Declaring 13 GiB (the refusing peaks rounded up) was this row's state until review 57202 on
/// gunbc#9545, which observed it left every budget from 13 to 15.14 GiB admitted with no
/// evidence it can complete — the fail-open class this constant exists to close, one band
/// narrower. Rounding UP is what makes the grain rule fail-closed: for a DEMAND figure it
/// refuses the marginal case rounding down would admit. A threshold must not sit below the
/// highest peak anyone has measured on this route.
///
/// The `.dag` authority `gunbc.whole_corpus_compile_admission`
/// `whole_corpus_compile_measured_demand_note` carries the full adoption argument, what adoption
/// does and does not assert, and why the CI runner slot is now REFUSED as a deliberate
/// over-refusal. Keep this paragraph and that note in step: the seed-mirror lens checks the
/// numeric value only (see `seed_mirror_reach_note`'s residual), so a stale justification here
/// is the unaudited-prose drift that residual names — it recurred in the very diff that
/// repaired another instance of it.
pub const DECLARED_WHOLE_CORPUS_COMPILE_MEASURED_DEMAND_BYTES: u64 = 17179869184;

/// Arm-time admission for a WHOLE-CORPUS compile — the seed mirror of
/// `gunbc.whole_corpus_compile_admission` `whole_corpus_compile_admission`.
///
/// Exists because the budget was read and printed but joined to nothing:
/// `cli_run::typed_module_cache_cap` emits `[floor-drain] degraded_budget_source` and the process
/// then starts a resolve it cannot hold. Measured twice on the BuildBuddy remote-execution runner
/// (invocations a39713da-8cfb-415d-a8f6-1e0ef150d075 and 13cf8d2e-173a-42d2-9a56-101bb3332740):
/// SIGKILL, exit 137, no diagnostic — a harness grepping the captured output reads a fabricated
/// zero rather than a failure.
///
/// What it does NOT claim: an admitted budget is not certified sufficient. The threshold is a
/// peak taken on THIS route, at a named sha, with a binary built from it (the "neighbouring
/// route at an older tree" qualification carried until 2026-08-28 is retired). It is ONE tree's
/// peak, and a demand figure does not shrink with corpus growth, so admission means "not
/// provably doomed at the tree that was measured" (mitigatable, §4b), never "will fit".
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
}

pub fn whole_corpus_compile_admission(
    budget: Option<u64>,
    source: &str,
) -> WholeCorpusCompileAdmission {
    let Some(budget_bytes) = budget else {
        return WholeCorpusCompileAdmission::RefusedBudgetUnreadable {
            source: source.to_string(),
        };
    };
    let required_bytes = DECLARED_WHOLE_CORPUS_COMPILE_MEASURED_DEMAND_BYTES;
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

/// `Some(diagnostic)` on either refusal arm, `None` when admitted. The diagnostic names the two
/// disagreeing quantities, the budget and its source, and the scoped `--entry` narrowing measured
/// to fit on the same runner — a refusal proposing no remedy is a stopped line nobody can restart.
///
/// Deliberately a free function, not an inherent method: `std.decl_ref` `DeclField` offers
/// `WholeDeclaration` or `NamedField`, neither naming an impl-block method, so an impl method
/// cannot be cited in the `SeedGrowthJustification` this change owes. The sibling receipt in
/// `gunbc.stage0_rust_host_observation` carries one such uncitable item in prose because its
/// `Display` realization had no citable form; this one has, so it takes it.
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
             bytes (source={source}) is below the measured whole-tree compile demand of \
             {required_bytes} bytes (gunbc.whole_corpus_compile_admission \
             whole_corpus_compile_measured_peak_demand). Refusing to start a run that is \
             provably \
             below measured demand — the previous behaviour was to start it and be SIGKILLed, \
             which reports as a silent exit-137 zero rather than a diagnostic, so any count \
             grepped from such a run is a memorial to a killed process. Remedy: scope the \
             compile with --entry <file.dag>, or run it where a larger budget is readable."
        )),
        WholeCorpusCompileAdmission::RefusedBudgetUnreadable { source } => Some(format!(
            "WholeCorpusCompileBudgetUnreadable: no modeled host memory source answered \
             ({source}), so the bound on a whole-corpus compile is UNKNOWN. Refusing rather \
             than admitting against the widest cap available — an unbounded resolve on an \
             unbounded host is the OOM-kill this arm exists to prevent. Declare one with \
             GUNBC_MEMORY_BUDGET_BYTES, or model this platform's memory source \
             (dag/gunbc/host/host_budget_source.dag)."
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
    request: Option<u64>,
    existing: Option<&HostBudgetObservation>,
    machine_memory: Option<u64>,
    memory_controller_available: bool,
    cgroup_tree_writable: bool,
) -> CgroupBindDecision {
    let Some(bound) = request else {
        return CgroupBindDecision::NotRequested;
    };
    if let Some(observed) = existing {
        return CgroupBindDecision::UnnecessaryLimitAlreadyBinds {
            source: observed.source.clone(),
        };
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
    if bound < machine {
        CgroupBindDecision::Applicable { bound }
    } else {
        CgroupBindDecision::Refused {
            cause: CgroupBindRefusalCause::RequestNotBelowMachineMemory {
                requested: bound,
                machine,
            },
        }
    }
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

/// Whether this process may create a child group under the cgroup2 root. Probed by CREATING the
/// leaf this bind would use rather than by inspecting permission bits: the question is whether
/// the write succeeds, and anything short of performing it is a prediction.
fn cgroup_leaf_create(dir: &Path) -> bool {
    match std::fs::create_dir(dir) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => true,
        Err(_) => false,
    }
}

/// Create the bound, move this process into it, and return the executed decision.
///
/// THE ORDER IS THE SAFETY PROPERTY. The decision is taken first from an observation read
/// through the ordinary budget path, so a machine that already binds this process is never
/// written to; the write happens only on the `Applicable` arm; and afterwards every consumer
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
    let request = std::env::var(BIND_MEMORY_CGROUP_ENV)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    if request.is_none() {
        return CgroupBindDecision::NotRequested;
    }
    let existing = match read_host_budget_resolution() {
        HostBudgetResolution::Resolved { observation, .. } => Some(observation),
        _ => None,
    };
    let leaf = Path::new("/sys/fs/cgroup").join(BIND_MEMORY_CGROUP_LEAF);
    let controller = cgroup_memory_controller_available();
    // The writability probe is only performed when a write could follow: creating the leaf on a
    // machine that already binds this process would leave a group behind for no reason.
    let writable = if controller && existing.is_none() {
        cgroup_leaf_create(&leaf)
    } else {
        false
    };
    let decision = cgroup_bind_decision(
        request,
        existing.as_ref(),
        machine_memory_total_bytes(),
        controller,
        writable || existing.is_some(),
    );
    if let CgroupBindDecision::Applicable { bound } = decision {
        // A FAILED WRITE IS A REFUSAL, NOT A DEGRADED SUCCESS. Either arm below leaves the
        // process exactly as unbounded as it was, so reporting anything but a refusal would be
        // the escape hatch this whole module exists to close.
        let _ = std::fs::write("/sys/fs/cgroup/cgroup.subtree_control", "+memory");
        if std::fs::write(leaf.join("memory.max"), bound.to_string()).is_err() {
            return CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
            };
        }
        if std::fs::write(leaf.join("cgroup.procs"), std::process::id().to_string()).is_err() {
            return CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
            };
        }
    }
    decision
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

    /// The discriminating RED: the budget the BuildBuddy runner answered with on the SIGKILLed
    /// run (recovered from its own `cap=1675` line — see
    /// `gunbc.whole_corpus_compile_admission`), paired with the fleet runner slot's declared
    /// `memory.high`, which the machine CI runs this instrument on reports. Both arms stand on
    /// independently measured machines, so each fails for its own reason.
    ///
    /// THE SECOND ARM WAS INVERTED 2026-08-28 AND NOT EDITED TO STAY GREEN. It asserted the CI
    /// runner slot is ADMITTED. Adopting the highest measured COMPLETING peak as the demand
    /// (review 57202 on gunbc#9545) put the threshold above the slot's reported `memory.high`,
    /// so the slot is now refused — deliberately: the budget a host reports is what it agreed
    /// to give, and admitting because the run will breach it and survive on swap below
    /// `memory.max` admits a known breach. The alternative — lowering the threshold until the
    /// assertion stayed true — derives a safety literal from a wanted outcome. Its `.dag` twin
    /// (`test.claim.whole_corpus_compile_admission_witness_test`
    /// `the_runner_ci_actually_uses_is_refused_at_the_completing_peak_demand`) is inverted in
    /// lockstep; the `runner_slot_refusal_note` annotation there carries the full argument.
    #[test]
    fn whole_corpus_compile_refuses_the_budget_that_was_sigkilled_and_refuses_the_ci_runner() {
        // The SIGKILLed run's budget, carried under a label the resolver can still produce:
        // `/proc/meminfo MemAvailable` was that run's attribution and is no longer an
        // authorable source (see `resolve_host_budget`). The budget, which is what this arm
        // judges, is unchanged.
        let doomed = whole_corpus_compile_admission(
            Some(5_269_094_400),
            "cgroup memory.high (/sys/fs/cgroup/runner.slice)",
        );
        assert!(matches!(
            doomed,
            WholeCorpusCompileAdmission::RefusedBudgetBelowMeasuredDemand { .. }
        ));
        let msg = whole_corpus_compile_refusal_diagnostic(&doomed).expect("refusal must diagnose");
        assert!(msg.contains("WholeCorpusCompileBudgetBelowMeasuredDemand"));
        assert!(msg.contains("cgroup memory.high (/sys/fs/cgroup/runner.slice)"));
        assert!(msg.contains("--entry"));

        let ci_slot = whole_corpus_compile_admission(
            Some(DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES),
            "cgroup memory.high",
        );
        assert!(matches!(
            ci_slot,
            WholeCorpusCompileAdmission::RefusedBudgetBelowMeasuredDemand { .. }
        ));
        assert!(whole_corpus_compile_refusal_diagnostic(&ci_slot).is_some());
    }

    #[test]
    fn whole_corpus_compile_admission_is_tight_at_the_measured_demand() {
        let at = whole_corpus_compile_admission(
            Some(DECLARED_WHOLE_CORPUS_COMPILE_MEASURED_DEMAND_BYTES),
            "env GUNBC_MEMORY_BUDGET_BYTES",
        );
        assert!(whole_corpus_compile_refusal_diagnostic(&at).is_none());
        let one_short = whole_corpus_compile_admission(
            Some(DECLARED_WHOLE_CORPUS_COMPILE_MEASURED_DEMAND_BYTES - 1),
            "env GUNBC_MEMORY_BUDGET_BYTES",
        );
        assert!(whole_corpus_compile_refusal_diagnostic(&one_short).is_some());
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

    /// An unreadable budget refuses rather than admitting against the widest cap available —
    /// the arm the `host_budget_source_no_fallback_arm_note` annotation records as having OOM-killed the
    /// witness corpus twice when it was a `.unwrap_or(CEIL)`.
    #[test]
    fn whole_corpus_compile_unreadable_budget_refuses_rather_than_widening() {
        let unreadable = whole_corpus_compile_admission(None, "unreadable: no modeled source");
        assert!(matches!(
            unreadable,
            WholeCorpusCompileAdmission::RefusedBudgetUnreadable { .. }
        ));
        assert!(whole_corpus_compile_refusal_diagnostic(&unreadable)
            .expect("refusal must diagnose")
            .contains("WholeCorpusCompileBudgetUnreadable"));
    }

    /// The admitting arm: an action holding no observation, on a machine larger than the bound
    /// it asks for, binds. This is the capability the module exists to restore.
    #[test]
    fn an_unbounded_action_on_a_sized_machine_binds() {
        assert_eq!(
            cgroup_bind_decision(Some(10_737_418_240), None, Some(26_448_039_936), true, true),
            CgroupBindDecision::Applicable {
                bound: 10_737_418_240
            }
        );
    }

    /// RED — the default remote runner, unsized: the bound asked for is larger than the whole
    /// machine, so a written memory.max could never be reached and would bound nothing while
    /// being cited as a bound. The refusal names both quantities and says which side to move.
    #[test]
    fn a_bound_not_below_machine_memory_refuses_and_names_both_quantities() {
        let decision =
            cgroup_bind_decision(Some(10_737_418_240), None, Some(7_838_253_056), true, true);
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
            cgroup_bind_decision(Some(7_838_253_056), None, Some(7_838_253_056), true, true),
            CgroupBindDecision::Refused { .. }
        ));
        assert!(matches!(
            cgroup_bind_decision(Some(7_838_253_055), None, Some(7_838_253_056), true, true),
            CgroupBindDecision::Applicable { .. }
        ));
    }

    /// Neither mechanism failure may degrade into proceeding unbounded: an executor that could
    /// not write its limit HAS no limit, which is the state being ended.
    #[test]
    fn an_unwritable_or_uncontrolled_tree_refuses_rather_than_proceeding_unbounded() {
        assert_eq!(
            cgroup_bind_decision(Some(1024), None, Some(26_448_039_936), false, true),
            CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::MemoryControllerUnavailable
            }
        );
        assert_eq!(
            cgroup_bind_decision(Some(1024), None, Some(26_448_039_936), true, false),
            CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::CgroupTreeNotWritable
            }
        );
    }

    /// An unreadable machine refuses rather than assuming the request fits — the arm that would
    /// otherwise be the absorbing one.
    #[test]
    fn an_unreadable_machine_refuses_rather_than_assuming_the_request_fits() {
        assert_eq!(
            cgroup_bind_decision(Some(1024), None, None, true, true),
            CgroupBindDecision::Refused {
                cause: CgroupBindRefusalCause::MachineMemoryUnreadable
            }
        );
    }

    /// THE CI CONTROL. A runner whose slice already binds memory.high takes the unnecessary
    /// arm: no write, no refusal, and the note names the limit that already holds. A green lane
    /// stays green and does not acquire a second authority for its own bound.
    #[test]
    fn a_machine_already_bound_is_not_written_to_and_is_not_a_failure() {
        let observation = HostBudgetObservation {
            source: HostBudgetSource::CgroupMemoryHigh {
                cgroup_dir: "/sys/fs/cgroup/actions-runner.slice".to_string(),
            },
            bytes: 16_106_127_360,
        };
        let decision = cgroup_bind_decision(
            Some(10_737_418_240),
            Some(&observation),
            Some(26_448_039_936),
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

    /// Binding is opt-in: with no request nothing is decided against, even on a machine that
    /// would have refused, so no existing caller changes behaviour.
    #[test]
    fn no_request_is_a_no_op_even_where_a_request_would_have_refused() {
        let decision = cgroup_bind_decision(None, None, Some(7_838_253_056), false, false);
        assert_eq!(decision, CgroupBindDecision::NotRequested);
        assert!(cgroup_bind_refusal_diagnostic(&decision).is_none());
        assert!(cgroup_bind_note(&decision).contains("not requested"));
    }
}
