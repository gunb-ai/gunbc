use std::rc::Rc;

use v2_compiler::infer_access;
use v2_compiler::infer_env::TypeEnv;
use v2_compiler::infer_patterns::{self, NodeLookupStatus};
use v2_compiler::infer_types::{
    bare_map_node, container_node, leaf_node, map_node, with_optional_cardinality,
};
use v2_compiler::v2_core::{Cardinality, InferredNode, MatchArm, MatchPattern, Node, NodeType, SourceSpan};

fn zero_span() -> SourceSpan {
    SourceSpan { start: 0, end: 0 }
}

fn unit_expr() -> Rc<Node> {
    leaf_node("Unit")
}

fn variant_arm(name: &str) -> Rc<MatchArm> {
    Rc::new(MatchArm {
        pattern: Rc::new(MatchPattern::VariantPattern {
            name: name.to_string(),
            parent_enum: None,
            field_bindings: Rc::new(Vec::new()),
        }),
        guard: None,
        body: unit_expr(),
    })
}

fn assert_compiler_error(return_type: &Option<Rc<InferredNode>>, message_fragment: &str) {
    match return_type.as_ref().expect("expected return_type").as_ref() {
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
    let result = infer_access::check_index_access_node(
        container_node("List", leaf_node("Int")),
        leaf_node("Int"),
        zero_span(),
        "test",
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.return_type, "indexing is only supported");
}

#[test]
fn malformed_map_index_returns_compiler_error_type() {
    let result = infer_access::check_index_access_node(
        bare_map_node(),
        leaf_node("String"),
        zero_span(),
        "test",
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.return_type, "malformed Map type");
}

#[test]
fn invalid_slice_returns_compiler_error_type() {
    let result = infer_access::check_slice_access_node(
        container_node("List", leaf_node("Int")),
        leaf_node("Int"),
        leaf_node("Int"),
        zero_span(),
        "test",
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.return_type, "slice is only supported");
}

#[test]
fn valid_map_index_preserves_optional_value_type() {
    let result = infer_access::check_index_access_node(
        map_node(leaf_node("String"), leaf_node("Int")),
        leaf_node("String"),
        zero_span(),
        "test",
    );

    assert!(result.diagnostics.is_empty());
    match result.return_type.as_ref().expect("expected return type").as_ref() {
        InferredNode::Resolved { node, .. } => {
            assert_eq!(node.name, "Int");
            assert!(matches!(node.return_cardinality, Cardinality::CardOptional));
        }
        other => panic!("expected resolved return type, got {:?}", other),
    }
}

#[test]
fn pattern_lookup_blocks_on_infer_error_without_cascade_diagnostic() {
    let subject = infer_patterns::pattern_subject_from_node_type(Rc::new(NodeType::InferError {
        message: "upstream failure".to_string(),
        span: zero_span(),
    }));
    let lookup = infer_patterns::lookup_variant_in_type(subject, "Some", "test");

    assert!(matches!(lookup.status.as_ref(), NodeLookupStatus::LookupFailed));
    assert!(
        lookup.diagnostics.is_empty(),
        "upstream infer failure should not add cascade diagnostics"
    );
}

#[test]
fn pattern_lookup_reports_dynamic_scrutinee_explicitly() {
    let subject = infer_patterns::pattern_subject_from_node(leaf_node("Dynamic"));
    let lookup = infer_patterns::lookup_variant_in_type(subject, "Some", "test");

    assert!(matches!(lookup.status.as_ref(), NodeLookupStatus::LookupFailed));
    assert_eq!(lookup.diagnostics.len(), 1);
    assert!(
        lookup.diagnostics[0].message.contains("Dynamic scrutinee"),
        "expected targeted Dynamic diagnostic, got {:?}",
        lookup.diagnostics[0].message
    );
}

#[test]
fn optional_pattern_lookup_still_resolves_some_variant() {
    let subject = infer_patterns::pattern_subject_from_node(with_optional_cardinality(leaf_node("String")));
    let lookup = infer_patterns::lookup_variant_in_type(subject, "Some", "test");

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
    let diags = infer_patterns::check_match_exhaustiveness(
        with_optional_cardinality(leaf_node("String")),
        Rc::new(vec![variant_arm("Some")]),
        Rc::new(TypeEnv::default()),
        zero_span(),
        "test",
    );

    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("non-exhaustive"));
    assert!(diags[0].message.contains("None"));
}

#[test]
fn optional_match_exhaustiveness_accepts_some_and_none() {
    let diags = infer_patterns::check_match_exhaustiveness(
        with_optional_cardinality(leaf_node("String")),
        Rc::new(vec![variant_arm("Some"), variant_arm("None")]),
        Rc::new(TypeEnv::default()),
        zero_span(),
        "test",
    );

    assert!(
        diags.is_empty(),
        "Some/None arms should exhaust Optional matches, got {:?}",
        diags
    );
}
