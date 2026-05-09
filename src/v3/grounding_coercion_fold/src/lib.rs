//! T-Ground-Coercion-Fold — structural emission fold (**coercion = emission**).
//!
//! Per [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md), this lane
//! reads program intent + substrate facts and returns a unique selected target per binding
//! or [`EmissionDiagnostic`](diagnostic::EmissionDiagnostic).
//!
//! ## Implemented today
//!
//! - [`fold_program_to_target`](fold::fold_program_to_target) with
//!   [`LanguageSpecProjection::Undeclared`](types::LanguageSpecProjection::Undeclared) stays
//!   fail-closed [`EmissionDiagnostic::FoldNotImplemented`](diagnostic::EmissionDiagnostic::FoldNotImplemented).
//! - [`LanguageSpecProjection::DeclaredIntegerIntents`](types::LanguageSpecProjection::DeclaredIntegerIntents)
//!   selects declared [`TargetIntegerTypeInhabitance`](../../std/emit_model.dag) rows per binding
//!   and returns [`SelectedTargetInhabitance`](types::SelectedTargetInhabitance) (the row’s
//!   `type_realization` reference) only after validating that reference is substrate
//!   `data …: TypeRealization` (**`dag.type_realization_meta()`** match — fail-closed otherwise).
//!   Before selection, the fold counts meta-tagged inhabitance rows
//!   in the bootstrap `dag` (**INVARIANTS.md E-6**).
//!
//! Declared **spec rows** can name `PlatformDependentFact` (e.g. Go native `int` in
//! `v3/spec/go.dag`); wiring **program** `PlatformDependent` bounds through
//! `LanguageSpecProjection` + lifetime facts remains #1133 / #1286.
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
pub use fold::{fold_program_to_target, MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS};
pub use types::{
    IntegerBoundProjection, IntegerTargetIntent, LanguageSpecProjection, SelectedTargetInhabitance,
};

#[cfg(test)]
mod tests {
    // Design-emission-model Examples 1 (`UnderRefined` / bound) and 5 (`UnderRefined` / algebra) were
    // scratch-simulated only. `DeclaredIntegerIntents` always carries explicit `IntegerTargetIntent`
    // bound + algebra ids, so those axis receipts wait on program-side projection (#1133 / #1286).
    // Post-G3 `rg` over `*.rs` under `src/v3` shows no `ScratchIntExamples` consumers.

    use std::collections::BTreeMap;

    use v3_compiler::dag::{Interval, IntervalWidth, PositiveIntervalWidth};
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

