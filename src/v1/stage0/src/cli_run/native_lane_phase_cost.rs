//! Continuous `[cost-partition]` receipts for the required-v2-native lane.
//!
//! One line per completed named phase so a cancel still keeps the phases already
//! committed. SCAFFOLD: same dissolution as `cli_run_exclusive_cost_partition_probe`
//! (`PerformanceReceipt` / `CostAccount` Measured). This module consumes
//! `exclusive_cost_partition_from_rows` + `render_exclusive_cost_partition_json`;
//! it does not mint a second JSON schema. The emitted binary prints `[native-lane-phase]`
//! clocks; this host re-renders them as `[cost-partition]` so parent and child share one
//! record shape on one stream.
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
pub const CHILD_PHASE_TAG: &str = "[native-lane-phase]";

pub const PHASE_SEED_EMISSION: &str = "seed_emission";
pub const PHASE_CARGO_BUILD: &str = "cargo_build";
pub const PHASE_MALFORMED_CONTROL_RUN: &str = "malformed_control_run";
pub const PHASE_UNIVERSE_DERIVATION: &str = "universe_derivation";
pub const PHASE_MODULE_PREPARATION: &str = "module_preparation";
pub const PHASE_IDENTITY_EVALUATION: &str = "identity_evaluation";
pub const PHASE_RECEIPT_ADMISSION: &str = "receipt_admission";
pub const PHASE_MODULE_BUNDLE: &str = "module_bundle";
pub const PHASE_SOURCE_LOAD: &str = "source_load";
pub const PHASE_TEST_CONTEXT: &str = "test_context";

const BASIS: &str = "required_v2_native_phase_wall";
const BINARY_PARENT_BASIS: &str = "source_root_eval_driver_main_wall";

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
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(CHILD_PHASE_TAG) {
            match render_child_phase_as_cost_partition(rest.trim()) {
                Ok(json) => eprintln!("{COST_PARTITION_TAG} {json}"),
                Err(cause) => {
                    eprintln!("{CHILD_PHASE_TAG} refused: {cause}: {line}")
                }
            }
        } else if line.starts_with(COST_PARTITION_TAG) {
            eprintln!(
                "{CHILD_PHASE_TAG} refused: child must not mint {COST_PARTITION_TAG}; use {CHILD_PHASE_TAG}: {line}"
            );
        } else {
            eprintln!("{line}");
        }
    }
    if !text.ends_with('\n') {
        eprintln!();
    }
}

fn intern_exclusive_name(name: &str) -> Option<&'static str> {
    Some(match name {
        "seed_emission" => PHASE_SEED_EMISSION,
        "cargo_build" => PHASE_CARGO_BUILD,
        "malformed_control_run" => PHASE_MALFORMED_CONTROL_RUN,
        "universe_derivation" => PHASE_UNIVERSE_DERIVATION,
        "module_preparation" => PHASE_MODULE_PREPARATION,
        "identity_evaluation" => PHASE_IDENTITY_EVALUATION,
        "receipt_admission" => PHASE_RECEIPT_ADMISSION,
        "module_bundle" => PHASE_MODULE_BUNDLE,
        "source_load" => PHASE_SOURCE_LOAD,
        "test_context" => PHASE_TEST_CONTEXT,
        _ => return None,
    })
}

fn intern_observation_key(name: &str) -> Option<&'static str> {
    Some(match name {
        "identities" => "identities",
        "modules" => "modules",
        "source_files" => "source_files",
        "eval_mean_nanos" => "eval_mean_nanos",
        "eval_p50_nanos" => "eval_p50_nanos",
        "eval_p95_nanos" => "eval_p95_nanos",
        "prepare_mean_nanos" => "prepare_mean_nanos",
        "prepare_p50_nanos" => "prepare_p50_nanos",
        "prepare_p95_nanos" => "prepare_p95_nanos",
        "cpu_nanos" => "cpu_nanos",
        "children_cpu_nanos" => "children_cpu_nanos",
        "peak_rss_bytes" => "peak_rss_bytes",
        "rss_bytes" => "rss_bytes",
        _ => return None,
    })
}

