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
//!   `LifetimeProgram` (currently returns an empty program until lowering exposes the R2 graph).
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
    BindingDef, BindingId, BindingRole, LifetimeProgram, R3Construct, UseKind, UseSite,
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
            BindingDef {
                name: "buf".to_string(),
                role: BindingRole::TopLevelData,
                uses: vec![UseSite {
                    kind: UseKind::GrowthMutation,
                    site_label: "buf.push".to_string(),
                }],
            },
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
}
