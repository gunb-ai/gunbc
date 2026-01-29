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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from_node: NodeId,
    pub from_port: PortName,
    pub to_node: NodeId,
    pub to_port: PortName,
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
        }
    }
}

/// A port on a node (input or output).
///
/// Every port has a cardinality that describes how many values can flow through it.
/// This enables semantic test generation and runtime validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Name of the port
    pub name: PortName,
    /// Type of data flowing through this port
    pub type_id: TypeId,
    /// Set-theoretic cardinality (how many values)
    pub cardinality: Cardinality,
    /// Optional guard predicate (for input ports)
    pub guard: Option<Guard>,
}

impl Port {
    /// Create a new port without a guard.
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

    /// Create a port with an equality guard.
    pub fn guarded(name: impl Into<PortName>, type_id: impl Into<TypeId>, expected: Value) -> Self {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            cardinality: Cardinality::One,
            guard: Some(Guard::Eq(expected)),
        }
    }

    /// Create a port with a guard and explicit cardinality.
    pub fn guarded_with_cardinality(
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
}

/// Guard predicate for conditional execution.
///
/// If a guard evaluates to false, the node is skipped and outputs `Skipped`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Guard {
    /// Value must equal expected
    Eq(Value),
    /// Value must not equal expected
    NotEq(Value),
}

impl Guard {
    /// Evaluate the guard against an actual value.
    pub fn evaluate(&self, actual: &Value) -> bool {
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

    /// Create a guarded port with equality check.
    pub fn guarded_port(name: &str, type_id: &str, expected: Value) -> Port {
        Port::guarded(name, type_id, expected)
    }

    /// Create an edge.
    pub fn edge(from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Edge {
        Edge::new(from_node, from_port, to_node, to_port)
    }
}
