//! DAG structure: edges, ports, and the graph itself.

use crate::node::Node;
use crate::types::{Cardinality, NodeId, PortName, TypeId};
use crate::value::Value;
use serde::{Deserialize, Serialize};

/// A directed acyclic graph of nodes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dag<T> {
    /// Nodes in the DAG
    pub nodes: Vec<Node<T>>,
    /// Edges connecting output ports to input ports
    pub edges: Vec<Edge>,
}

impl<T> Dag<T> {
    /// Create an empty DAG.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the DAG.
    pub fn add_node(&mut self, node: Node<T>) {
        self.nodes.push(node);
    }

    /// Add an edge to the DAG.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: &NodeId) -> Option<&Node<T>> {
        self.nodes.iter().find(|n| &n.id == id)
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut Node<T>> {
        self.nodes.iter_mut().find(|n| &n.id == id)
    }
}

/// An edge connecting an output port of one node to an input port of another.
///
/// The `index` field provides a tie-breaker for canonical ordering when multiple
/// edges have the same source node/port. This ensures deterministic fan-in collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from_node: NodeId,
    pub from_port: PortName,
    pub to_node: NodeId,
    pub to_port: PortName,
    /// Index for canonical ordering (tie-breaker for edges with same source)
    #[serde(default)]
    pub index: usize,
}

impl Edge {
    pub fn new(
        from_node: impl Into<NodeId>,
        from_port: impl Into<PortName>,
        to_node: impl Into<NodeId>,
        to_port: impl Into<PortName>,
    ) -> Self {
        Self {
            from_node: from_node.into(),
            from_port: from_port.into(),
            to_node: to_node.into(),
            to_port: to_port.into(),
            index: 0,
        }
    }

    /// Create an edge with an explicit index for canonical ordering.
    pub fn with_index(
        from_node: impl Into<NodeId>,
        from_port: impl Into<PortName>,
        to_node: impl Into<NodeId>,
        to_port: impl Into<PortName>,
        index: usize,
    ) -> Self {
        Self {
            from_node: from_node.into(),
            from_port: from_port.into(),
            to_node: to_node.into(),
            to_port: to_port.into(),
            index,
        }
    }

    /// Get the canonical sort key for this edge.
    ///
    /// Edges are ordered by: (from_node, from_port, index)
    /// This ensures deterministic collection order for fan-in scenarios.
    pub fn sort_key(&self) -> (&NodeId, &PortName, usize) {
        (&self.from_node, &self.from_port, self.index)
    }
}

/// Get edges in canonical order for deterministic fan-in collection.
///
/// Edges are sorted by: (from_node_id, from_port_name, index)
/// This ensures the same DAG always produces the same collection order.
pub fn canonical_edge_order(edges: &[Edge]) -> Vec<&Edge> {
    let mut sorted: Vec<&Edge> = edges.iter().collect();
    sorted.sort_by_key(|e| e.sort_key());
    sorted
}

/// Get edges targeting a specific input port, in canonical order.
///
/// Useful for fan-in scenarios where multiple edges feed into one port.
pub fn edges_to_port<'a>(edges: &'a [Edge], node: &NodeId, port: &PortName) -> Vec<&'a Edge> {
    let mut matching: Vec<&Edge> = edges
        .iter()
        .filter(|e| &e.to_node == node && &e.to_port == port)
        .collect();
    matching.sort_by_key(|e| e.sort_key());
    matching
}

/// A port on a node (input or output).
///
/// Every port has a cardinality that describes how many values can flow through it.
/// This enables semantic test generation and runtime validation.
///
/// Note: Conditional execution is modeled through explicit Branch patterns and
/// optional types (ZeroOrOne cardinality), not through user-facing guards on ports.
/// The `guard` field is used internally by patterns (Branch, etc.) for routing.
/// See the design doc for the "No Meta-Annotations" principle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Name of the port
    pub name: PortName,
    /// Type of data flowing through this port
    pub type_id: TypeId,
    /// Set-theoretic cardinality (how many values)
    pub cardinality: Cardinality,
    /// Internal routing guard (used by patterns, not public API)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) guard: Option<Guard>,
}

