//! Discriminating receipt: corpus-order last-write-wins vs precedence first-write-wins
//! for prepared-floor bare `fn_nodes`. Not CI-enrolled.
//!
//! ```text
//! cargo run -p v1-compiler --release --bin prepared_floor_bare_binding_receipt
//! ```
#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cli_run::run_prepared_floor_bare_binding_receipt;

fn main() -> ExitCode {
    match run_prepared_floor_bare_binding_receipt() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("prepared_floor_bare_binding_receipt: {e}");
            ExitCode::from(1)
        }
    }
}
