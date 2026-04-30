//! T-Ground-Coercion-Fold — **consumer-side scaffold** for the structural emission fold.
//!
//! Per [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md) lane table
//! (~381–384), this lane is the mechanical **coercion = emission** fold: read program
//! intent + substrate facts, return a unique target inhabitance per binding or
//! [`EmissionDiagnostic`](diagnostic::EmissionDiagnostic). It **replaces** the retracted
//! T-Ground-Engine framing (no separate coercion engine).
//!
//! ## Algorithm-shape (stub)
//!
//! - [`fold_program_to_target`](fold::fold_program_to_target) is the public entry matching
//!   the manager dispatch shape: [`v3_compiler::dag::Dag`], [`v3_grounding_lifetime::LifetimeAnalysisReport`],
//!   and [`types::LanguageSpecProjectionUndeclared`] (open carrier — **do not** guess final
//!   LanguageSpec projection).
//! - Until LanguageSpec + Dag boundary work land, the fold **fail-closed** returns
//!   [`EmissionDiagnostic::FoldNotImplemented`](diagnostic::EmissionDiagnostic::FoldNotImplemented)
//!   for all inputs (C-8).
//!
//! ## Worked examples (targets for future implementation; not implemented here)
//!
//! Authoritative walkthroughs live in [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md)
//! **§Worked examples** — including at minimum **Example 1** (unrefined `Int` → fail-closed
//! diagnostic), **Example 2** (`Int(0..2^32)` → Rust `u32`), **Examples 3–4** (`String`
//! ownership / lifetime), **Example 5** (algebra ambiguity), **Example 6** (`NoInhabitant`),
//! and **Example 7** / cross-target sketches tied to Modeling problem 7. This crate’s
//! rustdoc on [`fold`](crate::fold) cites the same authority without duplicating payloads.
//!
//! ## SG-0 / discipline
//!
//! - **No** edits under [`src/v3/compiler/`](../../compiler) — consume `v3-compiler` APIs only.
//! - Lane-local [`EmissionDiagnostic`](diagnostic::EmissionDiagnostic) mirrors the Lifetime-Analyzer
//!   pattern; converges on substrate carrier per `docs/briefs/t-ground-diagnostic.md` / #1216.

mod diagnostic;
mod fold;
mod types;

pub use diagnostic::EmissionDiagnostic;
pub use fold::fold_program_to_target;
pub use types::{LanguageSpecProjectionUndeclared, TargetInhabitance};

#[cfg(test)]
mod tests {
    use v3_compiler::dag::Dag;
    use v3_grounding_lifetime::LifetimeAnalysisReport;

    use super::*;

    #[test]
    fn fold_stub_fail_closed_fold_not_implemented() {
        let dag = Dag::new();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjectionUndeclared;
        let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("stub");
        assert_eq!(err, EmissionDiagnostic::FoldNotImplemented);
    }
}
