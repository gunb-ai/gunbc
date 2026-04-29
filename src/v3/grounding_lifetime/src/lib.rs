//! T-Ground-Lifetime-Analyzer — R2 structural ownership / lifetime / growability fold.
//!
//! **Scope:** (a) top-level data bindings, (b) function parameters with transient vs
//! escaping use, (c) function return values — per `docs/briefs/t-ground-lifetime-analyzer.md`
//! and `docs/design-emission-model.md:635`. **No annotation surface** (`THESIS.md:171`).
//!
//! ## P1 receipts (`INVARIANTS.md` §P1, brief §G)
//!
//! See [`facts`](crate::facts) module rustdoc for Step 1–3 citations.
//!
//! ## Pipeline split
//!
//! - [`analyze_lifetime_program`](analyze::analyze_lifetime_program) is the structural fold
//!   over a [`LifetimeProgram`](program::LifetimeProgram) (bindings + classified use sites).
//! - [`extract_lifetime_program`](extract::extract_lifetime_program) projects `Dag` →
//!   `LifetimeProgram` (bootstrap authority corpora only; **fail-closed** on user/test
//!   `data` / `fn` surface until lowering exposes the R2 bind/use graph).
//! - [`analyze_lifetime_facts`](extract::analyze_lifetime_facts) is the public entry:
//!   **`(&Dag, &LanguageSpecAxes)` only** — test plan item 7 / no annotation sidecar.

mod analyze;
pub mod axes;
pub mod diagnostic;
pub mod facts;
pub mod program;

mod extract;

pub use analyze::analyze_lifetime_program;
pub use axes::LanguageSpecAxes;
pub use diagnostic::{EmissionDiagnostic, SiteRef};
pub use extract::{analyze_lifetime_facts, extract_lifetime_program};
pub use facts::{Encoding, Growability, LifetimeFacts, LifetimeScope, Ownership};
pub use program::{
    BindingDef, BindingId, BindingRole, LifetimeProgram, ProgramTypeFamily, R3Construct, UseKind,
    UseSite,
};

use std::collections::BTreeMap;

