#![allow(clippy::disallowed_macros)]

//! Host-physics floor witness for dag_collect fingerprint oracle suite
//! (migrated from `src/v1/tests/src/dag_collect_fingerprint_witness_test.rs`).
//! Exercises bag/seq hash commutativity, recursive surface fingerprint
//! discrimination, and the leaf_mix contrastive control until
//! `v1.compiler.dag_collect_support` is witness-layer importable.

use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::v1_compiler_dag_collect_support::{
    dag_collect_fp_memo_reset, dag_node_bag_hash, dag_node_seq_hash, dag_node_surface_fingerprint,
    dag_node_surface_leaf_mix,
};
use v1_compiler::v1_rt::atom_identity_hash;
use v1_compiler::v1_std_core::empty_node_list;
use v1_compiler::v1_std_core::{Cardinality, Connective, ExprData, Node, SourceSpan};

type WitnessCase = (&'static str, fn());

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("dag_collect_fingerprint_witness: {msg}");
    ExitCode::from(1)
}

fn hashes(labels: Vec<&str>) -> Rc<im::Vector<String>> {
    Rc::new(
        labels
            .into_iter()
            .map(|s| atom_identity_hash(s.to_string()))
            .collect(),
    )
}

fn synth_span() -> Rc<SourceSpan> {
    Rc::new(SourceSpan {
        file: "witness.dag".to_string(),
        start: 0,
        end: 0,
    })
}

fn shell_node(
    name: &str,
    connective: Connective,
    children: Vec<Rc<Node>>,
    params: Vec<Rc<Node>>,
) -> Rc<Node> {
    Rc::new(Node {
        name: name.to_string(),
        ident: None,
        span: synth_span(),
        ident_span: None,
        children: Rc::new(children.into()),
        connective,
        params: Rc::new(params.into()),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: empty_node_list(),
        body: None,
        transport: None,
        properties: empty_node_list(),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
    })
}

fn bag_hash_is_commutative() {
    assert_eq!(
        dag_node_bag_hash(hashes(vec!["a", "b"])),
        dag_node_bag_hash(hashes(vec!["b", "a"])),
        "bag hash must be order-insensitive (Conj/Disj children commute)"
    );
}

fn seq_hash_is_order_sensitive() {
    assert_ne!(
        dag_node_seq_hash(hashes(vec!["a", "b"])),
        dag_node_seq_hash(hashes(vec!["b", "a"])),
        "seq hash must be order-sensitive (Arrow/NoConnective children are positional)"
    );
}

fn bag_hash_distinguishes_different_content() {
    assert_ne!(
        dag_node_bag_hash(hashes(vec!["a", "b"])),
        dag_node_bag_hash(hashes(vec!["a", "c"])),
        "bag hash must distinguish different child sets"
    );
}

fn seq_hash_distinguishes_different_content() {
    assert_ne!(
        dag_node_seq_hash(hashes(vec!["a", "b"])),
        dag_node_seq_hash(hashes(vec!["a", "c"])),
        "seq hash must distinguish different child sequences"
    );
}

fn empty_bag_and_seq_hash_are_different() {
    assert_ne!(
        dag_node_bag_hash(hashes(vec![])),
        dag_node_seq_hash(hashes(vec![])),
        "empty bag and empty seq must have distinct sentinel hashes"
    );
}

fn recursive_fingerprint_distinguishes_same_named_child_subtrees() {
    let child_a_conj = shell_node("a", Connective::Conj, vec![], vec![]);
    let child_a_disj = shell_node("a", Connective::Disj, vec![], vec![]);
    let left = shell_node("wrap", Connective::Conj, vec![child_a_conj], vec![]);
    let right = shell_node("wrap", Connective::Conj, vec![child_a_disj], vec![]);
    assert_ne!(
        dag_node_surface_fingerprint(left),
        dag_node_surface_fingerprint(right),
        "recursive fingerprint must distinguish structurally different 0..0 subtrees with identical child names"
    );
}

fn recursive_fingerprint_distinguishes_param_order() {
    let p_x = shell_node("x", Connective::NoConnective, vec![], vec![]);
    let p_y = shell_node("y", Connective::NoConnective, vec![], vec![]);
    let left = shell_node(
        "fn",
        Connective::Arrow,
        vec![],
        vec![p_x.clone(), p_y.clone()],
    );
    let right = shell_node("fn", Connective::Arrow, vec![], vec![p_y, p_x]);
    assert_ne!(
        dag_node_surface_fingerprint(left),
        dag_node_surface_fingerprint(right),
        "params are positional — swapped order must not alias"
    );
}

