#![allow(clippy::disallowed_macros)]

//! [LOCAL MEASUREMENT PROBE — investigation clever-seal-476, do not merge]
//! Runs the exact floor compile-clean receipt path (`compile_sources` with
//! `RenderTarget::Dag` over the WholeTree plan) so the emit leg's wall is
//! attributable next to the resolve leg measured by
//! `compile_clean_diagnostic_histogram`. Delete with the probe revert.

use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let started = Instant::now();
    eprintln!("emit_leg_probe: starting whole-tree --target dag compile (receipt path)…");
    let ok = v1_compiler::cli_run::witness_layer_roots_compile_clean_emit_check();
    eprintln!(
        "emit_leg_probe: ok={ok} total_wall_s={:.1}",
        started.elapsed().as_secs_f64()
    );
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
