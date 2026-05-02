//! T-Ground-Tests — L4 routing-correctness verification.
//!
//! See `docs/briefs/t-ground-tests.md` (lane 10 / `routing_correctness_l4_verified` gate).
//! This crate generalizes the pilot Stratum A pattern (`src/v3/grounding_pilot/src/lib.rs:408-460`)
//! onto Phase 1 `MethodTemplateContract` row authorities (`#1195` / `#1196` / `#1210`).
//!
//! **SG-0:** no `src/v3/compiler/` edits — consumers only.
//!
//! Stratum A routing-parity tests live in the `stratum_a` module’s `stratum_a_tests` section so
//! they exercise the module’s fail-closed API without reaching private helpers from here. Stratum B
//! currently exposes a readiness/checklist scaffold only; it does not assert production fold
//! outputs until the declared projection substrate gates named in the audit stack land.

pub mod diagnostic;
mod stratum_a;
mod stratum_b;

pub use diagnostic::GroundingTestsDiagnostic;
pub use stratum_a::{
    stratum_a_list_digest, verify_stratum_a_lockstep_all_targets, RowFingerprint,
    EXPECTED_STRATUM_A_ROW_COUNTS,
};
pub use stratum_b::{
    stratum_b_missing_prerequisites, stratum_b_readiness, StratumBPrerequisite,
    StratumBPrerequisiteState, StratumBReadinessEntry,
};

#[cfg(test)]
mod emission_diagnostic_lockstep;

#[cfg(test)]
mod integer_diagnostic_order;

#[cfg(test)]
mod string_axes_lockstep;

#[cfg(test)]
mod tests {
    use v3_compiler::dag::Dag;
    use v3_grounding_lifetime::{
        analyze_lifetime_facts, axes::LanguageSpecAxes, extract_lifetime_program,
    };

    use super::GroundingTestsDiagnostic;

    /// Sibling dep smoke: bootstrap `Dag::new()` stays empty for lifetime extraction.
    #[test]
    fn grounding_lifetime_sibling_wires_through_bootstrap() {
        let dag = Dag::new();
        let axes = LanguageSpecAxes::example_rust_string_family();
        let report = analyze_lifetime_facts(&dag, &axes).expect("bootstrap ok");
        assert!(report.is_empty());
        let program = extract_lifetime_program(&dag).expect("extract empty");
        assert!(program.bindings.is_empty());
    }

    #[test]
    fn stratum_a_diagnostic_display_is_non_empty() {
        let e = GroundingTestsDiagnostic::StratumARowCountMismatch {
            list_name: "x".to_string(),
            expected: 1,
            actual: 2,
        };
        assert!(!e.to_string().is_empty());

        let e2 = GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "test",
            detail: "detail".to_string(),
        };
        assert!(!e2.to_string().is_empty());

        let e3 = GroundingTestsDiagnostic::StratumALockstepListDigestMismatch {
            list_name: "rust_method_template_contracts".to_string(),
            detail: "detail".to_string(),
        };
        assert!(!e3.to_string().is_empty());

        let e4 = GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
            prerequisite: "program-bound lowering",
            detail: "detail".to_string(),
        };
        assert!(!e4.to_string().is_empty());
    }
}
