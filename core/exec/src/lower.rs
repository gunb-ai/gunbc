//! Lowering: flatten sub-DAGs into a single flat DAG.

use gunbc_ir::{detect_boundaries, detect_entrypoints, Dag, Edge, Node, NodeBody, NodeId, PortName};
use std::collections::HashMap;
use thiserror::Error;

/// Error during lowering.
#[derive(Debug, Error)]
pub enum LowerError {
    #[error("node '{0}' has SubDag with no export_node defined")]
    NoExportNode(String),
    #[error("SubDag '{node}' has no inner entrypoint for input port '{port}'")]
    NoInnerEntrypoint { node: String, port: String },
    #[error("SubDag '{node}' has no inner boundary for output port '{port}'")]
    NoInnerBoundary { node: String, port: String },
}

/// Mapping info for a SubDag's ports to its inner nodes.
struct SubDagMapping {
    /// Maps parent input port name -> list of (inner_node_id, inner_port_name)
    input_mappings: HashMap<PortName, Vec<(NodeId, PortName)>>,
    /// Maps parent output port name -> (inner_node_id, inner_port_name)
    output_mappings: HashMap<PortName, (NodeId, PortName)>,
}

/// Build the port mapping for a SubDag node.
fn build_subdag_mapping<T>(
    parent_node: &Node<T>,
    inner_dag: &Dag<T>,
    parent_prefix: &str,
) -> Result<SubDagMapping, LowerError> {
    let entrypoints = detect_entrypoints(inner_dag);
    let boundaries = detect_boundaries(inner_dag);

    let mut input_mappings: HashMap<PortName, Vec<(NodeId, PortName)>> = HashMap::new();
    let mut output_mappings: HashMap<PortName, (NodeId, PortName)> = HashMap::new();

    // Map parent input ports to inner entrypoints by matching port names
    for parent_port in &parent_node.inputs {
        let mut targets = Vec::new();
        for (inner_node_id, inner_port_name, _type_id) in &entrypoints.entrypoint_ports {
            if inner_port_name == &parent_port.name {
                // Prefix the inner node ID
                let prefixed_id = NodeId::new(format!("{}/{}", parent_prefix, inner_node_id.0));
                targets.push((prefixed_id, inner_port_name.clone()));
            }
        }
        if targets.is_empty() {
            return Err(LowerError::NoInnerEntrypoint {
                node: parent_prefix.to_string(),
                port: parent_port.name.0.clone(),
            });
        }
        input_mappings.insert(parent_port.name.clone(), targets);
    }

    // Map parent output ports to inner boundaries by matching port names
    for parent_port in &parent_node.outputs {
        let mut found = None;
        for (inner_node_id, inner_port_name) in &boundaries.boundary_ports {
            if inner_port_name == &parent_port.name {
                let prefixed_id = NodeId::new(format!("{}/{}", parent_prefix, inner_node_id.0));
                found = Some((prefixed_id, inner_port_name.clone()));
                break;
            }
        }
        match found {
            Some(mapping) => {
                output_mappings.insert(parent_port.name.clone(), mapping);
            }
            None => {
                return Err(LowerError::NoInnerBoundary {
                    node: parent_prefix.to_string(),
                    port: parent_port.name.0.clone(),
                });
            }
        }
    }

    Ok(SubDagMapping {
        input_mappings,
        output_mappings,
    })
}

