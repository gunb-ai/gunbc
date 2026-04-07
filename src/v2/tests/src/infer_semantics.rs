use std::rc::Rc;

use v2_compiler::v2_compiler_infer_access;
use v2_compiler::v2_compiler_infer_env::{TypeBinding, TypeEnv};
use v2_compiler::v2_compiler_infer_lookup;
use v2_compiler::v2_compiler_infer_patterns::{self, NodeLookupStatus};
use v2_compiler::v2_compiler_infer_resolve::resolve_node;
use v2_compiler::v2_compiler_infer_types::{
    bare_map_node, container_node, decl_resolved_type, is_fully_resolved, map_node, node_is_keyed_collection,
};
use v2_compiler::v2_compiler_parse;
use v2_compiler::v2_std_core::{
    leaf_node, make_arm_node, with_optional_cardinality, Cardinality, Connective, ExprData,
    InferredNode, MatchPattern, Node, NodeType, SourceSpan,
};

fn zero_span() -> Rc<SourceSpan> {
    Rc::new(SourceSpan { file: String::new(), start: 0, end: 0 })
}

fn unit_expr() -> Rc<Node> {
    leaf_node("Unit".to_string())
}

fn variant_arm(name: &str) -> Rc<Node> {
    make_arm_node(
        Rc::new(MatchPattern::VariantPattern {
            name: name.to_string(),
            parent_enum: None,
            field_bindings: Rc::new(vec![]),
        }),
        None,
        unit_expr(),
        zero_span(),
    )
}

fn assert_compiler_error(inferred: &Option<Rc<InferredNode>>, message_fragment: &str) {
    match inferred.as_ref().expect("expected inferred").as_ref() {
        InferredNode::CompilerError { message, .. } => {
            assert!(
                message.contains(message_fragment),
                "expected compiler error containing '{message_fragment}', got '{message}'"
            );
        }
        other => panic!("expected CompilerError return type, got {:?}", other),
    }
}

#[test]
fn list_int_index_returns_optional_element_type() {
    let result = v2_compiler_infer_access::check_index_access_node(
        container_node("List".to_string(), leaf_node("Int".to_string())),
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(result.diagnostics.len(), 0, "List<Int> indexed by Int should succeed");
    assert!(result.inferred.is_some(), "List<Int> index should produce a type");
}

#[test]
fn malformed_map_index_returns_compiler_error_type() {
    // bare_map_node() now has K/V wrapper children with TypeVariable inferred,
    // so it's recognized as a keyed collection — key type mismatch is diagnosed
    let result = v2_compiler_infer_access::check_index_access_node(
        bare_map_node(),
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.inferred, "key type does not match");
}

#[test]
fn invalid_slice_returns_compiler_error_type() {
    let result = v2_compiler_infer_access::check_slice_access_node(
        container_node("List".to_string(), leaf_node("Int".to_string())),
        leaf_node("Int".to_string()),
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.inferred, "slice is only supported");
}

#[test]
fn valid_map_index_preserves_optional_value_type() {
    let result = v2_compiler_infer_access::check_index_access_node(
        map_node(
            leaf_node("String".to_string()),
            leaf_node("Int".to_string()),
        ),
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
    );

    assert!(result.diagnostics.is_empty());
    match result
        .inferred
        .as_ref()
        .expect("expected return type")
        .as_ref()
    {
        InferredNode::Resolved { node, .. } => {
            assert_eq!(node.name, "Int");
            assert!(matches!(node.return_cardinality, Cardinality::CardOptional));
        }
        other => panic!("expected resolved return type, got {:?}", other),
    }
}

#[test]
fn pattern_lookup_blocks_on_infer_error_without_cascade_diagnostic() {
    let subject =
        v2_compiler_infer_patterns::pattern_subject_from_node_type(Rc::new(NodeType::InferError {
            message: "upstream failure".to_string(),
            span: zero_span(),
        }));
    let lookup = v2_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
    );

    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
    assert!(
        lookup.diagnostics.is_empty(),
        "upstream infer failure should not add cascade diagnostics"
    );
}

#[test]
fn pattern_lookup_reports_error_scrutinee_structurally() {
    // Error types carry CompilerError in inferred — pattern_subject_from_node
    // detects this structurally and returns PatternLookupBlocked.
    use v2_compiler::v2_std_core::error_type;
    let subject =
        v2_compiler_infer_patterns::pattern_subject_from_node(error_type());
    let lookup = v2_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
    );

    // PatternLookupBlocked produces LookupFailed with 0 diagnostics (silent failure)
    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
}

