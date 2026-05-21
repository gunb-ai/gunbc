//! **Layer:** integration
//!
//! Structural ratchet for `src/v4/lens/subsumption.dag`. The subject is the
//! v4 substrate carrier shape and first data row, so this stays parse-level
//! until v4 bundle loading can execute the row as native `.dag` TestClaim data.

use std::collections::BTreeSet;

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceRecordField, SurfaceType, VariantPayload,
};
use v3_compiler::tokenize_for_test;

const SUBSUMPTION_DAG: &str = include_str!("../../../../v4/lens/subsumption.dag");
const SUBSUMPTION_PATH: &str = "src/v4/lens/subsumption.dag";

#[test]
fn dissolution_subsumption_carrier_shape_locked() {
    let module = parse_module(SUBSUMPTION_DAG, SUBSUMPTION_PATH);

    assert_record_fields(&module.items, "DiffId", &[("id", "Symbol")]);
    assert_record_fields(&module.items, "TestClaimId", &[("id", "Symbol")]);
    assert_record_fields(&module.items, "ProducerStageId", &[("id", "Symbol")]);
    assert_record_fields(
        &module.items,
        "SubsumedFixes",
        &[("head", "DiffId"), ("tail", "List<DiffId>")],
    );
    assert_record_fields(
        &module.items,
        "ProducerStageDerivationPath",
        &[("head", "ProducerStageId"), ("tail", "List<ProducerStageId>")],
    );
    assert_sum_variants(
        &module.items,
        "SubsumptionVerification",
        &[
            ("MechanicalReverification", &[("test_claim", "TestClaimId")]),
            (
                "ProducerStageDerivation",
                &[("derivation_path", "ProducerStageDerivationPath")],
            ),
        ],
    );
    assert_record_fields(
        &module.items,
        "DissolutionSubsumption",
        &[
            ("root_fix", "DiffId"),
            ("subsumed_fixes", "SubsumedFixes"),
            ("verification", "SubsumptionVerification"),
        ],
    );
}

#[test]
fn dissolution_subsumption_first_row_is_producer_stage_derived() {
    let module = parse_module(SUBSUMPTION_DAG, SUBSUMPTION_PATH);
    let row = data_body(&module.items, "concept_home_rewrite_subsumption");

    let root_fix = field_value(row, "root_fix");
    assert_eq!(
        diff_id_symbol(root_fix),
        "concept_home_rewrite_root_fix",
        "{SUBSUMPTION_PATH}: first subsumption row root fix drifted"
    );

    let subsumed = field_value(row, "subsumed_fixes");
    let subsumed_fields = assert_variant_record(subsumed, "SubsumedFixes");
    let head = field_value(subsumed_fields, "head");
    let tail = field_value(subsumed_fields, "tail");
    let SurfaceExpr::List { elements, .. } = tail else {
        panic!("{SUBSUMPTION_PATH}: SubsumedFixes.tail must be a finite DiffId list");
    };
    let observed_leaf_ids = std::iter::once(diff_id_symbol(head))
        .chain(elements.iter().map(diff_id_symbol))
        .collect::<BTreeSet<_>>();
    let expected_leaf_ids = ["duplicate_home_leaf_fix", "wrong_home_alias_leaf_fix"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed_leaf_ids, expected_leaf_ids,
        "{SUBSUMPTION_PATH}: first subsumption row leaf fixes drifted"
    );

    let verification = field_value(row, "verification");
    let fields = assert_variant_record(verification, "ProducerStageDerivation");
    let path = field_value(fields, "derivation_path");
    let path_fields = assert_variant_record(path, "ProducerStageDerivationPath");
    let head = field_value(path_fields, "head");
    assert_eq!(
        producer_stage_id_symbol(head),
        "concept_home_producer_stage",
        "{SUBSUMPTION_PATH}: first subsumption row producer stage drifted"
    );
    let tail = field_value(path_fields, "tail");
    let SurfaceExpr::List { elements, .. } = tail else {
        panic!("{SUBSUMPTION_PATH}: ProducerStageDerivationPath.tail must be a list");
    };
    assert_eq!(
        elements.len(),
        0,
        "{SUBSUMPTION_PATH}: first subsumption row should name exactly one additional producer stage"
    );
}

fn assert_record_fields(items: &[SurfaceItem], name: &str, expected: &[(&str, &str)]) {
    let fields = items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(fields),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{SUBSUMPTION_PATH}: missing record type {name}"));
    let observed = fields
        .iter()
        .map(|field| (field.name.as_str(), type_expr_text(&field.ty)))
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "{SUBSUMPTION_PATH}: {name} fields drifted"
    );
}

