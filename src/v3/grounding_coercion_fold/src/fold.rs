//! Structural fold entry — **algorithm stub only**.
//!
//! ## Design authority (`docs/design-emission-model.md`)
//!
//! The eventual fold reads **program intent** + **substrate facts** (LanguageSpec rows,
//! algebra inhabitance, lifetime facts, …) and returns either a **unique** target
//! inhabitance per binding or a typed [`EmissionDiagnostic`](crate::diagnostic::EmissionDiagnostic).
//! Worked **Examples 1–7** in that doc (e.g. unrefined `Int` → `UnderRefined`, bounded
//! refinement → `u32`, `String` ownership cases, algebra ambiguity, no-inhabitant,
//! cross-target portability sketches) are the **behavioral targets** for the future
//! implementation — **not** implemented in this crate yet.
//!
//! ## Gates (dispatch)
//!
//! Full body is gated on T-Ground-LanguageSpec row work post-#1227 and lifetime /
//! substrate boundary metadata (#1130). This module only enforces the **public
//! signature** and **fail-closed** stub (`FoldNotImplemented`).

use std::collections::BTreeMap;

use v3_compiler::dag::Dag;
use v3_grounding_lifetime::{BindingId, LifetimeAnalysisReport};

use crate::diagnostic::EmissionDiagnostic;
use crate::types::{LanguageSpecProjectionUndeclared, TargetInhabitance};

/// Structural fold: program + lifetime analysis + (future) LanguageSpec projection →
/// per-binding target inhabitances, **or** a single typed diagnostic.
///
/// Today: **always** [`EmissionDiagnostic::FoldNotImplemented`](EmissionDiagnostic::FoldNotImplemented)
/// (C-8). Parameters are accepted so call sites compile against the eventual shape;
/// they are intentionally unused until the fold body lands.
pub fn fold_program_to_target(
    _dag: &Dag,
    _lifetime_facts: &LifetimeAnalysisReport,
    _language_spec: &LanguageSpecProjectionUndeclared,
) -> Result<BTreeMap<BindingId, TargetInhabitance>, EmissionDiagnostic> {
    Err(EmissionDiagnostic::FoldNotImplemented)
}