#[test]
fn optional_pattern_lookup_still_resolves_some_variant() {
    let subject = v2_compiler_infer_patterns::pattern_subject_from_node(with_optional_cardinality(
        leaf_node("String".to_string()),
    ));
    let lookup = v2_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
    );

    match lookup.status.as_ref() {
        NodeLookupStatus::LookupResolved { node, .. } => {
            assert_eq!(node.name, "Some");
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.children[0].name, "value");
        }
        status => panic!("expected Some lookup to resolve, got {:?}", status),
    }
}

#[test]
fn optional_match_exhaustiveness_reports_missing_none() {
    let diags = v2_compiler_infer_patterns::check_match_exhaustiveness(
        with_optional_cardinality(leaf_node("String".to_string())),
        Rc::new(vec![variant_arm("Some")]),
        Rc::new(TypeEnv {
            bindings: Rc::new(std::collections::HashMap::new()),
            recursive_types: Rc::new(vec![]),
            recursive_type_set: Rc::new(std::collections::HashMap::new()),
            recursive_variant_fields: Rc::new(std::collections::HashMap::new()),
            source_index: None,
        }),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(diags.len(), 1);
    let diag0_msg = v2_compiler::v2_std_core::diagnostic_to_message(diags[0].diagnostic.clone());
    assert!(diag0_msg.contains("non-exhaustive"));
    assert!(diag0_msg.contains("None"));
}

#[test]
fn optional_match_exhaustiveness_accepts_some_and_none() {
    let diags = v2_compiler_infer_patterns::check_match_exhaustiveness(
        with_optional_cardinality(leaf_node("String".to_string())),
        Rc::new(vec![variant_arm("Some"), variant_arm("None")]),
        Rc::new(TypeEnv {
            bindings: Rc::new(std::collections::HashMap::new()),
            recursive_types: Rc::new(vec![]),
            recursive_type_set: Rc::new(std::collections::HashMap::new()),
            recursive_variant_fields: Rc::new(std::collections::HashMap::new()),
            source_index: None,
        }),
        zero_span(),
        "test".to_string(),
    );

    assert!(
        diags.is_empty(),
        "Some/None arms should exhaust Optional matches, got {:?}",
        diags
    );
}

#[test]
fn resolve_node_uses_node_name_for_lookup() {
    let node_ref = Rc::new(Node {
        name: "User".to_string(),
        span: zero_span(),
        ident_span: None,
        children: Rc::new(vec![]),
        connective: v2_compiler::v2_std_core::Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
    });
    let env = Rc::new(TypeEnv {
        bindings: Rc::new(std::collections::HashMap::from([(
            "User".to_string(),
            Rc::new(TypeBinding {
                name: "User".to_string(),
                resolved: leaf_node("User".to_string()),
            }),
        )])),
        recursive_types: Rc::new(vec![]),
        recursive_type_set: Rc::new(std::collections::HashMap::new()),
        recursive_variant_fields: Rc::new(std::collections::HashMap::new()),
        source_index: None,
    });

    let result = resolve_node(node_ref, env, "test".to_string());

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.resolved.name, "User");
}

// =========================================================================
// Higher-order method instantiation tests
//
// These test observable behavior through the public lookup_structural_method
// API: given a receiver type and method name, does the method resolve, and
// what is the result type? No peeking into Node.inferred or params —
// the tests exercise the same contract downstream consumers use.
// =========================================================================

#[test]
fn structural_method_lookup_resolves_all_list_collection_methods() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let expected_methods = [
        "map", "filter", "flat_map", "fold", "any", "all", "count", "first", "last", "skip",
        "take", "sort_by", "append", "contains", "enumerate", "reverse", "join", "concat",
    ];
    for method_name in &expected_methods {
        assert!(
            v2_compiler_infer_lookup::lookup_structural_method(
                list_int.clone(),
                method_name.to_string()
            )
            .is_some(),
            "lookup_structural_method should resolve '{}' on List<Int>",
            method_name
        );
    }
}

#[test]
fn structural_method_any_on_list_returns_bool() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        list_int,
        "any".to_string(),
    )
    .expect("any must resolve on List<Int>");
    assert_eq!(result.result_type.name, "Bool", "any on List<Int> should return Bool");
}

