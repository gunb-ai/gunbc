//! Resource types and conflict detection.
//!
//! Resources represent external state (files, locks, connections, etc.) that
//! nodes may access. When resources are passed through edges, ordering is
//! explicit. When resources are accessed "out of band" (e.g., two nodes access
//! the same file without an edge between them), conflicts must be detected.
//!
//! # Resource Model
//!
//! Resources are typed values with identity:
//! - `ResourceId`: Unique identifier for a resource instance
//! - `AccessMode`: How the resource is accessed (Read, Write, Exclusive)
//! - `ResourceOp`: Operations that interact with resources
//!
//! # Conflict Detection
//!
//! A resource conflict occurs when:
//! 1. Two nodes access the same resource (same `ResourceId`)
//! 2. At least one access is Write or Exclusive
//! 3. There is no edge ordering between the nodes (they could run in parallel)
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::resource::{ResourceId, AccessMode, detect_conflicts};
//!
//! // Node A writes to file.txt
//! // Node B reads from file.txt
//! // If no edge A → B, this is a conflict
//! let conflicts = detect_conflicts(&dag, &resource_accesses);
//! ```

use crate::dag::Dag;
use crate::types::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Unique identifier for a resource.
///
/// Resources can be files, locks, connections, or any other external state.
/// Two accesses conflict if they reference the same ResourceId.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);

impl ResourceId {
    /// Create a new resource ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create a file resource ID.
    pub fn file(path: impl Into<String>) -> Self {
        Self(format!("file:{}", path.into()))
    }

    /// Create a lock resource ID.
    pub fn lock(name: impl Into<String>) -> Self {
        Self(format!("lock:{}", name.into()))
    }

    /// Create a connection resource ID.
    pub fn connection(name: impl Into<String>) -> Self {
        Self(format!("conn:{}", name.into()))
    }
    
    /// Create a tool resource ID.
    ///
    /// Used for CLI tool capability tracking. When a node requires a tool,
    /// it creates a resource access with this ID.
    pub fn tool(name: impl Into<String>) -> Self {
        Self(format!("tool:{}", name.into()))
    }
}

impl From<&str> for ResourceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// How a resource is accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessMode {
    /// Read-only access. Multiple reads can happen in parallel.
    Read,
    /// Write access. Conflicts with other writes and reads.
    Write,
    /// Exclusive access. Conflicts with any other access.
    Exclusive,
}

impl AccessMode {
    /// Check if this access mode conflicts with another.
    ///
    /// - Read + Read = OK
    /// - Read + Write = CONFLICT
    /// - Write + Write = CONFLICT
    /// - Exclusive + anything = CONFLICT
    pub fn conflicts_with(&self, other: &AccessMode) -> bool {
        match (self, other) {
            (AccessMode::Read, AccessMode::Read) => false,
            (AccessMode::Exclusive, _) | (_, AccessMode::Exclusive) => true,
            _ => true, // Write conflicts with everything except itself being analyzed
        }
    }
}

/// A resource access by a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAccess {
    /// The node performing the access.
    pub node_id: NodeId,
    /// The resource being accessed.
    pub resource_id: ResourceId,
    /// How the resource is accessed.
    pub mode: AccessMode,
}

impl ResourceAccess {
    /// Create a new resource access.
    pub fn new(node_id: impl Into<NodeId>, resource_id: impl Into<ResourceId>, mode: AccessMode) -> Self {
        Self {
            node_id: node_id.into(),
            resource_id: resource_id.into(),
            mode,
        }
    }

    /// Create a read access.
    pub fn read(node_id: impl Into<NodeId>, resource_id: impl Into<ResourceId>) -> Self {
        Self::new(node_id, resource_id, AccessMode::Read)
    }

    /// Create a write access.
    pub fn write(node_id: impl Into<NodeId>, resource_id: impl Into<ResourceId>) -> Self {
        Self::new(node_id, resource_id, AccessMode::Write)
    }

    /// Create an exclusive access.
    pub fn exclusive(node_id: impl Into<NodeId>, resource_id: impl Into<ResourceId>) -> Self {
        Self::new(node_id, resource_id, AccessMode::Exclusive)
    }
}

