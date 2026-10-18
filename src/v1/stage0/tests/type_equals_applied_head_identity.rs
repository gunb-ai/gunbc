use std::rc::Rc;
use v1_compiler::std_decl_ref::{DeclField, DeclarationRef};
use v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity;
use v1_compiler::v1_compiler_infer_env::empty_type_env;
use v1_compiler::v1_compiler_infer_resolve::generic_declaration_application;
use v1_compiler::v1_compiler_infer_types::{node_type_equals, TextJudgment, TextNotAskedReason};
use v1_compiler::v1_std_core::{
    make_expr_node, Connective, ExprData, InferredNode, Node, SourceSpan,
};

// Behavioural RED at the authority itself: node_type_equals is pure, so the refusal of an application
// against any non-applied spelling and the element-wise descent (two applied spellings) are driven with
// built nodes. The consumer of the arm is the #14152 rebind fold; this
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

// An expansion RETAINS NO ARGUMENTS (slots replaced by name, params cleared), so an application refuses
// even its own expansion, in both orders. The hand-built Disj pair below refuses too: an application
// against a non-applied spelling is unsupported whatever the other side carries.
#[test]
fn an_application_refuses_its_own_expansion_in_both_orders() {
    let int_box = box_application("Int");
    let expansion = expansion_of(&int_box);
    assert!(
        !equals(int_box.clone(), expansion.clone()),
        "Box<Int> vs its own expansion: no retained arguments, must refuse"
    );
    assert!(
        !equals(expansion, int_box),
        "reverse operand order must refuse too"
    );
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

// gentle-carp-16's #14152 control shape: both operands hand-built, declaration None and inferred None,
// the Disj side carrying its arguments positionally in children. An application against a non-applied
// spelling is not equality whatever its inferred state, so Box<Int> against a hand-built
// Disj Box<Int> is a refusal control too: no expansion evidence must not reopen a comparison against
// the other side's children, which may be fields or variants.
fn disj_with_arguments(name: &str, args: Vec<Rc<Node>>) -> Rc<Node> {
    Rc::new(named(name, Connective::Disj, args))
}

#[test]
fn hand_built_application_refuses_a_non_applied_disj_in_both_orders() {
    let bound = applied("Box", vec![leaf("Int")]);
    for arg in ["Int", "Bool"] {
        let disj = disj_with_arguments("Box", vec![leaf(arg)]);
        assert!(
            !equals(bound.clone(), disj.clone()),
            "Box<Int> vs Disj Box<{arg}> must refuse"
        );
        assert!(
            !equals(disj, bound.clone()),
            "reverse operand order must refuse too"
        );
    }
}

// A CHILDLESS spelling of the SAME declaration is no proof of an argument-free instantiation: Box<Int>
// declared in m.one against a bare Box declared in m.one passes head identity, so only the missing
// arguments refuse it. Both operand orders (the existing m.two negative tests identity, not evidence).
#[test]
fn an_application_refuses_a_childless_spelling_of_its_own_declaration_in_both_orders() {
    let applied_x = declared_in((*applied("X", vec![leaf("Int")])).clone(), "m.one");
    let bare_x = declared_in((*leaf("X")).clone(), "m.one");
    assert!(
        !equals(applied_x.clone(), bare_x.clone()),
        "X<Int> vs bare X, same declaration: no argument evidence, must refuse"
    );
    assert!(
        !equals(bare_x, applied_x),
        "reverse operand order must refuse too"
    );
}

// AN ARGUMENT THAT VANISHES FROM THE EXPANSION: Pair<T, U> whose body mentions only T. The real producer
// builds Pair<Int, Bool> and Pair<Int, Int>; their expansions are identical, so expansion equality
// alone cannot separate the instantiations. Each application argument must occur in the other side.
fn pair_application(first: &str, second: &str) -> Rc<Node> {
    let (t, u) = (leaf("T"), leaf("U"));
    let mut decl = named(
        "Pair",
        Connective::Disj,
        vec![Rc::new(named("Wrap", Connective::Conj, vec![t.clone()]))],
    );
    decl.params = Rc::new(vec![t, u].into_iter().collect());
    let use_site = named(
        "Pair",
        Connective::NoConnective,
        vec![leaf(first), leaf(second)],
    );
    let env = Rc::new((*empty_type_env()).clone());
    generic_declaration_application(
        Rc::new(use_site),
        Rc::new(decl),
        "Pair".to_string(),
        Rc::new(vec![leaf(first), leaf(second)].into_iter().collect()),
        Rc::new(
            im::HashMap::unit("T".to_string(), leaf(first)).update("U".to_string(), leaf(second)),
        ),
        env,
    )
}

#[test]
fn an_argument_that_vanishes_from_the_expansion_refuses_in_both_orders() {
    let int_int = pair_application("Int", "Int");
    let int_bool = pair_application("Int", "Bool");
    let other = expansion_of(&int_bool);
    assert!(
        equals(expansion_of(&int_int), other.clone()),
        "premise: the two expansions are equal, so expansion equality cannot tell the instantiations apart"
    );
    assert!(
        !equals(int_int.clone(), other.clone()),
        "Pair<Int, Int> vs the expansion of Pair<Int, Bool>: U is never established"
    );
    assert!(
        !equals(other.clone(), int_int),
        "reverse operand order must refuse too"
    );
    assert!(
        !equals(
            int_bool.clone(),
            expansion_of(&pair_application("Int", "Int"))
        ),
        "Pair<Int, Bool> vs the expansion of Pair<Int, Int> refuses as well"
    );
}

// A PRODUCT EXPANSION: Stamp<T> { marker: Int } never mentions T, so the expansions of Stamp<Int> and
// Stamp<Bool> are equal and the application's argument Int meets the field Int by accident. A Conj
// expansion retains no arguments either, so the application refuses it in both orders (the Box and Pair
// controls above are all Disj, which is why the product path was missed).
fn stamp_application(argument: &str) -> Rc<Node> {
    let slot = leaf("T");
    let mut decl = named(
        "Stamp",
        Connective::Conj,
        vec![Rc::new(named("marker", Connective::NoConnective, vec![]))],
    );
    decl.children = Rc::new(vec![leaf("Int")].into_iter().collect());
    decl.params = Rc::new(vec![slot].into_iter().collect());
    let use_site = named("Stamp", Connective::NoConnective, vec![leaf(argument)]);
    let env = Rc::new((*empty_type_env()).clone());
    generic_declaration_application(
        Rc::new(use_site),
        Rc::new(decl),
        "Stamp".to_string(),
        Rc::new(vec![leaf(argument)].into_iter().collect()),
        Rc::new(im::HashMap::unit("T".to_string(), leaf(argument))),
        env,
    )
}

#[test]
fn an_application_refuses_a_product_expansion_of_another_instantiation_in_both_orders() {
    let int_stamp = stamp_application("Int");
    let bool_expansion = expansion_of(&stamp_application("Bool"));
    assert_eq!(
        bool_expansion.connective,
        Connective::Conj,
        "premise: the expansion is a product"
    );
    assert!(
        equals(expansion_of(&int_stamp), bool_expansion.clone()),
        "premise: the two expansions are equal, so only the arguments could tell them apart"
    );
    assert!(
        !equals(int_stamp.clone(), bool_expansion.clone()),
        "Stamp<Int> vs the product expansion of Stamp<Bool> must refuse"
    );
    assert!(
        !equals(bool_expansion, int_stamp),
        "reverse operand order must refuse too"
    );
}

#[test]
fn missing_expansion_evidence_must_not_reopen_field_comparison() {
    let mut stripped = (*stamp_application("Int")).clone();
    stripped.inferred = None;
    let stripped = Rc::new(stripped);
    let bool_expansion = expansion_of(&stamp_application("Bool"));
    assert!(
        !equals(stripped.clone(), bool_expansion.clone()),
        "an application with no expansion evidence must refuse the expansion of another instantiation"
    );
    assert!(
        !equals(bool_expansion, stripped),
        "reverse operand order must refuse too"
    );
}