#[test]
fn structural_method_all_on_list_returns_bool() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        list_int,
        "all".to_string(),
    )
    .expect("all must resolve on List<Int>");
    assert_eq!(result.result_type.name, "Bool", "all on List<Int> should return Bool");
}

#[test]
fn structural_method_sort_by_on_list_returns_self() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        list_int,
        "sort_by".to_string(),
    )
    .expect("sort_by must resolve on List<Int>");
    assert_eq!(
        result.result_type.name, "List",
        "sort_by on List<Int> should return List (ReceiverSelf)"
    );
}

#[test]
fn structural_method_first_on_list_returns_optional_element() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        list_int,
        "first".to_string(),
    )
    .expect("first must resolve on List<Int>");
    assert_eq!(result.result_type.name, "Int", "first on List<Int> should return Int");
    assert!(
        matches!(result.result_type.return_cardinality, Cardinality::CardOptional),
        "first should return Optional"
    );
}

#[test]
fn structural_method_count_on_list_returns_int() {
    let list_string = container_node("List".to_string(), leaf_node("String".to_string()));
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        list_string,
        "count".to_string(),
    )
    .expect("count must resolve on List<String>");
    assert_eq!(result.result_type.name, "Int", "count should return Int");
}

#[test]
fn structural_method_lookup_resolves_all_int_ring_methods() {
    let int_node = leaf_node("Int".to_string());
    let expected_methods = ["add", "zero", "negate", "mul", "one", "compare"];
    for method_name in &expected_methods {
        assert!(
            v2_compiler_infer_lookup::lookup_structural_method(
                int_node.clone(),
                method_name.to_string()
            )
            .is_some(),
            "lookup_structural_method should resolve '{}' on Int",
            method_name
        );
    }
}

#[test]
fn structural_method_compare_on_int_returns_ordering() {
    let int_node = leaf_node("Int".to_string());
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        int_node,
        "compare".to_string(),
    )
    .expect("compare must resolve on Int");
    assert_eq!(
        result.result_type.name, "Ordering",
        "compare on Int should return Ordering"
    );
}

#[test]
fn structural_method_lookup_resolves_all_map_partial_function_methods() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let expected_methods = [
        "get", "map_get", "lookup", "map_insert", "map_merge", "has", "keys", "values",
        "contains", "length",
    ];
    for method_name in &expected_methods {
        assert!(
            v2_compiler_infer_lookup::lookup_structural_method(
                m.clone(),
                method_name.to_string()
            )
            .is_some(),
            "lookup_structural_method should resolve '{}' on Map<String,Int>",
            method_name
        );
    }
}

#[test]
fn structural_method_get_on_map_returns_optional_value() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        m,
        "get".to_string(),
    )
    .expect("get must resolve on Map<String,Int>");
    assert_eq!(result.result_type.name, "Int", "get on Map<String,Int> should return Int");
    assert!(
        matches!(result.result_type.return_cardinality, Cardinality::CardOptional),
        "get should return Optional"
    );
}

#[test]
fn structural_method_keys_on_map_returns_list_of_key_type() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v2_compiler_infer_lookup::lookup_structural_method(
        m,
        "keys".to_string(),
    )
    .expect("keys must resolve on Map<String,Int>");
    assert_eq!(result.result_type.name, "List", "keys should return List");
    assert_eq!(result.result_type.children.len(), 1, "keys result should have one child");
    // Children are now field-style wrappers — extract type from inferred
    let elem_child = &result.result_type.children[0];
    let elem_type = decl_resolved_type(elem_child.clone());
    assert_eq!(
        elem_type.name, "String",
        "keys on Map<String,Int> should return List<String>"
    );
}

#[test]
fn structural_method_lookup_returns_none_for_unknown_type() {
    let custom = leaf_node("MyType".to_string());
    assert!(
        v2_compiler_infer_lookup::lookup_structural_method(custom, "add".to_string()).is_none(),
        "custom types without algebra should not have structural methods"
    );
}

// =========================================================================
// Keyed-collection access tests
//
// These verify that KeyedCollectionParts correctly decomposes keyed
// collection types (Map) and that the structural predicate
// node_is_keyed_collection distinguishes maps from element collections.
// =========================================================================

#[test]
fn keyed_collection_parts_extracts_key_and_value() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let parts = v2_compiler_infer_access::keyed_collection_parts(m);
    let parts = parts.expect("Map<String,Int> should decompose to keyed parts");
    assert_eq!(parts.key_type.name, "String");
    assert_eq!(parts.value_type.name, "Int");
}

