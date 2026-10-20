use im::{vector::Vector, HashMap};
use std::rc::Rc;
use v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity;
use v1_compiler::v1_compiler_infer::substitute_generics_child;
use v1_compiler::v1_compiler_infer_types::child_type_node;
use v1_compiler::v1_std_core::{make_named_expr_node, no_span, ExprData, InferredNode, Node};

// Producer-level control for the slot substitution in src/v1/04_infer.dag: it builds the type nodes
// resolve_node_bounded hands the substitution and calls the exact function the repair edits, so a
// slot's payload and its slot boundary are observed directly. The route-level behavior is stated by
// test.claim.generic_slot_substitution_witness.

fn node(name: &str, children: Vec<Rc<Node>>, inferred: Option<InferredNode>) -> Rc<Node> {
    make_named_expr_node(
        Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
        name.to_string(),
        Rc::new(ExprData::NoExprData),
        Rc::new(children.into_iter().collect::<Vector<_>>()),
        inferred.map(Rc::new),
        no_span(),
        no_span(),
    )
}

fn plain(name: &str) -> Rc<Node> {
    node(name, vec![], None)
}

fn type_variable(id: &str) -> Rc<Node> {
    node(
        id,
        vec![],
        Some(InferredNode::TypeVariable { id: id.to_string() }),
    )
}

fn slot_holding(name: &str, held: Rc<Node>) -> Rc<Node> {
    node(name, vec![], Some(InferredNode::Resolved { node: held }))
}

fn substitute(c: Rc<Node>, bindings: &[(&str, Rc<Node>)]) -> Rc<Node> {
    let mut subst: HashMap<String, Rc<Node>> = HashMap::new();
    for (k, v) in bindings {
        subst.insert(k.to_string(), v.clone());
    }
    substitute_generics_child(c, Rc::new(subst), Rc::new(HashMap::new()))
}

fn is_slot(n: &Rc<Node>) -> bool {
    n.children.is_empty() && matches!(n.inferred.as_deref(), Some(InferredNode::Resolved { .. }))
}

#[test]
fn a_forwarded_type_variable_in_a_slot_takes_the_later_binding() {
    let slot_v = slot_holding("V", plain("Placeholder"));
    let forwarded = substitute(slot_v, &[("V", type_variable("A"))]);
    // V -> A: the slot holds the variable A and is still a slot.
    assert!(is_slot(&forwarded));
    assert_eq!(forwarded.name, "V");
    let at_int = substitute(forwarded, &[("A", plain("Int"))]);
    // A -> Int: the type the slot holds is Int, read through the projection every consumer applies.
    assert!(
        is_slot(&at_int),
        "the position stays one slot, named as before"
    );
    assert_eq!(at_int.name, "V");
    assert_eq!(child_type_node(at_int.clone()).name, "Int");
    assert!(!is_slot(&child_type_node(at_int)));
}

#[test]
fn a_slot_shaped_replacement_is_installed_under_one_slot_boundary() {
    let boxed = node(
        "Box",
        vec![plain("Int")],
        Some(InferredNode::Resolved {
            node: plain("BoxExpansion"),
        }),
    );
    let replacement = slot_holding("X", boxed.clone());
    let slot_v = slot_holding("V", plain("Placeholder"));
    let installed = substitute(slot_v, &[("V", replacement)]);
    let held = child_type_node(installed.clone());
    assert_eq!(held.name, "Box");
    assert!(!is_slot(&held), "the slot holds the type, not another slot");
    assert_eq!(
        held.children.len(),
        1,
        "the application keeps its arguments"
    );
}

#[test]
fn a_bare_argument_stays_bare() {
    let substituted = substitute(plain("A"), &[("A", plain("Int"))]);
    assert_eq!(substituted.name, "Int");
    assert!(substituted.inferred.is_none());
}
