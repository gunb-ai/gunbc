//! AIMD memory governor: adaptive worker admission against the slot's declared memory budget.
//!
//! Replaces the hand-pinned predicted-peak width folds (the retired
//! `gunbc.ci_floor_measurement` width path) with a TCP-shaped controller (operator model,
//! 2026-07-12): the budget is the pipe — the slot's own cgroup limits, so the fleet
//! allocation IS the declaration and no byte constant lives in the tree — work is admitted
//! while signals are calm, and the controller backs off on CREEP before the kill line.
//! Because `memory.high` reclaim (and swap, where present) buffer the overshoot, creep is
//! not loss — it is the EARLY congestion signal (TCP's ECN mark): the first hold of a
//! creep episode halves the window so workers drain while the buffer zone still has
//! drain-latency margin, and admissions stay held until calm. Hard events (throttle-line
//! crossings, OOM kills, and `memory.current` crossing the declared budget itself — the
//! reachable loss signal on uncapped hosts) halve it again as the backstop.
//!
//! Signals, read from the tightest-limit cgroup directory (else the leaf):
//!   `memory.current`             — instantaneous usage vs budget (high-water headroom)
//!   `memory.swap.current`        — per-sample growth = actively creeping past physical
//!   `memory.pressure` some avg10 — kernel reclaim stall %, the earliest creep signal
//!                                  (present with or without swap)
//!   `memory.events` high/oom_kill — cumulative counters; a positive delta is a hard event
//!
//! No-swap regimes degrade honestly: an absent `memory.swap.current` makes the swap check
//! inert (`None`, never a fabricated zero) while current + PSI carry the decision. The
//! budget chain is env override > tightest cgroup `memory.high` > tightest `memory.max` >
//! min(MemAvailable, declared runner-slot `memory.high`) > MemTotal — on uncapped hosts
//! MemAvailable alone is capped at the declared slot throttle line (15 GiB from
//! `gunbc.runner_slot_allocation`; srv3 2026-07-21 exit-137: uncapped MemAvailable ~35 GiB
//! let batch-2 retention reach ~38 GiB before OOM-kill). MemAvailable still precedes
//! MemTotal because the kernel's availability estimate excludes the co-tenant baseline
//! (run 29180195694: a MemTotal budget let demand reach physical RAM and starve the
//! runner agent). No readable cgroup at all leaves the creep checks inert — the window
//! arithmetic, the pacing/headroom gates, and the kill line still govern.
//!
//! Fail-closed discipline (DESIGN §5): every graceful hold and hard back-off is a typed,
//! counted, logged event surfaced in the end-of-run receipt — degradation is loud, bounded,
//! and observable, never an absorbed rerun. The width-1 floor is a progress guarantee, not
//! a widen: with zero workers active the governor always admits exactly one (counted as
//! `forced_serial` when signals were hot), so a floor that cannot afford parallelism runs
//! serially and says so; the cgroup kill line remains the terminal, attributable failure.
//!
//! Dissolve-on: graph-derived per-node demand (CostAccount.space measured/derived) replaces
//! the reactive estimator — the same admission loop then consumes predictions and this
//! controller demotes to a safety net.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdmitDecision {
    /// A worker slot was granted. `forced_serial` marks the width-1 progress floor firing
    /// while signals were hot (counted in the receipt).
    Admit {
        forced_serial: bool,
    },
    Hold(HoldReason),
}

/// Pure controller state — every transition is a function of (state, sample, limits) so the
/// policy is unit-testable without a cgroup.
#[derive(Debug, Clone)]
struct GovCore {
    target_width: usize,
    active: usize,
    prev_swap_bytes: Option<u64>,
    baseline_events_high: Option<u64>,
    baseline_events_oom_kill: Option<u64>,
    /// Episode edge for hold counting/logging: a hold "episode" spans consecutive held
    /// polls; it counts once and re-arms on the next successful admission.
    holding: bool,
    /// Admitted workers that have not yet paid their front-loaded admission cost (the
    /// whole-tree index build). Admission is paced to at most one outstanding, so the
    /// ramp rate is the index-build rate — the demand-relevant clock — not the unit
    /// completion rate.
    undigested: usize,
    /// Episode edge for pacing holds, symmetric with `holding`.
    pacing_episode: bool,
    /// Episode edge for the budget-exceeded hard event; re-arms below high-water.
    budget_exceeded_episode: bool,
    /// `memory.current` at the first poll of the run — the pre-work floor against which
    /// the first worker's share is measured.
    pool_baseline_current: Option<u64>,
    /// The first worker's measured cost (current at its first-cost landing minus the
    /// pool baseline): the run's own estimate of one worker's share, used to predict
    /// whether one more admission fits under high-water. Measured once; dissolves into
    /// graph-derived per-node demand.
    measured_worker_share: Option<u64>,
    /// Episode edge for headroom holds, symmetric with `holding`.
    headroom_episode: bool,
    /// Episode edge for admission-ceiling holds, symmetric with `holding`.
    ceiling_episode: bool,
    admissions: u64,
    forced_serial_admissions: u64,
    creep_backoffs: u64,
    pacing_holds: u64,
    headroom_holds: u64,
    ceiling_holds: u64,
    hard_backoffs: u64,
    budget_exceeded_backoffs: u64,
    width_growths: u64,
    max_width_reached: usize,
    peak_current_bytes: u64,
}

impl GovCore {
    fn new() -> Self {
        GovCore {
            target_width: 1,
            active: 0,
            prev_swap_bytes: None,
            baseline_events_high: None,
            baseline_events_oom_kill: None,
            holding: false,
            undigested: 0,
            pacing_episode: false,
            budget_exceeded_episode: false,
            pool_baseline_current: None,
            measured_worker_share: None,
            headroom_episode: false,
            ceiling_episode: false,
            admissions: 0,
            forced_serial_admissions: 0,
            creep_backoffs: 0,
            pacing_holds: 0,
            headroom_holds: 0,
            ceiling_holds: 0,
            hard_backoffs: 0,
            budget_exceeded_backoffs: 0,
            width_growths: 0,
            max_width_reached: 0,
            peak_current_bytes: 0,
        }
    }
}

/// A hard event absorbed from cumulative counters: positive deltas vs the run baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
struct HardEvent {
    high_delta: u64,
    oom_kill_delta: u64,
    /// `memory.current` crossed the declared budget itself. The budget IS the declaration
    /// (§5 correctness-by-construction: on uncapped hosts `memory.events` never fires, so
    /// without this arm the multiplicative decrease is unreachable and in-flight worker
    /// growth sails through the hold zone to physical RAM — CI run 29181858455: held at
    /// high-water 41.5GiB, dead at 52.6GiB seventy seconds later with zero admissions).
    budget_exceeded: bool,
    old_target: usize,
    new_target: usize,
}

impl HardEvent {
    fn describe(&self) -> String {
        let cause = if self.budget_exceeded {
            "memory.current exceeded the declared budget".to_string()
        } else {
            format!(
                "memory.events high +{} oom_kill +{}",
                self.high_delta, self.oom_kill_delta
            )
        };
        format!(
            "[governor] hard back-off: {cause} — target_width {}→{} (workers drain between units)",
            self.old_target, self.new_target
        )
    }
}

