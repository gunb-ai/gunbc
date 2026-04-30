//! Fail-closed diagnostic surface for the lifetime fold.
//!
//! ## Practice 4 — substrate authority LANDED (per #1133 dispatch 4355793511)
//!
//! The substrate `EmissionDiagnostic` carrier is now declared at
//! `src/v3/std/diagnostics.dag` (T-Ground-Diagnostic; #1216 brief). This
//! crate's Rust mirror below carries the same variant set; the lane-local
//! mirror persists because `v3-grounding-lifetime` cannot consume
//! reflected substrate types today (no `.dag` → Rust enum codegen path).
//!
//! **Lockstep discipline:** any new variant added to substrate
//! `EmissionDiagnostic` MUST land here as a parallel Rust mirror update.
//! Variant-name parity is enforced socially until reflection-driven
//! Rust generation lands. The `axis: String` field below corresponds to
//! the substrate carrier's `unspecified_axis: String` — naming will
//! reconcile when the reflection codegen retires this mirror.
//!
//! **Q6.5 anti-bridge preserved:** `EmissionDiagnostic` is a SEPARATE
//! substrate carrier from `CompilerDiagnosticKind`. Mapping to
//! `Diagnostic` / `AnyDiagnosticKind` is Coercion-Fold / emit-pipeline
//! consumer work, not authored here.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SiteRef {
    pub label: String,
}

/// **Practice 4 — 🟡 YELLOW (lane-local scaffold).** Named trigger: substrate
/// `EmissionDiagnostic` in T-Ground-Diagnostic + fold consumer mapping to
/// `CompilerDiagnosticKind` / `Diagnostic` replaces this mirror; free-form
/// `axis` / `construct` strings may later become typed carriers (review note).
///
/// Typed failures for the structural lifetime analyzer (`design-emission-model.md` + brief §F).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmissionDiagnostic {
    ContradictoryUse {
        binding: String,
        sites: Vec<SiteRef>,
    },
    UnderRefined {
        axis: String,
    },
    OutOfR2Scope {
        construct: String,
    },
    /// `Dag` → `LifetimeProgram` lowering is not wired yet, but the reflected DAG
    /// carries **non-authority** declarations (user / test modules) with `data` or
    /// `fn` surface — returning `Ok(empty)` would silently drop load-bearing program
    /// shape (C-8 / modeling-discipline principle 1).
    LifetimeProgramExtractionPending {
        detail: String,
    },
}
