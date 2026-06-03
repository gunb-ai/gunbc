//! Wave-1 §11.7 kill-criterion instrumentation bin — emits the per-PR affected-set CI receipt
//! (JSON) for upload as a workflow artifact. See `ci_affected_components::receipt` for the schema
//! and the operator question this answers.
//!
//! This bin only *measures*; it does not gate any job. `ci_floor` still runs unconditionally today,
//! so the receipt is the prediction the operator aggregates before deciding to affected-set-gate it.
#![allow(clippy::disallowed_macros)]

use std::collections::BTreeMap;
use std::process::ExitCode;
use std::{env, fs};

use ci_affected_components::git_diff_transport::{
    git_read_changed_paths_for_event, GitChangedPathsRead,
};
use ci_affected_components::receipt::{affected_set_ci_receipt, BASELINE_FULL_RUN_MINUTES_UNSET};
use ci_affected_components::{
    ci_component_affected_fail_closed, ci_component_affected_from_changed_paths,
};

fn usage() -> ! {
    eprintln!(
        "usage: emit-affected-set-ci-receipt <event_name> <output_json> [options]\n\
         event_name: pull_request | push\n\
         output_json: path to write the receipt JSON artifact\n\
         options:\n\
         \x20 --estimated-full-run-minutes <f64>  operator ci_floor p50 baseline (default 0 = unset)\n\
         \x20 --actual-run-minutes <f64>          observed wall-clock for this run (default 0)\n\
         \x20 --selected-claim-count <u32>        node-frontier claim count (default 0; shadow in v1)\n\
         \x20 --job-timings <path>                JSON map {{\"job_id\": seconds}} of per-job wall-clock"
    );
    std::process::exit(2);
}

struct Options {
    event_name: String,
    output_json: String,
    estimated_full_run_minutes: f64,
    actual_run_minutes: f64,
    selected_claim_count: u32,
    job_timings_path: Option<String>,
}

fn parse_f64(flag: &str, value: Option<String>) -> f64 {
    value
        .unwrap_or_else(|| usage())
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("error: {flag} expects a number");
            usage()
        })
}

fn parse_u32(flag: &str, value: Option<String>) -> u32 {
    value
        .unwrap_or_else(|| usage())
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("error: {flag} expects an integer");
            usage()
        })
}

fn parse_args() -> Options {
    let mut args = env::args().skip(1);
    let event_name = args.next().unwrap_or_else(|| usage());
    let output_json = args.next().unwrap_or_else(|| usage());
    let mut opts = Options {
        event_name,
        output_json,
        estimated_full_run_minutes: BASELINE_FULL_RUN_MINUTES_UNSET,
        actual_run_minutes: 0.0,
        selected_claim_count: 0,
        job_timings_path: None,
    };
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--estimated-full-run-minutes" => {
                opts.estimated_full_run_minutes = parse_f64(&flag, args.next());
            }
            "--actual-run-minutes" => {
                opts.actual_run_minutes = parse_f64(&flag, args.next());
            }
            "--selected-claim-count" => {
                opts.selected_claim_count = parse_u32(&flag, args.next());
            }
            "--job-timings" => {
                opts.job_timings_path = Some(args.next().unwrap_or_else(|| usage()));
            }
            _ => {
                eprintln!("error: unknown argument {flag}");
                usage();
            }
        }
    }
    opts
}

/// Read an optional `{"job_id": seconds}` JSON map. A missing path yields an empty map; a malformed
/// file warns and yields an empty map (timing enrichment is best-effort and must never fail the run).
fn read_job_timings(path: Option<&str>) -> BTreeMap<String, u64> {
    let Some(path) = path else {
        return BTreeMap::new();
    };
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("::warning::--job-timings {path} is not a JSON map of job->seconds ({e}); emitting empty wall_clock_by_job");
            BTreeMap::new()
        }),
        Err(e) => {
            eprintln!("::warning::--job-timings {path} unreadable ({e}); emitting empty wall_clock_by_job");
            BTreeMap::new()
        }
    }
}

fn main() -> ExitCode {
    let opts = parse_args();

    let (changed_paths, flags, fail_closed) =
        match git_read_changed_paths_for_event(opts.event_name.as_str()) {
            GitChangedPathsRead::Ok { paths, .. } => {
                let flags =
                    ci_component_affected_from_changed_paths(paths.iter().map(String::as_str));
                (paths, flags, false)
            }
            GitChangedPathsRead::FailClosed { detail, .. } => {
                eprintln!("error: {detail}");
                (Vec::new(), ci_component_affected_fail_closed(), true)
            }
        };

    let receipt = affected_set_ci_receipt(
        changed_paths,
        flags,
        fail_closed,
        opts.selected_claim_count,
        read_job_timings(opts.job_timings_path.as_deref()),
        opts.estimated_full_run_minutes,
        opts.actual_run_minutes,
    );

    let json = match serde_json::to_string_pretty(&receipt) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("error: serialize receipt: {e}");
            return ExitCode::from(1);
        }
    };

    // Echo to the log for at-a-glance reading, then persist the artifact.
    eprintln!("{json}");
    if let Err(e) = fs::write(&opts.output_json, format!("{json}\n")) {
        eprintln!("error: write {}: {e}", opts.output_json);
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