/// The governor's static configuration: the declared pipe.
#[derive(Debug, Clone)]
pub struct GovernorLimits {
    pub budget_bytes: Option<u64>,
    pub budget_source: String,
    pub max_width: usize,
}

/// Fold one sample's hard signals into the core. Two hard families, one halving:
/// cumulative `memory.events` counters (first sight sets the baseline — history, not an
/// event; a later positive delta is an event), and the budget-exceeded edge — `current`
/// above the declared budget is an exceeded pipe by definition, giving uncapped hosts
/// (where `memory.events` never fires) a reachable multiplicative decrease. The exceeded
/// edge is episode-counted with hysteresis: it fires once per crossing and re-arms only
/// when `current` falls back under the high-water line, so oscillation at the budget
/// line cannot halve the window once per poll. Any hard event halves the window
/// (floor 1); `should_retire` then drains workers between units, freeing their indexes.
fn absorb_hard_events(
    core: &mut GovCore,
    sig: &MemorySignals,
    limits: &GovernorLimits,
) -> Option<HardEvent> {
    if let Some(cur) = sig.current_bytes {
        core.peak_current_bytes = core.peak_current_bytes.max(cur);
    }
    let mut high_delta = 0u64;
    let mut oom_delta = 0u64;
    if let Some(h) = sig.events_high {
        match core.baseline_events_high {
            Some(base) if h > base => high_delta = h - base,
            Some(_) => {}
            None => {}
        }
        core.baseline_events_high = Some(h);
    }
    if let Some(o) = sig.events_oom_kill {
        match core.baseline_events_oom_kill {
            Some(base) if o > base => oom_delta = o - base,
            Some(_) => {}
            None => {}
        }
        core.baseline_events_oom_kill = Some(o);
    }
    let mut budget_exceeded = false;
    if let (Some(current), Some(budget)) = (sig.current_bytes, limits.budget_bytes) {
        let high_water = budget / HIGH_WATER_DEN * HIGH_WATER_NUM;
        if current > budget {
            if !core.budget_exceeded_episode {
                core.budget_exceeded_episode = true;
                core.budget_exceeded_backoffs += 1;
                budget_exceeded = true;
            }
        } else if current <= high_water {
            core.budget_exceeded_episode = false;
        }
    }
    if high_delta == 0 && oom_delta == 0 && !budget_exceeded {
        return None;
    }
    let old_target = core.target_width;
    // Budget exceeded is TCP's timeout, not its duplicate-ACK: collapse to 1 and let
    // slow-start rebuild. Runs 29181858455/29183064852 showed a halve leaves too many
    // growing workers alive when the crossing is discovered one gigabyte from death.
    core.target_width = if budget_exceeded {
        1
    } else {
        core.active.div_ceil(2).max(1).min(old_target.max(1))
    };
    core.hard_backoffs += 1;
    Some(HardEvent {
        high_delta,
        oom_kill_delta: oom_delta,
        budget_exceeded,
        old_target,
        new_target: core.target_width,
    })
}

/// The graceful arm: is this sample creeping? Reads `prev_swap_bytes` (last sample) and
/// leaves updating it to the caller so the delta is per-sample, not per-check.
fn creep_reason(
    core: &GovCore,
    sig: &MemorySignals,
    limits: &GovernorLimits,
) -> Option<HoldReason> {
    if let (Some(current), Some(budget)) = (sig.current_bytes, limits.budget_bytes) {
        let high_water = budget / HIGH_WATER_DEN * HIGH_WATER_NUM;
        if current > high_water {
            return Some(HoldReason::CurrentHighWater {
                current,
                high_water,
            });
        }
    }
    if let Some(avg10) = sig.psi_some_avg10 {
        if avg10 > PSI_HOLD_AVG10 {
            return Some(HoldReason::PsiPressure { avg10 });
        }
    }
    if let (Some(prev), Some(now)) = (core.prev_swap_bytes, sig.swap_current_bytes) {
        if now > prev && now - prev >= SWAP_GROWTH_HOLD_BYTES {
            return Some(HoldReason::SwapGrowth { delta: now - prev });
        }
    }
    None
}

/// One admission poll. Order: absorb hard events (may shrink the window), then the
/// progress floor (active == 0 always admits), then window arithmetic, then the creep arm.
fn decide_admission(
    core: &mut GovCore,
    sig: &MemorySignals,
    limits: &GovernorLimits,
) -> (AdmitDecision, Option<HardEvent>) {
    if core.pool_baseline_current.is_none() {
        core.pool_baseline_current = sig.current_bytes;
    }
    let hard = absorb_hard_events(core, sig, limits);
    let creep = creep_reason(core, sig, limits);
    core.prev_swap_bytes = sig.swap_current_bytes.or(core.prev_swap_bytes);
    if core.active == 0 {
        let forced = creep.is_some();
        core.active = 1;
        core.undigested += 1;
        core.admissions += 1;
        if forced {
            core.forced_serial_admissions += 1;
        }
        core.holding = false;
        core.pacing_episode = false;
        core.max_width_reached = core.max_width_reached.max(core.active);
        return (
            AdmitDecision::Admit {
                forced_serial: forced,
            },
            hard,
        );
    }
    if let Some(reason) = creep {
        creep_episode_backoff(core);
        return (AdmitDecision::Hold(reason), hard);
    }
    if core.active >= core.target_width {
        return (
            AdmitDecision::Hold(HoldReason::WindowFull {
                active: core.active,
                target: core.target_width,
            }),
            hard,
        );
    }
    if core.undigested > 0 {
        if !core.pacing_episode {
            core.pacing_episode = true;
            core.pacing_holds += 1;
        }
        return (
            AdmitDecision::Hold(HoldReason::AwaitFirstCost {
                undigested: core.undigested,
            }),
            hard,
        );
    }
    if let Some(reason) = headroom_hold(core, sig, limits) {
        if !core.headroom_episode {
            core.headroom_episode = true;
            core.headroom_holds += 1;
        }
        return (AdmitDecision::Hold(reason), hard);
    }
    core.headroom_episode = false;
    if let (Some(current), Some(budget)) = (sig.current_bytes, limits.budget_bytes) {
        let ceiling = budget / ADMIT_CEILING_DEN * ADMIT_CEILING_NUM;
        if current > ceiling {
            if !core.ceiling_episode {
                core.ceiling_episode = true;
                core.ceiling_holds += 1;
            }
            return (
                AdmitDecision::Hold(HoldReason::AdmissionCeiling { current, ceiling }),
                hard,
            );
        }
    }
    core.ceiling_episode = false;
    core.active += 1;
    core.undigested += 1;
    core.admissions += 1;
    core.holding = false;
    core.pacing_episode = false;
    core.max_width_reached = core.max_width_reached.max(core.active);
    (
        AdmitDecision::Admit {
            forced_serial: false,
        },
        hard,
    )
}

