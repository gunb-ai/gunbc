#![allow(clippy::disallowed_macros)]

//! Host transport for the expected-red roster identity join (Wave 1).
//!
//! Runs `run_required_floor` in join-only mode: evaluates enrolled expected-red witnesses
//! that appear in the manifest, classifies every roster identity into
//! `still_red | now_passes | not_evaluated`, and writes a TSV receipt.
//!
//! **Do not prune `floor_expected_red` from this output until after the rebase wave** when
//! host-tool verdicts are trustworthy again (#8420). The join is built now; pruning waits
//! for a run where `not_evaluated` is not dominated by infra distortion.

use std::process::ExitCode;

use v1_compiler::cli_run::{run_required_floor, workspace_root, ShardStyle};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    std::env::remove_var("GITHUB_ACTIONS");
    std::env::remove_var("GUNBC_CI_DIFF_BASE");

    let join_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/probes/expected_red_roster_join.tsv".to_string());
    std::env::set_var("GUNBC_EXPECTED_RED_ROSTER_JOIN", &join_path);
    std::env::set_var("GUNBC_EXPECTED_RED_ROSTER_JOIN_ONLY", "1");
    std::env::set_var(
        "GUNBC_EXPECTED_RED_ROSTER_JOIN_ONLY_CALLER",
        "expected_red_roster_join_bin",
    );

    let source_roots = vec!["dag".to_string(), "src/v2".to_string()];
    eprintln!("expected_red_roster_join: writing to {join_path}");

    match run_required_floor(&source_roots, "join", ShardStyle::single_shard()) {
        Ok(outcome) => {
            eprintln!(
                "JOIN_STATUS ok executed={} held={} failure_lines={}",
                outcome.claims_executed,
                outcome.known_red_held,
                outcome.failures.len()
            );
            if let Ok(contents) = std::fs::read_to_string(&join_path) {
                println!("--- JOIN_TSV ---");
                print!("{contents}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("JOIN_STATUS error");
            eprintln!("JOIN_ERROR {e}");
            ExitCode::from(2)
        }
    }
}
