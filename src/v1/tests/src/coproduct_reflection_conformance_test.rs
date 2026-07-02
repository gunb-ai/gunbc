//! RESIDUAL after 5-test-migration (2026-07-02): 3 of the original 4 tests are
//! deleted here — they were duplicate harnesses invoking floor-enrolled witnesses
//! (coproduct_reflection_conformance_holds + 3 arm-set/nullary witnesses in
//! src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag, and
//! concept_index_enumeration_witnesses in
//! src/v2/test/claim/concept_index_enumeration_test.dag), all marker-discovered.
//! The test below CANNOT migrate: it proves the v1 interpreter's intercept WIRING
//! by scanning v1_interpreter.rs source text — a fact about the pinned Rust
//! harness itself, dissolving only with the v1 interpreter.
use v1_compiler::v1_interpreter;

#[test]
fn coproduct_reflection_std_node_bridge_fns_are_intercept_wired() {
    let source = include_str!("../../stage0/src/v1_interpreter.rs");
    for name in v1_interpreter::std_node_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.node bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_node_query_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.node_query bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_concept_index_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.concept_index bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_fn_index_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.fn_index bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_data_index_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.data_index bridge `{name}`"
        );
    }
    assert!(
        source.contains("is_v4_std_node_bridge_call"),
        "eval_call must gate v2.std.node bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_node_query_bridge_call"),
        "eval_call must gate v2.std.node_query bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_concept_index_bridge_call"),
        "eval_call must gate v2.std.concept_index bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_fn_index_bridge_call"),
        "eval_call must gate v2.std.fn_index bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_data_index_bridge_call"),
        "eval_call must gate v2.std.data_index bridge dispatch"
    );
}