/// The single authority for "does ONE more worker-sized claim fit under the high-water
/// line", read by BOTH the admission gate and the additive-increase arm.
///
/// Two arms answering this differently is what let the window run away from realized
/// affordability: admission predicted `current + share <= high_water`, while increase
/// predicted nothing at all and grew on any calm completion (§3 — one fact, one
/// authority). `None` = no prediction is possible yet (no share measured, or the signal
/// is unreadable); the caller decides what an unpredictable state licenses, and the two
/// callers here differ only in that, never in the arithmetic.
fn headroom_hold(
    core: &GovCore,
    sig: &MemorySignals,
    limits: &GovernorLimits,
) -> Option<HoldReason> {
    let (Some(current), Some(share), Some(budget)) = (
        sig.current_bytes,
        core.measured_worker_share,
        limits.budget_bytes,
    ) else {
        return None;
    };
    let high_water = budget / HIGH_WATER_DEN * HIGH_WATER_NUM;
    (current.saturating_add(share) > high_water).then_some(HoldReason::InsufficientHeadroom {
        current,
        share,
        high_water,
    })
}

/// The creep-episode edge is the EARLY congestion signal — TCP's ECN mark, not its loss.
/// Run 29182481051 proved a hold alone cannot save the box: admissions stopped at the
/// high-water line, but the already-admitted workers' residency growth (~1GiB/min each,
/// request-major closure accumulation) carried memory from 47.9 to a fatal 52.7GiB in
/// eighty seconds, and the budget-exceed halve at 51.7GiB left no drain-latency margin.
/// So the FIRST hold of a creep episode also halves the window: workers drain while the
/// high-water→budget buffer still has ~10GiB of margin, which is the drain latency plus
/// the allocator's freed-page plateau. One halving per episode; re-arms on admission.
fn creep_episode_backoff(core: &mut GovCore) -> Option<(usize, usize)> {
    if core.holding {
        return None;
    }
    core.holding = true;
    core.creep_backoffs += 1;
    let old = core.target_width;
    core.target_width = core.active.div_ceil(2).max(1).min(old.max(1));
    Some((old, core.target_width))
}

/// What a completion is worth to the window. `target_width` is denominated in
/// WORKER-SIZED residency claims — the quantity `active` counts and `measured_worker_share`
/// calibrates — so only an event carrying evidence about that quantity may move it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum CompletionKind {
    /// A worker-sized cost LANDED and is now visible to the signals (an index build
    /// finished), or the pool is stepping off the width-1 floor. Either way the poll
    /// carries evidence about worker-sized residency, so it is an increase point — the
    /// index-build clock that `undigested`'s own contract already names as the ramp rate.
    IncreasePoint,
    /// A unit of work (a resolve-group, an entry-group) finished. It still re-reads the
    /// signals — completions may be the only polls at a full window, so the creep and
    /// hard arms must run — but it is NOT an increase point: a unit is not a worker.
    ///
    /// Growing here was the defect that made the un-latched window unsurvivable. Units
    /// are numerous and mostly cheap (2056 rows over 621 entry-groups), so the window
    /// tracked the unit-completion rate instead of realized affordability: CI run
    /// 29710324768 lifted it off the floor 34ms into the corpus, before any worker cost
    /// had EVER been measured, and admitted against that fiction until `memory.current`
    /// reached 101.6 GB and the box OOM-killed the run (exit 137). The grain conflation
    /// is the root cause; the resulting overshoot is the symptom.
    ObserveOnly,
}

/// Additive increase, gated on calm AND on the same headroom prediction the admission
/// path uses: a completion that carries worker-sized evidence, observed while no creep is
/// visible and while one more worker is predicted to fit, grows the window by one up to
/// the CPU bound. A creep edge observed here backs off exactly as in the admission path.
///
/// The window may never grow into a state admission would immediately refuse — that is
/// what makes the ramp mean something. When no prediction is possible (`headroom_hold`
/// returns `None` because no worker has ever landed a cost), growth is licensed: running
/// one worker is the only way to learn what a worker costs, and the step commits exactly
/// one. That exploratory step is bounded by construction — it can only be taken while
/// `measured_worker_share` is unset, and the first landing sets it.
fn note_completion(
    core: &mut GovCore,
    sig: &MemorySignals,
    limits: &GovernorLimits,
    kind: CompletionKind,
) -> (Option<HardEvent>, Option<(usize, usize)>) {
    let hard = absorb_hard_events(core, sig, limits);
    let creep = creep_reason(core, sig, limits);
    core.prev_swap_bytes = sig.swap_current_bytes.or(core.prev_swap_bytes);
    if hard.is_some() || creep.is_some() {
        let backoff = if creep.is_some() {
            creep_episode_backoff(core)
        } else {
            None
        };
        return (hard, backoff);
    }
    if kind == CompletionKind::IncreasePoint
        && headroom_hold(core, sig, limits).is_none()
        && core.target_width < limits.max_width
    {
        core.target_width += 1;
        core.width_growths += 1;
    }
    (hard, None)
}

/// A worker's front-loaded admission cost has landed (its index build finished): its
/// demand is now visible to the creep signals, so admission pacing may pass the next
/// one. The FIRST landing also fixes the run's measured worker share — current minus
/// the pool baseline — which the headroom gate uses to predict whether one more
/// admission fits. Measured once per run from the run's own first slot; no authored
/// constant (dissolves into graph-derived per-node demand).
fn note_first_cost(core: &mut GovCore, sig: &MemorySignals) {
    core.undigested = core.undigested.saturating_sub(1);
    if core.measured_worker_share.is_none() {
        if let (Some(current), Some(base)) = (sig.current_bytes, core.pool_baseline_current) {
            let share = current.saturating_sub(base);
            if share > 0 {
                core.measured_worker_share = Some(share);
            }
        }
    }
}

/// A worker released its slot. A release before the first cost was paid (error/panic/
/// early retire) clears its pacing debt too — a dead worker must not freeze admissions.
fn note_release(core: &mut GovCore, first_cost_paid: bool) {
    core.active = core.active.saturating_sub(1);
    if !first_cost_paid {
        core.undigested = core.undigested.saturating_sub(1);
    }
}

/// Where the signals come from: the tightest-limit cgroup directory, else the leaf.
/// `None` = no readable cgroup (signals inert; the budget fallback still applies).
pub struct SignalSource {
    dir: Option<PathBuf>,
}

impl SignalSource {
    pub fn read(&self) -> MemorySignals {
        let Some(dir) = &self.dir else {
            return MemorySignals::default();
        };
        MemorySignals {
            current_bytes: read_cgroup_u64(dir, "memory.current"),
            swap_current_bytes: read_cgroup_u64(dir, "memory.swap.current"),
            psi_some_avg10: read_cgroup_raw(dir, "memory.pressure")
                .and_then(|c| memory_pressure_some_avg10(&c))
                .and_then(|s| s.parse::<f64>().ok()),
            events_high: read_cgroup_raw(dir, "memory.events")
                .and_then(|c| memory_events_field(&c, "high")),
            events_oom_kill: read_cgroup_raw(dir, "memory.events")
                .and_then(|c| memory_events_field(&c, "oom_kill")),
        }
    }
}

/// The adaptive admission controller. One instance governs a whole run: every worker
/// slot — gate resolve-group threads and discovery-pool workers alike — is admitted
/// through it, so total concurrency tracks the one real budget instead of per-batch pins.
pub struct MemoryGovernor {
    limits: GovernorLimits,
    source: SignalSource,
    core: Mutex<GovCore>,
}