    fn declaration_id_by_name(
        dag: &v3_compiler::dag::Dag,
        name: &str,
    ) -> v3_compiler::dag::DeclarationId {
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("missing declaration `{name}`"))
            .id
    }

    fn example_8_i32_interval() -> Interval<i64> {
        Interval::BoundedInterval {
            lower: -2_147_483_648,
            width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
                units: 4_294_967_295,
            }),
        }
    }

    #[test]
    fn fold_undeclared_projection_stays_fold_not_implemented() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let spec = LanguageSpecProjection::Undeclared;
            let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("undeclared");
            assert_eq!(err, EmissionDiagnostic::FoldNotImplemented);
        });
    }

    #[test]
    fn declared_projection_requires_emit_model_inhabitance_rows() {
        // Opaque ids: row-count gate returns before dereferencing projections (witness-only ctor).
        let dummy_decl = v3_compiler::dag::DeclarationId::declaration_id_raw_for_testing(0);
        let dag = v3_compiler::dag::Dag::new_empty_for_testing();
        let lifetime: LifetimeAnalysisReport = Default::default();
        let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
            v3_grounding_lifetime::BindingId(0),
            IntegerTargetIntent {
                target_language: dummy_decl,
                kernel_integer: dummy_decl,
                algebra: dummy_decl,
                bound: IntegerBoundProjection::Static(Interval::Unbounded),
            },
        )]));
        let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("row gate");
        assert_eq!(
            err,
            EmissionDiagnostic::UnderRefined {
                unspecified_axis: "declared_TargetIntegerTypeInhabitance_rows".to_string(),
            }
        );
    }

    #[test]
    fn declared_language_spec_integer_projection_emits_per_binding_inhabitance() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let rust_u32 = declaration_id_by_name(&dag, "rust_u32");
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(7),
                IntegerTargetIntent {
                    target_language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "UInt32"),
                    algebra: declaration_id_by_name(&dag, "UInt32"),
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
                got.get(&v3_grounding_lifetime::BindingId(7))
                    .map(|s| s.type_realization()),
                Some(rust_u32)
            );
        });
    }

    #[test]
    fn design_doc_example_2_bounded_int_selects_declared_rust_u32_row() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let rust_u32 = declaration_id_by_name(&dag, "rust_u32");
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(0),
                IntegerTargetIntent {
                    target_language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "UInt32"),
                    algebra: declaration_id_by_name(&dag, "UInt32"),
                    bound: IntegerBoundProjection::Static(Interval::BoundedInterval {
                        lower: 0,
                        width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
                            units: 4_294_967_295,
                        }),
                    }),
                },
            )]));
            let got = fold_program_to_target(&dag, &lifetime, &spec).expect("ok");
            assert_eq!(
                got.get(&v3_grounding_lifetime::BindingId(0))
                    .map(|s| s.type_realization()),
                Some(rust_u32)
            );
        });
    }

    #[test]
    fn fold_intent_bound_with_no_matching_row_returns_no_inhabitant() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(0),
                IntegerTargetIntent {
                    target_language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "UInt32"),
                    algebra: declaration_id_by_name(&dag, "UInt32"),
                    bound: IntegerBoundProjection::Static(Interval::BoundedInterval {
                        lower: 0,
                        width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
                            units: 42,
                        }),
                    }),
                },
            )]));
            let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("no inhabitant");
            assert_eq!(err, EmissionDiagnostic::NoInhabitant);
        });
    }

    #[test]
    fn fold_example_8_rust_i32_row_via_declared_projection() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let rust_i32 = declaration_id_by_name(&dag, "rust_i32");
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(0),
                IntegerTargetIntent {
                    target_language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "Int32"),
                    algebra: declaration_id_by_name(&dag, "Int32"),
                    bound: IntegerBoundProjection::Static(example_8_i32_interval()),
                },
            )]));
            let got = fold_program_to_target(&dag, &lifetime, &spec).expect("ok");
            assert_eq!(
                got.get(&v3_grounding_lifetime::BindingId(0))
                    .map(|s| s.type_realization()),
                Some(rust_i32)
            );
        });
    }

    #[test]
    fn fold_example_8_python_int_row_via_declared_projection() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let python_int = declaration_id_by_name(&dag, "python_int");
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(0),
                IntegerTargetIntent {
                    target_language: dag.python_language_spec().expect("Python LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "Int"),
                    algebra: declaration_id_by_name(&dag, "Int"),
                    bound: IntegerBoundProjection::Static(example_8_i32_interval()),
                },
            )]));
            let got = fold_program_to_target(&dag, &lifetime, &spec).expect("ok");
            assert_eq!(
                got.get(&v3_grounding_lifetime::BindingId(0))
                    .map(|s| s.type_realization()),
                Some(python_int)
            );
        });
    }

    #[test]
    fn fold_example_8_go_int32_row_via_declared_projection() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let go_int32 = declaration_id_by_name(&dag, "go_int32");
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(0),
                IntegerTargetIntent {
                    target_language: dag.go_language_spec().expect("Go LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "Int32"),
                    algebra: declaration_id_by_name(&dag, "Int32"),
                    bound: IntegerBoundProjection::Static(example_8_i32_interval()),
                },
            )]));
            let got = fold_program_to_target(&dag, &lifetime, &spec).expect("ok");
            assert_eq!(
                got.get(&v3_grounding_lifetime::BindingId(0))
                    .map(|s| s.type_realization()),
                Some(go_int32)
            );
        });
    }

    #[test]
    fn fold_mismatched_static_bound_yields_no_inhabitant() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let wrong_bound = Interval::BoundedInterval {
                lower: -2_147_483_648,
                width: IntervalWidth::ZeroWidth,
            };
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(0),
                IntegerTargetIntent {
                    target_language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "Int32"),
                    algebra: declaration_id_by_name(&dag, "Int32"),
                    bound: IntegerBoundProjection::Static(wrong_bound),
                },
            )]));
            let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("no match");
            assert_eq!(err, EmissionDiagnostic::NoInhabitant);
        });
    }

    #[test]
    fn fold_wrong_kernel_algebra_pair_yields_no_inhabitant() {
        with_bootstrap_stack(|| {
            let dag = v3_compiler::dag::Dag::new();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let spec = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                v3_grounding_lifetime::BindingId(0),
                IntegerTargetIntent {
                    target_language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "UInt32"),
                    algebra: declaration_id_by_name(&dag, "UInt32"),
                    bound: IntegerBoundProjection::Static(example_8_i32_interval()),
                },
            )]));
            let err = fold_program_to_target(&dag, &lifetime, &spec).expect_err("no match");
            assert_eq!(err, EmissionDiagnostic::NoInhabitant);
        });
    }
}
