//! **Layer:** integration
//!
//! Ratchet coverage-defect acceptance keys into the v4 coverage-lens
//! substrate. The check is intentionally structural and stays at parse level
//! because v4 single-file checks do not load sibling lens/std imports as a
//! bundle yet.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem, SurfaceRecordField};
use v3_compiler::tokenize_for_test;

use std::collections::BTreeSet;

const COVERAGE_DAG: &str = include_str!("../../../../v4/lens/coverage.dag");
const COVERAGE_PATH: &str = "src/v4/lens/coverage.dag";

const EXPECTED_ACCEPTANCE_ROWS: &[(&str, &str)] = &[
    (
        "coverage_defect_discriminant_predicate",
        "DiscriminantPredicate",
    ),
    ("coverage_defect_degenerate_type", "DegenerateType"),
    ("coverage_defect_hollow_type", "HollowType"),
    ("coverage_defect_carrier_clone", "CarrierClone"),
    ("coverage_defect_catamorphism", "Catamorphism"),
    ("coverage_defect_template_hole", "TemplateHole"),
    ("coverage_defect_off_substrate_fact", "OffSubstrateFact"),
    ("coverage_defect_wrong_home", "WrongHome"),
    ("coverage_defect_vacuous_arm", "VacuousArm"),
    ("coverage_defect_canonical_carrier", "CanonicalCarrier"),
    ("coverage_defect_plausible_fallback", "PlausibleFallback"),
    ("coverage_defect_parallel_authority", "ParallelAuthority"),
    ("coverage_defect_skeleton_collapse", "SkeletonCollapse"),
];

const RETIRED_ACCEPTANCE_ROWS: &[(&str, &str)] = &[
    ("coverage_defect_emit_template", "EmitTemplate"),
    ("coverage_defect_string_escape_hatch", "StringEscapeHatch"),
];

#[test]
fn coverage_defect_acceptance_keys_match_declared_rows() {
    let module = parse_module(COVERAGE_DAG, COVERAGE_PATH);
    let observed_variants = module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum { name, variants, .. } if name == "CoverageDefectKey" => Some(
                variants
                    .iter()
                    .map(|variant| variant.name.as_str())
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{COVERAGE_PATH}: missing CoverageDefectKey coproduct"));
    let expected_variants = EXPECTED_ACCEPTANCE_ROWS
        .iter()
        .map(|(_, variant)| *variant)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        observed_variants, expected_variants,
        "{COVERAGE_PATH}: CoverageDefectKey variants must exactly match canonical coverage-defect keys"
    );

    let retired_variants = RETIRED_ACCEPTANCE_ROWS
        .iter()
        .map(|(_, variant)| *variant)
        .collect::<BTreeSet<_>>();
    assert!(
        observed_variants.is_disjoint(&retired_variants),
        "{COVERAGE_PATH}: retired emit-template / string-escape variants must not reappear"
    );

    let observed = module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Data { name, body, .. } if name.starts_with("coverage_defect_") => Some((
                name.as_str(),
                acceptance_defect_variant(name, body.as_ref()),
            )),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let expected = EXPECTED_ACCEPTANCE_ROWS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        observed, expected,
        "{COVERAGE_PATH}: coverage-defect acceptance rows must exactly map canonical keys"
    );

    let retired = RETIRED_ACCEPTANCE_ROWS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert!(
        observed.is_disjoint(&retired),
        "{COVERAGE_PATH}: retired emit-template / string-escape rows must not reappear"
    );
}

fn acceptance_defect_variant<'a>(row_name: &str, body: Option<&'a SurfaceExpr>) -> &'a str {
    let Some(SurfaceExpr::Record { fields, .. }) = body else {
        panic!("{COVERAGE_PATH}: {row_name} must have a record initializer");
    };
    let concern = field_value(fields, "concern")
        .unwrap_or_else(|| panic!("{COVERAGE_PATH}: {row_name} must initialize concern"));
    let SurfaceExpr::VariantRecord { target, fields, .. } = concern else {
        panic!("{COVERAGE_PATH}: {row_name}.concern must construct CoverageDefectAcceptanceKey");
    };
    assert_eq!(
        target, "CoverageDefectAcceptanceKey",
        "{COVERAGE_PATH}: {row_name}.concern must construct CoverageDefectAcceptanceKey"
    );
    let lens = field_value(fields, "lens")
        .unwrap_or_else(|| panic!("{COVERAGE_PATH}: {row_name}.concern must initialize lens"));
    let SurfaceExpr::Var { name, .. } = lens else {
        panic!("{COVERAGE_PATH}: {row_name}.concern.lens must be a CoverageDefectKey variant");
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