/// Lower a DAG by flattening all SubDag nodes into Opaque nodes.
///
/// After lowering, the DAG contains only Opaque nodes and can be executed.
/// Node IDs are prefixed with the parent's ID (e.g., "parent/child").
///
/// ## SubDag Boundary Wiring
///
/// When flattening a SubDag:
/// - Edges INTO the SubDag parent are rewired to inner entrypoint nodes (by port name)
/// - Edges FROM the SubDag parent are rewired from inner boundary nodes (by port name)
/// - A single parent input may fan out to multiple inner entrypoints with the same name
pub fn lower<T: Clone>(dag: &Dag<T>) -> Result<Dag<T>, LowerError> {
    let mut result = Dag::new();
    let mut subdag_mappings: HashMap<NodeId, SubDagMapping> = HashMap::new();

    // First pass: collect nodes and build SubDag mappings
    for node in &dag.nodes {
        match &node.body {
            NodeBody::Opaque(_) => {
                // Opaque nodes pass through unchanged
                result.add_node(node.clone());
            }
            NodeBody::SubDag(subdag) => {
                // Recursively lower the sub-DAG first
                let lowered_sub = lower(subdag)?;

                // Build mapping before we modify the lowered_sub
                let mapping = build_subdag_mapping(node, &lowered_sub, &node.id.0)?;
                subdag_mappings.insert(node.id.clone(), mapping);

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

    // Second pass: rewire edges, handling SubDag boundaries
    for edge in &dag.edges {
        let from_node = dag.get_node(&edge.from_node);
        let to_node = dag.get_node(&edge.to_node);

        let from_is_subdag = from_node.map(|n| n.is_subdag()).unwrap_or(false);
        let to_is_subdag = to_node.map(|n| n.is_subdag()).unwrap_or(false);

        match (from_is_subdag, to_is_subdag) {
            // Both opaque: edge passes through unchanged
            (false, false) => {
                result.add_edge(edge.clone());
            }

            // Source is SubDag: rewire from inner boundary node
            (true, false) => {
                if let Some(mapping) = subdag_mappings.get(&edge.from_node) {
                    if let Some((inner_node, inner_port)) =
                        mapping.output_mappings.get(&edge.from_port)
                    {
                        result.add_edge(Edge::new(
                            inner_node.0.clone(),
                            inner_port.0.clone(),
                            edge.to_node.0.clone(),
                            edge.to_port.0.clone(),
                        ));
                    }
                }
            }

            // Target is SubDag: rewire to inner entrypoint node(s)
            (false, true) => {
                if let Some(mapping) = subdag_mappings.get(&edge.to_node) {
                    if let Some(targets) = mapping.input_mappings.get(&edge.to_port) {
                        // Fan out to all inner entrypoints with matching name
                        for (inner_node, inner_port) in targets {
                            result.add_edge(Edge::new(
                                edge.from_node.0.clone(),
                                edge.from_port.0.clone(),
                                inner_node.0.clone(),
                                inner_port.0.clone(),
                            ));
                        }
                    }
                }
            }

            // Both SubDag: rewire from inner boundary to inner entrypoints
            (true, true) => {
                let from_mapping = subdag_mappings.get(&edge.from_node);
                let to_mapping = subdag_mappings.get(&edge.to_node);

                if let (Some(from_map), Some(to_map)) = (from_mapping, to_mapping) {
                    if let Some((from_inner, from_port)) =
                        from_map.output_mappings.get(&edge.from_port)
                    {
                        if let Some(targets) = to_map.input_mappings.get(&edge.to_port) {
                            for (to_inner, to_port) in targets {
                                result.add_edge(Edge::new(
                                    from_inner.0.clone(),
                                    from_port.0.clone(),
                                    to_inner.0.clone(),
                                    to_port.0.clone(),
                                ));
                            }
                        }
                    }
                }
            }
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
        // Create a sub-DAG with input and output ports that match parent
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "inner",
            vec![port("in", "S")],  // entrypoint
            vec![port("out", "S")], // boundary
            (),
        ));

        // Create the parent DAG with a SubDag node
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag(
            "wrapper",
            vec![port("in", "S")],
            vec![port("out", "S")],
            subdag,
        ));

        let lowered = lower(&dag).unwrap();

        // The inner node should be prefixed with "wrapper/"
        assert_eq!(lowered.nodes.len(), 1);
        assert_eq!(lowered.nodes[0].id.0, "wrapper/inner");
    }

    #[test]
    fn test_lower_subdag_with_edge_into() {
        // Create a sub-DAG
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "inner",
            vec![port("data", "S")],
            vec![port("result", "S")],
            (),
        ));

        // Create parent DAG: A -> SubDag
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::subdag(
            "wrapper",
            vec![port("data", "S")],
            vec![port("result", "S")],
            subdag,
        ));
        dag.add_edge(edge("A", "out", "wrapper", "data"));

        let lowered = lower(&dag).unwrap();

        // Should have 2 nodes
        assert_eq!(lowered.nodes.len(), 2);

        // Edge should be rewired: A -> wrapper/inner
        assert_eq!(lowered.edges.len(), 1);
        let e = &lowered.edges[0];
        assert_eq!(e.from_node.0, "A");
        assert_eq!(e.from_port.0, "out");
        assert_eq!(e.to_node.0, "wrapper/inner");
        assert_eq!(e.to_port.0, "data");
    }

    #[test]
    fn test_lower_subdag_with_edge_from() {
        // Create a sub-DAG
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "inner",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));

        // Create parent DAG: SubDag -> B
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag(
            "wrapper",
            vec![port("in", "S")],
            vec![port("out", "S")],
            subdag,
        ));
        dag.add_node(Node::opaque("B", vec![port("data", "S")], vec![], ()));
        dag.add_edge(edge("wrapper", "out", "B", "data"));

        let lowered = lower(&dag).unwrap();

        // Should have 2 nodes
        assert_eq!(lowered.nodes.len(), 2);

        // Edge should be rewired: wrapper/inner -> B
        assert_eq!(lowered.edges.len(), 1);
        let e = &lowered.edges[0];
        assert_eq!(e.from_node.0, "wrapper/inner");
        assert_eq!(e.from_port.0, "out");
        assert_eq!(e.to_node.0, "B");
        assert_eq!(e.to_port.0, "data");
    }

    #[test]
    fn test_lower_subdag_fanout() {
        // Create a sub-DAG with multiple nodes having the same input port name
        let mut subdag: Dag<()> = Dag::new();
        subdag.add_node(Node::opaque(
            "node1",
            vec![port("data", "S")],
            vec![port("out1", "S")],
            (),
        ));
        subdag.add_node(Node::opaque(
            "node2",
            vec![port("data", "S")],
            vec![port("out2", "S")],
            (),
        ));

        // Create parent DAG: A -> SubDag (should fan out to both inner nodes)
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::subdag(
            "wrapper",
            vec![port("data", "S")],
            vec![port("out1", "S"), port("out2", "S")],
            subdag,
        ));
        dag.add_edge(edge("A", "out", "wrapper", "data"));

        let lowered = lower(&dag).unwrap();

        // Should have 3 nodes
        assert_eq!(lowered.nodes.len(), 3);

        // Should have 2 edges (fanned out)
        assert_eq!(lowered.edges.len(), 2);

        // Both edges should come from A.out
        for e in &lowered.edges {
            assert_eq!(e.from_node.0, "A");
            assert_eq!(e.from_port.0, "out");
            assert!(e.to_node.0 == "wrapper/node1" || e.to_node.0 == "wrapper/node2");
            assert_eq!(e.to_port.0, "data");
        }
    }

    #[test]
    fn test_lower_subdag_to_subdag() {
        // Two SubDags connected to each other
        let mut subdag1: Dag<()> = Dag::new();
        subdag1.add_node(Node::opaque(
            "inner1",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));

        let mut subdag2: Dag<()> = Dag::new();
        subdag2.add_node(Node::opaque(
            "inner2",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag(
            "sub1",
            vec![port("in", "S")],
            vec![port("out", "S")],
            subdag1,
        ));
        dag.add_node(Node::subdag(
            "sub2",
            vec![port("in", "S")],
            vec![port("out", "S")],
            subdag2,
        ));
        dag.add_edge(edge("sub1", "out", "sub2", "in"));

        let lowered = lower(&dag).unwrap();

        // Should have 2 nodes
        assert_eq!(lowered.nodes.len(), 2);

        // Edge should connect inner nodes
        assert_eq!(lowered.edges.len(), 1);
        let e = &lowered.edges[0];
        assert_eq!(e.from_node.0, "sub1/inner1");
        assert_eq!(e.from_port.0, "out");
        assert_eq!(e.to_node.0, "sub2/inner2");
        assert_eq!(e.to_port.0, "in");
    }
}
