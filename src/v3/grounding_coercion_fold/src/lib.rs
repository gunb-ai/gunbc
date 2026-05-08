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
//!   and **§Example 2** (`Int(0..2^32)` / `Semiring` exact-bound → declared Rust `u32`
//!   inhabitance row, surfaced through [`TargetInhabitance::RustU32`](types::TargetInhabitance::RustU32))
//!   plus **§Example 5** (same bound without algebra annotation → `UnderRefined("algebra")`)
//!   and **§Example 6** (`Int(0..2^65)` → [`EmissionDiagnostic::NoInhabitant`](diagnostic::EmissionDiagnostic::NoInhabitant))
//!   and **§Example 8** (`Int(-2^31..2^31)` → Rust `i32`, Python `int`, Go `int32`)
//!   for a single synthetic binding — checkpoint until a real LanguageSpec projection replaces
//!   the scratch carrier (manager #1133 / #1286). Before scratch outcomes run, the fold counts
//!   declared `TargetIntegerTypeInhabitance` rows in the bootstrap `dag`
//!   (**INVARIANTS.md E-6** same-PR consumer); Examples 2 and 8 then select by the row's
//!   `language`, `kernel_integer`, `algebra`, `bound`, and `type_realization` facts.
//!
//! **ScratchIntExamples ceiling (S7 Phase 1):** the checkpoint still runs **only** Examples 1, 2,
//! 5, 6, and 8 — it does **not** exercise end-to-end selection for platform-sized program bounds
//! (`usize`, Go `int`, …). Separately, the fold module already implements the structural
//! **`match_bound` predicate** for `PlatformDependentFact` target rows vs program platform-kind
//! bounds (Track B kind-only gate). Declared **spec rows** can name `PlatformDependentFact` (e.g. Go
//! native `int` in `v3/spec/go.dag`); wiring **program** `PlatformDependent` bounds through
//! `LanguageSpecProjection` + lifetime facts remains #1133 / #1286. Opportunistic fixed-width widening
//! (u128 / UInt128 rows) stays **STOP+PING** per ROADMAP Pattern D unless tied to the Int\<N\>/Nat\<N\>
//! refinement path.
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
pub use types::{
    IntScratchExample, IntegerBoundProjection, IntegerTargetIntent, LanguageSpecProjection,
    TargetInhabitance, TargetLanguage,
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use v3_compiler::dag::{Dag, Interval, IntervalWidth, PositiveIntervalWidth};
    use v3_grounding_lifetime::LifetimeAnalysisReport;

    use super::*;

    /// Bootstrap `Dag::new()` inference can overflow the default test thread stack after deep std
    /// construction chains (e.g. post–#1466 `Int = AbelianGroup<GroupCompletion<Nat>>`).
    fn with_bootstrap_stack<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("spawn bootstrap stack test thread")
            .join()
            .expect("bootstrap stack test thread panicked")
    }

    fn assert_single_binding_inhabitance(example: IntScratchExample, expected: TargetInhabitance) {
        with_bootstrap_stack(move || {
            let dag = Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let spec = LanguageSpecProjection::ScratchIntExamples(example);
            let got = fold_program_to_target(&dag, &lifetime, &spec).expect("ok");
            assert_eq!(got.len(), 1);
            assert_eq!(
                got.get(&v3_grounding_lifetime::BindingId(0)),
                Some(&expected)
            );
        });
    }

    #[test]
    fn fold_undeclared_projection_stays_fold_not_implemented() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let spec = LanguageSpecProjection::Undeclared;
            let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("undeclared");
            assert_eq!(err, EmissionDiagnostic::FoldNotImplemented);
        });
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
        with_bootstrap_stack(|| {
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
        });
    }

    #[test]
    fn design_doc_example_2_bounded_int_emits_rust_u32() {
        assert_single_binding_inhabitance(
            IntScratchExample::DesignDocExample2BoundedU32,
            TargetInhabitance::RustU32,
        );
    }

    #[test]
    fn declared_language_spec_integer_projection_emits_per_binding_inhabitance() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(7),
                IntegerTargetIntent {
                    target_language: TargetLanguage::Rust,
                    kernel_integer: "UInt32".to_string(),
                    algebra: "UInt32".to_string(),
                    bound: IntegerBoundProjection::Static(Interval::BoundedInterval {
                        lower: 0,
                        width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
                            units: 4_294_967_295,
                        }),
                    }),
                },
            )]));
            let got = fold_program_to_target(&dag, &lifetime, &spec).expect("ok");
            assert_eq!(got.len(), 1);
            assert_eq!(
                got.get(&v3_grounding_lifetime::BindingId(7)),
                Some(&TargetInhabitance::RustU32)
            );
        });
    }

    #[test]
    fn fold_dag_int_ambiguous_algebra_fails_closed() {
        with_bootstrap_stack(|| {
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
        });
    }

    #[test]
    fn fold_dag_int_bound_exceeds_max_no_inhabitant() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let spec = LanguageSpecProjection::ScratchIntExamples(
                IntScratchExample::DesignDocExample6NoInhabitant,
            );
            let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("no inhabitant");
            assert_eq!(err, EmissionDiagnostic::NoInhabitant);
        });
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

    #[test]
    fn fold_dag_int_refined_cross_target_requires_matching_declared_bound() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let wrong_bound = Interval::BoundedInterval {
                lower: -2_147_483_648,
                width: IntervalWidth::ZeroWidth,
            };
            let err = crate::fold::fold_design_doc_example_8_for_testing(
                &dag,
                IntScratchExample::DesignDocExample8Rust,
                wrong_bound,
            )
            .expect_err("mismatched bound should not select the declared i32 row");
            assert_eq!(err, EmissionDiagnostic::NoInhabitant);
        });
    }

    #[test]
    fn fold_dag_int_refined_cross_target_requires_matching_kernel_and_algebra() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let i32_bound = Interval::BoundedInterval {
                lower: -2_147_483_648,
                width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
                    units: 4_294_967_295,
                }),
            };
            let err = crate::fold::select_program_integer_intent_for_testing(
                &dag,
                IntScratchExample::DesignDocExample8Rust,
                "UInt32",
                "UInt32",
                "rust_i32",
                i32_bound,
            )
            .expect_err(
                "right target, bound, and realization with wrong kernel/algebra must not select",
            );
            assert_eq!(err, EmissionDiagnostic::NoInhabitant);
        });
    }
}
