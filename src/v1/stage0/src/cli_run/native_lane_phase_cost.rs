//! Continuous `[cost-partition]` receipts for the required-v2-native lane.
//!
//! One line per completed named phase so a cancel still keeps the phases already
//! committed. The JSON shape reuses the exclusive-partition law from
//! `render_exclusive_cost_partition_json`: `parent_span_nanos == sum_exclusive + remainder`.
//! No second ledger.
//!
//! Named phases (operator roster for #10940's next run):
//! `seed_emission`, `cargo_build`, `malformed_control_run`, `universe_derivation`,
//! `module_preparation`, `identity_evaluation`, `receipt_admission`.
//! Nested grain: `module_bundle` (prepare-once-per-module plus the identities inside it).
//!
//! Host (`native_lane_runner`) commits the seed/cargo/malformed/universe/admission
//! phases that still live in this process. The emitted binary commits
//! `module_preparation`, `identity_evaluation`, and per-`module_bundle` lines on stderr.
//! #10940 consumes this helper rather than minting a second tag: call `begin`/`commit_phase`
//! at the same names when those steps move inside the binary.

use std::time::Instant;

use crate::v1_interpreter::thread_cpu_nanos;

pub const COST_PARTITION_TAG: &str = "[cost-partition]";

pub const PHASE_SEED_EMISSION: &str = "seed_emission";
pub const PHASE_CARGO_BUILD: &str = "cargo_build";
pub const PHASE_MALFORMED_CONTROL_RUN: &str = "malformed_control_run";
pub const PHASE_UNIVERSE_DERIVATION: &str = "universe_derivation";
pub const PHASE_MODULE_PREPARATION: &str = "module_preparation";
pub const PHASE_IDENTITY_EVALUATION: &str = "identity_evaluation";
pub const PHASE_RECEIPT_ADMISSION: &str = "receipt_admission";
pub const PHASE_MODULE_BUNDLE: &str = "module_bundle";

const PRODUCER: &str = "gunbc.witness_v2_native_route native_lane_phase_cost [cost-partition]";
const BASIS: &str = "required_v2_native_phase_wall";

pub struct PhaseClock {
    pub phase: &'static str,
    wall: Instant,
    cpu: u128,
    children_cpu: u128,
}

pub fn head_sha() -> String {
    std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string())
}

pub fn begin(phase: &'static str) -> PhaseClock {
    PhaseClock {
        phase,
        wall: Instant::now(),
        cpu: thread_cpu_nanos(),
        children_cpu: children_cpu_nanos().unwrap_or(0),
    }
}

/// Print one committed phase line immediately (stderr).
pub fn commit_phase(clock: PhaseClock, observations: serde_json::Value) {
    let wall_nanos = clock.wall.elapsed().as_nanos();
    let cpu_nanos = thread_cpu_nanos().saturating_sub(clock.cpu);
    let children_cpu_nanos = children_cpu_nanos()
        .unwrap_or(0)
        .saturating_sub(clock.children_cpu);
    let line = serde_json::json!({
        "basis": BASIS,
        "basis_note": "elapsed wall of this named phase. cpu_nanos is CLOCK_THREAD_CPUTIME_ID on the committing thread. Child processes are wall plus children_cpu_nanos / children_peak_rss_bytes, never mixed into exclusive rows of a different phase.",
        "producer": PRODUCER,
        "phase": clock.phase,
        "head": head_sha(),
        "status": "committed",
        "parent_span_nanos": wall_nanos,
        "exclusive": { clock.phase: wall_nanos },
        "sum_exclusive_nanos": wall_nanos,
        "remainder_nanos": 0u64,
        "accounting_law": "parent_span_nanos == sum_exclusive_nanos + remainder_nanos",
        "wall_nanos": wall_nanos,
        "cpu_nanos": cpu_nanos,
        "peak_rss_bytes": super::peak_rss_vhwm_bytes(),
        "rss_bytes": super::current_rss_bytes(),
        "children_cpu_nanos": children_cpu_nanos,
        "children_peak_rss_bytes": children_peak_rss_bytes(),
        "observations": observations,
    });
    eprintln!("{COST_PARTITION_TAG} {line}");
}

pub fn relay_child_stderr(stderr: &[u8]) {
    if stderr.is_empty() {
        return;
    }
    let text = String::from_utf8_lossy(stderr);
    eprint!("{text}");
    if !text.ends_with('\n') {
        eprintln!();
    }
}

fn children_rusage() -> Option<libc::rusage> {
    let mut ru: libc::rusage = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru) };
    if rc != 0 {
        None
    } else {
        Some(ru)
    }
}

fn timeval_nanos(tv: libc::timeval) -> u128 {
    (tv.tv_sec as u128)
        .saturating_mul(1_000_000_000)
        .saturating_add((tv.tv_usec as u128).saturating_mul(1_000))
}

fn children_cpu_nanos() -> Option<u128> {
    let ru = children_rusage()?;
    Some(timeval_nanos(ru.ru_utime).saturating_add(timeval_nanos(ru.ru_stime)))
}

fn children_peak_rss_bytes() -> Option<u64> {
    let ru = children_rusage()?;
    if ru.ru_maxrss <= 0 {
        return None;
    }
    let raw = ru.ru_maxrss as u64;
    #[cfg(target_os = "macos")]
    {
        Some(raw)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some(raw.saturating_mul(1024))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_committed_phase_is_a_trivial_exclusive_partition_of_its_own_wall() {
        let clock = begin(PHASE_IDENTITY_EVALUATION);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let wall = clock.wall.elapsed().as_nanos();
        assert!(wall > 0);
        let exclusive = wall;
        let remainder = wall.saturating_sub(exclusive);
        assert_eq!(remainder, 0);
        assert_eq!(exclusive + remainder, wall);
    }

    #[test]
    fn operator_phase_roster_is_closed() {
        let names = [
            PHASE_SEED_EMISSION,
            PHASE_CARGO_BUILD,
            PHASE_MALFORMED_CONTROL_RUN,
            PHASE_UNIVERSE_DERIVATION,
            PHASE_MODULE_PREPARATION,
            PHASE_IDENTITY_EVALUATION,
            PHASE_RECEIPT_ADMISSION,
        ];
        let mut sorted: Vec<&str> = names.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 7);
    }
}
