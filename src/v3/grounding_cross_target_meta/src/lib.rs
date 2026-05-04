//! T-Ground-CrossTarget-Meta — **L6 cross-product walker scaffold** (R2
//! closure-pre-pass per manager dispatch #1133 inbox 4353412409).
//!
//! Per the lane brief at [`docs/briefs/t-ground-cross-target-meta.md`](../../../docs/briefs/t-ground-cross-target-meta.md)
//! (#1224 + L6 form-axis anchor #1229), this lane owns the
//! **substrate-load-time cross-product completeness check** that walks
//! `(TypeConnective × Behavior × Shape A target)` cells and verifies
//! each pair has a LanguageSpec emission-path declaration.
//!
//! ## Cross-product axes
//!
//! - **Form axis** — six [`v3_compiler::dag::TypeConnective`] variants
//!   per `src/v3/std/substrate.dag:164` (`Atom`, `Conj`, `Disj`,
//!   `Arrow`, `Cardinality`, `Instantiation`). The form axis is the
//!   substrate single-authority for "which kind of declaration shape is
//!   this?" — anchored by #1229.
//! - **Behavior axis** — five [`v3_compiler::dag::Behavior`] variants
//!   per L1 model (`Value`, `Transform`, `Branch`, `Loop`, `Bind`).
//! - **Target axis** — three Shape A targets (Rust / Python / Go).
//!
//! Total: `6 × 5 × 3 = 90` cells. Today's LanguageSpec has intentional
//! gaps named in deferral receipts elsewhere in the program (Rust
//! higher-order Phase 1.5; Go `chars` tokenizer; per-input fields
//! parser-grammar; etc.); the walker reports those as
//! [`EmissionDiagnostic::MissingEmissionPath`] without panicking — the
//! gaps are tracked, not unexpected failures.
//!
//! ## Not a `Lens<C>` instance
//!
//! Per the brief (and `docs/design-emission-model.md:958-959` /
//! `:1107-1111`), L6 lives at the substrate-load boundary as a
//! **runtime check**, not as a `Lens<C>` instance. The lens framework's
//! `Lens<C>.read: (Dag, Behavior) → Witness<C>` reads PER-Behavior
//! substrate facts; L6's input space is the (form × target) cross
//! product, which doesn't fit. So this crate produces typed diagnostics
//! directly without going through the lens machinery.
//!
//! ## SG-0 / discipline
//!
//! - **No** edits under [`src/v3/compiler/`](../../compiler) — consume
//!   `v3-compiler` APIs only.
//! - Lane-local [`EmissionDiagnostic`] mirrors the
//!   `T-Ground-Coercion-Fold` / `T-Ground-Lifetime-Analyzer` /
//!   `T-Ground-Diagnostic` pattern (#1216 convergence target —
//!   substrate carrier replaces this lane-local enum once it lands).
//! - **PR-J cadence-confirmation** is the formal acceptance gate per
//!   `r2-grounding-manager.md:128` (`cross_target_meta_l6_load_completeness_landed`).
//!   This scaffold lands the walker; PR-J handles cadence-confirmation
//!   as a separate substantive merge.
//!
//! ## Status
//!
//! **R2 closure-pre-pass — real coverage fold.** The walker enumerates
//! all 90 cells and resolves each against landed `MethodTemplateContract`
//! row authorities (`src/v3/std/*_method_template_contracts.dag`). At
//! HEAD, Phase 1 rows cover **Cardinality × Transform × Shape A target**
//! for each non-empty per-target list (see `coverage` module audit); all
//! other cells report [`EmissionDiagnostic::MissingEmissionPath`] as
//! honest structural gaps until additional LanguageSpec tables land.

use v3_compiler::dag::Dag;

mod cells;
#[cfg(test)]
mod closure_ledger_gate;
mod coverage;
mod diagnostic;
mod walker;

pub use cells::{BehaviorAxis, Cell, FormAxis, ShapeATarget};
pub use diagnostic::EmissionDiagnostic;
pub use walker::{walk_cross_product, CrossProductReport};

/// Public entry: walk the (TypeConnective × Behavior × Shape A target)
/// cross product against the bootstrap Dag's LanguageSpec coverage and
/// produce a [`CrossProductReport`] enumerating present / missing cells.
///
/// Pure function over the Dag; no mutation, no side effects.
pub fn check_l6_load_completeness(dag: &Dag) -> CrossProductReport {
    walk_cross_product(dag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use v3_compiler::generated_full_bootstrap_dag;

    /// Expected L6 coverage at HEAD from Step 1 audit of
    /// `rust_method_template_contracts` / `python_method_template_contracts` /
    /// `go_method_template_contracts`: each list is non-empty (Rust 13, Python 17,
    /// Go 13 rows) and every Phase 1 row maps to the single structural bucket
    /// **Cardinality × Transform × &lt;target&gt;** for collection method templates.
    const EXPECTED_PRESENT_COUNT: usize = 3;
    const EXPECTED_MISSING_COUNT: usize = 90 - EXPECTED_PRESENT_COUNT;

    #[test]
    fn cross_product_walks_90_cells() {
        let dag = generated_full_bootstrap_dag();
        let report = check_l6_load_completeness(&dag);
        assert_eq!(
            report.total_cells(),
            90,
            "cross product must cover 6 connectives × 5 behaviors × 3 targets = 90 cells; got {}",
            report.total_cells()
        );
    }

    #[test]
    fn coverage_matches_audit_table() {
        let dag = generated_full_bootstrap_dag();
        let report = check_l6_load_completeness(&dag);
        assert_eq!(
            report.present.len(),
            EXPECTED_PRESENT_COUNT,
            "present cells must match audit table (Cardinality×Transform×each target \
             when that target's MethodTemplateContract list is non-empty)"
        );
        assert_eq!(report.missing.len(), EXPECTED_MISSING_COUNT);
        let expected_present = [
            Cell {
                connective: FormAxis::Cardinality,
                behavior: BehaviorAxis::Transform,
                target: ShapeATarget::Rust,
            },
            Cell {
                connective: FormAxis::Cardinality,
                behavior: BehaviorAxis::Transform,
                target: ShapeATarget::Python,
            },
            Cell {
                connective: FormAxis::Cardinality,
                behavior: BehaviorAxis::Transform,
                target: ShapeATarget::Go,
            },
        ];
        for cell in expected_present {
            assert!(
                report.present.contains(&cell),
                "expected {:?} present per landed row audit",
                cell
            );
        }
    }

    #[test]
    fn every_missing_cell_has_typed_diagnostic() {
        let dag = generated_full_bootstrap_dag();
        let report = check_l6_load_completeness(&dag);
        for (cell, diag) in &report.missing {
            // Each missing entry IS a typed diagnostic (per C-8 fail-closed
            // discipline + #1216 EmissionDiagnostic convergence pattern).
            // The walker pairs every missing cell with its
            // `MissingEmissionPath { connective, behavior, target }` diagnostic
            // carrying all three axis values for the closure ledger to consume.
            match diag {
                EmissionDiagnostic::MissingEmissionPath {
                    connective,
                    behavior,
                    target,
                } => {
                    assert_eq!(*connective, cell.connective);
                    assert_eq!(*behavior, cell.behavior);
                    assert_eq!(*target, cell.target);
                }
            }
        }
    }
}
