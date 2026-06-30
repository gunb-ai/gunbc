use std::rc::Rc;
use v1_compiler::v1_compiler_dag_collect_support::{dag_node_bag_hash, dag_node_seq_hash};
use v1_compiler::v1_rt::atom_identity_hash;

fn hashes(labels: Vec<&str>) -> Rc<Vec<String>> {
    Rc::new(labels.into_iter().map(|s| atom_identity_hash(s.to_string())).collect())
}

#[test]
fn bag_hash_is_commutative() {
    assert_eq!(
        dag_node_bag_hash(hashes(vec!["a", "b"])),
        dag_node_bag_hash(hashes(vec!["b", "a"])),
        "bag hash must be order-insensitive (Conj/Disj children commute)"
    );
}

#[test]
fn seq_hash_is_order_sensitive() {
    assert_ne!(
        dag_node_seq_hash(hashes(vec!["a", "b"])),
        dag_node_seq_hash(hashes(vec!["b", "a"])),
        "seq hash must be order-sensitive (Arrow/NoConnective children are positional)"
    );
}

#[test]
fn bag_hash_distinguishes_different_content() {
    assert_ne!(
        dag_node_bag_hash(hashes(vec!["a", "b"])),
        dag_node_bag_hash(hashes(vec!["a", "c"])),
        "bag hash must distinguish different child sets"
    );
}

#[test]
fn seq_hash_distinguishes_different_content() {
    assert_ne!(
        dag_node_seq_hash(hashes(vec!["a", "b"])),
        dag_node_seq_hash(hashes(vec!["a", "c"])),
        "seq hash must distinguish different child sequences"
    );
}

#[test]
fn empty_bag_and_seq_hash_are_different() {
    assert_ne!(
        dag_node_bag_hash(hashes(vec![])),
        dag_node_seq_hash(hashes(vec![])),
        "empty bag and empty seq must have distinct sentinel hashes"
    );
}
