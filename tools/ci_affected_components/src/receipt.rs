//! Per-run CI **wall-clock timing ledger** (latency monitoring).
//!
//! Axis-A consolidation (operator "go uniform", 2026-06-13): CI runs every job unconditionally
//! (floor jobs cache-gated, lens/corpus/ci `always()`), so the affected-set *selection* machinery
//! was pure shadow — it gated no job (the kill-criterion receipts measured `saved_minutes = 0`,
//! confirmed). That selection half (`affects_*`, `component_partition`, `bootstrap_required`,
//! `affected_set_ci_receipt`) is retired. What remains is the LATENCY half: project the run's
//! observed per-job timestamps into a wall-clock ledger, so the `affected_timings` job can keep
//! emitting `affected-set-ci-receipt-timed` for the CI-latency profilers.
//!
//! `job_windows_to_timings` is the single authority for turning observed timestamps into timings;
//! the shell-free `.dag` transport only relays the raw windows (RR-K §2.4 "projects, does not
//! recompute"). `ci_timings_collector` (Option B host) consumes it and writes the receipt.

use std::collections::BTreeMap;

use serde::Serialize;

/// JSON schema version for the emitted timed artifact. Bumped to 2 when the selection fields
/// (`selected/skipped_components`, `bootstrap_required`, `saved_minutes`, …) were dropped on the
/// Axis-A uniform consolidation; aggregation tooling can branch on shape.
pub const SCHEMA_VERSION: u32 = 2;

/// Per-run CI wall-clock timing ledger (serialized to the `affected-set-ci-receipt-timed` artifact).
///
/// Pure latency monitoring — no affected-set selection, no counterfactual savings (CI runs every
/// job under the uniform transport, so there is nothing to "save" by selection).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimedCiReceipt {
    pub schema_version: u32,
    /// Observed wall-clock per CI job id, in seconds (from GHA job/step timings). Empty when the
    /// timing fetch failed (fail-safe: the receipt keeps its zeros, the run is never failed).
    pub wall_clock_by_job: BTreeMap<String, u64>,
    /// Observed wall-clock span for this run (minutes) = latest completion − earliest start.
    /// `0.0` when no job window parsed.
    pub actual_run_minutes: f64,
}

/// Build the latency ledger receipt from projected timing inputs.
pub fn timed_ci_receipt(
    wall_clock_by_job: BTreeMap<String, u64>,
    actual_run_minutes: f64,
) -> TimedCiReceipt {
    TimedCiReceipt {
        schema_version: SCHEMA_VERSION,
        wall_clock_by_job,
        actual_run_minutes,
    }
}

/// Parse an RFC3339 UTC timestamp into Unix epoch seconds, dependency-free.
///
/// GitHub Actions job timestamps are a fixed shape: `YYYY-MM-DDTHH:MM:SSZ` (UTC, no
/// fractional seconds, always `Z`). Rather than pull in `chrono` for one fixed format,
/// this parses that exact shape via the civil-days algorithm. Returns `None` for any
/// input it does not recognize — queued/not-yet-started jobs emit `null`, so callers
/// fail-safe-skip rather than fabricate a window.
pub fn rfc3339_utc_to_epoch_secs(s: &str) -> Option<i64> {
    // Expect exactly "YYYY-MM-DDTHH:MM:SSZ" (20 chars). Reject anything else.
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let num = |lo: usize, hi: usize| -> Option<i64> { s.get(lo..hi)?.parse::<i64>().ok() };
    let (year, month, day) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hour, min, sec) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || min > 59 || sec > 60 {
        return None;
    }
    // Howard Hinnant's days_from_civil: days since 1970-01-01 (proleptic Gregorian).
    let y = year - i64::from(month <= 2);
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

