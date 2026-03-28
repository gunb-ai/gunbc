use std::rc::Rc;

use v2_compiler::v2_compiler_infer_access;
use v2_compiler::v2_compiler_infer_env::TypeEnv;
use v2_compiler::v2_compiler_infer_patterns::{self, NodeLookupStatus};
use v2_compiler::v2_compiler_infer_types::{
    bare_map_node, container_node, map_node,
};
use v2_compiler::v2_std_core::{Cardinality, InferredNode, MatchArm, MatchPattern, Node, NodeType, SourceSpan, leaf_node, with_optional_cardinality};

fn zero_span() -> Rc<SourceSpan> {
    Rc::new(SourceSpan { start: 0, end: 0 })
}

fn unit_expr() -> Rc<Node> {
    leaf_node("Unit".to_string())
}

fn variant_arm(name: &str) -> Rc<MatchArm> {
    Rc::new(MatchArm {
        pattern: Rc::new(MatchPattern::VariantPattern {
            name: name.to_string(),
            parent_enum: None,
            field_bindings: Vec::new(),
        }),
        guard: None,
        body: unit_expr(),
    })
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
        map_node(leaf_node("String".to_string()), leaf_node("Int".to_string())),
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
    );

    assert!(result.diagnostics.is_empty());
    match result.inferred.as_ref().expect("expected return type").as_ref() {
        InferredNode::Resolved { node, .. } => {
            assert_eq!(node.name, "Int");
            assert!(matches!(node.return_cardinality, Cardinality::CardOptional));
        }
        other => panic!("expected resolved return type, got {:?}", other),
    }
}

#[test]
fn pattern_lookup_blocks_on_infer_error_without_cascade_diagnostic() {
    let subject = v2_compiler_infer_patterns::pattern_subject_from_node_type(Rc::new(NodeType::InferError {
        message: "upstream failure".to_string(),
        span: zero_span(),
    }));
    let lookup = v2_compiler_infer_patterns::lookup_variant_in_type(subject, "Some".to_string(), "test".to_string());

    assert!(matches!(lookup.status.as_ref(), NodeLookupStatus::LookupFailed));
    assert!(
        lookup.diagnostics.is_empty(),
        "upstream infer failure should not add cascade diagnostics"
    );
}

#[test]
fn pattern_lookup_reports_dynamic_scrutinee_explicitly() {
    let subject = v2_compiler_infer_patterns::pattern_subject_from_node(leaf_node("Dynamic".to_string()));
    let lookup = v2_compiler_infer_patterns::lookup_variant_in_type(subject, "Some".to_string(), "test".to_string());

    assert!(matches!(lookup.status.as_ref(), NodeLookupStatus::LookupFailed));
    assert_eq!(lookup.diagnostics.len(), 1);
    assert!(
        lookup.diagnostics[0].name.contains("Dynamic scrutinee"),
        "expected targeted Dynamic diagnostic, got {:?}",
        lookup.diagnostics[0].name
    );
}

#[test]
fn optional_pattern_lookup_still_resolves_some_variant() {
    let subject = v2_compiler_infer_patterns::pattern_subject_from_node(with_optional_cardinality(leaf_node("String".to_string())));
    let lookup = v2_compiler_infer_patterns::lookup_variant_in_type(subject, "Some".to_string(), "test".to_string());

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
        Rc::new(TypeEnv { bindings: std::collections::HashMap::new(), recursive_types: vec![], recursive_type_set: std::collections::HashMap::new() }),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(diags.len(), 1);
    assert!(diags[0].name.contains("non-exhaustive"));
    assert!(diags[0].name.contains("None"));
}

#[test]
fn optional_match_exhaustiveness_accepts_some_and_none() {
    let diags = v2_compiler_infer_patterns::check_match_exhaustiveness(
        with_optional_cardinality(leaf_node("String".to_string())),
        vec![variant_arm("Some"), variant_arm("None")],
        Rc::new(TypeEnv { bindings: std::collections::HashMap::new(), recursive_types: vec![], recursive_type_set: std::collections::HashMap::new() }),
        zero_span(),
        "test".to_string(),
    );

    assert!(
        diags.is_empty(),
        "Some/None arms should exhaust Optional matches, got {:?}",
        diags
    );
}
