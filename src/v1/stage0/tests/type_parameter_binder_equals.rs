use im::HashMap;
use std::rc::Rc;
use v1_compiler::std_decl_ref::decl_type_parameter_ref;
use v1_compiler::v1_compiler_infer_resolve::node_with_declaration;
use v1_compiler::v1_compiler_infer_types::{
    node_type_equals, type_variable_node, TextJudgment, TextNotAskedReason,
};
use v1_compiler::v1_std_core::NewlineIndex;

fn empty_si() -> Rc<HashMap<String, Rc<NewlineIndex>>> {
    Rc::new(HashMap::new())
}

fn text() -> Rc<TextJudgment> {
    Rc::new(TextJudgment::TextNotAsked {
        reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
    })
}

fn t_of(module: &str, fn_name: &str) -> Rc<v1_compiler::v1_std_core::Node> {
    node_with_declaration(
        type_variable_node("T".to_string()),
        decl_type_parameter_ref(module.to_string(), fn_name.to_string(), "T".to_string()),
    )
}

#[test]
fn type_parameter_compares_by_binder_not_spelling() {
    let si = empty_si();
    let t = text();
    let f_t = t_of("mod", "f");
    let f_t_again = t_of("mod", "f");
    let g_t = t_of("mod", "g");
    assert!(
        node_type_equals(f_t.clone(), f_t_again, si.clone(), t.clone()),
        "T of f vs T of f must be equal by binder identity"
    );
    assert!(
        !node_type_equals(f_t, g_t, si, t),
        "T of f vs T of g must not be equal"
    );
}

#[test]
fn inference_variable_compares_by_id_not_leaf_name() {
    let si = empty_si();
    let t = text();
    let a = type_variable_node("tv.1".to_string());
    let a_again = type_variable_node("tv.1".to_string());
    let b = type_variable_node("tv.2".to_string());
    assert!(
        node_type_equals(a.clone(), a_again, si.clone(), t.clone()),
        "same inference-variable id must be equal"
    );
    assert!(
        !node_type_equals(a, b, si, t),
        "distinct inference-variable ids must not be equal"
    );
}

fn unnamed_leaf(name: &str) -> Rc<v1_compiler::v1_std_core::Node> {
    let mut n = (*type_variable_node("unused".to_string())).clone();
    n.name = name.to_string();
    n.inferred = None;
    Rc::new(n)
}

#[test]
fn unmarked_leaf_named_like_the_variable_is_that_variable() {
    let si = empty_si();
    let t = text();
    let tv = type_variable_node("T".to_string());
    let leaf = unnamed_leaf("T");
    assert!(
        node_type_equals(tv.clone(), leaf, si.clone(), t.clone()),
        "TypeVariable T vs unmarked leaf T is the same variable"
    );
    assert!(
        !node_type_equals(tv, unnamed_leaf("K"), si, t),
        "TypeVariable T vs unmarked leaf K is not"
    );
}