fn recursive_fingerprint_distinguishes_arrow_child_order() {
    let c_x = shell_node("x", Connective::NoConnective, vec![], vec![]);
    let c_y = shell_node("y", Connective::NoConnective, vec![], vec![]);
    let left = shell_node(
        "arr",
        Connective::Arrow,
        vec![c_x.clone(), c_y.clone()],
        vec![],
    );
    let right = shell_node("arr", Connective::Arrow, vec![c_y, c_x], vec![]);
    assert_ne!(
        dag_node_surface_fingerprint(left),
        dag_node_surface_fingerprint(right),
        "Arrow children are ordered — swapped operands must not alias"
    );
}

fn recursive_fingerprint_conj_child_order_insensitive() {
    let c_a = shell_node("a", Connective::NoConnective, vec![], vec![]);
    let c_b = shell_node("b", Connective::NoConnective, vec![], vec![]);
    let left = shell_node(
        "bag",
        Connective::Conj,
        vec![c_a.clone(), c_b.clone()],
        vec![],
    );
    let right = shell_node("bag", Connective::Conj, vec![c_b, c_a], vec![]);
    assert_eq!(
        dag_node_surface_fingerprint(left),
        dag_node_surface_fingerprint(right),
        "Conj siblings are commutative — reorder must not false-split"
    );
}

fn recursive_fingerprint_disj_child_order_insensitive() {
    let c_a = shell_node("a", Connective::NoConnective, vec![], vec![]);
    let c_b = shell_node("b", Connective::NoConnective, vec![], vec![]);
    let left = shell_node(
        "bag",
        Connective::Disj,
        vec![c_a.clone(), c_b.clone()],
        vec![],
    );
    let right = shell_node("bag", Connective::Disj, vec![c_b, c_a], vec![]);
    assert_eq!(
        dag_node_surface_fingerprint(left),
        dag_node_surface_fingerprint(right),
        "Disj siblings are commutative — reorder must not false-split"
    );
}

fn recursive_fingerprint_necessary_leaf_mix_collides_recursive_distinguishes() {
    let child_conj = shell_node("child", Connective::Conj, vec![], vec![]);
    let child_disj = shell_node("child", Connective::Disj, vec![], vec![]);
    let node_a = shell_node("root", Connective::NoConnective, vec![child_conj], vec![]);
    let node_b = shell_node("root", Connective::NoConnective, vec![child_disj], vec![]);
    assert_eq!(
        dag_node_surface_leaf_mix(node_a.clone()),
        dag_node_surface_leaf_mix(node_b.clone()),
        "leaf_mix must be identical (same name/connective/expr_data) — a simple leaf fingerprint would collide"
    );
    assert_ne!(
        dag_node_surface_fingerprint(node_a),
        dag_node_surface_fingerprint(node_b),
        "recursive fingerprint must distinguish the two nodes despite identical leaf_mix"
    );
}

const WITNESS_CASES: &[WitnessCase] = &[
    ("bag_hash_is_commutative", bag_hash_is_commutative),
    ("seq_hash_is_order_sensitive", seq_hash_is_order_sensitive),
    (
        "bag_hash_distinguishes_different_content",
        bag_hash_distinguishes_different_content,
    ),
    (
        "seq_hash_distinguishes_different_content",
        seq_hash_distinguishes_different_content,
    ),
    (
        "empty_bag_and_seq_hash_are_different",
        empty_bag_and_seq_hash_are_different,
    ),
    (
        "recursive_fingerprint_distinguishes_same_named_child_subtrees",
        recursive_fingerprint_distinguishes_same_named_child_subtrees,
    ),
    (
        "recursive_fingerprint_distinguishes_param_order",
        recursive_fingerprint_distinguishes_param_order,
    ),
    (
        "recursive_fingerprint_distinguishes_arrow_child_order",
        recursive_fingerprint_distinguishes_arrow_child_order,
    ),
    (
        "recursive_fingerprint_conj_child_order_insensitive",
        recursive_fingerprint_conj_child_order_insensitive,
    ),
    (
        "recursive_fingerprint_disj_child_order_insensitive",
        recursive_fingerprint_disj_child_order_insensitive,
    ),
    (
        "recursive_fingerprint_necessary_leaf_mix_collides_recursive_distinguishes",
        recursive_fingerprint_necessary_leaf_mix_collides_recursive_distinguishes,
    ),
];

fn main() -> ExitCode {
    for (name, case) in WITNESS_CASES {
        dag_collect_fp_memo_reset();
        if let Err(e) = std::panic::catch_unwind(case) {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "assertion failed".to_string()
            };
            return fail(format!("{name}: {msg}"));
        }
    }
    ExitCode::SUCCESS
}