#[test]
fn keyed_collection_parts_returns_none_for_element_collection() {
    let list = container_node("List".to_string(), leaf_node("Int".to_string()));
    let parts = v2_compiler_infer_access::keyed_collection_parts(list);
    assert!(
        parts.is_none(),
        "List<Int> is not a keyed collection, should return None"
    );
}

#[test]
fn keyed_collection_parts_returns_type_variables_for_bare_map() {
    // bare_map_node() now has K/V wrapper children with TypeVariable inferred
    let bare = bare_map_node();
    let parts = v2_compiler_infer_access::keyed_collection_parts(bare);
    assert!(
        parts.is_some(),
        "bare Map has K/V children (TypeVariable inferred)"
    );
}

#[test]
fn node_is_keyed_collection_true_for_map() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Bool".to_string()),
    );
    assert!(node_is_keyed_collection(m));
}

#[test]
fn node_is_keyed_collection_false_for_list() {
    let list = container_node("List".to_string(), leaf_node("Int".to_string()));
    assert!(!node_is_keyed_collection(list));
}

#[test]
fn node_is_keyed_collection_false_for_leaf() {
    let leaf = leaf_node("String".to_string());
    assert!(!node_is_keyed_collection(leaf));
}

// ── is_fully_resolved ─────────────────────────────────────────────────

#[test]
fn is_fully_resolved_rejects_under_parameterized_container() {
    // leaf_node("List") creates a node named "List" with 0 children.
    // container_expected_arity("List") = Some(1), so 0 < 1 → not fully resolved.
    let bare_list = leaf_node("List".to_string());
    assert!(!is_fully_resolved(bare_list));
}

#[test]
fn is_fully_resolved_accepts_parameterized_container() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    assert!(is_fully_resolved(list_int));
}

#[test]
fn is_fully_resolved_ignores_unknown_type_names() {
    // User-defined "Widget" with 0 children → arity is None → not under-parameterized.
    let widget = leaf_node("Widget".to_string());
    assert!(is_fully_resolved(widget));
}

#[test]
fn map_index_with_correct_key_type_succeeds() {
    let map_type = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v2_compiler_infer_access::check_index_access_node(
        map_type,
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
    );
    assert!(
        result.diagnostics.is_empty(),
        "Map<String,Int>[String] should succeed, got: {:?}",
        result.diagnostics.len()
    );
    match result.inferred.as_ref().map(|i| i.as_ref()) {
        Some(InferredNode::Resolved { node }) => {
            assert_eq!(node.name, "Int");
            assert!(matches!(node.return_cardinality, Cardinality::CardOptional));
        }
        other => panic!("expected Resolved(Int?), got {:?}", other),
    }
}

#[test]
fn map_index_with_wrong_key_type_reports_error() {
    let map_type = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v2_compiler_infer_access::check_index_access_node(
        map_type,
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "Map<String,Int>[Int] should report key type mismatch"
    );
}

// =========================================================================
// Boundary regression tests (review feedback 2026-04-02)
// =========================================================================

#[test]
fn node_inferred_to_outputs_returns_empty_when_child_has_error() {
    // Conj node with two children: one Typed, one CompilerError.
    // Fail-closed gate must return [] — no partial output synthesis.
    let typed_child = Rc::new(Node {
        name: "x".to_string(),
        inferred: Some(Rc::new(InferredNode::Resolved {
            node: leaf_node("Int".to_string()),
        })),
        connective: Connective::NoConnective,
        ..(*leaf_node("".to_string())).clone()
    });
    let error_child = Rc::new(Node {
        name: "y".to_string(),
        inferred: Some(Rc::new(InferredNode::CompilerError {
            message: "upstream failure".to_string(),
            span: zero_span(),
        })),
        connective: Connective::NoConnective,
        ..(*leaf_node("".to_string())).clone()
    });
    let conj_node = Rc::new(Node {
        name: "Result".to_string(),
        connective: Connective::Conj,
        children: Rc::new(vec![typed_child, error_child]),
        ..(*leaf_node("".to_string())).clone()
    });

    let outputs = v2_compiler_parse::node_inferred_to_outputs(conj_node);
    assert!(
        outputs.is_empty(),
        "fail-closed gate: Conj with error child must produce 0 outputs, got {}",
        outputs.len()
    );
}

