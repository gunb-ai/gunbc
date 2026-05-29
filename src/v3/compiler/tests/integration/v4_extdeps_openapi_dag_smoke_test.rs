//! **Layer:** integration
//!
//! T-4.6 / SL-3229-OPENAPI-WIRECONTRACT-COORDINATION ratchet for
//! `src/v4/extdeps/formats/openapi.dag`: `OpenApiOperationWireContractFacts`
//! composes `coordination.dag` `WireContractFacts`; exchange / settlement /
//! consistency authority routes through `coordination_effect_obligation(Http)`.
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching
//! `EXPECTED_HAND_AUTHORED_TEST` line in `sg0_census_test.rs` + INVARIANTS table
//! row land in the same PR.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const OPENAPI_DAG: &str = include_str!("../../../../v4/extdeps/formats/openapi.dag");
const OPENAPI_PATH: &str = "src/v4/extdeps/formats/openapi.dag";

fn openapi_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(OPENAPI_DAG, OPENAPI_PATH)
        .unwrap_or_else(|e| panic!("{OPENAPI_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, OPENAPI_PATH).unwrap_or_else(|e| panic!("{OPENAPI_PATH}: parse: {e:?}"))
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
fn v4_extdeps_openapi_dag_parses() {
    let _module = openapi_surface_or_panic();
}

#[test]
fn v4_extdeps_openapi_operation_wire_contract_composes_coordination_facts() {
    let module = openapi_surface_or_panic();
    let facts = type_record_fields(&module, "OpenApiOperationWireContractFacts");
    assert_eq!(
        record_field_type(facts, "coordination"),
        "WireContractFacts"
    );
    assert_eq!(
        record_field_type(facts, "admitted_method"),
        "OpenApiAdmittedHttpMethod"
    );
    assert_eq!(
        record_field_type(facts, "path_template"),
        "OpenApiPathTemplateKey"
    );
    assert!(
        OPENAPI_DAG.contains("fn openapi_http_obligation() -> CoordinationEffectObligation")
            && OPENAPI_DAG.contains("coordination_effect_obligation(effect: Http)"),
        "OpenAPI wire-contract facts must source the Http obligation row from coordination.dag"
    );
    assert!(
        OPENAPI_DAG.contains("fn openapi_http_settlement_from_obligation(")
            && OPENAPI_DAG.contains("ImmediateSettlement => Accepted { value: ImmediateSettlement")
            && OPENAPI_DAG.contains("openapi_http_bounded_settlement_not_admitted"),
        "Http settlement must project ImmediateSettlement from the obligation row and fail-closed on bounded required settlement"
    );
    assert!(
        OPENAPI_DAG.contains("fn openapi_http_consistency_from_obligation(")
            && OPENAPI_DAG.contains("NoReplicaConvergence => Accepted { value: NoReplicaConvergence")
            && OPENAPI_DAG.contains("openapi_http_bounded_consistency_not_admitted"),
        "Http consistency must project NoReplicaConvergence from the obligation row and fail-closed on bounded required consistency"
    );
    assert!(
        !OPENAPI_DAG.contains("data openapi_operation_exchange_request_reply")
            && !OPENAPI_DAG.contains("wire_contract_settlement_from_obligation")
            && !OPENAPI_DAG.contains("SettleBound { steps: 0 }"),
        "OpenAPI must not carry a parallel RequestReply witness, generic obligation coercion with fabricated bounds, or steps:0 stubs"
    );
    assert!(
        OPENAPI_DAG.contains("fn openapi_http_wire_contract_facts(")
            && OPENAPI_DAG.contains("-> Outcome<WireContractFacts>")
            && OPENAPI_DAG.contains("fn openapi_operation_wire_contract_facts(")
            && OPENAPI_DAG.contains("-> Outcome<OpenApiOperationWireContractFacts>")
            && OPENAPI_DAG.contains("fn openapi_operation_wire_contract(")
            && OPENAPI_DAG.contains("facts: facts.coordination"),
        "OpenApiOperationWireContractFacts must compose coordination WireContract via Outcome-guarded facts.coordination"
    );
}
