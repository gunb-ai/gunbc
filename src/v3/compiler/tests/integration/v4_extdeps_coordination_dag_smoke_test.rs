//! **Layer:** integration
//!
//! T-4.8 ratchet for `src/v4/extdeps/coordination.dag`: the PR #3207
//! decomposed shape is authoritative. `WireContractFacts` carries exchange /
//! settlement / consistency facts, `CoordinationBind` carries the closed
//! `CoordinationEffectKind`, and WIRECONTRACT-OBLIGATION-TABLE-T4.8 is visible
//! as executable per-effect obligation rows.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const COORDINATION_DAG: &str = include_str!("../../../../v4/extdeps/coordination.dag");
const COORDINATION_PATH: &str = "src/v4/extdeps/coordination.dag";

fn coordination_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(COORDINATION_DAG, COORDINATION_PATH)
        .unwrap_or_else(|e| panic!("{COORDINATION_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, COORDINATION_PATH)
        .unwrap_or_else(|e| panic!("{COORDINATION_PATH}: parse: {e:?}"))
}

fn type_record_fields<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a [SurfaceField] {
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

fn type_sum_variants(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Vec<String> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => Some(variants.iter().map(|v| v.name.clone()).collect()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type sum {name}"))
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

fn record_field_type(fields: &[SurfaceField], name: &str) -> String {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| surface_type_name(&field.ty))
        .unwrap_or_else(|| panic!("missing field {name}"))
}

#[test]
fn v4_extdeps_coordination_dag_parses() {
    let _module = coordination_surface_or_panic();
}

#[test]
fn v4_extdeps_coordination_wire_contract_uses_decomposed_facts() {
    let module = coordination_surface_or_panic();
    let facts = type_record_fields(&module, "WireContractFacts");
    assert_eq!(record_field_type(facts, "exchange"), "ExchangePattern");
    assert_eq!(
        record_field_type(facts, "settlement"),
        "SettlementGuarantee"
    );
    assert_eq!(
        record_field_type(facts, "consistency"),
        "ConsistencyGuarantee"
    );

    let wire = type_record_fields(&module, "WireContract");
    assert_eq!(record_field_type(wire, "facts"), "WireContractFacts");
    assert_eq!(record_field_type(wire, "bind"), "CoordinationBind");
}

#[test]
fn v4_extdeps_coordination_effect_kind_has_obligation_table() {
    let module = coordination_surface_or_panic();
    assert_eq!(
        type_sum_variants(&module, "CoordinationEffectKind"),
        ["Http", "Queue", "Stream", "PubSub"]
    );

    let obligation = type_record_fields(&module, "CoordinationEffectObligation");
    assert_eq!(
        record_field_type(obligation, "effect"),
        "CoordinationEffectKind"
    );
    assert_eq!(
        record_field_type(obligation, "required_exchange"),
        "ExchangePattern"
    );
    assert_eq!(
        record_field_type(obligation, "required_settlement"),
        "CoordinationSettlementRequirement"
    );
    assert_eq!(
        record_field_type(obligation, "required_consistency"),
        "CoordinationConsistencyRequirement"
    );

    assert!(
        COORDINATION_DAG.contains("fn coordination_effect_obligation"),
        "CoordinationEffectKind arms must route through executable obligation rows"
    );
    assert!(
        COORDINATION_DAG.contains("fn wire_contract_obligation_table_t4_8"),
        "WIRECONTRACT-OBLIGATION-TABLE-T4.8 table must be present"
    );
}
