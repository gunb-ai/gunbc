// AUTO-GENERATED from dsl/std/runtime/bin_shims/regen_lens.dag - DO NOT EDIT.
//
// Unified lens-regen driver. Reads LensRegistryEntry records and regenerates
// each declared lens module.

use std::io::Write;
use std::process::ExitCode;

use v3_compiler::dag::Dag;
use v3_compiler::process_exit::ProcessExit;

fn main() -> ExitCode {
    match v3_compiler::regen_lens::regen_lens_main(&Dag::new()) {
        ProcessExit::ExitSuccess => ExitCode::SUCCESS,
        ProcessExit::ExitFailure { code, reason } => {
            let _ = writeln!(std::io::stderr(), "{reason}");
            ExitCode::from((code.max(1).min(255)) as u8)
        }
    }
}
