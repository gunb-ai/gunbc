use std::rc::Rc;
use v1_compiler::std_decl_ref::{DeclField, DeclarationRef};
use v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity;
use v1_compiler::v1_compiler_infer_types::{node_type_equals, TextJudgment, TextNotAskedReason};
use v1_compiler::v1_std_core::{make_expr_node, Connective, ExprData, Node, SourceSpan};

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
        equals(applied_x.clone(), enum_x),
        "positive control: same declaration"
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
