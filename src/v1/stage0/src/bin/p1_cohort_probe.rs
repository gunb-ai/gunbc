#![allow(clippy::disallowed_macros)]

// Width-2 crossover cohort A/B harness (dashboard width-2 lane). Calls the SAME production
// entrypoint `run_discovery_corpus_with_options` that `claim_executor` uses, scoped to the
// fixed 50-entry cohort from `p1_cohort_roster.txt`.
//
// Width selection (one run per invocation — interleave A/B by alternating env):
//   GUNBC_P1_COHORT_WIDTH=1  → DiscoveryWidthPolicy::Serial (width-1 baseline)
//   GUNBC_P1_COHORT_WIDTH=2  → DiscoveryWidthPolicy::ControlledWidthTwo (shared typed store)
// Default: 1 (serial baseline).

use std::process::ExitCode;
use std::time::Instant;

use v1_compiler::cli_run::{
    run_discovery_corpus_with_options, shared_typecheck_store_counters_snapshot, workspace_root,
    DiscoveryCorpusOptions, DiscoveryWidthPolicy, NodeFrontierSelectionMode,
};
use v1_compiler::v1_interpreter::ExecutionMode;

fn cohort_relative_paths() -> Vec<&'static str> {
    include_str!("p1_cohort_roster.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

fn cohort_width_policy() -> Result<DiscoveryWidthPolicy, String> {
    match std::env::var("GUNBC_P1_COHORT_WIDTH")
        .unwrap_or_else(|_| "1".to_string())
        .trim()
    {
        "1" => Ok(DiscoveryWidthPolicy::Serial),
        "2" => Ok(DiscoveryWidthPolicy::ControlledWidthTwo),
        other => Err(format!(
            "GUNBC_P1_COHORT_WIDTH must be 1 (serial) or 2 (controlled-width-two); got {other:?}"
        )),
    }
}

fn main() -> ExitCode {
    let ws = workspace_root();
    std::env::set_current_dir(&ws).expect("chdir to workspace root");

    let width_policy = match cohort_width_policy() {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("p1_cohort_probe: refused: {msg}");
            return ExitCode::from(1);
        }
    };
    let width_label = match &width_policy {
        DiscoveryWidthPolicy::Serial => "serial-width-1",
        DiscoveryWidthPolicy::ControlledWidthTwo => "controlled-width-2",
        DiscoveryWidthPolicy::Adaptive(_) => "adaptive",
    };

    let source_roots = vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ];
    let explicit_entries: Vec<(String, String)> = cohort_relative_paths()
        .into_iter()
        .map(|rel| (ws.join(rel).to_string_lossy().into_owned(), String::new()))
        .collect();

    eprintln!(
        "p1_cohort_probe: {} explicit cohort entr(y/ies), width={}, source_roots={:?}",
        explicit_entries.len(),
        width_label,
        source_roots
    );

    let options = DiscoveryCorpusOptions {
        node_frontier_selection: NodeFrontierSelectionMode::Off,
        execution_authority_source_roots: source_roots.clone(),
        explicit_roster_only: true,
        ..Default::default()
    };

    let wall_start = Instant::now();
    match run_discovery_corpus_with_options(
        &source_roots,
        &[],
        &explicit_entries,
        ExecutionMode::Wet,
        width_policy,
        options,
    ) {
        Ok(summary) => {
            let wall_ms = wall_start.elapsed().as_millis();
            let counters = shared_typecheck_store_counters_snapshot();
            eprintln!(
                "p1_cohort_probe: PASS width={} wall_ms={} total={} skipped={} failures={} resolve_ms={:.3} eval_ms={:.3}",
                width_label,
                wall_ms,
                summary.total,
                summary.skipped,
                summary.failures.len(),
                summary.total_resolve_nanos as f64 / 1.0e6,
                summary.total_measured_nanos as f64 / 1.0e6,
            );
            eprintln!(
                "p1_cohort_probe: shared_store hit={} miss={} encode={} decode={} private_fallback={}",
                counters.shared_store_hit,
                counters.shared_store_miss,
                counters.shared_store_encode,
                counters.shared_store_decode,
                counters.private_store_fallback,
            );
            if summary.failures.is_empty() {
                ExitCode::SUCCESS
            } else {
                for f in &summary.failures {
                    eprintln!("p1_cohort_probe: FAIL {f}");
                }
                ExitCode::from(1)
            }
        }
        Err(msg) => {
            eprintln!("p1_cohort_probe: refused: {msg}");
            ExitCode::from(1)
        }
    }
}
