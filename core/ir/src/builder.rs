//! Generational DAG builder that prevents cycles by construction.
//!
//! The builder tracks node "generations" (topological levels) and rejects edges
//! that would create cycles. This makes cycles impossible by construction.
//!
//! # Example
//!
//! ```rust,ignore
//! use gunbc_ir::builder::DagBuilder;
//! use gunbc_ir::{Node, Port};

// Allow large error types - rich error context is valuable for debugging
#![allow(clippy::result_large_err)]
//!
//! let mut builder = DagBuilder::new();
//!
//! // Root nodes are generation 0
//! let a = builder.add_root_node(node_a)?;
//! let b = builder.add_root_node(node_b)?;
//!
//! // Dependent nodes are generation max(deps) + 1
//! let c = builder.add_node_after(node_c, &a)?;  // gen 1
//! let d = builder.add_node_after_all(node_d, &[&b, &c])?;  // gen 2
//!
//! // Edges must go from lower to higher generation
//! builder.add_edge(a.out("x"), c.in_port("y"))?;  // OK: 0 → 1
//! builder.add_edge(c.out("z"), a.in_port("w"))?;  // ERROR: 1 → 0 (cycle!)
//!
//! let dag = builder.build();
//! ```

use crate::dag::{Dag, Edge};
use crate::node::Node;
use crate::types::{Cardinality, NodeId, PortName, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

/// Error types for DAG builder operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuilderError {
    /// Attempted to create an edge that would form a cycle.
    /// Edges must go from lower generation to higher generation.
    CycleDetected {
        from: NodeId,
        to: NodeId,
        from_gen: usize,
        to_gen: usize,
    },
    /// A node with this ID already exists in the builder.
    DuplicateNodeId(NodeId),
    /// The specified port was not found on the node.
    PortNotFound {
        node: NodeId,
        port: PortName,
        kind: PortKind,
    },
    /// Edge connects ports with incompatible types.
    TypeMismatch {
        from_node: NodeId,
        from_port: PortName,
        from_type: TypeId,
        to_node: NodeId,
        to_port: PortName,
        to_type: TypeId,
    },
    /// Edge connects ports with incompatible cardinalities.
    CardinalityMismatch {
        from_node: NodeId,
        from_port: PortName,
        from_cardinality: Cardinality,
        to_node: NodeId,
        to_port: PortName,
        to_cardinality: Cardinality,
    },
    /// Multiple incoming edges to a scalar input port.
    FanInOnScalar {
        node: NodeId,
        port: PortName,
        existing_edges: usize,
        cardinality: Cardinality,
    },
    /// Output port uses the reserved `res:` prefix (reserved for resource inputs).
    InvalidResourceOutputPort { node: NodeId, port: PortName },
    /// Resource input port is not wired to any upstream edge.
    UnwiredResourceInput { node: NodeId, port: PortName },
}

/// Whether a port is an input or output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    Input,
    Output,
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::CycleDetected {
                from,
                to,
                from_gen,
                to_gen,
            } => {
                write!(
                    f,
                    "cycle detected: edge from '{}' (gen {}) to '{}' (gen {}) would create a cycle \
                     (edges must go from lower to higher generation)",
                    from, from_gen, to, to_gen
                )
            }
            BuilderError::DuplicateNodeId(id) => {
                write!(f, "duplicate node ID: '{}'", id)
            }
            BuilderError::PortNotFound { node, port, kind } => {
                write!(
                    f,
                    "{} port '{}' not found on node '{}'",
                    match kind {
                        PortKind::Input => "input",
                        PortKind::Output => "output",
                    },
                    port,
                    node
                )
            }
            BuilderError::TypeMismatch {
                from_node,
                from_port,
                from_type,
                to_node,
                to_port,
                to_type,
            } => {
                write!(
                    f,
                    "type mismatch: {}:{} has type '{}', but {}:{} expects type '{}'",
                    from_node, from_port, from_type, to_node, to_port, to_type
                )
            }
            BuilderError::CardinalityMismatch {
                from_node,
                from_port,
                from_cardinality,
                to_node,
                to_port,
                to_cardinality,
            } => {
                write!(
                    f,
                    "cardinality mismatch: {}:{} produces {:?}, but {}:{} requires {:?}",
                    from_node, from_port, from_cardinality, to_node, to_port, to_cardinality
                )
            }
            BuilderError::FanInOnScalar {
                node,
                port,
                existing_edges,
                cardinality,
            } => {
                write!(
                    f,
                    "fan-in on scalar input '{}:{}' ({} incoming edges, cardinality {:?}). \
                     Use a list input or an explicit merge node.",
                    node,
                    port,
                    existing_edges + 1,
                    cardinality
                )
            }
            BuilderError::InvalidResourceOutputPort { node, port } => {
                write!(
                    f,
                    "invalid output port '{}:{}': 'res:' prefix is reserved for resource inputs",
                    node, port
                )
            }
            BuilderError::UnwiredResourceInput { node, port } => {
                write!(
                    f,
                    "unwired resource input '{}:{}' (res:* inputs must be connected)",
                    node, port
                )
            }
        }
    }
}

