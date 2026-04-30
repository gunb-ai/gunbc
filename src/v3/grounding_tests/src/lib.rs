//! T-Ground-Tests — L4 routing-correctness verification (Stratum A scaffold).
//!
//! See `docs/briefs/t-ground-tests.md` (lane 10 / `routing_correctness_l4_verified` gate).
//! This crate generalizes the pilot Stratum A pattern (`src/v3/grounding_pilot/src/lib.rs:408-460`)
//! onto Phase 1 `MethodTemplateContract` row authorities (`#1195` / `#1196` / `#1210`).
//!
//! **SG-0:** no `src/v3/compiler/` edits — consumers only.

pub mod diagnostic;
mod stratum_a;

pub use diagnostic::GroundingTestsDiagnostic;
pub use stratum_a::{
    stratum_a_list_digest, verify_stratum_a_lockstep_all_targets, RowFingerprint,
    EXPECTED_STRATUM_A_ROW_COUNTS,
};

#[cfg(test)]
mod tests {
    use v3_compiler::dag::Dag;
    use v3_compiler::generated_full_bootstrap_dag;
    use v3_grounding_lifetime::{
        analyze_lifetime_facts, axes::LanguageSpecAxes, extract_lifetime_program,
    };

    use super::stratum_a::{
        fingerprints_for_list, stratum_a_list_digest, verify_stratum_a_lockstep_all_targets,
        RUST_LIST,
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
    fn stratum_a_routing_parity_lockstep_bootstrap_vs_standalone() {
        verify_stratum_a_lockstep_all_targets().unwrap_or_else(|e| panic!("{e}"));
    }

    #[test]
    fn stratum_a_row_counts_match_director_phase1() {
        let dag = generated_full_bootstrap_dag();
        for &(name, expected) in super::EXPECTED_STRATUM_A_ROW_COUNTS {
            let n = dag
                .declaration_by_name(name)
                .and_then(|d| d.value_body.as_ref())
                .and_then(|b| match b {
                    v3_compiler::dag::ValueBody::List(rows) => Some(rows.len()),
                    _ => None,
                })
                .unwrap_or(0);
            assert_eq!(
                n, expected,
                "`{name}` row count drift — update EXPECTED_STRATUM_A_ROW_COUNTS or substrate"
            );
        }
    }

    /// Determinism: multiset digest is identical whether rows are walked forward or backward
    /// before insertion into the canonical `BTreeMap` keying (`t-ground-tests.md` item 3).
    #[test]
    fn stratum_a_determinism_independent_of_list_walk_order() {
        let dag = generated_full_bootstrap_dag();
        let rows = dag
            .declaration_by_name(RUST_LIST)
            .expect(RUST_LIST)
            .value_body
            .as_ref()
            .and_then(|b| match b {
                v3_compiler::dag::ValueBody::List(r) => Some(r.as_slice()),
                _ => None,
            })
            .expect("list body");

        let mut forward = std::collections::BTreeMap::new();
        for (idx, row) in rows.iter().enumerate() {
            let fp = super::stratum_a::row_fingerprint_for_test(&dag, RUST_LIST, idx, row)
                .unwrap_or_else(|e| panic!("{e}"));
            forward.insert(fp.method_name.clone(), fp);
        }
        let mut backward = std::collections::BTreeMap::new();
        for (idx, row) in rows.iter().enumerate().rev() {
            let fp = super::stratum_a::row_fingerprint_for_test(&dag, RUST_LIST, idx, row)
                .unwrap_or_else(|e| panic!("{e}"));
            backward.insert(fp.method_name.clone(), fp);
        }
        assert_eq!(forward, backward);

        let d1 = stratum_a_list_digest(&dag, RUST_LIST).unwrap();
        let d2 = stratum_a_list_digest(&dag, RUST_LIST).unwrap();
        assert_eq!(d1, d2);

        let via_helper = fingerprints_for_list(&dag, RUST_LIST).unwrap();
        assert_eq!(
            super::stratum_a::list_digest_from_fingerprints_for_test(&via_helper),
            d1
        );
    }

    #[test]
    fn stratum_a_diagnostic_display_is_non_empty() {
        let e = GroundingTestsDiagnostic::StratumARowCountMismatch {
            list_name: "x".to_string(),
            expected: 1,
            actual: 2,
        };
        assert!(!e.to_string().is_empty());
    }
}
