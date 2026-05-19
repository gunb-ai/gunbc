//! **Layer:** integration
//!
//! Closeout ratchets for the v4 T-19/T-20 test/bootstrap-infra lane. These
//! checks stay at the parse-surface boundary: they prove the structural
//! authorities exist and remain joined without claiming T-22 execution.

use std::collections::{BTreeMap, BTreeSet};

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceModule, SurfaceRecordField, SurfaceType,
    SurfaceVariant, TypeAngleArg,
};
use v3_compiler::tokenize_for_test;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const TESTGEN_PATH: &str = "src/v4/lens/testgen.dag";
const BOOTSTRAP_DAG: &str = include_str!("../../../../v4/workflow/bootstrap.dag");
const BOOTSTRAP_PATH: &str = "src/v4/workflow/bootstrap.dag";
const CONNECTIVE_ANCHORS_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/connective_anchors.dag");
const CONNECTIVE_ANCHORS_PATH: &str = "src/v4/test/claim/manual/connective_anchors.dag";
const NAT_LAW_ANCHORS_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/nat_law_anchors.dag");
const NAT_LAW_ANCHORS_PATH: &str = "src/v4/test/claim/manual/nat_law_anchors.dag";
const T19_MANIFEST_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/t19_manual_anchor_manifest.dag");
const T19_MANIFEST_PATH: &str = "src/v4/test/claim/manual/t19_manual_anchor_manifest.dag";

#[test]
fn t19_testgen_concept_surface_stays_closed_and_classified() {
    let module = parse_module(TESTGEN_DAG, TESTGEN_PATH);

    assert_eq!(
        variant_name_set(type_sum(&module, "TestgenConcept")),
        expected_name_set(&[
            "TypeConstruction",
            "AlgebraLaw",
            "DiagnosticExhaustiveness",
            "LensApplicability",
            "BidirectionalRoundtrip",
        ]),
        "T-19 scheduling arms must stay the closed five-way set from TASKS.md"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "Generator")),
        expected_field_type_map(&[
            ("classification", "TestClassification"),
            ("kind", "AssertKind"),
            ("t19_anchor", "T19ManualAnchorKey"),
            ("slot", "C"),
        ]),
        "Generator<C> must carry the claim kind, anchor, classification, and parameterized slot"
    );
}

#[test]
fn t19_manual_manifest_matches_claim_anchor_discriminants() {
    let connective = parse_module(CONNECTIVE_ANCHORS_DAG, CONNECTIVE_ANCHORS_PATH);
    let nat_laws = parse_module(NAT_LAW_ANCHORS_DAG, NAT_LAW_ANCHORS_PATH);
    let manifest = parse_module(T19_MANIFEST_DAG, T19_MANIFEST_PATH);

    let manifest_keys = manifest_anchor_values(&manifest);
    let claim_keys = claim_anchor_values(&[&connective, &nat_laws]);

    assert_eq!(
        manifest_keys.len(),
        12,
        "T-19 manifest is the twelve live anchors"
    );
    assert_eq!(
        claim_keys, manifest_keys,
        "manual TestClaim.t19_anchor values must join to the manifest by the same T19ManualAnchorKey discriminants"
    );
    assert!(
        !claim_keys.contains("T19ManualAnchorAbsent"),
        "the twelve live manual anchors must not route through the absent sentinel"
    );
}