impl Port {
    /// Create a new port.
    /// Defaults to `Cardinality::One` (scalar, required).
    pub fn new(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            cardinality: Cardinality::One,
            guard: None,
        }
    }

    /// Create a port with explicit cardinality.
    pub fn with_cardinality(
        name: impl Into<PortName>,
        type_id: impl Into<TypeId>,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            cardinality,
            guard: None,
        }
    }

    /// Create a scalar port (exactly one value, required).
    /// This is the most common case for simple data flow.
    pub fn scalar(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::with_cardinality(name, type_id, Cardinality::One)
    }

    /// Create an optional port (zero or one value).
    /// Use for nullable or optional data.
    pub fn optional(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::with_cardinality(name, type_id, Cardinality::ZeroOrOne)
    }

    /// Create a list port (zero or more values).
    /// Use for collections that may be empty.
    pub fn list(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::with_cardinality(name, type_id, Cardinality::ZeroOrMore)
    }

    /// Create a non-empty list port (one or more values).
    /// Use for collections that must have at least one element.
    pub fn non_empty_list(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::with_cardinality(name, type_id, Cardinality::OneOrMore)
    }

    /// Create a void port (zero values).
    /// Use for signals that carry no data, just timing.
    pub fn void(name: impl Into<PortName>) -> Self {
        Self::with_cardinality(name, "Unit", Cardinality::Zero)
    }

    /// Create a port with an equality guard (internal use only).
    ///
    /// This is used internally by Branch and other patterns for routing.
    /// Not part of the public API — use explicit Branch patterns instead.
    #[allow(dead_code)]
    pub(crate) fn guarded(
        name: impl Into<PortName>,
        type_id: impl Into<TypeId>,
        expected: Value,
    ) -> Self {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            cardinality: Cardinality::One,
            guard: Some(Guard::Eq(expected)),
        }
    }

    /// Create a port with a guard and explicit cardinality (internal use only).
    ///
    /// This is used internally by Branch and other patterns for routing.
    /// Not part of the public API — use explicit Branch patterns instead.
    pub(crate) fn guarded_with_cardinality(
        name: impl Into<PortName>,
        type_id: impl Into<TypeId>,
        cardinality: Cardinality,
        guard: Guard,
    ) -> Self {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            cardinality,
            guard: Some(guard),
        }
    }

    /// Check if this port has a guard and if the guard passes for the given value.
    ///
    /// Returns `true` if either:
    /// - The port has no guard (always passes)
    /// - The port has a guard and it evaluates to true for the given value
    ///
    /// Returns `false` if the port has a guard that evaluates to false.
    pub fn check_guard(&self, value: &Value) -> bool {
        match &self.guard {
            Some(guard) => guard.evaluate(value),
            None => true,
        }
    }

    /// Check if this port has a guard.
    pub fn has_guard(&self) -> bool {
        self.guard.is_some()
    }

    /// Infer cardinality from the type registry.
    ///
    /// If the port's type is registered in the registry, returns the cardinality
    /// derived from the type DAG structure. Otherwise, returns the port's
    /// declared cardinality.
    ///
    /// This enables type-driven cardinality inference:
    /// - `Optional<T>` types → `ZeroOrOne`
    /// - `List<T>` types → `ZeroOrMore`
    /// - `NonEmptyList<T>` types → `OneOrMore`
    /// - Everything else → `One` (or the declared cardinality)
    pub fn infer_cardinality(&self, registry: &crate::type_registry::TypeRegistry) -> Cardinality {
        // Try to look up the type in the registry and infer cardinality from it
        if let Some(inferred) = registry.infer_cardinality(&self.type_id) {
            inferred
        } else {
            // Fall back to declared cardinality
            self.cardinality
        }
    }
}

/// Guard predicate for conditional routing in patterns (internal use only).
///
/// Guards are used internally by Branch and other patterns to route values
/// based on conditions. They are NOT exposed on Port — conditional execution
/// should use explicit Branch patterns and optional types instead.
///
/// See the design doc "No Meta-Annotations" principle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Guard {
    /// Value must equal expected
    Eq(Value),
    /// Value must not equal expected
    NotEq(Value),
}

impl Guard {
    /// Evaluate the guard against an actual value.
    pub(crate) fn evaluate(&self, actual: &Value) -> bool {
        match self {
            Guard::Eq(expected) => values_equal(actual, expected),
            Guard::NotEq(expected) => !values_equal(actual, expected),
        }
    }
}

/// Compare two values for equality (structural).
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Unit, Value::Unit) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::StrList(a), Value::StrList(b)) => a == b,
        (Value::MapStrStr(a), Value::MapStrStr(b)) => a == b,
        (Value::Json(a), Value::Json(b)) => a == b,
        (Value::Skipped, Value::Skipped) => true,
        _ => false,
    }
}

/// Helper functions for building DAGs.
pub mod build {
    use super::*;

    /// Create a simple port (defaults to Cardinality::One).
    pub fn port(name: &str, type_id: &str) -> Port {
        Port::new(name, type_id)
    }

    /// Create a scalar port (exactly one value).
    pub fn scalar(name: &str, type_id: &str) -> Port {
        Port::scalar(name, type_id)
    }

    /// Create an optional port (zero or one value).
    pub fn optional(name: &str, type_id: &str) -> Port {
        Port::optional(name, type_id)
    }

