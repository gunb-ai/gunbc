//! Entrypoint detection: find inputs that enter the DAG (world reads).
//!
//! The core insight: **unconnected input ports are entrypoints**.
//! Data entering the DAG necessarily comes from the world.
//!
//! This is symmetric to boundary detection:
//! - Boundaries = outputs with no downstream edge = world writes
//! - Entrypoints = inputs with no upstream edge = world reads

use crate::dag::Dag;
use crate::types::{NodeId, PortName, TypeId};
use std::collections::HashSet;

/// Information about DAG entrypoints.
#[derive(Debug, Clone, Default)]
pub struct EntrypointInfo {
    /// Nodes that have at least one entrypoint input
    pub entrypoint_nodes: Vec<NodeId>,
    /// Specific (node, port, type) tuples that are entrypoints
    pub entrypoint_ports: Vec<(NodeId, PortName, TypeId)>,
}

impl EntrypointInfo {
    /// Check if a node is an entrypoint node.
    pub fn is_entrypoint_node(&self, node_id: &NodeId) -> bool {
        self.entrypoint_nodes.iter().any(|n| n == node_id)
    }

    /// Check if a specific port is an entrypoint.
    pub fn is_entrypoint_port(&self, node_id: &NodeId, port_name: &PortName) -> bool {
        self.entrypoint_ports
            .iter()
            .any(|(n, p, _)| n == node_id && p == port_name)
    }

    /// Get all entrypoint ports for a specific node.
    pub fn ports_for_node(&self, node_id: &NodeId) -> Vec<(&PortName, &TypeId)> {
        self.entrypoint_ports
            .iter()
            .filter(|(n, _, _)| n == node_id)
            .map(|(_, p, t)| (p, t))
            .collect()
    }

    /// Get all entrypoint ports with their types.
    pub fn all_ports(&self) -> Vec<(&NodeId, &PortName, &TypeId)> {
        self.entrypoint_ports
            .iter()
            .map(|(n, p, t)| (n, p, t))
            .collect()
    }
}

/// Detect entrypoints in a DAG.
///
/// An entrypoint is an input port that has no upstream edge —
/// data entering this port comes from outside the DAG universe,
/// necessarily from the world.
///
/// # Example
///
/// ```
/// use gunbc_ir::{Dag, Node, Port, Edge, detect_entrypoints};
///
/// let mut dag: Dag<()> = Dag::new();
///
/// // Node A has an input with no upstream (entrypoint)
/// dag.add_node(Node::opaque("A", vec![Port::new("config", "String")], vec![Port::new("out", "String")], ()));
/// dag.add_node(Node::opaque("B", vec![Port::new("in", "String")], vec![Port::new("result", "String")], ()));
/// dag.add_edge(Edge::new("A", "out", "B", "in"));
///
/// let entrypoints = detect_entrypoints(&dag);
///
/// // A's "config" port is unconnected — it's an entrypoint
/// assert!(entrypoints.is_entrypoint_port(&"A".into(), &"config".into()));
/// // B's "in" port is connected — not an entrypoint
/// assert!(!entrypoints.is_entrypoint_port(&"B".into(), &"in".into()));
/// ```
pub fn detect_entrypoints<T>(dag: &Dag<T>) -> EntrypointInfo {
    // Collect all (to_node, to_port) pairs that are targets of edges
    let connected: HashSet<(NodeId, PortName)> = dag
        .edges
        .iter()
        .map(|e| (e.to_node.clone(), e.to_port.clone()))
        .collect();

    // Find all input ports that are NOT connected
    let entrypoint_ports: Vec<(NodeId, PortName, TypeId)> = dag
        .nodes
        .iter()
        .flat_map(|n| {
            n.inputs
                .iter()
                .map(|p| (n.id.clone(), p.name.clone(), p.type_id.clone()))
                .filter(|(node_id, port_name, _)| {
                    !connected.contains(&(node_id.clone(), port_name.clone()))
                })
        })
        .collect();

    // Derive unique entrypoint nodes
    let mut entrypoint_nodes: Vec<NodeId> = entrypoint_ports
        .iter()
        .map(|(n, _, _)| n.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    entrypoint_nodes.sort_by(|a, b| a.0.cmp(&b.0));

    EntrypointInfo {
        entrypoint_nodes,
        entrypoint_ports,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::build::{edge, port};
    use crate::node::Node;

    #[test]
    fn test_single_node_all_inputs_are_entrypoints() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "single",
            vec![port("in1", "String"), port("in2", "Int")],
            vec![],
            (),
        ));

        let entrypoints = detect_entrypoints(&dag);

        assert_eq!(entrypoints.entrypoint_nodes.len(), 1);
        assert_eq!(entrypoints.entrypoint_ports.len(), 2);
        assert!(entrypoints.is_entrypoint_port(&"single".into(), &"in1".into()));
        assert!(entrypoints.is_entrypoint_port(&"single".into(), &"in2".into()));
    }

    #[test]
    fn test_connected_port_not_entrypoint() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![port("config", "String")],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "B",
            vec![port("in", "String")],
            vec![port("result", "String")],
            (),
        ));
        dag.add_edge(edge("A", "out", "B", "in"));

        let entrypoints = detect_entrypoints(&dag);

        // A.config is unconnected -> is an entrypoint
        assert!(entrypoints.is_entrypoint_port(&"A".into(), &"config".into()));
        // B.in is connected -> not an entrypoint
        assert!(!entrypoints.is_entrypoint_port(&"B".into(), &"in".into()));
        // Only A is an entrypoint node
        assert_eq!(entrypoints.entrypoint_nodes.len(), 1);
        assert!(entrypoints.is_entrypoint_node(&"A".into()));
    }

    #[test]
    fn test_pipeline_only_first_is_entrypoint() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "A",
            vec![port("in", "S")],
            vec![port("out", "S")],
            (),
        ));
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

        let entrypoints = detect_entrypoints(&dag);

        assert_eq!(entrypoints.entrypoint_nodes.len(), 1);
        assert!(entrypoints.is_entrypoint_node(&"A".into()));
        assert!(entrypoints.is_entrypoint_port(&"A".into(), &"in".into()));
    }

    #[test]
    fn test_entrypoint_includes_type() {
        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "node",
            vec![port("path", "String"), port("count", "Int")],
            vec![],
            (),
        ));

        let entrypoints = detect_entrypoints(&dag);

        // Check that types are captured
        let ports = entrypoints.all_ports();
        assert_eq!(ports.len(), 2);

        // Find the path port and check its type
        let path_port = ports.iter().find(|(_, p, _)| p.0 == "path").unwrap();
        assert_eq!(path_port.2 .0, "String");

        let count_port = ports.iter().find(|(_, p, _)| p.0 == "count").unwrap();
        assert_eq!(count_port.2 .0, "Int");
    }
}
