//! Positive control for R1 dispatch-authority lookup routing.
//! On every site, a roster row without a matching handler macro arm fails `cargo build`
//! via a non-exhaustive inner `match` on the generated enum (not exercised here). Each
//! `EvalCallBridgeFamilySite` bridge family gets its own generated enum and lookup
//! function keyed by the family's module, so a handler gap in one family cannot be
//! papered over by a shared wildcard.

use v1_compiler::v1_interpreter_dispatch_generated::{
    lookup_eval_algebra_method_inner, lookup_eval_builtin_inner,
    lookup_eval_call_bridge_std_compilers_lexing, lookup_eval_call_bridge_std_node,
    lookup_eval_call_native_intercept, lookup_try_parse_table_memo_dispatch,
    lookup_try_v2_std_collection_map_primitive_grounding, EvalAlgebraMethodArm, EvalBuiltinArm,
    EvalCallBridgeStdCompilersLexingArm, EvalCallBridgeStdNodeArm, EvalCallNativeInterceptArm,
    TryParseTableMemoDispatchArm, TryV2StdCollectionMapPrimitiveGroundingArm,
};

fn all_eval_builtin_variants_reachable() {
    let spellings = ["to_string", "map_insert", "get", "observed_monotonic_nanos"];
    for spelling in spellings {
        assert!(
            lookup_eval_builtin_inner(spelling).is_some(),
            "missing lookup for eval_builtin_inner spelling: {spelling}"
        );
    }
}

fn all_eval_algebra_variants_reachable() {
    let spellings = ["lookup", "map", "length", "fold"];
    for spelling in spellings {
        assert!(
            lookup_eval_algebra_method_inner(spelling).is_some(),
            "missing lookup for eval_algebra_method_inner spelling: {spelling}"
        );
    }
}

fn all_bridge_variants_reachable() {
    assert!(
        lookup_eval_call_bridge_std_node("resolve_type_node").is_some(),
        "missing lookup for eval_call bridge (std_node) spelling: resolve_type_node"
    );
    assert!(
        lookup_eval_call_bridge_std_compilers_lexing("symbol_lexeme").is_some(),
        "missing lookup for eval_call bridge (std_compilers_lexing) spelling: symbol_lexeme"
    );
}

#[test]
fn generated_lookup_covers_representative_spellings_per_site() {
    all_eval_builtin_variants_reachable();
    all_eval_algebra_variants_reachable();
    all_bridge_variants_reachable();
    assert!(lookup_try_v2_std_collection_map_primitive_grounding("map_insert").is_some());
    assert!(lookup_eval_call_native_intercept("fold_list").is_some());
    assert!(lookup_try_parse_table_memo_dispatch("parse_table_lookup").is_some());
}

#[test]
fn generated_enums_are_copy_and_distinct() {
    assert_ne!(
        EvalBuiltinArm::FreeCallToString,
        EvalBuiltinArm::FreeCallMapInsert
    );
    assert_ne!(
        EvalAlgebraMethodArm::MethodCallLookup,
        EvalAlgebraMethodArm::MethodCallMap
    );
    assert_ne!(
        EvalCallBridgeStdCompilersLexingArm::V4BridgeSymbolInternLexeme,
        EvalCallBridgeStdCompilersLexingArm::V4BridgeSymbolLexeme
    );
    assert_eq!(
        EvalCallBridgeStdNodeArm::V4BridgeResolveTypeNode,
        EvalCallBridgeStdNodeArm::V4BridgeResolveTypeNode
    );
    assert_ne!(
        TryV2StdCollectionMapPrimitiveGroundingArm::MapGroundingMapInsert,
        TryV2StdCollectionMapPrimitiveGroundingArm::MapGroundingEmptyMap
    );
    assert_ne!(
        EvalCallNativeInterceptArm::NativeInterceptFoldList,
        EvalCallNativeInterceptArm::NativeInterceptFoldListRight
    );
    assert_ne!(
        TryParseTableMemoDispatchArm::ParseTableMemoParseTableInsert,
        TryParseTableMemoDispatchArm::ParseTableMemoParseTableLookup
    );
}
