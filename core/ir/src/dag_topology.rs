//! Topology fingerprint for DAG structural diffing.
//!
//! `DagTopology` is a recursive, `T`-erased representation of a DAG's structure.
//! It captures node IDs, port signatures, edges, and SubDag nesting --- but not
//! the operation type `T`. This enables structural comparison across different
//! graph op types and across git commits (via JSON serialization).
//!
//! # Usage
//!
//! ```text
//! let dag = build_workspace_dag().unwrap();
//! let topo = dag.topology();
//! let json = serde_json::to_string_pretty(&topo).unwrap();
//! ```

use crate::dag::{Dag, Port};
use crate::node::NodeBody;
use crate::types::{Cardinality, NodeId, PortName, TypeId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Recursive, T-erased topology fingerprint of a DAG.
///
/// Serializable to JSON for snapshot storage and structural diffing.
/// Two `DagTopology` values are equal iff their DAGs have the same
/// node IDs, port signatures, edges, and SubDag nesting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagTopology {
    pub nodes: Vec<NodeTopology>,
    pub edges: Vec<EdgeTopology>,
}

/// Topology of a single node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeTopology {
    pub id: NodeId,
    pub inputs: Vec<PortTopology>,
    pub outputs: Vec<PortTopology>,
    /// `None` = opaque leaf node; `Some` = SubDag with recursive children.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<DagTopology>,
    /// Optional canonical kind metadata for downstream visualization/rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_kind: Option<String>,
}

/// Topology of a single port (name + type + cardinality).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortTopology {
    pub name: PortName,
    pub type_id: TypeId,
    pub cardinality: Cardinality,
}

/// Topology of a single edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EdgeTopology {
    pub from_node: NodeId,
    pub from_port: PortName,
    pub to_node: NodeId,
    pub to_port: PortName,
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

impl<T> Dag<T> {
    /// Extract a recursive, T-erased topology fingerprint.
    ///
    /// Walks all nodes and edges, recurses into `NodeBody::SubDag` children,
    /// and erases the operation type `T`. The result is serializable to JSON
    /// and comparable across different graph op types or git commits.
    pub fn topology(&self) -> DagTopology {
        self.topology_with_kind(|_| None)
    }

    /// Extract topology fingerprint with optional per-node canonical kind hints.
    pub fn topology_with_kind<F>(&self, mut kind_of: F) -> DagTopology
    where
        F: FnMut(&crate::node::Node<T>) -> Option<String>,
    {
        dag_topology_with_kind(self, &mut kind_of)
    }
}

fn dag_topology_with_kind<T, F>(dag: &Dag<T>, kind_of: &mut F) -> DagTopology
where
    F: FnMut(&crate::node::Node<T>) -> Option<String>,
{
    DagTopology {
        nodes: dag
            .nodes
            .iter()
            .map(|n| node_topology(n, kind_of))
            .collect(),
        edges: dag.edges.iter().map(edge_topology).collect(),
    }
}

fn node_topology<T, F>(node: &crate::node::Node<T>, kind_of: &mut F) -> NodeTopology
where
    F: FnMut(&crate::node::Node<T>) -> Option<String>,
{
    let canonical_kind = kind_of(node);
    let children = match &node.body {
        NodeBody::SubDag(dag) => Some(dag_topology_with_kind(dag, kind_of)),
        NodeBody::Opaque(_) => None,
    };

    NodeTopology {
        id: node.id.clone(),
        inputs: node.inputs.iter().map(port_topology).collect(),
        outputs: node.outputs.iter().map(port_topology).collect(),
        children,
        canonical_kind,
    }
}

fn port_topology(port: &Port) -> PortTopology {
    PortTopology {
        name: port.name.clone(),
        type_id: port.type_id.clone(),
        cardinality: port.cardinality,
    }
}

fn edge_topology(edge: &crate::dag::Edge) -> EdgeTopology {
    EdgeTopology {
        from_node: edge.from_node.clone(),
        from_port: edge.from_port.clone(),
        to_node: edge.to_node.clone(),
        to_port: edge.to_port.clone(),
    }
}

// ---------------------------------------------------------------------------
// Convenience
// ---------------------------------------------------------------------------

impl NodeTopology {
    /// Returns `true` if this node is a SubDag (has children).
    pub fn is_subdag(&self) -> bool {
        self.children.is_some()
    }
}

impl DagTopology {
    /// Get a node by ID.
    pub fn get_node(&self, id: &NodeId) -> Option<&NodeTopology> {
        self.nodes.iter().find(|n| &n.id == id)
    }

