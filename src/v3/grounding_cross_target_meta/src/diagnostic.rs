//! Lane-local `EmissionDiagnostic` mirror — same convergence pattern as
//! `T-Ground-Coercion-Fold` / `T-Ground-Lifetime-Analyzer` lane-local
//! mirrors. Substrate-side `EmissionDiagnostic` carrier (per
//! `docs/briefs/t-ground-diagnostic.md` / #1216) replaces this enum
//! when it lands; until then this lane carries its own typed channel
//! per the C-8 fail-closed discipline (every detectable problem is a
//! typed diagnostic).

use crate::cells::{BehaviorAxis, Cell, FormAxis, ShapeATarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmissionDiagnostic {
    /// L6 cross-product cell has no LanguageSpec emission-path
    /// declaration. This is the structural authority for
    /// "the substrate is missing a target-mapping for this
    /// (form × behavior × target) triple." Per
    /// `docs/design-emission-model.md:408-410` and the
    /// CrossTarget-Meta brief: missing entries surface as typed
    /// diagnostics, not silent passes.
    MissingEmissionPath {
        connective: FormAxis,
        behavior: BehaviorAxis,
        target: ShapeATarget,
    },
}

impl EmissionDiagnostic {
    /// Construct a `MissingEmissionPath` diagnostic from a `Cell`.
    pub fn missing_emission_path(cell: &Cell) -> EmissionDiagnostic {
        EmissionDiagnostic::MissingEmissionPath {
            connective: cell.connective,
            behavior: cell.behavior,
            target: cell.target,
        }
    }
}
