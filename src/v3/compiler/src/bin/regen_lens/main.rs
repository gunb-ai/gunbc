// Thin entrypoint for the `regen_lens` Cargo bin (R3 gate #7).
// Implementation: `v3_compiler::regen_lens`.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match v3_compiler::regen_lens::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(io::stderr(), "{message}");
            ExitCode::FAILURE
        }
    }
}
