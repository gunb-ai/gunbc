//! Continuous `[cost-partition]` receipts for the required-v2-native lane.
//!
//! Host `commit_phase` prints one `[cost-partition]` line per completed named
//! host phase as it finishes, so a cancel still keeps those host phases.
//! Child `[native-lane-phase]` clocks are captured with `Command::output` and
//! reach the stream only after the child exits; a cancel mid-run does not keep
//! `module_bundle` receipts. SCAFFOLD: same dissolution as `cli_run_exclusive_cost_partition_probe`
//! (`PerformanceReceipt` / `CostAccount` Measured). This module consumes
//! `exclusive_cost_partition_from_rows` + `render_exclusive_cost_partition_json`;
//! it does not mint a second JSON schema. The emitted binary prints `[native-lane-phase]`
//! clocks; this host re-renders them as `[cost-partition]` so parent and child share one
//! record shape on one stream.
//!
//! Named host phases (cancel-surviving `commit_phase` lines):
//! `seed_emission`, `cargo_build`, `universe_derivation`, `malformed_control_run`,
//! `receipt_admission`. Those lines carry `labels.head` (`GITHUB_SHA`, else `local`)
//! and `labels.phase` equal to the exclusive row name.
//! Child exclusive rows on `module_bundle` / `binary_parent` (not host phase lines):
//! `source_load`, `test_context`, `module_preparation`, `identity_evaluation`,
//! `receipt_serialization`. The profile number for #10940 is
//! `exclusive.identity_evaluation` on the `binary_parent` `[cost-partition]` line
//! (`basis` `source_root_eval_driver_main_wall`), joined by `labels.head` — not
//! `phase: identity_evaluation` `wall_nanos`.
//! Nested child grain: `module_bundle` (prepare-once-per-module plus the identities inside it).
//! Child exclusive `receipt_serialization` is println/serde of an observation, not host
//! `receipt_admission` (workspace resolve + admission join).

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
pub const PHASE_RECEIPT_SERIALIZATION: &str = "receipt_serialization";
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
    let mut partition = exclusive_cost_partition_from_rows(
        BASIS,
        wall_nanos,
        vec![CostPartitionRow {
            name: clock.phase,
            nanos: wall_nanos,
        }],
        Vec::new(),
    );
    partition.labels = vec![
        ("phase".to_string(), clock.phase.to_string()),
        ("head".to_string(), committed_head()),
    ];
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

fn committed_head() -> String {
    std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string())
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

fn intern_child_exclusive_name(name: &str) -> Result<&'static str, String> {
    match name {
        "seed_emission"
        | "cargo_build"
        | "malformed_control_run"
        | "universe_derivation"
        | "receipt_admission" => Err(format!(
            "host exclusive row {name} is not admissible on the child path"
        )),
        "module_preparation" => Ok(PHASE_MODULE_PREPARATION),
        "identity_evaluation" => Ok(PHASE_IDENTITY_EVALUATION),
        "receipt_serialization" => Ok(PHASE_RECEIPT_SERIALIZATION),
        "module_bundle" => Ok(PHASE_MODULE_BUNDLE),
        "source_load" => Ok(PHASE_SOURCE_LOAD),
        "test_context" => Ok(PHASE_TEST_CONTEXT),
        _ => Err(format!("unknown exclusive row {name}")),
    }
}