/// Map from binding id to derived facts for one analysis pass.
pub type LifetimeAnalysisReport = BTreeMap<BindingId, LifetimeFacts>;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use v3_compiler::dag::Dag;

    use super::*;

    fn facts_for(program: &LifetimeProgram) -> LifetimeAnalysisReport {
        analyze_lifetime_program(program, &LanguageSpecAxes::example_rust_string_family())
            .expect("analysis ok")
    }

    /// Test plan item 1 — Example 3 (`design-emission-model.md:523-583`).
    #[test]
    fn example3_top_level_string_derives_owned_self_non_growable() {
        let program = LifetimeProgram::example3_top_level_string_name();
        let m = facts_for(&program);
        let f = m.get(&BindingId(0)).expect("name binding");
        assert_eq!(f.ownership, Ownership::Owned);
        assert_eq!(f.lifetime, LifetimeScope::Self_);
        assert_eq!(f.growable, Growability::No);
        assert_eq!(f.encoding, Encoding::Utf8FreeMonoidChar);
    }

    /// Test plan item 2 — Example 4 Case A (`design-emission-model.md:585-635`).
    #[test]
    fn example4_case_a_param_transient_borrowed_caller() {
        let program = LifetimeProgram::example4_case_a_param_n_transient();
        let m = facts_for(&program);
        let f = m.get(&BindingId(0)).expect("n");
        assert_eq!(f.ownership, Ownership::Borrowed);
        assert_eq!(f.lifetime, LifetimeScope::Caller);
        assert_eq!(f.growable, Growability::NotApplicable);
    }

    /// Test plan item 3 — Example 4 Case B.
    #[test]
    fn example4_case_b_param_stored_owned() {
        let program = LifetimeProgram::example4_case_b_param_n_stored();
        let m = facts_for(&program);
        let f = m.get(&BindingId(0)).expect("n");
        assert_eq!(f.ownership, Ownership::Owned);
        assert_eq!(f.lifetime, LifetimeScope::Caller);
    }

    /// Test plan item 4 — function return is always `Owned` (brief).
    #[test]
    fn function_return_binding_is_owned() {
        let program = LifetimeProgram::example_function_return_owned();
        let m = facts_for(&program);
        let f = m.get(&BindingId(0)).expect("ret");
        assert_eq!(f.ownership, Ownership::Owned);
        assert_eq!(f.lifetime, LifetimeScope::Self_);
        assert_eq!(f.growable, Growability::NotApplicable);
    }

    /// Test plan item 5 — contradictory use sites fail closed.
    #[test]
    fn contradictory_use_emits_diagnostic() {
        let program = LifetimeProgram::contradictory_borrow_and_escape();
        let err =
            analyze_lifetime_program(&program, &LanguageSpecAxes::example_rust_string_family())
                .expect_err("contradiction");
        match err {
            EmissionDiagnostic::ContradictoryUse { binding, sites } => {
                assert_eq!(binding, "x");
                assert_eq!(sites.len(), 2);
            }
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }

    /// Test plan item 6 — under-refined growability axis.
    #[test]
    fn under_refined_growability_when_indeterminate() {
        let program = LifetimeProgram::underrefined_growability_indeterminate();
        let err =
            analyze_lifetime_program(&program, &LanguageSpecAxes::example_rust_string_family())
                .expect_err("under refined");
        match err {
            EmissionDiagnostic::UnderRefined { axis } => assert_eq!(axis, "growability"),
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }

    /// Test plan item 7 — public entrypoint input domain is `(Dag, LanguageSpecAxes)` only.
    #[test]
    fn analyzer_public_entrypoint_is_dag_and_axes_only() {
        let dag = Dag::new();
        let axes = LanguageSpecAxes::example_rust_string_family();
        let report = analyze_lifetime_facts(&dag, &axes).expect("bootstrap dag ok");
        assert!(report.is_empty());
    }

    /// Bootstrap `Dag::new()` extraction stays empty until lowering surfaces R2 bind/use graphs.
    #[test]
    fn extract_bootstrap_dag_yields_empty_lifetime_program() {
        let dag = Dag::new();
        let program = extract_lifetime_program(&dag).expect("extract ok");
        assert!(program.bindings.is_empty());
        assert!(program.r3_markers.is_empty());
    }

    /// User- or test-range `data` / `fn` declarations must not map to `Ok(empty)` (C-8).
    #[test]
    fn extract_fail_closed_for_user_range_module_with_data_or_fn() {
        let source =
            include_str!("../../compiler/tests/fixtures/r1_mock_backed_invariant_gate.dag");
        let file = "src/v3/compiler/tests/fixtures/r1_mock_backed_invariant_gate.dag";
        let dag = v3_compiler::compile_to_dag(source, file).expect("fixture compile");
        let err = extract_lifetime_program(&dag).expect_err("extraction must fail closed");
        match err {
            EmissionDiagnostic::LifetimeProgramExtractionPending { detail } => {
                assert!(
                    detail.contains("mock_service_status")
                        || detail.contains("r1_mock_backed_invariant_gate"),
                    "unexpected detail: {detail}"
                );
            }
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }

    /// `span.file` is caller-controlled; authority-looking paths must not bypass C-8.
    #[test]
    fn extract_fail_closed_when_user_code_uses_authority_looking_span_path() {
        let source = "module spoof\n\nfn f() -> Int = 0\n";
        let file = "dsl/std/user_spoof_under_extractor_test.dag";
        let dag = v3_compiler::compile_to_dag(source, file).expect("compile");
        let err = extract_lifetime_program(&dag).expect_err("must not Ok(empty) on spoof path");
        match err {
            EmissionDiagnostic::LifetimeProgramExtractionPending { detail } => {
                assert!(detail.contains('f'), "unexpected detail: {detail}");
            }
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }

    #[test]
    fn function_param_indeterminate_growability_fails_closed_even_when_borrowed() {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef::r2_string_binding(
                "n",
                BindingRole::FunctionParameter {
                    function: "greet".to_string(),
                },
                vec![UseSite {
                    kind: UseKind::IndeterminateGrowability,
                    site_label: "opaque.call".to_string(),
                }],
            ),
        );
        let program = LifetimeProgram {
            bindings,
            r3_markers: vec![],
        };
        let err =
            analyze_lifetime_program(&program, &LanguageSpecAxes::example_rust_string_family())
                .expect_err("growability under-refined");
        match err {
            EmissionDiagnostic::UnderRefined { axis } => assert_eq!(axis, "growability"),
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }

    /// Opaque growability is not a transient proof: without `Transient`, do not
    /// conclude `Borrowed` when the growability axis is not load-bearing (no
    /// growability diagnostic to catch the gap).
    #[test]
    fn function_param_indeterminate_without_transient_fails_ownership_when_growability_optional() {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef::r2_string_binding(
                "n",
                BindingRole::FunctionParameter {
                    function: "greet".to_string(),
                },
                vec![UseSite {
                    kind: UseKind::IndeterminateGrowability,
                    site_label: "opaque.call".to_string(),
                }],
            ),
        );
        let program = LifetimeProgram {
            bindings,
            r3_markers: vec![],
        };
        let err = analyze_lifetime_program(
            &program,
            &LanguageSpecAxes::string_family_growability_not_load_bearing(),
        )
        .expect_err("ownership under-refined");
        match err {
            EmissionDiagnostic::UnderRefined { axis } => assert_eq!(axis, "ownership"),
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }

    #[test]
    fn function_param_transient_plus_indeterminate_ok_when_growability_optional() {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef::r2_string_binding(
                "n",
                BindingRole::FunctionParameter {
                    function: "greet".to_string(),
                },
                vec![
                    UseSite {
                        kind: UseKind::Transient,
                        site_label: "greet.body.read".to_string(),
                    },
                    UseSite {
                        kind: UseKind::IndeterminateGrowability,
                        site_label: "opaque.call".to_string(),
                    },
                ],
            ),
        );
        let program = LifetimeProgram {
            bindings,
            r3_markers: vec![],
        };
        let f = analyze_lifetime_program(
            &program,
            &LanguageSpecAxes::string_family_growability_not_load_bearing(),
        )
        .expect("analysis ok")
        .remove(&BindingId(0))
        .expect("n");
        assert_eq!(f.ownership, Ownership::Borrowed);
        assert_eq!(f.growable, Growability::NotApplicable);
    }

    /// Test plan item 8 — R3 constructs rejected.
    #[test]
    fn r3_construct_out_of_scope() {
        let program = LifetimeProgram::with_r3_construct(R3Construct::Closure);
        let err =
            analyze_lifetime_program(&program, &LanguageSpecAxes::example_rust_string_family())
                .expect_err("r3");
        match err {
            EmissionDiagnostic::OutOfR2Scope { construct } => {
                assert_eq!(construct, "closure");
            }
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }

    /// Growth mutation forces `Growability::Yes` on top-level string-like binding.
    #[test]
    fn growth_mutation_forces_yes() {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef::r2_string_binding(
                "buf",
                BindingRole::TopLevelData,
                vec![UseSite {
                    kind: UseKind::GrowthMutation,
                    site_label: "buf.push".to_string(),
                }],
            ),
        );
        let program = LifetimeProgram {
            bindings,
            r3_markers: vec![],
        };
        let f = facts_for(&program)
            .get(&BindingId(0))
            .copied()
            .expect("buf");
        assert_eq!(f.growable, Growability::Yes);
        assert_eq!(f.ownership, Ownership::Owned);
    }

    #[test]
    fn unclassified_type_family_fails_closed_on_encoding_axis() {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef {
                name: "opaque".to_string(),
                role: BindingRole::TopLevelData,
                uses: vec![],
                type_family: ProgramTypeFamily::Unclassified,
            },
        );
        let program = LifetimeProgram {
            bindings,
            r3_markers: vec![],
        };
        let err =
            analyze_lifetime_program(&program, &LanguageSpecAxes::example_rust_string_family())
                .expect_err("encoding under-refined");
        match err {
            EmissionDiagnostic::UnderRefined { axis } => assert_eq!(axis, "encoding"),
            other => panic!("unexpected diagnostic: {other:?}"),
        }
    }
}