/// A resource conflict between two nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceConflict {
    /// First node in the conflict.
    pub node_a: NodeId,
    /// Second node in the conflict.
    pub node_b: NodeId,
    /// The resource that is being contended.
    pub resource_id: ResourceId,
    /// How node A accesses the resource.
    pub mode_a: AccessMode,
    /// How node B accesses the resource.
    pub mode_b: AccessMode,
}

impl std::fmt::Display for ResourceConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resource conflict: {} ({:?}) and {} ({:?}) both access {}",
            self.node_a, self.mode_a, self.node_b, self.mode_b, self.resource_id
        )
    }
}

/// Detect resource conflicts in a DAG.
///
/// Returns a list of conflicts where:
/// 1. Two nodes access the same resource
/// 2. The access modes conflict (not Read + Read)
/// 3. The nodes have no ordering edge between them
///
/// # Arguments
///
/// * `dag` - The DAG to analyze
/// * `accesses` - List of resource accesses by nodes
///
/// # Returns
///
/// List of detected conflicts
pub fn detect_conflicts<T>(dag: &Dag<T>, accesses: &[ResourceAccess]) -> Vec<ResourceConflict> {
    let mut conflicts = Vec::new();

    // Build a map of resource → accesses
    let mut resource_accesses: HashMap<&ResourceId, Vec<&ResourceAccess>> = HashMap::new();
    for access in accesses {
        resource_accesses
            .entry(&access.resource_id)
            .or_default()
            .push(access);
    }

    // Build a set of ordered pairs (nodes where one must execute before the other)
    let ordered_pairs = compute_ordered_pairs(dag);

    // Check each resource for conflicts
    for (resource_id, accesses) in resource_accesses {
        // Check all pairs of accesses to this resource
        for i in 0..accesses.len() {
            for j in (i + 1)..accesses.len() {
                let access_a = accesses[i];
                let access_b = accesses[j];

                // Check if modes conflict
                if !access_a.mode.conflicts_with(&access_b.mode) {
                    continue;
                }

                // Check if nodes are ordered (either A→B or B→A)
                let a_before_b = ordered_pairs.contains(&(&access_a.node_id, &access_b.node_id));
                let b_before_a = ordered_pairs.contains(&(&access_b.node_id, &access_a.node_id));

                if !a_before_b && !b_before_a {
                    // No ordering — conflict!
                    conflicts.push(ResourceConflict {
                        node_a: access_a.node_id.clone(),
                        node_b: access_b.node_id.clone(),
                        resource_id: resource_id.clone(),
                        mode_a: access_a.mode,
                        mode_b: access_b.mode,
                    });
                }
            }
        }
    }

    conflicts
}

/// Compute all ordered pairs of nodes (A, B) where A must execute before B.
///
/// This is the transitive closure of the edge relationship.
fn compute_ordered_pairs<T>(dag: &Dag<T>) -> HashSet<(&NodeId, &NodeId)> {
    let mut ordered = HashSet::new();

    // Direct edges
    for edge in &dag.edges {
        let from_node = dag.nodes.iter().find(|n| n.id == edge.from_node);
        let to_node = dag.nodes.iter().find(|n| n.id == edge.to_node);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            ordered.insert((&from.id, &to.id));
        }
    }

    // Compute transitive closure
    let node_ids: Vec<&NodeId> = dag.nodes.iter().map(|n| &n.id).collect();
    
    // Floyd-Warshall style transitive closure
    loop {
        let mut added = false;
        for &a in &node_ids {
            for &b in &node_ids {
                for &c in &node_ids {
                    if ordered.contains(&(a, b))
                        && ordered.contains(&(b, c))
                        && ordered.insert((a, c))
                    {
                        added = true;
                    }
                }
            }
        }
        if !added {
            break;
        }
    }

    ordered
}