impl MemoryGovernor {
    /// Detect the budget and sensor directory from the environment:
    /// `GUNBC_MEMORY_BUDGET_BYTES` (explicit declaration) > tightest cgroup
    /// `memory.high` (the speed line — pack to it, not the kill line) > tightest
    /// `memory.max` > `/proc/meminfo` MemTotal. Announces itself on stderr — the
    /// governor replaces the plan-evaluated spawn width, so its one log line is the
    /// width story for the run.
    pub fn from_environment(max_width: usize) -> MemoryGovernor {
        let (budget, source_label) = read_host_budget_bytes();
        let sensor_dir = binding_high_cgroup_dir()
            .or_else(binding_cap_cgroup_dir)
            .or_else(leaf_cgroup_dir);
        let limits = GovernorLimits {
            budget_bytes: budget,
            budget_source: source_label,
            max_width: max_width.max(1),
        };
        eprintln!(
            "[governor] adaptive width (AIMD): budget={} source={} max_width={} sensors={}",
            limits
                .budget_bytes
                .map(|b| b.to_string())
                .unwrap_or_else(|| "unknown".into()),
            limits.budget_source,
            limits.max_width,
            sensor_dir
                .as_ref()
                .map(|d| d.display().to_string())
                .unwrap_or_else(|| "none (creep checks inert)".into()),
        );
        MemoryGovernor {
            limits,
            source: SignalSource { dir: sensor_dir },
            core: Mutex::new(GovCore::new()),
        }
    }

    /// One non-blocking admission poll (logs hard events and creep-episode back-offs).
    pub fn try_admit(&self) -> AdmitDecision {
        let sig = self.source.read();
        let mut core = self.core.lock().unwrap();
        let was_holding = core.holding;
        let old_target = core.target_width;
        let (decision, hard) = decide_admission(&mut core, &sig, &self.limits);
        let new_target = core.target_width;
        drop(core);
        if let Some(h) = hard {
            eprintln!("{}", h.describe());
        }
        if let AdmitDecision::Hold(reason) = &decision {
            if reason.is_memory_creep() && !was_holding {
                eprintln!(
                    "[governor] creep back-off: {} — target_width {}→{} (admissions held; workers drain between units)",
                    reason.describe(),
                    old_target,
                    new_target
                );
            }
        }
        if let AdmitDecision::Admit {
            forced_serial: true,
        } = &decision
        {
            eprintln!(
                "[governor] progress floor: signals hot but zero workers active — admitting one (counted forced_serial)"
            );
        }
        decision
    }

    /// Block until a worker slot is granted.
    pub fn admit_blocking(&self, label: &str) {
        loop {
            match self.try_admit() {
                AdmitDecision::Admit { .. } => return,
                AdmitDecision::Hold(reason) => {
                    if reason.is_memory_creep() {
                        // Edge already logged by try_admit; keep the label visible once.
                        let _ = label;
                    }
                    std::thread::sleep(HOLD_POLL);
                }
            }
        }
    }

    /// A worker finished one unit of work (a resolve-group, an entry-group): a SIGNAL POLL,
    /// not an increase point (see `CompletionKind::ObserveOnly` — a unit is not a worker).
    /// Reached through `AdmittedSlot::note_unit_complete` so the pacing bookkeeping cannot
    /// be skipped, and it still carries the creep/hard back-off arms, which at a full
    /// window may be the only polls the controller gets.
    fn note_unit_observed(&self) {
        self.observe(CompletionKind::ObserveOnly)
    }

    fn observe(&self, kind: CompletionKind) {
        let sig = self.source.read();
        let mut core = self.core.lock().unwrap();
        let (hard, creep_backoff) = note_completion(&mut core, &sig, &self.limits, kind);
        drop(core);
        if let Some(h) = hard {
            eprintln!("{}", h.describe());
        }
        if let Some((old, new)) = creep_backoff {
            eprintln!(
                "[governor] creep back-off (observed at completion): target_width {old}→{new} (admissions held; workers drain between units)"
            );
        }
    }

    fn note_first_cost_paid(&self) {
        let sig = self.source.read();
        let mut core = self.core.lock().unwrap();
        let had_share = core.measured_worker_share.is_some();
        note_first_cost(&mut core, &sig);
        let share = core.measured_worker_share;
        drop(core);
        if !had_share {
            if let Some(s) = share {
                eprintln!(
                    "[governor] measured worker share: {s} bytes (first slot's cost over the pool baseline) — headroom gate armed"
                );
            }
        }
        // THE additive-increase point. A landed cost is the only event that carries
        // evidence about the quantity the window denominates: this worker's demand is now
        // visible to the signals, so a calm, headroom-predicted poll here says one more
        // fits. Sequenced after `note_first_cost` so the prediction reads the share this
        // very landing measured, never a stale one.
        self.observe(CompletionKind::IncreasePoint);
    }

    /// A worker released its slot.
    fn release_slot(&self, first_cost_paid: bool) {
        let mut core = self.core.lock().unwrap();
        note_release(&mut core, first_cost_paid);
    }

    /// Multiplicative decrease drains through here: a worker between units retires when
    /// concurrency sits above the (possibly just-halved) window.
    pub fn should_retire(&self) -> bool {
        let core = self.core.lock().unwrap();
        core.active > core.target_width
    }

    pub fn active(&self) -> usize {
        self.core.lock().unwrap().active
    }

    pub fn current_target_width(&self) -> usize {
        self.core.lock().unwrap().target_width
    }

    /// The end-of-run receipt: the counted degradations (§5 — observable, prioritizable).
    pub fn receipt_line(&self) -> String {
        let core = self.core.lock().unwrap();
        format!(
            "[governor] receipt: budget={} source={} max_width_reached={} admissions={} \
             width_growths={} creep_backoffs={} pacing_holds={} headroom_holds={} \
             ceiling_holds={} hard_backoffs={} budget_exceeded={} forced_serial={} \
             peak_current={}",
            self.limits
                .budget_bytes
                .map(|b| b.to_string())
                .unwrap_or_else(|| "unknown".into()),
            self.limits.budget_source,
            core.max_width_reached,
            core.admissions,
            core.width_growths,
            core.creep_backoffs,
            core.pacing_holds,
            core.headroom_holds,
            core.ceiling_holds,
            core.hard_backoffs,
            core.budget_exceeded_backoffs,
            core.forced_serial_admissions,
            if core.peak_current_bytes == 0 {
                "unreadable".to_string()
            } else {
                core.peak_current_bytes.to_string()
            },
        )
    }
}

/// RAII slot guard: `release()` on drop so a panicking worker frees its slot and the
/// governor's active count cannot leak upward (which would wedge admissions). Owns an
/// `Arc` so it can ride into worker threads.
pub struct AdmittedSlot {
    governor: std::sync::Arc<MemoryGovernor>,
    first_cost_paid: bool,
}

impl AdmittedSlot {
    /// Wrap a slot that `try_admit`/`admit_blocking` ALREADY granted (the grant is what
    /// incremented `active`; this guard owns the matching release).
    pub fn from_admitted(governor: std::sync::Arc<MemoryGovernor>) -> AdmittedSlot {
        AdmittedSlot {
            governor,
            first_cost_paid: false,
        }
    }

