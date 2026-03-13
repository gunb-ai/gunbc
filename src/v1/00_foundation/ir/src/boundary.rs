//! Boundary detection: find outputs that leave the DAG (world writes).
//!
//! The core insight: **unconnected output ports are boundaries**.
//! Data leaving the DAG necessarily goes to the world.

use crate::dag::Dag;
use crate::types::{NodeId, PortName};
use std::collections::HashSet;

/// Information about DAG boundaries.
#[derive(Debug, Clone, Default)]
pub struct BoundaryInfo {
    /// Nodes that have at least one boundary output
    pub boundary_nodes: Vec<NodeId>,
    /// Specific (node, port) pairs that are boundaries
    pub boundary_ports: Vec<(NodeId, PortName)>,
}

impl BoundaryInfo {
    /// Check if a node is a boundary node.
    pub fn is_boundary_node(&self, node_id: &NodeId) -> bool {
        self.boundary_nodes.iter().any(|n| n == node_id)
    }

    /// Check if a specific port is a boundary.
    pub fn is_boundary_port(&self, node_id: &NodeId, port_name: &PortName) -> bool {
        self.boundary_ports
            .iter()
            .any(|(n, p)| n == node_id && p == port_name)
    }

    /// Get all boundary ports for a specific node.
    pub fn ports_for_node(&self, node_id: &NodeId) -> Vec<&PortName> {
        self.boundary_ports
            .iter()
            .filter(|(n, _)| n == node_id)
            .map(|(_, p)| p)
            .collect()
    }
}

/// Detect boundaries in a DAG.
///
/// A boundary is an output port that has no downstream edge —
/// data leaving this port exits the DAG universe and necessarily
/// goes to the world.
///
/// # Example
///
/// ```
/// use gunbc_ir::{Dag, Node, Port, Edge, NodeBody, detect_boundaries};
///
/// let mut dag: Dag<()> = Dag::new();
///
/// // Node A outputs to Node B (connected)
/// dag.add_node(Node::opaque("A", vec![], vec![Port::new("out", "String")], ()));
/// dag.add_node(Node::opaque("B", vec![Port::new("in", "String")], vec![Port::new("result", "String")], ()));
/// dag.add_edge(Edge::new("A", "out", "B", "in"));
///
/// let boundaries = detect_boundaries(&dag);
///
/// // B's "result" port is unconnected — it's a boundary
/// assert!(boundaries.is_boundary_port(&"B".into(), &"result".into()));
/// // A's "out" port is connected — not a boundary
/// assert!(!boundaries.is_boundary_port(&"A".into(), &"out".into()));
/// ```
pub fn detect_boundaries<T>(dag: &Dag<T>) -> BoundaryInfo {
    // Collect all (from_node, from_port) pairs that are sources of edges
    let connected: HashSet<(NodeId, PortName)> = dag
        .edges
        .iter()
        .filter(|e| e.kind.carries_data())
        .map(|e| (e.from_node.clone(), e.from_port.clone()))
        .collect();

    // Find all output ports that are NOT connected
    let boundary_ports: Vec<(NodeId, PortName)> = dag
        .nodes
        .iter()
        .flat_map(|n| {
            n.outputs
                .iter()
                .map(|p| (n.id.clone(), p.name.clone()))
                .filter(|port| !connected.contains(port))
        })
        .collect();

    // Derive unique boundary nodes
    let mut boundary_nodes: Vec<NodeId> = boundary_ports
        .iter()
        .map(|(n, _)| n.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    boundary_nodes.sort_by(|a, b| a.0.cmp(&b.0));

    BoundaryInfo {
        boundary_nodes,
        boundary_ports,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::build::{edge, port};
    use crate::node::Node;

    #[test]
    fn test_single_node_all_outputs_are_boundaries() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "single",
            vec![],
            vec![port("out1", "String"), port("out2", "Int")],
            (),
        ));

        let boundaries = detect_boundaries(&dag);

        assert_eq!(boundaries.boundary_nodes.len(), 1);
        assert_eq!(boundaries.boundary_ports.len(), 2);
        assert!(boundaries.is_boundary_port(&"single".into(), &"out1".into()));
        assert!(boundaries.is_boundary_port(&"single".into(), &"out2".into()));
    }

    #[test]
    fn test_connected_port_not_boundary() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "String")], ()));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        let boundaries = detect_boundaries(&dag);

        // A.out is connected -> not a boundary
        assert!(!boundaries.is_boundary_port(&"A".into(), &"out".into()));
        // B.result is unconnected -> is a boundary
        assert!(boundaries.is_boundary_port(&"B".into(), &"result".into()));
        // Only B is a boundary node
        assert_eq!(boundaries.boundary_nodes.len(), 1);
        assert!(boundaries.is_boundary_node(&"B".into()));
    }

    #[test]
    fn test_pipeline_only_last_is_boundary() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque("A", vec![], vec![port("out", "S")], ()));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));
        dag.add_node(Node::opaque(
            "C",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));
        dag.add_edge(edge("B", "out", "C", "in"));

        let boundaries = detect_boundaries(&dag);

        assert_eq!(boundaries.boundary_nodes.len(), 1);
        assert!(boundaries.is_boundary_node(&"C".into()));
        assert!(boundaries.is_boundary_port(&"C".into(), &"out".into()));
    }
}