    /// Create a list port (zero or more values).
    pub fn list(name: &str, type_id: &str) -> Port {
        Port::list(name, type_id)
    }

    /// Create a non-empty list port (one or more values).
    pub fn non_empty_list(name: &str, type_id: &str) -> Port {
        Port::non_empty_list(name, type_id)
    }

    /// Create a void port (zero values, signal only).
    pub fn void(name: &str) -> Port {
        Port::void(name)
    }

    /// Create an edge.
    pub fn edge(from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Edge {
        Edge::new(from_node, from_port, to_node, to_port)
    }

    /// Create an edge with explicit index.
    pub fn edge_indexed(from_node: &str, from_port: &str, to_node: &str, to_port: &str, index: usize) -> Edge {
        Edge::with_index(from_node, from_port, to_node, to_port, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_edge_order_by_node() {
        let edges = vec![
            Edge::with_index("c", "out", "d", "in", 0),
            Edge::with_index("a", "out", "d", "in", 0),
            Edge::with_index("b", "out", "d", "in", 0),
        ];

        let sorted = canonical_edge_order(&edges);
        
        assert_eq!(sorted[0].from_node.0, "a");
        assert_eq!(sorted[1].from_node.0, "b");
        assert_eq!(sorted[2].from_node.0, "c");
    }

    #[test]
    fn test_canonical_edge_order_by_port() {
        let edges = vec![
            Edge::with_index("a", "z_port", "d", "in", 0),
            Edge::with_index("a", "a_port", "d", "in", 0),
            Edge::with_index("a", "m_port", "d", "in", 0),
        ];

        let sorted = canonical_edge_order(&edges);
        
        assert_eq!(sorted[0].from_port.0, "a_port");
        assert_eq!(sorted[1].from_port.0, "m_port");
        assert_eq!(sorted[2].from_port.0, "z_port");
    }

    #[test]
    fn test_canonical_edge_order_by_index() {
        // Same source node/port, different indices
        let edges = vec![
            Edge::with_index("a", "out", "d", "in", 2),
            Edge::with_index("a", "out", "d", "in", 0),
            Edge::with_index("a", "out", "d", "in", 1),
        ];

        let sorted = canonical_edge_order(&edges);
        
        assert_eq!(sorted[0].index, 0);
        assert_eq!(sorted[1].index, 1);
        assert_eq!(sorted[2].index, 2);
    }

    #[test]
    fn test_edges_to_port() {
        let edges = vec![
            Edge::with_index("a", "out", "target", "in", 0),
            Edge::with_index("b", "out", "other", "in", 1),   // Different target
            Edge::with_index("c", "out", "target", "in", 2),
            Edge::with_index("d", "out", "target", "other_port", 3),  // Different port
        ];

        let target_node = NodeId("target".to_string());
        let target_port = PortName("in".to_string());
        
        let matching = edges_to_port(&edges, &target_node, &target_port);
        
        assert_eq!(matching.len(), 2);
        assert_eq!(matching[0].from_node.0, "a");
        assert_eq!(matching[1].from_node.0, "c");
    }

    #[test]
    fn test_edge_sort_key() {
        let edge = Edge::with_index("node", "port", "target", "in", 5);
        let (node, port, index) = edge.sort_key();
        
        assert_eq!(node.0, "node");
        assert_eq!(port.0, "port");
        assert_eq!(index, 5);
    }

    #[test]
    fn test_edge_default_index() {
        let edge = Edge::new("a", "out", "b", "in");
        assert_eq!(edge.index, 0);
    }

    #[test]
    fn test_port_infer_cardinality() {
        use crate::type_lib;
        use crate::type_registry::TypeRegistry;

        let mut registry = TypeRegistry::with_primitives();
        registry.register("OptionalString", type_lib::optional(type_lib::string()));
        registry.register("StringList", type_lib::list(type_lib::string()));
        registry.register("NonEmptyStringList", type_lib::non_empty_list(type_lib::string()));

        // Port with registered type - should infer cardinality
        let port1 = Port::scalar("p1", "String");
        assert_eq!(port1.infer_cardinality(&registry), Cardinality::One);

        let port2 = Port::scalar("p2", "OptionalString");
        assert_eq!(port2.infer_cardinality(&registry), Cardinality::ZeroOrOne);

        let port3 = Port::scalar("p3", "StringList");
        assert_eq!(port3.infer_cardinality(&registry), Cardinality::ZeroOrMore);

        let port4 = Port::scalar("p4", "NonEmptyStringList");
        assert_eq!(port4.infer_cardinality(&registry), Cardinality::OneOrMore);

        // Port with unregistered type - should fall back to declared cardinality
        let port5 = Port::optional("p5", "Unknown");
        assert_eq!(port5.infer_cardinality(&registry), Cardinality::ZeroOrOne);
    }
}
