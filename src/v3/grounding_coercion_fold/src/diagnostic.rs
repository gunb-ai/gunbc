//! Lane-local Rust mirror of the substrate `EmissionDiagnostic` carrier.
//!
//! Substrate authority **LANDED** at `src/v3/std/diagnostics.dag` (T-Ground-Diagnostic;
//! #1216 brief + #1133 dispatch 4355793511). The substrate carrier declares
//! `FoldNotImplemented` among its variants. This Rust mirror persists because
//! `v3-grounding-coercion-fold` cannot consume reflected substrate types today
//! (no `.dag` → Rust enum codegen path); when reflection-driven generation
//! lands, the mirror retires.
//!
//! **Lockstep discipline:** any new variant added to substrate
//! `EmissionDiagnostic` MUST land here as a parallel Rust mirror update.

/// Rust mirror of substrate `EmissionDiagnostic` (subset; this crate
/// only emits `FoldNotImplemented`). Variant-name lockstep enforced
/// socially until reflection codegen retires the mirror.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmissionDiagnostic {
    /// Full structural fold is not implemented; returning `Ok` would fabricate
    /// target choices (C-8 / `INVARIANTS.md`).
    FoldNotImplemented,
}
