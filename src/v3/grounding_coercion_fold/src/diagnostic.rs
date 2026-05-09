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
//!
//! **UnderRefined axes:** mirror substrate’s string axis payload; declared projection
//! selection may surface axes such as `"target_integer_inhabitance"` or row-population gates.

/// Rust mirror of substrate `EmissionDiagnostic` (subset; grows with substrate).
///
/// Practice 4 (`docs/modeling-discipline.md`): **🟡 YELLOW** — hand-maintained mirror until this
/// lane consumes reflected substrate types (`.dag` → Rust enum codegen); dissolution aligns with
/// #1216 / #1133 (no second authority for the same diagnostic facts — lockstep only).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmissionDiagnostic {
    /// Program intent or structural axis under-specified (substrate `UnderRefined`).
    UnderRefined {
        /// Mirrors substrate’s string axis payload for now; when substrate grows structured
        /// axes (or this mirror gains a sum type), replace free-form spelling here rather than
        /// widening implicit string conventions.
        unspecified_axis: String,
    },
    /// No target inhabitant can satisfy the requested structural facts.
    NoInhabitant,
    /// Full structural fold is not implemented; returning `Ok` would fabricate
    /// target choices (C-8 / `INVARIANTS.md`).
    FoldNotImplemented,
}
