//! Lane-local fail-closed diagnostic mirror for the coercion fold scaffold.
//!
//! **Staging debt (Practice 4):** matches the lane-local `EmissionDiagnostic` pattern
//! in `v3-grounding-lifetime` and the T-Ground-Diagnostic brief — migrates to the
//! substrate `EmissionDiagnostic` carrier when T-Ground-Diagnostic lands; see
//! `docs/briefs/t-ground-diagnostic.md` and #1216 convergence notes.

/// **🟡 Lane-local scaffold** — not the final substrate sum.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmissionDiagnostic {
    /// Full structural fold is not implemented; returning `Ok` would fabricate
    /// target choices (C-8 / `INVARIANTS.md`).
    FoldNotImplemented,
}
