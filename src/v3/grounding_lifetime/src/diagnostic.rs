//! Fail-closed diagnostic surface for the lifetime fold.
//!
//! ## Practice 4 (`docs/modeling-discipline.md` §4)
//!
//! The multi-variant diagnostic sum below is **lane-local** until substrate
//! authoring lands; see enum doc for 🟡 trigger.
//!
//! **T-Ground-Diagnostic** owns extending `CompilerDiagnosticKind` in `dsl/`.
//! This crate does **not** add Layer-1 variants; it carries a **lane-local**
//! mirror shape so callers and tests lock the emission payload before the
//! substrate `EmissionDiagnostic` carrier lands. Mapping to
//! `Diagnostic` / `CompilerDiagnosticKind` stays Coercion-Fold / Diagnostic-lane work.

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
