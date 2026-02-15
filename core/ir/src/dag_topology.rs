//! Topology fingerprint for DAG structural diffing.
//!
//! `DagTopology` is a recursive, `T`-erased representation of a DAG's structure.
//! It captures node IDs, port signatures, edges, and SubDag nesting --- but not
//! the operation type `T`. This enables structural comparison across different
//! graph op types and across git commits (via JSON serialization).
//!
//! # Usage
//!
//! ```ignore
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
        DagTopology {
            nodes: self.nodes.iter().map(|n| node_topology(n)).collect(),
            edges: self.edges.iter().map(edge_topology).collect(),
        }
    }
}

fn node_topology<T>(node: &crate::node::Node<T>) -> NodeTopology {
    let children = match &node.body {
        NodeBody::SubDag(dag) => Some(dag.topology()),
        NodeBody::Opaque(_) => None,
    };

    NodeTopology {
        id: node.id.clone(),
        inputs: node.inputs.iter().map(port_topology).collect(),
        outputs: node.outputs.iter().map(port_topology).collect(),
        children,
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

    /// Group transport triplets (`prepare_X`, `execute_X`, `parse_X`) into
    /// SubDag nodes, applied recursively at every nesting level.
    ///
    /// This structural transformation detects the standard transport triplet
    /// pattern and wraps each matched group into a SubDag node named `X`.
    /// The SubDag's interface is inferred from external edges:
    ///
    /// - **Inputs**: prepare's input ports + execute's resource input ports
    /// - **Outputs**: parse's output ports
    ///
    /// Internal edges (prepare→execute, execute→parse) move into the SubDag.
    /// External edges are rewired to the group node.
    pub fn group_transport_triplets(&mut self) {
        use std::collections::{HashMap, HashSet};

        // 1. Recurse into existing SubDag children first.
        for node in &mut self.nodes {
            if let Some(ref mut children) = node.children {
                children.group_transport_triplets();
            }
        }

        // 2. Find triplet groups at this level.
        let mut groups: Vec<(String, NodeId, NodeId, NodeId)> = Vec::new();
        let mut claimed: HashSet<String> = HashSet::new();

        // Build a quick lookup set of node IDs at this level.
        let node_ids: HashSet<String> = self.nodes.iter().map(|n| n.id.0.clone()).collect();

        for node in &self.nodes {
            if let Some(suffix) = node.id.0.strip_prefix("prepare_") {
                let execute_id = format!("execute_{suffix}");
                let parse_id = format!("parse_{suffix}");
                if node_ids.contains(&execute_id)
                    && node_ids.contains(&parse_id)
                    && !claimed.contains(&node.id.0)
                {
                    claimed.insert(node.id.0.clone());
                    claimed.insert(execute_id.clone());
                    claimed.insert(parse_id.clone());
                    groups.push((
                        suffix.to_string(),
                        node.id.clone(),
                        NodeId::new(&execute_id),
                        NodeId::new(&parse_id),
                    ));
                }
            }
        }

        if groups.is_empty() {
            return;
        }

        // 3. Build a node-ID → group-name lookup for edge rewiring.
        let mut node_to_group: HashMap<String, String> = HashMap::new();
        for (group_name, prep, exec, parse) in &groups {
            node_to_group.insert(prep.0.clone(), group_name.clone());
            node_to_group.insert(exec.0.clone(), group_name.clone());
            node_to_group.insert(parse.0.clone(), group_name.clone());
        }

        // 4. Extract grouped nodes by ID for SubDag children construction.
        let mut node_map: HashMap<String, NodeTopology> = HashMap::new();
        for node in self.nodes.drain(..) {
            node_map.insert(node.id.0.clone(), node);
        }

        // 5. Partition edges into internal (within a group) vs external.
        let mut internal_edges: HashMap<String, Vec<EdgeTopology>> = HashMap::new();
        let mut external_edges: Vec<EdgeTopology> = Vec::new();

        for edge in self.edges.drain(..) {
            let from_group = node_to_group.get(&edge.from_node.0);
            let to_group = node_to_group.get(&edge.to_node.0);

            match (from_group, to_group) {
                (Some(fg), Some(tg)) if fg == tg => {
                    // Both endpoints in the same group → internal edge.
                    internal_edges
                        .entry(fg.clone())
                        .or_default()
                        .push(edge);
                }
                (Some(fg), Some(_tg)) => {
                    // Cross-group edge: rewrite both endpoints.
                    external_edges.push(EdgeTopology {
                        from_node: NodeId::new(fg),
                        from_port: edge.from_port,
                        to_node: NodeId::new(_tg),
                        to_port: edge.to_port,
                    });
                }
                (Some(fg), None) => {
                    // From grouped node to external node.
                    external_edges.push(EdgeTopology {
                        from_node: NodeId::new(fg),
                        from_port: edge.from_port,
                        to_node: edge.to_node,
                        to_port: edge.to_port,
                    });
                }
                (None, Some(tg)) => {
                    // From external node to grouped node.
                    external_edges.push(EdgeTopology {
                        from_node: edge.from_node,
                        from_port: edge.from_port,
                        to_node: NodeId::new(tg),
                        to_port: edge.to_port,
                    });
                }
                (None, None) => {
                    // Neither endpoint is grouped → pass through.
                    external_edges.push(edge);
                }
            }
        }

        // 6. Build SubDag nodes for each group + collect ungrouped nodes.
        let mut new_nodes: Vec<NodeTopology> = Vec::new();

        for (group_name, prep_id, exec_id, parse_id) in &groups {
            let prep = node_map.remove(&prep_id.0).unwrap();
            let exec = node_map.remove(&exec_id.0).unwrap();
            let parse = node_map.remove(&parse_id.0).unwrap();

            // Compute the SubDag's external interface.
            //
            // Inputs: all of prepare's inputs + execute's resource inputs
            // (ports prefixed with "res:" that aren't wired from prepare).
            let mut inputs: Vec<PortTopology> = prep.inputs.clone();
            for port in &exec.inputs {
                if port.name.0.starts_with("res:") {
                    inputs.push(port.clone());
                }
            }

            // Outputs: all of parse's outputs.
            let outputs: Vec<PortTopology> = parse.outputs.clone();

            let children = DagTopology {
                nodes: vec![prep, exec, parse],
                edges: internal_edges.remove(group_name).unwrap_or_default(),
            };

            new_nodes.push(NodeTopology {
                id: NodeId::new(group_name),
                inputs,
                outputs,
                children: Some(children),
            });
        }

        // 7. Re-add ungrouped nodes (preserving original order).
        // `node_map` now only contains ungrouped nodes.
        // We can't preserve original order easily from a HashMap,
        // so we sort by ID for determinism.
        let mut ungrouped: Vec<NodeTopology> = node_map.into_values().collect();
        ungrouped.sort_by(|a, b| a.id.0.cmp(&b.id.0));

        new_nodes.extend(ungrouped);
        self.nodes = new_nodes;

        // 8. Deduplicate external edges (grouping may collapse parallel edges).
        external_edges.sort_by(|a, b| {
            (&a.from_node.0, &a.from_port.0, &a.to_node.0, &a.to_port.0).cmp(&(
                &b.from_node.0,
                &b.from_port.0,
                &b.to_node.0,
                &b.to_port.0,
            ))
        });
        external_edges.dedup();
        self.edges = external_edges;
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
    fn test_group_transport_triplets() {
        // Build a flat DAG with a transport triplet pattern:
        // prepare_fetch → execute_fetch → parse_fetch, plus a standalone node.
        let mut dag: Dag<TestOp> = Dag::new();
        dag.add_node(Node::opaque(
            "prepare_fetch",
            vec![Port::scalar("url", "String")],
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("skip", "Bool"),
            ],
            TestOp::A,
        ));
        dag.add_node(Node::opaque(
            "execute_fetch",
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("skip", "Bool"),
                Port::scalar("res:file", "FilesystemHandle"),
            ],
            vec![Port::scalar("response", "TransportResponse")],
            TestOp::A,
        ));
        dag.add_node(Node::opaque(
            "parse_fetch",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("data", "String")],
            TestOp::B,
        ));
        dag.add_node(Node::opaque(
            "standalone",
            vec![Port::scalar("data", "String")],
            vec![],
            TestOp::B,
        ));

        // Internal triplet edges.
        dag.add_edge(Edge::new("prepare_fetch", "request", "execute_fetch", "request"));
        dag.add_edge(Edge::new("prepare_fetch", "skip", "execute_fetch", "skip"));
        dag.add_edge(Edge::new("execute_fetch", "response", "parse_fetch", "response"));
        // External edge: parse_fetch → standalone.
        dag.add_edge(Edge::new("parse_fetch", "data", "standalone", "data"));

        let mut topo = dag.topology();
        assert_eq!(topo.node_count(), 4);
        assert_eq!(topo.edge_count(), 4);

        topo.group_transport_triplets();

        // Should now have 2 nodes: "fetch" (SubDag) and "standalone".
        assert_eq!(topo.node_count(), 2);

        let fetch = topo.get_node(&"fetch".into()).unwrap();
        assert!(fetch.is_subdag());
        let children = fetch.children.as_ref().unwrap();
        assert_eq!(children.nodes.len(), 3);
        assert_eq!(children.edges.len(), 3); // internal edges

        // SubDag interface: inputs = prepare's inputs + execute's res: ports.
        assert_eq!(fetch.inputs.len(), 2); // url + res:file
        assert!(fetch.inputs.iter().any(|p| p.name.0 == "url"));
        assert!(fetch.inputs.iter().any(|p| p.name.0 == "res:file"));
        // SubDag outputs: parse's outputs.
        assert_eq!(fetch.outputs.len(), 1);
        assert_eq!(fetch.outputs[0].name.0, "data");

        let standalone = topo.get_node(&"standalone".into()).unwrap();
        assert!(!standalone.is_subdag());

        // External edge should be rewired: fetch → standalone.
        assert_eq!(topo.edges.len(), 1);
        assert_eq!(topo.edges[0].from_node.0, "fetch");
        assert_eq!(topo.edges[0].to_node.0, "standalone");
    }

    #[test]
    fn test_group_transport_triplets_recursive() {
        // Build an outer DAG with a SubDag that contains a triplet.
        let mut inner: Dag<TestOp> = Dag::new();
        inner.add_node(Node::opaque(
            "prepare_write",
            vec![Port::scalar("content", "String")],
            vec![Port::scalar("request", "TransportRequest"), Port::scalar("skip", "Bool")],
            TestOp::A,
        ));
        inner.add_node(Node::opaque(
            "execute_write",
            vec![Port::scalar("request", "TransportRequest"), Port::scalar("skip", "Bool")],
            vec![Port::scalar("response", "TransportResponse")],
            TestOp::A,
        ));
        inner.add_node(Node::opaque(
            "parse_write",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("ok", "Bool")],
            TestOp::B,
        ));
        inner.add_edge(Edge::new("prepare_write", "request", "execute_write", "request"));
        inner.add_edge(Edge::new("prepare_write", "skip", "execute_write", "skip"));
        inner.add_edge(Edge::new("execute_write", "response", "parse_write", "response"));

        let mut outer: Dag<TestOp> = Dag::new();
        outer.add_node(Node::subdag("tool", inner));

        let mut topo = outer.topology();
        // Before grouping: tool has 3 flat children.
        assert_eq!(topo.nodes[0].children.as_ref().unwrap().nodes.len(), 3);

        topo.group_transport_triplets();

        // After grouping: tool's children should have 1 SubDag "write".
        let tool = &topo.nodes[0];
        let children = tool.children.as_ref().unwrap();
        assert_eq!(children.nodes.len(), 1);
        assert_eq!(children.nodes[0].id.0, "write");
        assert!(children.nodes[0].is_subdag());
        assert_eq!(children.nodes[0].children.as_ref().unwrap().nodes.len(), 3);
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