/// Check if all resource accesses in a DAG are properly ordered.
///
/// Returns `Ok(())` if there are no conflicts, or `Err(conflicts)` with the list
/// of detected conflicts.
pub fn validate_resource_ordering<T>(
    dag: &Dag<T>,
    accesses: &[ResourceAccess],
) -> Result<(), Vec<ResourceConflict>> {
    let conflicts = detect_conflicts(dag, accesses);
    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(conflicts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{Dag, Edge, Port};
    use crate::node::Node;

    fn test_dag() -> Dag<String> {
        let mut dag = Dag::new();
        
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![Port::scalar("out", "String")],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            "op_b".to_string(),
        ));
        dag.add_node(Node::opaque(
            "c",
            vec![Port::scalar("in", "String")],
            vec![],
            "op_c".to_string(),
        ));
        
        // a → b → c
        dag.add_edge(Edge::new("a", "out", "b", "in"));
        dag.add_edge(Edge::new("b", "out", "c", "in"));
        
        dag
    }

    fn parallel_dag() -> Dag<String> {
        let mut dag = Dag::new();
        
        // Two independent nodes
        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![],
            vec![],
            "op_b".to_string(),
        ));
        
        dag
    }

    #[test]
    fn test_resource_id_creation() {
        let file = ResourceId::file("/tmp/test.txt");
        assert!(file.0.starts_with("file:"));

        let lock = ResourceId::lock("my_lock");
        assert!(lock.0.starts_with("lock:"));

        let conn = ResourceId::connection("db");
        assert!(conn.0.starts_with("conn:"));
    }

    #[test]
    fn test_access_mode_conflicts() {
        assert!(!AccessMode::Read.conflicts_with(&AccessMode::Read));
        assert!(AccessMode::Read.conflicts_with(&AccessMode::Write));
        assert!(AccessMode::Write.conflicts_with(&AccessMode::Read));
        assert!(AccessMode::Write.conflicts_with(&AccessMode::Write));
        assert!(AccessMode::Exclusive.conflicts_with(&AccessMode::Read));
        assert!(AccessMode::Exclusive.conflicts_with(&AccessMode::Write));
        assert!(AccessMode::Exclusive.conflicts_with(&AccessMode::Exclusive));
    }

    #[test]
    fn test_no_conflict_when_ordered() {
        let dag = test_dag();
        
        // a writes, then b reads — ordered, no conflict
        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::read("b", "file.txt"),
        ];
        
        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_conflict_when_parallel() {
        let dag = parallel_dag();
        
        // a and b both write — parallel, conflict!
        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];
        
        let conflicts = detect_conflicts(&dag, &accesses);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].resource_id.0, "file.txt");
    }

    #[test]
    fn test_no_conflict_for_parallel_reads() {
        let dag = parallel_dag();
        
        // a and b both read — parallel, but reads don't conflict
        let accesses = vec![
            ResourceAccess::read("a", "file.txt"),
            ResourceAccess::read("b", "file.txt"),
        ];
        
        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_conflict_read_write_parallel() {
        let dag = parallel_dag();
        
        // a reads, b writes — parallel, conflict!
        let accesses = vec![
            ResourceAccess::read("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];
        
        let conflicts = detect_conflicts(&dag, &accesses);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_transitive_ordering() {
        let dag = test_dag(); // a → b → c
        
        // a writes, c reads — transitively ordered through b
        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::read("c", "file.txt"),
        ];
        
        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_multiple_resources() {
        let dag = parallel_dag();
        
        // a writes to file1, b writes to file2 — different resources, no conflict
        let accesses = vec![
            ResourceAccess::write("a", "file1.txt"),
            ResourceAccess::write("b", "file2.txt"),
        ];
        
        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_validate_resource_ordering() {
        let dag = parallel_dag();
        
        // Valid: parallel reads
        let read_accesses = vec![
            ResourceAccess::read("a", "file.txt"),
            ResourceAccess::read("b", "file.txt"),
        ];
        assert!(validate_resource_ordering(&dag, &read_accesses).is_ok());
        
        // Invalid: parallel writes
        let write_accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];
        assert!(validate_resource_ordering(&dag, &write_accesses).is_err());
    }
}
