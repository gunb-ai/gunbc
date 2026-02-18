//! Recursive structural diff between two DAG topologies.
//!
//! Compares two [`DagTopology`] instances and classifies every node and edge
//! as Added, Removed, Changed, or Unchanged. The diff recurses into SubDag
//! children, so a node that is "unchanged at its interface" but "changed
//! internally" is classified as Changed with `structure_changed: true`.
//!
//! # Example
//!
//! ```ignore
//! let old_topo = old_dag.topology();
//! let new_topo = new_dag.topology();
//! let result = diff_topologies(&old_topo, &new_topo);
//! println!("Added {} nodes", result.added_nodes.len());
//! ```

use crate::dag_topology::{DagTopology, EdgeTopology, NodeTopology, PortTopology};
use crate::types::NodeId;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of diffing two `DagTopology` values at one level.
#[derive(Debug, Clone, Default)]
pub struct DagDiffResult {
    /// Nodes present only in the new topology.
    pub added_nodes: Vec<NodeId>,
    /// Nodes present only in the old topology.
    pub removed_nodes: Vec<NodeId>,
    /// Nodes present in both but with structural differences.
    pub changed_nodes: Vec<NodeChangeSummary>,
    /// Nodes identical in both topologies.
    pub unchanged_nodes: Vec<NodeId>,
    /// Edges present only in the new topology.
    pub added_edges: Vec<EdgeTopology>,
    /// Edges present only in the old topology.
    pub removed_edges: Vec<EdgeTopology>,
    /// Edges identical in both topologies.
    pub unchanged_edges: Vec<EdgeTopology>,
}

/// Summary of what changed for a single node.
#[derive(Debug, Clone)]
pub struct NodeChangeSummary {
    pub id: NodeId,
    /// Port-level changes (added/removed/changed ports).
    pub port_changes: Vec<PortChange>,
    /// True if the node changed from opaque to SubDag (or vice versa),
    /// or if SubDag internals changed even though the interface is identical.
    pub structure_changed: bool,
    /// Recursive diff of SubDag children (if both old and new are SubDags).
    pub child_diff: Option<Box<DagDiffResult>>,
}

/// A change to a single port.
#[derive(Debug, Clone)]
pub struct PortChange {
    pub name: String,
    pub direction: PortDirection,
    pub kind: PortChangeKind,
}

/// Whether the port is an input or output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

/// What happened to the port.
#[derive(Debug, Clone)]
pub enum PortChangeKind {
    Added,
    Removed,
    TypeChanged { old_type: String, new_type: String },
    CardinalityChanged { old: String, new: String },
}

// ---------------------------------------------------------------------------
// Diff algorithm
// ---------------------------------------------------------------------------

/// Compute a recursive structural diff between two DAG topologies.
pub fn diff_topologies(old: &DagTopology, new: &DagTopology) -> DagDiffResult {
    let mut result = DagDiffResult::default();

    // Index old nodes by ID for O(1) lookup.
    let old_map: BTreeMap<&NodeId, &NodeTopology> = old.nodes.iter().map(|n| (&n.id, n)).collect();
    let new_map: BTreeMap<&NodeId, &NodeTopology> = new.nodes.iter().map(|n| (&n.id, n)).collect();

    // Classify nodes
    for (id, new_node) in &new_map {
        match old_map.get(id) {
            None => result.added_nodes.push((*id).clone()),
            Some(old_node) => {
                if let Some(summary) = diff_nodes(old_node, new_node) {
                    result.changed_nodes.push(summary);
                } else {
                    result.unchanged_nodes.push((*id).clone());
                }
            }
        }
    }

    for id in old_map.keys() {
        if !new_map.contains_key(id) {
            result.removed_nodes.push((*id).clone());
        }
    }

    // Classify edges
    let old_edge_set: Vec<&EdgeTopology> = old.edges.iter().collect();
    let new_edge_set: Vec<&EdgeTopology> = new.edges.iter().collect();

    for edge in &new.edges {
        if old_edge_set.iter().any(|e| edges_equal(e, edge)) {
            result.unchanged_edges.push(edge.clone());
        } else {
            result.added_edges.push(edge.clone());
        }
    }

    for edge in &old.edges {
        if !new_edge_set.iter().any(|e| edges_equal(e, edge)) {
            result.removed_edges.push(edge.clone());
        }
    }

    result
}

