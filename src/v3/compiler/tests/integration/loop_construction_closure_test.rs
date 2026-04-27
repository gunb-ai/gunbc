//! B5 — Loop construction-closure structural receipt.
//!
//! Audit-derived invariant (W-B5, 2026-04-27): every `Behavior::Loop` in a
//! lowered Dag originates from recursive-function lowering. The two production
//! construction sites are `lower.rs::finalize_mutual_clusters` (mutual-recursion
//! cluster, `LoopBound::Descent`) and `lower.rs::lower_fn_item_expr_body`
//! (single recursive function, `LoopBound::Cardinality`). The structural
//! signature of recursive-function lowering is: the loop's `output` port is the
//! `value` port of some `Behavior::Bind` node (the function body's binding).
//!
//! This test is the closure-holds receipt that retires the speculative
//! `LoopKind` marker — closure is structurally observable, no marker needed.

use crate::common::cached_compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, LoopBound, LoopNode};

fn loops(dag: &Dag) -> Vec<&LoopNode> {
    dag.nodes().iter().filter_map(Behavior::as_loop).collect()
}

fn loop_output_is_bind_value(dag: &Dag, loop_node: &LoopNode) -> bool {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .any(|bind| bind.value == loop_node.output)
}

#[test]
fn every_loop_node_originates_from_recursive_function_lowering() {
    let src = "\
type IntList = Empty | Cons { head: Int, tail: IntList }

fn count(list: IntList) -> Int = match list {
    Empty => 0,
    Cons(payload) => 1 + count(payload.tail),
}

fn even(list: IntList) -> Bool = match list {
    Empty => true,
    Cons(payload) => odd(payload.tail),
}

fn odd(list: IntList) -> Bool = match list {
    Empty => false,
    Cons(payload) => even(payload.tail),
}
";
    let dag = cached_compile_to_dag(src, "loop_construction_closure_receipt.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "fixture should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let loop_nodes = loops(&dag);
    assert!(
        !loop_nodes.is_empty(),
        "fixture must produce at least one Behavior::Loop, otherwise the closure assertion is vacuous"
    );

    let mut saw_cardinality = false;
    let mut saw_descent = false;
    for loop_node in &loop_nodes {
        assert!(
            loop_output_is_bind_value(&dag, loop_node),
            "Behavior::Loop {:?} output port {:?} is not the value of any Behavior::Bind \
             — would indicate a construction site outside recursive-function lowering",
            loop_node.id,
            loop_node.output
        );
        match loop_node.bound {
            LoopBound::Cardinality { .. } => saw_cardinality = true,
            LoopBound::Descent { cluster } => {
                saw_descent = true;
                assert!(
                    (cluster.raw() as usize) < dag.clusters().len(),
                    "LoopBound::Descent cluster id {:?} must reference a real cluster",
                    cluster
                );
            }
        }
    }
    assert!(
        saw_cardinality,
        "fixture's single-recursive `count` should produce a LoopBound::Cardinality"
    );
    assert!(
        saw_descent,
        "fixture's mutual-recursive `even`/`odd` should produce a LoopBound::Descent"
    );
}
