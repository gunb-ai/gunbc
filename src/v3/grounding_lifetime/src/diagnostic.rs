//! Fail-closed diagnostic surface for the lifetime fold.
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
}