impl std::error::Error for BuilderError {}

/// A reference to a node in the builder, tracking its generation.
///
/// `NodeRef` is returned when adding nodes and is used to create edges.
/// The generation tracks the node's topological level in the DAG.
#[derive(Debug, Clone)]
pub struct NodeRef<T> {
    id: NodeId,
    generation: usize,
    _phantom: PhantomData<T>,
}

impl<T> NodeRef<T> {
    /// Get the node's ID.
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    /// Get the node's generation (topological level).
    ///
    /// - Root nodes have generation 0
    /// - Nodes added with `add_node_after` have generation = dep.generation + 1
    /// - Nodes added with `add_node_after_all` have generation = max(deps.generation) + 1
    pub fn generation(&self) -> usize {
        self.generation
    }

    /// Create a reference to an output port on this node.
    pub fn out(&self, port: impl Into<PortName>) -> OutputRef<T> {
        OutputRef {
            node_id: self.id.clone(),
            generation: self.generation,
            port: port.into(),
            _phantom: PhantomData,
        }
    }

    /// Create a reference to an input port on this node.
    pub fn in_port(&self, port: impl Into<PortName>) -> InputRef<T> {
        InputRef {
            node_id: self.id.clone(),
            generation: self.generation,
            port: port.into(),
            _phantom: PhantomData,
        }
    }
}

/// A reference to an output port on a node.
#[derive(Debug, Clone)]
pub struct OutputRef<T> {
    node_id: NodeId,
    generation: usize,
    port: PortName,
    _phantom: PhantomData<T>,
}

impl<T> OutputRef<T> {
    /// Get the node ID this output belongs to.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Get the port name.
    pub fn port(&self) -> &PortName {
        &self.port
    }

    /// Get the generation of the node this output belongs to.
    pub fn generation(&self) -> usize {
        self.generation
    }
}

/// A reference to an input port on a node.
#[derive(Debug, Clone)]
pub struct InputRef<T> {
    node_id: NodeId,
    generation: usize,
    port: PortName,
    _phantom: PhantomData<T>,
}

impl<T> InputRef<T> {
    /// Get the node ID this input belongs to.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Get the port name.
    pub fn port(&self) -> &PortName {
        &self.port
    }

    /// Get the generation of the node this input belongs to.
    pub fn generation(&self) -> usize {
        self.generation
    }
}

/// A builder for constructing DAGs with generation tracking.
///
/// The builder prevents cycles by tracking node "generations" (topological levels)
/// and rejecting edges that would go from higher to lower generations.
///
/// # Generation Rules
///
/// - Root nodes (added with `add_root_node`) have generation 0
/// - Dependent nodes have generation = max(dependencies) + 1
/// - Edges must go from lower generation to higher generation
///
/// # Example
///
/// ```rust,ignore
/// let mut builder = DagBuilder::new();
/// let a = builder.add_root_node(node_a)?;
/// let b = builder.add_node_after(node_b, &a)?;
/// builder.add_edge(a.out("x"), b.in_port("y"))?;
/// let dag = builder.build();
/// ```
pub struct DagBuilder<T> {
    nodes: Vec<Node<T>>,
    edges: Vec<Edge>,
    generations: HashMap<NodeId, usize>,
    /// Counter for assigning unique edge indices
    next_edge_index: usize,
}

