use std::rc::Rc;

use v2_compiler::v2_compiler_infer_access;
use v2_compiler::v2_compiler_infer_env::{TypeBinding, TypeEnv};
use v2_compiler::v2_compiler_infer_lookup;
use v2_compiler::v2_compiler_infer_patterns::{self, NodeLookupStatus};
use v2_compiler::v2_compiler_infer_resolve::resolve_node;
use v2_compiler::v2_compiler_infer_types::{
    bare_map_node, container_node, enrich_kernel_type, map_node, node_is_keyed_collection,
};
use v2_compiler::v2_std_core::{
    leaf_node, make_arm_node, with_optional_cardinality, Cardinality, ExprData, InferredNode,
    MatchPattern, Node, NodeType, SourceSpan,
};

fn zero_span() -> Rc<SourceSpan> {
    SourceSpan::new(0, 0)
}

fn unit_expr() -> Rc<Node> {
    leaf_node("Unit".to_string())
}

fn variant_arm(name: &str) -> Rc<Node> {
    make_arm_node(
        Rc::new(MatchPattern::VariantPattern {
            name: name.to_string(),
            parent_enum: None,
            field_bindings: Vec::new(),
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
fn invalid_list_index_returns_compiler_error_type() {
    let result = v2_compiler_infer_access::check_index_access_node(
        container_node("List".to_string(), leaf_node("Int".to_string())),
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.inferred, "indexing is only supported");
}

#[test]
fn malformed_map_index_returns_compiler_error_type() {
    let result = v2_compiler_infer_access::check_index_access_node(
        bare_map_node(),
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.inferred, "indexing is only supported");
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
fn pattern_lookup_reports_dynamic_scrutinee_explicitly() {
    let subject =
        v2_compiler_infer_patterns::pattern_subject_from_node(leaf_node("Dynamic".to_string()));
    let lookup = v2_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
    );

    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
    assert_eq!(lookup.diagnostics.len(), 1);
    let diag_msg =
        v2_compiler::v2_std_core::diagnostic_to_message(lookup.diagnostics[0].diagnostic.clone());
    assert!(
        diag_msg.contains("variant") && diag_msg.contains("not found"),
        "expected targeted VariantNotFound diagnostic, got {:?}",
        diag_msg
    );
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
        vec![variant_arm("Some")],
        Rc::new(TypeEnv {
            bindings: Rc::new(std::collections::HashMap::new()),
            recursive_types: Rc::new(vec![]),
            recursive_type_set: Rc::new(std::collections::HashMap::new()),
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
        vec![variant_arm("Some"), variant_arm("None")],
        Rc::new(TypeEnv {
            bindings: Rc::new(std::collections::HashMap::new()),
            recursive_types: Rc::new(vec![]),
            recursive_type_set: Rc::new(std::collections::HashMap::new()),
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
        children: Vec::new(),
        connective: None,
        params: Vec::new(),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Vec::new(),
        body: None,
        transport: None,
        properties: Vec::new(),
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
        source_index: None,
    });

    let result = resolve_node(node_ref, env, "test".to_string());

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.resolved.name, "User");
}

// =========================================================================
// Higher-order method instantiation tests
//
// These verify that enrich_kernel_type produces structural algebra fields
// with correct callable shapes, especially for higher-order methods like
// any, all, filter, map, sort_by, and fold on collection types.
// =========================================================================

fn find_enriched_field(enriched: &Node, field_name: &str) -> Option<Rc<Node>> {
    enriched
        .children
        .iter()
        .find(|c| c.name == field_name)
        .cloned()
}

fn field_is_callable(field: &Node) -> bool {
    match field.inferred.as_ref().map(|i| i.as_ref()) {
        Some(InferredNode::Resolved { node: rt }) => !rt.params.is_empty(),
        _ => false,
    }
}

fn field_return_type(field: &Node) -> Option<Rc<Node>> {
    match field.inferred.as_ref().map(|i| i.as_ref()) {
        Some(InferredNode::Resolved { node: rt }) => {
            if rt.params.is_empty() {
                Some(rt.clone())
            } else {
                match rt.inferred.as_ref().map(|i| i.as_ref()) {
                    Some(InferredNode::Resolved { node: ret }) => Some(ret.clone()),
                    _ => None,
                }
            }
        }
        _ => None,
    }
}

#[test]
fn enrich_list_produces_collection_method_fields() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let enriched = enrich_kernel_type("List".to_string(), list_int);

    assert!(
        !enriched.children.is_empty(),
        "enriched List<Int> should have algebra fields"
    );

    let expected_methods = [
        "map", "filter", "flat_map", "fold", "any", "all", "count", "first", "last", "skip",
        "take", "sort_by", "append", "contains", "enumerate", "reverse", "join", "concat",
    ];
    for method_name in &expected_methods {
        assert!(
            find_enriched_field(&enriched, method_name).is_some(),
            "enriched List<Int> should have '{}' field",
            method_name
        );
    }
}

#[test]
fn enrich_list_any_is_callable_returning_bool() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let enriched = enrich_kernel_type("List".to_string(), list_int);
    let any_field = find_enriched_field(&enriched, "any").expect("any field must exist");

    assert!(
        field_is_callable(&any_field),
        "any should be a callable field"
    );
    let ret = field_return_type(&any_field).expect("any must have return type");
    assert_eq!(ret.name, "Bool", "any should return Bool");
}

#[test]
fn enrich_list_all_is_callable_returning_bool() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let enriched = enrich_kernel_type("List".to_string(), list_int);
    let all_field = find_enriched_field(&enriched, "all").expect("all field must exist");

    assert!(
        field_is_callable(&all_field),
        "all should be a callable field"
    );
    let ret = field_return_type(&all_field).expect("all must have return type");
    assert_eq!(ret.name, "Bool", "all should return Bool");
}

#[test]
fn enrich_list_sort_by_is_callable_returning_self() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let enriched = enrich_kernel_type("List".to_string(), list_int);
    let sort_field = find_enriched_field(&enriched, "sort_by").expect("sort_by field must exist");

    assert!(
        field_is_callable(&sort_field),
        "sort_by should be a callable field"
    );
    let ret = field_return_type(&sort_field).expect("sort_by must have return type");
    assert_eq!(ret.name, "List", "sort_by should return List (ReceiverSelf)");
}

#[test]
fn enrich_list_first_returns_optional_element() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let enriched = enrich_kernel_type("List".to_string(), list_int);
    let first_field = find_enriched_field(&enriched, "first").expect("first field must exist");

    assert!(
        field_is_callable(&first_field),
        "first should be a callable field"
    );
    let ret = field_return_type(&first_field).expect("first must have return type");
    assert!(
        matches!(ret.return_cardinality, Cardinality::CardOptional),
        "first should return Optional (CardOptional)"
    );
    assert_eq!(ret.name, "Int", "first on List<Int> should return Int?");
}