#[test]
fn t20_bootstrap_plan_keeps_self_hosting_chain_as_data() {
    let module = parse_module(BOOTSTRAP_DAG, BOOTSTRAP_PATH);

    assert_eq!(
        record_field_type_map(type_record(&module, "BootstrapHashPin")),
        expected_field_type_map(&[("digest", "Hash"), ("pin", "Symbol")]),
        "BootstrapHashPin must pair digest facts with symbolic pins"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "CompileStage")),
        expected_field_type_map(&[
            ("consumes", "List<Symbol>"),
            ("produces", "Symbol"),
            ("produces_hash", "BootstrapHashPin"),
            ("compiled_by", "Symbol"),
        ]),
        "CompileStage must keep compiler-of-record and per-stage hash pins as structural fields"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "FixptStage1Stage2")),
        expected_field_type_map(&[
            ("left", "Symbol"),
            ("left_hash", "BootstrapHashPin"),
            ("right", "Symbol"),
            ("right_hash", "BootstrapHashPin"),
            ("pinned_hash", "BootstrapHashPin"),
            ("via", "Symbol"),
        ]),
        "FixptStage1Stage2 must carry paired stage hashes and the pinned fixed-point pin"
    );
    assert_eq!(
        record_field_type_map(type_record(&module, "BootstrapPlan")),
        expected_field_type_map(&[
            ("seed", "CompileStage"),
            ("self0", "CompileStage"),
            ("self1", "CompileStage"),
            ("fixpt", "FixptStage1Stage2"),
        ]),
        "BootstrapPlan must stay the seed/self0/self1/fixpt chain"
    );

    let plan = data_expr(&module, "bootstrap_plan");
    let call_args = match plan {
        SurfaceExpr::Call { target, args, .. } => {
            assert_eq!(target, "bootstrap_plan_well_formed");
            args
        }
        other => panic!("bootstrap_plan must call bootstrap_plan_well_formed, got {other:?}"),
    };
    assert_eq!(
        call_args.len(),
        1,
        "bootstrap_plan_well_formed takes one plan"
    );
    let bootstrap_plan_arg = match &call_args[0] {
        SurfaceExpr::Record { fields, .. } => record_field_expr(fields, "p"),
        other => other,
    };
    let bootstrap_plan_fields = match bootstrap_plan_arg {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "BootstrapPlan");
            fields
        }
        other => panic!("bootstrap_plan argument must be a BootstrapPlan record, got {other:?}"),
    };

    assert_compile_stage(
        record_field_expr(bootstrap_plan_fields, "seed"),
        &["v4_dag_source"],
        "v4_stage0_binary",
        "v4_stage0_hash_pin",
        "v2_pipeline",
    );
    assert_compile_stage(
        record_field_expr(bootstrap_plan_fields, "self0"),
        &["v4_dag_source", "v4_stage0_binary"],
        "v4_stage1_binary",
        "v4_stage1_hash_pin",
        "v4_stage0_binary",
    );
    assert_compile_stage(
        record_field_expr(bootstrap_plan_fields, "self1"),
        &["v4_dag_source", "v4_stage1_binary"],
        "v4_stage2_binary",
        "v4_stage2_hash_pin",
        "v4_stage1_binary",
    );
    assert_fixpt(
        record_field_expr(bootstrap_plan_fields, "fixpt"),
        "v4_stage1_binary",
        "v4_stage1_hash_pin",
        "v4_stage2_binary",
        "v4_stage2_hash_pin",
        "pinned_v4_fixed_point_hash_pin",
        "bit_identical_check",
    );
}

fn parse_module(source: &str, file: &str) -> SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn type_sum<'a>(module: &'a SurfaceModule, name: &str) -> &'a [SurfaceVariant] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => Some(variants.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type sum {name}"))
}

fn type_record<'a>(module: &'a SurfaceModule, name: &str) -> &'a [SurfaceField] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record {name}"))
}

fn data_expr<'a>(module: &'a SurfaceModule, name: &str) -> &'a SurfaceExpr {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body: Some(body),
                ..
            } if item_name == name => Some(body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data {name}"))
}

fn variant_name_set(variants: &[SurfaceVariant]) -> BTreeSet<&str> {
    variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect()
}

fn expected_name_set(names: &[&'static str]) -> BTreeSet<&'static str> {
    names.iter().copied().collect()
}

fn record_field_type_map(fields: &[SurfaceField]) -> BTreeMap<&str, String> {
    fields
        .iter()
        .map(|field| (field.name.as_str(), surface_type_name(&field.ty)))
        .collect()
}

fn expected_field_type_map(
    fields: &[(&'static str, &'static str)],
) -> BTreeMap<&'static str, String> {
    fields
        .iter()
        .map(|(name, ty)| (*name, (*ty).to_string()))
        .collect()
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    TypeAngleArg::TypeExpr { ty } => surface_type_name(ty),
                    TypeAngleArg::WidthNatLiteral { decimal, .. } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        SurfaceType::Optional { inner, .. } => format!("?{}", surface_type_name(inner)),
        SurfaceType::Arrow { .. } => "fn".to_string(),
    }
}