impl<T> Default for DagBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DagBuilder<T> {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            generations: HashMap::new(),
            next_edge_index: 0,
        }
    }

    /// Add a root node (generation 0, no dependencies).
    ///
    /// Root nodes are the entry points of the DAG — they don't depend on other nodes.
    ///
    /// # Errors
    ///
    /// Returns `BuilderError::DuplicateNodeId` if a node with the same ID already exists.
    pub fn add_root_node(&mut self, node: Node<T>) -> Result<NodeRef<T>, BuilderError> {
        self.add_node_with_generation(node, 0)
    }

    /// Add a node that depends on one other node.
    ///
    /// The new node's generation will be `dep.generation + 1`.
    ///
    /// # Errors
    ///
    /// Returns `BuilderError::DuplicateNodeId` if a node with the same ID already exists.
    pub fn add_node_after(
        &mut self,
        node: Node<T>,
        dep: &NodeRef<T>,
    ) -> Result<NodeRef<T>, BuilderError> {
        let generation = dep.generation + 1;
        self.add_node_with_generation(node, generation)
    }

    /// Add a node that depends on multiple other nodes.
    ///
    /// The new node's generation will be `max(deps.generation) + 1`.
    ///
    /// # Panics
    ///
    /// Panics if `deps` is empty. Use `add_root_node` for nodes with no dependencies.
    ///
    /// # Errors
    ///
    /// Returns `BuilderError::DuplicateNodeId` if a node with the same ID already exists.
    pub fn add_node_after_all(
        &mut self,
        node: Node<T>,
        deps: &[&NodeRef<T>],
    ) -> Result<NodeRef<T>, BuilderError> {
        assert!(
            !deps.is_empty(),
            "deps must not be empty; use add_root_node for nodes with no dependencies"
        );
        let max_gen = deps.iter().map(|d| d.generation).max().unwrap();
        let generation = max_gen + 1;
        self.add_node_with_generation(node, generation)
    }

    /// Internal: add a node with a specific generation.
    fn add_node_with_generation(
        &mut self,
        node: Node<T>,
        generation: usize,
    ) -> Result<NodeRef<T>, BuilderError> {
        // Check for duplicate ID
        if self.generations.contains_key(&node.id) {
            return Err(BuilderError::DuplicateNodeId(node.id.clone()));
        }

        // Enforce resource port naming convention: `res:*` reserved for inputs.
        for port in &node.outputs {
            if port.name.0.starts_with("res:") {
                return Err(BuilderError::InvalidResourceOutputPort {
                    node: node.id.clone(),
                    port: port.name.clone(),
                });
            }
        }

        let id = node.id.clone();
        self.generations.insert(id.clone(), generation);
        self.nodes.push(node);

        Ok(NodeRef {
            id,
            generation,
            _phantom: PhantomData,
        })
    }

    /// Add an edge between an output port and an input port.
    ///
    /// The edge is validated immediately:
    /// - Generation check: from.generation < to.generation (prevents cycles)
    /// - Port existence check: both ports must exist on their respective nodes
    /// - Type check: port types must match
    /// - Cardinality check: output cardinality must satisfy input cardinality
    ///
    /// # Errors
    ///
    /// - `BuilderError::CycleDetected` if the edge would create a cycle
    /// - `BuilderError::PortNotFound` if a port doesn't exist
    /// - `BuilderError::TypeMismatch` if port types don't match
    /// - `BuilderError::CardinalityMismatch` if cardinalities are incompatible
    /// - `BuilderError::FanInOnScalar` if multiple edges target a scalar/optional input
    pub fn add_edge(&mut self, from: OutputRef<T>, to: InputRef<T>) -> Result<(), BuilderError> {
        // Check generation ordering (cycle prevention)
        if from.generation >= to.generation {
            return Err(BuilderError::CycleDetected {
                from: from.node_id.clone(),
                to: to.node_id.clone(),
                from_gen: from.generation,
                to_gen: to.generation,
            });
        }

        // Find the source and target nodes
        let from_node = self.nodes.iter().find(|n| n.id == from.node_id);
        let to_node = self.nodes.iter().find(|n| n.id == to.node_id);

        // Verify output port exists
        let from_port = from_node.and_then(|n| n.outputs.iter().find(|p| p.name == from.port));
        if from_port.is_none() {
            return Err(BuilderError::PortNotFound {
                node: from.node_id.clone(),
                port: from.port.clone(),
                kind: PortKind::Output,
            });
        }
        let from_port = from_port.unwrap();

        // Verify input port exists
        let to_port = to_node.and_then(|n| n.inputs.iter().find(|p| p.name == to.port));
        if to_port.is_none() {
            return Err(BuilderError::PortNotFound {
                node: to.node_id.clone(),
                port: to.port.clone(),
                kind: PortKind::Input,
            });
        }
        let to_port = to_port.unwrap();

        // Check type compatibility
        if from_port.type_id != to_port.type_id {
            return Err(BuilderError::TypeMismatch {
                from_node: from.node_id.clone(),
                from_port: from.port.clone(),
                from_type: from_port.type_id.clone(),
                to_node: to.node_id.clone(),
                to_port: to.port.clone(),
                to_type: to_port.type_id.clone(),
            });
        }

        // Check cardinality compatibility
        if !from_port.cardinality.satisfies(to_port.cardinality) {
            return Err(BuilderError::CardinalityMismatch {
                from_node: from.node_id.clone(),
                from_port: from.port.clone(),
                from_cardinality: from_port.cardinality,
                to_node: to.node_id.clone(),
                to_port: to.port.clone(),
                to_cardinality: to_port.cardinality,
            });
        }

        // Reject fan-in to scalar/optional ports (must be list-typed to accept multiple edges).
        let existing_edges = self.edge_count_to_port(&to.node_id, &to.port);
        if existing_edges > 0 && !to_port.cardinality.is_list() {
            return Err(BuilderError::FanInOnScalar {
                node: to.node_id.clone(),
                port: to.port.clone(),
                existing_edges,
                cardinality: to_port.cardinality,
            });
        }

        // Add the edge with auto-assigned index
        let index = self.next_edge_index;
        self.next_edge_index += 1;

        self.edges.push(Edge {
            from_node: from.node_id,
            from_port: from.port,
            to_node: to.node_id,
            to_port: to.port,
            index,
        });

        Ok(())
    }

    /// Count the number of edges already connected to a specific input port.
    ///
    /// This is useful for fan-in detection — when multiple edges feed into
    /// the same port.
    pub fn edge_count_to_port(&self, node_id: &NodeId, port: &PortName) -> usize {
        self.edges
            .iter()
            .filter(|e| &e.to_node == node_id && &e.to_port == port)
            .count()
    }

    /// Count the number of edges coming from a specific output port.
    ///
    /// This is useful for fan-out detection — when one port feeds multiple
    /// downstream ports.
    pub fn edge_count_from_port(&self, node_id: &NodeId, port: &PortName) -> usize {
        self.edges
            .iter()
            .filter(|e| &e.from_node == node_id && &e.from_port == port)
            .count()
    }

    /// Check if a port has fan-in (multiple incoming edges).
    pub fn has_fan_in(&self, node_id: &NodeId, port: &PortName) -> bool {
        self.edge_count_to_port(node_id, port) > 1
    }

    /// Check if a port has fan-out (multiple outgoing edges).
    pub fn has_fan_out(&self, node_id: &NodeId, port: &PortName) -> bool {
        self.edge_count_from_port(node_id, port) > 1
    }

    /// Get information about fan-in for all input ports.
    ///
    /// Returns a map of (node_id, port_name) → edge_count for ports with
    /// more than one incoming edge.
    pub fn fan_in_ports(&self) -> HashMap<(NodeId, PortName), usize> {
        let mut counts: HashMap<(NodeId, PortName), usize> = HashMap::new();

        for edge in &self.edges {
            let key = (edge.to_node.clone(), edge.to_port.clone());
            *counts.entry(key).or_insert(0) += 1;
        }

        counts.into_iter().filter(|(_, count)| *count > 1).collect()
    }

    /// Get information about fan-out for all output ports.
    ///
    /// Returns a map of (node_id, port_name) → edge_count for ports with
    /// more than one outgoing edge.
    pub fn fan_out_ports(&self) -> HashMap<(NodeId, PortName), usize> {
        let mut counts: HashMap<(NodeId, PortName), usize> = HashMap::new();

        for edge in &self.edges {
            let key = (edge.from_node.clone(), edge.from_port.clone());
            *counts.entry(key).or_insert(0) += 1;
        }

        counts.into_iter().filter(|(_, count)| *count > 1).collect()
    }

    /// Consume the builder and produce the DAG.
    ///
    /// Since all edges were validated during construction, the resulting DAG
    /// is guaranteed to be acyclic.
    pub fn build(self) -> Dag<T> {
        Dag {
            nodes: self.nodes,
            edges: self.edges,
        }
    }

    /// Get the current number of nodes in the builder.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the current number of edges in the builder.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::Port;
    use crate::node::Node;

    /// Helper to create a simple opaque node for testing.
    fn test_node(id: &str, inputs: Vec<(&str, &str)>, outputs: Vec<(&str, &str)>) -> Node<String> {
        Node::opaque(
            id,
            inputs
                .into_iter()
                .map(|(name, ty)| Port::new(name, ty))
                .collect(),
            outputs
                .into_iter()
                .map(|(name, ty)| Port::new(name, ty))
                .collect(),
            format!("op_{}", id),
        )
    }

    // ==================== Basic Construction Tests ====================

    #[test]
    fn test_empty_builder() {
        let builder: DagBuilder<String> = DagBuilder::new();
        assert_eq!(builder.node_count(), 0);
        assert_eq!(builder.edge_count(), 0);

        let dag = builder.build();
        assert!(dag.nodes.is_empty());
        assert!(dag.edges.is_empty());
    }

    #[test]
    fn test_single_node() {
        let mut builder: DagBuilder<String> = DagBuilder::new();
        let node = test_node("a", vec![], vec![("out", "String")]);

        let a = builder.add_root_node(node).unwrap();
        assert_eq!(a.id().0.as_str(), "a");
        assert_eq!(a.generation(), 0);
        assert_eq!(builder.node_count(), 1);

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.nodes[0].id.0.as_str(), "a");
    }

    #[test]
    fn test_linear_chain() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![("out", "String")]);
        let node_c = test_node("c", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &b).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();
        builder.add_edge(b.out("out"), c.in_port("in")).unwrap();

        assert_eq!(a.generation(), 0);
        assert_eq!(b.generation(), 1);
        assert_eq!(c.generation(), 2);

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 2);
    }

    // ==================== Diamond Pattern Tests ====================

    #[test]
    fn test_diamond_pattern() {
        // A → B, A → C, B → D, C → D
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![("out", "String")]);
        let node_c = test_node("c", vec![("in", "String")], vec![("out", "String")]);
        let node_d = test_node("d", vec![("in1", "String"), ("in2", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &a).unwrap();
        let d = builder.add_node_after_all(node_d, &[&b, &c]).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();
        builder.add_edge(a.out("out"), c.in_port("in")).unwrap();
        builder.add_edge(b.out("out"), d.in_port("in1")).unwrap();
        builder.add_edge(c.out("out"), d.in_port("in2")).unwrap();

        assert_eq!(a.generation(), 0);
        assert_eq!(b.generation(), 1);
        assert_eq!(c.generation(), 1);
        assert_eq!(d.generation(), 2);

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 4);
    }

    #[test]
    fn test_wide_fan_out() {
        // A → B, A → C, A → D (all at gen 1)
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![]);
        let node_c = test_node("c", vec![("in", "String")], vec![]);
        let node_d = test_node("d", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &a).unwrap();
        let d = builder.add_node_after(node_d, &a).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();
        builder.add_edge(a.out("out"), c.in_port("in")).unwrap();
        builder.add_edge(a.out("out"), d.in_port("in")).unwrap();

        assert_eq!(b.generation(), 1);
        assert_eq!(c.generation(), 1);
        assert_eq!(d.generation(), 1);

        let dag = builder.build();
        assert_eq!(dag.edges.len(), 3);
    }

    #[test]
    fn test_wide_fan_in() {
        // A, B, C → D
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![], vec![("out", "String")]);
        let node_c = test_node("c", vec![], vec![("out", "String")]);
        let node_d = test_node(
            "d",
            vec![("in1", "String"), ("in2", "String"), ("in3", "String")],
            vec![],
        );

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_root_node(node_b).unwrap();
        let c = builder.add_root_node(node_c).unwrap();
        let d = builder.add_node_after_all(node_d, &[&a, &b, &c]).unwrap();

        builder.add_edge(a.out("out"), d.in_port("in1")).unwrap();
        builder.add_edge(b.out("out"), d.in_port("in2")).unwrap();
        builder.add_edge(c.out("out"), d.in_port("in3")).unwrap();

        assert_eq!(a.generation(), 0);
        assert_eq!(b.generation(), 0);
        assert_eq!(c.generation(), 0);
        assert_eq!(d.generation(), 1);

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 3);
    }

    // ==================== Cycle Detection Tests ====================

    #[test]
    fn test_direct_cycle_rejected() {
        // A → B, B → A (cycle)
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![("in", "String")], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![("out", "String")]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();

        // Forward edge works
        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();

        // Backward edge should fail
        let result = builder.add_edge(b.out("out"), a.in_port("in"));
        assert!(matches!(result, Err(BuilderError::CycleDetected { .. })));
    }

    #[test]
    fn test_self_loop_rejected() {
        // A → A (self loop)
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![("in", "String")], vec![("out", "String")]);
        let a = builder.add_root_node(node_a).unwrap();

        let result = builder.add_edge(a.out("out"), a.in_port("in"));
        assert!(matches!(result, Err(BuilderError::CycleDetected { .. })));
    }

    #[test]
    fn test_indirect_cycle_rejected() {
        // A → B → C, trying C → A (indirect cycle through generations)
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![("in", "String")], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![("out", "String")]);
        let node_c = test_node("c", vec![("in", "String")], vec![("out", "String")]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &b).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();
        builder.add_edge(b.out("out"), c.in_port("in")).unwrap();

        // Trying to create C → A should fail (gen 2 → gen 0)
        let result = builder.add_edge(c.out("out"), a.in_port("in"));
        assert!(matches!(result, Err(BuilderError::CycleDetected { .. })));
    }

    #[test]
    fn test_cycle_error_message() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![("in", "String")], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![("out", "String")]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();

        let result = builder.add_edge(b.out("out"), a.in_port("in"));

        match result {
            Err(BuilderError::CycleDetected {
                from,
                to,
                from_gen,
                to_gen,
            }) => {
                assert_eq!(from.0.as_str(), "b");
                assert_eq!(to.0.as_str(), "a");
                assert_eq!(from_gen, 1);
                assert_eq!(to_gen, 0);

                // Test Display impl
                let msg = format!(
                    "{}",
                    BuilderError::CycleDetected {
                        from,
                        to,
                        from_gen,
                        to_gen
                    }
                );
                assert!(msg.contains("cycle detected"));
                assert!(msg.contains("gen 1"));
                assert!(msg.contains("gen 0"));
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    // ==================== Generation Tracking Tests ====================

    #[test]
    fn test_root_nodes_are_gen_zero() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let a = builder
            .add_root_node(test_node("a", vec![], vec![]))
            .unwrap();
        let b = builder
            .add_root_node(test_node("b", vec![], vec![]))
            .unwrap();
        let c = builder
            .add_root_node(test_node("c", vec![], vec![]))
            .unwrap();

        assert_eq!(a.generation(), 0);
        assert_eq!(b.generation(), 0);
        assert_eq!(c.generation(), 0);
    }

    #[test]
    fn test_dependent_nodes_increment_gen() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let a = builder
            .add_root_node(test_node("a", vec![], vec![]))
            .unwrap();
        let b = builder
            .add_node_after(test_node("b", vec![], vec![]), &a)
            .unwrap();
        let c = builder
            .add_node_after(test_node("c", vec![], vec![]), &b)
            .unwrap();
        let d = builder
            .add_node_after(test_node("d", vec![], vec![]), &c)
            .unwrap();

        assert_eq!(a.generation(), 0);
        assert_eq!(b.generation(), 1);
        assert_eq!(c.generation(), 2);
        assert_eq!(d.generation(), 3);
    }

    #[test]
    fn test_multi_dep_uses_max_gen() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        // Create nodes at different generations
        let a = builder
            .add_root_node(test_node("a", vec![], vec![]))
            .unwrap(); // gen 0
        let b = builder
            .add_node_after(test_node("b", vec![], vec![]), &a)
            .unwrap(); // gen 1
        let c = builder
            .add_node_after(test_node("c", vec![], vec![]), &b)
            .unwrap(); // gen 2

        // Node d depends on a (gen 0), b (gen 1), c (gen 2) → should be gen 3
        let d = builder
            .add_node_after_all(test_node("d", vec![], vec![]), &[&a, &b, &c])
            .unwrap();

        assert_eq!(d.generation(), 3); // max(0, 1, 2) + 1 = 3
    }

    #[test]
    fn test_generation_accessible() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let a = builder
            .add_root_node(test_node("a", vec![], vec![("out", "String")]))
            .unwrap();

        // Test NodeRef.generation()
        assert_eq!(a.generation(), 0);

        // Test OutputRef.generation()
        let out = a.out("out");
        assert_eq!(out.generation(), 0);

        // Test InputRef.generation() (need a node with input)
        let b = builder
            .add_node_after(test_node("b", vec![("in", "String")], vec![]), &a)
            .unwrap();
        let inp = b.in_port("in");
        assert_eq!(inp.generation(), 1);
    }

    // ==================== Edge Cases Tests ====================

    #[test]
    fn test_duplicate_node_id_rejected() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        builder
            .add_root_node(test_node("a", vec![], vec![]))
            .unwrap();

        // Try to add another node with the same ID
        let result = builder.add_root_node(test_node("a", vec![], vec![]));

        match result {
            Err(BuilderError::DuplicateNodeId(id)) => {
                assert_eq!(id.0.as_str(), "a");
            }
            _ => panic!("Expected DuplicateNodeId error"),
        }
    }

    #[test]
    fn test_multiple_edges_same_ports() {
        // Fan-out: one output to multiple inputs (same output port)
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![]);
        let node_c = test_node("c", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &a).unwrap();

        // Both edges from same output port - should work
        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();
        builder.add_edge(a.out("out"), c.in_port("in")).unwrap();

        let dag = builder.build();
        assert_eq!(dag.edges.len(), 2);
    }

    #[test]
    fn test_edges_between_same_gen_rejected() {
        // Two root nodes at gen 0 - edge between them should be rejected
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_root_node(node_b).unwrap();

        // Both at gen 0 - edge should fail (can't have edge from gen 0 to gen 0)
        let result = builder.add_edge(a.out("out"), b.in_port("in"));
        assert!(matches!(result, Err(BuilderError::CycleDetected { .. })));
    }

    #[test]
    fn test_port_not_found_output() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();

        // Try to use non-existent output port
        let result = builder.add_edge(a.out("nonexistent"), b.in_port("in"));

        match result {
            Err(BuilderError::PortNotFound { node, port, kind }) => {
                assert_eq!(node.0.as_str(), "a");
                assert_eq!(port.0.as_str(), "nonexistent");
                assert_eq!(kind, PortKind::Output);
            }
            _ => panic!("Expected PortNotFound error"),
        }
    }

    #[test]
    fn test_port_not_found_input() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();

        // Try to use non-existent input port
        let result = builder.add_edge(a.out("out"), b.in_port("nonexistent"));

        match result {
            Err(BuilderError::PortNotFound { node, port, kind }) => {
                assert_eq!(node.0.as_str(), "b");
                assert_eq!(port.0.as_str(), "nonexistent");
                assert_eq!(kind, PortKind::Input);
            }
            _ => panic!("Expected PortNotFound error"),
        }
    }

    #[test]
    fn test_type_mismatch_rejected() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = Node::opaque(
            "b",
            vec![Port::new("in", "Int")], // Different type!
            vec![],
            "op_b".to_string(),
        );

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();

        let result = builder.add_edge(a.out("out"), b.in_port("in"));

        match result {
            Err(BuilderError::TypeMismatch {
                from_type, to_type, ..
            }) => {
                assert_eq!(from_type.0.as_str(), "String");
                assert_eq!(to_type.0.as_str(), "Int");
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_cardinality_mismatch_rejected() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        // Output is ZeroOrOne (optional), input requires One (required)
        let node_a = Node::opaque(
            "a",
            vec![],
            vec![Port::optional("out", "String")], // ZeroOrOne
            "op_a".to_string(),
        );
        let node_b = Node::opaque(
            "b",
            vec![Port::scalar("in", "String")], // One (required)
            vec![],
            "op_b".to_string(),
        );

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();

        let result = builder.add_edge(a.out("out"), b.in_port("in"));

        match result {
            Err(BuilderError::CardinalityMismatch {
                from_cardinality,
                to_cardinality,
                ..
            }) => {
                assert_eq!(from_cardinality, Cardinality::ZERO_OR_ONE);
                assert_eq!(to_cardinality, Cardinality::ONE);
            }
            _ => panic!("Expected CardinalityMismatch error"),
        }
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_builder_produces_valid_dag() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![("out", "String")]);
        let node_c = test_node("c", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &b).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();
        builder.add_edge(b.out("out"), c.in_port("in")).unwrap();

        let dag = builder.build();

        // Verify DAG structure
        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 2);

        // Verify nodes are present
        assert!(dag.get_node(&"a".into()).is_some());
        assert!(dag.get_node(&"b".into()).is_some());
        assert!(dag.get_node(&"c".into()).is_some());

        // Verify edges
        assert_eq!(dag.edges[0].from_node.0.as_str(), "a");
        assert_eq!(dag.edges[0].to_node.0.as_str(), "b");
        assert_eq!(dag.edges[1].from_node.0.as_str(), "b");
        assert_eq!(dag.edges[1].to_node.0.as_str(), "c");
    }

    #[test]
    fn test_builder_with_subdags() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        // Create an inner DAG
        let mut inner_builder: DagBuilder<String> = DagBuilder::new();
        let inner_a = inner_builder
            .add_root_node(test_node("inner_a", vec![], vec![("out", "String")]))
            .unwrap();
        let inner_b = inner_builder
            .add_node_after(
                test_node("inner_b", vec![("in", "String")], vec![]),
                &inner_a,
            )
            .unwrap();
        inner_builder
            .add_edge(inner_a.out("out"), inner_b.in_port("in"))
            .unwrap();
        let inner_dag = inner_builder.build();

        // Create outer DAG with subdag node
        let subdag_node = Node::subdag("subdag", inner_dag);

        let s = builder.add_root_node(subdag_node).unwrap();
        assert_eq!(s.generation(), 0);

        let dag = builder.build();
        assert_eq!(dag.nodes.len(), 1);
        assert!(dag.nodes[0].is_subdag());
    }

    #[test]
    fn test_builder_with_guards() {
        use crate::dag::Guard;
        use crate::value::Value;

        let mut builder: DagBuilder<String> = DagBuilder::new();

        // Create a node with a guarded input port
        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = Node::opaque(
            "b",
            vec![Port::guarded(
                "in",
                "String",
                Value::Str("expected".to_string()),
            )],
            vec![],
            "op_b".to_string(),
        );

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();

        let dag = builder.build();

        // Verify guard is preserved
        let node_b = dag.get_node(&"b".into()).unwrap();
        let input_port = &node_b.inputs[0];
        assert!(input_port.guard.is_some());

        match &input_port.guard {
            Some(Guard::Eq(Value::Str(s))) => assert_eq!(s, "expected"),
            _ => panic!("Expected Eq guard with string value"),
        }
    }

    // ==================== Fan-in/Fan-out Tests ====================

    #[test]
    fn test_fan_in_detection() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        // Create a diamond pattern: a, b → c
        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![], vec![("out", "String")]);
        let node_c = Node::opaque(
            "c",
            vec![Port::list("in", "String")], // List to accept fan-in
            vec![],
            "op_c".to_string(),
        );

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_root_node(node_b).unwrap();
        let c = builder.add_node_after_all(node_c, &[&a, &b]).unwrap();

        builder.add_edge(a.out("out"), c.in_port("in")).unwrap();

        // Before second edge, no fan-in
        assert!(!builder.has_fan_in(&c.id().clone(), &PortName::from("in")));
        assert_eq!(
            builder.edge_count_to_port(&c.id().clone(), &PortName::from("in")),
            1
        );

        builder.add_edge(b.out("out"), c.in_port("in")).unwrap();

        // After second edge, has fan-in
        assert!(builder.has_fan_in(&c.id().clone(), &PortName::from("in")));
        assert_eq!(
            builder.edge_count_to_port(&c.id().clone(), &PortName::from("in")),
            2
        );
    }

    #[test]
    fn test_fan_in_scalar_rejected() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![], vec![("out", "String")]);
        let node_c = test_node("c", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_root_node(node_b).unwrap();
        let c = builder.add_node_after_all(node_c, &[&a, &b]).unwrap();

        builder.add_edge(a.out("out"), c.in_port("in")).unwrap();
        let result = builder.add_edge(b.out("out"), c.in_port("in"));

        assert!(matches!(result, Err(BuilderError::FanInOnScalar { .. })));
    }

    #[test]
    fn test_fan_out_detection() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        // Create a broadcast pattern: a → b, c
        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![]);
        let node_c = test_node("c", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &a).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();

        // Before second edge, no fan-out
        assert!(!builder.has_fan_out(&a.id().clone(), &PortName::from("out")));
        assert_eq!(
            builder.edge_count_from_port(&a.id().clone(), &PortName::from("out")),
            1
        );

        builder.add_edge(a.out("out"), c.in_port("in")).unwrap();

        // After second edge, has fan-out
        assert!(builder.has_fan_out(&a.id().clone(), &PortName::from("out")));
        assert_eq!(
            builder.edge_count_from_port(&a.id().clone(), &PortName::from("out")),
            2
        );
    }

    #[test]
    fn test_fan_in_ports_summary() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![], vec![("out", "String")]);
        let node_c = test_node("c", vec![], vec![("out", "String")]);
        let node_d = Node::opaque(
            "d",
            vec![Port::list("in", "String")],
            vec![],
            "op_d".to_string(),
        );

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_root_node(node_b).unwrap();
        let c = builder.add_root_node(node_c).unwrap();
        let d = builder.add_node_after_all(node_d, &[&a, &b, &c]).unwrap();

        builder.add_edge(a.out("out"), d.in_port("in")).unwrap();
        builder.add_edge(b.out("out"), d.in_port("in")).unwrap();
        builder.add_edge(c.out("out"), d.in_port("in")).unwrap();

        let fan_ins = builder.fan_in_ports();
        assert_eq!(fan_ins.len(), 1);

        let key = (NodeId::from("d"), PortName::from("in"));
        assert_eq!(fan_ins.get(&key), Some(&3));
    }

    #[test]
    fn test_fan_out_ports_summary() {
        let mut builder: DagBuilder<String> = DagBuilder::new();

        let node_a = test_node("a", vec![], vec![("out", "String")]);
        let node_b = test_node("b", vec![("in", "String")], vec![]);
        let node_c = test_node("c", vec![("in", "String")], vec![]);
        let node_d = test_node("d", vec![("in", "String")], vec![]);

        let a = builder.add_root_node(node_a).unwrap();
        let b = builder.add_node_after(node_b, &a).unwrap();
        let c = builder.add_node_after(node_c, &a).unwrap();
        let d = builder.add_node_after(node_d, &a).unwrap();

        builder.add_edge(a.out("out"), b.in_port("in")).unwrap();
        builder.add_edge(a.out("out"), c.in_port("in")).unwrap();
        builder.add_edge(a.out("out"), d.in_port("in")).unwrap();

        let fan_outs = builder.fan_out_ports();
        assert_eq!(fan_outs.len(), 1);

        let key = (NodeId::from("a"), PortName::from("out"));
        assert_eq!(fan_outs.get(&key), Some(&3));
    }
}