    /// Block until admitted, then wrap the slot.
    pub fn acquire_blocking(
        governor: &std::sync::Arc<MemoryGovernor>,
        label: &str,
    ) -> AdmittedSlot {
        governor.admit_blocking(label);
        AdmittedSlot {
            governor: governor.clone(),
            first_cost_paid: false,
        }
    }

    /// The worker's front-loaded admission cost is paid (its index build returned):
    /// admission pacing may pass the next worker. Idempotent.
    pub fn note_first_cost_paid(&mut self) {
        if !self.first_cost_paid {
            self.first_cost_paid = true;
            self.governor.note_first_cost_paid();
        }
    }

    /// One unit of work completed under this slot: a signal poll, not an increase point.
    /// It still settles the first cost for slots whose first unit IS their front cost
    /// (gate resolve-groups) — and THAT landing is the increase point, so a slot's first
    /// unit grows the window exactly once and its remaining units only observe. No caller
    /// can grow the window while still owing pacing debt, and none can grow it per-unit.
    pub fn note_unit_complete(&mut self) {
        self.note_first_cost_paid();
        self.governor.note_unit_observed();
    }
}

impl Drop for AdmittedSlot {
    fn drop(&mut self) {
        self.governor.release_slot(self.first_cost_paid);
    }
}

// ---- shared cgroup sensor primitives (single authority; the executor's heartbeat and
// ---- measurement lines consume these same readers) ----

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority: `dag/gunbc/runner_slot_allocation.dag`
/// `gunbc_runner_slot_desired().memory_high` = `byte_size(16106127360)`; corroborated by
/// `gunbc.ci_floor_measurement.gunbc_ci_runner_slot_memory_high_live`, which derives from
/// the same row and forbids a duplicate literal):
/// uncapped-host MemAvailable cap for `read_host_budget_bytes`. When no cgroup limit
/// binds, raw MemAvailable must not be the sole budget — uncapped hosts let the floor
/// reach physical RAM and OOM-kill (runs 29180195694, srv3 2026-07-21 exit-137).
/// dissolve-on: v2 emit of stage0 host-budget constants from `gunbc.runner_slot_allocation`
/// (self-host frontier row for `memory_governor` cgroup-budget readers).
pub const DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES: u64 = 16_106_127_360;

/// Cap an uncapped-host MemAvailable sample at the declared runner-slot throttle
/// line. Returns `(budget, capped)` where `capped` is true when `avail` exceeded
/// the declaration.
pub fn uncapped_host_budget_from_mem_available(avail: u64) -> (u64, bool) {
    if avail > DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES {
        (DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES, true)
    } else {
        (avail, false)
    }
}

