//! Lane-local Rust mirror of the substrate `EmissionDiagnostic` carrier.
//!
//! Substrate authority **LANDED** at `src/v3/std/diagnostics.dag`
//! (T-Ground-Diagnostic; #1216 brief + #1133 dispatch 4355793511). The
//! substrate carrier declares `MissingEmissionPath { connective, behavior, target }`
//! among its variants. This Rust mirror persists because
//! `v3-grounding-cross-target-meta` cannot consume reflected substrate
//! types today (no `.dag` → Rust enum codegen path); when reflection-
//! driven generation lands, the mirror retires.
//!
//! **Lockstep discipline:** any new variant added to substrate
//! `EmissionDiagnostic` MUST land here (or in the analogous mirror in
//! the consuming crate) as a parallel Rust mirror update.

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