/// Project raw per-job `(name, started_at, completed_at)` timestamp windows into the
/// receipt's timing inputs: `wall_clock_by_job` (per-job duration in seconds) and
/// `actual_run_minutes` (the run's wall span = latest completion − earliest start, in
/// minutes, rounded to 2dp).
///
/// This is the SINGLE authority for turning observed timestamps into timings: the
/// shell-free `.dag` transport only relays the raw windows (RR-K §2.4 "projects, does
/// not recompute"). Jobs whose timestamps don't parse (still running / `null`) or run
/// backwards are skipped — fail-safe.
pub fn job_windows_to_timings(
    windows: &[(String, String, String)],
) -> (BTreeMap<String, u64>, f64) {
    let mut by_job: BTreeMap<String, u64> = BTreeMap::new();
    let mut min_start: Option<i64> = None;
    let mut max_end: Option<i64> = None;
    for (name, started, completed) in windows {
        let (Some(s), Some(e)) = (
            rfc3339_utc_to_epoch_secs(started),
            rfc3339_utc_to_epoch_secs(completed),
        ) else {
            continue;
        };
        if e < s {
            continue;
        }
        // Last writer wins on a duplicate job name (matrix/rerun) — a wall-clock ledger.
        by_job.insert(name.clone(), (e - s) as u64);
        min_start = Some(min_start.map_or(s, |m| m.min(s)));
        max_end = Some(max_end.map_or(e, |m| m.max(e)));
    }
    let actual_run_minutes = match (min_start, max_end) {
        (Some(s), Some(e)) => (((e - s) as f64 / 60.0) * 100.0).round() / 100.0,
        _ => 0.0,
    };
    (by_job, actual_run_minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_receipt_serializes_latency_keys_only() {
        let mut by_job = BTreeMap::new();
        by_job.insert("v2_lens_ci".to_string(), 777u64);
        let receipt = timed_ci_receipt(by_job, 13.05);
        let json = serde_json::to_value(&receipt).expect("serialize");
        for key in ["schema_version", "wall_clock_by_job", "actual_run_minutes"] {
            assert!(json.get(key).is_some(), "missing key {key}");
        }
        // No selection fields leak into the latency ledger.
        for key in [
            "selected_components",
            "skipped_components",
            "bootstrap_required",
            "saved_minutes",
            "release_distribution_only",
            "changed_paths",
        ] {
            assert!(json.get(key).is_none(), "selection key {key} must be gone");
        }
        assert_eq!(receipt.schema_version, 2);
        assert_eq!(receipt.wall_clock_by_job.get("v2_lens_ci"), Some(&777));
    }

    #[test]
    fn rfc3339_parses_github_job_timestamps_and_rejects_junk() {
        // Unix epoch and a known instant.
        assert_eq!(rfc3339_utc_to_epoch_secs("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            rfc3339_utc_to_epoch_secs("2000-01-01T00:00:00Z"),
            Some(946_684_800)
        );
        // A real GitHub Actions job stamp from run 27454321323.
        assert_eq!(
            rfc3339_utc_to_epoch_secs("2026-06-13T02:48:58Z"),
            Some(1_781_318_938)
        );
        // Difference of two stamps = duration in seconds (ci_floor: 02:48:58 → 02:55:55 = 417s).
        let s = rfc3339_utc_to_epoch_secs("2026-06-13T02:48:58Z").unwrap();
        let e = rfc3339_utc_to_epoch_secs("2026-06-13T02:55:55Z").unwrap();
        assert_eq!(e - s, 417);
        // Fail-safe rejections: null (queued job), wrong length, missing Z, bad field.
        assert_eq!(rfc3339_utc_to_epoch_secs("null"), None);
        assert_eq!(rfc3339_utc_to_epoch_secs(""), None);
        assert_eq!(rfc3339_utc_to_epoch_secs("2026-06-13T02:48:58"), None);
        assert_eq!(rfc3339_utc_to_epoch_secs("2026-13-13T02:48:58Z"), None);
        assert_eq!(rfc3339_utc_to_epoch_secs("2026-06-13 02:48:58Z"), None);
    }

    #[test]
    fn job_windows_project_to_timings_and_skip_unparseable() {
        // Mirrors the real run 27454321323 shape: completed jobs + one still-running (null end).
        let windows = vec![
            (
                "ci_floor".to_string(),
                "2026-06-13T02:48:58Z".to_string(),
                "2026-06-13T02:55:55Z".to_string(),
            ),
            (
                "infra_isolation".to_string(),
                "2026-06-13T02:48:58Z".to_string(),
                "2026-06-13T02:49:05Z".to_string(),
            ),
            // Still running: null completion → skipped, contributes no timing and no span.
            (
                "still_running".to_string(),
                "2026-06-13T02:48:58Z".to_string(),
                "null".to_string(),
            ),
        ];
        let (by_job, actual_run_minutes) = job_windows_to_timings(&windows);
        assert_eq!(by_job.get("ci_floor"), Some(&417));
        assert_eq!(by_job.get("infra_isolation"), Some(&7));
        assert!(!by_job.contains_key("still_running"));
        // Span = earliest start (02:48:58) to latest completion (02:55:55) = 417s = 6.95 min.
        assert_eq!(actual_run_minutes, 6.95);
    }

    #[test]
    fn job_windows_empty_or_all_unparseable_yields_zero() {
        assert_eq!(job_windows_to_timings(&[]), (BTreeMap::new(), 0.0));
        let only_running = vec![("queued".to_string(), "null".to_string(), "null".to_string())];
        assert_eq!(
            job_windows_to_timings(&only_running),
            (BTreeMap::new(), 0.0)
        );
    }
}
