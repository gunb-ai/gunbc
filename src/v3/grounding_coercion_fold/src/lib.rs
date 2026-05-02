//! T-Ground-Coercion-Fold — structural emission fold (**coercion = emission**).
//!
//! Per [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md), this lane
//! reads program intent + substrate facts and returns a unique target inhabitance per binding
//! or [`EmissionDiagnostic`](diagnostic::EmissionDiagnostic).
//!
//! ## Implemented today
//!
//! - [`fold_program_to_target`](fold::fold_program_to_target) with
//!   [`LanguageSpecProjection::Undeclared`](types::LanguageSpecProjection::Undeclared) stays
//!   fail-closed [`EmissionDiagnostic::FoldNotImplemented`](diagnostic::EmissionDiagnostic::FoldNotImplemented).
//! - [`LanguageSpecProjection::ScratchIntExamples`](types::LanguageSpecProjection::ScratchIntExamples)
//!   drives **design-emission-model §Example 1** (unrefined `Int` → substrate-shaped
//!   [`EmissionDiagnostic::UnderRefined`](diagnostic::EmissionDiagnostic::UnderRefined) with axis `"bound"`)
//!   and **§Example 2** (`Int(0..2^32)` / `Semiring` exact-bound → [`TargetInhabitance::RustU32`](types::TargetInhabitance::RustU32))
//!   plus **§Example 5** (same bound without algebra annotation → `UnderRefined("algebra")`)
//!   and **§Example 6** (`Int(0..2^65)` → [`EmissionDiagnostic::NoInhabitant`](diagnostic::EmissionDiagnostic::NoInhabitant))
//!   and **§Example 8** (`Int(-2^31..2^31)` → Rust `i32`, Python `int`, Go `int32`)
//!   for a single synthetic binding — checkpoint until a real LanguageSpec projection replaces
//!   the scratch carrier (manager #1133 / #1286). **Slice B:** before scratch outcomes run,
//!   the fold counts declared `TargetIntegerTypeInhabitance` rows in the bootstrap `dag`
//!   (**INVARIANTS.md E-6** same-PR consumer).
//!
//! ## SG-0 / discipline
//!
//! - **No** edits under [`src/v3/compiler/`](../../compiler) — consume `v3-compiler` APIs only.
//! - Lane-local [`EmissionDiagnostic`](diagnostic::EmissionDiagnostic) converges per
//!   `docs/briefs/t-ground-diagnostic.md` / #1216.

mod diagnostic;
mod fold;
mod types;

pub use diagnostic::EmissionDiagnostic;
pub use fold::fold_program_to_target;
pub use types::{IntScratchExample, LanguageSpecProjection, TargetInhabitance};

#[cfg(test)]
mod tests {
    use v3_compiler::dag::Dag;
    use v3_grounding_lifetime::LifetimeAnalysisReport;

    use super::*;

    fn assert_single_binding_inhabitance(example: IntScratchExample, expected: TargetInhabitance) {
        let dag = Dag::new();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjection::ScratchIntExamples(example);
        let got = fold_program_to_target(&dag, &lifetime, &spec).expect("ok");
        assert_eq!(got.len(), 1);
        assert_eq!(
            got.get(&v3_grounding_lifetime::BindingId(0)),
            Some(&expected)
        );
    }

    #[test]
    fn fold_undeclared_projection_stays_fold_not_implemented() {
        let dag = Dag::new();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjection::Undeclared;
        let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("undeclared");
        assert_eq!(err, EmissionDiagnostic::FoldNotImplemented);
    }

    #[test]
    fn scratch_int_examples_require_emit_model_inhabitance_rows() {
        let dag = Dag::new_empty_for_testing();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjection::ScratchIntExamples(
            IntScratchExample::DesignDocExample2BoundedU32,
        );
        let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("row gate");
        assert_eq!(
            err,
            EmissionDiagnostic::UnderRefined {
                unspecified_axis: "declared_TargetIntegerTypeInhabitance_rows".to_string(),
            }
        );
    }

    #[test]
    fn design_doc_example_1_unrefined_int_under_refined() {
        let dag = Dag::new();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjection::ScratchIntExamples(
            IntScratchExample::DesignDocExample1UnrefinedInt,
        );
        let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("under-refined");
        assert_eq!(
            err,
            EmissionDiagnostic::UnderRefined {
                unspecified_axis: "bound".to_string(),
            }
        );
    }

    #[test]
    fn design_doc_example_2_bounded_int_emits_rust_u32() {
        assert_single_binding_inhabitance(
            IntScratchExample::DesignDocExample2BoundedU32,
            TargetInhabitance::RustU32,
        );
    }

    #[test]
    fn fold_dag_int_ambiguous_algebra_fails_closed() {
        let dag = Dag::new();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjection::ScratchIntExamples(
            IntScratchExample::DesignDocExample5AmbiguousAlgebra,
        );
        let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("under-refined");
        assert_eq!(
            err,
            EmissionDiagnostic::UnderRefined {
                unspecified_axis: "algebra".to_string(),
            }
        );
    }

    #[test]
    fn fold_dag_int_bound_exceeds_max_no_inhabitant() {
        let dag = Dag::new();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjection::ScratchIntExamples(
            IntScratchExample::DesignDocExample6NoInhabitant,
        );
        let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("no inhabitant");
        assert_eq!(err, EmissionDiagnostic::NoInhabitant);
    }

    #[test]
    fn fold_dag_int_refined_cross_target_consistent_rust() {
        assert_single_binding_inhabitance(
            IntScratchExample::DesignDocExample8Rust,
            TargetInhabitance::RustI32,
        );
    }

    #[test]
    fn fold_dag_int_refined_cross_target_consistent_python() {
        assert_single_binding_inhabitance(
            IntScratchExample::DesignDocExample8Python,
            TargetInhabitance::PythonInt,
        );
    }

    #[test]
    fn fold_dag_int_refined_cross_target_consistent_go() {
        assert_single_binding_inhabitance(
            IntScratchExample::DesignDocExample8Go,
            TargetInhabitance::GoInt32,
        );
    }
}
