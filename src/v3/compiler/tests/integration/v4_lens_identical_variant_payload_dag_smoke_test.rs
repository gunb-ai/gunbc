//! **Layer:** integration
//!
//! Parse-level ratchet for `src/v4/lens/identical_variant_payload.dag` — L1.4
//! IdenticalVariantPayload sub-signature substrate (Carrier-clone family;
//! shared `coverage_defect_carrier_clone` acceptance key).

const IDENTICAL_VARIANT_PAYLOAD_DAG: &str =
    include_str!("../../../../v4/lens/identical_variant_payload.dag");
const IDENTICAL_VARIANT_PAYLOAD_PATH: &str = "src/v4/lens/identical_variant_payload.dag";

#[test]
fn v4_lens_identical_variant_payload_dag_tokenizes_and_parses() {
    let tokens = v3_compiler::tokenize_for_test(
        IDENTICAL_VARIANT_PAYLOAD_DAG,
        IDENTICAL_VARIANT_PAYLOAD_PATH,
    )
    .unwrap_or_else(|diag| {
        panic!("{IDENTICAL_VARIANT_PAYLOAD_PATH}: tokenization failed: {diag:?}")
    });
    v3_compiler::parse_for_test(&tokens, IDENTICAL_VARIANT_PAYLOAD_PATH)
        .unwrap_or_else(|diag| panic!("{IDENTICAL_VARIANT_PAYLOAD_PATH}: parse failed: {diag:?}"));
}

#[test]
fn v4_lens_identical_variant_payload_module_authority_and_entrypoints() {
    let source = IDENTICAL_VARIANT_PAYLOAD_DAG;
    assert!(
        source.contains("module v4.lens.identical_variant_payload"),
        "{IDENTICAL_VARIANT_PAYLOAD_PATH}: module path must be v4.lens.identical_variant_payload"
    );
    for needle in [
        "type CarrierCloneTrigger",
        "WholeCarrierClone",
        "VariantParameterClone",
        "IdenticalVariantPayload",
        "type IdenticalVariantPayloadVerdict",
        "type IdenticalVariantPayloadFact",
        "fn identical_variant_payload_fact_for_type",
        "coverage_defect_carrier_clone",
    ] {
        assert!(
            source.contains(needle),
            "{IDENTICAL_VARIANT_PAYLOAD_PATH}: must declare `{needle}`"
        );
    }
}
