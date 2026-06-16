//! `ci_timings_collector` — shell-free affected-set CI job-timings collector (Option B host).
//!
//! Replaces `scripts/ci_collect_job_timings.py`. Embeds the v2 interpreter (the
//! `claim_batch` precedent — a Rust host evaluating a `.dag` function), reads
//! `GITHUB_RUN_ID` natively via `std::env::var` (the same call `resolve_auth` uses — NOT
//! shell), binds it as the `run_id` arg of the `.dag` transform `collect_run_job_windows`,
//! and evaluates it. The job-timing FETCH fires through the `.dag` `github.Actions` extdep
//! → `dispatch_rest` → `ureq` — real authenticated HTTP, ZERO shell (gh/curl/subprocess).
//!
//! The transform returns raw per-job timestamp windows; this host projects them into the
//! receipt's timing inputs via the single-authority `job_windows_to_timings`
//! (`ci_affected_components::receipt`) and writes the final `TimedCiReceipt` latency ledger
//! (the `affected-set-ci-receipt-timed` artifact) directly. The Axis-A uniform consolidation
//! retired the separate selection emit bin, so this host is now the sole writer of the timed
//! receipt — a pure wall-clock ledger, no affected-set selection.
//!
//! FAIL-SAFE: any error (missing run id/token, resolve failure, REST/HTTP error, parse
//! failure) warns and writes a receipt with an empty timings map + `0` minutes — the run is
//! never failed, exactly as the Python collector did.

// Binary entrypoint: `eprintln!` is the GitHub Actions `::warning::` annotation channel
// (fail-safe diagnostics), not library logging.
#![allow(clippy::disallowed_macros)]

use std::collections::BTreeMap;
use std::process::ExitCode;

use ci_affected_components::receipt::{job_windows_to_timings, timed_ci_receipt};
use v1_compiler::cli_run::{build_multi_entry_index, make_eval_context, resolve_entry_with_index};
use v1_compiler::v1_interpreter::{run_in_context_with_args, Value};

/// `dsl` is the dependency pool; the driver entry is resolved with its transitive imports.
const SOURCE_ROOT: &str = "dsl";
const DRIVER_ENTRY: &str = "dsl/gunbc/tools/affected_timings.dag";
const DRIVER_FN: &str = "collect_run_job_windows";

fn warn(msg: &str) {
    eprintln!("::warning::ci_timings_collector: {msg}");
}

/// Evaluate the `.dag` transform for `run_id`, returning its TSV windows string.
fn run_driver(run_id: i64) -> Result<String, String> {
    let index = build_multi_entry_index(&[SOURCE_ROOT.to_string()]);
    let (graph, si) = resolve_entry_with_index(&index, DRIVER_ENTRY)?;
    let ctx = make_eval_context(&graph, si);
    let args = [(Some("run_id".to_string()), Value::Int(run_id))];
    match run_in_context_with_args(&ctx, DRIVER_FN, &args, false) {
        Ok(Value::Str(s)) => Ok(s),
        Ok(other) => Err(format!(
            "{DRIVER_FN} returned `{other}`, expected a String of TSV windows"
        )),
        Err(e) => Err(format!("{e}")),
    }
}

/// Parse the driver's `name\tstarted_at\tcompleted_at` lines into windows. Lines that
/// don't have all three fields (or have an empty name) are skipped; receipt.rs separately
/// skips any window whose timestamps don't parse (still-running jobs emit `null`).
fn parse_windows(tsv: &str) -> Vec<(String, String, String)> {
    tsv.lines()
        .filter_map(|line| {
            let mut it = line.split('\t');
            let name = it.next()?;
            let started = it.next()?;
            let completed = it.next()?;
            if name.is_empty() {
                return None;
            }
            Some((name.to_string(), started.to_string(), completed.to_string()))
        })
        .collect()
}

fn collect() -> Result<(BTreeMap<String, u64>, f64), String> {
    let run_id_str =
        std::env::var("GITHUB_RUN_ID").map_err(|_| "GITHUB_RUN_ID not set".to_string())?;
    let run_id: i64 = run_id_str
        .parse()
        .map_err(|_| format!("GITHUB_RUN_ID '{run_id_str}' is not an integer"))?;
    let tsv = run_driver(run_id)?;
    Ok(job_windows_to_timings(&parse_windows(&tsv)))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: collect-affected-set-timings <out_timed_receipt_json>");
        return ExitCode::from(2);
    }
    let out_receipt = &args[1];

    let (job_timings, actual_run_minutes) = match collect() {
        Ok(v) => v,
        Err(e) => {
            warn(&format!(
                "{e}; emitting empty timings (receipt keeps zeros, run not failed)"
            ));
            (BTreeMap::new(), 0.0)
        }
    };

    let job_count = job_timings.len();
    let receipt = timed_ci_receipt(job_timings, actual_run_minutes);
    // Best-effort write; a write failure also fails safe (no downstream consumer hard-depends).
    let receipt_json = serde_json::to_string_pretty(&receipt).unwrap_or_else(|_| "{}".to_string());
    if let Err(e) = std::fs::write(out_receipt, receipt_json) {
        warn(&format!("could not write {out_receipt}: {e}"));
    }
    eprintln!("collected {job_count} job timings; actual_run_minutes={actual_run_minutes}");
    ExitCode::SUCCESS
}