fn assert_sum_variants(items: &[SurfaceItem], name: &str, expected: &[(&str, &[(&str, &str)])]) {
    let variants = items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => Some(variants),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{SUBSUMPTION_PATH}: missing sum type {name}"));
    let observed = variants
        .iter()
        .map(|variant| {
            let VariantPayload::Record(variant_fields) = &variant.payload else {
                panic!(
                    "{SUBSUMPTION_PATH}: {}.{} must use record payload fields",
                    name, variant.name
                );
            };
            let fields = surface_fields(variant_fields);
            (variant.name.as_str(), fields)
        })
        .collect::<BTreeSet<_>>();
    let expected = expected
        .iter()
        .map(|(variant, fields)| (*variant, fields.iter().copied().collect::<BTreeSet<_>>()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "{SUBSUMPTION_PATH}: {name} variants drifted"
    );
}

fn data_body<'a>(items: &'a [SurfaceItem], name: &str) -> &'a [SurfaceRecordField] {
    let body = items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body,
                ..
            } if item_name == name => body.as_ref(),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{SUBSUMPTION_PATH}: missing data row {name}"));
    let SurfaceExpr::Record { fields, .. } = body else {
        panic!("{SUBSUMPTION_PATH}: {name} must use a record initializer");
    };
    fields
}

fn surface_fields(fields: &[SurfaceField]) -> BTreeSet<(&str, &'static str)> {
    fields
        .iter()
        .map(|field| (field.name.as_str(), type_expr_text(&field.ty)))
        .collect()
}

fn field_value<'a>(fields: &'a [SurfaceRecordField], name: &str) -> &'a SurfaceExpr {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
        .unwrap_or_else(|| panic!("{SUBSUMPTION_PATH}: missing field {name}"))
}

fn assert_variant_record<'a>(expr: &'a SurfaceExpr, target: &str) -> &'a [SurfaceRecordField] {
    let SurfaceExpr::VariantRecord {
        target: observed,
        fields,
        ..
    } = expr
    else {
        panic!("{SUBSUMPTION_PATH}: expected {target} constructor, got {expr:?}");
    };
    assert_eq!(
        observed, target,
        "{SUBSUMPTION_PATH}: expected {target} constructor"
    );
    fields
}

fn diff_id_symbol(expr: &SurfaceExpr) -> &str {
    let fields = assert_variant_record(expr, "DiffId");
    let id = field_value(fields, "id");
    let SurfaceExpr::Var { name, .. } = id else {
        panic!("{SUBSUMPTION_PATH}: DiffId.id must name a symbolic fix handle, got {id:?}");
    };
    name
}

fn type_expr_text(expr: &SurfaceType) -> &'static str {
    match expr {
        SurfaceType::Named { name, .. } => match name.as_str() {
            "Symbol" => "Symbol",
            "DiffId" => "DiffId",
            "TestClaimId" => "TestClaimId",
            "ProducerStageId" => "ProducerStageId",
            "SubsumedFixes" => "SubsumedFixes",
            "ProducerStageDerivationPath" => "ProducerStageDerivationPath",
            "SubsumptionVerification" => "SubsumptionVerification",
            other => panic!("{SUBSUMPTION_PATH}: unexpected named type {other}"),
        },
        SurfaceType::Parameterized { name, args, .. } if name == "List" => {
            assert_eq!(
                args.len(),
                1,
                "{SUBSUMPTION_PATH}: List should have one type argument"
            );
            match &args[0] {
                v3_compiler::parse_surface::TypeAngleArg::TypeExpr { ty } => {
                    let SurfaceType::Named { name, .. } = ty.as_ref() else {
                        panic!("{SUBSUMPTION_PATH}: unexpected List argument {ty:?}");
                    };
                    match name.as_str() {
                        "DiffId" => "List<DiffId>",
                        "ProducerStageId" => "List<ProducerStageId>",
                        other => panic!("{SUBSUMPTION_PATH}: unexpected List argument {other}"),
                    }
                }
                other => panic!("{SUBSUMPTION_PATH}: unexpected List argument {other:?}"),
            }
        }
        other => panic!("{SUBSUMPTION_PATH}: unexpected type expression {other:?}"),
    }
}

fn producer_stage_id_symbol(expr: &SurfaceExpr) -> &str {
    let fields = assert_variant_record(expr, "ProducerStageId");
    let id = field_value(fields, "id");
    let SurfaceExpr::Var { name, .. } = id else {
        panic!("{SUBSUMPTION_PATH}: ProducerStageId.id must name a symbolic stage handle, got {id:?}");
    };
    name
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}
