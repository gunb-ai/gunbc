// Shared std-bridge shim — curated minimal v2.std.integer surface.
//
// AUTHORITY: src/v2/std/integer.dag (Int = GroupCompletion<Nat>, per dag/std/algebra.dag).
//
// WHY A SHIM AND NOT THE EMITTED MODULE: the emitted v2_std_integer.rs is a closure-projection
// stub that reads `use crate::std_nat::Nat;`, but the emitted lib.rs never declares
// `pub mod std_nat` — the closure manifest omits std_nat while the stub that needs it is
// emitted anyway. That is an emitter closure-projection defect, not shim drift; it is reported
// separately and is NOT fixed here.
//
// REPRESENTATION DIVERGENCE (deliberate, and narrow): every consumer in the curated pilot
// closure re-exports `Int` and none of them use it in a typed position, so the checkpoint
// scalar rendering (rust_scalar_checkpoint_render_base, Int -> i64) is sufficient here and
// keeps the bridge free of the Nat coproduct. If a consumer ever uses Int in a typed position
// this alias is wrong and must be grounded on the real GroupCompletion pair construction.
//
// dissolve-on: emitted closure declares std_nat alongside the stub that imports it; then this
// file is deleted and the emitted module is used directly.
pub type Int = i64;
