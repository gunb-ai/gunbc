//! R1C-E — emit-gate check functions shared by the host `#[test]` harness and
//! the `r1c_e_emit_gates` `bin` (the `ExecuteCommand` logical child for the
//! T-Emit `.dag` `TestClaim` wrappers; the `.dag` source is spliced into a
//! `Dag` at integration-test compile time via `env!("CARGO_BIN_EXE_…")` —
//! see the integration-test driver for the on-disk path of the template).
//!
//! Each `check_*` returns `Ok(())` when the gate holds, or `Err(String)` with a
//! human-readable failure detail. The `bin` maps `Ok` → exit 0 / `Err` → exit 1
//! (no stdout/stderr capture by `ExecuteCommand` — exit code is the receipt).
//! `#[test]` callers panic with the detail to preserve the original failure
//! message.
//!
//! **Single source of truth.** The `#[test]` harness and the `bin` both call
//! these functions; do not duplicate the assertion bodies into either caller.
//!
//! **Public surface (R1 close scaffold).** The module is `pub` only so the
//! single bin in this crate can call it (Cargo bins compile against the public
//! lib API). Downstream crates must not depend on `r1c_e_gates::*`; this is a
//! scaffold that dissolves at R1 close together with the wrappers themselves.

use crate::compile_to_dag;
use crate::emit_rust::emit_rust_module;

/// `emit_generic_bounds_survive` (host receipt: `m1_3_emit_rust_test::emit_generic_bounds_survive`,
/// PR #650 post-mortem).
///
/// Pins the **Rust type line** for callable parameters: `impl Fn(...) -> ... + Clone`,
/// not `&impl Fn`. Body avoids higher-order `f(...)` calls — those are a separate
/// emit seam; this receipt only pins the parameter type spelling.
pub fn check_generic_bounds_survive() -> Result<(), String> {
    let src = "fn twice(f: fn(Int) -> Int) -> Int = 0\n";
    let dag = compile_to_dag(src, "r1c_e_generic_bounds.v3")
        .map_err(|e| format!("compile failed: {e:?}"))?;
    let out = emit_rust_module(&dag).map_err(|e| format!("emit failed: {e:?}"))?;

    let sig = "fn twice(p0: impl Fn(i64) -> i64 + Clone) -> i64";
    if !out.contains(sig) {
        return Err(format!(
            "callable param should carry synthesized + Clone (downstream rustc / stage0 contract); got:\n{out}"
        ));
    }
    if out.contains("&impl Fn") {
        return Err(format!(
            "borrowed callable param type must not be spelled as &impl Fn; got:\n{out}"
        ));
    }
    Ok(())
}
