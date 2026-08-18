//! In-floor toll receipt: measures reclaimed wall time for prepared-subject reuse.
//! Run: `cargo run -p v1-compiler --bin floor_prepared_toll_receipt`
#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cli_run::{run_floor_prepared_toll_receipt, workspace_root};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir workspace");
    run_floor_prepared_toll_receipt();
    ExitCode::SUCCESS
}
