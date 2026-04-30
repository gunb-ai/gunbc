//! L6 cross-product walker. Pure function over the bootstrap [`Dag`];
//! reports per-cell coverage against the LanguageSpec emission-path
//! table.
//!
//! ## Coverage lookup
//!
//! Today's `LanguageSpec` (`src/v3/std/emit_model.dag:303`) does NOT
//! yet carry an `emission_paths: Map<(FormAxis, BehaviorAxis), ...>`
//! table. The intended substrate authority for that table lands as
//! part of `T-Ground-LanguageSpec` Phase 1.5+. Until it lands, every
//! cell resolves to "not covered" and the walker reports 90/90
//! `MissingEmissionPath` diagnostics. The lookup itself is wired
//! structurally so that when the table lands, this walker's coverage
//! check grows without test rewrite — the intended `lookup_coverage`
//! signature stays the same; only its body grows from "always
//! `None`" to "consult the LanguageSpec table per cell."

use v3_compiler::dag::Dag;

use crate::cells::Cell;
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
    /// Total cells walked. Always 6 × 5 × 3 = 90 (the cross-product
    /// shape is structural; only coverage of each cell varies).
    pub fn total_cells(&self) -> usize {
        self.present.len() + self.missing.len()
    }
}

/// Walk every (form × behavior × target) cell and resolve coverage
/// against the bootstrap Dag's LanguageSpec.
///
/// **Pure function** — no mutation, no side effects, no panics.
pub fn walk_cross_product(dag: &Dag) -> CrossProductReport {
    let mut present = Vec::new();
    let mut missing = Vec::new();
    for cell in Cell::all() {
        if cell_covered_by_language_spec(dag, &cell) {
            present.push(cell);
        } else {
            missing.push((cell, EmissionDiagnostic::missing_emission_path(&cell)));
        }
    }
    CrossProductReport { present, missing }
}

/// Cell-coverage probe against `LanguageSpec`. Today's substrate
/// has no per-cell emission-paths table, so this returns `false` for
/// all cells. When `T-Ground-LanguageSpec` Phase 1.5+ adds the
/// table, this body grows to consult it without changing the public
/// signature or the walker.
fn cell_covered_by_language_spec(_dag: &Dag, _cell: &Cell) -> bool {
    // Structural placeholder — LanguageSpec.emission_paths table not
    // yet declared (Phase 1.5+ Substrate slice). All cells report
    // missing until the table lands; gaps are tracked, not silent.
    false
}