#[test]
fn enrich_list_count_returns_int() {
    let list_string = container_node("List".to_string(), leaf_node("String".to_string()));
    let enriched = enrich_kernel_type("List".to_string(), list_string);
    let count_field = find_enriched_field(&enriched, "count").expect("count field must exist");

    assert!(
        field_is_callable(&count_field),
        "count should be a callable field"
    );
    let ret = field_return_type(&count_field).expect("count must have return type");
    assert_eq!(ret.name, "Int", "count should return Int");
}

#[test]
fn enrich_int_produces_ordered_ring_fields() {
    let int_node = leaf_node("Int".to_string());
    let enriched = enrich_kernel_type("Int".to_string(), int_node);

    let expected_methods = ["add", "zero", "negate", "mul", "one", "compare"];
    for method_name in &expected_methods {
        assert!(
            find_enriched_field(&enriched, method_name).is_some(),
            "enriched Int should have '{}' field",
            method_name
        );
    }
}

#[test]
fn enrich_int_compare_returns_ordering() {
    let int_node = leaf_node("Int".to_string());
    let enriched = enrich_kernel_type("Int".to_string(), int_node);
    let cmp = find_enriched_field(&enriched, "compare").expect("compare field must exist");

    assert!(field_is_callable(&cmp), "compare should be callable");
    let ret = field_return_type(&cmp).expect("compare must have return type");
    assert_eq!(ret.name, "Ordering", "compare should return Ordering");
}

