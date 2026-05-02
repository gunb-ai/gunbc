//! Host-side mirror of `dsl/std/process.dag` `ProcessExit`.
//!
//! Used by PB-1 emitted bin-shim `main.rs` shells (`emit_rust_bin_shim`) so
//! generated sources can `match` on the same structural shape the `.dag`
//! substrate declares, without inventing a parallel exit carrier.

// Practice 4 (coproduct checkpoint, `docs/modeling-discipline.md` §P1): 🟢 GREEN —
// terminal host mirror of `dsl/std/process.dag` `ProcessExit`; no extra variants
// or semantics beyond that substrate coproduct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessExit {
    ExitSuccess,
    ExitFailure { code: i32, reason: String },
}