/// Diff two nodes with the same ID. Returns `None` if they are identical,
/// or `Some(summary)` if they differ.
fn diff_nodes(old: &NodeTopology, new: &NodeTopology) -> Option<NodeChangeSummary> {
    let mut port_changes = Vec::new();

    // Compare input ports
    diff_ports(
        &old.inputs,
        &new.inputs,
        PortDirection::Input,
        &mut port_changes,
    );

    // Compare output ports
    diff_ports(
        &old.outputs,
        &new.outputs,
        PortDirection::Output,
        &mut port_changes,
    );

    // Compare SubDag structure
    let (structure_changed, child_diff) = match (&old.children, &new.children) {
        (None, None) => (false, None),
        (Some(_), None) | (None, Some(_)) => {
            // Changed from SubDag to opaque or vice versa
            (true, None)
        }
        (Some(old_children), Some(new_children)) => {
            if old_children == new_children {
                (false, None)
            } else {
                let diff = diff_topologies(old_children, new_children);
                (true, Some(Box::new(diff)))
            }
        }
    };

    if port_changes.is_empty() && !structure_changed {
        None
    } else {
        Some(NodeChangeSummary {
            id: new.id.clone(),
            port_changes,
            structure_changed,
            child_diff,
        })
    }
}

/// Compare two sets of ports and record differences.
fn diff_ports(
    old_ports: &[PortTopology],
    new_ports: &[PortTopology],
    direction: PortDirection,
    changes: &mut Vec<PortChange>,
) {
    let old_map: BTreeMap<&str, &PortTopology> =
        old_ports.iter().map(|p| (p.name.0.as_str(), p)).collect();
    let new_map: BTreeMap<&str, &PortTopology> =
        new_ports.iter().map(|p| (p.name.0.as_str(), p)).collect();

    // Added ports
    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            changes.push(PortChange {
                name: name.to_string(),
                direction,
                kind: PortChangeKind::Added,
            });
        }
    }

    // Removed ports
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            changes.push(PortChange {
                name: name.to_string(),
                direction,
                kind: PortChangeKind::Removed,
            });
        }
    }

    // Changed ports (same name, different type or cardinality)
    for (name, new_port) in &new_map {
        if let Some(old_port) = old_map.get(name) {
            if old_port.type_id != new_port.type_id {
                changes.push(PortChange {
                    name: name.to_string(),
                    direction,
                    kind: PortChangeKind::TypeChanged {
                        old_type: old_port.type_id.0.clone(),
                        new_type: new_port.type_id.0.clone(),
                    },
                });
            } else if old_port.cardinality != new_port.cardinality {
                changes.push(PortChange {
                    name: name.to_string(),
                    direction,
                    kind: PortChangeKind::CardinalityChanged {
                        old: format!("{}", old_port.cardinality),
                        new: format!("{}", new_port.cardinality),
                    },
                });
            }
        }
    }
}

