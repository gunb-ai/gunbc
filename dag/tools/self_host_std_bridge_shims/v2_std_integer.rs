// Shared std-bridge shim — curated minimal v2.std.integer surface.
//
// AUTHORITY: src/v2/std/integer.dag (Int = GroupCompletion<Nat>, per dag/std/algebra.dag).
//
// WHY A SHIM AND NOT THE EMITTED MODULE: the emitted v2_std_integer.rs is a closure-projection
// stub that reads `use crate::std_nat::Nat;`, but the emitted lib.rs never declares
// `pub mod std_nat` — the closure manifest omits std_nat while the stub that needs it is
// emitted anyway. That is an emitter closure-projection defect, not shim drift, and it is NOT
// fixed here. Receipt (2026-07-29): building any of these transports with a derived whole-
// closure lib.rs instead of its narrow hand manifest reds on exactly
// `error[E0432]: unresolved import crate::std_nat --> src/v2_std_integer.rs:6:12`. That is
// also why the narrow hand lib.rs files are load-bearing and stay: they declare WHICH emitted
// modules participate, so an unshimmed broken stub is never compiled.
//
// REPRESENTATION DIVERGENCE (deliberate, and narrow): its one consumer today is the
// 03_normalize row, whose narrow lib re-exports `Int` without using it in a typed position, so
// the checkpoint
// scalar rendering (rust_scalar_checkpoint_reference_base, Int -> i64) is sufficient here and
// keeps the bridge free of the Nat coproduct. If a consumer ever uses Int in a typed position
// this alias is wrong and must be grounded on the real GroupCompletion pair construction.
//
// dissolve-on: emitted closure declares std_nat alongside the stub that imports it; then this
// file is deleted and the emitted module is used directly.
pub type Int = i64;
