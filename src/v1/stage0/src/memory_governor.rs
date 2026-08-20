//! Host memory budget authority and scheduler-hold observation mirrors.
//!
//! The AIMD admission controller that lived here is deleted — concurrency is now a
//! fixed width derived up front by `derived_realization_schedule` from `std.realize_pack`

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
/// SEED MIRROR of `gunbc.runner_slot_allocation` `gunbc_runner_slot_desired` field `memory_high`
/// — the declared per-slot throttle line. This constant is a mirror, not an independent value:
/// it may only move toward its authority row. Joined by
/// `test.claim.seed_mirror_constant_lens_witness_test`.
pub const DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES: u64 = 13_958_643_712;

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority: `dag/gunbc/runner_slot_allocation.dag`
/// `gunbc_floor_minimum_viable_armed_budget` = `byte_size(12884901888)`; doomed/success witness
/// receipts in the same module):
/// arm-time floor refusal when the governor budget is below the measured minimum viable
/// footprint — crowded uncapped hosts with low MemAvailable would otherwise start a doomed
/// ~30min walk (runs 29834380839, 29845210061).
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
    // Terminal arms. Authority for the source vocabulary and its rendering is
    // `dag/gunbc/host_budget_source.dag` (`HostBudgetSource`); `dag/extdeps/linux/procfs.dag`
    // carries why a Darwin host never reaches the meminfo arm.
    //
    // The meminfo arm used to be unconditional: `(mem_total_bytes(), "/proc/meminfo
    // MemTotal".to_string())`, with the label composed beside a read that returns `None`
    // whenever /proc/meminfo is absent — which on macOS is always. Every local run printed
    // `source=/proc/meminfo MemTotal` on a machine with no /proc: a source attribution for
    // a read that never happened. A reader could not tell "MemTotal said something we
    // distrusted" from "there is no MemTotal here".
    //
    // The repair is not a better label for the absence. Darwin exposes total physical
    // memory through sysctl hw.memsize, so it now gets a REAL read like any other platform
    // (`BudgetSourceDarwinPhysicalMemory`). What remains `None` is a host where every
    // modeled source refused, and that is a refusal rather than a source — see the caller,
    // which must not turn it into a number.
    if let Some(total) = mem_total_bytes() {
        return (Some(total), "/proc/meminfo MemTotal".to_string());
    }
    if let Some(physical) = darwin_physical_memory_bytes() {
        return (Some(physical), "sysctl hw.memsize".to_string());
    }
    (
        None,
        format!(
            "unreadable: no modeled host memory source answered on this platform (target_os={})",
            std::env::consts::OS
        ),
    )
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
/// Authority: `dag/extdeps/darwin/sysctl.dag` (`HwMemsize`), cited to Apple's sysctl.3.
/// This is Darwin's answer to `/proc/meminfo` MemTotal and it is denominated in BYTES,
/// where meminfo's fields are kibibytes — the one detail a shared parser would get wrong
/// by 1024x. Observed live on macOS 15: 17179869184 (exactly 16 GiB).
///
/// Exists because the governor previously had NO source on Darwin and fell back to the
/// most permissive cap it could name. macOS is not a platform without memory facts; its
/// memory facts were never asked for.
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

    #[test]
    fn uncapped_host_budget_caps_mem_available_at_declared_runner_slot_high() {
        let (budget, capped) = uncapped_host_budget_from_mem_available(35_161_423_872);
        assert_eq!(budget, DECLARED_RUNNER_SLOT_MEMORY_HIGH_BYTES);
        assert!(capped);
        let (small, capped_small) = uncapped_host_budget_from_mem_available(8_000_000_000);
        assert_eq!(small, 8_000_000_000);
        assert!(!capped_small);
    }
}
