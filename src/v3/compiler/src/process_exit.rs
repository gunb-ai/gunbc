//! Host-side mirror of `dsl/std/process.dag` `ProcessExit`.
//!
//! Used by PB-1 emitted bin-shim `main.rs` shells (`emit_rust_bin_shim`) so
//! generated sources can `match` on the same structural shape the `.dag`
//! substrate declares, without inventing a parallel exit carrier.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessExit {
    ExitSuccess,
    ExitFailure { code: i32, reason: String },
}