fn intern_observation_key(name: &str) -> Option<&'static str> {
    Some(match name {
        "identities" => "identities",
        "modules" => "modules",
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
        let interned = intern_child_exclusive_name(name)?;
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
            | "producer"
            | "ingested_files"
            | "ingested_file_count" => continue,
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
            continue;
        }
        return Err(format!("unhandled value for {k}"));
    }
    partition.labels = labels;
    if let Some(files_v) = obj.get("ingested_files") {
        let arr = files_v
            .as_array()
            .ok_or_else(|| "ingested_files must be an array of paths".to_string())?;
        let mut files: Vec<String> = Vec::new();
        for item in arr {
            let path = item
                .as_str()
                .ok_or_else(|| "ingested_files entries must be strings".to_string())?;
            files.push(path.to_string());
        }
        files.sort();
        files.dedup();
        partition.ingested_file_population = Some(files);
    }
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
        assert!(
            line.contains("\"phase\":\"cargo_build\""),
            "host line must name the phase as a label, not only as the exclusive row: {line}"
        );
        assert!(
            line.contains("\"head\":\"local\"") || line.contains("\"head\":\""),
            "host cancel-surviving lines must join on GITHUB_SHA (local when unset): {line}"
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
        assert!(
            !line.contains("ingested_files"),
            "unmeasured ingest population must be omitted, not an empty or zeroed set: {line}"
        );
    }

    #[test]
    fn child_ingested_files_are_the_identity_set_not_a_count() {
        let json = render_child_phase_as_cost_partition(
            r#"{"phase":"binary_parent","basis":"source_root_eval_driver_main_wall","parent_span_nanos":10,"exclusive":{"source_load":10},"ingested_files":["b.dag","a.dag","a.dag"]}"#,
        )
        .expect("parse");
        assert!(
            json.contains(r#""ingested_files":["a.dag","b.dag"]"#),
            "population is sorted unique paths: {json}"
        );
        assert!(
            json.contains("\"ingested_file_count\":2"),
            "count is derived from the set: {json}"
        );
        assert!(
            !json.contains("\"source_files\""),
            "a lone count is not the population: {json}"
        );
    }

    #[test]
    fn a_source_files_count_observation_is_refused() {
        let err = render_child_phase_as_cost_partition(
            r#"{"phase":"binary_parent","parent_span_nanos":10,"exclusive":{"source_load":10},"source_files":5433}"#,
        )
        .expect_err("count is not a population");
        assert!(err.contains("unknown observation source_files"), "{err}");
    }

    #[test]
    fn child_receipt_admission_row_is_refused_as_a_meaning_fork() {
        let err = render_child_phase_as_cost_partition(
            r#"{"phase":"module_bundle","parent_span_nanos":10,"exclusive":{"receipt_admission":10}}"#,
        )
        .expect_err("child println clock is not host admission");
        assert!(
            err.contains(
                "host exclusive row receipt_admission is not admissible on the child path"
            ),
            "{err}"
        );
    }

    #[test]
    fn child_host_phase_exclusive_rows_are_refused() {
        for name in [
            "seed_emission",
            "cargo_build",
            "malformed_control_run",
            "universe_derivation",
        ] {
            let err = render_child_phase_as_cost_partition(&format!(
                r#"{{"phase":"module_bundle","parent_span_nanos":10,"exclusive":{{"{name}":10}}}}"#
            ))
            .expect_err(name);
            assert!(
                err.contains(&format!(
                    "host exclusive row {name} is not admissible on the child path"
                )),
                "{name}: {err}"
            );
        }
    }

    #[test]
    fn child_receipt_serialization_is_a_distinct_exclusive_row() {
        let json = render_child_phase_as_cost_partition(
            r#"{"phase":"module_bundle","parent_span_nanos":10,"exclusive":{"receipt_serialization":10}}"#,
        )
        .expect("parse");
        assert!(json.contains("\"receipt_serialization\":10"));
        assert!(!json.contains("\"receipt_admission\""));
    }

    #[test]
    fn child_unhandled_field_values_are_refused() {
        for (json, needle) in [
            (
                r#"{"phase":"module_bundle","parent_span_nanos":10,"exclusive":{"source_load":10},"flag":true}"#,
                "unhandled value for flag",
            ),
            (
                r#"{"phase":"module_bundle","parent_span_nanos":10,"exclusive":{"source_load":10},"frac":1.5}"#,
                "unhandled value for frac",
            ),
            (
                r#"{"phase":"module_bundle","parent_span_nanos":10,"exclusive":{"source_load":10},"neg":-1}"#,
                "unhandled value for neg",
            ),
        ] {
            let err = render_child_phase_as_cost_partition(json).expect_err(needle);
            assert!(err.contains(needle), "{needle}: {err}");
        }
    }
}
