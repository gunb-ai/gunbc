use std::rc::Rc;
use v1_compiler::std_decl_ref::{DeclField, DeclarationRef};
use v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity;
use v1_compiler::v1_compiler_infer_env::empty_type_env;
use v1_compiler::v1_compiler_infer_resolve::generic_declaration_application;
use v1_compiler::v1_compiler_infer_types::{node_type_equals, TextJudgment, TextNotAskedReason};
use v1_compiler::v1_std_core::{
    make_expr_node, Connective, ExprData, InferredNode, Node, SourceSpan,
};

// Behavioural RED at the authority itself: node_type_equals is pure, so the head-declaration-identity
// arm (an applied spelling against an unapplied one) and the element-wise descent (two applied
// spellings) are driven with built nodes. The consumer of the arm is the #14152 rebind fold; this
// file is the v1-test class receipt, deleted with src/v1 (DESIGN.md 7, seed shrinks toward zero).

fn named(name: &str, connective: Connective, children: Vec<Rc<Node>>) -> Node {
    let base = make_expr_node(
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
    Node {
        name: name.to_string(),
        ident_span: Some(Rc::new(SourceSpan {
            file: format!("<kernel:{}>", name),
            start: 0,
            end: name.len() as i64,
        })),
        connective,
        children: Rc::new(children.into_iter().collect()),
        ..(*base).clone()
    }
}

fn leaf(name: &str) -> Rc<Node> {
    Rc::new(named(name, Connective::NoConnective, vec![]))
}

fn applied(name: &str, args: Vec<Rc<Node>>) -> Rc<Node> {
    Rc::new(named(name, Connective::NoConnective, args))
}

fn declared_in(mut n: Node, module_path: &str) -> Rc<Node> {
    n.declaration = Some(Rc::new(DeclarationRef {
        module_path: module_path.to_string(),
        decl_name: n.name.clone(),
        field: Rc::new(DeclField::WholeDeclaration),
    }));
    Rc::new(n)
}

fn disj(name: &str, variants: &[&str]) -> Node {
    named(
        name,
        Connective::Disj,
        variants.iter().map(|v| leaf(v)).collect(),
    )
}

fn equals(left: Rc<Node>, right: Rc<Node>) -> bool {
    node_type_equals(
        left,
        right,
        Rc::new(im::HashMap::new()),
        Rc::new(TextJudgment::TextNotAsked {
            reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
        }),
    )
}

#[test]
fn applied_spellings_descend_element_wise() {
    let int_list = applied("List", vec![leaf("Int")]);
    assert!(
        equals(int_list.clone(), applied("List", vec![leaf("Int")])),
        "positive control"
    );
    assert!(
        !equals(int_list, applied("List", vec![leaf("Bool")])),
        "List<Int> vs List<Bool> must refuse; dropping the element-wise descent admits it"
    );
}

#[test]
fn applied_spelling_joins_its_unapplied_spelling_only_by_declaration_identity() {
    let applied_x = declared_in((*applied("X", vec![leaf("Int")])).clone(), "m.one");
    let enum_x = declared_in(disj("X", &["A", "B"]), "m.one");
    assert!(
        !equals(applied_x.clone(), enum_x),
        "an application whose arguments cannot be recovered from the other side refuses, never agrees; the positive control runs the real producer below"
    );

    let other_x = declared_in(disj("X", &["A", "B"]), "m.two");
    assert!(
        !equals(applied_x.clone(), other_x),
        "a same-named type declared elsewhere has a different identity"
    );
    let plain_x = declared_in((*leaf("X")).clone(), "m.two");
    assert!(
        !equals(applied_x, plain_x),
        "a same-named non-enum with a different declaration must refuse"
    );
}

// THE REAL PRODUCER, BOTH OPERAND ORDERS: generic_declaration_application builds the use-site node (the
// arguments in children) and its expansion (a coproduct with the arguments substituted into its variant
// fields). The same declaration at a different instantiation must refuse in either order.
fn box_application(argument: &str) -> Rc<Node> {
    let slot = leaf("T");
    let mut decl = named(
        "Box",
        Connective::Disj,
        vec![Rc::new(named("Wrap", Connective::Conj, vec![slot.clone()]))],
    );
    decl.params = Rc::new(vec![slot].into_iter().collect());
    let use_site = named("Box", Connective::NoConnective, vec![leaf(argument)]);
    let env = Rc::new((*empty_type_env()).clone());
    generic_declaration_application(
        Rc::new(use_site),
        Rc::new(decl),
        "Box".to_string(),
        Rc::new(vec![leaf(argument)].into_iter().collect()),
        Rc::new(im::HashMap::unit("T".to_string(), leaf(argument))),
        env,
    )
}

fn expansion_of(application: &Rc<Node>) -> Rc<Node> {
    match application.inferred.clone().as_deref() {
        Some(InferredNode::Resolved { node }) => node.clone(),
        other => panic!(
            "generic_declaration_application must resolve to its expansion, got {:?}",
            other
        ),
    }
}

#[test]
fn an_application_agrees_with_its_own_expansion_in_both_orders() {
    let int_box = box_application("Int");
    let expansion = expansion_of(&int_box);
    assert!(
        equals(int_box.clone(), expansion.clone()),
        "Box<Int> vs its own expansion"
    );
    assert!(equals(expansion, int_box), "its own expansion vs Box<Int>");
}

#[test]
fn an_application_refuses_the_expansion_of_another_instantiation_in_both_orders() {
    let int_box = box_application("Int");
    let bool_expansion = expansion_of(&box_application("Bool"));
    assert!(
        !equals(int_box.clone(), bool_expansion.clone()),
        "Box<Int> vs the expansion of Box<Bool> must refuse: head declaration identity alone does not separate two instantiations"
    );
    assert!(
        !equals(bool_expansion, int_box),
        "reverse operand order must refuse too"
    );
}