    /// Total number of nodes at this level (does not recurse).
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of edges at this level (does not recurse).
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Total number of nodes across all levels (recurses into SubDags).
    pub fn total_node_count(&self) -> usize {
        let mut count = self.nodes.len();
        for node in &self.nodes {
            if let Some(ref children) = node.children {
                count += children.total_node_count();
            }
        }
        count
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{Edge, Port};
    use crate::node::Node;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    enum TestOp {
        A,
        B,
    }

    #[test]
    fn test_topology_opaque_nodes() {
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "Int")],
            TestOp::A,
        ));
        dag.add_node(Node::opaque("n2", vec![], vec![], TestOp::B));
        dag.add_edge(Edge::new("n1", "out", "n2", "in"));

        let topo = dag.topology();
        assert_eq!(topo.nodes.len(), 2);
        assert_eq!(topo.edges.len(), 1);

        let n1 = topo.get_node(&"n1".into()).unwrap();
        assert!(!n1.is_subdag());
        assert_eq!(n1.inputs.len(), 1);
        assert_eq!(n1.inputs[0].name.0, "in");
        assert_eq!(n1.inputs[0].type_id.0, "String");
        assert_eq!(n1.inputs[0].cardinality, Cardinality::ONE);

        let e = &topo.edges[0];
        assert_eq!(e.from_node.0, "n1");
        assert_eq!(e.from_port.0, "out");
        assert_eq!(e.to_node.0, "n2");
        assert_eq!(e.to_port.0, "in");
    }

    #[test]
    fn test_topology_subdag_recursive() {
        let mut inner: Dag<TestOp> = Dag::new();
        inner.add_node(Node::opaque(
            "child1",
            vec![Port::scalar("x", "String")],
            vec![Port::scalar("y", "String")],
            TestOp::A,
        ));

        let mut outer: Dag<TestOp> = Dag::new();
        outer.add_node(Node::subdag("sub", inner));

        let topo = outer.topology();
        assert_eq!(topo.nodes.len(), 1);

        let sub = &topo.nodes[0];
        assert!(sub.is_subdag());
        let children = sub.children.as_ref().unwrap();
        assert_eq!(children.nodes.len(), 1);
        assert_eq!(children.nodes[0].id.0, "child1");
    }

    #[test]
    fn test_topology_equality() {
        let mut dag1: Dag<TestOp> = Dag::new();
        dag1.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "String")],
            vec![],
            TestOp::A,
        ));

        let mut dag2: Dag<TestOp> = Dag::new();
        dag2.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "String")],
            vec![],
            TestOp::B, // Different op, but topology should be equal
        ));

        assert_eq!(dag1.topology(), dag2.topology());
    }

    #[test]
    fn test_topology_inequality_different_port() {
        let mut dag1: Dag<TestOp> = Dag::new();
        dag1.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "String")],
            vec![],
            TestOp::A,
        ));

        let mut dag2: Dag<TestOp> = Dag::new();
        dag2.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "Int")], // Different type
            vec![],
            TestOp::A,
        ));

        assert_ne!(dag1.topology(), dag2.topology());
    }

    #[test]
    fn test_total_node_count() {
        let mut inner: Dag<TestOp> = Dag::new();
        inner.add_node(Node::opaque("c1", vec![], vec![], TestOp::A));
        inner.add_node(Node::opaque("c2", vec![], vec![], TestOp::B));

        let mut outer: Dag<TestOp> = Dag::new();
        outer.add_node(Node::opaque("n1", vec![], vec![], TestOp::A));
        outer.add_node(Node::subdag("sub", inner));

        let topo = outer.topology();
        assert_eq!(topo.node_count(), 2); // n1, sub
        assert_eq!(topo.total_node_count(), 4); // n1, sub, c1, c2
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut inner: Dag<TestOp> = Dag::new();
        inner.add_node(Node::opaque(
            "child",
            vec![Port::optional("x", "String")],
            vec![Port::list("y", "Int")],
            TestOp::A,
        ));

        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::subdag("sub", inner));
        dag.add_node(Node::opaque(
            "leaf",
            vec![Port::scalar("a", "Bool")],
            vec![],
            TestOp::B,
        ));
        dag.add_edge(Edge::new("sub", "y", "leaf", "a"));

        let topo = dag.topology();
        let json = serde_json::to_string_pretty(&topo).unwrap();
        let back: DagTopology = serde_json::from_str(&json).unwrap();
        assert_eq!(topo, back);
    }
}
