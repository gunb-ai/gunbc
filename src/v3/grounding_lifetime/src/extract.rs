//! Dag → `LifetimeProgram` projection (R2 extraction).
//!
//! Today’s `Dag::new()` bootstrap does not yet lower user `data` / `fn` bodies into
//! the bind/use graph this analyzer needs. Extraction returns an empty program until
//! that lowering lands; the structural fold is validated via [`LifetimeProgram`] fixtures.

use v3_compiler::dag::Dag;

use crate::analyze;
use crate::axes::LanguageSpecAxes;
use crate::diagnostic::EmissionDiagnostic;
use crate::facts::LifetimeFacts;
use crate::program::{BindingId, LifetimeProgram};

use std::collections::BTreeMap;

/// Project the reflected `Dag` into the lifetime analyzer’s program slice.
///
/// Fail-closed on constructs the R2 analyzer does not model (once lowering surfaces them).
pub fn extract_lifetime_program(dag: &Dag) -> Result<LifetimeProgram, EmissionDiagnostic> {
    let _ = dag;
    Ok(LifetimeProgram::empty())
}

/// Public entry: **`Dag` + `LanguageSpecAxes` only** — no annotation table, no env sidecar
/// (`t-ground-lifetime-analyzer.md` test plan item 7).
pub fn analyze_lifetime_facts(
    dag: &Dag,
    axes: &LanguageSpecAxes,
) -> Result<BTreeMap<BindingId, LifetimeFacts>, EmissionDiagnostic> {
    let program = extract_lifetime_program(dag)?;
    analyze::analyze_lifetime_program(&program, axes)
}
