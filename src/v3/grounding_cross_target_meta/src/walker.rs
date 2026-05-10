//! L6 cross-product walker. Pure function over the bootstrap [`Dag`];
//! reports per-cell coverage against landed LanguageSpec **row**
//! authorities (Phase 1 `MethodTemplateContract` lists — see
//! [`crate::coverage`]).
//!
//! ## Coverage lookup
//!
//! Coverage is derived from substrate-loaded `List<MethodTemplateContract>`
//! declarations per Shape A target (`*_method_template_contracts.dag`), not
//! from a dedicated `emission_paths` map. [`walk_cross_product`] builds the
//! covered-cell set via [`crate::coverage::language_spec_emission_cells_covered`],
//! then partitions [`crate::cells::Cell::all`] with `covered.contains(&cell)`.

use v3_compiler::dag::Dag;

use crate::cells::{Cell, ShapeATarget};
use crate::coverage::{language_spec_emission_cells_covered, language_spec_targets};
use crate::diagnostic::EmissionDiagnostic;

/// Per-cell coverage report produced by the L6 walker.
///
/// `present` lists cells where the LanguageSpec declares an emission
/// path (covered). `missing` lists cells where no declaration exists,
/// each accompanied by a typed `EmissionDiagnostic`.
#[derive(Debug, Clone)]
pub struct CrossProductReport {
    pub present: Vec<Cell>,
    pub missing: Vec<(Cell, EmissionDiagnostic)>,
}

impl CrossProductReport {
    /// Total cells walked.
    pub fn total_cells(&self) -> usize {
        self.present.len() + self.missing.len()
    }
}

/// Walk every (form × behavior × target) cell and resolve coverage
/// against the bootstrap Dag's LanguageSpec.
///
/// **Pure function** — no mutation, no side effects, no panics.
pub fn walk_cross_product(dag: &Dag) -> CrossProductReport {
    let covered = language_spec_emission_cells_covered(dag);
    let targets = language_spec_targets(dag).unwrap_or_else(|| {
        covered
            .iter()
            .map(|cell| cell.target)
            .collect::<Vec<ShapeATarget>>()
    });
    let mut present = Vec::new();
    let mut missing = Vec::new();
    for cell in Cell::all(&targets) {
        if covered.contains(&cell) {
            present.push(cell);
        } else {
            missing.push((cell, EmissionDiagnostic::missing_emission_path(&cell)));
        }
    }
    CrossProductReport { present, missing }
}
