//! Continuous `[cost-partition]` receipts for the required-v2-native lane.
//!
//! One line per completed named phase so a cancel still keeps the phases already
//! committed. SCAFFOLD: same dissolution as `cli_run_exclusive_cost_partition_probe`
//! (`PerformanceReceipt` / `CostAccount` Measured). This module consumes
//! `exclusive_cost_partition_from_rows` + `render_exclusive_cost_partition_json`;
//! it does not mint a second JSON schema.
//!
//! Named phases (operator roster for #10940's next run):
//! `seed_emission`, `cargo_build`, `malformed_control_run`, `universe_derivation`,
//! `module_preparation`, `identity_evaluation`, `receipt_admission`.
//! Nested grain: `module_bundle` (prepare-once-per-module plus the identities inside it).

use std::time::Instant;

use crate::v1_interpreter::thread_cpu_nanos;

use super::{
    exclusive_cost_partition_from_rows, render_exclusive_cost_partition_json, CostPartitionRow,
};

pub const COST_PARTITION_TAG: &str = "[cost-partition]";

pub const PHASE_SEED_EMISSION: &str = "seed_emission";
pub const PHASE_CARGO_BUILD: &str = "cargo_build";
pub const PHASE_MALFORMED_CONTROL_RUN: &str = "malformed_control_run";
pub const PHASE_UNIVERSE_DERIVATION: &str = "universe_derivation";
pub const PHASE_MODULE_PREPARATION: &str = "module_preparation";
pub const PHASE_IDENTITY_EVALUATION: &str = "identity_evaluation";
pub const PHASE_RECEIPT_ADMISSION: &str = "receipt_admission";
pub const PHASE_MODULE_BUNDLE: &str = "module_bundle";

const BASIS: &str = "required_v2_native_phase_wall";

pub struct PhaseClock {
    pub phase: &'static str,
    wall: Instant,
    cpu: u128,
    children_cpu: u128,
}

pub fn begin(phase: &'static str) -> PhaseClock {
    PhaseClock {
        phase,
        wall: Instant::now(),
        cpu: thread_cpu_nanos(),
        children_cpu: children_cpu_nanos().unwrap_or(0),
    }
}

pub fn render_committed_phase(clock: &PhaseClock, observations: &[(&str, u128)]) -> String {
    let wall_nanos = clock.wall.elapsed().as_nanos();
    let cpu_nanos = thread_cpu_nanos().saturating_sub(clock.cpu);
    let children_cpu_nanos = children_cpu_nanos()
        .unwrap_or(0)
        .saturating_sub(clock.children_cpu);
    let partition = exclusive_cost_partition_from_rows(
        BASIS,
        wall_nanos,
        vec![CostPartitionRow {
            name: clock.phase,
            nanos: wall_nanos,
        }],
        Vec::new(),
    );
    let mut obs: Vec<(&str, u128)> = vec![
        ("cpu_nanos", cpu_nanos),
        ("children_cpu_nanos", children_cpu_nanos),
        (
            "peak_rss_bytes",
            super::peak_rss_vhwm_bytes().unwrap_or(0) as u128,
        ),
        ("rss_bytes", super::current_rss_bytes().unwrap_or(0) as u128),
    ];
    obs.extend_from_slice(observations);
    render_exclusive_cost_partition_json(&partition, &obs)
}

/// Print one committed phase line immediately (stderr).
pub fn commit_phase(clock: PhaseClock, observations: &[(&str, u128)]) {
    eprintln!(
        "{COST_PARTITION_TAG} {}",
        render_committed_phase(&clock, observations)
    );
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

#[cfg(test)]
mod tests {
    use super::super::{
        exclusive_cost_partition_from_rows, CostAccountingRefusal, CostAccountingVerdict,
        CostPartitionRow,
    };
    use super::*;

    #[test]
    fn overattributed_rows_refuse_rather_than_clamping_remainder() {
        let p = exclusive_cost_partition_from_rows(
            BASIS,
            10,
            vec![
                CostPartitionRow {
                    name: PHASE_IDENTITY_EVALUATION,
                    nanos: 8,
                },
                CostPartitionRow {
                    name: PHASE_MODULE_PREPARATION,
                    nanos: 8,
                },
            ],
            Vec::new(),
        );
        assert!(matches!(
            p.verdict,
            CostAccountingVerdict::Refused {
                cause: CostAccountingRefusal::OverAttributed { .. }
            }
        ));
        assert_eq!(p.remainder_nanos, 0);
        assert!(p.share_of_parent(PHASE_IDENTITY_EVALUATION).is_none());
    }

    #[test]
    fn a_committed_phase_line_is_the_existing_renderer_not_a_private_json_object() {
        let clock = begin(PHASE_CARGO_BUILD);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let line = render_committed_phase(&clock, &[]);
        assert!(
            line.contains("\"state\":\"Reconciled\""),
            "a private serde_json fork would omit the renderer's verdict: {line}"
        );
        assert!(line.contains("\"cargo_build\":"));
        assert!(line.contains("accounting_law"));
        assert!(
            !line.contains("\"status\":\"committed\""),
            "status was the forked schema; the renderer names the phase as the exclusive row"
        );
    }
}
