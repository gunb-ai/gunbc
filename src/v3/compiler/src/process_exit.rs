//! Host-side mirror of `dsl/std/process.dag` `ProcessExit`.
//!
//! **SG-0 / bounded-seed receipt:** hand-authored path is enumerated in
//! `EXPECTED_HAND_AUTHORED_NON_TEST` in `sg0_census_test.rs` (not
//! `GENERATED_FILES` / `build.rs` output).
//!
//! Used by PB-1 emitted bin-shim `main.rs` shells (`emit_rust_bin_shim`) so
//! generated sources can `match` on the same structural shape the `.dag`
//! substrate declares, without inventing a parallel exit carrier.

// Practice 4 (coproduct checkpoint, `docs/modeling-discipline.md` §P1): 🟢 GREEN —
// terminal host mirror of `dsl/std/process.dag` `ProcessExit`; no extra variants
// or semantics beyond that substrate coproduct. `ExitFailure.code` uses `i64`
// — the same host width as `LiteralBits::Int` / substrate `Int` in `dag.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessExit {
    ExitSuccess,
    ExitFailure { code: i64, reason: String },
}
