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
    assert_no_item(&module.items, "SubsumedFixes");
    assert_no_item(&module.items, "ProducerStageDerivationPath");
    assert_sum_variants(
        &module.items,
        "SubsumptionVerification",
        &[
            ("MechanicalReverification", &[("test_claim", "TestClaimId")]),
            (
                "ProducerStageDerivation",
                &[("derivation_path", "List<ProducerStageId>")],
            ),
        ],
    );
    assert_record_fields(
        &module.items,
        "DissolutionSubsumption",
        &[
            ("root_fix", "DiffId"),
            ("subsumed_fixes", "Set<DiffId>"),
            ("verification", "SubsumptionVerification"),
        ],
    );
}

#[test]
fn dissolution_subsumption_first_row_is_mechanical_reverification() {
    let module = parse_module(SUBSUMPTION_DAG, SUBSUMPTION_PATH);
    let row = data_body(&module.items, "affected_set_irt1_subsumption");

    let root_fix = field_value(row, "root_fix");
    assert_eq!(
        diff_id_symbol(root_fix),
        "affected_set_irt1_frontier_root_fix",
        "{SUBSUMPTION_PATH}: first subsumption row root fix drifted"
    );

    let subsumed = field_value(row, "subsumed_fixes");
    let SurfaceExpr::Var { name, .. } = subsumed else {
        panic!(
            "{SUBSUMPTION_PATH}: subsumed_fixes must reference the data-row Set<DiffId> binding, got {subsumed:?}"
        );
    };
    assert_eq!(
        name, "affected_set_irt1_subsumed_fixes",
        "{SUBSUMPTION_PATH}: first subsumption row subsumed_fixes drifted"
    );

    let verification = field_value(row, "verification");
    let fields = assert_variant_record(verification, "MechanicalReverification");
    let test_claim = field_value(fields, "test_claim");
    let test_claim_fields = assert_variant_record(test_claim, "TestClaimId");
    let id = field_value(test_claim_fields, "id");
    let SurfaceExpr::Var { name, .. } = id else {
        panic!("{SUBSUMPTION_PATH}: TestClaimId.id must name a symbolic claim handle, got {id:?}");
    };
    assert_eq!(
        name, "affected_set_irt1_leaf_claim_suite",
        "{SUBSUMPTION_PATH}: first subsumption row test claim drifted"
    );
}

#[test]
fn affected_set_irt1_subsumed_fixes_membership_predicate_enumerates_six_leaves() {
    let module = parse_module(SUBSUMPTION_DAG, SUBSUMPTION_PATH);
    let (body_start, body_end) = module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Fn { name, span, .. }
                if name == "affected_set_irt1_subsumed_fixes_value" =>
            {
                Some((span.byte_start, span.byte_end))
            }
            SurfaceItem::FnExternalBody {
                name, body_span, ..
            } if name == "affected_set_irt1_subsumed_fixes_value" => {
                Some((body_span.byte_start, body_span.byte_end))
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("{SUBSUMPTION_PATH}: missing fn affected_set_irt1_subsumed_fixes_value")
        });
    // Source-text scan (rather than a SurfaceExpr walk) because the predicate
    // is a fn body and parse-surface doesn't expose a stable `||`-chain
    // iterator yet. Each leaf is asserted in a `d.id ==` comparison position
    // (not just a bare mention in a stray comment) and the total count of
    // `d.id ==` clauses is pinned to 6, so an off-by-one regression — extra
    // comparison, missing comparison, or a leaf mentioned only in surrounding
    // prose — fails the ratchet.
    let body_source = &SUBSUMPTION_DAG[body_start as usize..body_end as usize];
    let expected = [
        "affected_set_boundary_receipt_leaf_fix",
        "affected_set_dimension_receipt_leaf_fix",
        "affected_set_propagation_receipt_leaf_fix",
        "affected_set_pending_escalation_leaf_fix",
        "affected_set_fail_closed_absorption_leaf_fix",
        "affected_set_empty_diff_leaf_fix",
    ];
    for leaf in expected {
        let needle = format!("d.id == {leaf}");
        assert!(
            body_source.contains(&needle),
            "{SUBSUMPTION_PATH}: subsumed-fixes membership predicate must compare d.id to {leaf}"
        );
    }
    let comparison_count = body_source.matches("d.id ==").count();
    assert_eq!(
        comparison_count,
        expected.len(),
        "{SUBSUMPTION_PATH}: subsumed-fixes membership predicate must contain exactly {} `d.id ==` clauses, found {}",
        expected.len(),
        comparison_count,
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

fn assert_no_item(items: &[SurfaceItem], name: &str) {
    let exists = items.iter().any(|item| match item {
        SurfaceItem::TypeRecord {
            name: item_name, ..
        }
        | SurfaceItem::TypeSum {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    });
    assert!(
        !exists,
        "{SUBSUMPTION_PATH}: {name} must not be declared after Set<DiffId> reshape"
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
    match body {
        SurfaceExpr::Record { fields, .. } => fields,
        SurfaceExpr::VariantRecord { fields, .. } => fields,
        other => panic!("{SUBSUMPTION_PATH}: {name} must use a record initializer, got {other:?}"),
    }
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
            "SubsumptionVerification" => "SubsumptionVerification",
            other => panic!("{SUBSUMPTION_PATH}: unexpected named type {other}"),
        },
        SurfaceType::Parameterized { name, args, .. } => {
            assert_eq!(
                args.len(),
                1,
                "{SUBSUMPTION_PATH}: {name} should have one type argument"
            );
            let v3_compiler::parse_surface::TypeAngleArg::TypeExpr { ty } = &args[0] else {
                panic!(
                    "{SUBSUMPTION_PATH}: unexpected {name} argument {:?}",
                    args[0]
                );
            };
            let SurfaceType::Named { name: inner, .. } = ty.as_ref() else {
                panic!("{SUBSUMPTION_PATH}: unexpected {name} argument {ty:?}");
            };
            match (name.as_str(), inner.as_str()) {
                ("List", "DiffId") => "List<DiffId>",
                ("List", "ProducerStageId") => "List<ProducerStageId>",
                ("Set", "DiffId") => "Set<DiffId>",
                (outer, arg) => {
                    panic!("{SUBSUMPTION_PATH}: unexpected parameterized type {outer}<{arg}>")
                }
            }
        }
        other => panic!("{SUBSUMPTION_PATH}: unexpected type expression {other:?}"),
    }
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}
