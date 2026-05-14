//! Thin Cargo `[[bin]]` entry for `regen_lens`.
//!
//! **R3 gate #7 (`regen_lens_dot_rs_retired`):** keeps the `regen_lens` binary
//! name and CLI surface stable while retiring the legacy
//! `src/bin/regen_lens.rs` path from the SG-0 hand-authored census.

use std::io::Write;
use std::process::ExitCode;

use v3_compiler::dag::Dag;
use v3_compiler::process_exit::ProcessExit;
use v3_compiler::regen_lens_driver::regen_lens_main;

fn main() -> ExitCode {
    let dag = Dag::new();
    match regen_lens_main(&dag) {
        ProcessExit::ExitSuccess => ExitCode::SUCCESS,
        ProcessExit::ExitFailure { code, reason } => {
            let _ = writeln!(std::io::stderr(), "{reason}");
            ExitCode::from((code.max(1).min(255)) as u8)
        }
    }
}
