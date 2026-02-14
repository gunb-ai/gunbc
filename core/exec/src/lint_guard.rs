//! DAG injection utility for lint guard nodes.
//!
//! `inject_lint_guard` adds a `lint_check` node to a DAG and wires it as a
//! blocking dependency to all existing root nodes. The graph structure itself
//! models WHY downstream nodes must wait — they depend on `lint_check.done`
//! via their `_lint_guard` input port.

use gunbc_ir::node::Node;
use gunbc_ir::{Dag, Edge, NodeId, Port};
use std::collections::HashSet;

/// Inject a lint guard node into a DAG as a blocking dependency.
///
/// This mutates the DAG in place:
/// 1. Finds all root nodes (nodes that are not the target of any edge)
/// 2. Adds a `_lint_guard: Bool` input port to each root node
/// 3. Adds a `lint_check` opaque node with output port `done: Bool`
/// 4. Wires edges from `lint_check.done` to each root's `_lint_guard`
///
/// After injection, the parallel executor will not start any original root
/// node until `lint_check` completes — the dependency is enforced by real
/// edges, not by a separate execution path.
///
/// The caller provides `lint_op: T` — the operation value for the lint_check
/// node. This is typically a `LintCheck` variant of the tool's op enum that
/// delegates to `execute_lint_check()`.
pub fn inject_lint_guard<T>(dag: &mut Dag<T>, lint_op: T) {
    // Find root nodes: nodes that are not the target of any edge.
    let targets: HashSet<NodeId> = dag.edges.iter().map(|e| e.to_node.clone()).collect();
    let roots: Vec<NodeId> = dag
        .nodes
        .iter()
        .filter(|n| !targets.contains(&n.id))
        .map(|n| n.id.clone())
        .collect();

    // Add _lint_guard input port to each root node.
    for node in &mut dag.nodes {
        if roots.contains(&node.id) {
            node.inputs.push(Port::new("_lint_guard", "Bool"));
        }
    }

    // Add lint_check node with output port done: Bool.
    dag.nodes.push(Node::opaque(
        "lint_check",
        vec![],
        vec![Port::new("done", "Bool")],
        lint_op,
    ));

    // Wire edges from lint_check.done to each root._lint_guard.
    for root_id in &roots {
        dag.edges
            .push(Edge::new("lint_check", "done", &root_id.0, "_lint_guard"));
    }
}