fn json_u128(v: &serde_json::Value) -> Option<u128> {
    v.as_u64()
        .map(|n| n as u128)
        .or_else(|| v.as_i64().and_then(|n| (n >= 0).then_some(n as u128)))
}

fn render_child_phase_as_cost_partition(json_text: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(json_text).map_err(|e| format!("json: {e}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "native-lane-phase body must be an object".to_string())?;
    let parent_span_nanos = obj
        .get("parent_span_nanos")
        .and_then(json_u128)
        .ok_or_else(|| "parent_span_nanos required".to_string())?;
    let exclusive_obj = obj
        .get("exclusive")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "exclusive object required".to_string())?;
    let mut exclusive = Vec::new();
    for (name, nanos_v) in exclusive_obj {
        let interned =
            intern_exclusive_name(name).ok_or_else(|| format!("unknown exclusive row {name}"))?;
        let nanos = json_u128(nanos_v).ok_or_else(|| format!("nanos for {name}"))?;
        exclusive.push(CostPartitionRow {
            name: interned,
            nanos,
        });
    }
    let basis = match obj.get("basis").and_then(|v| v.as_str()) {
        Some(BINARY_PARENT_BASIS) => BINARY_PARENT_BASIS,
        Some(BASIS) | None => BASIS,
        Some(other) => {
            return Err(format!("unknown basis {other}"));
        }
    };
    let mut partition =
        exclusive_cost_partition_from_rows(basis, parent_span_nanos, exclusive, Vec::new());
    let mut labels = Vec::new();
    let mut observations: Vec<(&str, u128)> = Vec::new();
    for (k, v) in obj {
        match k.as_str() {
            "basis"
            | "exclusive"
            | "inclusive"
            | "parent_span_nanos"
            | "sum_exclusive_nanos"
            | "remainder_nanos"
            | "verdict"
            | "basis_note"
            | "accounting_law"
            | "spans"
            | "nested_spans"
            | "producer" => continue,
            _ => {}
        }
        if let Some(s) = v.as_str() {
            labels.push((k.clone(), s.to_string()));
            continue;
        }
        if let Some(n) = json_u128(v) {
            let key =
                intern_observation_key(k).ok_or_else(|| format!("unknown observation {k}"))?;
            observations.push((key, n));
        }
    }
    partition.labels = labels;
    Ok(render_exclusive_cost_partition_json(
        &partition,
        &observations,
    ))
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
        exclusive_cost_partition_from_rows, render_exclusive_cost_partition_json,
        CostAccountingRefusal, CostAccountingVerdict, CostPartitionRow,
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
        assert!(
            line.contains("\"resolve_volume\":\"unmeasured\""),
            "unobserved resolve volume must not print as measured zeros: {line}"
        );
        assert!(
            !line.contains("edge_index_construction"),
            "fabricated zeros would look measured: {line}"
        );
    }

    #[test]
    fn child_overattribution_is_rerendered_as_overattributed() {
        let json = render_child_phase_as_cost_partition(
            r#"{"phase":"module_bundle","parent_span_nanos":10,"exclusive":{"module_preparation":8,"identity_evaluation":8},"module":"std.x"}"#,
        )
        .expect("parse");
        assert!(
            json.contains("\"kind\":\"OverAttributed\""),
            "host must refuse, not clamp: {json}"
        );
        assert!(!json.contains("\"remainder_nanos\":0") || json.contains("OverAttributed"));
    }

    #[test]
    fn from_rows_omits_unmeasured_resolve_volume() {
        let p = exclusive_cost_partition_from_rows(BASIS, 5, Vec::new(), Vec::new());
        let line = render_exclusive_cost_partition_json(&p, &[]);
        assert!(line.contains("\"resolve_volume\":\"unmeasured\""));
        assert!(!line.contains("edge_index_construction"));
        assert!(!line.contains("pool_parse"));
    }
}
