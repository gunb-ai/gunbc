//! **Layer:** integration
//!
//! Ratchet the PREFIX dissolution lens acceptance batch into the v4
//! coverage-lens substrate. The check is intentionally structural and stays
//! at parse level because v4 single-file compile smokes do not load sibling
//! lens/std imports as a bundle yet.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

use std::collections::BTreeSet;

const COVERAGE_DAG: &str = include_str!("../../../../v4/lens/coverage.dag");
const COVERAGE_PATH: &str = "src/v4/lens/coverage.dag";

const EXPECTED_ACCEPTANCE_ROWS: &[&str] = &[
    "dissolution_l1_1_discriminant_predicate",
    "dissolution_l1_2_degenerate_type",
    "dissolution_l1_3_hollow_type",
    "dissolution_l1_4_carrier_clone",
    "dissolution_l1_5_catamorphism",
    "dissolution_l1_6_emit_template",
    "dissolution_l1_7_off_substrate_fact",
    "dissolution_l1_8_wrong_home",
    "dissolution_l1_9_vacuous_arm",
    "dissolution_l1_10_string_escape_hatch",
    "dissolution_l1_11_plausible_fallback",
    "dissolution_l1_12_parallel_authority",
];

#[test]
fn v4_lens_coverage_names_all_l1_acceptance_rows() {
    let module = parse_module(COVERAGE_DAG, COVERAGE_PATH);
    let observed = module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Data { name, .. } if name.starts_with("dissolution_l1_") => {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let expected = EXPECTED_ACCEPTANCE_ROWS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        observed, expected,
        "{COVERAGE_PATH}: PREFIX dissolution acceptance rows must exactly match L1.1-L1.12"
    );
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}