/// The host memory budget in bytes to admit against: env override -> cgroup
/// memory.high -> memory.max -> min(MemAvailable, declared runner-slot memory.high)
/// -> MemTotal, with a readable source label. Single authority shared by the
/// MemoryGovernor (which SCHEDULES against it) and the P4 realize advisory (which
/// PREDICTS against it) — the advisory must price against the same budget the
/// governor uses, never a partial re-read (§3 single authority).
pub fn read_host_budget_bytes() -> (Option<u64>, String) {
    if let Some(b) = std::env::var("GUNBC_MEMORY_BUDGET_BYTES")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
    {
        return (Some(b), "env GUNBC_MEMORY_BUDGET_BYTES".to_string());
    }
    if let Some(dir) = binding_high_cgroup_dir() {
        return (
            read_cgroup_u64(&dir, "memory.high"),
            format!("cgroup memory.high ({})", dir.display()),
        );
    }
    if let Some(dir) = binding_cap_cgroup_dir() {
        return (
            read_cgroup_u64(&dir, "memory.max"),
            format!("cgroup memory.max ({})", dir.display()),
        );
    }
    if let Some(avail) = mem_available_bytes() {
        let (budget, capped) = uncapped_host_budget_from_mem_available(avail);
        let source = if capped {
            format!(
                "min(MemAvailable, declared runner-slot memory.high {} bytes)",
                DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES
            )
        } else {
            "/proc/meminfo MemAvailable".to_string()
        };
        return (Some(budget), source);
    }
    (mem_total_bytes(), "/proc/meminfo MemTotal".to_string())
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

/// Total physical RAM in bytes (`/proc/meminfo` MemTotal, kB→bytes) — the last-resort
/// budget when no cgroup limit binds and MemAvailable is unreadable.
pub fn mem_total_bytes() -> Option<u64> {
    meminfo_field_bytes_in(&std::fs::read_to_string("/proc/meminfo").ok()?, "MemTotal")
}

/// The kernel's estimate of allocatable memory without swapping (`/proc/meminfo`
/// MemAvailable, kB→bytes), sampled at arm time — the honest budget on uncapped hosts.
pub fn mem_available_bytes() -> Option<u64> {
    meminfo_field_bytes_in(
        &std::fs::read_to_string("/proc/meminfo").ok()?,
        "MemAvailable",
    )
}

fn meminfo_field_bytes_in(meminfo: &str, key: &str) -> Option<u64> {
    let line = meminfo
        .lines()
        .find(|l| l.strip_prefix(key).is_some_and(|r| r.starts_with(':')))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(budget: u64, max_width: usize) -> GovernorLimits {
        GovernorLimits {
            budget_bytes: Some(budget),
            budget_source: "test".into(),
            max_width,
        }
    }

    fn calm() -> MemorySignals {
        MemorySignals {
            current_bytes: Some(1_000),
            swap_current_bytes: Some(0),
            psi_some_avg10: Some(0.0),
            events_high: Some(0),
            events_oom_kill: Some(0),
        }
    }

    #[test]
    fn uncapped_host_budget_caps_mem_available_at_declared_runner_slot_high() {
        let (budget, capped) = uncapped_host_budget_from_mem_available(35_161_423_872);
        assert_eq!(budget, DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES);
        assert!(capped);
        let (small, capped_small) = uncapped_host_budget_from_mem_available(8_000_000_000);
        assert_eq!(small, 8_000_000_000);
        assert!(!capped_small);
    }

    #[test]
    fn calm_signals_admit_and_grow_additively() {
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        let (d, _) = decide_admission(&mut core, &calm(), &lim);
        assert_eq!(
            d,
            AdmitDecision::Admit {
                forced_serial: false
            }
        );
        // Window is 1 and it's occupied: second admission holds on window arithmetic.
        let (d2, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(
            d2,
            AdmitDecision::Hold(HoldReason::WindowFull { .. })
        ));
        assert_eq!(core.creep_backoffs, 0, "window-full is not a memory hold");
        // The first worker's front cost lands: THAT is the increase point, and with the
        // pacing debt settled the next admission fits.
        note_first_cost(&mut core, &calm());
        note_completion(&mut core, &calm(), &lim, CompletionKind::IncreasePoint);
        assert_eq!(core.target_width, 2);
        assert_eq!(core.width_growths, 1);
        let (d3, _) = decide_admission(&mut core, &calm(), &lim);
        assert_eq!(
            d3,
            AdmitDecision::Admit {
                forced_serial: false
            }
        );
        assert_eq!(core.active, 2);
        assert_eq!(core.max_width_reached, 2);
    }

    #[test]
    fn creep_is_never_grown_through_at_an_increase_point() {
        // Calm is a PRECONDITION of additive increase, not a property of which event
        // carries the evidence: an increase point observed under creep backs the window
        // off, exactly as the admission path would. The symmetric case (creep seen at an
        // observe-only unit poll) is asserted in
        // `the_window_tracks_worker_cost_not_the_unit_completion_rate`.
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        assert_eq!(core.target_width, 1, "a run starts at the width-1 floor");
        note_completion(&mut core, &calm(), &lim, CompletionKind::IncreasePoint);
        assert_eq!(
            core.target_width, 2,
            "a calm increase point lifts the window"
        );
        assert_eq!(core.width_growths, 1);
        // RED control: the same event under creep must NOT be grown through.
        let mut hot_core = GovCore::new();
        let hot = MemorySignals {
            current_bytes: Some(900_000), // > 800_000 high-water
            ..calm()
        };
        note_completion(&mut hot_core, &hot, &lim, CompletionKind::IncreasePoint);
        assert_eq!(hot_core.target_width, 1, "creep is never grown through");
        assert_eq!(hot_core.creep_backoffs, 1);
        assert_eq!(hot_core.width_growths, 0);
    }

    #[test]
    fn the_window_tracks_worker_cost_not_the_unit_completion_rate() {
        // `target_width` is denominated in WORKER-SIZED residency claims. Growing it per
        // unit of work conflated two grains and made the un-latched window unsurvivable:
        // units are numerous and mostly cheap, so the window tracked the completion rate
        // instead of realized affordability (CI run 29710324768 — off the floor 34ms into
        // the corpus, then admitted against that fiction to 101.6 GB and OOM-killed).
        let lim = limits(1_000_000, 128);
        let mut core = GovCore::new();
        // A worker is admitted and lands its cost: ONE growth, the real increase point.
        let (_, _) = decide_admission(&mut core, &calm(), &lim);
        note_first_cost(&mut core, &calm());
        note_completion(&mut core, &calm(), &lim, CompletionKind::IncreasePoint);
        assert_eq!(core.target_width, 2);
        // That worker now completes a long run of cheap units under perfectly calm
        // signals. None of them is evidence about worker-sized residency, so none of them
        // may move the window — this is the assertion that goes RED on the grain
        // conflation (pre-fix this reached max_width, here 128, in 100 completions).
        for _ in 0..100 {
            note_completion(&mut core, &calm(), &lim, CompletionKind::ObserveOnly);
        }
        assert_eq!(
            core.target_width, 2,
            "cheap unit completions must not ramp the window"
        );
        assert_eq!(core.width_growths, 1, "exactly one landed cost, one growth");
        // ...and observe-only polls are still real polls: a creep edge seen at a unit
        // completion must back the window off exactly as it would at an increase point.
        let hot = MemorySignals {
            current_bytes: Some(900_000), // > 800_000 high-water
            ..calm()
        };
        note_completion(&mut core, &hot, &lim, CompletionKind::ObserveOnly);
        assert_eq!(core.creep_backoffs, 1, "a unit poll still sees congestion");
        assert!(core.target_width < 2, "and still backs the window off");
    }

    #[test]
    fn growth_never_outruns_the_headroom_the_admission_gate_requires() {
        // The increase arm and the admission gate must answer "does one more worker fit"
        // from ONE authority (§3). Before this, admission predicted `current + share <=
        // high_water` while increase predicted nothing, so the window could grow into a
        // state admission would refuse on the very next poll — a ramp that means nothing.
        let lim = limits(1_000_000, 128);
        let mut core = GovCore::new();
        let (_, _) = decide_admission(&mut core, &calm(), &lim);
        // A worker lands, and it turns out to be expensive: 700_000 over the baseline,
        // against a high-water of 800_000. One more of those does NOT fit.
        let landed = MemorySignals {
            current_bytes: Some(700_000),
            ..calm()
        };
        note_first_cost(&mut core, &landed);
        assert_eq!(core.measured_worker_share, Some(700_000 - 1_000));
        note_completion(&mut core, &landed, &lim, CompletionKind::IncreasePoint);
        assert_eq!(
            core.target_width, 1,
            "an increase point with no headroom must not grow the window"
        );
        assert_eq!(core.width_growths, 0);
        // The admission gate agrees — the two arms cannot disagree, they share the fn.
        assert!(matches!(
            headroom_hold(&core, &landed, &lim),
            Some(HoldReason::InsufficientHeadroom { .. })
        ));
        // Once the pool drains back down, the same landed share now fits and growth resumes.
        let drained = MemorySignals {
            current_bytes: Some(50_000),
            ..calm()
        };
        note_completion(&mut core, &drained, &lim, CompletionKind::IncreasePoint);
        assert_eq!(core.target_width, 2, "headroom recovered, the ramp resumes");
    }

    #[test]
    fn pacing_holds_admissions_until_first_cost_paid() {
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        // Worker 1 admitted; the window has room (a gate unit's calm completion grew it)
        // but worker 1 has not paid its index build yet.
        let (d, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(d, AdmitDecision::Admit { .. }));
        core.target_width = 4;
        let (d2, _) = decide_admission(&mut core, &calm(), &lim);
        assert_eq!(
            d2,
            AdmitDecision::Hold(HoldReason::AwaitFirstCost { undigested: 1 }),
            "calm signals must not outrun the un-landed admission cost"
        );
        let (d3, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(d3, AdmitDecision::Hold(_)));
        assert_eq!(core.pacing_holds, 1, "one episode, not one per poll");
        assert_eq!(core.creep_backoffs, 0, "pacing is not a memory hold");
        // The index build lands: the next admission passes and re-arms the episode.
        note_first_cost(&mut core, &calm());
        let (d4, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(d4, AdmitDecision::Admit { .. }));
        assert_eq!(
            core.undigested, 1,
            "the new worker now owes its own first cost"
        );
        let (d5, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(
            d5,
            AdmitDecision::Hold(HoldReason::AwaitFirstCost { .. })
        ));
        assert_eq!(core.pacing_holds, 2);
    }

    #[test]
    fn budget_exceed_is_a_hard_event_once_per_episode_with_hysteresis() {
        let lim = limits(1_000_000, 16);
        let mut core = GovCore::new();
        core.target_width = 8;
        core.active = 8;
        // Current above the DECLARED BUDGET (not merely high-water): hard event — halve.
        let over = MemorySignals {
            current_bytes: Some(1_000_001),
            ..calm()
        };
        let (d, _) = decide_admission(&mut core, &over, &lim);
        assert!(matches!(d, AdmitDecision::Hold(_)));
        assert_eq!(
            core.target_width, 1,
            "budget exceed is the timeout signal: collapse to 1, not a halve"
        );
        assert_eq!(core.hard_backoffs, 1);
        assert_eq!(core.budget_exceeded_backoffs, 1);
        // The drain path is reachable: concurrency now sits above the collapsed window.
        assert!(core.active > core.target_width, "workers must drain");
        // Still over on the next poll: same episode, no second event.
        let (_, h2) = decide_admission(&mut core, &over, &lim);
        assert!(h2.is_none());
        assert_eq!(core.target_width, 1);
        assert_eq!(core.budget_exceeded_backoffs, 1);
        // Dropping under the budget but ABOVE high-water does not re-arm (hysteresis)…
        core.active = 4;
        let between = MemorySignals {
            current_bytes: Some(900_000),
            ..calm()
        };
        let (_, h3) = decide_admission(&mut core, &between, &lim);
        assert!(h3.is_none());
        let (_, h4) = decide_admission(&mut core, &over, &lim);
        assert!(h4.is_none(), "not re-armed while above high-water");
        // …falling under high-water re-arms; a second crossing halves again.
        let calm_low = MemorySignals {
            current_bytes: Some(100_000),
            ..calm()
        };
        let (_, _) = decide_admission(&mut core, &calm_low, &lim);
        let (_, h5) = decide_admission(&mut core, &over, &lim);
        assert!(h5.is_some(), "re-armed below high-water");
        assert_eq!(core.target_width, 1);
        assert_eq!(core.budget_exceeded_backoffs, 2);
    }

    #[test]
    fn measured_share_headroom_gate_predicts_before_high_water() {
        let lim = limits(1_000_000, 8); // high-water 800_000
        let mut core = GovCore::new();
        // First poll fixes the pool baseline (100k) and admits worker 1.
        let at = |current: u64| MemorySignals {
            current_bytes: Some(current),
            ..calm()
        };
        let (d, _) = decide_admission(&mut core, &at(100_000), &lim);
        assert!(matches!(d, AdmitDecision::Admit { .. }));
        // Worker 1's index build lands at 400k: measured share = 300k, gate armed.
        note_first_cost(&mut core, &at(400_000));
        assert_eq!(core.measured_worker_share, Some(300_000));
        core.target_width = 8;
        // 400k + 300k fits under 800k: worker 2 admitted.
        let (d2, _) = decide_admission(&mut core, &at(400_000), &lim);
        assert!(matches!(d2, AdmitDecision::Admit { .. }));
        note_first_cost(&mut core, &at(400_000));
        // 600k is still BELOW high-water, but 600k + 300k predicts a crossing: hold
        // before the reactive arms would ever see it.
        let (d3, _) = decide_admission(&mut core, &at(600_000), &lim);
        assert_eq!(
            d3,
            AdmitDecision::Hold(HoldReason::InsufficientHeadroom {
                current: 600_000,
                share: 300_000,
                high_water: 800_000,
            })
        );
        let (d4, _) = decide_admission(&mut core, &at(600_000), &lim);
        assert!(matches!(d4, AdmitDecision::Hold(_)));
        assert_eq!(core.headroom_holds, 1, "one episode, not one per poll");
        assert_eq!(core.creep_backoffs, 0, "prediction is not a creep event");
        assert_eq!(
            core.target_width, 8,
            "prediction does not shrink the window"
        );
        // Demand settles to 450k: 450k + 300k fits again — admission resumes.
        let (d5, _) = decide_admission(&mut core, &at(450_000), &lim);
        assert!(matches!(d5, AdmitDecision::Admit { .. }));
    }

    #[test]
    fn admission_ceiling_reserves_half_the_budget_for_maturation() {
        let lim = limits(1_000_000, 8); // ceiling 500_000, high-water 800_000
        let mut core = GovCore::new();
        core.target_width = 8;
        core.active = 3;
        let at = |current: u64| MemorySignals {
            current_bytes: Some(current),
            ..calm()
        };
        // 510k is calm by every reactive measure (below high-water, no PSI, no swap),
        // but past the ceiling: the maturation reserve is spoken for — hold.
        let (d, _) = decide_admission(&mut core, &at(510_000), &lim);
        assert_eq!(
            d,
            AdmitDecision::Hold(HoldReason::AdmissionCeiling {
                current: 510_000,
                ceiling: 500_000,
            })
        );
        let (d2, _) = decide_admission(&mut core, &at(510_000), &lim);
        assert!(matches!(d2, AdmitDecision::Hold(_)));
        assert_eq!(core.ceiling_holds, 1, "one episode, not one per poll");
        assert_eq!(core.creep_backoffs, 0, "the ceiling is not a creep event");
        assert_eq!(
            core.target_width, 8,
            "the ceiling does not shrink the window"
        );
        // Demand matures and drains below the line: admission resumes.
        let (d3, _) = decide_admission(&mut core, &at(490_000), &lim);
        assert!(matches!(d3, AdmitDecision::Admit { .. }));
    }

    #[test]
    fn release_before_first_cost_clears_pacing_debt() {
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        let (d, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(d, AdmitDecision::Admit { .. }));
        core.target_width = 4;
        // Worker 1 dies before its index build returns (error/panic): its release must
        // clear the pacing debt or admissions freeze behind a dead worker.
        note_release(&mut core, false);
        assert_eq!(core.active, 0);
        assert_eq!(core.undigested, 0);
        let (d2, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(d2, AdmitDecision::Admit { .. }));
        // A digested worker's release does NOT touch the (absent) debt.
        note_first_cost(&mut core, &calm());
        note_release(&mut core, true);
        assert_eq!(core.undigested, 0);
    }

    #[test]
    fn high_water_creep_edge_backs_off_once_per_episode() {
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        core.target_width = 4;
        core.active = 3;
        let hot = MemorySignals {
            current_bytes: Some(900_000), // > 800_000 high-water of 1_000_000
            ..calm()
        };
        // The episode EDGE is the early-congestion signal: hold AND halve, so workers
        // drain while the high-water→budget buffer still has drain-latency margin.
        let (d, _) = decide_admission(&mut core, &hot, &lim);
        assert!(matches!(
            d,
            AdmitDecision::Hold(HoldReason::CurrentHighWater { .. })
        ));
        assert_eq!(core.target_width, 2, "creep edge halves the window");
        assert!(core.active > core.target_width, "workers must drain");
        let (d2, _) = decide_admission(&mut core, &hot, &lim);
        assert!(matches!(d2, AdmitDecision::Hold(_)));
        assert_eq!(core.creep_backoffs, 1, "one episode, not one per poll");
        assert_eq!(core.target_width, 2, "no re-halve within an episode");
        // Drain to calm: workers retire, an admission re-arms the episode, and a second
        // hot spell counts (and halves) again.
        note_release(&mut core, true);
        note_release(&mut core, true);
        note_release(&mut core, true);
        let (d3, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(d3, AdmitDecision::Admit { .. }));
        note_first_cost(&mut core, &calm());
        let (_, _) = decide_admission(&mut core, &hot, &lim);
        assert_eq!(core.creep_backoffs, 2);
        assert_eq!(core.target_width, 1);
    }

    #[test]
    fn psi_pressure_holds_admission() {
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        core.target_width = 4;
        core.active = 2;
        let hot = MemorySignals {
            psi_some_avg10: Some(37.5),
            ..calm()
        };
        let (d, _) = decide_admission(&mut core, &hot, &lim);
        assert_eq!(
            d,
            AdmitDecision::Hold(HoldReason::PsiPressure { avg10: 37.5 })
        );
    }

    #[test]
    fn swap_growth_holds_on_delta_not_level() {
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        core.target_width = 4;
        core.active = 1;
        // Establish a high-but-stable swap level: first sample sets the baseline.
        let steady = MemorySignals {
            swap_current_bytes: Some(500 * 1024 * 1024),
            ..calm()
        };
        let (d, _) = decide_admission(&mut core, &steady, &lim);
        assert!(
            matches!(d, AdmitDecision::Admit { .. }),
            "absolute swap level alone does not hold"
        );
        note_first_cost(&mut core, &calm());
        // Same level again: no growth, no hold.
        let (d2, _) = decide_admission(&mut core, &steady, &lim);
        assert!(matches!(d2, AdmitDecision::Admit { .. }));
        note_first_cost(&mut core, &calm());
        // Now it grows by 100 MiB in one sample: actively creeping — hold.
        let grown = MemorySignals {
            swap_current_bytes: Some(600 * 1024 * 1024),
            ..calm()
        };
        let (d3, _) = decide_admission(&mut core, &grown, &lim);
        assert!(matches!(
            d3,
            AdmitDecision::Hold(HoldReason::SwapGrowth { .. })
        ));
    }

    #[test]
    fn hard_event_halves_target_with_floor_one() {
        let lim = limits(1_000_000, 16);
        let mut core = GovCore::new();
        core.target_width = 8;
        core.active = 8;
        // Baseline the counters first (cumulative history is not an event).
        let (d0, h0) = decide_admission(
            &mut core,
            &MemorySignals {
                events_high: Some(4821),
                ..calm()
            },
            &lim,
        );
        assert!(
            h0.is_none(),
            "first sight of a counter is baseline, not an event"
        );
        assert!(matches!(
            d0,
            AdmitDecision::Hold(HoldReason::WindowFull { .. })
        ));
        assert_eq!(core.target_width, 8);
        // The counter moves: hard event, window halves against active.
        let (_, h1) = decide_admission(
            &mut core,
            &MemorySignals {
                events_high: Some(4822),
                ..calm()
            },
            &lim,
        );
        assert_eq!(
            h1,
            Some(HardEvent {
                high_delta: 1,
                oom_kill_delta: 0,
                budget_exceeded: false,
                old_target: 8,
                new_target: 4
            })
        );
        assert_eq!(core.hard_backoffs, 1);
        // Repeated events keep halving down to the floor of 1.
        core.active = 1;
        for step in 0..4u64 {
            let (_, _) = decide_admission(
                &mut core,
                &MemorySignals {
                    events_high: Some(4823 + step),
                    ..calm()
                },
                &lim,
            );
        }
        assert_eq!(core.target_width, 1);
    }

    #[test]
    fn oom_kill_delta_is_a_hard_event() {
        let lim = limits(1_000_000, 8);
        let mut core = GovCore::new();
        core.target_width = 6;
        core.active = 6;
        let (_, h0) = decide_admission(
            &mut core,
            &MemorySignals {
                events_oom_kill: Some(0),
                ..calm()
            },
            &lim,
        );
        assert!(h0.is_none());
        let (_, h1) = decide_admission(
            &mut core,
            &MemorySignals {
                events_oom_kill: Some(1),
                ..calm()
            },
            &lim,
        );
        assert_eq!(
            h1,
            Some(HardEvent {
                high_delta: 0,
                oom_kill_delta: 1,
                budget_exceeded: false,
                old_target: 6,
                new_target: 3
            })
        );
    }

    #[test]
    fn zero_active_always_admits_as_progress_floor() {
        let lim = limits(1_000, 8);
        let mut core = GovCore::new();
        let hot = MemorySignals {
            current_bytes: Some(999),
            psi_some_avg10: Some(90.0),
            ..calm()
        };
        let (d, _) = decide_admission(&mut core, &hot, &lim);
        assert_eq!(
            d,
            AdmitDecision::Admit {
                forced_serial: true
            }
        );
        assert_eq!(core.forced_serial_admissions, 1);
        assert_eq!(core.active, 1);
    }

    #[test]
    fn unreadable_signals_are_inert_never_fabricated() {
        let lim = GovernorLimits {
            budget_bytes: None,
            budget_source: "test-none".into(),
            max_width: 4,
        };
        let mut core = GovCore::new();
        core.target_width = 2;
        core.active = 1;
        let blind = MemorySignals::default();
        let (d, h) = decide_admission(&mut core, &blind, &lim);
        assert!(h.is_none());
        assert!(
            matches!(
                d,
                AdmitDecision::Admit {
                    forced_serial: false
                }
            ),
            "no readable signal ≠ hot; window arithmetic still governs"
        );
    }

    #[test]
    fn receipt_line_carries_the_counted_degradations() {
        let gov = MemoryGovernor {
            limits: limits(123_456, 4),
            source: SignalSource { dir: None },
            core: Mutex::new({
                let mut c = GovCore::new();
                c.creep_backoffs = 3;
                c.pacing_holds = 5;
                c.hard_backoffs = 2;
                c.budget_exceeded_backoffs = 7;
                c.forced_serial_admissions = 1;
                c.admissions = 40;
                c.max_width_reached = 4;
                c.peak_current_bytes = 99;
                c
            }),
        };
        let line = gov.receipt_line();
        assert!(line.contains("creep_backoffs=3"));
        assert!(line.contains("pacing_holds=5"));
        assert!(line.contains("hard_backoffs=2"));
        assert!(line.contains("budget_exceeded=7"));
        assert!(line.contains("forced_serial=1"));
        assert!(line.contains("admissions=40"));
        assert!(line.contains("budget=123456"));
        assert!(line.contains("peak_current=99"));
    }

    #[test]
    fn events_parse_reads_the_named_counter() {
        let content = "low 0\nhigh 4821\nmax 12\noom 0\noom_kill 0\n";
        assert_eq!(memory_events_field(content, "high"), Some(4821));
        assert_eq!(memory_events_field(content, "oom_kill"), Some(0));
        assert_eq!(memory_events_field(content, "absent_key"), None);
    }

    #[test]
    fn events_parse_does_not_prefix_match_keys() {
        let content = "oom 3\noom_kill 1\n";
        assert_eq!(memory_events_field(content, "oom"), Some(3));
        assert_eq!(memory_events_field(content, "oom_kill"), Some(1));
        // `oom` must not read the `oom_kill` line when only the latter exists.
        assert_eq!(memory_events_field("oom_kill 7\n", "oom"), None);
    }

    #[test]
    fn meminfo_parse_reads_exact_keys() {
        let content = "MemTotal:       53570453 kB\nMemFree:        1200000 kB\nMemAvailable:   45000000 kB\n";
        assert_eq!(
            meminfo_field_bytes_in(content, "MemTotal"),
            Some(53570453 * 1024)
        );
        assert_eq!(
            meminfo_field_bytes_in(content, "MemAvailable"),
            Some(45000000 * 1024)
        );
        // Exact key match: `Mem` must not read the MemTotal line.
        assert_eq!(meminfo_field_bytes_in(content, "Mem"), None);
        assert_eq!(meminfo_field_bytes_in(content, "SwapTotal"), None);
    }

    #[test]
    fn psi_parse_extracts_from_psi_shape() {
        let content = "some avg10=1.23 avg60=0.50 total=1234\nfull avg10=0.00 avg60=0.00 total=0\n";
        assert_eq!(
            memory_pressure_some_avg10(content),
            Some("1.23".to_string())
        );
        assert_eq!(memory_pressure_some_avg10("garbage\n"), None);
    }
}