/// Check if two edges are structurally equal (ignoring index).
fn edges_equal(a: &EdgeTopology, b: &EdgeTopology) -> bool {
    a.from_node == b.from_node
        && a.from_port == b.from_port
        && a.to_node == b.to_node
        && a.to_port == b.to_port
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

impl DagDiffResult {
    /// Returns `true` if there are no changes at this level.
    pub fn is_empty(&self) -> bool {
        self.added_nodes.is_empty()
            && self.removed_nodes.is_empty()
            && self.changed_nodes.is_empty()
            && self.added_edges.is_empty()
            && self.removed_edges.is_empty()
    }

    /// Returns `true` if there are no changes at any level (recursive).
    pub fn is_unchanged(&self) -> bool {
        if !self.is_empty() {
            return false;
        }
        // Check that no changed node has a non-empty child diff
        // (this shouldn't happen since changed_nodes is empty, but be safe)
        true
    }

    /// Total number of added nodes (does not recurse).
    pub fn added_count(&self) -> usize {
        self.added_nodes.len()
    }

    /// Total number of removed nodes (does not recurse).
    pub fn removed_count(&self) -> usize {
        self.removed_nodes.len()
    }

    /// Total number of changed nodes (does not recurse).
    pub fn changed_count(&self) -> usize {
        self.changed_nodes.len()
    }

    /// Generate a summary stats line like "+2 nodes, -1 node, ~3 changed".
    pub fn stats_summary(&self) -> String {
        let mut parts = Vec::new();

        let added = self.added_count();
        if added > 0 {
            parts.push(format!(
                "+{} {}",
                added,
                if added == 1 { "node" } else { "nodes" }
            ));
        }

        let removed = self.removed_count();
        if removed > 0 {
            parts.push(format!(
                "-{} {}",
                removed,
                if removed == 1 { "node" } else { "nodes" }
            ));
        }

        let changed = self.changed_count();
        if changed > 0 {
            parts.push(format!("~{} changed", changed));
        }

        if parts.is_empty() {
            "no changes".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Classify a node ID as Added, Removed, Changed, or Unchanged.
    pub fn node_status(&self, id: &NodeId) -> NodeDiffStatus {
        if self.added_nodes.contains(id) {
            NodeDiffStatus::Added
        } else if self.removed_nodes.contains(id) {
            NodeDiffStatus::Removed
        } else if self.changed_nodes.iter().any(|c| &c.id == id) {
            NodeDiffStatus::Changed
        } else {
            NodeDiffStatus::Unchanged
        }
    }
}

/// Classification of a node's diff status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeDiffStatus {
    Added,
    Removed,
    Changed,
    Unchanged,
}

impl std::fmt::Display for PortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortDirection::Input => write!(f, "input"),
            PortDirection::Output => write!(f, "output"),
        }
    }
}

