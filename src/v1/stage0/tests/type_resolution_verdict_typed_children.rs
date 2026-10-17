use std::rc::Rc;
use v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity;
use v1_compiler::v1_compiler_infer_types::{type_resolution_verdict, TypeResolutionVerdict};
use v1_compiler::v1_std_core::{
    make_expr_node, Connective, ExprData, InferredNode, Node, SourceSpan,
};

// Behavioural RED at the verdict authority: a child whose inferred slot is NOT Resolved used to be read
// through resolved_type, which fabricated the nameless error_type node, and that node answered
// FullyResolved. The typed read keeps the four states apart. Deleted with src/v1 (DESIGN.md 7); the
// consumer is is_fully_resolved's thirteen call sites. S1 of the nameless-type-leaf cutover.

fn base(name: &str, children: Vec<Rc<Node>>, inferred: Option<InferredNode>) -> Rc<Node> {
    let n = make_expr_node(
        Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
        Rc::new(ExprData::NoExprData),
        Rc::new(im::Vector::new()),
        None,
        Rc::new(SourceSpan {
            file: "<synthetic>".to_string(),
            start: 0,
            end: 0,
        }),
    );
    Rc::new(Node {
        name: name.to_string(),
        connective: Connective::NoConnective,
        children: Rc::new(children.into_iter().collect()),
        inferred: inferred.map(Rc::new),
        ..(*n).clone()
    })
}

fn resolved_int() -> Rc<Node> {
    base("Int", vec![], None)
}

fn verdict_with_child(child_inferred: InferredNode) -> TypeResolutionVerdict {
    let child = base("slot", vec![], Some(child_inferred));
    let parent = base("Pair", vec![child], None);
    (*type_resolution_verdict(parent, Rc::new(im::HashMap::new()))).clone()
}

#[test]
fn a_resolved_child_is_fully_resolved() {
    let v = verdict_with_child(InferredNode::Resolved {
        node: resolved_int(),
    });
    assert!(
        matches!(v, TypeResolutionVerdict::FullyResolved),
        "positive control: {:?}",
        v
    );
}

#[test]
fn an_unbound_type_variable_child_is_under_resolved() {
    let v = verdict_with_child(InferredNode::TypeVariable {
        id: "T".to_string(),
    });
    assert!(matches!(v, TypeResolutionVerdict::UnderResolved), "{:?}", v);
}

#[test]
fn a_carried_compiler_error_child_is_under_resolved() {
    let v = verdict_with_child(InferredNode::CompilerError {
        message: "carried".to_string(),
        span: Rc::new(SourceSpan {
            file: "<synthetic>".to_string(),
            start: 0,
            end: 0,
        }),
    });
    assert!(matches!(v, TypeResolutionVerdict::UnderResolved), "{:?}", v);
}

#[test]
fn a_divergent_child_is_divergent() {
    let v = verdict_with_child(InferredNode::Divergent);
    assert!(
        matches!(v, TypeResolutionVerdict::DivergentExpression),
        "{:?}",
        v
    );
}
