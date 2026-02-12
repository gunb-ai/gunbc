//! Topological sorting for DAG execution.

use gunbc_ir::{Dag, NodeId};
use std::collections::{HashMap, VecDeque};

/// Topologically sort the nodes in a DAG using Kahn's algorithm.
///
/// Returns nodes in an order where dependencies come before dependents.
///
/// # Panics
///
/// Panics if any edge references a node ID that is not present in `dag.nodes`.
/// To propagate errors instead, this function would need to return
/// `Result<Vec<NodeId>, ExecError>`, which would require updating callers in:
/// - `core/exec/src/execute.rs` (`simulate`, `execute_flat`, and parallel executor)
/// - `core/exec/src/execute.rs` (`compute_critical_path` — also returns `Vec<NodeId>`)
/// - `core/exec/src/display.rs` (`run_with_progress` — does not return `Result`)
/// - `core/codegen/src/testgen/codegen.rs`
pub fn topo_sort<T>(dag: &Dag<T>) -> Vec<NodeId> {
    let node_ids: Vec<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|id| (*id, 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = node_ids.iter().map(|id| (*id, Vec::new())).collect();

    for edge in &dag.edges {
        *in_degree.get_mut(edge.to_node.0.as_str()).unwrap() += 1;
        adj.get_mut(edge.from_node.0.as_str())
            .unwrap()
            .push(&edge.to_node.0);
    }

    // Start with nodes that have no incoming edges
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&id, _)| id)
        .collect();

    // Sort initial queue for deterministic ordering
    let mut initial: Vec<&str> = queue.drain(..).collect();
    initial.sort();
    queue.extend(initial);

    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        result.push(NodeId::new(id));
        if let Some(neighbors) = adj.get(id) {
            let mut next = Vec::new();
            for &neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    next.push(neighbor);
                }
            }
            next.sort();
            queue.extend(next);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{build::*, Dag, Node};

    #[test]
    fn test_topo_sort_simple_chain() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));
        dag.add_node(Node::opaque("C", vec![port("in", "S")], vec![], ()));
        dag.add_edge(edge("A", "out", "B", "in"));
        dag.add_edge(edge("B", "out", "C", "in"));

        let order = topo_sort(&dag);

        assert_eq!(order.len(), 3);
        assert_eq!(order[0].0, "A");
        assert_eq!(order[1].0, "B");
        assert_eq!(order[2].0, "C");
    }

    #[test]
    fn test_topo_sort_independent_nodes() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("C", vec![], vec![], ()));
        dag.add_node(Node::opaque("A", vec![], vec![], ()));
        dag.add_node(Node::opaque("B", vec![], vec![], ()));

        let order = topo_sort(&dag);

        // Should be sorted alphabetically when there are no dependencies
        assert_eq!(order.len(), 3);
        assert_eq!(order[0].0, "A");
        assert_eq!(order[1].0, "B");
        assert_eq!(order[2].0, "C");
    }
}