impl std::fmt::Display for PortChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortChangeKind::Added => write!(f, "added"),
            PortChangeKind::Removed => write!(f, "removed"),
            PortChangeKind::TypeChanged { old_type, new_type } => {
                write!(f, "type changed {} -> {}", old_type, new_type)
            }
            PortChangeKind::CardinalityChanged { old, new } => {
                write!(f, "cardinality changed {} -> {}", old, new)
            }
        }
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
    use crate::Dag;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    enum TestOp {
        A,
        B,
    }

    fn simple_dag() -> Dag<TestOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "Int")],
            TestOp::A,
        ));
        dag.add_node(Node::opaque(
            "n2",
            vec![Port::scalar("x", "Int")],
            vec![Port::scalar("y", "Bool")],
            TestOp::B,
        ));
        dag.add_edge(Edge::new("n1", "out", "n2", "x"));
        dag
    }

    #[test]
    fn test_identical_dags() {
        let dag = simple_dag();
        let topo = dag.topology();
        let result = diff_topologies(&topo, &topo);

        assert!(result.is_empty());
        assert_eq!(result.unchanged_nodes.len(), 2);
        assert_eq!(result.unchanged_edges.len(), 1);
    }

    #[test]
    fn test_added_node() {
        let old = simple_dag();
        let mut new = simple_dag();
        new.add_node(Node::opaque("n3", vec![], vec![], TestOp::A));

        let result = diff_topologies(&old.topology(), &new.topology());

        assert_eq!(result.added_nodes, vec![NodeId::from("n3")]);
        assert_eq!(result.removed_nodes.len(), 0);
        assert_eq!(result.unchanged_nodes.len(), 2);
    }

    #[test]
    fn test_removed_node() {
        let old = simple_dag();
        let mut new: Dag<TestOp> = Dag::new();
        new.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "Int")],
            TestOp::A,
        ));

        let result = diff_topologies(&old.topology(), &new.topology());

        assert_eq!(result.removed_nodes, vec![NodeId::from("n2")]);
        assert_eq!(result.added_nodes.len(), 0);
        assert_eq!(result.unchanged_nodes.len(), 1);
    }

    #[test]
    fn test_changed_port_type() {
        let old = simple_dag();
        let mut new: Dag<TestOp> = Dag::new();
        new.add_node(Node::opaque(
            "n1",
            vec![Port::scalar("in", "Bool")], // Changed from String to Bool
            vec![Port::scalar("out", "Int")],
            TestOp::A,
        ));
        new.add_node(Node::opaque(
            "n2",
            vec![Port::scalar("x", "Int")],
            vec![Port::scalar("y", "Bool")],
            TestOp::B,
        ));
        new.add_edge(Edge::new("n1", "out", "n2", "x"));

        let result = diff_topologies(&old.topology(), &new.topology());

        assert_eq!(result.changed_nodes.len(), 1);
        assert_eq!(result.changed_nodes[0].id.0, "n1");
        assert_eq!(result.changed_nodes[0].port_changes.len(), 1);
        assert!(matches!(
            result.changed_nodes[0].port_changes[0].kind,
            PortChangeKind::TypeChanged { .. }
        ));
    }

    #[test]
    fn test_added_edge() {
        let old = simple_dag();
        let mut new = simple_dag();
        new.add_edge(Edge::new("n1", "out", "n2", "y"));

        let result = diff_topologies(&old.topology(), &new.topology());

        assert_eq!(result.added_edges.len(), 1);
        assert_eq!(result.removed_edges.len(), 0);
    }

    #[test]
    fn test_subdag_internal_change() {
        // Old: SubDag with 1 child
        let mut inner_old: Dag<TestOp> = Dag::new();
        inner_old.add_node(Node::opaque(
            "child1",
            vec![Port::scalar("x", "String")],
            vec![Port::scalar("y", "String")],
            TestOp::A,
        ));

        let mut old: Dag<TestOp> = Dag::new();
        old.add_node(Node::subdag("sub", inner_old));

        // New: SubDag with 2 children (internal change)
        let mut inner_new: Dag<TestOp> = Dag::new();
        inner_new.add_node(Node::opaque(
            "child1",
            vec![Port::scalar("x", "String")],
            vec![Port::scalar("y", "String")],
            TestOp::A,
        ));
        inner_new.add_node(Node::opaque(
            "child2",
            vec![Port::scalar("a", "Int")],
            vec![Port::scalar("b", "Int")],
            TestOp::B,
        ));

        let mut new: Dag<TestOp> = Dag::new();
        new.add_node(Node::subdag("sub", inner_new));

        let result = diff_topologies(&old.topology(), &new.topology());

        // The SubDag node should be "changed" due to internal differences
        assert_eq!(result.changed_nodes.len(), 1);
        assert_eq!(result.changed_nodes[0].id.0, "sub");
        assert!(result.changed_nodes[0].structure_changed);

        // The child diff should show the added node
        let child_diff = result.changed_nodes[0].child_diff.as_ref().unwrap();
        assert_eq!(child_diff.added_nodes, vec![NodeId::from("child2")]);
    }

    #[test]
    fn test_stats_summary() {
        let old = simple_dag();
        let mut new = simple_dag();
        new.add_node(Node::opaque("n3", vec![], vec![], TestOp::A));

        let result = diff_topologies(&old.topology(), &new.topology());
        assert_eq!(result.stats_summary(), "+1 node");

        // Empty diff
        let empty = diff_topologies(&old.topology(), &old.topology());
        assert_eq!(empty.stats_summary(), "no changes");
    }

    #[test]
    fn test_node_status() {
        let old = simple_dag();
        let mut new = simple_dag();
        new.add_node(Node::opaque("n3", vec![], vec![], TestOp::A));

        let result = diff_topologies(&old.topology(), &new.topology());

        assert_eq!(
            result.node_status(&NodeId::from("n1")),
            NodeDiffStatus::Unchanged
        );
        assert_eq!(
            result.node_status(&NodeId::from("n3")),
            NodeDiffStatus::Added
        );
    }
}