fn manifest_anchor_values(module: &SurfaceModule) -> BTreeSet<&str> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Data {
                ty: SurfaceType::Named { name: ty_name, .. },
                body: Some(SurfaceExpr::Var { name, .. }),
                ..
            } if ty_name == "T19ManualAnchorKey" => Some(name.as_str()),
            _ => None,
        })
        .collect()
}

fn claim_anchor_values<'a>(modules: &[&'a SurfaceModule]) -> BTreeSet<&'a str> {
    modules
        .iter()
        .flat_map(|module| module.items.iter())
        .filter_map(|item| match item {
            SurfaceItem::Data {
                ty: SurfaceType::Named { name: ty_name, .. },
                body: Some(SurfaceExpr::VariantRecord { fields, .. }),
                ..
            } if ty_name == "TestClaim" => match record_field_expr(fields, "t19_anchor") {
                SurfaceExpr::Var { name, .. } => Some(name.as_str()),
                other => {
                    panic!("TestClaim.t19_anchor must be a discriminant var, got {other:?}")
                }
            },
            _ => None,
        })
        .collect()
}

fn record_field_expr<'a>(fields: &'a [SurfaceRecordField], name: &str) -> &'a SurfaceExpr {
    fields
        .iter()
        .find_map(|field| (field.name == name).then_some(&field.value))
        .unwrap_or_else(|| panic!("missing record field {name}"))
}

fn assert_compile_stage(
    expr: &SurfaceExpr,
    expected_consumes: &[&str],
    expected_produces: &str,
    expected_produces_hash_pin: &str,
    expected_compiled_by: &str,
) {
    let fields = match expr {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "CompileStage");
            fields
        }
        other => panic!("expected CompileStage record, got {other:?}"),
    };

    assert_eq!(
        list_var_names(record_field_expr(fields, "consumes")),
        expected_consumes,
        "CompileStage.consumes drifted"
    );
    assert_eq!(
        var_name(record_field_expr(fields, "produces")),
        expected_produces,
        "CompileStage.produces drifted"
    );
    assert_hash_pin(
        record_field_expr(fields, "produces_hash"),
        expected_produces_hash_pin,
    );
    assert_eq!(
        var_name(record_field_expr(fields, "compiled_by")),
        expected_compiled_by,
        "CompileStage.compiled_by drifted"
    );
}

fn assert_hash_pin(expr: &SurfaceExpr, expected_pin: &str) {
    let fields = match expr {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "BootstrapHashPin");
            fields
        }
        other => panic!("expected BootstrapHashPin record, got {other:?}"),
    };
    assert_eq!(
        var_name(record_field_expr(fields, "pin")),
        expected_pin,
        "BootstrapHashPin.pin drifted"
    );
}

fn assert_fixpt(
    expr: &SurfaceExpr,
    expected_left: &str,
    expected_left_hash_pin: &str,
    expected_right: &str,
    expected_right_hash_pin: &str,
    expected_pinned_hash_pin: &str,
    expected_via: &str,
) {
    let fields = match expr {
        SurfaceExpr::VariantRecord { target, fields, .. } => {
            assert_eq!(target, "FixptStage1Stage2");
            fields
        }
        other => panic!("expected FixptStage1Stage2 record, got {other:?}"),
    };

    assert_eq!(var_name(record_field_expr(fields, "left")), expected_left);
    assert_hash_pin(
        record_field_expr(fields, "left_hash"),
        expected_left_hash_pin,
    );
    assert_eq!(var_name(record_field_expr(fields, "right")), expected_right);
    assert_hash_pin(
        record_field_expr(fields, "right_hash"),
        expected_right_hash_pin,
    );
    assert_hash_pin(
        record_field_expr(fields, "pinned_hash"),
        expected_pinned_hash_pin,
    );
    assert_eq!(var_name(record_field_expr(fields, "via")), expected_via);
}

fn list_var_names(expr: &SurfaceExpr) -> Vec<&str> {
    match expr {
        SurfaceExpr::List { elements, .. } => elements.iter().map(var_name).collect(),
        other => panic!("expected list of symbols, got {other:?}"),
    }
}

fn var_name(expr: &SurfaceExpr) -> &str {
    match expr {
        SurfaceExpr::Var { name, .. } => name,
        other => panic!("expected symbol var, got {other:?}"),
    }
}
