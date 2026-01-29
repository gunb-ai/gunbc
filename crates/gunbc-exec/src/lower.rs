//! Lowering: flatten sub-DAGs into a single flat DAG.

use gunbc_ir::{Dag, Edge, Node, NodeBody, NodeId};
use thiserror::Error;

/// Error during lowering.
#[derive(Debug, Error)]
pub enum LowerError {
    #[error("node '{0}' has SubDag with no export_node defined")]
    NoExportNode(String),
}

/// Lower a DAG by flattening all SubDag nodes into Opaque nodes.
///
/// After lowering, the DAG contains only Opaque nodes and can be executed.
/// Node IDs are prefixed with the parent's ID (e.g., "parent/child").
pub fn lower<T: Clone>(dag: &Dag<T>) -> Result<Dag<T>, LowerError> {
    let mut result = Dag::new();

    for node in &dag.nodes {
        match &node.body {
            NodeBody::Opaque(_) => {
                // Opaque nodes pass through unchanged
                result.add_node(node.clone());
            }
            NodeBody::SubDag(subdag) => {
                // Recursively lower the sub-DAG first
                let lowered_sub = lower(subdag)?;

                // Add all nodes from the sub-DAG with prefixed IDs
                for sub_node in &lowered_sub.nodes {
                    let prefixed_id = format!("{}/{}", node.id.0, sub_node.id.0);
                    let prefixed_node = Node {
                        id: NodeId::new(prefixed_id),
                        inputs: sub_node.inputs.clone(),
                        outputs: sub_node.outputs.clone(),
                        body: sub_node.body.clone(),
                    };
                    result.add_node(prefixed_node);
                }

                // Add internal edges from the sub-DAG with prefixed node IDs
                for sub_edge in &lowered_sub.edges {
                    let prefixed_edge = Edge::new(
                        format!("{}/{}", node.id.0, sub_edge.from_node.0),
                        sub_edge.from_port.0.clone(),
                        format!("{}/{}", node.id.0, sub_edge.to_node.0),
                        sub_edge.to_port.0.clone(),
                    );
                    result.add_edge(prefixed_edge);
                }
            }
        }
    }

    // Add edges from the original DAG, adjusting for SubDag nodes
    for edge in &dag.edges {
        let from_node = dag.get_node(&edge.from_node);
        let to_node = dag.get_node(&edge.to_node);

        // For SubDag nodes, we need to wire to/from the appropriate internal node
        // For now, we handle the simple case where edges connect Opaque nodes
        // TODO: Handle SubDag boundary wiring properly
        if from_node.map(|n| n.is_opaque()).unwrap_or(false)
            && to_node.map(|n| n.is_opaque()).unwrap_or(false)
        {
            result.add_edge(edge.clone());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::build::*;

    #[test]
    fn test_lower_flat_dag() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::opaque("B", vec![port("in", "S")], vec![], ()));
        dag.add_edge(edge("A", "out", "B", "in"));

        let lowered = lower(&dag).unwrap();

        assert_eq!(lowered.nodes.len(), 2);
        assert_eq!(lowered.edges.len(), 1);
    }

    #[test]
    fn test_lower_subdag() {
        // Create a sub-DAG
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque("inner", vec![], vec![port("out", "S")], ()));

        // Create the parent DAG with a SubDag node
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag(
            "wrapper",
            vec![],
            vec![port("out", "S")],
            subdag,
        ));

        let lowered = lower(&dag).unwrap();

        // The inner node should be prefixed with "wrapper/"
        assert_eq!(lowered.nodes.len(), 1);
        assert_eq!(lowered.nodes[0].id.0, "wrapper/inner");
    }
}
