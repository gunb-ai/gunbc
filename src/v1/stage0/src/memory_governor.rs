//! AIMD memory governor: adaptive worker admission against the slot's declared memory budget.
//!
//! Replaces the hand-pinned predicted-peak width folds (the retired
//! `gunbc.ci_floor_measurement` width path) with a TCP-shaped controller (operator model,
//! 2026-07-12): the budget is the pipe — the slot's own cgroup limits, so the fleet
//! allocation IS the declaration and no byte constant lives in the tree — work is admitted
//! while signals are calm, and the controller backs off on CREEP before the kill line.
//! Because `memory.high` reclaim (and swap, where present) buffer the overshoot, creep is
//! not loss: the graceful arm stops admitting and lets in-flight work drain; only hard
//! events (throttle-line crossings, OOM kills) shrink the window multiplicatively.
//!
//! Signals, read from the tightest-limit cgroup directory (else the leaf):
//!   `memory.current`             — instantaneous usage vs budget (high-water headroom)
//!   `memory.swap.current`        — per-sample growth = actively creeping past physical
//!   `memory.pressure` some avg10 — kernel reclaim stall %, the earliest creep signal
//!                                  (present with or without swap)
//!   `memory.events` high/oom_kill — cumulative counters; a positive delta is a hard event
//!
//! No-swap regimes degrade honestly: an absent `memory.swap.current` makes the swap check
//! inert (`None`, never a fabricated zero) while current + PSI carry the decision; no
//! readable cgroup at all falls back to `/proc/meminfo` MemTotal for the budget and leaves
//! the creep checks inert — the window arithmetic (and the kill line) still governs.
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
        }
    }
    fn is_memory_creep(&self) -> bool {
        !matches!(self, HoldReason::WindowFull { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdmitDecision {
    /// A worker slot was granted. `forced_serial` marks the width-1 progress floor firing
    /// while signals were hot (counted in the receipt).
    Admit { forced_serial: bool },
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
    admissions: u64,
    forced_serial_admissions: u64,
    graceful_holds: u64,
    hard_backoffs: u64,
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
            admissions: 0,
            forced_serial_admissions: 0,
            graceful_holds: 0,
            hard_backoffs: 0,
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
    old_target: usize,
    new_target: usize,
}

/// The governor's static configuration: the declared pipe.
#[derive(Debug, Clone)]
pub struct GovernorLimits {
    pub budget_bytes: Option<u64>,
    pub budget_source: String,
    pub max_width: usize,
}

/// Fold one sample's cumulative counters into the core: first sight sets the baseline
/// (the counters count since cgroup creation, so a nonzero first read is history, not an
/// event); a later positive delta is a hard event and halves the window (floor 1).
fn absorb_hard_events(core: &mut GovCore, sig: &MemorySignals) -> Option<HardEvent> {
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
    if high_delta == 0 && oom_delta == 0 {
        return None;
    }
    let old_target = core.target_width;
    core.target_width = core.active.div_ceil(2).max(1).min(old_target.max(1));
    core.hard_backoffs += 1;
    Some(HardEvent {
        high_delta,
        oom_kill_delta: oom_delta,
        old_target,
        new_target: core.target_width,
    })
}

/// The graceful arm: is this sample creeping? Reads `prev_swap_bytes` (last sample) and
/// leaves updating it to the caller so the delta is per-sample, not per-check.
fn creep_reason(core: &GovCore, sig: &MemorySignals, limits: &GovernorLimits) -> Option<HoldReason> {
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
    let hard = absorb_hard_events(core, sig);
    let creep = creep_reason(core, sig, limits);
    core.prev_swap_bytes = sig.swap_current_bytes.or(core.prev_swap_bytes);
    if core.active == 0 {
        let forced = creep.is_some();
        core.active = 1;
        core.admissions += 1;
        if forced {
            core.forced_serial_admissions += 1;
        }
        core.holding = false;
        core.max_width_reached = core.max_width_reached.max(core.active);
        return (AdmitDecision::Admit {
            forced_serial: forced,
        }, hard);
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
    if let Some(reason) = creep {
        if !core.holding {
            core.holding = true;
            core.graceful_holds += 1;
        }
        return (AdmitDecision::Hold(reason), hard);
    }
    core.active += 1;
    core.admissions += 1;
    core.holding = false;
    core.max_width_reached = core.max_width_reached.max(core.active);
    (AdmitDecision::Admit {
        forced_serial: false,
    }, hard)
}

/// Additive increase, gated on calm: a completed unit of work while no creep is visible
/// grows the window by one, up to the CPU bound.
fn note_completion(core: &mut GovCore, sig: &MemorySignals, limits: &GovernorLimits) {
    let hard = absorb_hard_events(core, sig);
    let creep = creep_reason(core, sig, limits);
    core.prev_swap_bytes = sig.swap_current_bytes.or(core.prev_swap_bytes);
    if hard.is_none() && creep.is_none() && core.target_width < limits.max_width {
        core.target_width += 1;
        core.width_growths += 1;
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
        let env_budget = std::env::var("GUNBC_MEMORY_BUDGET_BYTES")
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok());
        let high_dir = binding_high_cgroup_dir();
        let cap_dir = binding_cap_cgroup_dir();
        let (budget, source_label) = if let Some(b) = env_budget {
            (Some(b), "env GUNBC_MEMORY_BUDGET_BYTES".to_string())
        } else if let Some(dir) = &high_dir {
            (
                read_cgroup_u64(dir, "memory.high"),
                format!("cgroup memory.high ({})", dir.display()),
            )
        } else if let Some(dir) = &cap_dir {
            (
                read_cgroup_u64(dir, "memory.max"),
                format!("cgroup memory.max ({})", dir.display()),
            )
        } else {
            (mem_total_bytes(), "/proc/meminfo MemTotal".to_string())
        };
        let sensor_dir = high_dir.or(cap_dir).or_else(leaf_cgroup_dir);
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

    /// One non-blocking admission poll (logs hard events and hold-episode edges).
    pub fn try_admit(&self) -> AdmitDecision {
        let sig = self.source.read();
        let mut core = self.core.lock().unwrap();
        let was_holding = core.holding;
        let (decision, hard) = decide_admission(&mut core, &sig, &self.limits);
        drop(core);
        if let Some(h) = hard {
            eprintln!(
                "[governor] hard back-off: memory.events high +{} oom_kill +{} — target_width {}→{}",
                h.high_delta, h.oom_kill_delta, h.old_target, h.new_target
            );
        }
        if let AdmitDecision::Hold(reason) = &decision {
            if reason.is_memory_creep() && !was_holding {
                eprintln!("[governor] holding admissions: {}", reason.describe());
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

    /// A worker finished one unit of work (a resolve-group, an entry-group): the additive
    /// increase point.
    pub fn note_unit_complete(&self) {
        let sig = self.source.read();
        let mut core = self.core.lock().unwrap();
        note_completion(&mut core, &sig, &self.limits);
    }

    /// A worker released its slot.
    pub fn release(&self) {
        let mut core = self.core.lock().unwrap();
        core.active = core.active.saturating_sub(1);
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
             width_growths={} graceful_holds={} hard_backoffs={} forced_serial={} peak_current={}",
            self.limits
                .budget_bytes
                .map(|b| b.to_string())
                .unwrap_or_else(|| "unknown".into()),
            self.limits.budget_source,
            core.max_width_reached,
            core.admissions,
            core.width_growths,
            core.graceful_holds,
            core.hard_backoffs,
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
/// governor's active count cannot leak upward (which would wedge admissions).
pub struct GovernorSlot<'a> {
    governor: &'a MemoryGovernor,
}

impl<'a> GovernorSlot<'a> {
    pub fn acquire(governor: &'a MemoryGovernor, label: &str) -> GovernorSlot<'a> {
        governor.admit_blocking(label);
        GovernorSlot { governor }
    }
}

impl Drop for GovernorSlot<'_> {
    fn drop(&mut self) {
        self.governor.release();
    }
}

// ---- shared cgroup sensor primitives (single authority; the executor's heartbeat and
// ---- measurement lines consume these same readers) ----

/// The cgroup directory whose `memory.max` is the TIGHTEST along the `/proc/self/cgroup`
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

/// Total physical RAM in bytes (`/proc/meminfo` MemTotal, kB→bytes) — the effective
/// budget when no cgroup limit binds.
pub fn mem_total_bytes() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = s.lines().find(|l| l.starts_with("MemTotal"))?;
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
        assert_eq!(core.graceful_holds, 0, "window-full is not a memory hold");
        // A calm completion grows the window; the next admission fits.
        note_completion(&mut core, &calm(), &lim);
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
    fn high_water_creep_holds_and_counts_once_per_episode() {
        let lim = limits(1_000, 8);
        let mut core = GovCore::new();
        core.target_width = 4;
        core.active = 1;
        let hot = MemorySignals {
            current_bytes: Some(900), // > 800 high-water of 1000
            ..calm()
        };
        let (d, _) = decide_admission(&mut core, &hot, &lim);
        assert!(matches!(
            d,
            AdmitDecision::Hold(HoldReason::CurrentHighWater { .. })
        ));
        let (d2, _) = decide_admission(&mut core, &hot, &lim);
        assert!(matches!(d2, AdmitDecision::Hold(_)));
        assert_eq!(core.graceful_holds, 1, "one episode, not one per poll");
        // Calm again: admit, episode re-arms, a second hot spell counts again.
        let (d3, _) = decide_admission(&mut core, &calm(), &lim);
        assert!(matches!(d3, AdmitDecision::Admit { .. }));
        let (_, _) = decide_admission(&mut core, &hot, &lim);
        assert_eq!(core.graceful_holds, 2);
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
        // Same level again: no growth, no hold.
        let (d2, _) = decide_admission(&mut core, &steady, &lim);
        assert!(matches!(d2, AdmitDecision::Admit { .. }));
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
        assert!(h0.is_none(), "first sight of a counter is baseline, not an event");
        assert!(matches!(d0, AdmitDecision::Hold(HoldReason::WindowFull { .. })));
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
        assert_eq!(d, AdmitDecision::Admit { forced_serial: true });
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
            matches!(d, AdmitDecision::Admit { forced_serial: false }),
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
                c.graceful_holds = 3;
                c.hard_backoffs = 2;
                c.forced_serial_admissions = 1;
                c.admissions = 40;
                c.max_width_reached = 4;
                c.peak_current_bytes = 99;
                c
            }),
        };
        let line = gov.receipt_line();
        assert!(line.contains("graceful_holds=3"));
        assert!(line.contains("hard_backoffs=2"));
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
