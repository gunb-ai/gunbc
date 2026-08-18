#![allow(clippy::disallowed_macros)]

//! SCAFFOLD (DESIGN §7 seed-retained HAND-RUST / issue 11) — host transport for the
//! expected-red failure identity-grain census (Wave 1 read-only partition).
//!
//! Runs `run_required_floor` once, evaluates only enrolled expected-red witnesses
//! (`GUNBC_REQUIRED_FLOOR_FAILURE_CENSUS_ONLY=1`), and writes a TSV for operator review
//! before qualification vs binding-wall (#8282).
//!
//! NOT floor-enrolled — run standalone for migration planning only (whole-subject prep OOM
//! risk in cargo test). Carrier: `CLI_RUN_REQUIRED_FLOOR_FAILURE_CENSUS_SCAFFOLD_MARKER`
//! in `cli_run.rs`.
//!
//! DISSOLUTION: delete this bin and the marker-gated helpers when Wave 1 name-resolution
//! debt is repaid (qualification or binding-wall lands and `floor_expected_red` shrinks) OR
//! a floor-enrolled census lens subsumes this host transport. Receipt:
//! `rg required_floor_failure_census src/v1/stage0` == 1 until deletion. Authority:
//! `docs/probes/floor_cut_name_resolution_partition.md`.

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
