//! **Layer:** integration
//!
//! Ratchet the PREFIX dissolution lens acceptance batch into the v4
//! coverage-lens substrate. The check is intentionally structural and stays
//! at parse level because v4 single-file compile smokes do not load sibling
//! lens/std imports as a bundle yet.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem, SurfaceRecordField};
use v3_compiler::tokenize_for_test;

use std::collections::BTreeSet;

const COVERAGE_DAG: &str = include_str!("../../../../v4/lens/coverage.dag");
const COVERAGE_PATH: &str = "src/v4/lens/coverage.dag";

const EXPECTED_ACCEPTANCE_ROWS: &[(&str, &str)] = &[
    (
        "dissolution_l1_1_discriminant_predicate",
        "L1_1DiscriminantPredicate",
    ),
    ("dissolution_l1_2_degenerate_type", "L1_2DegenerateType"),
    ("dissolution_l1_3_hollow_type", "L1_3HollowType"),
    ("dissolution_l1_4_carrier_clone", "L1_4CarrierClone"),
    ("dissolution_l1_5_catamorphism", "L1_5Catamorphism"),
    ("dissolution_l1_10_a_template_hole", "L1_10ATemplateHole"),
    (
        "dissolution_l1_7_off_substrate_fact",
        "L1_7OffSubstrateFact",
    ),
    ("dissolution_l1_8_wrong_home", "L1_8WrongHome"),
    ("dissolution_l1_9_vacuous_arm", "L1_9VacuousArm"),
    (
        "dissolution_l1_10_b_canonical_carrier",
        "L1_10BCanonicalCarrier",
    ),
    (
        "dissolution_l1_11_plausible_fallback",
        "L1_11PlausibleFallback",
    ),
    (
        "dissolution_l1_12_parallel_authority",
        "L1_12ParallelAuthority",
    ),
];

const RETIRED_ACCEPTANCE_ROWS: &[(&str, &str)] = &[
    ("dissolution_l1_6_emit_template", "L1_6EmitTemplate"),
    (
        "dissolution_l1_10_string_escape_hatch",
        "L1_10StringEscapeHatch",
    ),
];

#[test]
fn v4_lens_coverage_names_all_l1_acceptance_keys() {
    let module = parse_module(COVERAGE_DAG, COVERAGE_PATH);
    let observed_variants = module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum { name, variants, .. } if name == "DissolutionLensKey" => Some(
                variants
                    .iter()
                    .map(|variant| variant.name.as_str())
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{COVERAGE_PATH}: missing DissolutionLensKey coproduct"));
    let expected_variants = EXPECTED_ACCEPTANCE_ROWS
        .iter()
        .map(|(_, variant)| *variant)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        observed_variants, expected_variants,
        "{COVERAGE_PATH}: DissolutionLensKey variants must exactly match canonical Layer-1 keys"
    );

    let retired_variants = RETIRED_ACCEPTANCE_ROWS
        .iter()
        .map(|(_, variant)| *variant)
        .collect::<BTreeSet<_>>();
    assert!(
        observed_variants.is_disjoint(&retired_variants),
        "{COVERAGE_PATH}: retired L1.6 / monolithic L1.10 variants must not reappear"
    );

    let observed = module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Data { name, body, .. } if name.starts_with("dissolution_l1_") => {
                Some((name.as_str(), acceptance_lens_variant(name, body.as_ref())))
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
        "{COVERAGE_PATH}: PREFIX dissolution acceptance rows must exactly map canonical Layer-1 keys"
    );

    let retired = RETIRED_ACCEPTANCE_ROWS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert!(
        observed.is_disjoint(&retired),
        "{COVERAGE_PATH}: retired L1.6 / monolithic L1.10 rows must not reappear"
    );
}

fn acceptance_lens_variant<'a>(row_name: &str, body: Option<&'a SurfaceExpr>) -> &'a str {
    let Some(SurfaceExpr::Record { fields, .. }) = body else {
        panic!("{COVERAGE_PATH}: {row_name} must have a record initializer");
    };
    let concern = field_value(fields, "concern")
        .unwrap_or_else(|| panic!("{COVERAGE_PATH}: {row_name} must initialize concern"));
    let SurfaceExpr::VariantRecord { target, fields, .. } = concern else {
        panic!("{COVERAGE_PATH}: {row_name}.concern must construct DissolutionLensAcceptanceKey");
    };
    assert_eq!(
        target, "DissolutionLensAcceptanceKey",
        "{COVERAGE_PATH}: {row_name}.concern must construct DissolutionLensAcceptanceKey"
    );
    let lens = field_value(fields, "lens")
        .unwrap_or_else(|| panic!("{COVERAGE_PATH}: {row_name}.concern must initialize lens"));
    let SurfaceExpr::Var { name, .. } = lens else {
        panic!("{COVERAGE_PATH}: {row_name}.concern.lens must be a DissolutionLensKey variant");
    };
    name.as_str()
}

fn field_value<'a>(fields: &'a [SurfaceRecordField], name: &str) -> Option<&'a SurfaceExpr> {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}