#[test]
fn enrich_map_produces_partial_function_fields() {
    let map_str_int = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let enriched = enrich_kernel_type("Map".to_string(), map_str_int);

    let expected_methods = [
        "get",
        "map_get",
        "lookup",
        "map_insert",
        "map_merge",
        "has",
        "keys",
        "values",
        "contains",
        "length",
    ];
    for method_name in &expected_methods {
        assert!(
            find_enriched_field(&enriched, method_name).is_some(),
            "enriched Map<String,Int> should have '{}' field",
            method_name
        );
    }
}

#[test]
fn enrich_map_get_returns_optional_value() {
    let map_str_int = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let enriched = enrich_kernel_type("Map".to_string(), map_str_int);
    let get_field = find_enriched_field(&enriched, "get").expect("get field must exist");

    assert!(field_is_callable(&get_field), "get should be callable");
    let ret = field_return_type(&get_field).expect("get must have return type");
    assert!(
        matches!(ret.return_cardinality, Cardinality::CardOptional),
        "get should return Optional"
    );
    assert_eq!(
        ret.name, "Int",
        "get on Map<String,Int> should return Int?"
    );
}

#[test]
fn enrich_map_keys_returns_list_of_key_type() {
    let map_str_int = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let enriched = enrich_kernel_type("Map".to_string(), map_str_int);
    let keys_field = find_enriched_field(&enriched, "keys").expect("keys field must exist");

    assert!(field_is_callable(&keys_field), "keys should be callable");
    let ret = field_return_type(&keys_field).expect("keys must have return type");
    assert_eq!(ret.name, "List", "keys should return List");
    assert_eq!(ret.children.len(), 1, "keys List should have one element");
    assert_eq!(
        ret.children[0].name, "String",
        "keys on Map<String,Int> should return List<String>"
    );
}

#[test]
fn enrich_unknown_type_passes_through_unchanged() {
    let custom = leaf_node("MyCustomType".to_string());
    let enriched = enrich_kernel_type("MyCustomType".to_string(), custom.clone());
    assert_eq!(enriched.children.len(), 0);
    assert_eq!(enriched.name, "MyCustomType");
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
fn keyed_collection_parts_returns_none_for_bare_map() {
    let bare = bare_map_node();
    let parts = v2_compiler_infer_access::keyed_collection_parts(bare);
    assert!(
        parts.is_none(),
        "bare Map (no children) should return None"
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
// Structural method lookup tests
//
// These verify that lookup_structural_method resolves methods from the
// algebra templates instantiated by enrich_kernel_type.
// =========================================================================

#[test]
fn lookup_structural_method_finds_list_count() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result =
        v2_compiler_infer_lookup::lookup_structural_method(list_int, "count".to_string());
    assert!(result.is_some(), "count should be found on List<Int>");
    let result_node = result.unwrap();
    assert_eq!(result_node.name, "Int", "count should return Int");
}

#[test]
fn lookup_structural_method_finds_int_add() {
    let int_node = leaf_node("Int".to_string());
    let result = v2_compiler_infer_lookup::lookup_structural_method(int_node, "add".to_string());
    assert!(result.is_some(), "add should be found on Int");
    let result_node = result.unwrap();
    assert_eq!(result_node.name, "Int", "add on Int should return Int");
}

#[test]
fn lookup_structural_method_returns_none_for_unknown_method() {
    let int_node = leaf_node("Int".to_string());
    let result =
        v2_compiler_infer_lookup::lookup_structural_method(int_node, "nonexistent".to_string());
    assert!(
        result.is_none(),
        "nonexistent method should not be found on Int"
    );
}

#[test]
fn lookup_structural_method_finds_map_get() {
    let map_node = map_node(
        leaf_node("String".to_string()),
        leaf_node("Bool".to_string()),
    );
    let result =
        v2_compiler_infer_lookup::lookup_structural_method(map_node, "get".to_string());
    assert!(
        result.is_some(),
        "get should be found on Map<String,Bool>"
    );
    let result_node = result.unwrap();
    assert_eq!(result_node.name, "Bool", "get on Map<String,Bool> should return Bool");
    assert!(
        matches!(result_node.return_cardinality, Cardinality::CardOptional),
        "get should return Optional"
    );
}

#[test]
fn lookup_structural_method_returns_none_for_custom_type() {
    let custom = leaf_node("MyType".to_string());
    let result =
        v2_compiler_infer_lookup::lookup_structural_method(custom, "add".to_string());
    assert!(
        result.is_none(),
        "custom types without algebra should not have structural methods"
    );
}
