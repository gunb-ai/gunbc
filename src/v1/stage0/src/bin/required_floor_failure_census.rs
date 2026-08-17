#![allow(clippy::disallowed_macros)]

//! Expected-red failure census for the floor-cut name-resolution partition.
//!
//! Runs the required-floor preparation once, then evaluates only the enrolled
//! expected-red witnesses (not the full ~9,400-claim fold) and writes an
//! identity-grain TSV for operator review.
//!
//! NOT floor-enrolled — run standalone for migration planning only.

use std::process::ExitCode;

use v1_compiler::cli_run::{run_required_floor, workspace_root, ShardStyle};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    std::env::remove_var("GITHUB_ACTIONS");
    std::env::remove_var("GUNBC_CI_DIFF_BASE");

    let census_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/probes/floor_cut_name_resolution_census.tsv".to_string());
    std::env::set_var("GUNBC_REQUIRED_FLOOR_FAILURE_CENSUS", &census_path);
    std::env::set_var("GUNBC_REQUIRED_FLOOR_FAILURE_CENSUS_ONLY", "1");

    let source_roots = vec!["dag".to_string(), "src/v2".to_string()];
    eprintln!("required_floor_failure_census: writing to {census_path}");

    match run_required_floor(&source_roots, "census", ShardStyle::single_shard()) {
        Ok(outcome) => {
            eprintln!(
                "CENSUS_STATUS ok held={} passing_now={} unexpected_failures={}",
                outcome.known_red_held,
                outcome
                    .failures
                    .iter()
                    .filter(|f| f.contains("expected-red and PASSED"))
                    .count(),
                outcome.failures.len()
            );
            if let Ok(contents) = std::fs::read_to_string(&census_path) {
                println!("--- CENSUS_TSV ---");
                print!("{contents}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("CENSUS_STATUS error");
            eprintln!("CENSUS_ERROR {e}");
            ExitCode::from(2)
        }
    }
}
