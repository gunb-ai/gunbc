#![allow(clippy::disallowed_macros)]

// P1 retention-vs-drain cohort receipt probe (dashboard work item
// node://adhoc-9af14fee-a7f). Calls the SAME production entrypoint
// `run_discovery_corpus_with_options` that `claim_executor` uses for the real floor
// path, with `DiscoveryWidthPolicy::Adaptive` (the governor-admitted width the fleet
// actually runs under) over an explicit fixed cohort instead of a full-corpus scan.
// This is not a reimplementation of retention/discovery logic — it is a thinner CLI
// front end onto the identical mechanism claim_executor drives, scoped to the fixed
// 50-entry cohort from `docs/plans/receipts/entry-graph-union-slice2/receipt-post-merge-representative-50.json`
// so mode A / mode B receipts stay directly comparable entry-by-entry.

use std::process::ExitCode;
use std::sync::Arc;

use v1_compiler::cli_run::{
    run_discovery_corpus_with_options, typecheck_compute_count, workspace_root,
    DiscoveryCorpusOptions, DiscoveryWidthPolicy, NodeFrontierSelectionMode,
};
use v1_compiler::memory_governor::MemoryGovernor;
use v1_compiler::v1_interpreter::ExecutionMode;

fn cohort_relative_paths() -> Vec<&'static str> {
    include_str!("p1_cohort_roster.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

fn main() -> ExitCode {
    let ws = workspace_root();
    std::env::set_current_dir(&ws).expect("chdir to workspace root");

    let source_roots = vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ];
    let explicit_entries: Vec<(String, String)> = cohort_relative_paths()
        .into_iter()
        .map(|rel| (ws.join(rel).to_string_lossy().into_owned(), String::new()))
        .collect();

    eprintln!(
        "p1_cohort_probe: {} explicit cohort entr(y/ies), source_roots={:?}",
        explicit_entries.len(),
        source_roots
    );

    let options = DiscoveryCorpusOptions {
        node_frontier_selection: NodeFrontierSelectionMode::Off,
        execution_authority_source_roots: source_roots.clone(),
        explicit_roster_only: true,
        ..Default::default()
    };

    // Arc-1 / P1 cohort receipts require width-1 (Serial inline drain), not adaptive ramp.
    let width_policy = if std::env::var("GUNBC_P1_COHORT_ADAPTIVE")
        .ok()
        .as_deref()
        .map(|v| matches!(v, "1" | "true" | "TRUE"))
        .unwrap_or(false)
    {
        let governor = Arc::new(MemoryGovernor::from_environment(
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
        ));
        DiscoveryWidthPolicy::Adaptive(governor)
    } else {
        DiscoveryWidthPolicy::Serial
    };

    match run_discovery_corpus_with_options(
        &source_roots,
        &[],
        &explicit_entries,
        ExecutionMode::Wet,
        width_policy,
        options,
    ) {
        Ok(summary) => {
            eprintln!(
                "p1_cohort_probe: PASS total={} skipped={} failures={} resolve_ms={:.3} eval_ms={:.3} typecheck_compute_count={}",
                summary.total,
                summary.skipped,
                summary.failures.len(),
                summary.total_resolve_nanos as f64 / 1.0e6,
                summary.total_measured_nanos as f64 / 1.0e6,
                typecheck_compute_count(),
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
