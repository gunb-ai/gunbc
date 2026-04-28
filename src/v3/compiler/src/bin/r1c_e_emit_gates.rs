// `eprintln!` is on the project's disallowed-macros list (library code must
// use structured error returns). Binary entrypoints may opt in per the lint
// note; this bin is a host-shim whose only behavior is exit-code + stderr
// for human debugging when a gate fails.
#![allow(clippy::disallowed_macros)]

//! R1C-E — `ExecuteCommand` logical child for the T-Emit `.dag` `TestClaim`
//! wrappers. The on-disk `.dag` template (path resolved by the matching
//! integration-test driver) substitutes `env!("CARGO_BIN_EXE_r1c_e_emit_gates")`
//! at test-crate compile time so no checked-in absolute path goes stale.
//!
//! Each subcommand calls into `v3_compiler::r1c_e_gates::check_*`. The function
//! returns `Ok(())` (exit 0) or `Err(detail)` (write detail to stderr, exit 1).
//! `ExecuteCommand` reads only the exit code — stderr is for human debugging
//! when a gate flips red.
//!
//! Subcommands (one per `.dag` `TestClaim` in the R1C-E template set):
//!   - `generic-bounds`  → `check_generic_bounds_survive`
//!   - `rust-fixtures`  → `check_emit_rust_fixtures_rustc_green`
//!   - `omni-demo`  → `check_omni_demo_fixtures_green` (requires `go` + `python3`)
//!
//! Adding a subcommand: extend the `match` below and the `.dag` template in
//! lockstep. **Do not** add stdin/stdout capture or recursive `cargo` here —
//! see PR #792 for the bounded `ExecuteCommand` discipline.

use std::process::ExitCode;

use v3_compiler::r1c_e_gates;

fn usage() -> ! {
    eprintln!(
        "usage: r1c_e_emit_gates <subcommand>\n\
         subcommands: generic-bounds | rust-fixtures | omni-demo"
    );
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let sub = args.next().unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }

    let result = match sub.as_str() {
        "generic-bounds" => r1c_e_gates::check_generic_bounds_survive(),
        "rust-fixtures" => r1c_e_gates::check_emit_rust_fixtures_rustc_green(),
        "omni-demo" => r1c_e_gates::check_omni_demo_fixtures_green(),
        _ => usage(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(detail) => {
            eprintln!("r1c_e_emit_gates {sub}: {detail}");
            ExitCode::FAILURE
        }
    }
}
